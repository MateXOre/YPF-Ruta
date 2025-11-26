#!/bin/bash
tmux new-session -d -s region_2 'cargo run 5'
tmux split-window -v 'cargo run 6'
tmux split-window -v 'cargo run 7'
tmux split-window -v 'cargo run 8'
tmux split-window -v 'cargo run 9'
tmux select-layout tiled
tmux attach-session -t region_2
