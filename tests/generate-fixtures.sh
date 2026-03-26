#!/usr/bin/env bash
set -euo pipefail

# Fixture generator for y1 tests
# Runs real yarn v1.22.22 and captures output as reference.
# Then derives expected y1 output by applying documented deviations.
#
# Usage: ./tests/generate-fixtures.sh
# Must be run from the repo root with node_modules installed.

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
YARN="$REPO_ROOT/node_modules/.bin/yarn"
FIXTURES_DIR="$REPO_ROOT/tests/fixtures"

if [[ ! -x "$YARN" ]]; then
  echo "ERROR: yarn not found at $YARN — run 'npm run refresh' first" >&2
  exit 1
fi

# --- helpers ---

generate_case() {
  local name="$1"
  local cwd="$2"
  shift 2
  local args=("$@")

  local dir="$FIXTURES_DIR/$name"
  mkdir -p "$dir"

  # Write args file (one arg per line)
  if [[ ${#args[@]} -gt 0 ]]; then
    printf '%s\n' "${args[@]}" > "$dir/args"
  else
    : > "$dir/args"
  fi
  # Write cwd relative to repo root
  echo "$cwd" > "$dir/cwd"

  # Run yarn from the example dir, capture stdout/stderr/exitcode
  local abs_cwd="$REPO_ROOT/$cwd"
  local exitcode=0
  (cd "$abs_cwd" && "$YARN" "${args[@]}") \
    >"$dir/yarn-stdout" \
    2>"$dir/yarn-stderr" \
    || exitcode=$?
  echo "$exitcode" > "$dir/yarn-exitcode"

  # Normalize yarn output (timing, paths) for stable diffs
  normalize "$dir/yarn-stdout"
  normalize "$dir/yarn-stderr"

  # Derive expected y1 output from yarn output
  cp "$dir/yarn-stdout" "$dir/expected-stdout"
  cp "$dir/yarn-stderr" "$dir/expected-stderr"
  cp "$dir/yarn-exitcode" "$dir/expected-exitcode"
  apply_deviations "$dir"

  echo "  generated: $name"
}

normalize() {
  local file="$1"
  sed -i 's/Done in [0-9]*\.[0-9]*s\./Done in {DURATION}./g' "$file"
  sed -i "s|$(pwd)|{CWD}|g" "$file"
  sed -i "s|$REPO_ROOT|{CWD}|g" "$file"
}

apply_deviations() {
  local dir="$1"

  # Strip "info Visit https://yarnpkg.com/en/docs/cli/..." lines from stdout
  sed -i '/^info Visit https:\/\/yarnpkg\.com\/en\/docs\/cli\//d' "$dir/expected-stdout"

  # Strip "  Visit https://yarnpkg.com/en/docs/cli/..." lines from stdout (help output)
  sed -i '/^  Visit https:\/\/yarnpkg\.com\/en\/docs\/cli\//d' "$dir/expected-stdout"
}

# --- manual fixture dirs (preserved across regeneration) ---

MANUAL_CASES=(no-args rejected-install help help-run)

# --- clean generated dirs ---

for dir in "$FIXTURES_DIR"/*/; do
  name="$(basename "$dir")"
  skip=false
  for manual in "${MANUAL_CASES[@]}"; do
    if [[ "$name" == "$manual" ]]; then skip=true; break; fi
  done
  if ! $skip; then
    rm -rf "$dir"
  fi
done

# --- scenarios ---

echo "Generating fixtures..."

# Basic task execution
generate_case "run-test-success" "examples/minimal" run test
generate_case "run-task-failure" "examples/minimal" run fail
generate_case "run-nonexistent" "examples/minimal" run nonexistent

# Task with stdout and stderr output
generate_case "run-stdout-stderr" "examples/minimal" run stdout-stderr

# run with no task (lists scripts)
generate_case "run-no-task" "examples/minimal" run

# Shorthand (yarn test = yarn run test)
generate_case "shorthand-test" "examples/minimal" test

# Manual fixture cases — not generated, maintained by hand.
echo "  skipped: help (manual fixtures)"
echo "  skipped: help-run (manual fixtures)"
echo "  skipped: no-args (manual fixtures)"
echo "  skipped: rejected-install (manual fixtures)"

echo ""
echo "Fixtures generated in $FIXTURES_DIR/"

# Check for yarn.lock side effects from running yarn v1
lockfiles=$(find "$REPO_ROOT" -name yarn.lock -not -path '*/node_modules/*' -not -path '*/.git/*')
if [[ -n "$lockfiles" ]]; then
  echo "" >&2
  echo "ERROR: yarn.lock file(s) found — yarn v1 produced side effects:" >&2
  echo "$lockfiles" >&2
  exit 1
fi
