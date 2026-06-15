#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

features_file="tools/status-dashboard-features.tsv"
status_md="STATUS.md"
status_html="STATUS.html"

if [[ ! -f "$features_file" ]]; then
  echo "missing $features_file" >&2
  exit 1
fi

meta_value() {
  local key="$1"
  awk -v prefix="# ${key}=" '
    index($0, prefix) == 1 {
      print substr($0, length(prefix) + 1)
      exit
    }
  ' "$features_file"
}

html_escape() {
  printf '%s' "$1" \
    | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' -e 's/"/\&quot;/g'
}

md_cell() {
  local value="$1"
  value="${value//\\/\\\\}"
  value="${value//|/\\|}"
  printf '%s' "$value"
}

require_number() {
  local value="$1"
  local field="$2"
  local line="$3"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "$features_file:$line: $field must be a non-negative integer" >&2
    exit 1
  fi
}

require_hour() {
  local value="$1"
  local field="$2"
  local line="$3"
  if [[ ! "$value" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:00Z$ ]]; then
    echo "$features_file:$line: $field must use YYYY-MM-DDTHH:00Z" >&2
    exit 1
  fi
}

refresh="$(meta_value refreshed_at_utc)"
source_commit="$(meta_value source_commit)"
php_src_revision="$(meta_value php_src_revision)"
window_start="$(meta_value window_start_utc)"
window_hours="$(meta_value window_hours)"

if [[ -z "$refresh" || -z "$source_commit" || -z "$php_src_revision" || -z "$window_start" || -z "$window_hours" ]]; then
  echo "$features_file: missing required metadata" >&2
  exit 1
fi

require_hour "$window_start" window_start_utc 1
require_number "$window_hours" window_hours 1
if [[ "$window_hours" -ne 24 ]]; then
  echo "$features_file: window_hours must be 24" >&2
  exit 1
fi

declare -a features=()
declare -a ported_tests=()
declare -a passed_tests=()
declare -a upstream_tests=()
declare -A hourly_new=()

