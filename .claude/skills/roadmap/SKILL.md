---
name: roadmap
description: Check progress against docs/ROADMAP.md, or update it. Use when asked what is left, what to work on next, or to record that something shipped.
---

# roadmap

`docs/ROADMAP.md` is the source of truth for what is being built and in what order. It is a todo
list and nothing else: **Now** (in dependency order), **Next**, **Later**, **Unscheduled**,
**Done**, **Out of scope**. Keep it that way — a note that isn't a todo doesn't belong in it.

## Reporting progress

Check claims against the code, not against the checkboxes — a box is a claim, the tree is the
fact. Before saying an item is done, confirm the code exists and the tests cover it.

Report as: what shipped since, what is in flight, what is blocked and on what. Be specific about
blockers; "waiting on the GPU work" is not a blocker, "needs `DeviceBuffer` round-trip before a
kernel can be benchmarked" is.

## Updating it

- Tick a box only when the work is merged and `just check` passes with it.
- Moving an item between sections needs a reason in the commit message.
- Adding to **Out of scope** is a decision — write an ADR (`/adr`).
- Something found broken and not being fixed now becomes an item under **Unscheduled**. An honest
  list is more useful than a short one; do not quietly delete items.

## The bias to correct for

Roadmaps drift optimistic. When updating, ask what got *worse* or was discovered broken, not only
what got done. If nothing was added to **Unscheduled** in a while, the list is probably stale
rather than the code being perfect.
