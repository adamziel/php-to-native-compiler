#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: tools/check-phpt-campaign-reports.sh REPORT.md [REPORT.md ...]

Validate PHPT campaign evidence/report documents before publishing them.
Each supplied report must be a real, non-template document with at least 1,000
words by default, and multiple reports in one invocation must be materially
distinct from each other.

Environment:
  PTN_CAMPAIGN_REPORT_MIN_WORDS           minimum word count, default 1000
  PTN_CAMPAIGN_REPORT_MAX_SHARED_PERCENT maximum shared vocabulary percent
                                          between any two reports, default 90
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

min_words=${PTN_CAMPAIGN_REPORT_MIN_WORDS:-1000}
max_shared_percent=${PTN_CAMPAIGN_REPORT_MAX_SHARED_PERCENT:-90}

if [[ ! "$min_words" =~ ^[0-9]+$ || "$min_words" -le 0 ]]; then
  echo "PTN_CAMPAIGN_REPORT_MIN_WORDS must be a positive integer: $min_words" >&2
  exit 2
fi
if [[ ! "$max_shared_percent" =~ ^[0-9]+$ || "$max_shared_percent" -le 0 || "$max_shared_percent" -gt 100 ]]; then
  echo "PTN_CAMPAIGN_REPORT_MAX_SHARED_PERCENT must be 1..100: $max_shared_percent" >&2
  exit 2
fi

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

word_count() {
  tr -cs "[:alnum:]_'" '\n' < "$1" | awk 'NF { count++ } END { print count + 0 }'
}

normalize_report() {
  tr '[:upper:]' '[:lower:]' < "$1" \
    | tr -cs '[:alnum:]' ' ' \
    | sed 's/[[:space:]][[:space:]]*/ /g; s/^ //; s/ $//'
}

write_vocabulary() {
  tr '[:upper:]' '[:lower:]' < "$1" \
    | tr -cs '[:alnum:]' '\n' \
    | awk 'length($0) > 2 { print }' \
    | LC_ALL=C sort -u
}

declare -a reports=()
declare -a normalized_files=()
declare -a vocabulary_files=()
declare -a hashes=()
declare -a vocabulary_counts=()

index=0
for report in "$@"; do
  if [[ ! -f "$report" ]]; then
    echo "report not found: $report" >&2
    exit 1
  fi

  words=$(word_count "$report")
  if [[ "$words" -lt "$min_words" ]]; then
    echo "report too short: $report has $words words; minimum is $min_words" >&2
    exit 1
  fi

  if grep -Eiq '(^|[^[:alpha:]])(todo|tbd|lorem ipsum|template placeholder|copy/paste|fill[ -]?me)([^[:alpha:]]|$)' "$report"; then
    echo "report still looks like a template or draft: $report" >&2
    exit 1
  fi

  normalized="$tmp_dir/normalized-$index.txt"
  vocabulary="$tmp_dir/vocabulary-$index.txt"
  normalize_report "$report" > "$normalized"
  write_vocabulary "$report" > "$vocabulary"
  hash=$(sha256sum "$normalized" | awk '{ print $1 }')
  vocab_count=$(wc -l < "$vocabulary")
  vocab_count=${vocab_count//[[:space:]]/}

  reports+=("$report")
  normalized_files+=("$normalized")
  vocabulary_files+=("$vocabulary")
  hashes+=("$hash")
  vocabulary_counts+=("$vocab_count")
  index=$((index + 1))
done

for ((i = 0; i < ${#reports[@]}; i++)); do
  for ((j = i + 1; j < ${#reports[@]}; j++)); do
    if [[ "${hashes[$i]}" == "${hashes[$j]}" ]]; then
      echo "reports are identical after normalization: ${reports[$i]} and ${reports[$j]}" >&2
      exit 1
    fi

    shared=$(comm -12 "${vocabulary_files[$i]}" "${vocabulary_files[$j]}" | wc -l)
    shared=${shared//[[:space:]]/}
    small=${vocabulary_counts[$i]}
    if [[ "${vocabulary_counts[$j]}" -lt "$small" ]]; then
      small=${vocabulary_counts[$j]}
    fi
    if [[ "$small" -gt 0 ]]; then
      shared_percent=$((shared * 100 / small))
      if [[ "$shared_percent" -ge "$max_shared_percent" ]]; then
        echo "reports are not materially distinct: ${reports[$i]} and ${reports[$j]} share ${shared_percent}% of the smaller vocabulary" >&2
        exit 1
      fi
    fi
  done
done

for ((i = 0; i < ${#reports[@]}; i++)); do
  words=$(word_count "${reports[$i]}")
  echo "report-ok: path=${reports[$i]} words=$words unique-terms=${vocabulary_counts[$i]}"
done
