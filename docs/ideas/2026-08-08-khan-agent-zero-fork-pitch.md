# Khan — Agent Zero fork pitch

## Vision
A private, self-hosted agent framework forked from Agent Zero, rebadged as **Khan**, and rebuilt for people who want first-class plugins, serious secret handling, a Perplexity-powered research harness, and Hermes-style memory/skills — all deployable via Docker and Kubernetes.

## Why fork instead of build from scratch?
Agent Zero already gives us: a loop engine, tool system, subordinates, skills, plugins, WebUI, and MCP support. Forking lets us keep that engine while replacing the UX, hardening the architecture, and adding proprietary enterprise layers. MIT license permits private modification and SaaS use; we must preserve the original copyright notice.

## The private-repo path
GitHub does **not** allow a public fork to be made private. The clean path:
1. Rename the existing public `Khan` fork to `Khan-agent-zero-upstream` (or delete it).
2. Create a new **private** repo named `Khan`.
3. Push a mirror of Agent Zero `main` into it.
4. Development proceeds in `Khan` as an independent repo; we can still pull upstream Agent Zero changes manually.

## Differentiation
- **Secret-first**: vault-backed secrets, per-project encryption, zero secrets in git, MCP secret providers.
- **Perplexity Computer harness**: built-in `ask_perplexity` tool with memory-backed result storage.
- **Hermes-inspired memory/skills**: persistent memory, autonomous skill creation, cron jobs, cross-session recall.
- **OpenRouter + direct + local**: unified routing with fallback; direct provider overrides; Ollama/vLLM for offline use.
- **First-class plugins**: SDK, marketplace, sandboxed execution, community index.
- **Kubernetes-native**: Helm chart, Docker images, operator for multi-tenant deployments.

## Roadmap (Mongol-themed, simple names)

| Milestone | Theme | Goal |
|-----------|-------|------|
| **Yurt** | Foundation | Rebrand, re-UI, harden core loop, async execution, observability. |
| **Steppe** | Connectors | JSON/markdown/Notion/Slack/GitHub ingestion; universal connector contract. |
| **Horde** | Plugins | Plugin SDK, sandboxed execution, marketplace, community contributions. |
| **Kurultai** | Knowledge brain | Vector + graph memory, skill auto-generation, memory nudges, cron. |
| **Silk Road** | Cloud & community | Managed cloud beta, Helm/K8s operator, Discord/docs, contributor program. |

## Secret handling
- Secrets live in a vault (HashiCorp Vault, AWS Secrets Manager, or 1Password) and are injected at runtime.
- Per-project secret scopes with encryption at rest.
- Pre-commit hooks + CI scanning block accidental commits.
- MCP servers receive credentials via runtime binding, never via prompt.

## Model routing
- **Default**: OpenRouter for breadth, fallback, and cost controls.
- **Override**: direct Anthropic/OpenAI/Google endpoints for latency or special features.
- **Offline**: Ollama/vLLM/llama.cpp for local, air-gapped, or zero-cost work.

## Free research we can do now
Use Kurultai's existing connectors to ingest these repos with a local embedder (no API cost):
- `agent0ai/agent-zero` — the base engine.
- `NousResearch/hermes-agent` — memory, skills, cron patterns.
- `OpenHands/OpenHands` — coding-agent sandbox patterns.
- `significant-gravitas/autogpt` — autonomous loop ideas.
- `a0-community-plugins` — existing plugin ecosystem.

## Success criteria for Yurt (MVP)
- [ ] Private `Khan` repo mirrored from Agent Zero.
- [ ] Rebranded WebUI with dark/light mode.
- [ ] Core loop hardened with structured errors and tracing.
- [ ] Secret vault plugin working.
- [ ] Perplexity Computer harness tool integrated.
- [ ] Docker image and Helm chart installable locally.
- [ ] `cargo test` / `pytest` green.

## Not doing (yet)
- Public community marketplace (comes after Horde is stable).
- Multi-tenant managed cloud (Silk Road phase).
- Mobile native apps.
- Training custom models.

## Next step
Confirm the private-repo path, then create the milestones and issues in GitHub (or Linear) and start Yurt.