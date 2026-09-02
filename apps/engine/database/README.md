# piramid-database

Where vectors live: the bytes on disk, and the metadata they are filtered by.

`storage` owns records, the write-ahead log, mmap, sidecar files, the offset index and the
manifest. It decides nothing about API behaviour or collection lifecycle — a `RecordStore` does
not know what a collection is.

`metadata` is the payload vocabulary: `Metadata`, `MetadataValue`, and the `Filter`
predicates over them. It lives beside the bytes it describes rather than in `core`, since storage
writes it and search reads it.

`vectors::VectorReader` is how indexes read vectors they don't own. `as_slab()` is the fast
path and `gather_into()` the fallback; both have defaults, so a new reader costs nothing.

Part of [Piramid](https://github.com/ashworks1706/piramid). See
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) for how the crates fit together.
