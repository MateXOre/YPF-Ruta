#!/bin/bash
tmux new-session -d -s region_1 'cargo run 0'
tmux new-window -t region_1 'cargo run 1'
tmux new-window -t region_1 'cargo run 2'
tmux new-window -t region_1 'cargo run 3'
tmux new-window -t region_1 'cargo run 4'
tmux select-window -t region_1:0
tmux attach-session -t region_1