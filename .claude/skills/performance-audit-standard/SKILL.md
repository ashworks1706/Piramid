---
name: performance-audit-standard
description: >
  Performance audit methodology for codebases. Use when profiling applications,
  identifying bottlenecks, or optimizing hot paths. Covers Big O analysis, hot path
  identification, data structure optimization (Set/Map over Array), algorithm improvement
  patterns, async I/O fixes, connection pooling, caching strategies, and priority
  quick-wins matrix format. Derived from production performance optimization work.
---

# Performance Audit Standard

Methodology for identifying and fixing performance bottlenecks.

## Audit Process

### Step 1: Identify Hot Paths

Find code that runs frequently or processes large datasets:
- Request handlers (every HTTP request)
- Event processors (every WebSocket message)
- Loop bodies processing collections (per-item)
- Interval timers (every N seconds)
- Middleware/interceptors (every request)

Ask: "How many times per second does this code execute?" and "What's the typical input size?"

### Step 2: Analyze Complexity

For each hot path, determine actual Big O:

| Pattern | Complexity | Example |
|---|---|---|
| `array.includes(x)` | O(n) | Linear scan per check |
| `array.find(x => ...)` | O(n) | Linear scan |
| `array.filter().map().filter()` | O(3n) + 3 allocations | Multiple passes |
| `Object.entries().find()` | O(n) | Linear scan of object |
| `array.sort()` to find min/max | O(n log n) | Overkill for single value |
| Nested loops with includes | O(n*m) | Quadratic |

### Step 3: Apply Fixes

## Common Performance Anti-Patterns

### 1. O(n) Membership Test → Use Set

```javascript
// BAD: O(n) per check, O(n*m) in a loop
const newIds = currentIds.filter(x => !savedIds.includes(x));

// GOOD: O(1) per check, O(n+m) total
const savedSet = new Set(savedIds);
const newIds = currentIds.filter(x => !savedSet.has(x));
```

Impact: 1000x on large collections (1000 items: 1M comparisons → 1K).

### 2. O(n log n) Selection → Single-Pass

```javascript
// BAD: filter + sort + take first = O(n) + O(n log n) + O(1)
const available = tokens.filter(t => !t.expired);
available.sort((a, b) => a.lastUsed - b.lastUsed);
const best = available[0];

// GOOD: Single O(n) pass
let best = null;
for (const t of tokens) {
  if (t.expired) continue;
  if (!best || t.lastUsed < best.lastUsed) best = t;
}
```

Impact: 10-50x faster, zero intermediate arrays.

### 3. Linear Lookup → Map Index

```javascript
// BAD: O(n) per lookup
function findToken(value) {
  return tokens.find(t => t.value === value);
}

// GOOD: O(1) per lookup
const tokenIndex = new Map(tokens.map(t => [t.value, t]));
function findToken(value) {
  return tokenIndex.get(value);
}
```

Impact: 10-20x per lookup. Maintain index on add/remove.

### 4. Per-Request Parsing → Cache on Init

```javascript
// BAD: Parse every time
function getProxy() {
  const proxies = proxySource.split(/\r?\n/);                    // regex split
  const parts = proxies[random()].split(":");                     // string split
  return `http://${parts[2]}:${parts[3]}@${parts[0]}:${parts[1]}`; // format
}

// GOOD: Parse once, cache formatted URLs
class ProxyPool {
  constructor(source) {
    this.urls = source.split(/\r?\n/).filter(Boolean).map(line => {
      const [h, p, u, pw] = line.split(":");
      return `http://${u}:${pw}@${h}:${p}`;
    });
  }
  random() { return this.urls[Math.floor(Math.random() * this.urls.length)]; }
}
```

Impact: 100-1000x. Regex + string operations are expensive per-call.

### 5. Sync File I/O → Async

```javascript
// BAD: Blocks event loop 5-50ms
fs.writeFileSync(path, JSON.stringify(data, null, 2));

// GOOD: Non-blocking
await fs.promises.writeFile(path, JSON.stringify(data, null, 2));

// BETTER: Skip if unchanged
const hash = crypto.createHash("md5").update(JSON.stringify(data)).digest("hex");
if (hash !== this.lastHash) {
  await fs.promises.writeFile(path, JSON.stringify(data, null, 2));
  this.lastHash = hash;
}
```

### 6. Per-Request Client Creation → Singleton

```rust
// BAD: New TLS handshake + connection pool per call
async fn send(body: Value) {
    let client = reqwest::Client::builder().build().unwrap();
    client.post(url).json(&body).send().await;
}

