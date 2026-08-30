# piramid-search

Query execution: filtering, scoring, ranking.

Plans overfetch, drives an index, scores candidates, and applies metadata filters.

It takes a `SearchTarget` — index, readers, defaults — rather than a `Collection`, which is what
keeps search below the domain layer instead of circular with it.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../../docs/ARCHITECTURE.md) for how the crates fit together.
