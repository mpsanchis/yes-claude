# Yes, Claude

This is a toy project that I created to have an excuse to learn a bit more about IPC, websockets, and the Tokio runtime.

Its goal is to do the following:
1. Monitor the current tmux server running in the same machine
2. Find all panes (in all windows, from all sessions) that have Claude Code running in it
3. Allow user to automatically send the Enter key to as many of the panes as they want with a certain frequency, so that the Claude process does not wait for user input and gets a 'yes' for an answer

More documentation can be found in the `docs/` folder.
