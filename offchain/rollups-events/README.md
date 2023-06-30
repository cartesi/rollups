# Rollups Events

This crate works as an abstraction layer for producing and consuming Cartesi Rollups events.
Currently, it uses Redis Streams as the event broker and defines the following event streams:

- `rollups-inputs`, for exchanging *Input* events;
- `rollups-outputs`, for exchanging *Output* events;
- `rollups-claims`, for exchanging *Claim* events.
