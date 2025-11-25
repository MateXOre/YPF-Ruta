#!/bin/bash
tmux new-session -d -s cluster_est 'cargo run 0'
tmux split-window -h 'cargo run 1'
tmux split-window -v 'cargo run 2'
tmux select-layout tiled
tmux attach-session -t cluster_est
