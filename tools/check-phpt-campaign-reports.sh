#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: tools/check-phpt-campaign-reports.sh REPORT.md [REPORT.md ...]

Validate PHPT campaign progress reports before publishing them. Reports must
be markdown-table-only and contain exactly these columns:

| Ported Tests | Passed Tests |

Every non-empty line must be part of that table, and data cells must be
non-negative integer counts.
EOF
}

if [[ $# -eq 0 ]]; then
  usage >&2
  exit 2
fi

case "${1:-}" in
  -h|--help)
    usage
    exit 0
    ;;
esac

for report in "$@"; do
  if [[ ! -f "$report" ]]; then
    echo "report not found: $report" >&2
    exit 1
  fi

  awk -v report="$report" '
    function trim(value) {
      sub(/^[[:space:]]+/, "", value)
      sub(/[[:space:]]+$/, "", value)
      return value
    }

    function normalize_header(value) {
      value = trim(tolower(value))
      gsub(/[[:space:]]+/, " ", value)
      return value
    }

    function split_table_row(line, cells,    body, count, i) {
      body = line
      sub(/^[[:space:]]*\|/, "", body)
      sub(/\|[[:space:]]*$/, "", body)
      count = split(body, cells, /\|/)
      for (i = 1; i <= count; i++) {
        cells[i] = trim(cells[i])
      }
      return count
    }

    /^[[:space:]]*$/ { next }

    {
      if ($0 !~ /^[[:space:]]*\|.*\|[[:space:]]*$/) {
        printf "report must be table-only: %s line %d\n", report, FNR > "/dev/stderr"
        exit 1
      }

      row_count++
      cell_count = split_table_row($0, cells)
      if (cell_count != 2) {
        printf "report must contain only Ported Tests and Passed Tests columns: %s line %d\n", report, FNR > "/dev/stderr"
        exit 1
      }

      if (row_count == 1) {
        if (normalize_header(cells[1]) != "ported tests" || normalize_header(cells[2]) != "passed tests") {
          printf "report header must be: | Ported Tests | Passed Tests |: %s line %d\n", report, FNR > "/dev/stderr"
          exit 1
        }
        next
      }

      if (row_count == 2) {
        if (cells[1] !~ /^:?-{3,}:?$/ || cells[2] !~ /^:?-{3,}:?$/) {
          printf "report table separator is invalid: %s line %d\n", report, FNR > "/dev/stderr"
          exit 1
        }
        next
      }

      if (cells[1] !~ /^[0-9]+$/ || cells[2] !~ /^[0-9]+$/) {
        printf "report data cells must be integer counts: %s line %d\n", report, FNR > "/dev/stderr"
        exit 1
      }
      data_rows++
    }

    END {
      if (row_count == 0) {
        printf "report is empty: %s\n", report > "/dev/stderr"
        exit 1
      }
      if (row_count < 3 || data_rows < 1) {
        printf "report must contain at least one data row: %s\n", report > "/dev/stderr"
        exit 1
      }
      printf "report-ok: path=%s rows=%d\n", report, data_rows
    }
  ' "$report"
done
