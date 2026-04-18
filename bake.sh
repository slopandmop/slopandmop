#!/usr/bin/env bash
# bake.sh — render one workload spec into plain JSON.
#
# Lifted from dd's apps/_infra/local-agents.sh (same shape, same
# behavior). Two modes:
#
#   *.json.tmpl   — envsubst `${UPPERCASE_VAR}` placeholders from
#                   the caller's env, then jq-drop env-array entries
#                   whose value ended up empty. envsubst is restricted
#                   to the ALL-CAPS `${VAR}` references that appear in
#                   the template itself — lowercase `$i`, `${i}`, and
#                   bare `$((…))` inside `cmd` strings are left alone.
#   *.json        — pass through jq -c (validate + compact).
#
# Usage:
#   MODEL=qwen2.5:7b ./bake.sh openclaw/workload.json.tmpl
set -euo pipefail

[ $# -eq 1 ] || { echo "usage: $0 <workload.json|workload.json.tmpl>" >&2; exit 2; }

case "$1" in
  *.json.tmpl)
    vars=$(grep -oE '\$\{[A-Z_][A-Z0-9_]*\}' "$1" | sort -u | tr -d '\n')
    envsubst "$vars" < "$1" \
      | jq -c 'if .env then .env |= map(select(test("^[^=]+=.+"))) else . end'
    ;;
  *.json)
    jq -c . "$1"
    ;;
  *)
    echo "bake.sh: unknown workload file type: $1" >&2
    exit 1
    ;;
esac
