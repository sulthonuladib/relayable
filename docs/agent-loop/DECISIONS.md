# DECISIONS — index

Detailed ADRs live under `docs/adr/`. Each ADR contains: context, options, decision, consequences, and what evidence would cause reconsideration (`full.md` §5.2).

| ID | Decision | Status | ADR |
|---|---|---|---|
| DEC-000 | Work state is tracked in-repository (`docs/agent-loop/`), not in chat | **Accepted** | — |
| DEC-001 | Repo is Phase 0 spec-only; establish minimal workspace rather than migrating existing code | **Accepted** | — |
| DEC-002 | Domain model and dependency direction | Pending | `docs/adr/ADR-001` |
| DEC-003 | Protocol serialization + transport | Pending — depends on DOC-001 | `docs/adr/ADR-002` |
| DEC-004 | Terminal engine selection | Pending — spike required | `docs/adr/ADR-003` |
| DEC-005 | Local/remote process ownership and persistence guarantees | Pending | `docs/adr/ADR-004` |
| DEC-006 | Env var namespace: `RELAYABLE_*`, REX aliases, or both | Pending — ADR + tests required (§8.1) | `docs/adr/ADR-005` |

## DEC-000 — Work state in-repository

**Context:** `full.md` §15 requires a durable checkpointed loop; §19.14 requires resumability from files alone.
**Decision:** All phase/status/task/next-step state lives under `docs/agent-loop/`. Chat is not authoritative.
**Consequences:** Every agent must read these files at run start and update them at each checkpoint.

## DEC-001 — Phase 0 empty-repo bootstrap

**Context:** Phase 0 discovery found only `full.md`; no code, manifest, CI, or `AGENTS.md`.
**Decision:** Establish the minimal workspace (`AGENTS.md`, loop docs, Cargo workspace, smoke tests) rather than adapting existing architecture.
**Consequences:** No migration risk; §0.4 "if the repository is empty, establish a minimal workspace" applies.
**Reconsider if:** uncommitted or on-disk work is discovered that the initial inventory missed.

## Open questions requiring evidence (not yet decidable)

- Terminal engine: which maintained Rust-compatible engine meets §5.4 coverage/licensing/API-stability bar.
- Protocol: protobuf vs alternatives, and TLS vs QUIC vs WebSocket-over-TLS for remote (§6.1, §6.2).
- Env namespace: `RELAYABLE_*` vs REX-compatible aliases vs both (§8.1).
- Whether any REX wire protocol is documented enough to attempt interoperability (currently assumed **not**).
