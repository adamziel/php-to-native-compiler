#!/usr/bin/env bash

ptn_is_php_src_checkout() {
  local candidate=$1
  [[ -f "$candidate/run-tests.php" ]]
}

ptn_fetch_php_src_checkout() {
  local repo_root=$1
  local target=$2
  local url=${PTN_PHP_SRC_URL:-https://github.com/php/php-src.git}
  local ref=${PTN_PHP_SRC_REF:-}
  local tmp

  if [[ "${PTN_PHPT_AUTO_FETCH:-1}" == "0" ]]; then
    return 1
  fi

  if ! command -v git >/dev/null 2>&1; then
    return 1
  fi

  mkdir -p "$repo_root/.runtime"
  tmp=$(mktemp -d "$repo_root/.runtime/php-src-phpt.tmp.XXXXXX")

  echo "PHP PHPT corpus not found; cloning $url into $target" >&2
  if ! git clone --depth 1 "$url" "$tmp" >&2; then
    rm -rf "$tmp"
    return 1
  fi

  if [[ -n "$ref" ]]; then
    if ! git -C "$tmp" fetch --depth 1 origin "$ref" >&2; then
      rm -rf "$tmp"
      return 1
    fi
    if ! git -C "$tmp" checkout --detach FETCH_HEAD >&2; then
      rm -rf "$tmp"
      return 1
    fi
  fi

  if ! ptn_is_php_src_checkout "$tmp"; then
    echo "Cloned PHP source is missing run-tests.php: $tmp" >&2
    rm -rf "$tmp"
    return 1
  fi

  if [[ -e "$target" ]]; then
    echo "PHPT corpus cache path appeared during clone: $target" >&2
    rm -rf "$tmp"
    return 1
  fi

  mv "$tmp" "$target"
}

ptn_resolve_phpt_corpus() {
  local repo_root=$1
  local explicit=${PHP_SRC_PHPT:-}
  local home_default=/home/claude/php-src-phpt
  local cache_default="$repo_root/.runtime/php-src-phpt"
  local candidate

  if [[ -n "$explicit" ]]; then
    if ptn_is_php_src_checkout "$explicit"; then
      printf '%s\n' "$explicit"
      return 0
    fi
    echo "PHP_SRC_PHPT does not contain run-tests.php: $explicit" >&2
    return 2
  fi

  for candidate in "$home_default" "$cache_default"; do
    if ptn_is_php_src_checkout "$candidate"; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  if ptn_fetch_php_src_checkout "$repo_root" "$cache_default"; then
    printf '%s\n' "$cache_default"
    return 0
  fi

  cat >&2 <<EOF
PHP source checkout with run-tests.php not found.
Checked:
  $home_default
  $cache_default
Set PHP_SRC_PHPT to a php-src checkout, or allow the default .runtime cache fetch.
EOF
  return 2
}
