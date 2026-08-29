#!/usr/bin/env bash
# Verify the workspace dependency rule.
#
# The layering in docs/ARCHITECTURE.md only holds if something checks it. Cargo already makes
# an undeclared edge a compile error; this script makes a *declared* one a CI failure, so adding
# `piramid-server` to `piramid-compute`'s manifest fails here rather than being noticed in review.
#
# Run by `just check-rust`, the pre-commit hook, and CI.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

command -v jq >/dev/null || { echo "check-deps: jq is required"; exit 1; }

# Allowed in-repo dependencies, one "crate -> dependency" per line.
# Anything not listed is a violation. Keep this in sync with docs/ARCHITECTURE.md.
ALLOWED=$(cat <<'EOF'
piramid-core -> piramid-compute
piramid-storage -> piramid-core
piramid-index -> piramid-core
piramid-index -> piramid-compute
piramid-index -> piramid-storage
piramid-search -> piramid-core
piramid-search -> piramid-compute
piramid-search -> piramid-storage
piramid-search -> piramid-index
piramid-collections -> piramid-core
piramid-collections -> piramid-compute
piramid-collections -> piramid-storage
piramid-collections -> piramid-index
piramid-collections -> piramid-search
piramid-embeddings -> piramid-core
piramid-inference -> piramid-core
piramid-inference -> piramid-gpu
piramid-server -> piramid-core
piramid-server -> piramid-compute
piramid-server -> piramid-storage
piramid-server -> piramid-index
piramid-server -> piramid-search
piramid-server -> piramid-collections
piramid-server -> piramid-embeddings
piramid-server -> piramid-observability
piramid-observability -> piramid-core
piramid -> piramid-observability
piramid -> piramid-core
piramid -> piramid-compute
piramid -> piramid-gpu
piramid -> piramid-storage
piramid -> piramid-index
piramid -> piramid-search
piramid -> piramid-collections
piramid -> piramid-embeddings
piramid -> piramid-inference
piramid -> piramid-server
EOF
)

# Every in-repo edge Cargo actually sees (path dependencies between workspace members).
ACTUAL=$(cargo metadata --format-version 1 --no-deps \
  | jq -r '.packages[] | .name as $from | .dependencies[]
           | select(.path != null)
           | "\($from) -> \(.name)"' \
  | sort -u)

status=0

while IFS= read -r edge; do
  [ -z "$edge" ] && continue
  if ! grep -Fxq "$edge" <<<"$ALLOWED"; then
    echo "FAIL undeclared dependency: $edge"
    echo "     if this edge is intended, add it to scripts/check-deps.sh and docs/ARCHITECTURE.md"
    status=1
  fi
done <<<"$ACTUAL"

# `compute` and `gpu` are leaves: they must depend on nothing in the workspace. This is what lets
# kernels be lifted into a standalone benchmark, and what keeps inference from reaching retrieval
# math to get at a device.
for leaf in piramid-compute piramid-gpu; do
  if grep -q "^$leaf -> " <<<"$ACTUAL"; then
    echo "FAIL $leaf must be a leaf crate but depends on:"
    grep "^$leaf -> " <<<"$ACTUAL" | sed 's/^/       /'
    status=1
  fi
done

# The model runtime must not depend on the retrieval stack. `inference::retrieval` holds only the
# RetrievalHook trait; a strategy that actually queries an index is a separate crate depending on
# both piramid-inference and piramid-search. If the runtime itself ever gains a retrieval
# dependency, a collection stops being queryable without a model loaded and the single-process
# design loses its point.
for retrieval in piramid-storage piramid-index piramid-search piramid-collections; do
  if grep -q "^piramid-inference -> $retrieval\$" <<<"$ACTUAL"; then
    echo "FAIL piramid-inference must not depend on the retrieval stack: $retrieval"
    echo "     a retrieval strategy belongs in its own crate depending on both"
    status=1
  fi
done

if [ $status -eq 0 ]; then
  echo "dependency direction ok ($(wc -l <<<"$ACTUAL") in-repo edges)"
fi
exit $status