// GOOD: Build once, reuse forever
static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder().timeout(Duration::from_secs(10)).build().unwrap()
});
```

### 7. Lock Held Across I/O → Clone and Release

```rust
// BAD: Lock held during disk write
let mut data = state.data.write().await;
data.insert(key, value);
let json = serde_json::to_string(&*data).unwrap();
std::fs::write(path, json); // DISK I/O WHILE LOCKED

// GOOD: Clone, release, then write
let json = {
    let mut data = state.data.write().await;
    data.insert(key, value);
    serde_json::to_string(&*data).unwrap()
}; // Lock dropped here
tokio::fs::write(path, json).await;
```

### 8. Repeated Computation in Loops → Hoist

```javascript
// BAD: toLowerCase() called 1000x per event
for (const seat of seats) {
  const name = seat.section.toLowerCase();
  const match = filters.find(f => name.includes(f.keyword.toLowerCase()));
}

// GOOD: Pre-compute outside loop
const lowerKeywords = filters.map(f => ({
  ...f,
  lowerKeyword: f.keyword.toLowerCase(),
}));
for (const seat of seats) {
  const name = seat.section.toLowerCase();
  const match = lowerKeywords.find(f => name.includes(f.lowerKeyword));
}
```

### 9. Multiple Array Passes → Single Pass

```javascript
// BAD: 3 passes, 3 intermediate arrays
const resale = picks.filter(p => p.resaleId);
const normal = picks.filter(p => !p.resaleId && !isFiltered(p));
const combined = [...resale.map(transform), ...normal.map(transform)];

// GOOD: Single pass
const results = [];
for (const p of picks) {
  if (p.resaleId) {
    results.push(transformResale(p));
  } else if (!isFiltered(p)) {
    results.push(transformNormal(p));
  }
}
```

### 10. Database: N+1 Queries

```javascript
// BAD: 1 + N queries (1 list, N detail loads)
const users = await db.user.findMany();
for (const u of users) {
  u.orders = await db.order.findMany({ where: { userId: u.id } });
}

// GOOD: 2 queries, joined in app
const users = await db.user.findMany({ include: { orders: true } });
```

How to find them: log queries per request, set a budget, fail tests if exceeded. Most ORMs have an `include` / `with` / `Preload` to fetch in one round trip.

### 11. Database: Missing Indexes

```sql
EXPLAIN ANALYZE SELECT * FROM events WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50;
-- Seq Scan ... rows=2_500_000  → missing index
CREATE INDEX events_user_id_created_at_idx ON events (user_id, created_at DESC);
```

Audit: every column in `WHERE`, `JOIN`, or `ORDER BY` on a hot query needs an index. Composite index order matches query order (equality columns first, then range).

### 12. Frontend: Bundle Size and Core Web Vitals

| Metric | Good | Tool |
|---|---|---|
| LCP (Largest Contentful Paint) | < 2.5s | Lighthouse, web-vitals |
| INP (Interaction to Next Paint) | < 200ms | web-vitals |
| CLS (Cumulative Layout Shift) | < 0.1 | web-vitals |
| JS bundle (gzipped, initial) | < 170KB | rollup-plugin-visualizer, source-map-explorer |

Common wins: code-split routes, lazy-load below-fold images (`loading="lazy"`), tree-shake icon libs, swap large polyfills.

## Priority Quick-Wins Matrix

Report findings in this format:

| Priority | Fix | Location | Effort | Speedup |
|---|---|---|---|---|
| 1 | Set-based membership | file:line | 2 lines | 1000x |
| 2 | Single-pass selection | file:line | 15 lines | 10-50x |
| 3 | Cache parsed data | file:line | 20 lines | 100-1000x |
| 4 | Index Map for lookups | file:line | 10 lines | 10-20x |
| 5 | Static/singleton client | file:line | 5 lines | per-alloc savings |

Sort by impact/effort ratio. Estimate combined CPU reduction.

## Language-Specific Tools

### JavaScript/Node.js
- `node --prof` + `node --prof-process` for V8 profiling
- `clinic.js` (doctor, flame, bubbleprof)
- `0x` for flame graphs
- `process.memoryUsage()` for heap monitoring

### Rust
- `cargo flamegraph` for flame graphs
- `criterion` for microbenchmarks
- `dhat` for heap profiling
- `tokio-console` for async task inspection
- `perf` (Linux) or Instruments (macOS)

### Go
- `pprof` (CPU + memory profiles)
- `go test -bench`
- `trace` for goroutine analysis

### Swift/iOS
- Instruments: Time Profiler, Allocations, Leaks
- Xcode Memory Graph Debugger
- `os_signpost` for custom measurements

## When NOT to Optimize

- Code runs < 100 times/second with < 100 items
- Optimization adds significant complexity for < 2x gain
- The bottleneck is I/O bound (network, disk), not CPU
- Profiling shows the code isn't actually hot
- "Profile first, optimize second": always measure before and after
