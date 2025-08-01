use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use ferris_swarm_core::chunk::Chunk;
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};

use super::comms::{send_chunk_for_encoding, NodeConnection};

/// Manages the state of chunks during the encoding process.
#[derive(Debug)]
pub struct EncodingTaskState {
    pub pending_chunks:   Vec<Chunk>, // Chunks waiting to be assigned to a node
    pub completed_chunks: Vec<Chunk>, /* Chunks successfully encoded and saved locally
                                       * Chunks that failed could be moved to a separate list if
                                       * retry logic is complex */
}

impl EncodingTaskState {
    pub fn new(initial_chunks: Vec<Chunk>) -> Self {
        Self {
            pending_chunks:   initial_chunks,
            completed_chunks: Vec::new(),
        }
    }
}

/// Processes chunks on a given node, respecting its concurrency limit
/// (semaphore). This function is typically spawned as a task for each available
/// `NodeConnection`.
#[instrument(skip(node_connection, task_state, client_side_encoded_chunk_dir), fields(node_address = %node_connection.address))]
pub async fn process_chunks_on_node_worker(
    node_connection: NodeConnection,
    task_state: Arc<Mutex<EncodingTaskState>>,
    client_side_encoded_chunk_dir: PathBuf, // Directory to save encoded chunks received from node
) -> Result<()> {
    info!("Worker started for node {}", node_connection.address);
    let mut active_node_tasks = FuturesUnordered::new();

    loop {
        // Try to acquire a permit to send a task to this node
        let permit = match node_connection.semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                // Semaphore full or closed
                if active_node_tasks.is_empty()
                    && node_connection.semaphore.available_permits() == 0
                {
                    // If no active tasks for this node and semaphore is full (or closed),
                    // this worker might be done if no more pending chunks globally.
                    // However, the main loop should handle global pending chunks.
                    // This worker yields if it can't get a permit.
                    tokio::task::yield_now().await;
                    continue;
                }
                // Wait for an active task on this node to complete to free up a slot
                if let Some(task_result) = active_node_tasks.next().await {
                    if let Err(e) = task_result {
                        error!(
                            "A sub-task for encoding on node {} failed: {:?}",
                            node_connection.address, e
                        );
                    }
                }
                // Try to acquire permit again after a task potentially finished
                match node_connection.semaphore.clone().try_acquire_owned() {
                    Ok(p) => p,
                    Err(_) => {
                        tokio::task::yield_now().await; // Yield if still can't get permit
                        continue;
                    },
                }
            },
        };

        let chunk_to_process = {
            let mut state_guard = task_state.lock().await;
            state_guard.pending_chunks.pop() // Get a chunk from the global
                                             // pending list
        };

        match chunk_to_process {
            Some(current_chunk) => {
                info!(
                    "Assigning chunk {} to node {}",
                    current_chunk.index, node_connection.address
                );
                let node_client = node_connection.client.clone();
                let state_clone = Arc::clone(&task_state);
                let dir_clone = client_side_encoded_chunk_dir.clone();
                let node_addr_clone = node_connection.address.clone();

                active_node_tasks.push(tokio::spawn(async move {
                    let result =
                        send_chunk_for_encoding(current_chunk.clone(), node_client, &dir_clone)
                            .await;
                    drop(permit); // Release the semaphore permit for this node

                    let mut state_guard = state_clone.lock().await;
                    match result {
                        Ok(encoded_chunk) => {
                            info!(
                                "Chunk {} successfully processed by node {} and saved locally.",
                                encoded_chunk.index, node_addr_clone,
                            );
                            state_guard.completed_chunks.push(encoded_chunk);
                        },
                        Err(e) => {
                            error!(
                                "Failed to process chunk {} on node {}: {}. Re-adding to pending \
                                 list.",
                                current_chunk.index, node_addr_clone, e
                            );
                            // Simple retry: add back to pending. More sophisticated retry needed
                            // for production.
                            state_guard.pending_chunks.push(current_chunk);
                        },
                    }
                }));
            },
            None => {
                // No more chunks in the global pending list for this worker to pick up
                drop(permit); // Release the permit we might have acquired
                debug!(
                    "No more pending chunks for worker on node {}. Checking active tasks.",
                    node_connection.address
                );
                // Wait for any remaining active tasks on this node to complete
                while let Some(task_result) = active_node_tasks.next().await {
                    if let Err(e) = task_result {
                        error!(
                            "A lingering sub-task for encoding on node {} failed: {:?}",
                            node_connection.address, e
                        );
                    }
                }
                info!(
                    "Worker for node {} finished processing.",
                    node_connection.address
                );
                return Ok(()); // This worker's job is done
            },
        }
    }
}
