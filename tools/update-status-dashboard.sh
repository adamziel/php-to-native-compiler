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

# shellcheck source=tools/phpt-corpus.sh
source "tools/phpt-corpus.sh"

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

hour_at_offset() {
  local offset="$1"
  date -u -d "${window_start} + ${offset} hour" '+%Y-%m-%dT%H:00Z'
}

percent() {
  local numerator="$1"
  local denominator="$2"
  if (( denominator == 0 )); then
    printf '0.0'
    return
  fi
  awk -v numerator="$numerator" -v denominator="$denominator" \
    'BEGIN { printf "%.1f", (numerator * 100.0) / denominator }'
}

progress_bar_html() {
  local passed="$1"
  local ported="$2"
  local upstream="$3"
  local label="$4"
  local passed_pct ported_only_pct ported_only

  passed_pct="$(percent "$passed" "$upstream")"
  ported_only=$((ported - passed))
  if (( ported_only < 0 )); then
    ported_only=0
  fi
  ported_only_pct="$(percent "$ported_only" "$upstream")"

  printf '<div class="progress" aria-label="%s: %s passed, %s ported, %s upstream">' \
    "$(html_escape "$label")" \
    "$(html_escape "$passed")" \
    "$(html_escape "$ported")" \
    "$(html_escape "$upstream")"
  printf '<span class="progress-pass" style="width: %s%%"></span>' "$passed_pct"
  printf '<span class="progress-ported" style="left: %s%%; width: %s%%"></span>' "$passed_pct" "$ported_only_pct"
  printf '</div>'
}

refresh="$(meta_value refreshed_at_utc)"
source_commit="$(meta_value source_commit)"
php_src_revision_hint="$(meta_value php_src_revision)"
window_start="$(meta_value window_start_utc)"
window_hours="$(meta_value window_hours)"

if [[ -z "$refresh" || -z "$source_commit" || -z "$php_src_revision_hint" || -z "$window_start" || -z "$window_hours" ]]; then
  echo "$features_file: missing required metadata" >&2
  exit 1
fi
refresh="${STATUS_DASHBOARD_REFRESH_AT_UTC:-$refresh}"
source_commit="${STATUS_DASHBOARD_SOURCE_COMMIT:-$source_commit}"

require_hour "$window_start" window_start_utc 1
require_number "$window_hours" window_hours 1
if (( window_hours < 1 || window_hours > 168 )); then
  echo "$features_file: window_hours must be between 1 and 168" >&2
  exit 1
fi

if [[ -z "${PHP_SRC_PHPT:-}" && -z "${PTN_PHP_SRC_REF:-}" && "$php_src_revision_hint" != "unknown" ]]; then
  export PTN_PHP_SRC_REF="$php_src_revision_hint"
fi

corpus_dir="$(ptn_resolve_phpt_corpus "$repo_root")"
php_src_revision="$(ptn_phpt_corpus_revision "$corpus_dir")"
corpus_count="$(find "$corpus_dir" -type f -name '*.phpt' -print | wc -l | tr -d '[:space:]')"
require_number "$corpus_count" corpus_count 1
if (( corpus_count == 0 )); then
  echo "$corpus_dir: no PHPT tests found" >&2
  exit 1
fi
corpus_identity="php-src@${php_src_revision} (${corpus_count} PHPT tests)"

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

known_upstream=0
for upstream in "${upstream_tests[@]}"; do
  known_upstream=$((known_upstream + upstream))
done

if (( known_upstream > corpus_count )); then
  echo "$features_file: upstream_tests total $known_upstream exceeds PHPT corpus count $corpus_count from $corpus_dir" >&2
  exit 1
fi

remaining_upstream=$((corpus_count - known_upstream))
if (( remaining_upstream > 0 )); then
  features+=("unported/unmeasured upstream PHPT corpus")
  ported_tests+=(0)
  passed_tests+=(0)
  upstream_tests+=("$remaining_upstream")
fi

