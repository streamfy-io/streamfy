# Task 14: Web UI Dashboard

## Status: Not Started
## Priority: Medium | Effort: Large

## Problem

Streamfy has no visual interface. All operations require the CLI or Rust SDK. This means:

- **No at-a-glance cluster visibility.** Operators can't quickly see if something is wrong.
- **No message browsing.** Debugging requires `streamfy consume` with the right flags.
- **No visual topic management.** Creating, configuring, and monitoring topics is all terminal-based.
- **Higher onboarding friction.** New users must learn the CLI before they can do anything.

Competing platforms all have web UIs: Kafka has Conduktor/Kafka UI/Redpanda Console, Pulsar has Pulsar Manager, RabbitMQ has its built-in management UI.

## Goal

Ship a built-in web dashboard (`streamfy-ui`) that covers cluster monitoring, topic management, message browsing, and connector/SmartModule operations.

## Design

### Tech Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Frontend** | React + TypeScript + Tailwind CSS | Industry standard, huge ecosystem, fast iteration |
| **Build** | Vite | Fast dev server, optimized production builds |
| **UI Components** | shadcn/ui | High-quality, composable, no vendor lock-in |
| **Charts** | Recharts or Tremor | Lightweight, React-native charting |
| **State** | TanStack Query (React Query) | Server-state caching, auto-refresh, optimistic updates |
| **Backend API** | Task #8 (HTTP Admin API) or direct MCP-over-SSE | Reuse existing work |
| **Bundling** | Embedded in Rust binary via `rust-embed` | Single binary deployment, no separate web server needed |

### Architecture

Two deployment options:

**Option A: Embedded in SC (recommended for simplicity)**
```
┌─────────────────────────────────────┐
│          Streamfy SC                │
│                                     │
│  :9003  ← Streamfy native protocol │
│  :9080  ← HTTP Admin API (Task #8) │
│  :9090  ← Prometheus metrics (#1)  │
│  :8080  ← Web UI (static files)    │
│          └── served by axum        │
│              via rust-embed         │
└─────────────────────────────────────┘
```

The SC serves the pre-built React app as static files. The frontend calls the HTTP Admin API on the same host. Zero additional services to deploy.

**Option B: Standalone binary**
```
┌──────────────────┐     ┌──────────────────┐
│   streamfy-ui    │────►│   Streamfy SC    │
│   :8080          │     │   :9080 (API)    │
│   (static + proxy)     └──────────────────┘
└──────────────────┘
```

A separate `streamfy-ui` binary that serves the frontend and proxies API calls to the SC. More flexible but one more thing to deploy.

**Recommendation**: Start with Option A (embedded). Extract to Option B later if needed.

### Pages & Features

#### 1. Dashboard (Home)

The landing page — cluster health at a glance.

```
┌─────────────────────────────────────────────────────┐
│  Streamfy Dashboard                          [⚙️]   │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Cluster: healthy ●    SPUs: 3/3 online             │
│  Topics: 12    Partitions: 47    Connectors: 4      │
│                                                     │
│  ┌─ Throughput (last 1h) ─────────────────────────┐ │
│  │  ▁▂▃▅▇█▇▅▃▂▁▂▃▅▇█  Produce: 12.4K msg/s      │ │
│  │  ▁▁▂▃▅▇▇▅▃▂▁▁▂▃▅▇  Consume: 11.8K msg/s      │ │
│  └────────────────────────────────────────────────┘ │
│                                                     │
│  ┌─ Consumer Lag ─────────────────────────────────┐ │
│  │  orders      ████░░░░░░  lag: 234              │ │
│  │  events      ██░░░░░░░░  lag: 89               │ │
│  │  logs        █░░░░░░░░░  lag: 12               │ │
│  └────────────────────────────────────────────────┘ │
│                                                     │
│  Recent Alerts                                      │
│  ⚠ SPU-2 replication lag > 1000 (2 min ago)        │
│  ✓ Connector postgres-cdc restarted (15 min ago)   │
└─────────────────────────────────────────────────────┘
```

