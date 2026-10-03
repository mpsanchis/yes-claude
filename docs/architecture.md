# Client-Server

The program has a client-server architecture:
- A server (daemon) is always running and waits for connections from potential clients
- At least one client is needed to interact graphically with the server as a user
- Closing all the clients doesn't mean that the server has to stop

# Processes

There might be 2 or 3 processes involved, depending on whether the user decides to use a web UI or a terminal UI.

When 3 processes are involved, the diagram looks like:
```
┌──────────────┐                             ┌──────────────────┐
│              │                             │                  │
│     TUI      │                             │     Browser      │
│              │                             │                  │
└──────▲───────┘                             └────────▲─────────┘
       │                                              │
       │◄───── Unix Socket          WebSocket ───────►│
       │                                              │
       │                                              │
       │                                              │
┌──────▼──────────────────────────────────────────────▼────┐
│                         daemon                           │
│                                                          │
│  Tokio                                                   │
│   ├── tmux monitor                                       │
│   ├── WebSocket server                                   │
│   └── application model                                  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

