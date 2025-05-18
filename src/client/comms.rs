use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use tokio::sync::Semaphore;
use tonic::transport::Channel;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    chunk::Chunk,
    protos::video_encoding::{
        video_encoding_service_client::VideoEncodingServiceClient,
        EncodeChunkRequest,
    },
};

const MAX_MESSAGE_SIZE_BYTES: usize = 1 * 1024 * 1024 * 1024; // 1 GB

#[derive(Clone)]
pub struct NodeConnection {
    pub client:    VideoEncodingServiceClient<Channel>,
    pub address:   String,
    pub semaphore: Arc<Semaphore>, // Controls concurrent tasks for this specific node
}

#[instrument(skip(node_addresses, node_slots))]
pub async fn initialize_node_connections(
    node_addresses: &[String],
    node_slots: &[usize], // Number of concurrent tasks per node
) -> Result<Vec<NodeConnection>> {
    info!(
        "Initializing connections to {} nodes...",
        node_addresses.len()
    );
    if node_addresses.len() != node_slots.len() {
        return Err(anyhow::anyhow!(
            "Mismatch between number of node addresses ({}) and slot counts ({}).",
            node_addresses.len(),
            node_slots.len()
        ));
    }

    let mut connections = Vec::new();
    for (i, address_str) in node_addresses.iter().enumerate() {
        let slots = node_slots[i];
        if slots == 0 {
            return Err(anyhow::anyhow!(
                "Node {} ({}) must have at least 1 slot.",
                i,
                address_str
            ));
        }
        debug!(
            "Attempting to connect to node {} at {} with {} slots",
            i, address_str, slots
        );

        let channel = Channel::from_shared(address_str.clone())
            .map_err(|e| anyhow::anyhow!("Invalid node address format '{}': {}", address_str, e))?
            .connect_timeout(std::time::Duration::from_secs(10)) // Add connect timeout
            .connect()
            .await
            .with_context(|| format!("Failed to connect to node at {}", address_str))?;

        let client = VideoEncodingServiceClient::new(channel)
            .max_decoding_message_size(MAX_MESSAGE_SIZE_BYTES)
            .max_encoding_message_size(MAX_MESSAGE_SIZE_BYTES);

        connections.push(NodeConnection {
            client,
            address: address_str.clone(),
            semaphore: Arc::new(Semaphore::new(slots)),
        });
        info!(
            "Successfully connected to node {} at {} with {} slots",
            i, address_str, slots
        );
    }

    if connections.is_empty() {
        // This case might depend on whether having no nodes is an error or a valid
        // scenario For distributed encoding, it's likely an error if nodes were
        // expected.
        warn!("No node connections were initialized. Check configuration and node availability.");
        // return Err(anyhow::anyhow!("No nodes available for processing."));
    }

    Ok(connections)
}

#[instrument(skip(chunk, client, client_side_encoded_chunk_dir), fields(chunk_index = chunk.index, ))]
pub async fn send_chunk_for_encoding(
    chunk: Chunk, // The chunk to be sent (contains source_path on client)
    mut client: VideoEncodingServiceClient<Channel>, // Tonic client for a specific node
    client_side_encoded_chunk_dir: &Path, // Dir on client to save the received encoded data
) -> Result<Chunk> {
    // Returns a new Chunk with encoded_path set on client
    debug!("Preparing to send chunk {} for encoding.", chunk.index);

    let chunk_source_data = tokio::fs::read(&chunk.source_path) // Use tokio::fs for async read
        .await
        .with_context(|| {
            format!(
                "Failed to read chunk source data from {:?}",
                chunk.source_path
            )
        })?;

    let request = tonic::Request::new(EncodeChunkRequest {
        chunk_data:         chunk_source_data,
        chunk_index:        chunk.index as i32,
        encoder_parameters: chunk.encoder_parameters.clone(),
    });

    debug!("Sending EncodeChunkRequest for chunk {}...", chunk.index);
    let response = client
        .encode_chunk(request)
        .await
        .with_context(|| format!("gRPC call to encode_chunk {} failed", chunk.index))?
        .into_inner();

    if response.success {
        info!(
            "Chunk {} successfully encoded by node. Received {} bytes.",
            chunk.index,
            response.encoded_chunk_data.len()
        );

        let client_side_encoded_path =
            client_side_encoded_chunk_dir.join(format!("encoded_chunk_{}.mkv", chunk.index)); // Standardized name

        tokio::fs::write(&client_side_encoded_path, response.encoded_chunk_data) // Use tokio::fs
            .await
            .with_context(|| {
                format!(
                    "Failed to write received encoded chunk data to {:?}",
                    client_side_encoded_path
                )
            })?;
        debug!(
            "Saved encoded chunk {} to {:?}",
            chunk.index, client_side_encoded_path
        );

        Ok(Chunk {
            source_path:        chunk.source_path, // Original source path (segment)
            encoded_path:       Some(client_side_encoded_path), // Path to the file saved on client
            index:              chunk.index,
            encoder_parameters: chunk.encoder_parameters,
        })
    } else {
        error!(
            "Node failed to encode chunk {}: {}",
            chunk.index, response.error_message
        );
        Err(anyhow::anyhow!(
            "Node reported failure for chunk {}: {}",
            chunk.index,
            response.error_message
        ))
    }
}
