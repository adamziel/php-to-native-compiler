#!/usr/bin/env sh
set -eu

die() {
  printf '%s\n' "$*" >&2
  exit 1
}

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || die "wordpress-inventory: not inside a git repository"
cd "$repo_root"

normalize=0
if [ "${1:-}" = "--normalize" ]; then
  normalize=1
  shift
fi

wp_root=${1:-${WORDPRESS_ROOT:-}}
[ -n "$wp_root" ] || die "usage: tools/wordpress-inventory.sh [--normalize] /path/to/wordpress"
[ -d "$wp_root" ] || die "wordpress-inventory: missing directory: $wp_root"
[ -f "$wp_root/wp-settings.php" ] || die "wordpress-inventory: missing wp-settings.php in $wp_root"

phpc_bin=${PHPC_BIN:-$repo_root/target/debug/phpc}
if [ ! -x "$phpc_bin" ]; then
  phpc_bin="cargo run -q -p phpc --"
fi
probe_timeout=${WORDPRESS_PROBE_TIMEOUT:-30s}
has_timeout=0
if command -v timeout >/dev/null 2>&1; then
  has_timeout=1
fi

display_root=$wp_root
display_phpc_bin=$phpc_bin
if [ "$normalize" -eq 1 ]; then
  display_root="<wordpress-root>"
  display_phpc_bin="<phpc>"
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
printf 'root: %s\n' "$display_root"
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
printf '  closures: %s\n' "$(count_matching_files '(^|[^[:alnum:]_])function[[:space:]]*[(]')"
printf '  arrow_functions: %s\n' "$(count_matching_files '=>')"
printf '\n'

tmp_stdout=$(mktemp)
tmp_stderr=$(mktemp)
tmp_shim=$(mktemp)
trap 'rm -f "$tmp_stdout" "$tmp_stderr" "$tmp_shim"' EXIT

run_probe() {
  probe_name=$1
  command_path=$2
  display_path=$3

  set +e
  if [ "$has_timeout" -eq 1 ]; then
    # shellcheck disable=SC2086
    timeout "$probe_timeout" $phpc_bin run "$command_path" >"$tmp_stdout" 2>"$tmp_stderr"
  else
    # shellcheck disable=SC2086
    $phpc_bin run "$command_path" >"$tmp_stdout" 2>"$tmp_stderr"
  fi
  status=$?
  set -e
  timed_out=no
  if [ "$has_timeout" -eq 1 ] && [ "$status" -eq 124 ]; then
    timed_out=yes
  fi

  printf '%s:\n' "$probe_name"
  printf '  command: %s run %s\n' "$display_phpc_bin" "$display_path"
  if [ "$has_timeout" -eq 1 ]; then
    printf '  timeout: %s\n' "$probe_timeout"
  else
    printf '  timeout: unavailable\n'
  fi
  printf '  timed_out: %s\n' "$timed_out"
  printf '  exit: %s\n' "$status"
  printf '  stdout_bytes: %s\n' "$(wc -c <"$tmp_stdout" | tr -d ' ')"
  printf '  first_stderr_line: '
  if [ -s "$tmp_stderr" ]; then
    first_stderr_line=$(sed -n '1p' "$tmp_stderr")
    if [ "$normalize" -eq 1 ]; then
      first_stderr_line=$(printf '%s\n' "$first_stderr_line" |
        sed "s#$wp_root#<wordpress-root>#g" |
        sed "s#$tmp_shim#<bootstrap-shim>#g")
    fi
    printf '%s\n' "$first_stderr_line"
  else
    printf '<none>\n'
  fi
  printf '  last_stderr_line: '
  if [ -s "$tmp_stderr" ]; then
    last_stderr_line=$(sed -n '$p' "$tmp_stderr")
    if [ "$normalize" -eq 1 ]; then
      last_stderr_line=$(printf '%s\n' "$last_stderr_line" |
        sed "s#$wp_root#<wordpress-root>#g" |
        sed "s#$tmp_shim#<bootstrap-shim>#g")
    fi
    printf '%s\n' "$last_stderr_line"
  else
    printf '<none>\n'
  fi
}

escaped_wp_root=$(printf '%s/' "$wp_root" | sed "s#//*#/#g; s#'#\\\\'#g")
cat >"$tmp_shim" <<EOF
<?php
define('ABSPATH', '$escaped_wp_root');
\$table_prefix = 'wp_';
require ABSPATH . 'wp-settings.php';
EOF

run_probe "direct_settings_probe" "$wp_root/wp-settings.php" "$display_root/wp-settings.php"
printf '\n'
if [ "$normalize" -eq 1 ]; then
  display_shim="<bootstrap-shim>"
else
  display_shim="$tmp_shim"
fi
run_probe "bootstrap_shim_probe" "$tmp_shim" "$display_shim"
if [ -f "$wp_root/wp-blog-header.php" ]; then
  printf '\n'
  run_probe "front_controller_probe" "$wp_root/wp-blog-header.php" "$display_root/wp-blog-header.php"
fi
