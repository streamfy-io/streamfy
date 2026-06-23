# Task 13: MCP Server (Model Context Protocol)

## Status: Not Started
## Priority: High | Effort: Medium

## Why This Matters

[MCP (Model Context Protocol)](https://modelcontextprotocol.io/) is the open standard for connecting AI assistants and LLMs to external tools and data sources. It's supported by Claude, Cursor, Windsurf, VS Code Copilot, and a growing ecosystem. Shipping an MCP server for Streamfy means:

- **AI agents can operate Streamfy** — create topics, produce/consume, deploy connectors, inspect cluster state — all through natural language.
- **LLM-powered data pipelines** — an agent can read from one topic, transform data via prompt, write to another.
- **Ops automation** — "show me topics with consumer lag > 1000" or "create a topic with 6 partitions and 7-day retention" without memorizing CLI flags.
- **Developer experience** — engineers interact with Streamfy from their AI-assisted IDE instead of switching to a terminal.

This is a differentiation opportunity. No major streaming platform (Kafka, Pulsar, Redpanda) ships a first-party MCP server today.

## Goal

Ship a `streamfy-mcp-server` that exposes Streamfy operations as MCP tools and resources, runnable via `stdio` transport (for IDE integration) and `SSE` transport (for remote/web use).

## Design

### Tools (Actions)

MCP tools are functions the AI can invoke. The Streamfy MCP server exposes:

**Cluster Management**
| Tool | Description | Parameters |
|------|-------------|------------|
| `cluster_status` | Get cluster health and SPU status | — |
| `cluster_start` | Start a local cluster | `spu_count`, `log_dir` |
| `cluster_shutdown` | Shut down the cluster | `force` |

**Topic Operations**
| Tool | Description | Parameters |
|------|-------------|------------|
| `topic_create` | Create a new topic | `name`, `partitions`, `replication`, `retention_time`, `cleanup_policy` |
| `topic_delete` | Delete a topic | `name` |
| `topic_list` | List all topics with metadata | `filter` |
| `topic_describe` | Show topic details, partitions, config | `name` |
| `topic_add_partition` | Add partitions to a topic | `name`, `count` |

**Produce & Consume**
| Tool | Description | Parameters |
|------|-------------|------------|
| `produce` | Produce records to a topic | `topic`, `records[]` (key/value pairs), `partition` |
| `produce_file` | Produce contents of a file to a topic | `topic`, `file_path`, `key` |
| `consume` | Consume N records from a topic | `topic`, `partition`, `offset` (start/end/N), `count`, `format` |
| `consume_tail` | Get the last N records | `topic`, `count` |

**SmartModules**
| Tool | Description | Parameters |
|------|-------------|------------|
| `smartmodule_list` | List installed SmartModules | — |
| `smartmodule_create` | Upload a SmartModule WASM binary | `name`, `wasm_path` |
| `smartmodule_delete` | Delete a SmartModule | `name` |
| `smartmodule_test` | Test a SmartModule against sample data | `name`, `input_records[]` |

**Connectors**
| Tool | Description | Parameters |
|------|-------------|------------|
| `connector_list` | List running connectors | — |
| `connector_deploy` | Deploy a connector from config | `config_yaml` |
| `connector_delete` | Stop and remove a connector | `name` |
| `connector_logs` | Get recent connector logs | `name`, `lines` |

**Diagnostics**
| Tool | Description | Parameters |
|------|-------------|------------|
| `consumer_lag` | Show consumer lag per partition | `topic`, `consumer_id` |
| `partition_status` | Show partition leader/follower/ISR details | `topic` |
| `spu_list` | List SPUs with status and load | — |

### Resources (Read-only Data)

MCP resources are data the AI can read for context:

| Resource URI | Description |
|---|---|
| `streamfy://cluster/status` | Cluster health overview (JSON) |
| `streamfy://topics` | All topics with partition counts and configs |
| `streamfy://topics/{name}` | Single topic detail |
| `streamfy://topics/{name}/messages?offset=-10` | Last 10 messages from a topic |
| `streamfy://smartmodules` | Installed SmartModules |
| `streamfy://connectors` | Running connectors and their status |
| `streamfy://metrics` | Current throughput, lag, error rates |

### Prompts (Pre-built Workflows)

| Prompt | Description |
|---|---|
| `debug_topic` | Inspect a topic: show config, partitions, lag, recent messages, and diagnose issues |
| `setup_pipeline` | Guide the user through creating a source → transform → sink pipeline |
| `migrate_from_kafka` | Help plan a migration from Kafka to Streamfy |

### Transport

- **stdio**: Default. The MCP server runs as a child process of the AI client (Claude Desktop, Cursor, etc.). Communication over stdin/stdout with JSON-RPC.
- **SSE (Server-Sent Events)**: For remote access. Runs as an HTTP server. Enables web-based AI assistants to connect.

### Architecture

```
┌──────────────────────┐
│  AI Client (Claude,  │
│  Cursor, VS Code)    │
│                      │
│  stdio / SSE ────────┤
└──────────┬───────────┘
           │ JSON-RPC (MCP protocol)
┌──────────▼───────────┐
│  streamfy-mcp-server │
│                      │
│  - Tool handlers     │
│  - Resource providers│
│  - Streamfy SDK      │
│    (TopicProducer,   │
│     PartitionConsumer│
│     StreamfyAdmin)   │
└──────────┬───────────┘
           │ Native Streamfy protocol
┌──────────▼───────────┐
│   Streamfy Cluster   │
│   (SC + SPUs)        │
└──────────────────────┘
```

## Implementation

### Crate Structure

```
crates/
  streamfy-mcp/
    Cargo.toml
    src/
      main.rs             # CLI entry point: streamfy-mcp --transport stdio|sse
      server.rs           # MCP server setup, tool/resource registration
      tools/
        mod.rs
        cluster.rs        # cluster_status, cluster_start, cluster_shutdown
        topics.rs         # topic_create, topic_delete, topic_list, topic_describe
        produce.rs        # produce, produce_file
        consume.rs        # consume, consume_tail
        smartmodules.rs   # smartmodule_list, smartmodule_create, smartmodule_test
        connectors.rs     # connector_deploy, connector_delete, connector_logs
        diagnostics.rs    # consumer_lag, partition_status, spu_list
      resources/
        mod.rs
        cluster.rs
        topics.rs
        messages.rs
      prompts/
        mod.rs
        debug_topic.rs
        setup_pipeline.rs
```

### Dependencies

- `rmcp` — Rust MCP SDK (handles JSON-RPC, stdio/SSE transport, tool schema generation)
- `streamfy` — the Streamfy client SDK (already in-workspace)
- `streamfy-cluster` — for cluster start/shutdown operations
- `serde_json` — for JSON serialization of tool inputs/outputs
- `clap` — for CLI argument parsing
- `tokio` — async runtime

### Installation & Usage

```bash
# Build
cargo build -p streamfy-mcp

# Run with stdio (for Claude Desktop / Cursor)
streamfy-mcp --transport stdio

# Run with SSE (for remote/web access)
streamfy-mcp --transport sse --port 3001

# Claude Desktop config (~/.claude/claude_desktop_config.json)
{
  "mcpServers": {
    "streamfy": {
      "command": "streamfy-mcp",
      "args": ["--transport", "stdio"]
    }
  }
}
```

## Phases

### Phase 1: Core Tools (2 weeks)
- `cluster_status`, `topic_list`, `topic_create`, `topic_delete`, `topic_describe`
- `produce`, `consume`, `consume_tail`
- stdio transport only
- Ship as `streamfy-mcp` binary

### Phase 2: SmartModules & Connectors (1 week)
- `smartmodule_list`, `smartmodule_create`, `smartmodule_test`
- `connector_deploy`, `connector_delete`, `connector_logs`
- Resources: `streamfy://topics`, `streamfy://cluster/status`

### Phase 3: SSE Transport & Prompts (1 week)
- SSE transport for remote/web access
- Pre-built prompts: `debug_topic`, `setup_pipeline`
- `consumer_lag`, `partition_status` diagnostics

### Phase 4: Sampling & Notifications (1 week)
- MCP sampling: let the server ask the AI to analyze data patterns
- Notifications: push alerts on consumer lag spikes, connector failures
- Resource subscriptions: live-updating topic message streams

## Success Criteria

- `streamfy-mcp` runs as a Claude Desktop MCP server; user can say "create a topic called orders with 6 partitions" and it works.
- `streamfy-mcp` runs as an SSE server that a web UI (Task #14) can connect to.
- All tools have proper JSON Schema input validation and descriptive error messages.
- Produce/consume works end-to-end through natural language.
- Response times < 500ms for all metadata operations, < 2s for produce/consume.
