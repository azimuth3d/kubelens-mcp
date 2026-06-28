# Architecture Guide: KubeLens-MCP

This document defines the architectural patterns, module boundaries, and design principles for `KubeLens-MCP`. This guide serves as the source of truth for both human developers and AI coding assistants (like Aider) to maintain code quality, decoupling, and high performance.

## 1. System Overview

KubeLens-MCP is a high-performance Model Context Protocol (MCP) server written in Rust. It exposes Kubernetes diagnostic capabilities and observability metrics to LLM clients (e.g., Claude Desktop, Cursor) via the standard MCP specification (JSON-RPC 2.0 over Stdio).


+------------+                 +------------------+                 +-------------------+
|            |     Stdio       |   KubeLens-MCP   |   kube-rs SDK   |    Kubernetes     |
| LLM Client | <-------------> |   (MCP Server)   | <-------------> |   API (kind/GKE)  |
|            |   JSON-RPC 2.0  |   [Rust/Tokio]   |                 +-------------------+
+------------+                 +--------+---------+
|
| HTTP / reqwest
v
+-------------------+
|  Prometheus API   |
+-------------------+


### Core Design Principles
* **Loose Coupling (Hexagonal Architecture):** The core MCP logic and tools must not depend directly on `kube-rs` or any specific HTTP client. They interact strictly via abstract Traits (Ports).
* **Graceful Degradation:** Infrastructure adapters (like `KubeSdkAdapter`) are initialized asynchronously with fallback mechanisms to mock clients if live cluster access fails, ensuring zero-crash stability during startup.
* **High Performance & Low Footprint:** Async-first implementation using `tokio`. Minimize allocations and keep memory footprint minimal for cluster deployments.
* **Security - Least Privilege:** The application operates primarily in Read-Only mode. Write operations are strictly restricted and must explicitly be guarded by feature flags or specific config. All infrastructure interactions enforce strict RBAC boundaries as documented in `README.md`.

---

## 2. Directory Structure

```text
kubelens-mcp/
├── Cargo.toml
├── ARCHITECTURE.md
└── src/
    ├── main.rs            # Application entry point & Stdio/Event loop setup
    ├── mcp/               # MCP Protocol implementation
    │   ├── mod.rs         # Module definition & Router
    │   ├── protocol.rs    # JSON-RPC request/response serialization
    │   └── types.rs       # MCP spec payloads (Tools, Resources, Prompts)
    ├── cluster/           # Infrastructure Layer (Kubernetes & Metrics)
    │   ├── mod.rs         # Module exports and trait re-exports
    │   ├── traits.rs      # Abstraction layer (Ports) - `ClusterDiagnostics`
    │   ├── kube_client.rs # Real implementation using kube-rs (Adapter) - Phase 3
    │   └── mock_client.rs # Mock implementation for testing - Phase 2
    └── tools/             # Business Logic / MCP Tool Handlers
        ├── mod.rs         # Tool registry
        ├── diagnostics.rs # Pod status analyzer & logs logic
        ├── ingress.rs     # APISIX and cert-manager logic
        └── metrics.rs     # Prometheus queries execution
