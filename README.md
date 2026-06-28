# KubeLens-MCP

A high-performance Model Context Protocol (MCP) server written in Rust, exposing Kubernetes diagnostic capabilities and observability metrics to LLM clients via JSON-RPC 2.0 over Stdio.

## 🚀 Features
- **Kubernetes Diagnostics**: Real-time pod failure analysis, ingress routing checks, and cert-manager validation.
- **Observability**: Direct Prometheus metrics querying via MCP tools.
- **Security-First**: Read-only cluster access with strict RBAC guidelines.
- **Zero-Crash Stability**: Graceful degradation to mock clients on infrastructure failure.

## 📦 Installation & Setup

### Prerequisites
- Rust 1.75+ (Edition 2024)
- `kind` or access to a Kubernetes cluster
- `kubectl` configured with valid credentials (`~/.kube/config`)
- Prometheus running in-cluster or accessible via `PROMETHEUS_URL` env var

### Building from Source
```bash
cargo build --release
```

## 🤖 LLM Client Integration

### Claude Desktop Integration
1. Build the binary: `cargo build --release`
2. Locate the executable at `target/release/kubelens-mcp`.
3. Open your Claude Desktop configuration file (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS, or `%APPDATA%\Claude\claude_desktop_config.json` on Windows).
4. Add the MCP server configuration:
```json
{
  "mcpServers": {
    "kubelens": {
      "command": "/path/to/target/release/kubelens-mcp",
      "args": []
    }
  }
}
```
5. Restart Claude Desktop. The server will initialize via Stdio and expose diagnostic tools automatically.

### llama.cpp / CLI Integration
For headless or custom LLM environments (e.g., `llama.cpp`, `ollama`):
1. Run the server directly:
```bash
./target/release/kubelens-mcp
```
2. Pipe JSON-RPC requests via stdin and read responses from stdout. The server adheres strictly to MCP JSON-RPC 2.0 specifications, making it compatible with any standard MCP client implementation.

#### example with llama.cpp 

```terminal
./llama-server -m models/qwen2.5-coder.gguf -c 4096 --port 8080
python3 agent_bridge_llama_cpp.py
```

## 🐳 Local Development with `kind`
1. Create a local cluster:
```bash
kind create cluster --name kubelens-dev
```
2. Ensure your kubeconfig points to the new cluster:
```bash
kubectl config use-context kind-kubelens-dev
```
3. Run the server locally (uses mock client by default for safety, or live if configured):
```bash
# Use live cluster client
KUBELENS_USE_MOCK=false ./target/release/kubelens-mcp

# Use mock client (default)
KUBELENS_USE_MOCK=true ./target/release/kubelens-mcp
```

## 🔒 Security & RBAC Guidelines
KubeLens-MCP operates in a **strictly read-only** mode by design. To ensure least-privilege access:

1. **Service Account**: Create a dedicated ServiceAccount for the MCP server.
2. **ClusterRole**: Bind only `get`, `list`, and `watch` permissions to necessary resources (`pods`, `services`, `ingresses`, `certificates`).
3. **RBAC Manifest Example**:
```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: kubelens-mcp-sa
  namespace: default
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: kubelens-mcp-role
rules:
- apiGroups: [""]
  resources: ["pods", "services", "endpoints"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["networking.k8s.io"]
  resources: ["ingresses"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["cert-manager.io"]
  resources: ["certificates"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: kubelens-mcp-binding
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: kubelens-mcp-role
subjects:
- kind: ServiceAccount
  name: kubelens-mcp-sa
  namespace: default
```
4. **Environment Variables**: Never pass sensitive credentials via command line arguments. Use mounted secrets or environment variables (`KUBECONFIG`, `PROMETHEUS_URL`).

## 📜 Architecture & Design
- **Hexagonal Architecture**: Decoupled MCP tools from Kubernetes SDK via `ClusterDiagnostics` trait.
- **Zero-Unwrap Policy**: All fallible operations use safe error propagation, mapping to JSON-RPC `-32603` frames on failure.
- **Graceful Degradation**: Falls back to mock adapters if live cluster initialization fails.

## 📄 License
MIT