total_ported=0
total_passed=0
total_upstream=0
for ((i = 0; i < ${#features[@]}; i++)); do
  total_ported=$((total_ported + ported_tests[i]))
  total_passed=$((total_passed + passed_tests[i]))
  total_upstream=$((total_upstream + upstream_tests[i]))
done

if (( total_upstream != corpus_count )); then
  echo "generated upstream_tests total $total_upstream does not equal PHPT corpus count $corpus_count" >&2
  exit 1
fi

declare -a history_hours=()
declare -a history_new_values=()
declare -a history_cumulative_values=()

history_new_total=0
for ((i = 0; i < window_hours; i++)); do
  hour="$(hour_at_offset "$i")"
  new_value="${hourly_new["$hour"]:-0}"
  history_hours+=("$hour")
  history_new_values+=("$new_value")
  history_new_total=$((history_new_total + new_value))
done

history_baseline=$((total_passed - history_new_total))
if (( history_baseline < 0 )); then
  echo "$features_file: hourly newly_passed_tests total $history_new_total exceeds passed_tests total $total_passed" >&2
  exit 1
fi

cumulative="$history_baseline"
for ((i = 0; i < window_hours; i++)); do
  cumulative=$((cumulative + history_new_values[i]))
  history_cumulative_values+=("$cumulative")
done

chart_points="$(
  printf '%s\n' "${history_cumulative_values[@]}" \
    | awk -v left=38 -v top=14 -v width=656 -v height=112 -v max="$total_passed" '
      BEGIN {
        if (max < 1) {
          max = 1
        }
      }
      {
        values[count++] = $1
      }
      END {
        for (i = 0; i < count; i++) {
          if (count == 1) {
            x = left
          } else {
            x = left + (i * width / (count - 1))
          }
          y = top + height - (values[i] * height / max)
          printf "%s%.2f,%.2f", sep, x, y
          sep = " "
        }
      }
    '
)"
chart_last_hour="${history_hours[$((window_hours - 1))]}"

{
  echo "# PTN Status"
  echo
  echo "| field | value |"
  echo "| --- | --- |"
  printf '| last refresh | %s |\n' "$(md_cell "$refresh")"
  printf '| source commit | `%s` |\n' "$(md_cell "$source_commit")"
  printf '| php-src revision | `%s` |\n' "$(md_cell "$php_src_revision")"
  printf '| upstream PHPT corpus | `%s` |\n' "$(md_cell "$corpus_identity")"
  printf '| evidence source | `%s` |\n' "$features_file"
  printf '| generator | `%s` |\n' "tools/update-status-dashboard.sh"
  echo
  echo "## Totals"
  echo
  echo "| metric | tests |"
  echo "| --- | ---: |"
  printf '| ported tests | %s |\n' "$total_ported"
  printf '| passed tests | %s |\n' "$total_passed"
  printf '| upstream tests | %s |\n' "$total_upstream"
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
  echo "## Last 7 Days"
  echo
  echo "| hour (UTC) | newly passed tests | cumulative passed tests |"
  echo "| --- | ---: | ---: |"
  for ((i = 0; i < window_hours; i++)); do
    printf '| %s | %s | %s |\n' \
      "${history_hours[$i]}" \
      "${history_new_values[$i]}" \
      "${history_cumulative_values[$i]}"
  done
} > "$status_md"

{
  echo '<!doctype html>'
  echo '<html lang="en">'
  echo '<meta charset="utf-8">'
  echo '<meta name="viewport" content="width=device-width, initial-scale=1">'
  echo '<title>PTN Status</title>'
  echo '<style>'
  echo ':root { color-scheme: light dark; font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; --border: #8a8f9840; --muted: #667085; --surface: #8a8f9818; --pass: #238636; --ported: #d29922; --line: #0969da; }'
  echo 'body { margin: 0; padding: 24px; line-height: 1.35; background: Canvas; color: CanvasText; }'
  echo 'main { max-width: 1180px; margin: 0 auto; }'
  echo 'h1, h2 { margin: 0 0 12px; }'
  echo 'h2 { margin-top: 28px; }'
  echo '.summary { color: var(--muted); margin: 0 0 12px; }'
  echo 'table { width: 100%; border-collapse: collapse; margin: 0 0 18px; font-size: 14px; }'
  echo 'th, td { border: 1px solid var(--border); padding: 8px 10px; text-align: left; vertical-align: top; }'
  echo 'th { background: var(--surface); }'
  echo 'td.num, th.num { text-align: right; font-variant-numeric: tabular-nums; }'
  echo 'code { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }'
  echo '.sortable th button { all: unset; box-sizing: border-box; cursor: pointer; display: inline-flex; align-items: center; gap: 6px; width: 100%; }'
  echo '.sortable th.num button { justify-content: flex-end; }'
  echo '.sortable th button::after { color: var(--muted); content: "-"; font-size: 11px; }'
  echo '.sortable th[aria-sort="ascending"] button::after { content: "^"; }'
  echo '.sortable th[aria-sort="descending"] button::after { content: "v"; }'
  echo '.feature-name { margin-bottom: 6px; }'
  echo '.progress { background: var(--surface); border: 1px solid var(--border); border-radius: 999px; height: 8px; overflow: hidden; position: relative; }'
  echo '.progress span { bottom: 0; display: block; position: absolute; top: 0; }'
  echo '.progress-pass { background: var(--pass); left: 0; z-index: 2; }'
  echo '.progress-ported { background: var(--ported); z-index: 1; }'
  echo '.chart-panel { margin: 0 0 20px; }'
  echo '.line-chart { display: block; height: auto; max-width: 100%; width: 100%; }'
  echo '.chart-grid { stroke: var(--border); stroke-width: 1; }'
  echo '.chart-line { fill: none; stroke: var(--line); stroke-linecap: round; stroke-linejoin: round; stroke-width: 3; }'
  echo '.chart-label { fill: var(--muted); font: 12px ui-sans-serif, system-ui, sans-serif; }'
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
  printf '    <tr><td>upstream PHPT corpus</td><td><code>%s</code></td></tr>\n' "$(html_escape "$corpus_identity")"
  printf '    <tr><td>evidence source</td><td><code>%s</code></td></tr>\n' "$(html_escape "$features_file")"
  echo '    <tr><td>generator</td><td><code>tools/update-status-dashboard.sh</code></td></tr>'
  echo '  </tbody>'
  echo '</table>'
  echo '<h2>Totals</h2>'
  echo '<table aria-label="status totals">'
  echo '  <thead><tr><th>metric</th><th class="num">tests</th></tr></thead>'
  echo '  <tbody>'
  printf '    <tr><td>ported tests</td><td class="num">%s</td></tr>\n' "$(html_escape "$total_ported")"
  printf '    <tr><td>passed tests</td><td class="num">%s</td></tr>\n' "$(html_escape "$total_passed")"
  printf '    <tr><td>upstream tests</td><td class="num">%s</td></tr>\n' "$(html_escape "$total_upstream")"
  echo '  </tbody>'
  echo '</table>'
  echo '<section class="chart-panel" aria-labelledby="passing-history-title">'
  echo '  <h2 id="passing-history-title">7-Day Passing History</h2>'
  printf '  <p class="summary">%s cumulative passed tests across %s hourly points, ending %s.</p>\n' \
    "$(html_escape "$total_passed")" \
    "$(html_escape "$window_hours")" \
    "$(html_escape "$chart_last_hour")"
  echo '  <svg id="passing-history-chart" class="line-chart" data-history-points="'"$(html_escape "$window_hours")"'" data-corpus-total="'"$(html_escape "$total_upstream")"'" viewBox="0 0 720 152" role="img" aria-label="Cumulative passing tests by hour">'
  echo '    <line class="chart-grid" x1="38" y1="14" x2="38" y2="126"></line>'
  echo '    <line class="chart-grid" x1="38" y1="126" x2="694" y2="126"></line>'
  printf '    <polyline id="passing-history-line" class="chart-line" data-points="%s" points="%s"></polyline>\n' \
    "$(html_escape "$window_hours")" \
    "$(html_escape "$chart_points")"
  printf '    <text class="chart-label" x="38" y="146">%s</text>\n' "$(html_escape "$window_start")"
  printf '    <text class="chart-label" x="694" y="146" text-anchor="end">%s</text>\n' "$(html_escape "$chart_last_hour")"
  printf '    <text class="chart-label" x="44" y="24">%s passed</text>\n' "$(html_escape "$total_passed")"
  echo '  </svg>'
  echo '</section>'
  echo '<h2>Feature Table</h2>'
  echo '<table id="feature-table" class="sortable" aria-label="feature table">'
  echo '  <thead><tr><th data-sort="text" aria-sort="none"><button type="button">feature</button></th><th class="num" data-sort="number" aria-sort="none"><button type="button">ported tests</button></th><th class="num" data-sort="number" aria-sort="none"><button type="button">passed tests</button></th><th class="num" data-sort="number" aria-sort="none"><button type="button">upstream tests</button></th></tr></thead>'
  echo '  <tbody>'
  for ((i = 0; i < ${#features[@]}; i++)); do
    printf '    <tr><td><div class="feature-name">%s</div>%s</td><td class="num" data-value="%s">%s</td><td class="num" data-value="%s">%s</td><td class="num" data-value="%s">%s</td></tr>\n' \
      "$(html_escape "${features[$i]}")" \
      "$(progress_bar_html "${passed_tests[$i]}" "${ported_tests[$i]}" "${upstream_tests[$i]}" "${features[$i]}")" \
      "$(html_escape "${ported_tests[$i]}")" \
      "$(html_escape "${ported_tests[$i]}")" \
      "$(html_escape "${passed_tests[$i]}")" \
      "$(html_escape "${passed_tests[$i]}")" \
      "$(html_escape "${upstream_tests[$i]}")" \
      "$(html_escape "${upstream_tests[$i]}")"
  done
  echo '  </tbody>'
  echo '</table>'
  echo '<h2>Last 7 Days</h2>'
  echo '<table aria-label="last 7 days">'
  echo '  <thead><tr><th>hour (UTC)</th><th class="num">newly passed tests</th><th class="num">cumulative passed tests</th></tr></thead>'
  echo '  <tbody>'
  for ((i = 0; i < window_hours; i++)); do
    printf '    <tr><td>%s</td><td class="num">%s</td><td class="num">%s</td></tr>\n' \
      "$(html_escape "${history_hours[$i]}")" \
      "$(html_escape "${history_new_values[$i]}")" \
      "$(html_escape "${history_cumulative_values[$i]}")"
  done
  echo '  </tbody>'
  echo '</table>'
  echo '<script>'
  echo '(() => {'
  echo '  const table = document.getElementById("feature-table");'
  echo '  if (!table) return;'
  echo '  const headers = Array.from(table.querySelectorAll("th[data-sort]"));'
  echo '  const tbody = table.tBodies[0];'
  echo '  const valueFor = (row, index, type) => {'
  echo '    const cell = row.cells[index];'
  echo '    if (type === "number") return Number(cell.dataset.value || cell.textContent.trim());'
  echo '    return cell.textContent.trim().toLocaleLowerCase();'
  echo '  };'
  echo '  headers.forEach((header, index) => {'
  echo '    const button = header.querySelector("button");'
  echo '    button.addEventListener("click", () => {'
  echo '      const nextDirection = header.getAttribute("aria-sort") === "ascending" ? "descending" : "ascending";'
  echo '      headers.forEach((item) => item.setAttribute("aria-sort", "none"));'
  echo '      header.setAttribute("aria-sort", nextDirection);'
  echo '      const type = header.dataset.sort;'
  echo '      const rows = Array.from(tbody.rows);'
  echo '      rows.sort((left, right) => {'
  echo '        const leftValue = valueFor(left, index, type);'
  echo '        const rightValue = valueFor(right, index, type);'
  echo '        let result;'
  echo '        if (type === "number") {'
  echo '          result = leftValue - rightValue;'
  echo '        } else {'
  echo '          result = leftValue.localeCompare(rightValue, undefined, { numeric: true });'
  echo '        }'
  echo '        return nextDirection === "ascending" ? result : -result;'
  echo '      });'
  echo '      rows.forEach((row) => tbody.appendChild(row));'
  echo '    });'
  echo '  });'
  echo '})();'
  echo '</script>'
  echo '</main>'
  echo '</html>'
} > "$status_html"
