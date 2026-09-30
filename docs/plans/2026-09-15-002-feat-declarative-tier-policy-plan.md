# feat(store): declarative tier policy for hot/medium/cold membership

Issue: [#325](https://github.com/duketopceo/kurultai/issues/325) · Milestone: Wave H (Serve-path hardening, CobbleDB-inspired)

## Problem

`TierPolicy` (`src/memory/tier.rs`) classifies atoms purely by time thresholds
(`hot_access_days`, `hot_index_hours`, `cold_days`). The "sequester noisy
sources" doctrine (pond/session/transcript noise → medium/cold) currently has
no mechanism — it's a rule in AGENTS.md, not in code. CobbleDB's Pillar
"subsets" are the model: policy-defined groups gating what reaches the
expensive tier.

## Scope

Add **declarative sequester rules** that cap an atom's tier regardless of
timestamps, configured in `config.toml`. Small, self-contained: config
parsing + a rule-eval step in `classify` + tests.

Non-goals: changing default thresholds, new storage, per-atom pinned tiers,
object-storage archive (#34).

## Design

### 1. Rules on `TierPolicy`

```rust
pub struct TierRule {
    /// All present fields must match (AND).
    pub source: Option<String>,      // atom.source, e.g. "pond"
    pub tag: Option<String>,         // membership in atom tags
    pub trust_lane: Option<TrustLane>,
    /// Highest tier a matching atom may reach.
    pub cap: MemoryTier,             // Warm or Cold
}
```

`TierPolicy` gains `rules: Vec<TierRule>` (default empty — behavior
unchanged). `classify` keeps its signature for the pure time-based path; add
`classify_atom(atom, now, policy)` that runs `classify` then applies the
first matching rule's cap (`min(time_tier, cap)` — rules can only lower, never
promote). Ordering: first match wins; document that.

Callers that have a `KnowledgeAtom` (brain graph serve, `count_by_tier`, http
browse) switch to `classify_atom`; the pure `classify` stays for tests/callers
without tags.

### 2. Config

```toml
[tiers]
hot_access_days = 7
hot_index_hours = 48
cold_days = 180

# First match wins; rules can only cap, never promote.
[[tiers.rule]]
source = "pond"
cap = "cold"

[[tiers.rule]]
tag = "session-transcript"
cap = "warm"
```

`src/config/file.rs` gets a `TiersConfig` (serde defaults so a missing
`[tiers]` = today's defaults) and `Config::tier_policy() -> TierPolicy`
does the mapping. Unknown rule fields warn, don't fail. Invalid `cap` values
are rejected at load with a clear error (fail closed beats silent no-op).

### 3. Wiring

`TierPolicy::default()` call sites pass through `Config::tier_policy()` where
a `Config` is in scope (`mcp/brain.rs`, `http/mod.rs`); store-level
`count_by_tier` already takes a policy — the caller supplies the configured
one.

## Units

- `U1` — `TierRule` type + `classify_atom` + first-match cap semantics (`src/memory/tier.rs`)
- `U2` — `TiersConfig` parsing + `Config::tier_policy()` (`src/config/file.rs`)
- `U3` — call-site wiring to configured policy (`src/mcp/brain.rs`, `src/http/mod.rs`, store count path)
- `U4` — tests: rule match/cap, first-match order, missing `[tiers]` = default, bad cap rejected, pond-source atom caps cold

## Verification

- `cargo test --lib` (new tier-policy tests + full suite)
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`
- Manual: temp config with `[[tiers.rule]] source="pond" cap="cold"`, run daemon, confirm pond atoms classify cold via `count_by_tier` / browse API.

## Index

- Update `src/memory/INDEX.md`, `src/config/INDEX.md`, `docs/plans/INDEX.md`, root `INDEX.md` Recent.
