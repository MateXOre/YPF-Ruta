#!/bin/bash
tmux new-session -d -s cluster 'cargo run 1'
tmux split-window -h 'cargo run 2'
tmux split-window -v 'cargo run 3 lider'
tmux select-layout tiled
tmux attach-session -t cluster
