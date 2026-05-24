#!/usr/bin/env bash
set -euo pipefail

BENCHMARK_DIR="$(cd "$(dirname "$0")/.." && pwd)/benchmarks"
SMALL_BIN="${SMALL_BIN:-cargo run --}"
TIMEOUT_SEC="${BENCHMARK_TIMEOUT:-300}"
PASS=0
FAIL=0
SKIP=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()  { echo "[$(date '+%H:%M:%S')] $*"; }
pass() { echo -e "  ${GREEN}PASS${NC}"; }
fail() { echo -e "  ${RED}FAIL${NC}: $1"; }
skip() { echo -e "  ${YELLOW}SKIP${NC}: $1"; }

run_one() {
  local dir="$1"
  local name
  name=$(basename "$dir")

  log " $name"

  if [[ ! -f "$dir/small.yaml" ]]; then
    skip "no small.yaml"
    SKIP=$((SKIP + 1))
    return
  fi

  cd "$dir"

  log "  install..."
  if timeout "$TIMEOUT_SEC" $SMALL_BIN install </dev/null >/tmp/small-benchmark-install.log 2>&1; then
    :
  else
    fail "install failed — tail: $(tail -3 /tmp/small-benchmark-install.log 2>/dev/null | tr '\n' ' ')"
    FAIL=$((FAIL + 1))
    return
  fi

  if grep -q '^test:' small.yaml 2>/dev/null; then
    log "  test..."
    if timeout "$TIMEOUT_SEC" $SMALL_BIN test </dev/null >/tmp/small-benchmark-test.log 2>&1; then
      :
    else
      fail "test failed — tail: $(tail -3 /tmp/small-benchmark-test.log 2>/dev/null | tr '\n' ' ')"
      FAIL=$((FAIL + 1))
      return
    fi
  fi

  log "  clean..."
  $SMALL_BIN clean </dev/null >/dev/null 2>&1 || true

  pass
  PASS=$((PASS + 1))
}

main() {
  mkdir -p "$BENCHMARK_DIR"

  log "=== small benchmark ==="
  log "bin: $SMALL_BIN"
  log ""

  local targets=()
  if [[ $# -gt 0 ]]; then
    for pattern in "$@"; do
      for d in "$BENCHMARK_DIR"/*/; do
        [[ -d "$d" ]] || continue
        local name=$(basename "$d")
        if echo "$name" | grep -qE "$pattern"; then
          targets+=("$d")
        fi
      done
    done
  else
    for d in "$BENCHMARK_DIR"/*/; do
      [[ -d "$d" ]] || continue
      targets+=("$d")
    done
  fi

  if [[ ${#targets[@]} -eq 0 ]]; then
    log "No benchmark projects found in $BENCHMARK_DIR/"
    log "Add a project: git clone <url> $BENCHMARK_DIR/<name> && cd $BENCHMARK_DIR/<name> && small init"
    exit 0
  fi

  for d in "${targets[@]}"; do
    run_one "$d"
    echo ""
  done

  local total=$((PASS + FAIL + SKIP))
  log "=== Results ==="
  log "  Pass: $PASS | Fail: $FAIL | Skip: $SKIP | Total: $total"
  echo ""

  if [[ $FAIL -gt 0 ]]; then
    echo -e "  ${RED}$FAIL benchmark(s) failed${NC}"
    exit 1
  fi

  echo -e "  ${GREEN}All $PASS benchmarks passed${NC}"
}

main "$@"
