---
id: 01M4D7S5YART7AGWN7RDSRNRM1
scope: "project:ai-memory"
type: decision
title: Adopt Vertical Slice Architecture (VSA) for UMA
tags:
  - vsa
  - architecture
  - rust
  - design
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T07:48:55.370316300+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T07:48:55.370316300+00:00"
since: "2026-10-08T07:48:55.370316300+00:00"
---
### Context
UMA (Unified Memory Architecture) is designed as a standalone, multi-client memory engine for AI coding agents. To ensure high cohesion, low coupling, and easy extensibility across progressive slices (S0 to S9), we need a modular architecture.

### Decision
The project adopts Vertical Slice Architecture (VSA) based on the standard defined in `herdr-plugin-dev`.
- **Composition Root (`src/main.rs`)**: CLI parsing and dispatch only. Zero business logic.
- **Shared Kernel (`uma-core`, `src/shared/`)**: Domain models, storage engine, serialization, and shared helpers. Has no knowledge of slices.
- **Feature Slices (`src/slices/`)**: Each user-visible capability (write, read, list, search, supersede, etc.) is isolated in its own slice end-to-end.
- **Inviolable Slice Boundary**: Feature slices never import each other directly.

### Consequences
- Adding new capabilities means creating a new vertical slice without touching existing slice code.
- File sizes remain strictly bounded under 400 lines (soft target 300).