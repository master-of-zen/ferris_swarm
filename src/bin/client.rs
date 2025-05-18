use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use ferris_swarm::{
    chunk::convert_files_to_chunks,
    client::{
        cli::Cli,
        comms::initialize_node_connections,
        config::load_settings_with_cli_overrides,
        tasks::{process_chunks_on_node_worker, EncodingTaskState},
    },
    ffmpeg::{
        concatenator::concatenate_videos_ffmpeg,
        segmenter::extract_non_video_streams,
        utils::verify_ffmpeg,
    },
    job_config::create_job_temp_config,
    logging::init_logging,
    orchestration::split_video_into_segments, // Only used for type hint, not directly
};
use futures::stream::{FuturesUnordered, StreamExt}; // For managing node worker tasks
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, warn};

#[tokio::main]
#[instrument]
async fn main() -> Result<()> {
    init_logging();
    info!("Ferris Swarm Client: Starting video encoding process...");

    let cli_args = Cli::parse();
    debug!("Parsed CLI arguments: {:?}", cli_args);

    let settings =
        load_settings_with_cli_overrides(&cli_args).context("Failed to load settings")?;
    debug!("Effective settings: {:?}", settings);

    verify_ffmpeg().context("FFmpeg verification failed")?;

    let job_temp_config =
        create_job_temp_config(&settings, &cli_args.input_file, &cli_args.output_file);
    info!("Job temporary directory: {:?}", job_temp_config.base_dir);

    // Determine node addresses and slots (CLI overrides config)
    let node_addresses_to_use = if !cli_args.nodes.is_empty() {
        &cli_args.nodes
    } else {
        &settings.client.node_addresses
    };
    // Slots must be provided if CLI nodes are, or use default/empty from settings.
    // This logic is tricky; for now assume config_loader ensures valid combination
    // or use default. A simple default if not specified:
    let node_slots_to_use = if !cli_args.slots.is_empty() {
        cli_args.slots.clone()
    } else if !node_addresses_to_use.is_empty() {
        // If addresses are present but no slots, default to 1 slot per node
        vec![1; node_addresses_to_use.len()]
    } else {
        Vec::new() // No nodes, no slots
    };

    if node_addresses_to_use.is_empty() {
        warn!("No nodes configured or specified. Encoding will not be distributed.");
        // Potentially could fall back to local-only encoding if implemented, or
        // error out. For now, we'll proceed, and
        // initialize_node_connections will handle it.
    }

    let node_connections = initialize_node_connections(node_addresses_to_use, &node_slots_to_use)
        .await
        .context("Failed to initialize node connections")?;

    if node_connections.is_empty() && !node_addresses_to_use.is_empty() {
        return Err(anyhow::anyhow!(
            "Nodes were specified, but no connections could be established."
        ));
    } else if node_connections.is_empty() {
        info!(
            "No active nodes to distribute work to. Check node configuration if distribution is \
             expected."
        );
        // This implies a local-only workflow or an error state depending on
        // requirements. For this example, if no nodes, it can't proceed with
        // distributed encoding.
        return Err(anyhow::anyhow!(
            "No worker nodes available. Cannot proceed with distributed encoding."
        ));
    }

    info!("Splitting video into segments...");
    let video_segments = split_video_into_segments(
        &cli_args.input_file,
        settings.processing.segment_duration,
        &job_temp_config.segments_dir(), // Output segments to job's temp segment dir
    )
     // if split_video_into_segments is async, otherwise remove .await
    .context("Failed to split video into segments")?;

    info!("Extracting non-video streams...");
    let non_video_streams_path = extract_non_video_streams(
        &cli_args.input_file,
        &job_temp_config.base_dir, // Store in the base job temp dir
    )?;

    let initial_chunks =
        convert_files_to_chunks(video_segments, settings.client.encoder_params.clone())
            .context("Failed to convert video segments to chunks")?;
    let total_chunks_count = initial_chunks.len();
    info!("Created {} chunks from video segments.", total_chunks_count);

    if total_chunks_count == 0 {
        warn!("No chunks were created from the video. Check video duration and segment settings.");
        // Clean up and exit if no work to do
        job_temp_config
            .delete_job_temp_dirs()
            .map_err(|e| warn!("Failed to clean up job temp dirs: {}", e))
            .ok();
        return Ok(());
    }

    let encoding_task_state = Arc::new(Mutex::new(EncodingTaskState::new(initial_chunks)));
    let mut node_worker_handles = FuturesUnordered::new();

    info!(
        "Dispatching encoding tasks to {} connected nodes...",
        node_connections.len()
    );
    for node_conn in node_connections {
        node_worker_handles.push(tokio::spawn(process_chunks_on_node_worker(
            node_conn,
            Arc::clone(&encoding_task_state),
            job_temp_config.encoded_chunks_dir(), // Client saves received encoded chunks here
        )));
    }

    // Wait for all node worker tasks to complete
    while let Some(result) = node_worker_handles.next().await {
        if let Err(e) = result {
            // This is a panic from a spawned task, or a JoinError
            error!("A node worker task failed (joined with error): {}", e);
            // Consider how to handle this: stop all, retry, etc. For now, just
            // log.
        }
    }
    info!("All node workers have completed their processing loops.");

    let final_state = encoding_task_state.lock().await;
    if !final_state.pending_chunks.is_empty() {
        warn!(
            "{} chunks remain in pending state. Encoding may be incomplete.",
            final_state.pending_chunks.len()
        );
        // Log details of pending chunks if necessary for debugging
        for chunk in &final_state.pending_chunks {
            warn!(
                "Pending chunk: index {}, source {:?}",
                chunk.index, chunk.source_path
            );
        }
    }

    let mut successfully_encoded_chunks = final_state.completed_chunks.clone();
    // Important: Sort chunks by index before concatenation
    successfully_encoded_chunks.sort_by_key(|chunk| chunk.index);

    info!(
        "Total chunks processed: {}. Successfully encoded and received: {}.",
        total_chunks_count,
        successfully_encoded_chunks.len()
    );

    if successfully_encoded_chunks.len() != total_chunks_count {
        error!(
            "Encoding incomplete: Expected {} chunks, but only {} were successfully processed.",
            total_chunks_count,
            successfully_encoded_chunks.len()
        );
        // Decide on cleanup or partial result. For now, attempt concatenation with what
        // we have. Or, better, return an error.
        job_temp_config
            .delete_job_temp_dirs()
            .map_err(|e| warn!("Failed to clean up job temp dirs: {}", e))
            .ok();
        return Err(anyhow::anyhow!(
            "Encoding failed: Not all chunks were processed successfully."
        ));
    }

    info!(
        "Concatenating {} encoded chunks...",
        successfully_encoded_chunks.len()
    );
    let encoded_chunk_paths: Vec<PathBuf> = successfully_encoded_chunks
        .into_iter()
        .map(|chunk| chunk.encoded_path.expect("Completed chunk must have an encoded_path"))
        .collect();

    concatenate_videos_ffmpeg(
        encoded_chunk_paths,
        &non_video_streams_path,
        &PathBuf::from(&cli_args.output_file),
        &job_temp_config.base_dir, // For the concat list file
        total_chunks_count,        // Expected number of segments for concatenation
    )?;

    info!(
        "Video encoding completed successfully. Output: {}",
        cli_args.output_file
    );

    job_temp_config
        .delete_job_temp_dirs()
        .map_err(|e| warn!("Failed to clean up job temporary directories: {}", e))
        .ok(); // Log error but don't fail the whole process for cleanup issues

    Ok(())
}
