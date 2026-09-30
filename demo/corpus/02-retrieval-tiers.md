---
tags: [demo, retrieval, tiers]
---

# Retrieval tiers

Atoms are tiered hot / medium / cold. Hot serves interactive recall;
medium and cold hold background noise (session transcripts, pond history)
so dogfood search stays usable.

Tier rules are config-driven (`[tier.rules]`): match on source and tag,
assign a trust lane, cap counts. Noise is sequestered by policy, not by
deletion — nothing leaves the store, it just stops surfacing.
