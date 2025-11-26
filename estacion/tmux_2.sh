#!/bin/bash
tmux new-session -d -s region_2 'cargo run 5'
tmux new-window -t region_2 'cargo run 6'
tmux new-window -t region_2 'cargo run 7'
tmux new-window -t region_2 'cargo run 8'
tmux new-window -t region_2 'cargo run 9'
tmux select-window -t region_2:0
tmux attach-session -t region_2