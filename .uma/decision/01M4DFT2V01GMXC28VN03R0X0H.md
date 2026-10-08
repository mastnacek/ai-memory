---
id: 01M4DFT2V01GMXC28VN03R0X0H
scope: "project:ai-memory"
type: decision
title: Consolidation proposes; it never applies
tags:
  - consolidate
  - s5
  - safety
  - approval
status: stable
generated:
  by: pi-agent/1.1
  at: "2026-10-08T10:09:13.568571200+00:00"
verified:
  - by: "human:operator"
    at: "2026-10-08T10:09:13.568572700+00:00"
since: "2026-10-08T10:09:13.568572700+00:00"
---
### Context
Memory accumulates restatements and live contradictions, and nothing in the store notices. Supersession keeps memory consistent only when somebody spots the duplicate first.

### Decision
Slice S5 is split by capability, not by convenience:
- `uma consolidate` / the `uma_consolidate` tool only **detect**: they group near-duplicate facts (union-find over pairs scoring above `--threshold`) and flag polarity conflicts. Read-only, and therefore deliberately not behind the approval gate.
- **Applying** a proposal stays with `uma_write` / `uma_supersede`, the gated tools. The agent reads the report and then performs the write, which raises the modal.

Analysis is deterministic and offline (`uma-core/src/similarity.rs` + `consolidate.rs`): tokenize → light stemming → Jaccard, blended 60% title / 40% body. Embeddings are an enhancement, never a prerequisite, so the slice is fully unit-testable with no network or API key.

Polarity is read from the **title only**: bodies contain comparative asides ("use pnpm, not npm") that would otherwise register as false contradictions.

### Consequences
- A bulk cleanup can never become a silent rewrite — every application is one consented, gated write.
- Recall is preferred over precision: a false proposal costs one glance, a missed duplicate rots into two competing truths.
- Verified end-to-end: two "use pnpm" facts → one merge proposal (score 0.84); "Do not use pnpm" flagged against both positives; an unrelated fact ignored.