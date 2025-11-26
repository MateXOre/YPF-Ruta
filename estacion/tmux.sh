#!/bin/bash
tmux new-session -d -s region_1 'cargo run 0'
tmux split-window -v 'cargo run 1'
tmux split-window -v 'cargo run 2'
tmux split-window -v 'cargo run 3'
tmux split-window -v 'cargo run 4'
tmux select-layout tiled
tmux attach-session -t region_1
