#!/usr/bin/env bash
# Verify the workspace dependency rule from docs/ARCHITECTURE.md.
#
# Cargo rejects an undeclared edge; this rejects a declared one. Run by just check-rust, the
# pre-commit hook, and CI.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

command -v jq >/dev/null || { echo "check-deps: jq is required"; exit 1; }

# Allowed in-repo dependencies, one "crate -> dependency" per line.
# Anything not listed is a violation. Keep this in sync with docs/ARCHITECTURE.md.
ALLOWED=$(cat <<'EOF'
piramid-core -> piramid-hardware
piramid-observability -> piramid-core
piramid-database -> piramid-core
piramid-retrieval -> piramid-core
piramid-retrieval -> piramid-hardware
piramid-retrieval -> piramid-database
piramid-collections -> piramid-core
piramid-collections -> piramid-hardware
piramid-collections -> piramid-database
piramid-collections -> piramid-retrieval
piramid-fusion -> piramid-core
piramid-fusion -> piramid-hardware
piramid-model -> piramid-core
piramid-model -> piramid-hardware
piramid-model -> piramid-fusion
piramid-serving -> piramid-core
piramid-serving -> piramid-hardware
piramid-serving -> piramid-database
piramid-serving -> piramid-retrieval
piramid-serving -> piramid-collections
piramid-serving -> piramid-model
piramid-serving -> piramid-observability
piramid -> piramid-core
piramid -> piramid-observability
piramid -> piramid-hardware
piramid -> piramid-database
piramid -> piramid-retrieval
piramid -> piramid-collections
piramid -> piramid-fusion
piramid -> piramid-model
piramid -> piramid-serving
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

# hardware is a leaf, so kernels stay liftable into a standalone benchmark and nothing above
# has to be present to measure them.
for leaf in piramid-hardware; do
  if grep -q "^$leaf -> " <<<"$ACTUAL"; then
    echo "FAIL $leaf must be a leaf crate but depends on:"
    grep "^$leaf -> " <<<"$ACTUAL" | sed 's/^/       /'
    status=1
  fi
done

# The model runtime must not depend on retrieval, or a collection stops being queryable without a
# model loaded. A hook implementation belongs in its own crate depending on both.
for retrieval in piramid-database piramid-retrieval piramid-collections; do
  for runtime in piramid-model piramid-fusion; do
    if grep -q "^$runtime -> $retrieval\$" <<<"$ACTUAL"; then
      echo "FAIL $runtime must not depend on the retrieval stack: $retrieval"
      echo "     a hook implementation belongs in its own crate depending on both"
      status=1
    fi
  done
done

if [ $status -eq 0 ]; then
  echo "dependency direction ok ($(wc -l <<<"$ACTUAL") in-repo edges)"
fi
exit $status