**Data sources**: Prometheus metrics (Task #1), SC admin API (Task #8).

#### 2. Topics

List, create, configure, and delete topics.

| Feature | Description |
|---------|-------------|
| Topic list | Table with name, partitions, replication factor, retention, message count, throughput |
| Create topic | Form with name, partition count, replication, retention policy, cleanup policy |
| Topic detail | Partition breakdown, per-partition leader/ISR, lag, config |
| Topic config edit | Modify retention, cleanup policy, max message size inline |
| Delete topic | Confirmation dialog with topic name input |

#### 3. Message Browser

Browse, search, and inspect messages in a topic.

| Feature | Description |
|---------|-------------|
| Live tail | Real-time message stream (WebSocket or SSE) |
| Offset navigation | Jump to offset, timestamp, or "latest - N" |
| Format toggle | Raw bytes, JSON (pretty-printed), Avro (if Schema Registry), string |
| Key/value view | Split view showing key and value separately |
| Filter | Client-side text filter on key, value, or headers |
| Produce message | Inline form to produce a test message (key + value + headers) |
| Partition picker | Filter by specific partition |
| Export | Download visible messages as JSON or CSV |

```
┌─────────────────────────────────────────────────────┐
│  Topic: orders  │ Partition: All ▾ │ Format: JSON ▾ │
├─────────────────────────────────────────────────────┤
│  [Filter: ___________]  [⏮ Oldest] [⏭ Latest] [▶ Live] │
├─────┬──────────┬─────────────────────────────────────┤
│ Off │ Key      │ Value                               │
├─────┼──────────┼─────────────────────────────────────┤
│ 142 │ user-001 │ { "action": "purchase",             │
│     │          │   "amount": 49.99,                  │
│     │          │   "item": "widget-pro" }            │
├─────┼──────────┼─────────────────────────────────────┤
│ 143 │ user-034 │ { "action": "refund",               │
│     │          │   "amount": 12.00 }                 │
├─────┼──────────┼─────────────────────────────────────┤
│ 144 │ user-001 │ { "action": "purchase",             │
│     │          │   "amount": 29.99 }                 │
└─────┴──────────┴─────────────────────────────────────┘
```

#### 4. SPUs (Streaming Processing Units)

| Feature | Description |
|---------|-------------|
| SPU list | Table with ID, status (online/offline), address, rack, assigned partitions |
| SPU detail | Partition assignments, replication status, resource usage |
| Register/unregister | For custom SPU management |

#### 5. SmartModules

| Feature | Description |
|---------|-------------|
| SmartModule list | Name, type (filter/map/aggregate/...), size, upload date |
| Upload | Drag-and-drop WASM upload |
| Test playground | Paste sample input records, select a SmartModule, see output — live in the browser |
| Delete | With confirmation |

#### 6. Connectors

| Feature | Description |
|---------|-------------|
| Connector list | Name, type (source/sink), status (running/stopped/error), topic, throughput |
| Deploy | YAML editor with syntax highlighting + validation |
| Logs | Streaming log viewer per connector |
| Stop/restart | One-click operations |
| Config viewer | Current configuration in YAML |

#### 7. Cluster Settings

| Feature | Description |
|---------|-------------|
| Cluster info | Version, SC address, deployment mode (local/k8s) |
| Profile management | Switch between cluster profiles |
| Config viewer | Current SC/SPU configuration |

### API Layer

The frontend communicates with the backend via:

1. **REST API** (Task #8) for CRUD operations.
2. **SSE/WebSocket** for live tailing and real-time metric updates.
3. **Prometheus query** (Task #1) for historical charts (or the backend pre-aggregates).

If Task #13 (MCP) ships with SSE transport, the Web UI could optionally embed an AI chat panel that uses MCP to interact with the cluster through natural language.

### Project Structure

```
ui/
  package.json
  tsconfig.json
  vite.config.ts
  src/
    main.tsx
    App.tsx
    api/
      client.ts            # API client (fetch wrapper, types)
      hooks.ts             # React Query hooks (useTopics, useSPUs, etc.)
    pages/
      Dashboard.tsx
      Topics.tsx
      TopicDetail.tsx
      MessageBrowser.tsx
      SPUs.tsx
      SmartModules.tsx
      Connectors.tsx
      Settings.tsx
    components/
      Layout.tsx            # Sidebar + header shell
      TopicTable.tsx
      MessageTable.tsx
      MetricsChart.tsx
      ProduceDialog.tsx
      CreateTopicDialog.tsx
      ConnectorLogViewer.tsx
      StatusBadge.tsx
    lib/
      types.ts              # TypeScript types matching API responses
      utils.ts
```

The production build output (`ui/dist/`) is embedded into the Rust binary using `rust-embed`:

```rust
#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct UiAssets;

// In the axum router:
async fn serve_ui(path: Path<String>) -> impl IntoResponse {
    UiAssets::get(&path).map(|f| ([(CONTENT_TYPE, f.metadata.mimetype())], f.data))
}
```

## Phases

### Phase 1: Dashboard + Topics (3 weeks)
- Project scaffolding (React + Vite + Tailwind + shadcn/ui)
- Dashboard page with cluster status, throughput chart, consumer lag
- Topics page: list, create, delete, describe
- Embed in SC binary via `rust-embed`
- `streamfy cluster start` serves UI on `:8080` automatically

### Phase 2: Message Browser (2 weeks)
- Message browser with offset navigation, JSON formatting, key/value split
- Live tail via SSE
- Inline produce form
- Client-side filtering

### Phase 3: SmartModules + Connectors (2 weeks)
- SmartModule list, upload, delete
- SmartModule test playground
- Connector list, deploy (YAML editor), logs, stop/restart

### Phase 4: Polish & AI Chat (1 week)
- SPU management page
- Cluster settings page
- Dark mode / theme toggle
- Optional: embedded AI chat panel using MCP SSE (Task #13)

## Dependencies

- **Task #8 (HTTP Admin API)** — required. The Web UI needs a REST backend.
- **Task #1 (Observability)** — required for dashboard charts and metrics.
- **Task #13 (MCP Server)** — optional. Enables the AI chat panel.

## Build & Dev Workflow

```bash
# Development (hot reload)
cd ui && npm run dev          # Vite dev server on :5173, proxies API to :9080

# Production build
cd ui && npm run build        # Outputs to ui/dist/
cargo build -p streamfy-sc   # Embeds ui/dist/ via rust-embed

# Access
open http://localhost:8080    # After `streamfy cluster start`
```

## Success Criteria

- `streamfy cluster start` serves the Web UI on `:8080` with zero extra setup.
- Dashboard shows live throughput and consumer lag charts.
- Message browser can tail a topic in real-time and display JSON messages prettily.
- Topics can be created, configured, and deleted entirely from the UI.
- SmartModules can be uploaded, tested, and managed from the UI.
- Connectors can be deployed and monitored from the UI.
- Page load < 1s, UI interaction latency < 200ms.
- Works in Chrome, Firefox, Safari (latest versions).
