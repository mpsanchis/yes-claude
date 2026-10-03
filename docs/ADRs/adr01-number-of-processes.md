# Number of tasks

The system designed clearly has different tasks that are independent in their scope:
- tmux pane model
  - Listens for events from a channel
  - Updates the model based on events received
  - When model changes, broadcasts information to any interested listener
- Web server (acts as a router)
  - Listens for connections
  - Establishes connections/sessions
  - Serves a web UI over HTTP
  - Has one websocket open to communicate with clients:
    - Received messages from browsers are forwarded to the tmux pane model
    - Received messages from the tmux pane model are forwarded to the browsers
- Tmux monitor
  - Periodically polls tmux, gets its data, and forwards it to the tmux pane model
- TUI
  - Listens for keyboard events
  - Sends state update request events to model based on keyboard events
  - Updates an internal state (terminal representation) based on events received by the model
  - Paints the terminal when internal state changes

Given their diverse nature, these "tasks" are a very good fit for a multi-threaded or multi-process software design:
instead of having a single loop checking for everything (poll tmux, check for keyboard inputs, check for HTTP or
websocket messages, etc.), we can design separate loops for each task.

# Number of processes

All of this seems to fit perfectly in a single Tokio runtime managing different Futures. However, given that this is
a learning project, I have decided to do it in 2 main processes:
1. **daemon**: contains the Model, web server, and tmux monitor
2. **tui**: single entrypoint for user, that represents the model the daemon communicates, and requests UI changes

and a potential 3rd process:
3. **web UI (browser)**: an alternative visualization of the model
