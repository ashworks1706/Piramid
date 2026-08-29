---
name: adr
description: Write an architecture decision record in docs/decisions/. Use when a change moves a crate boundary, forecloses an option, introduces a dependency, or will look arbitrary to someone reading the tree in a year.
---

# adr

## When a decision earns a record

- It moves a boundary between crates.
- It forecloses an option that a reasonable person would otherwise try.
- It adds a dependency that will be hard to remove.
- It will look arbitrary without the context that produced it.

Routine choices do not need one. If you cannot name the alternative you rejected, it is not a
decision — it is just the code.

## Format

`docs/decisions/NNNN-slug.md`, next free number, four bolded sections:

```markdown
# NNNN — Short imperative title

**Context.** What was true that forced a choice. Concrete: the actual cycle, the actual panic, the
actual measurement. Not "the code was messy".

**Decision.** What we chose, stated so someone can check whether the code matches.

**Consequences.** What this buys and what it costs. Both. A record with no costs is marketing.

**Not decided.** What was deliberately left open, so the next person knows it is an open question
rather than an oversight.
```

## Rules

- Write it when the decision is made, not afterwards. A reconstructed rationale is a after-the-fact story.
- Record the evidence *against* as well as for. `0006` is the example: it cites three findings that
  undercut its own thesis, which is what makes it useful rather than ornamental.
- Name the falsifier where there is one. "What would change our mind" is the most valuable line in
  most records.
- Past records are immutable. A reversal is a new record that supersedes the old one; edit the old
  one only to add a link forward.
- Add a row to the table in `docs/decisions/README.md` in the same commit.

## Style

Short. Most of these are under a page. Prose, not bullets, because the reasoning is the content
and bullets hide the connective tissue. No filler, no restating what the diff shows.
