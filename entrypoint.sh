#!/bin/sh
set -e # Exit immediately if a command exits with a non-zero status.

# The first argument to this script determines the mode.
MODE="$1"

# Default node arguments if none are explicitly provided for the node
DEFAULT_NODE_ARGS="--config-file /app/config.toml"

if [ -z "$MODE" ]; then
  echo "No mode specified. Defaulting to 'node'."
  MODE="node"
  # If defaulting to node and no further args, use default node args
  if [ "$#" -eq 0 ]; then
      set -- "$MODE" $DEFAULT_NODE_ARGS
  fi
fi

case "$MODE" in
  node)
    echo "Starting Ferris Swarm Node..."
    # Remove the 'node' argument itself from the list before passing to the binary
    shift
    # If no arguments were passed after 'node', use default arguments
    if [ "$#" -eq 0 ]; then
        exec ferris_swarm_node $DEFAULT_NODE_ARGS
    else
        exec ferris_swarm_node "$@"
    fi
    ;;
  client)
    echo "Starting Ferris Swarm Client..."
    # Remove the 'client' argument itself from the list
    shift
    exec ferris_swarm_client "$@"
    ;;
  *)
    echo "Error: Unknown mode '$MODE'."
    echo "Usage: docker run <image_name> [node|client] [arguments_for_binary...]"
    echo "  node   [node_arguments...]   - Runs the ferris_swarm_node"
    echo "  client [client_arguments...] - Runs the ferris_swarm_client"
    echo ""
    echo "Example (node default config): docker run <image_name> node"
    echo "Example (node with args):    docker run <image_name> node --node-address 0.0.0.0:50052"
    echo "Example (client):            docker run <image_name> client --input-file /in.mp4 --output-file /out.mkv ..."
    exit 1
    ;;
esac