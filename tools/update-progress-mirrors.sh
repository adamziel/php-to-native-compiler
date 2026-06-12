#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

progress_file="PROGRESS.md"

if [[ ! -f "$progress_file" ]]; then
  echo "missing $progress_file" >&2
  exit 1
fi

trim() {
  local value="$1"
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

html_escape() {
  printf '%s' "$1" \
    | sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g'
}

refresh="$(awk -F': ' '/^Refresh:/ { print $2; exit }' "$progress_file")"
measured="$(awk '/^Measured:/ { sub(/^Measured:[[:space:]]*/, ""); print; exit }' "$progress_file")"

if [[ -z "$refresh" ]]; then
  echo "missing Refresh line in $progress_file" >&2
  exit 1
fi
if [[ -z "$measured" ]]; then
  measured="not recorded"
fi
measured_html="${measured//\`/}"

declare -a names=()
declare -a ported=()
declare -a passing=()
declare -a needs=()

while IFS='|' read -r _ raw_name raw_ported raw_passing raw_needs _; do
  name="$(trim "${raw_name:-}")"
  total="$(trim "${raw_ported:-}")"
  pass="$(trim "${raw_passing:-}")"
  need="$(trim "${raw_needs:-}")"

  [[ -n "$name" ]] || continue
  [[ "$name" != "Format / source" ]] || continue
  [[ "$name" != "---" ]] || continue
  [[ "$total" =~ ^[0-9]+$ ]] || continue
  [[ "$pass" =~ ^[0-9]+$ ]] || continue
  [[ "$need" =~ ^[0-9]+$ ]] || continue

  names+=("$name")
  ported+=("$total")
  passing+=("$pass")
  needs+=("$need")
done < <(awk '
  /^## Dashboard[[:space:]]*$/ { in_dashboard = 1; next }
  in_dashboard && /^## / { exit }
  in_dashboard && /^\|/ { print }
' "$progress_file")

if [[ "${#names[@]}" -eq 0 ]]; then
  echo "no dashboard rows found in $progress_file" >&2
  exit 1
fi

compact_signal() {
  local limit=6
  local signal=""
  local i

  if [[ "${#names[@]}" -lt "$limit" ]]; then
    limit="${#names[@]}"
  fi

  for ((i = 0; i < limit; i++)); do
    if [[ -n "$signal" ]]; then
      signal+="; "
    fi
    signal+="${names[$i]} ${passing[$i]}/${ported[$i]}"
  done

  printf '%s' "$signal"
}

{
  echo "# PTN Progress Mirror"
  echo
  echo "Last refresh: $refresh"
  echo 'Source: `PROGRESS.md`'
  echo "Measured: $measured"
  echo
  echo "Compact signal: $(compact_signal)."
  echo
  echo "| Format / source | Passing |"
  echo "| --- | ---: |"
  for ((i = 0; i < ${#names[@]}; i++)); do
    printf '| %s | %s/%s |\n' "${names[$i]}" "${passing[$i]}" "${ported[$i]}"
  done
  echo
  echo 'Canonical dashboard: `PROGRESS.md`. Regenerate with'
  echo '`tools/update-progress-mirrors.sh` after changing canonical progress.'
} > progress.md

{
  echo '<!doctype html>'
  echo '<meta charset="utf-8">'
  echo '<title>PTN Progress</title>'
  echo '<h1>PTN Progress</h1>'
  printf '<p><strong>Last refresh:</strong> %s<br>\n' "$(html_escape "$refresh")"
  printf '<strong>Measured:</strong> <code>%s</code></p>\n' "$(html_escape "$measured_html")"
  echo '<table>'
  echo '  <thead><tr><th>Format/source</th><th>Ported</th><th>Passing</th><th>Needs work</th></tr></thead>'
  echo '  <tbody>'
  for ((i = 0; i < ${#names[@]}; i++)); do
    printf '    <tr><td>%s</td><td>%s</td><td>%s</td><td>%s</td></tr>\n' \
      "$(html_escape "${names[$i]}")" \
      "$(html_escape "${ported[$i]}")" \
      "$(html_escape "${passing[$i]}")" \
      "$(html_escape "${needs[$i]}")"
  done
  echo '  </tbody>'
  echo '</table>'
  echo '<p>Canonical text dashboard lives in <code>PROGRESS.md</code>.'
  echo 'Regenerate mirrors with <code>tools/update-progress-mirrors.sh</code>.</p>'
} > progress.html

{
  echo "# PTN Status"
  echo
  echo "Last refresh: $refresh"
  echo "Measured: $measured"
  echo
  echo "## Operating Goal"
  echo
  echo "Hold the RC line to generic PHP semantics while expanding toward broad"
  echo "php-src PHPT coverage. Report numbers, not compatibility claims."
  echo
  echo "## Current Signal"
  echo
  echo "$(compact_signal)."
  echo
  echo "## Active Buckets"
  echo
  echo "| Bucket | Count |"
  echo "| --- | ---: |"
  for ((i = 0; i < ${#names[@]}; i++)); do
    printf '| %s | %s/%s |\n' "${names[$i]}" "${passing[$i]}" "${ported[$i]}"
  done
  echo
  echo "## Rules"
  echo
  echo "- Update \`PROGRESS.md\`, then run \`tools/update-progress-mirrors.sh\`."
  echo "- Keep mirrors compact and numeric."
  echo "- Never claim broad PHP compatibility from row-specific patches."
} > STATUS.md

{
  echo '<!doctype html>'
  echo '<meta charset="utf-8">'
  echo '<title>PTN Status</title>'
  echo '<h1>PTN Status</h1>'
  printf '<p><strong>Last refresh:</strong> %s<br>\n' "$(html_escape "$refresh")"
  printf '<strong>Measured:</strong> <code>%s</code></p>\n' "$(html_escape "$measured_html")"
  echo '<p>Hold the RC line to generic PHP semantics while expanding toward broad php-src PHPT coverage.</p>'
  echo '<table>'
  echo '  <thead><tr><th>Bucket</th><th>Count</th></tr></thead>'
  echo '  <tbody>'
  for ((i = 0; i < ${#names[@]}; i++)); do
    printf '    <tr><td>%s</td><td>%s/%s</td></tr>\n' \
      "$(html_escape "${names[$i]}")" \
      "$(html_escape "${passing[$i]}")" \
      "$(html_escape "${ported[$i]}")"
  done
  echo '  </tbody>'
  echo '</table>'
  echo '<p>Canonical text status lives in <code>STATUS.md</code>.</p>'
} > STATUS.html
