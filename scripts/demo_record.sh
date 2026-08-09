#!/usr/bin/env bash
set -e
cd /root/khtop
tmux kill-session -t demo 2>/dev/null || true
tmux kill-session -t rec 2>/dev/null || true
rm -f /root/khtop/demo.cast

tmux new-session -d -s demo 'khtop'
tmux resize-window -t demo -x 200 -y 50
sleep 8

tmux new-session -d -s rec 'asciinema rec -y /root/khtop/demo.cast -c "tmux attach -t demo"'
tmux resize-window -t rec -x 200 -y 50
sleep 6

# 1. select first run and load its audit tail
tmux send-keys -t demo 'g'
sleep 2
tmux send-keys -t demo Enter
sleep 5

# 2. start a transfer: amount -> simulate -> broadcast
tmux send-keys -t demo 't'
sleep 2
tmux send-keys -t demo '0.0001'
sleep 2
tmux send-keys -t demo Enter
sleep 7
tmux send-keys -t demo Enter
sleep 15

# 3. select the tracked direct run (newest at top) and load its step log with the tx link
tmux send-keys -t demo 'g'
sleep 2
tmux send-keys -t demo Enter
sleep 8

# 4. quit
tmux send-keys -t demo 'q'
sleep 4

wait
echo "recorded: $(ls -la /root/khtop/demo.cast)"
