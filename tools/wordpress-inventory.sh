#!/usr/bin/env sh
set -eu

die() {
  printf '%s\n' "$*" >&2
  exit 1
}

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || die "wordpress-inventory: not inside a git repository"
cd "$repo_root"

wp_root=${1:-${WORDPRESS_ROOT:-}}
[ -n "$wp_root" ] || die "usage: tools/wordpress-inventory.sh /path/to/wordpress"
[ -d "$wp_root" ] || die "wordpress-inventory: missing directory: $wp_root"
[ -f "$wp_root/wp-settings.php" ] || die "wordpress-inventory: missing wp-settings.php in $wp_root"

phpc_bin=${PHPC_BIN:-$repo_root/target/debug/phpc}
if [ ! -x "$phpc_bin" ]; then
  phpc_bin="cargo run -q -p phpc --"
fi

count_php_files() {
  find "$wp_root" -type f -name '*.php' | wc -l | tr -d ' '
}

count_matching_files() {
  pattern=$1
  find "$wp_root" -type f -name '*.php' -exec grep -E -l "$pattern" {} + 2>/dev/null |
    wc -l |
    tr -d ' '
}

wp_version="unknown"
version_file="$wp_root/wp-includes/version.php"
if [ -f "$version_file" ]; then
  parsed_version=$(sed -n "s/^[[:space:]]*\\\$wp_version[[:space:]]*=[[:space:]]*['\"]\\([^'\"]*\\)['\"].*/\\1/p" "$version_file" | head -n 1)
  if [ -n "$parsed_version" ]; then
    wp_version=$parsed_version
  fi
fi

printf 'WordPress inventory\n'
printf 'root: %s\n' "$wp_root"
printf 'version: %s\n' "$wp_version"
printf 'php_files: %s\n' "$(count_php_files)"
printf '\n'
printf 'syntax_surface_files:\n'
printf '  include_require: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])(include|include_once|require|require_once)([[:space:]]|\()')"
printf '  namespaces: %s\n' "$(count_matching_files '^[[:space:]]*namespace[[:space:]]')"
printf '  imports: %s\n' "$(count_matching_files '^[[:space:]]*use[[:space:]]')"
printf '  interfaces: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])interface[[:space:]]+[A-Za-z_]')"
printf '  traits: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])trait[[:space:]]+[A-Za-z_]')"
printf '  enums: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])enum[[:space:]]+[A-Za-z_]')"
printf '  class_extends: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])class[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]+extends[[:space:]]')"
printf '  exceptions: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])(try|catch|finally|throw)[[:space:]({]')"
printf '  closures: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])function[[:space:]]*\\(')"
printf '  arrow_functions: %s\n' "$(count_matching_files '=>')"
printf '\n'

tmp_stdout=$(mktemp)
tmp_stderr=$(mktemp)
trap 'rm -f "$tmp_stdout" "$tmp_stderr"' EXIT

set +e
# shellcheck disable=SC2086
$phpc_bin run "$wp_root/wp-settings.php" >"$tmp_stdout" 2>"$tmp_stderr"
status=$?
set -e

printf 'bootstrap_probe:\n'
printf '  command: %s run %s/wp-settings.php\n' "$phpc_bin" "$wp_root"
printf '  exit: %s\n' "$status"
printf '  stdout_bytes: %s\n' "$(wc -c <"$tmp_stdout" | tr -d ' ')"
printf '  first_stderr_line: '
if [ -s "$tmp_stderr" ]; then
  sed -n '1p' "$tmp_stderr"
else
  printf '<none>\n'
fi
