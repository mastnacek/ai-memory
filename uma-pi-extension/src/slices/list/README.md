# `uma_list` slice (Pi)

**Tool:** `uma_list`

## What it does
Registers `uma_list`, which enumerates facts for a scope and optional type via `uma list`, inheriting the CLI's default of hiding deprecated facts.

## Why it exists
Fast orientation without a query — "what does this project already know?". It is also the cheapest way for an agent to check whether a fact already exists before writing a near-duplicate, which is the behaviour that keeps the store from filling with restatements.

## Invariant
- Must not surface deprecated facts as if they were current; the CLI contract is the single source of that rule.
