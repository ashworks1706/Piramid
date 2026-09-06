---
name: comment-style
description: The comment rules for this repo. Use when writing or editing any Rust comment, doc comment or module header, and when reviewing a diff that touches comments.
---

# Comment style

Every `//`, `///` and `//!` in the workspace follows these rules. They apply to new code and to
any comment a change touches.

## Say what the code does

A comment states behaviour. It does not argue for it.

Do not write rationale, trade-offs, history, alternatives considered, or what a past version did.
Those belong in a commit message or in docs/decisions. A reader of the source wants to know what
the line in front of them does.

```rust
// Rows are scored one block at a time.
const CHUNK: usize = 1024;
```

Not:

```rust
// Chunked rather than one buffer for the whole collection, because that buffer is the size of
// the data and would be allocated and thrown away once per query.
const CHUNK: usize = 1024;
```

## ASCII only

No backticks, no quotation marks, no em or en dashes, no ellipsis characters, no arrows, no
bullets, no box drawing, no emoji. A comment is plain ASCII prose and ordinary punctuation: full
stops, commas, colons, semicolons, hyphens, parentheses.

A possessive apostrophe is ordinary punctuation and is allowed. A quotation mark used to quote,
name or scare-quote a term is not. Write the term plainly instead.

Intra-doc links keep their brackets and lose their backticks: `[VectorStore]`, not
``[`VectorStore`]``. The link still resolves.

## One register

No opening flourishes and no closing ones. Drop "Note that", "Important:", "The point is",
"Keep in mind", "In other words", "Simply", "Just", "Obviously", "Of course".

No emphasis, no rhetorical questions, no addressing the reader, no jokes, no scare quotes. Write
declarative sentences in the present tense about what the code does.

## Length

One line where one line does. A doc comment on a public item states what the item is. A module
header states what the module holds.

## What stays

- `/// ` on every public item and `//! ` on every module. That convention is unchanged.
- `// SAFETY:` blocks. State the precondition that holds, as a plain sentence.
- `#[allow(..., reason = "...")]` attributes. These are attributes, not comments, and the reason
  string is required to be a string literal.

## Checking

```
rg -n '^\s*(//|///|//!).*[`"]' --glob '*.rs'      # backticks and quotation marks
rg -nP '^\s*(//|///|//!).*[^\x00-\x7F]' --glob '*.rs'  # any non-ASCII byte
```

Any hit is a violation. Neither pattern matches a possessive apostrophe.