line_no=0
header_seen=0
while IFS= read -r line || [[ -n "$line" ]]; do
  line_no=$((line_no + 1))
  [[ -n "$line" ]] || continue
  [[ "$line" != \#* ]] || continue

  IFS=$'\t' read -r feature ported passed upstream completed_hour newly_passed evidence extra <<< "$line"
  if [[ "$header_seen" -eq 0 ]]; then
    expected=$'feature\tported_tests\tpassed_tests\tupstream_tests\tcompleted_hour_utc\tnewly_passed_tests\tevidence'
    if [[ "$line" != "$expected" ]]; then
      echo "$features_file:$line_no: unexpected header" >&2
      exit 1
    fi
    header_seen=1
    continue
  fi

  if [[ -n "${extra:-}" || -z "${feature:-}" || -z "${ported:-}" || -z "${passed:-}" || -z "${upstream:-}" || -z "${completed_hour:-}" || -z "${newly_passed:-}" || -z "${evidence:-}" ]]; then
    echo "$features_file:$line_no: expected 7 tab-separated fields" >&2
    exit 1
  fi

  require_number "$ported" ported_tests "$line_no"
  require_number "$passed" passed_tests "$line_no"
  require_number "$upstream" upstream_tests "$line_no"
  require_number "$newly_passed" newly_passed_tests "$line_no"
  require_hour "$completed_hour" completed_hour_utc "$line_no"

  if (( passed > ported )); then
    echo "$features_file:$line_no: passed_tests exceeds ported_tests" >&2
    exit 1
  fi
  if (( ported > upstream )); then
    echo "$features_file:$line_no: ported_tests exceeds upstream_tests" >&2
    exit 1
  fi

  features+=("$feature")
  ported_tests+=("$ported")
  passed_tests+=("$passed")
  upstream_tests+=("$upstream")
  hourly_new["$completed_hour"]=$(( ${hourly_new["$completed_hour"]:-0} + newly_passed ))
done < "$features_file"

if [[ "$header_seen" -ne 1 || "${#features[@]}" -eq 0 ]]; then
  echo "$features_file: no feature rows found" >&2
  exit 1
fi

hour_at_offset() {
  local offset="$1"
  date -u -d "${window_start} + ${offset} hour" '+%Y-%m-%dT%H:00Z'
}

{
  echo "# PTN Status"
  echo
  echo "| field | value |"
  echo "| --- | --- |"
  printf '| last refresh | %s |\n' "$(md_cell "$refresh")"
  printf '| source commit | `%s` |\n' "$(md_cell "$source_commit")"
  printf '| php-src revision | `%s` |\n' "$(md_cell "$php_src_revision")"
  printf '| evidence source | `%s` |\n' "$features_file"
  printf '| generator | `%s` |\n' "tools/update-status-dashboard.sh"
  echo
  echo "## Feature Table"
  echo
  echo "| feature | ported tests | passed tests | upstream tests |"
  echo "| --- | ---: | ---: | ---: |"
  for ((i = 0; i < ${#features[@]}; i++)); do
    printf '| %s | %s | %s | %s |\n' \
      "$(md_cell "${features[$i]}")" \
      "${ported_tests[$i]}" \
      "${passed_tests[$i]}" \
      "${upstream_tests[$i]}"
  done
  echo
  echo "## Last 24 Hours"
  echo
  echo "| hour (UTC) | newly passed tests |"
  echo "| --- | ---: |"
  for ((i = 0; i < window_hours; i++)); do
    hour="$(hour_at_offset "$i")"
    printf '| %s | %s |\n' "$hour" "${hourly_new["$hour"]:-0}"
  done
} > "$status_md"

{
  echo '<!doctype html>'
  echo '<html lang="en">'
  echo '<meta charset="utf-8">'
  echo '<meta name="viewport" content="width=device-width, initial-scale=1">'
  echo '<title>PTN Status</title>'
  echo '<style>'
  echo ':root { color-scheme: light dark; font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }'
  echo 'body { margin: 0; padding: 24px; line-height: 1.35; }'
  echo 'main { max-width: 1180px; margin: 0 auto; }'
  echo 'h1, h2 { margin: 0 0 12px; }'
  echo 'h2 { margin-top: 28px; }'
  echo 'table { width: 100%; border-collapse: collapse; margin: 0 0 18px; font-size: 14px; }'
  echo 'th, td { border: 1px solid #8a8f982e; padding: 8px 10px; text-align: left; vertical-align: top; }'
  echo 'th { background: #8a8f981f; }'
  echo 'td.num, th.num { text-align: right; font-variant-numeric: tabular-nums; }'
  echo 'code { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }'
  echo '@media (max-width: 760px) { body { padding: 12px; } table { font-size: 12px; } th, td { padding: 6px; } }'
  echo '</style>'
  echo '<main>'
  echo '<h1>PTN Status</h1>'
  echo '<table aria-label="status metadata">'
  echo '  <thead><tr><th>field</th><th>value</th></tr></thead>'
  echo '  <tbody>'
  printf '    <tr><td>last refresh</td><td>%s</td></tr>\n' "$(html_escape "$refresh")"
  printf '    <tr><td>source commit</td><td><code>%s</code></td></tr>\n' "$(html_escape "$source_commit")"
  printf '    <tr><td>php-src revision</td><td><code>%s</code></td></tr>\n' "$(html_escape "$php_src_revision")"
  printf '    <tr><td>evidence source</td><td><code>%s</code></td></tr>\n' "$(html_escape "$features_file")"
  echo '    <tr><td>generator</td><td><code>tools/update-status-dashboard.sh</code></td></tr>'
  echo '  </tbody>'
  echo '</table>'
  echo '<h2>Feature Table</h2>'
  echo '<table aria-label="feature table">'
  echo '  <thead><tr><th>feature</th><th class="num">ported tests</th><th class="num">passed tests</th><th class="num">upstream tests</th></tr></thead>'
  echo '  <tbody>'
  for ((i = 0; i < ${#features[@]}; i++)); do
    printf '    <tr><td>%s</td><td class="num">%s</td><td class="num">%s</td><td class="num">%s</td></tr>\n' \
      "$(html_escape "${features[$i]}")" \
      "$(html_escape "${ported_tests[$i]}")" \
      "$(html_escape "${passed_tests[$i]}")" \
      "$(html_escape "${upstream_tests[$i]}")"
  done
  echo '  </tbody>'
  echo '</table>'
  echo '<h2>Last 24 Hours</h2>'
  echo '<table aria-label="last 24 hours">'
  echo '  <thead><tr><th>hour (UTC)</th><th class="num">newly passed tests</th></tr></thead>'
  echo '  <tbody>'
  for ((i = 0; i < window_hours; i++)); do
    hour="$(hour_at_offset "$i")"
    printf '    <tr><td>%s</td><td class="num">%s</td></tr>\n' \
      "$(html_escape "$hour")" \
      "$(html_escape "${hourly_new["$hour"]:-0}")"
  done
  echo '  </tbody>'
  echo '</table>'
  echo '</main>'
  echo '</html>'
} > "$status_html"
