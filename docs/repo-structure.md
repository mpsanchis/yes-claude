# Repo structure

This repo is a multi-module cargo-managed repository. It contains the modules (prepended by yes-claude, as namespacing):
- daemon
- tmux-monitor
- tmux-model
- tui
- types

# Executables

The only two executable modules are:
- daemon
- tui

The rest are libraries that expose utilities to be used by them.

# Dependencies

- the `daemon` depends on `tmux-monitor` and `tmux-model` as the tooling to observe the state of tmux and update an internal model
- any package can depend on `types` that are shared
