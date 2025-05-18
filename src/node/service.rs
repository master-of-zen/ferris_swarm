use std::{fs, path::PathBuf};

use tonic::{Request, Response, Status};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    chunk::Chunk,
    protos::video_encoding::{
        video_encoding_service_server::VideoEncodingService,
        EncodeChunkRequest,
        EncodeChunkResponse,
    },
};

/// Implements the gRPC VideoEncodingService for a node.
#[derive(Debug)]
pub struct NodeEncodingService {
    /// Base temporary directory for this node's operations.
    node_temp_dir: PathBuf,
}

impl NodeEncodingService {
    pub fn new(node_temp_dir: PathBuf) -> Self {
        // Ensure the base temp directory for the node exists
        if let Err(e) = fs::create_dir_all(&node_temp_dir) {
            // Panicking here as node cannot function without its temp directory
            panic!(
                "Failed to create node's base temporary directory at {:?}: {}",
                node_temp_dir, e
            );
        }
        info!(
            "NodeEncodingService initialized with temp_dir: {:?}",
            node_temp_dir
        );
        Self {
            node_temp_dir,
        }
    }

    // Helper to get specific subdirectories within the node's temp space
    fn get_received_chunks_dir(&self) -> PathBuf {
        self.node_temp_dir.join("received_chunks")
    }

    fn get_locally_encoded_dir(&self) -> PathBuf {
        self.node_temp_dir.join("locally_encoded")
    }
}

#[tonic::async_trait]
impl VideoEncodingService for NodeEncodingService {
    #[instrument(skip(self, request), fields(chunk_index = request.get_ref().chunk_index))]
    async fn encode_chunk(
        &self,
        request: Request<EncodeChunkRequest>,
    ) -> Result<Response<EncodeChunkResponse>, Status> {
        let req = request.into_inner();
        info!("Received encode request for chunk {}", req.chunk_index);

        let received_chunks_dir = self.get_received_chunks_dir();
        let locally_encoded_dir = self.get_locally_encoded_dir();

        // Ensure subdirectories exist for this specific encoding operation
        fs::create_dir_all(&received_chunks_dir).map_err(|e| {
            error!(
                "Node: Failed to create received_chunks_dir {:?}: {}",
                received_chunks_dir, e
            );
            Status::internal("Node temporary directory error")
        })?;
        fs::create_dir_all(&locally_encoded_dir).map_err(|e| {
            error!(
                "Node: Failed to create locally_encoded_dir {:?}: {}",
                locally_encoded_dir, e
            );
            Status::internal("Node temporary directory error")
        })?;

        let temp_input_path =
            received_chunks_dir.join(format!("chunk_{}_received.mkv", req.chunk_index));
        let temp_output_path =
            locally_encoded_dir.join(format!("chunk_{}_encoded.mkv", req.chunk_index));

        debug!(
            "Writing received chunk {} data to temp file: {:?}",
            req.chunk_index, temp_input_path
        );
        fs::write(&temp_input_path, &req.chunk_data).map_err(|e| {
            error!(
                "Node: Failed to write chunk {} data to temp file {:?}: {}",
                req.chunk_index, temp_input_path, e
            );
            Status::internal("Failed to write received chunk data to file")
        })?;

        let chunk_to_encode = Chunk::new(
            temp_input_path.clone(), // Path to the data just saved by node
            req.chunk_index as usize,
            req.encoder_parameters.clone(), // Use parameters from request
        );

        match chunk_to_encode.encode(temp_output_path.clone()) {
            Ok(encoded_chunk_info) => {
                let final_encoded_path = encoded_chunk_info.encoded_path.ok_or_else(|| {
                    error!(
                        "Node: Encoded chunk {} info is missing encoded_path.",
                        req.chunk_index
                    );
                    Status::internal("Encoding process error: missing output path")
                })?;

                debug!(
                    "Node: Reading encoded chunk {} data from {:?}",
                    req.chunk_index, final_encoded_path
                );
                let encoded_data = fs::read(&final_encoded_path).map_err(|e| {
                    error!(
                        "Node: Failed to read encoded chunk {} from {:?}: {}",
                        req.chunk_index, final_encoded_path, e
                    );
                    Status::internal("Failed to read locally encoded chunk")
                })?;

                info!(
                    "Node: Successfully encoded chunk {}, size of encoded data: {} bytes",
                    req.chunk_index,
                    encoded_data.len()
                );

                // Clean up temporary files on node
                debug!(
                    "Node: Cleaning up temp files for chunk {}: {:?} and {:?}",
                    req.chunk_index, temp_input_path, final_encoded_path
                );
                if let Err(e) = fs::remove_file(&temp_input_path) {
                    warn!(
                        "Node: Failed to remove temp input file {:?}: {}",
                        temp_input_path, e
                    );
                }
                if let Err(e) = fs::remove_file(&final_encoded_path) {
                    warn!(
                        "Node: Failed to remove temp output file {:?}: {}",
                        final_encoded_path, e
                    );
                }

                Ok(Response::new(EncodeChunkResponse {
                    encoded_chunk_data: encoded_data,
                    chunk_index:        req.chunk_index,
                    success:            true,
                    error_message:      String::new(),
                }))
            },
            Err(e) => {
                error!("Node: Failed to encode chunk {}: {}", req.chunk_index, e);
                // Clean up the input file if it exists from the failed attempt
                if temp_input_path.exists() {
                    if let Err(re) = fs::remove_file(&temp_input_path) {
                        warn!(
                            "Node: Failed to remove temp input file {:?} after encoding error: {}",
                            temp_input_path, re
                        );
                    }
                }
                Ok(Response::new(EncodeChunkResponse {
                    encoded_chunk_data: Vec::new(),
                    chunk_index:        req.chunk_index,
                    success:            false,
                    error_message:      e.to_string(), // Send back the error message
                }))
            },
        }
    }
}
