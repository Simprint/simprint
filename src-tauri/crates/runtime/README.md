# Embedded environment runtime

This crate owns browser process supervision, the browser EventBus, environment status, CDP
endpoints, authentication propagation, and synchronized-input routing.

It intentionally has no binary target. The Tauri application creates `RuntimeHost` in-process and
forwards runtime events to the existing frontend event names. Chrome remains a separate supervised
process and communicates with this crate through the per-environment EventBus transport.

A browser launch is successful only after the browser process connects and completes the EventBus
handshake. Process exit, transport failure, or the startup deadline transitions the environment to
`error` and emits `environment.launch_failed`.
