# Kurultai Year 1 Milestones & Work Orders

**Timeline:** July 2026 → July 2027  
**Target Revenue:** $50-100K first year  
**Strategy:** Any cashflow is good — start with team tier, layer in enterprise

**Version note:** Product **v0.4.0** already shipped as Brain UI (solar · pulse · purple · max). This Year-1 cashflow track starts team pricing at **v0.5.0** (not v0.4.0).


---

## Q3 2026 (Aug–Sep) — Foundation Hardening

### Milestone: v0.3.1 Foundation
**Target:** September 30, 2026  
**Cashflow:** None (stability release)  
**GitHub Issue:** Create tracking issue

#### Work Orders

- **[WO-101]** feat: Built-in metrics + error tracking (GlitchTip-like, self-hosted) [#102]
  - Prometheus metrics export
  - Error tracking infrastructure  
  - Health check endpoints
  - Grafana dashboard templates
  - **Estimated:** 2 weeks
  - **Dependencies:** None
  - **Cashflow Impact:** None (infra)

- **[WO-102]** feat(mcp): HTTP/SSE MCP transport for Perplexity and remote agent integration [#104]
  - SSE server implementation
  - Remote agent authentication
  - Connection pooling
  - Client libraries
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-101 (metrics)
  - **Cashflow Impact:** Enables remote agent partnerships

- **[WO-103]** feat: Cloud-hosted Brain UI — GitHub login → tunnel to local daemon [#101]
  - GitHub OAuth integration
  - WebSocket tunneling
  - Cloud UI deployment
  - Local daemon proxy
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-102 (SSE transport)
  - **Cashflow Impact:** Foundation for hosted tier

- **[WO-104]** fix: Post-v0.3.0 stabilization
  - UI edge case fixes
  - Connector race conditions
  - Memory leak fixes
  - Performance tuning
  - **Estimated:** 2 weeks
  - **Dependencies:** None (parallel)
  - **Cashflow Impact:** None (stability)

**Total:** 10 weeks (2.5 months)  
**Team:** 1 engineer  
**Release:** v0.3.1 to crates.io + GitHub Release

---

## Q4 2026 (Oct–Dec) — Team Foundations

### Milestone: v0.5.0 Team
**Target:** December 15, 2026  
**Cashflow:** Early team access ($5-10/user/mo)  
**GitHub Issue:** Create tracking issue

#### Work Orders

- **[WO-201]** feat: Multi-tenant Postgres backend
  - Postgres + pgvector schema
  - Tenant isolation
  - Migration tools (SQLite → Postgres)
  - Connection pooling
  - **Estimated:** 4 weeks
  - **Dependencies:** None
  - **Cashflow Impact:** Enables team tier pricing

- **[WO-202]** feat: RBAC + audit logging
  - Role-based permissions
  - User management API
  - Audit log for all access
  - Admin UI
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-201 (multi-tenant)
  - **Cashflow Impact:** Enterprise requirement

- **[WO-203]** feat: Clerk Organization enforcement
  - Org → store binding
  - Team membership sync
  - Auth → store access control
  - Multi-tenant middleware
  - **Estimated:** 2 weeks
  - **Dependencies:** WO-201 (multi-tenant)
  - **Cashflow Impact:** Team onboarding UX

- **[WO-204]** feat: Promote-to-shared-index workflow
  - Personal → shared atom promotion
  - Approval workflow
  - Merge conflict resolution
  - API endpoints
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-202 (RBAC)
  - **Cashflow Impact:** Team collaboration value

**Total:** 12 weeks (3 months)  
**Team:** 1-2 engineers (scale for complexity)  
**Release:** v0.5.0 + team tier pricing live

---

### Milestone: v0.5.1 Enterprise Connectors
**Target:** January 31, 2027  
**Cashflow:** Enterprise connector licenses  
**GitHub Issue:** [#12] (Phase 2 tracking)

#### Work Orders

- **[WO-301]** feat: Slack webhook connector (real-time)
  - Slack webhook endpoint
  - Signature verification
  - Message parsing
  - Real-time ingestion
  - **Estimated:** 3 weeks
  - **Dependencies:** None
  - **Cashflow Impact:** Enterprise demand driver

- **[WO-302]** feat: Notion connector (database sync)
  - Notion API integration
  - Incremental sync
  - Rate limit handling
  - Rich text parsing
  - **Estimated:** 3 weeks
  - **Dependencies:** None (parallel to WO-301)
  - **Cashflow Impact:** Migration tool revenue

- **[WO-303]** feat: LLM distillation pipeline
  - Summarizer (200 char max)
  - Entity extractor
  - Question extraction
  - Tagging system
  - **Estimated:** 4 weeks
  - **Dependencies:** WO-301 + WO-302 (data sources)
  - **Cashflow Impact:** Quality differentiator

- **[WO-304]** feat: Multi-tier storage (Redis + Postgres)
  - Redis hot data (30 days)
  - Postgres warm data (6 months)
  - Automatic tier migration
  - Unified query interface
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-201 (Postgres backend)
  - **Cashflow Impact:** Performance tier pricing

**Total:** 13 weeks (3.25 months)  
**Team:** 1-2 engineers  
**Release:** v0.5.1 + enterprise connector marketplace

---

## Q1 2027 (Feb–Apr) — Scale & Ecosystem

### Milestone: v0.6.0 Scale
**Target:** March 31, 2027  
**Cashflow:** Scale tier pricing  
**GitHub Issue:** [#13] (Phase 3 tracking)

#### Work Orders

- **[WO-401]** feat: S3 cold storage tier
  - S3/MinIO integration
  - Parquet format
  - Lazy loading
  - Lifecycle policies
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-304 (multi-tier storage)
  - **Cashflow Impact:** Storage cost reduction

- **[WO-402]** feat: Advanced caching
  - L1 in-memory LRU
  - L2 Redis cache
  - L3 materialized views
  - Cache warming
  - **Estimated:** 4 weeks
  - **Dependencies:** WO-304 (Redis integration)
  - **Cashflow Impact:** Performance tier value

- **[WO-403]** feat: Query optimization engine
  - Smart query routing
  - Cost estimation
  - Automatic fallback
  - Performance tuning
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-402 (caching)
  - **Cashflow Impact:** SLA guarantees

- **[WO-404]** feat: Monitoring + observability
  - OpenTelemetry integration
  - Query latency histograms
  - Storage tier statistics
  - Error rate tracking
  - **Estimated:** 2 weeks
  - **Dependencies:** WO-101 (metrics foundation)
  - **Cashflow Impact:** Enterprise trust

**Total:** 12 weeks (3 months)  
**Team:** 1-2 engineers  
**Release:** v0.6.0 + scale tier pricing

---

### Milestone: v0.6.1 Plugins
**Target:** April 30, 2027  
**Cashflow:** Plugin marketplace fees  
**GitHub Issue:** [#14] (Phase 4 tracking)

#### Work Orders

- **[WO-501]** feat: Plugin system (Python + WASM)
  - Plugin API design
  - Python runtime embedding
  - WASM runtime (Wasmer)
  - Plugin lifecycle
  - **Estimated:** 4 weeks
  - **Dependencies:** None
  - **Cashflow Impact:** Ecosystem growth

- **[WO-502]** feat: Plugin discovery
  - `~/.kurultai/plugins/` scanning
  - Plugin validation
  - Version management
  - Dependency resolution
  - **Estimated:** 2 weeks
  - **Dependencies:** WO-501 (plugin API)
  - **Cashflow Impact:** Developer experience

- **[WO-503]** feat: Plugin permissions
  - Filesystem permissions
  - Network permissions
  - Sandboxing
  - Security model
  - **Estimated:** 2 weeks
  - **Dependencies:** WO-501 (plugin API)
  - **Cashflow Impact:** Enterprise security

- **[WO-504]** docs: Plugin development guide
  - Tutorial documentation
  - Example plugins
  - API reference
  - Best practices
  - **Estimated:** 2 weeks
  - **Dependencies:** WO-501 + WO-502 + WO-503
  - **Cashflow Impact:** Community adoption

**Total:** 10 weeks (2.5 months)  
**Team:** 1 engineer  
**Release:** v0.6.1 + plugin marketplace MVP

---

## Q2 2027 (May–Jul) — Launch

### Milestone: v0.7.0 Enterprise
**Target:** June 15, 2027  
**Cashflow:** Enterprise contracts ($20-50/user/mo)  
**GitHub Issue:** [#15] (Phase 5 tracking)

#### Work Orders

- **[WO-601]** feat: SSO (SAML/OIDC)
  - SAML integration
  - OIDC integration
  - Provider configuration
  - User provisioning
  - **Estimated:** 4 weeks
  - **Dependencies:** WO-203 (Clerk integration)
  - **Cashflow Impact:** Enterprise requirement

- **[WO-602]** feat: VPC deployment + K8s
  - Helm charts
  - K8s deployment guides
  - VPC networking
  - Secret management
  - **Estimated:** 3 weeks
  - **Dependencies:** None
  - **Cashflow Impact:** Self-hosted enterprise

- **[WO-603]** feat: Security compliance
  - SOC2 Type II prep
  - GDPR compliance
  - Penetration testing
  - Security audit logs
  - **Estimated:** 4 weeks
  - **Dependencies:** WO-202 (audit logging)
  - **Cashflow Impact:** Enterprise trust

- **[WO-604]** feat: Migration tools
  - Notion → Kurultai importer
  - Confluence → Kurultai importer
  - CSV/JSON importer
  - Migration validation
  - **Estimated:** 3 weeks
  - **Dependencies:** WO-302 (Notion connector)
  - **Cashflow Impact:** Migration services revenue

**Total:** 14 weeks (3.5 months)  
**Team:** 1-2 engineers + security consultant  
**Release:** v0.7.0 + enterprise tier live

---

### Milestone: v1.0.0 Launch
**Target:** July 31, 2027  
**Cashflow:** General availability + all tiers  
**GitHub Issue:** [#10] (Phase 6 tracking)

#### Work Orders

- **[WO-701]** feat: Website / docs site
  - Landing page
  - Architecture diagrams
  - Quickstart guide
  - Full documentation
  - **Estimated:** 3 weeks
  - **Dependencies:** None
  - **Cashflow Impact:** Conversion optimization

- **[WO-702]** feat: Release packaging
  - Crates.io publish
  - Homebrew tap
  - Docker images
  - Prebuilt binaries
  - **Estimated:** 2 weeks
  - **Dependencies:** None
  - **Cashflow Impact:** Distribution reach

- **[WO-703]** feat: CI/CD pipeline
  - GitHub Actions setup
  - Test matrix (Linux, macOS, Windows)
  - Automated releases
  - Rollback mechanism
  - **Estimated:** 2 weeks
  - **Dependencies:** WO-702 (release packaging)
  - **Cashflow Impact:** Engineering efficiency

- **[WO-704]** feat: Community contribution guide
  - CONTRIBUTING.md
  - Issue templates
  - PR templates
  - Code of conduct
  - **Estimated:** 1 week
  - **Dependencies:** None
  - **Cashflow Impact:** Community growth

- **[WO-705]** feat: Launch content
  - Show HN post
  - Blog posts
  - Demo videos
  - Press outreach
  - **Estimated:** 2 weeks
  - **Dependencies:** WO-701 (website)
  - **Cashflow Impact:** User acquisition

**Total:** 10 weeks (2.5 months)  
**Team:** 1 engineer + content creator  
**Release:** v1.0.0 + general availability

---

## Cashflow Timeline

| Month | Milestone | Cashflow Source | Est. Users | Est. MRR |
|-------|-----------|-----------------|------------|----------|
| Aug-Sep 2026 | v0.3.1 | None | 0 | $0 |
| Oct-Dec 2026 | v0.5.0/v0.5.1 | Early team access | 50-200 | $1-3K |
| Jan-Mar 2027 | v0.6.0/v0.6.1 | Scale tier + plugins | 200-500 | $3-8K |
| Apr-Jun 2027 | v0.7.0 | Enterprise contracts | 500-1K | $8-15K |
| Jul 2027 | v1.0.0 | General availability | 1K-2K | $15-25K |

**Year 1 Total Revenue Estimate:** $50-100K  
**Cashflow Philosophy:** Any revenue is good — start small, layer in tiers

---

## GitHub Issues to Create

Use this template for each milestone tracking issue:

```markdown
## Milestone: v0.X.X [Name]

**Target:** [Date]
**Cashflow:** [Source]
**Work Orders:** [WO-XXX, WO-YYY, ...]

### Progress
- [ ] WO-XXX: [Title]
- [ ] WO-YYY: [Title]

### Dependencies
- Blocks: [Issue #]
- Blocked by: [Issue #]

### Definition of Done
- [ ] All work orders completed
- [ ] Tests pass
- [ ] Documentation updated
- [ ] Release tagged
- [ ] Cashflow source live
```

Create tracking issues for:
- v0.3.1 Foundation
- v0.5.0 Team  
- v0.5.1 Enterprise Connectors
- v0.6.0 Scale
- v0.6.1 Plugins
- v0.7.0 Enterprise
- v1.0.0 Launch

---

## Deferred to Year 2

These are valuable but not cashflow-critical for Year 1:

- **Managed hosting** (Kurultai Cloud) — high ops cost, defer until $25K+ MRR
- **Full plugin marketplace** (centralized) — community can use GitHub initially
- **Enterprise connectors** (Mattermost, WhatsApp) — demand-driven, add after v1.0
- **Learning-to-rank** from usage patterns — R&D project, nice-to-have
- **Advanced graph queries** (knowledge graph) — v2.0 feature
- **Mobile apps** (iOS, Android) — expensive, desktop-first works
- **Advanced compliance** (HIPAA, FedRAMP) — customer-driven, not proactive

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Multi-tenant Postgres complexity | High | High | Start with SQLite, migrate carefully |
| SOC2 compliance timeline | Medium | High | Start prep in Q4, use consultant |
| Enterprise connector maintenance | High | Medium | Community plugins first, official later |
| Plugin system security | Medium | High | Sandboxing + strict permissions |
| Cashflow slower than expected | Medium | High | Keep burn low, prioritize team tier |

---

## Success Metrics

**Technical:**
- All 7 milestones shipped on time
- Test coverage >80%
- Zero critical bugs in releases
- Multi-tenant isolation verified

**Business:**
- $50-100K Year 1 revenue
- 1K+ users by v1.0.0
- 10+ paying teams
- 5+ enterprise contracts
- 20+ community plugins

**Community:**
- 1K+ GitHub stars
- 50+ community contributors
- Active Discord/Slack community
- 10+ blog posts about Kurultai
