#!/usr/bin/env bash

# Shared PHPT preflight classification for PTN measurement runs.
#
# The classifier filters only PHPT harness/environment requirements that PTN
# does not currently model: extension availability, unsupported ini/runtime
# modes, unmodeled SAPI sections, external service harnesses, process-boundary
# rows, broad harness cleanup/setup sections, opt-in harness preconditions,
# noisy upstream rows, broad unsupported language surfaces, source-level
# runtime diagnostic APIs, and upstream XFAILs. Generic PHP semantic gaps
# inside the modeled surface remain runnable and should surface as PTN failures.

PTN_PHPT_SUPPORTED_EXTENSIONS_DEFAULT="bcmath,calendar,Core,ctype,curl,date,dom,filter,hash,iconv,intl,json,libxml,mbstring,mysqli,odbc,opcache,openssl,pcre,pdo,pdo_dblib,pdo_firebird,pdo_mysql,pdo_odbc,pdo_pgsql,pdo_sqlite,pgsql,Phar,phar,random,Reflection,session,simplexml,sockets,soap,SPL,sqlite3,standard,tokenizer,uri,xml,xmlreader,xmlwriter,zip,zend_test,zlib"
PTN_PHPT_SUPPORTED_INI_DEFAULT="allow_url_fopen,allow_url_include,always_populate_raw_post_data,arg_separator.input,arg_separator.output,assert.active,assert.bail,assert.callback,assert.exception,assert.warning,bcmath.scale,date.timezone,default_charset,detect_unicode,display_errors,html_errors,enable_post_data_reading,enable_dl,error_log,error_reporting,expose_php,extension_dir,file_uploads,filter.default,highlight.comment,highlight.default,highlight.html,highlight.keyword,highlight.string,iconv.input_encoding,iconv.internal_charset,iconv.internal_encoding,iconv.output_encoding,include_path,input_encoding,internal_encoding,intl.default_locale,intl.use_exceptions,mail.add_x_header,max_input_nesting_level,max_input_vars,max_memory_limit,mbstring.detect_order,mbstring.encoding_translation,mbstring.http_input,mbstring.http_output,mbstring.internal_encoding,mbstring.language,mbstring.regex_retry_limit,mbstring.regex_stack_limit,mbstring.strict_detection,mbstring.substitute_character,memory_limit,opcache.blacklist_filename,opcache.enable,opcache.enable_cli,opcache.fast_shutdown,opcache.file_cache,opcache.file_cache_only,opcache.file_update_protection,opcache.interned_strings_buffer,opcache.jit,opcache.jit_buffer_size,opcache.jit_hot_func,opcache.log_verbosity_level,opcache.opt_debug_level,opcache.optimization_level,opcache.preload,opcache.preload_user,opcache.protect_memory,opcache.revalidate_freq,opcache.revalidate_path,opcache.save_comments,opcache.validate_timestamps,open_basedir,output_encoding,output_handler,pcre.backtrack_limit,pcre.jit,pcre.recursion_limit,phar.cache_list,phar.readonly,phar.require_hash,post_max_size,precision,register_argc_argv,sendmail_path,serialize_precision,session.auto_start,session.cache_expire,session.cache_limiter,session.cookie_domain,session.cookie_httponly,session.cookie_lifetime,session.cookie_partitioned,session.cookie_path,session.cookie_samesite,session.cookie_secure,session.gc_divisor,session.gc_maxlifetime,session.gc_probability,session.lazy_write,session.name,session.referer_check,session.save_handler,session.save_path,session.serialize_handler,session.sid_bits_per_character,session.sid_length,session.trans_sid_hosts,session.upload_progress.cleanup,session.upload_progress.enabled,session.upload_progress.freq,session.upload_progress.min_freq,session.upload_progress.name,session.upload_progress.prefix,session.use_cookies,session.use_only_cookies,session.use_strict_mode,session.use_trans_sid,short_open_tag,soap.wsdl_cache_enabled,upload_tmp_dir,user_agent,variables_order,zend.assertions,zend.enable_gc,zend.exception_string_param_max_len,zend.multibyte,zend.script_encoding,zend_test.observer.enabled"
PTN_PHPT_UNSUPPORTED_SECTIONS_DEFAULT="CAPTURE_STDIO,COOKIE_RAW,EXPECTHEADERS,FILE_EXTERNAL,HEADERS,PHPDBG,PUT,REDIRECTTEST,REQUEST"
PTN_PHPT_ENVIRONMENT_SECTIONS_DEFAULT=""
PTN_PHPT_HARNESS_SECTIONS_DEFAULT=""
PTN_PHPT_NOISY_SECTIONS_DEFAULT="EXPECT_EXTERNAL,EXPECTF_EXTERNAL,EXPECTREGEX_EXTERNAL,FLAKY,WHITESPACE_SENSITIVE"
PTN_PHPT_SKIPIF_HARNESS_SECTIONS_DEFAULT="SKIPIF"

ptn_phpt_supported_extensions() {
    printf '%s\n' "${PTN_PHPT_SUPPORTED_EXTENSIONS:-$PTN_PHPT_SUPPORTED_EXTENSIONS_DEFAULT}"
}

ptn_phpt_supported_ini() {
    printf '%s\n' "${PTN_PHPT_SUPPORTED_INI:-$PTN_PHPT_SUPPORTED_INI_DEFAULT}"
}

ptn_phpt_unsupported_sections() {
    printf '%s\n' "${PTN_PHPT_UNSUPPORTED_SECTIONS:-$PTN_PHPT_UNSUPPORTED_SECTIONS_DEFAULT}"
}

ptn_phpt_environment_sections() {
    printf '%s\n' "${PTN_PHPT_ENVIRONMENT_SECTIONS:-$PTN_PHPT_ENVIRONMENT_SECTIONS_DEFAULT}"
}

ptn_phpt_harness_sections() {
    printf '%s\n' "${PTN_PHPT_HARNESS_SECTIONS:-$PTN_PHPT_HARNESS_SECTIONS_DEFAULT}"
}

ptn_phpt_noisy_sections() {
    printf '%s\n' "${PTN_PHPT_NOISY_SECTIONS:-$PTN_PHPT_NOISY_SECTIONS_DEFAULT}"
}

ptn_phpt_skipif_harness_sections() {
    printf '%s\n' "${PTN_PHPT_SKIPIF_HARNESS_SECTIONS:-$PTN_PHPT_SKIPIF_HARNESS_SECTIONS_DEFAULT}"
}

ptn_phpt_classify_harness_programs() {
    [[ "${PTN_PHPT_CLASSIFY_HARNESS_PROGRAMS:-0}" == "1" ]]
}

ptn_phpt_php_int_size() {
    if [[ -n "${PTN_PHPT_PHP_INT_SIZE:-}" ]]; then
        printf '%s\n' "$PTN_PHPT_PHP_INT_SIZE"
        return 0
    fi

    local bits
    bits=$(getconf LONG_BIT 2>/dev/null || printf '64')
    if [[ "$bits" -ge 64 ]]; then
        printf '8\n'
    else
        printf '4\n'
    fi
}

ptn_phpt_available_locales() {
    if [[ -n "${PTN_PHPT_AVAILABLE_LOCALES:-}" ]]; then
        printf '%s\n' "$PTN_PHPT_AVAILABLE_LOCALES" | tr ',:' '\n'
        return 0
    fi

    locale -a 2>/dev/null || printf 'C\nPOSIX\n'
}

ptn_phpt_lower() {
    local value=$1
    printf '%s' "${value,,}"
}

ptn_phpt_trim() {
    local value=$1
    value=${value#"${value%%[![:space:]]*}"}
    value=${value%"${value##*[![:space:]]}"}
    printf '%s' "$value"
}

declare -gA PTN_PHPT_SECTION_CACHE_KEY_BY_PATH=()
declare -gA PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH=()
declare -g PTN_PHPT_MODELED_FUNCTIONS_LOADED=0
declare -gA PTN_PHPT_MODELED_FUNCTIONS_BY_NAME=()

ptn_phpt_repo_root() {
    cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd
}

ptn_phpt_normalized_function_name() {
    local value
    value=$(ptn_phpt_trim "$1")
    value=${value#\\}
    ptn_phpt_lower "$value"
}

ptn_phpt_load_modeled_functions() {
    [[ "$PTN_PHPT_MODELED_FUNCTIONS_LOADED" -eq 0 ]] || return 0

    local repo_root
    local registry
    local function_name
    repo_root=$(ptn_phpt_repo_root)
    registry="$repo_root/src/backend/runtime/internals_internal_functions.c"

    if [[ -r "$registry" ]]; then
        while IFS= read -r function_name; do
            function_name=$(ptn_phpt_normalized_function_name "$function_name")
            [[ -n "$function_name" ]] || continue
            [[ "$function_name" != *::* ]] || continue
            PTN_PHPT_MODELED_FUNCTIONS_BY_NAME[$function_name]=1
        done < <(
            LC_ALL=C sed -nE 's/^[[:space:]]*\{[[:space:]]*"([^"]+)"[[:space:]]*,.*/\1/p' "$registry"
        )
    fi

    PTN_PHPT_MODELED_FUNCTIONS_LOADED=1
}

ptn_phpt_modeled_function_exists() {
    local function_name
    function_name=$(ptn_phpt_normalized_function_name "$1")
    [[ -n "$function_name" ]] || return 1

    if [[ -n "${PTN_PHPT_AVAILABLE_FUNCTIONS:-}" ]]; then
        ptn_phpt_csv_contains_ci "$function_name" "$PTN_PHPT_AVAILABLE_FUNCTIONS"
        return $?
    fi

    ptn_phpt_load_modeled_functions
    [[ -v "PTN_PHPT_MODELED_FUNCTIONS_BY_NAME[$function_name]" ]]
}

ptn_phpt_build_section_cache() {
    local manifest=$1
    local php_src=$2
    local cache_dir=$3
    local index_file="$cache_dir/index.tsv"

    mkdir -p "$cache_dir"
    : > "$index_file"

    LC_ALL=C awk -v php_src="$php_src" -v cache_dir="$cache_dir" -v index_file="$index_file" '
        function trim(value) {
            sub(/^[[:space:]]+/, "", value)
            sub(/[[:space:]]+$/, "", value)
            return value
        }
        function section_name(header, value) {
            value = header
            sub(/^--/, "", value)
            sub(/--[[:space:]]*$/, "", value)
            return value
        }
        function append_section_name(section) {
            if (sections_csv == "") {
                sections_csv = section
            } else {
                sections_csv = sections_csv "," section
            }
        }
        function scan_phpt(path,    status, line, section) {
            sections_csv = ""
            while ((status = getline line < path) > 0) {
                if (line ~ /^--[A-Z0-9_]+--[[:space:]]*$/) {
                    section = section_name(line)
                    append_section_name(section)
                }
            }
            if (status < 0) {
                printf "could not read PHPT row for section cache: %s\n", path > "/dev/stderr"
                exit 1
            }
            close(path)
        }
        {
            raw = trim($0)
            if (raw == "" || raw ~ /^#/) {
                next
            }
            row = raw
            sub(/#.*/, "", row)
            row = trim(row)
            if (row == "") {
                next
            }
            path = row
            if (substr(row, 1, 1) != "/") {
                path = php_src "/" row
            }
            key++
            scan_phpt(path)
            printf "%s\t%s\t%s\t%s\n", row, path, key, sections_csv >> index_file
        }
    ' "$manifest"
}

ptn_phpt_load_section_cache_index() {
    local index=$1
    local row
    local path
    local key
    local sections

    PTN_PHPT_SECTION_CACHE_KEY_BY_PATH=()
    PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH=()

    while IFS=$'\t' read -r row path key sections; do
        [[ -n "$path" && -n "$key" ]] || continue
        PTN_PHPT_SECTION_CACHE_KEY_BY_PATH[$path]=$key
        PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH[$path]=${sections:-}
    done < "$index"
}

ptn_phpt_section_cache_key() {
    local path=$1

    [[ -n "${PTN_PHPT_SECTION_CACHE_DIR:-}" ]] || return 1
    if [[ -v "PTN_PHPT_SECTION_CACHE_KEY_BY_PATH[$path]" ]]; then
        printf '%s\n' "${PTN_PHPT_SECTION_CACHE_KEY_BY_PATH[$path]}"
        return 0
    fi
    return 1
}

ptn_phpt_csv_contains_ci() {
    local needle
    needle=$(ptn_phpt_lower "$(ptn_phpt_trim "$1")")
    local csv=$2
    local item
    local old_ifs=$IFS

    IFS=,
    for item in $csv; do
        item=$(ptn_phpt_lower "$(ptn_phpt_trim "$item")")
        if [[ "$item" == "$needle" ]]; then
            IFS=$old_ifs
            return 0
        fi
    done
    IFS=$old_ifs
    return 1
}

ptn_phpt_section() {
    local path=$1
    local target=$2

    awk -v target="$target" '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            active = section == target
            next
        }
        active { print }
    ' "$path"
}

ptn_phpt_has_section() {
    local path=$1
    local target=$2
    local sections

    if [[ -n "${PTN_PHPT_SECTION_CACHE_DIR:-}" && -v "PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH[$path]" ]]; then
        sections=${PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH[$path]}
        ptn_phpt_csv_contains_ci "$target" "$sections"
        return $?
    fi

    awk -v target="$target" '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            if (section == target) {
                found = 1
                exit
            }
        }
        END { exit found ? 0 : 1 }
    ' "$path"
}

ptn_phpt_sections() {
    local path=$1
    local sections
    local section
    local old_ifs

    if [[ -n "${PTN_PHPT_SECTION_CACHE_DIR:-}" && -v "PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH[$path]" ]]; then
        sections=${PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH[$path]}
        old_ifs=$IFS
        IFS=,
        for section in $sections; do
            printf '%s\n' "$section"
        done
        IFS=$old_ifs
        return 0
    fi

    awk '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            print section
        }
    ' "$path"
}

ptn_phpt_sections_csv() {
    local path=$1
    local -a sections=()
    local section

    if [[ -n "${PTN_PHPT_SECTION_CACHE_DIR:-}" && -v "PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH[$path]" ]]; then
        printf '%s\n' "${PTN_PHPT_SECTION_CACHE_SECTIONS_BY_PATH[$path]}"
        return 0
    fi

    while IFS= read -r section; do
        sections+=("$section")
    done < <(ptn_phpt_sections "$path")

    local old_ifs=$IFS
    IFS=,
    printf '%s\n' "${sections[*]}"
    IFS=$old_ifs
}

ptn_phpt_first_section_in_sections_csv() {
    local sections_csv=$1
    local search_csv=$2
    local section
    local old_ifs=$IFS

    IFS=,
    for section in $sections_csv; do
        if ptn_phpt_csv_contains_ci "$section" "$search_csv"; then
            IFS=$old_ifs
            printf '%s\n' "$section"
            return 0
        fi
    done
    IFS=$old_ifs
    return 1
}

ptn_phpt_squash_ws() {
    sed -E 's/[[:space:]]+/ /g; s/^ //; s/ $//'
}

ptn_phpt_skipif_code() {
    local path=$1

    ptn_phpt_section "$path" SKIPIF \
        | tr '\n' ' ' \
        | sed -E 's/<\?php//g; s/\?>//g' \
        | ptn_phpt_squash_ws
}

ptn_phpt_strip_php_strings() {
    LC_ALL=C awk '
        {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length($0); i++) {
                ch = substr($0, i, 1)
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                out = out ch
            }
            print out
        }
    '
}

ptn_phpt_php_string_literals() {
    LC_ALL=C awk '
        {
            quote = ""
            escaped = 0
            value = ""
            for (i = 1; i <= length($0); i++) {
                ch = substr($0, i, 1)
                if (quote != "") {
                    if (escaped) {
                        value = value ch
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        print value
                        quote = ""
                        value = ""
                    } else {
                        value = value ch
                    }
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                }
            }
        }
    '
}

ptn_phpt_count_matches() {
    local pattern=$1
    local text=$2
    local matches
    matches=$(printf '%s\n' "$text" | grep -Eo "$pattern" || true)
    if [[ -z "$matches" ]]; then
        printf '0\n'
    else
        printf '%s\n' "$matches" | wc -l | tr -d ' '
    fi
}

ptn_phpt_eval_int_condition() {
    local actual=$1
    local op=$2
    local expected=$3

    case "$op" in
        '!='|'!==') [[ "$actual" -ne "$expected" ]] ;;
        '=='|'===') [[ "$actual" -eq "$expected" ]] ;;
        '<') [[ "$actual" -lt "$expected" ]] ;;
        '<=') [[ "$actual" -le "$expected" ]] ;;
        '>') [[ "$actual" -gt "$expected" ]] ;;
        '>=') [[ "$actual" -ge "$expected" ]] ;;
        *) return 1 ;;
    esac
}

ptn_phpt_php_int_max() {
    if [[ "$(ptn_phpt_php_int_size)" -ge 8 ]]; then
        printf '9223372036854775807\n'
    else
        printf '2147483647\n'
    fi
}

ptn_phpt_php_os_family() {
    if [[ -n "${PTN_PHPT_PHP_OS_FAMILY:-}" ]]; then
        printf '%s\n' "$PTN_PHPT_PHP_OS_FAMILY"
        return 0
    fi

    local kernel
    kernel=$(uname -s 2>/dev/null || printf 'Unknown')
    case "$kernel" in
        Linux) printf 'Linux\n' ;;
        Darwin) printf 'Darwin\n' ;;
        FreeBSD|NetBSD|OpenBSD|DragonFly) printf 'BSD\n' ;;
        SunOS) printf 'Solaris\n' ;;
        CYGWIN*|MINGW*|MSYS*|Windows_NT) printf 'Windows\n' ;;
        *) printf 'Unknown\n' ;;
    esac
}

ptn_phpt_php_os() {
    if [[ -n "${PTN_PHPT_PHP_OS:-}" ]]; then
        printf '%s\n' "$PTN_PHPT_PHP_OS"
        return 0
    fi
    uname -s 2>/dev/null || printf 'Unknown\n'
}

ptn_phpt_php_debug() {
    printf '%s\n' "${PTN_PHPT_PHP_DEBUG:-0}"
}

ptn_phpt_php_zts() {
    printf '%s\n' "${PTN_PHPT_PHP_ZTS:-0}"
}

ptn_phpt_effective_uid() {
    if [[ -n "${PTN_PHPT_EFFECTIVE_UID:-}" ]]; then
        printf '%s\n' "$PTN_PHPT_EFFECTIVE_UID"
        return 0
    fi

    id -u 2>/dev/null || printf '1\n'
}

ptn_phpt_php_truthy() {
    local value=$1
    [[ -n "$value" && "$value" != "0" ]]
}

ptn_phpt_run_slow_tests() {
    ptn_phpt_php_truthy "${PTN_PHPT_RUN_SLOW_TESTS:-0}"
}

ptn_phpt_run_perf_sensitive_tests() {
    ptn_phpt_php_truthy "${PTN_PHPT_RUN_PERF_SENSITIVE:-0}"
}

ptn_phpt_default_runnable_resource_limit_skipif() {
    local rel=$1
    local env_var=$2

    case "$env_var:$rel" in
        SKIP_SLOW_TESTS:ext/standard/tests/file/001.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/bug36365.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/file_get_contents_error001.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_basic.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation10.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation11.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation13.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation16.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation21.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation4.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation5.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/lstat_stat_variation8.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/file/touch_basic.phpt|\
        SKIP_SLOW_TESTS:ext/date/tests/bug73837.phpt|\
        SKIP_SLOW_TESTS:ext/pcre/tests/bug69864.phpt|\
        SKIP_SLOW_TESTS:ext/pcre/tests/cache_limit.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/general_functions/sleep_basic.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/general_functions/usleep_basic.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/network/gethostbyname_basic001.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/password/password_hash.phpt|\
        SKIP_SLOW_TESTS:ext/standard/tests/password/password_removed_salt_option.phpt)
            return 0
            ;;
    esac

    return 1
}

ptn_phpt_php_constant_defined() {
    local constant=$1

    if [[ -n "${PTN_PHPT_DEFINED_CONSTANTS:-}" ]]; then
        ptn_phpt_csv_contains_ci "$constant" "$PTN_PHPT_DEFINED_CONSTANTS"
        return
    fi

    case "$constant" in
        GLOB_BRACE)
            if command -v php >/dev/null 2>&1; then
                [[ "$(php -r "echo defined('$constant') ? '1' : '0';" 2>/dev/null)" == "1" ]]
                return
            fi
            return 1
            ;;
    esac

    return 1
}

ptn_phpt_eval_string_condition() {
    local actual=$1
    local op=$2
    local expected=$3

    case "$op" in
        '!='|'!==') [[ "$actual" != "$expected" ]] ;;
        '=='|'===') [[ "$actual" == "$expected" ]] ;;
        *) return 1 ;;
    esac
}

ptn_phpt_normalized_locale_name() {
    local value
    value=$(ptn_phpt_lower "$(ptn_phpt_trim "$1")")
    value=${value//utf-8/utf8}
    value=${value//-/_}
    printf '%s\n' "$value"
}

ptn_phpt_locale_candidate_available() {
    local candidate=$1
    local normalized_candidate
    normalized_candidate=$(ptn_phpt_normalized_locale_name "$candidate")
    [[ -n "$normalized_candidate" ]] || return 1

    local available
    local normalized_available
    while IFS= read -r available; do
        normalized_available=$(ptn_phpt_normalized_locale_name "$available")
        [[ -n "$normalized_available" ]] || continue
        if [[ "$normalized_available" == "$normalized_candidate" ]]; then
            return 0
        fi
        if [[ "$normalized_candidate" != *_* && "$normalized_available" == "$normalized_candidate"_* ]]; then
            return 0
        fi
    done < <(ptn_phpt_available_locales)

    return 1
}

ptn_phpt_skipif_locale_candidates() {
    local code=$1
    local literal

    printf '%s\n' "$code" | ptn_phpt_php_string_literals | while IFS= read -r literal; do
        literal=$(ptn_phpt_trim "$literal")
        [[ -n "$literal" ]] || continue
        case "$(ptn_phpt_lower "$literal")" in
            invalid|skip*|xleak*) continue ;;
        esac
        if [[ "$literal" =~ [[:space:]] ]]; then
            continue
        fi
        printf '%s\n' "$literal"
    done
}

ptn_phpt_modeled_skipif_precondition() {
    local path=$1
    local rel=${2:-$1}
    local code
    local code_without_strings
    local code_for_identifiers
    code=$(ptn_phpt_skipif_code "$path")
    [[ -n "$code" ]] || return 1

    local inactive_windows_helper_count=0
    if [[ "$(ptn_phpt_php_os_family)" != "Windows" ]] \
        && printf '%s\n' "$code" | grep -Eq "if[[:space:]]*\\([[:space:]]*PHP_OS_FAMILY[[:space:]]*(===|==)[[:space:]]*['\"]Windows['\"][[:space:]]*\\)[[:space:]]*\\{[^{}]*skipIfSeCreateSymbolicLinkPrivilegeIsDisabled[^{}]*\\}"; then
        inactive_windows_helper_count=$(ptn_phpt_count_matches 'skipIfSeCreateSymbolicLinkPrivilegeIsDisabled[[:space:]]*\(' "$code")
        code=$(printf '%s\n' "$code" \
            | sed -E "s/if[[:space:]]*\\([[:space:]]*PHP_OS_FAMILY[[:space:]]*(===|==)[[:space:]]*['\"]Windows['\"][[:space:]]*\\)[[:space:]]*\\{[^{}]*skipIfSeCreateSymbolicLinkPrivilegeIsDisabled[^{}]*\\}/ /g" \
            | ptn_phpt_squash_ws)
    fi

    code_without_strings=$(printf '%s\n' "$code" | ptn_phpt_strip_php_strings | ptn_phpt_squash_ws)
    code_for_identifiers=$(printf '%s\n' "$code_without_strings" | sed -E 's/\$[A-Za-z_][A-Za-z0-9_]*/ /g')

    local identifier
    while IFS= read -r identifier; do
        [[ -n "$identifier" ]] || continue
        case "$identifier" in
            if|getenv|die|exit|echo|print|require|require_once|include|include_once|defined|function_exists|in_array|stream_get_filters|__DIR__|PHP_INT_SIZE|PHP_INT_MAX|PHP_OS_FAMILY|PHP_OS|PHP_DEBUG|PHP_ZTS|PHP_VERSION|version_compare|substr|setlocale|LC_ALL|LC_COLLATE|LC_CTYPE|LC_MESSAGES|LC_MONETARY|LC_NUMERIC|LC_TIME)
                ;;
            *)
                return 1
                ;;
        esac
    done < <(printf '%s\n' "$code_for_identifiers" | grep -Eo '[A-Za-z_][A-Za-z0-9_]*' || true)

    local if_count
    local output_count
    local getenv_count
    local php_int_count
    local php_int_max_count
    local os_family_count
    local php_os_count
    local php_debug_count
    local php_zts_count
    local setlocale_count
    local defined_count
    local function_exists_count
    local in_array_count
    local stream_get_filters_count
    local include_count
    if_count=$(ptn_phpt_count_matches '(^|[^A-Za-z0-9_])if[[:space:]]*\(' "$code_without_strings")
    output_count=$(ptn_phpt_count_matches '(^|[^A-Za-z0-9_])(die|exit|echo|print)([[:space:]]*\(|[[:space:]])' "$code_without_strings")
    getenv_count=$(ptn_phpt_count_matches 'getenv[[:space:]]*\(' "$code_without_strings")
    php_int_count=$(ptn_phpt_count_matches 'PHP_INT_SIZE' "$code_without_strings")
    php_int_max_count=$(ptn_phpt_count_matches 'PHP_INT_MAX' "$code_without_strings")
    os_family_count=$(ptn_phpt_count_matches 'PHP_OS_FAMILY' "$code_without_strings")
    php_os_count=$(ptn_phpt_count_matches 'PHP_OS([^_A-Za-z0-9]|$)' "$code_without_strings")
    php_debug_count=$(ptn_phpt_count_matches 'PHP_DEBUG' "$code_without_strings")
    php_zts_count=$(ptn_phpt_count_matches 'PHP_ZTS' "$code_without_strings")
    setlocale_count=$(ptn_phpt_count_matches 'setlocale[[:space:]]*\(' "$code_without_strings")
    defined_count=$(ptn_phpt_count_matches 'defined[[:space:]]*\(' "$code_without_strings")
    function_exists_count=$(ptn_phpt_count_matches 'function_exists[[:space:]]*\(' "$code_without_strings")
    in_array_count=$(ptn_phpt_count_matches 'in_array[[:space:]]*\(' "$code_without_strings")
    stream_get_filters_count=$(ptn_phpt_count_matches 'stream_get_filters[[:space:]]*\(' "$code_without_strings")
    include_count=$(ptn_phpt_count_matches '(^|[^A-Za-z0-9_])(require|include)(_once)?[[:space:]]+' "$code_without_strings")

    local env_probe_lines
    local parsed_env_count=0
    local env_family="sanitizer-env"
    env_probe_lines=$(printf '%s\n' "$code" \
        | grep -Eo "getenv[[:space:]]*\\([[:space:]]*['\"][A-Za-z_][A-Za-z0-9_]*['\"][[:space:]]*\\)" \
        || true)
    local env_var
    local -a modeled_families=()
    if [[ -n "$env_probe_lines" ]]; then
        while IFS= read -r env_var; do
            env_var=$(printf '%s\n' "$env_var" | sed -E "s/.*['\"]([^'\"]+)['\"].*/\\1/")
            case "$env_var" in
                SKIP_ASAN|SKIP_MSAN|SKIP_UBSAN)
                    ;;
                SKIP_PERF_SENSITIVE)
                    env_family="resource-limit"
                    ;;
                SKIP_SLOW_TESTS)
                    env_family="resource-limit"
                    ;;
                SKIP_*|USE_ZEND_ALLOC|USE_TRACKED_ALLOC|RUN_RESOURCE_HEAVY_TESTS|STACK_LIMIT_DEFAULTS_CHECK|CIRRUS_CI)
                    env_family="environment"
                    ;;
                *)
                    return 1
                    ;;
            esac
            parsed_env_count=$((parsed_env_count + 1))
            local env_value=${!env_var:-}
            local env_is_truthy=0
            local env_is_modeled_resource_limit=0
            case "$env_var" in
                SKIP_PERF_SENSITIVE)
                    if ! ptn_phpt_run_perf_sensitive_tests \
                        && ! ptn_phpt_default_runnable_resource_limit_skipif "$rel" "$env_var"; then
                        env_is_truthy=1
                        env_is_modeled_resource_limit=1
                    fi
                    ;;
                SKIP_SLOW_TESTS)
                    if ! ptn_phpt_run_slow_tests \
                        && ! ptn_phpt_default_runnable_resource_limit_skipif "$rel" "$env_var"; then
                        env_is_truthy=1
                        env_is_modeled_resource_limit=1
                    fi
                    ;;
            esac
            if ptn_phpt_php_truthy "$env_value"; then
                env_is_truthy=1
            fi
            if printf '%s\n' "$code" | grep -Eq "![[:space:]]*getenv[[:space:]]*\\([[:space:]]*['\"]$env_var['\"][[:space:]]*\\)"; then
                if [[ "$env_is_truthy" -eq 0 ]]; then
                    printf 'skipif-precondition\tmodeled static --SKIPIF-- environment gate requires %s set; current environment leaves it unset\n' "$env_var"
                    return 0
                fi
            elif [[ "$env_is_truthy" -eq 1 ]]; then
                if [[ "$env_is_modeled_resource_limit" -eq 1 ]]; then
                    printf 'skipif-precondition\tmodeled static --SKIPIF-- resource-limit gate keeps %s rows out of default PTN sweeps; set PTN_PHPT_RUN_SLOW_TESTS=1 or PTN_PHPT_RUN_PERF_SENSITIVE=1 to opt in\n' "$env_var"
                    return 0
                fi
                printf 'skipif-precondition\tmodeled static --SKIPIF-- environment gate requires %s unset; current environment sets it\n' "$env_var"
                return 0
            fi
        done <<< "$env_probe_lines"
        modeled_families+=("$env_family")
    fi
    [[ "$getenv_count" -eq "$parsed_env_count" ]] || return 1

    local int_condition_lines
    local parsed_int_count=0
    local int_size
    int_size=$(ptn_phpt_php_int_size)
    int_condition_lines=$(printf '%s\n' "$code" \
        | grep -Eo 'PHP_INT_SIZE[[:space:]]*(===|!==|==|!=|<=|>=|<|>)[[:space:]]*[0-9]+' \
        || true)
    local condition
    local op
    local expected
    if [[ -n "$int_condition_lines" ]]; then
        while IFS= read -r condition; do
            op=$(printf '%s\n' "$condition" | sed -E 's/PHP_INT_SIZE[[:space:]]*(===|!==|==|!=|<=|>=|<|>)[[:space:]]*([0-9]+)/\1/')
            expected=$(printf '%s\n' "$condition" | sed -E 's/PHP_INT_SIZE[[:space:]]*(===|!==|==|!=|<=|>=|<|>)[[:space:]]*([0-9]+)/\2/')
            parsed_int_count=$((parsed_int_count + 1))
            if ptn_phpt_eval_int_condition "$int_size" "$op" "$expected"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_INT_SIZE guard skips when PHP_INT_SIZE %s %s; modeled PHP_INT_SIZE=%s\n' \
                    "$op" "$expected" "$int_size"
                return 0
            fi
        done <<< "$int_condition_lines"
        modeled_families+=("PHP_INT_SIZE")
    fi
    [[ "$php_int_count" -eq "$parsed_int_count" ]] || return 1

    local int_max_condition_lines
    local parsed_int_max_count=0
    local int_max
    int_max=$(ptn_phpt_php_int_max)
    int_max_condition_lines=$(printf '%s\n' "$code" \
        | grep -Eo 'PHP_INT_MAX[[:space:]]*(===|!==|==|!=|<=|>=|<|>)[[:space:]]*[0-9]+' \
        || true)
    if [[ -n "$int_max_condition_lines" ]]; then
        while IFS= read -r condition; do
            op=$(printf '%s\n' "$condition" | sed -E 's/PHP_INT_MAX[[:space:]]*(===|!==|==|!=|<=|>=|<|>)[[:space:]]*([0-9]+)/\1/')
            expected=$(printf '%s\n' "$condition" | sed -E 's/PHP_INT_MAX[[:space:]]*(===|!==|==|!=|<=|>=|<|>)[[:space:]]*([0-9]+)/\2/')
            parsed_int_max_count=$((parsed_int_max_count + 1))
            if ptn_phpt_eval_int_condition "$int_max" "$op" "$expected"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_INT_MAX guard skips when PHP_INT_MAX %s %s; modeled PHP_INT_MAX=%s\n' \
                    "$op" "$expected" "$int_max"
                return 0
            fi
        done <<< "$int_max_condition_lines"
        modeled_families+=("PHP_INT_MAX")
    fi
    [[ "$php_int_max_count" -eq "$parsed_int_max_count" ]] || return 1

    local string_condition_lines
    local parsed_os_family_count=0
    local os_family
    os_family=$(ptn_phpt_php_os_family)
    string_condition_lines=$(printf '%s\n' "$code" \
        | grep -Eo "PHP_OS_FAMILY[[:space:]]*(===|!==|==|!=)[[:space:]]*['\"][A-Za-z0-9_ -]+['\"]" \
        || true)
    if [[ -n "$string_condition_lines" ]]; then
        while IFS= read -r condition; do
            op=$(printf '%s\n' "$condition" | sed -E "s/PHP_OS_FAMILY[[:space:]]*(===|!==|==|!=)[[:space:]]*['\"][^'\"]+['\"]/\1/")
            expected=$(printf '%s\n' "$condition" | sed -E "s/PHP_OS_FAMILY[[:space:]]*(===|!==|==|!=)[[:space:]]*['\"]([^'\"]+)['\"]/\2/")
            parsed_os_family_count=$((parsed_os_family_count + 1))
            if ptn_phpt_eval_string_condition "$os_family" "$op" "$expected"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_OS_FAMILY guard skips when PHP_OS_FAMILY %s %s; modeled PHP_OS_FAMILY=%s\n' \
                    "$op" "$expected" "$os_family"
                return 0
            fi
        done <<< "$string_condition_lines"
        modeled_families+=("PHP_OS_FAMILY")
    fi
    [[ "$os_family_count" -eq "$parsed_os_family_count" ]] || return 1

    local parsed_php_os_count=0
    local php_os
    php_os=$(ptn_phpt_php_os)
    string_condition_lines=$(printf '%s\n' "$code" \
        | grep -Eo "substr[[:space:]]*\\([[:space:]]*PHP_OS[[:space:]]*,[[:space:]]*0[[:space:]]*,[[:space:]]*[0-9]+[[:space:]]*\\)[[:space:]]*(===|!==|==|!=)[[:space:]]*['\"][A-Za-z0-9_ -]+['\"]" \
        || true)
    if [[ -n "$string_condition_lines" ]]; then
        while IFS= read -r condition; do
            local length
            length=$(printf '%s\n' "$condition" | sed -E "s/substr[[:space:]]*\\([[:space:]]*PHP_OS[[:space:]]*,[[:space:]]*0[[:space:]]*,[[:space:]]*([0-9]+)[[:space:]]*\\).*/\1/")
            op=$(printf '%s\n' "$condition" | sed -E "s/.*\\)[[:space:]]*(===|!==|==|!=)[[:space:]]*['\"][^'\"]+['\"]/\1/")
            expected=$(printf '%s\n' "$condition" | sed -E "s/.*\\)[[:space:]]*(===|!==|==|!=)[[:space:]]*['\"]([^'\"]+)['\"]/\2/")
            parsed_php_os_count=$((parsed_php_os_count + 1))
            if ptn_phpt_eval_string_condition "${php_os:0:length}" "$op" "$expected"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_OS prefix guard skips when substr(PHP_OS, 0, %s) %s %s; modeled PHP_OS=%s\n' \
                    "$length" "$op" "$expected" "$php_os"
                return 0
            fi
        done <<< "$string_condition_lines"
        modeled_families+=("PHP_OS-prefix")
    fi
    [[ "$php_os_count" -eq "$parsed_php_os_count" ]] || return 1

    local parsed_debug_count=0
    local php_debug
    php_debug=$(ptn_phpt_php_debug)
    if [[ "$php_debug_count" -gt 0 ]]; then
        if printf '%s\n' "$code_without_strings" | grep -Eq '![[:space:]]*PHP_DEBUG'; then
            parsed_debug_count=$((parsed_debug_count + 1))
            if ! ptn_phpt_php_truthy "$php_debug"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_DEBUG guard skips when PHP_DEBUG is false\n'
                return 0
            fi
        fi
        if printf '%s\n' "$code_without_strings" | grep -Eq '(^|[^!A-Za-z0-9_])PHP_DEBUG([^A-Za-z0-9_]|$)'; then
            parsed_debug_count=$((parsed_debug_count + 1))
            if ptn_phpt_php_truthy "$php_debug"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_DEBUG guard skips when PHP_DEBUG is true\n'
                return 0
            fi
        fi
        modeled_families+=("PHP_DEBUG")
    fi
    [[ "$php_debug_count" -eq "$parsed_debug_count" ]] || return 1

    local parsed_zts_count=0
    local php_zts
    php_zts=$(ptn_phpt_php_zts)
    if [[ "$php_zts_count" -gt 0 ]]; then
        if printf '%s\n' "$code_without_strings" | grep -Eq '![[:space:]]*PHP_ZTS'; then
            parsed_zts_count=$((parsed_zts_count + 1))
            if ! ptn_phpt_php_truthy "$php_zts"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_ZTS guard skips when PHP_ZTS is false\n'
                return 0
            fi
        fi
        if printf '%s\n' "$code_without_strings" | grep -Eq '(^|[^!A-Za-z0-9_])PHP_ZTS([^A-Za-z0-9_]|$)'; then
            parsed_zts_count=$((parsed_zts_count + 1))
            if ptn_phpt_php_truthy "$php_zts"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- PHP_ZTS guard skips when PHP_ZTS is true\n'
                return 0
            fi
        fi
        modeled_families+=("PHP_ZTS")
    fi
    [[ "$php_zts_count" -eq "$parsed_zts_count" ]] || return 1

    local locale_invalid_count
    local locale_availability_count
    local parsed_locale_count=0
    locale_invalid_count=$(ptn_phpt_count_matches "setlocale[[:space:]]*\\([[:space:]]*LC_ALL[[:space:]]*,[[:space:]]*['\"]invalid['\"][[:space:]]*\\)[[:space:]]*===[[:space:]]*['\"]invalid['\"]" "$code")
    locale_availability_count=$(ptn_phpt_count_matches '![[:space:]]*setlocale[[:space:]]*\([[:space:]]*LC_ALL[[:space:]]*,' "$code_without_strings")
    if [[ "$locale_invalid_count" -gt 0 ]]; then
        parsed_locale_count=$((parsed_locale_count + locale_invalid_count))
        if ptn_phpt_locale_candidate_available invalid; then
            printf 'skipif-precondition\tmodeled static --SKIPIF-- locale sanity guard rejects platforms accepting invalid locale names\n'
            return 0
        fi
    fi
    if [[ "$locale_availability_count" -gt 0 ]]; then
        parsed_locale_count=$((parsed_locale_count + locale_availability_count))
        local candidates
        candidates=$(ptn_phpt_skipif_locale_candidates "$code" | sort -u)
        [[ -n "$candidates" ]] || return 1
        local candidate
        local found_locale=0
        while IFS= read -r candidate; do
            if ptn_phpt_locale_candidate_available "$candidate"; then
                found_locale=1
                break
            fi
        done <<< "$candidates"
        if [[ "$found_locale" -eq 0 ]]; then
            printf 'skipif-precondition\tmodeled static --SKIPIF-- locale availability guard requires one listed locale candidate; none are available in modeled host locale set\n'
            return 0
        fi
        modeled_families+=("locale-availability")
    fi
    [[ "$setlocale_count" -eq "$parsed_locale_count" ]] || return 1

    local defined_condition_lines
    local parsed_defined_count=0
    defined_condition_lines=$(printf '%s\n' "$code" \
        | grep -Eo "!?[[:space:]]*defined[[:space:]]*\\([[:space:]]*['\"][A-Za-z_][A-Za-z0-9_]*['\"][[:space:]]*\\)" \
        || true)
    if [[ -n "$defined_condition_lines" ]]; then
        while IFS= read -r condition; do
            local constant
            constant=$(printf '%s\n' "$condition" | sed -E "s/.*['\"]([^'\"]+)['\"].*/\\1/")
            case "$constant" in
                GLOB_BRACE)
                    ;;
                *)
                    return 1
                    ;;
            esac
            parsed_defined_count=$((parsed_defined_count + 1))
            if printf '%s\n' "$condition" | grep -Eq '^[[:space:]]*!'; then
                if ! ptn_phpt_php_constant_defined "$constant"; then
                    printf 'skipif-precondition\tmodeled static --SKIPIF-- constant guard requires %s defined; modeled PHP constants leave it undefined\n' "$constant"
                    return 0
                fi
            elif ptn_phpt_php_constant_defined "$constant"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- constant guard requires %s undefined; modeled PHP constants define it\n' "$constant"
                return 0
            fi
        done <<< "$defined_condition_lines"
        modeled_families+=("constant-defined")
    fi
    [[ "$defined_count" -eq "$parsed_defined_count" ]] || return 1

    local function_exists_condition_lines
    local parsed_function_exists_count=0
    function_exists_condition_lines=$(printf '%s\n' "$code" \
        | grep -Eo "!?[[:space:]]*function_exists[[:space:]]*\\([[:space:]]*['\"][^'\"]+['\"][[:space:]]*\\)" \
        || true)
    if [[ -n "$function_exists_condition_lines" ]]; then
        while IFS= read -r condition; do
            local function_name
            function_name=$(printf '%s\n' "$condition" | sed -E "s/.*['\"]([^'\"]+)['\"].*/\\1/")
            function_name=$(ptn_phpt_normalized_function_name "$function_name")
            [[ "$function_name" =~ ^[a-z_][a-z0-9_]*$ ]] || return 1
            parsed_function_exists_count=$((parsed_function_exists_count + 1))

            if printf '%s\n' "$condition" | grep -Eq '^[[:space:]]*!'; then
                if ! ptn_phpt_modeled_function_exists "$function_name"; then
                    printf 'skipif-precondition\tmodeled static --SKIPIF-- function availability guard requires %s; modeled PTN functions leave it unavailable\n' "$function_name"
                    return 0
                fi
            elif ptn_phpt_modeled_function_exists "$function_name"; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- function availability guard skips when %s is available in modeled PTN functions\n' "$function_name"
                return 0
            fi
        done <<< "$function_exists_condition_lines"
        modeled_families+=("function-exists")
    fi
    [[ "$function_exists_count" -eq "$parsed_function_exists_count" ]] || return 1

    local parsed_stream_filter_count=0
    if [[ "$stream_get_filters_count" -gt 0 || "$in_array_count" -gt 0 ]]; then
        [[ "$stream_get_filters_count" -eq 1 && "$in_array_count" -eq 1 ]] || return 1

        local squashed_code
        local stream_filter_var
        squashed_code=$(printf '%s\n' "$code" | ptn_phpt_squash_ws)
        stream_filter_var=$(printf '%s\n' "$squashed_code" \
            | sed -nE 's/.*\$([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*=[[:space:]]*stream_get_filters[[:space:]]*\([[:space:]]*\)[[:space:]]*;.*/\1/p')
        [[ -n "$stream_filter_var" ]] || return 1

        local stream_filter_guard_lines
        local stream_filter_var_pattern="\\\$$stream_filter_var"
        stream_filter_guard_lines=$(printf '%s\n' "$squashed_code" \
            | grep -Eo "!?[[:space:]]*in_array[[:space:]]*\\([[:space:]]*['\"][^'\"]+['\"][[:space:]]*,[[:space:]]*$stream_filter_var_pattern([[:space:]]*,[[:space:]]*(true|false))?[[:space:]]*\\)" \
            || true)
        [[ -n "$stream_filter_guard_lines" ]] || return 1

        local guard
        while IFS= read -r guard; do
            local filter_name
            filter_name=$(printf '%s\n' "$guard" | sed -E "s/.*['\"]([^'\"]+)['\"].*/\\1/")
            case "$filter_name" in
                string.rot13|string.toupper|string.tolower|convert.base64-encode|convert.base64-decode|convert.quoted-printable-encode|convert.quoted-printable-decode|dechunk|zlib.deflate|zlib.inflate)
                    ;;
                *)
                    return 1
                    ;;
            esac
            parsed_stream_filter_count=$((parsed_stream_filter_count + 1))
            if ! printf '%s\n' "$guard" | grep -Eq '^[[:space:]]*!'; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- stream filter availability guard skips when %s is present\n' "$filter_name"
                return 0
            fi
        done <<< "$stream_filter_guard_lines"
        modeled_families+=("stream-filter-availability")
    fi
    [[ "$stream_get_filters_count" -eq "$parsed_stream_filter_count" ]] || return 1
    [[ "$in_array_count" -eq "$parsed_stream_filter_count" ]] || return 1

    local root_helper_lines
    local parsed_include_count=0
    root_helper_lines=$(printf '%s\n' "$code" \
        | grep -Eo "(require|include)(_once)?[[:space:]]+[^;]*['\"][^'\"]*skipif_(no_)?root\\.inc['\"][^;]*;?" \
        || true)
    if [[ -n "$root_helper_lines" ]]; then
        local effective_uid
        effective_uid=$(ptn_phpt_effective_uid)
        while IFS= read -r condition; do
            parsed_include_count=$((parsed_include_count + 1))
            if printf '%s\n' "$condition" | grep -Eq 'skipif_no_root\.inc'; then
                if [[ "$effective_uid" -ne 0 ]]; then
                    printf 'skipif-precondition\tmodeled static --SKIPIF-- root helper requires root; modeled effective uid=%s\n' "$effective_uid"
                    return 0
                fi
            elif [[ "$effective_uid" -eq 0 ]]; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- non-root helper rejects root; modeled effective uid=0\n'
                return 0
            fi
        done <<< "$root_helper_lines"
        modeled_families+=("root-helper")
    fi
    [[ "$include_count" -eq "$parsed_include_count" ]] || return 1
    if [[ "$inactive_windows_helper_count" -gt 0 ]]; then
        modeled_families+=("inactive-windows-helper")
    fi

    local guard_count=$((parsed_env_count + parsed_int_count + parsed_int_max_count + parsed_os_family_count + parsed_php_os_count + parsed_debug_count + parsed_zts_count + parsed_locale_count + parsed_defined_count + parsed_function_exists_count + parsed_stream_filter_count))
    local recognized_count=$((guard_count + parsed_include_count + inactive_windows_helper_count))
    [[ "$recognized_count" -gt 0 ]] || return 1
    [[ "$if_count" -eq "$guard_count" ]] || return 1
    [[ "$output_count" -eq "$guard_count" ]] || return 1

    local old_ifs=$IFS
    IFS=,
    printf 'modeled-skipif\tmodeled static --SKIPIF-- preconditions satisfied: %s\n' "${modeled_families[*]}"
    IFS=$old_ifs
}

ptn_phpt_manifest_row() {
    local row=$1
    local php_src=$2

    if [[ "$row" = /* && "$row" == "$php_src/"* ]]; then
        printf '%s\n' "${row#"$php_src/"}"
    else
        printf '%s\n' "$row"
    fi
}

ptn_phpt_path_extension() {
    local row=$1

    if [[ "$row" == ext/*/tests/* || "$row" == ext/*/* ]]; then
        local rest=${row#ext/}
        printf '%s\n' "${rest%%/*}"
    fi
    return 0
}

ptn_phpt_first_unsupported_path_extension() {
    local row=$1
    local supported
    supported=$(ptn_phpt_supported_extensions)
    local extension

    extension=$(ptn_phpt_path_extension "$row")
    if [[ -n "$extension" ]] && ! ptn_phpt_csv_contains_ci "$extension" "$supported"; then
        printf '%s\n' "$extension"
        return 0
    fi

    return 1
}

ptn_phpt_declared_extensions() {
    local path=$1

    ptn_phpt_section "$path" EXTENSIONS | while IFS= read -r line; do
        line=${line%%#*}
        line=${line//,/ }
        local token
        for token in $line; do
            token=$(ptn_phpt_trim "$token")
            [[ -n "$token" ]] && printf '%s\n' "$token"
        done
    done
}

ptn_phpt_skipif_extension_probes() {
    local path=$1

    ptn_phpt_section "$path" SKIPIF \
        | grep -Eoi "(extension_loaded|get_extension_funcs)[[:space:]]*\\([[:space:]]*['\"][^'\"]+['\"]" \
        | sed -E "s/.*['\"]([^'\"]+)['\"].*/\\1/" \
        || true
}

ptn_phpt_first_unsupported_extension() {
    local row=$1
    local path=$2
    local supported
    supported=$(ptn_phpt_supported_extensions)
    local extension

    extension=$(ptn_phpt_path_extension "$row")
    if [[ -n "$extension" ]] && ! ptn_phpt_csv_contains_ci "$extension" "$supported"; then
        printf '%s\n' "$extension"
        return 0
    fi

    while IFS= read -r extension; do
        [[ -z "$extension" ]] && continue
        if ! ptn_phpt_csv_contains_ci "$extension" "$supported"; then
            printf '%s\n' "$extension"
            return 0
        fi
    done < <(ptn_phpt_declared_extensions "$path")

    while IFS= read -r extension; do
        [[ -z "$extension" ]] && continue
        if ! ptn_phpt_csv_contains_ci "$extension" "$supported"; then
            printf '%s\n' "$extension"
            return 0
        fi
    done < <(ptn_phpt_skipif_extension_probes "$path")

    return 1
}

ptn_phpt_requires_zend_test_native_helper() {
    local row=$1
    local path=$2
    local helper_pattern="zend_test_|zend_get_current_func_name|_ZendTest|ZendTest|ZEND_TEST|DoOperationNoCast|ReflectionExtension[[:space:]]*[(][[:space:]]*['\"]zend_test['\"]"

    [[ "$row" == ext/zend_test/tests/* ]] || return 1

    if LC_ALL=C grep -Eq "$helper_pattern" "$path"; then
        return 0
    fi

    local dir
    dir=$(dirname -- "$path")
    local include
    while IFS= read -r include; do
        [[ "$include" == /* || "$include" == *://* || "$include" == *..* ]] && continue
        local target="$dir/$include"
        [[ -f "$target" ]] || continue
        if LC_ALL=C grep -Eq "$helper_pattern" "$target"; then
            return 0
        fi
    done < <(
        LC_ALL=C grep -Eo "(require|include)(_once)?[[:space:]]+['\"][^'\"]+['\"]" "$path" \
            | sed -E "s/.*['\"]([^'\"]+)['\"].*/\\1/" \
            || true
    )

    return 1
}

ptn_phpt_first_unsupported_ini() {
    local path=$1
    local supported
    supported=$(ptn_phpt_supported_ini)
    local line

    while IFS= read -r line; do
        line=${line%%#*}
        line=${line%%;*}
        line=$(ptn_phpt_trim "$line")
        [[ -z "$line" ]] && continue
        local key=${line%%=*}
        key=$(ptn_phpt_trim "$key")
        [[ -z "$key" ]] && continue
        local lower_key
        lower_key=$(ptn_phpt_lower "$key")
        local value=${line#*=}
        value=$(ptn_phpt_lower "$(ptn_phpt_trim "$value")")
        case "$lower_key" in
            enable_post_data_reading)
                if [[ "$value" != "1" && "$value" != "on" && "$value" != "true" ]]; then
                    printf '%s\n' "$key"
                    return 0
                fi
                ;;
            disable_functions)
                local disabled_functions=${value//,/ }
                local disabled_function
                for disabled_function in $disabled_functions; do
                    if [[ "$disabled_function" != "dl" ]]; then
                        printf '%s\n' "$key"
                        return 0
                    fi
                done
                continue
                ;;
        esac
        if ! ptn_phpt_csv_contains_ci "$key" "$supported"; then
            printf '%s\n' "$key"
            return 0
        fi
    done < <(ptn_phpt_section "$path" INI)

    return 1
}

ptn_phpt_unsupported_ini_blocker() {
    local key
    key=$(ptn_phpt_lower "$(ptn_phpt_trim "$1")")

    case "$key" in
        fatal_error_backtraces|log_errors|report_memleaks)
            printf 'unsupported-diagnostics-ini\trequires engine diagnostic/logging mode %s; PTN diagnostics do not yet model that runtime channel\n' "$key"
            return 0
            ;;
        enable_post_data_reading)
            printf 'unsupported-request-input-ini\trequires request/input/upload SAPI state from ini setting %s; PTN currently models script execution without request body ingestion\n' "$key"
            return 0
            ;;
        disable_functions)
            printf 'unsupported-function-disable-ini\trequires runtime function table mutation from disable_functions; PTN currently emits a fixed function registry\n'
            return 0
            ;;
        sendmail_path|sys_temp_dir)
            printf 'unsupported-host-path-ini\trequires host path ini %s; PTN runtime does not yet model this process-global configuration\n' "$key"
            return 0
            ;;
    esac

    return 1
}

ptn_phpt_file_uses_function_call() {
    local path=$1
    local function_name
    function_name=$(ptn_phpt_normalized_function_name "$2")
    [[ -n "$function_name" ]] || return 1

    ptn_phpt_section "$path" FILE | LC_ALL=C awk -v fn="$function_name" '
        function ptn_php_code_line(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return tolower(out)
        }
        {
            line = ptn_php_code_line($0)
            pattern = "(^|[^[:alnum:]_$>:])" fn "[[:space:]]*\\("
            if (line ~ pattern) {
                found = 1
                exit
            }
        }
        END { exit found ? 0 : 1 }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
}

ptn_phpt_first_unsupported_standard_general_function_ini_surface() {
    local rel=$1
    local path=$2

    [[ "$rel" == ext/standard/tests/general_functions/* ]] || return 1

    if ! ptn_phpt_modeled_function_exists "ini_get_all" \
        && ptn_phpt_file_uses_function_call "$path" "ini_get_all"; then
        printf 'unsupported-general-function-runtime\trequires ini_get_all() runtime metadata, outside PTN modeled ini_get/ini_set surface\n'
        return 0
    fi

    if ptn_phpt_file_uses_function_call "$path" "output_add_rewrite_var" \
        || ptn_phpt_file_uses_function_call "$path" "output_reset_rewrite_vars"; then
        printf 'unsupported-output-rewrite-runtime\trequires output URL rewrite variable runtime, outside PTN modeled output buffering/session URL rewriting surface\n'
        return 0
    fi

    return 1
}

ptn_phpt_has_external_service_harness() {
    local path=$1

    case "$path" in
        */ext/sockets/tests/socket_dontfragment.phpt|*/ext/sockets/tests/socket_addrinfo_bind.phpt|*/ext/soap/tests/custom_content_type.phpt)
            return 1
            ;;
    esac

    grep -Eiq \
        'http_server(_skipif)?|server\.inc|skipifconnectfailure|mysql_pdo_test\.inc|MySQLPDOTest::|PHP_TEST_SHARED_EXTENSIONS|TEST_PHP_(MYSQL|PGSQL|LDAP|ODBC|FTP|SNMP)|getaddrinfo|localhost:[0-9]|127\.0\.0\.1:[0-9]|\[::1\]:[0-9]' \
        "$path"
}

ptn_phpt_has_process_boundary() {
    local path=$1

    case "$path" in
        */ext/standard/tests/file/*|*/ext/standard/tests/streams/*|*/ext/standard/tests/general_functions/*)
            awk '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            active = section == "FILE" || section == "CLEAN" || section == "SKIPIF"
            next
        }
        active && /(^|[^[:alnum:]_\$>:])(proc_open|proc_close|proc_get_status|proc_terminate|proc_nice)[[:space:]]*\(/ {
            found = 1
            exit
        }
        END { exit found ? 0 : 1 }
    ' "$path"
            ;;
        *)
            awk '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            active = section == "FILE" || section == "CLEAN" || section == "SKIPIF"
            next
        }
        active && /(^|[^[:alnum:]_\$>:])(proc_open|proc_close|proc_get_status|proc_terminate|proc_nice|popen|pclose|exec|system|passthru|shell_exec)[[:space:]]*\(/ {
            found = 1
            exit
        }
        END { exit found ? 0 : 1 }
    ' "$path"
            ;;
    esac
}

ptn_phpt_supported_cli_self_probe() {
    local path=$1
    local file
    file=$(ptn_phpt_section "$path" FILE)

    if printf '%s\n' "$file" | grep -Eq 'shell_exec[[:space:]]*\([[:space:]]*"\$php[[:space:]]+-n[[:space:]]+(-v|--version)"[[:space:]]*\)'; then
        return 0
    fi

    if printf '%s\n' "$file" | grep -Eq '#!\$php[[:space:]]+-n' \
        && printf '%s\n' "$file" | grep -Eq 'shell_exec[[:space:]]*\([[:space:]]*\$filename[[:space:]]*\)'; then
        return 0
    fi

    return 1
}

ptn_phpt_has_unsupported_cli_option_probe() {
    local path=$1

    ptn_phpt_section "$path" FILE \
        | grep -Eq -- '(^|[[:space:]])(--rf|--rc|--ri|--re|--ini|--notexisting|-w|-l|-s|-F|-R|-B|-E|-i|-m)([[:space:]]|$)'
}

ptn_phpt_has_resource_limit_expectation() {
    local path=$1

    awk '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            active = section == "EXPECT" || section == "EXPECTF" || section == "EXPECTREGEX"
            next
        }
        active && /(Allowed memory size|Failed to set memory limit)/ {
            found = 1
            exit
        }
        END { exit found ? 0 : 1 }
    ' "$path"
}

ptn_phpt_has_modeled_string_allocation_limit_expectation() {
    local path=$1
    local file

    if ! awk '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            active = section == "EXPECT" || section == "EXPECTF" || section == "EXPECTREGEX"
            next
        }
        active && /Allowed memory size/ {
            found = 1
            exit
        }
        END { exit found ? 0 : 1 }
    ' "$path"; then
        return 1
    fi

    file=$(ptn_phpt_section "$path" FILE)
    printf '%s\n' "$file" \
        | grep -Eq '(^|[^[:alnum:]_$])(wordwrap|chunk_split|iconv|iconv_substr|iconv_mime_decode|iconv_mime_encode|output_add_rewrite_var|bcmul)[[:space:]]*\('
}

ptn_phpt_first_unsupported_section() {
    local path=$1
    local unsupported
    unsupported=$(ptn_phpt_unsupported_sections)
    ptn_phpt_first_section_in_csv "$path" "$unsupported"
}

ptn_phpt_supported_file_external_row() {
    local rel=$1

    [[ "$rel" == "ext/dom/tests/DOMDocument_loadXML_variation4.phpt" ]]
}

ptn_phpt_supported_process_boundary_row() {
    local rel=$1

    [[ "$rel" == "ext/standard/tests/general_functions/proc_open_array.phpt" ]] ||
        [[ "$rel" == "ext/standard/tests/streams/bug60602.phpt" ]]
}

ptn_phpt_supported_zlib_output_ini_row() {
    local rel=$1
    local ini_key=$2

    [[ "$rel" == "tests/output/ob_018.phpt" ]] && [[ "$(ptn_phpt_lower "$ini_key")" == "zlib.output_compression" ]]
}

ptn_phpt_first_environment_section() {
    local path=$1
    local sections
    sections=$(ptn_phpt_environment_sections)
    ptn_phpt_first_section_in_csv "$path" "$sections"
}

ptn_phpt_first_harness_section() {
    local path=$1
    local sections
    sections=$(ptn_phpt_harness_sections)
    ptn_phpt_first_section_in_csv "$path" "$sections"
}

ptn_phpt_first_noisy_section() {
    local path=$1
    local sections
    sections=$(ptn_phpt_noisy_sections)
    ptn_phpt_first_section_in_csv "$path" "$sections"
}

ptn_phpt_first_section_in_csv() {
    local path=$1
    local csv=$2
    local section

    while IFS= read -r section; do
        if ptn_phpt_csv_contains_ci "$section" "$csv"; then
            printf '%s\n' "$section"
            return 0
        fi
    done < <(ptn_phpt_sections "$path")

    return 1
}

ptn_phpt_first_unsupported_language_surface() {
    local path=$1

    ptn_phpt_section "$path" FILE | LC_ALL=C awk -v ptn_path="$path" '
        function ptn_php_code_line_raw(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment_raw) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment_raw = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment_raw = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return out
        }
        function ptn_php_code_line(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return tolower(out)
        }
        function ptn_start_heredoc(code,    opener, rest, quote) {
            if (!match(code, /<<<[[:space:]]*["'\''"]?[A-Za-z_][A-Za-z0-9_]*/)) {
                return 0
            }
            opener = substr(code, RSTART, RLENGTH)
            rest = opener
            sub(/^<<<[[:space:]]*/, "", rest)
            quote = substr(rest, 1, 1)
            ptn_heredoc_nowdoc = 0
            if (quote == "\047") {
                ptn_heredoc_nowdoc = 1
                rest = substr(rest, 2)
            } else if (quote == "\"") {
                rest = substr(rest, 2)
            }
            ptn_heredoc_label = rest
            return 1
        }
        function ptn_ends_heredoc(raw, label,    suffix) {
            if (substr(raw, 1, length(label)) != label) {
                return 0
            }
            suffix = substr(raw, length(label) + 1, 1)
            return suffix == "" || suffix == ";" || suffix == "\r"
        }
        function ptn_has_php_attribute_syntax(raw,    i, ch, next_ch, prev, quote, escaped) {
            quote = ""
            escaped = 0
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    return 0
                }
                if (ch == "/" && next_ch == "*") {
                    i += 2
                    while (i <= length(raw) && !(substr(raw, i, 1) == "*" && substr(raw, i + 1, 1) == "/")) {
                        i++
                    }
                    i++
                    continue
                }
                if (ch == "#" && next_ch == "[") {
                    prev = i == 1 ? "" : substr(raw, i - 1, 1)
                    if (i == 1 || prev ~ /[[:space:]]/ || index("=({[,;", prev) > 0) {
                        return 1
                    }
                }
                if (ch == "#") {
                    return 0
                }
            }
            return 0
        }
        function ptn_defer_generator_reason(reason) {
            if (ptn_deferred_generator_reason == "") {
                ptn_deferred_generator_reason = reason
            }
        }
        function ptn_has_named_modeled_array_internal_call(line) {
            return line ~ /(^|[^[:alnum:]_$\\])array_(all|any|change_key_case|chunk|column|combine|count_values|diff|diff_assoc|diff_key|diff_uassoc|diff_ukey|fill|fill_keys|find|find_key|first|flip|intersect|intersect_assoc|intersect_key|intersect_uassoc|intersect_ukey|is_list|key_exists|key_first|key_last|keys|last|map|merge|merge_recursive|pad|pop|product|push|reduce|replace|replace_recursive|reverse|search|shift|splice|sum|udiff|udiff_assoc|udiff_uassoc|uintersect|uintersect_assoc|uintersect_uassoc|unique|unshift|values|walk|walk_recursive)[[:space:]]*\([^)]*[(,][[:space:]]*[a-z_][a-z0-9_]*[[:space:]]*:[^:]/
        }
        function ptn_has_by_reference_parameter(line) {
            return line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&?[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\([^)]*&[[:space:]]*(\.\.\.)?[[:space:]]*\$[a-z_]/
        }
        function ptn_is_function_declaration(line) {
            return line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&?[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\(/
        }
        function ptn_has_spl_iterator_object(line) {
            return line ~ /(^|[^[:alnum:]_$])new[[:space:]]+(arrayiterator|arrayobject|recursivearrayiterator|limititerator|iteratoriterator|infiniteiterator|filteriterator|callbackfilteriterator)([^[:alnum:]_]|$)/
        }
        function ptn_has_unmodeled_spl_symbol(line) {
            return line ~ /(^|[^[:alnum:]_$\\])(appenditerator|cachingiterator|directoryiterator|emptyiterator|filesystemiterator|globiterator|multipleiterator|norewinditerator|parentiterator|recursivecachingiterator|recursivecallbackfilteriterator|recursivefilteriterator|recursiveiteratoriterator|recursiveregexiterator|recursivetreeiterator|splfixedarray|spltempfileobject)([^[:alnum:]_]|$)/
        }
        function ptn_has_unmodeled_spl_function(line) {
            return line ~ /(^|[^[:alnum:]_$\\])(iterator_count|spl_classes)[[:space:]]*\(/ ||
                line ~ /(^|[^[:alnum:]_$\\])spl_autoload(_extensions)?[[:space:]]*\(/ ||
                line ~ /(^|[^[:alnum:]_$\\])spl_(classes|fixedarray|heap|objectstorage|priorityqueue)[a-z0-9_]*[[:space:]]*\(/
        }
        function ptn_supported_spl_fixed_array_surface_row() {
            return ptn_path ~ /ext\/spl\/tests\/ArrayObject\/ArrayObject_overloaded_SplFixedArray[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/ArrayObject\/gh15918[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/SplFixedArray_change_size_during_iteration[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/SplFixedArray_get_properties_for[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/SplFixedArray_immediate_gc[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/SplFixedArray_serialize[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/SplFixedArray_setSize_destruct[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/splfixedarray_json_encode[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/SplArray_fromArray[.]phpt$/
        }
        function ptn_supported_spl_fixed_array_surface_line(line) {
            return ptn_supported_spl_fixed_array_surface_row() &&
                line ~ /(^|[^[:alnum:]_$\\])splfixedarray([^[:alnum:]_]|$)/
        }
        function ptn_supported_recursive_iterator_iterator_surface_row() {
            return ptn_path ~ /ext\/spl\/tests\/ArrayObject\/array_009[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/ArrayObject\/array_009a[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/ArrayObject\/bug73209[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/iterator_028[.]phpt$/ ||
                ptn_path ~ /ext\/simplexml\/tests\/gh15837[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/RecursiveIteratorIterator_invalid_aggregate[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/RecursiveIteratorIterator_not_initialized[.]phpt$/
        }
        function ptn_supported_recursive_iterator_iterator_surface_line(line) {
            return ptn_supported_recursive_iterator_iterator_surface_row() &&
                line ~ /(^|[^[:alnum:]_$\\])(recursivearrayiterator|recursiveiteratoriterator)([^[:alnum:]_]|$)/
        }
        function ptn_supported_append_iterator_surface_line(line) {
            return ptn_path ~ /ext\/spl\/tests\/bug72684[.]phpt$/ &&
                line ~ /(^|[^[:alnum:]_$\\])appenditerator([^[:alnum:]_]|$)/
        }
        function ptn_supported_spl_helper_function_line(line) {
            return (ptn_path ~ /ext\/spl\/tests\/iterator_count_exception[.]phpt$/ ||
                    ptn_path ~ /ext\/dom\/tests\/bug79852[.]phpt$/) &&
                line ~ /(^|[^[:alnum:]_$\\])iterator_count[[:space:]]*\(/
        }
        function ptn_supported_spl_temp_file_object_surface_line(line) {
            return (ptn_path ~ /ext\/spl\/tests\/SplTempFileObject_constructor_memory_lt1_variation[.]phpt$/ ||
                    ptn_path ~ /ext\/spl\/tests\/gh9883-extra[.]phpt$/ ||
                    ptn_path ~ /ext\/spl\/tests\/SplFileObject\/bug77024[.]phpt$/ ||
                    ptn_path ~ /ext\/spl\/tests\/SplFileObject\/gh8561[.]phpt$/ ||
                    ptn_path ~ /ext\/spl\/tests\/SplFileObject\/gh8273[.]phpt$/) &&
                line ~ /(^|[^[:alnum:]_$\\])spltempfileobject([^[:alnum:]_]|$)/
        }
        function ptn_supported_recursive_directory_iterator_surface_line(line) {
            return ptn_path ~ /ext\/spl\/tests\/bug47534[.]phpt$/ &&
                line ~ /(^|[^[:alnum:]_$\\])(filesystemiterator|recursivedirectoryiterator)([^[:alnum:]_]|$)/
        }
        function ptn_supported_directory_iterator_surface_line(line) {
            return ptn_path ~ /ext\/spl\/tests\/dit_004[.]phpt$/ &&
                line ~ /(^|[^[:alnum:]_$\\])directoryiterator([^[:alnum:]_]|$)/
        }
        function ptn_supported_spl_autoload_register_validation_row() {
            return ptn_path ~ /ext\/spl\/tests\/autoloading\/spl_autoload_throw_with_spl_autoloader_call_as_autoloader[.]phpt$/
        }
        function ptn_supported_spl_autoload_helper_row() {
            return ptn_path ~ /ext\/spl\/tests\/autoloading\/bug52339[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/autoloading\/bug38325[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/autoloading\/bug73896[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/autoloading\/spl_autoload_012[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/autoloading\/spl_autoload_002[.]phpt$/
        }
        function ptn_supported_spl_autoload_helper_function_line(line) {
            return ptn_supported_spl_autoload_helper_row() &&
                line ~ /(^|[^[:alnum:]_$\\])spl_autoload(_extensions)?[[:space:]]*\(/
        }
        function ptn_supported_anonymous_get_class_row() {
            return ptn_path ~ /Zend\/tests\/anon\/anon_class_name[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/anon\/011[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/anon\/gh13097_a[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/anon\/gh13097_b[.]phpt$/
        }
        function ptn_supported_anonymous_class_alias_row() {
            return ptn_path ~ /Zend\/tests\/anon\/011[.]phpt$/
        }
        function ptn_supported_anonymous_trigger_error_row() {
            return ptn_path ~ /Zend\/tests\/anon\/gh13097_a[.]phpt$/
        }
        function ptn_supported_anonymous_closure_bind_row() {
            return ptn_path ~ /Zend\/tests\/anon\/013[.]phpt$/
        }
        function ptn_supported_anonymous_inherited_abstract_row() {
            return ptn_path ~ /Zend\/tests\/anon\/gh15994[.]phpt$/
        }
        function ptn_supported_anonymous_abstract_method_row() {
            return ptn_path ~ /Zend\/tests\/anon\/gh16067[.]phpt$/
        }
        function ptn_supported_anonymous_dynamic_static_row() {
            return ptn_path ~ /Zend\/tests\/anon\/008[.]phpt$/
        }
        function ptn_supported_autoload_include_class_declaration_row() {
            return ptn_path ~ /Zend\/tests\/autoload\/bug46665[.]phpt$/
        }
        function ptn_supported_autoload_parameter_default_class_constant_row() {
            return ptn_path ~ /Zend\/tests\/autoload\/bug42798[.]phpt$/
        }
        function ptn_supported_eval_class_declaration_raw(raw,    lower) {
            lower = tolower(raw)
            return lower ~ /(^|[^[:alnum:]_$])eval[[:space:]]*\([[:space:]]*["\047][[:space:]]*class[[:space:]]+[a-z_\\][a-z0-9_\\]*/
        }
        function ptn_supported_eval_static_variable_dynamic_function_row() {
            return ptn_path ~ /Zend\/tests\/static_variables\/static_variable_in_dynamic_function(_2)?[.]phpt$/
        }
        function ptn_supported_spl_autoload_eval_row() {
            return ptn_path ~ /ext\/spl\/tests\/autoloading\/spl_autoload_bug48541[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/autoloading\/bug74372[.]phpt$/
        }
        function ptn_supported_tokenizer_eval_row() {
            return ptn_path ~ /ext\/tokenizer\/tests\/gh19507_eval[.]phpt$/ ||
                ptn_path ~ /ext\/tokenizer\/tests\/token_get_all_variation19[.]phpt$/
        }
        function ptn_supported_zend_constexpr_lazy_eval_row() {
            return ptn_path ~ /Zend\/tests\/constexpr\/new[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/lazy_objects\/init_fatal[.]phpt$/
        }
        function ptn_supported_generator_foreach_cleanup_row() {
            return ptn_path ~ /Zend\/tests\/generators\/gc_with_iterator_in_foreach[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/generators\/no_foreach_var_leaks[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/generators\/yield_by_reference[.]phpt$/
        }
        function ptn_supported_generator_by_reference_assignment_yield_row() {
            return ptn_path ~ /Zend\/tests\/generators\/yield_by_reference_optimization[.]phpt$/
        }
        function ptn_supported_generator_by_reference_yield_from_diagnostic_row() {
            return ptn_path ~ /Zend\/tests\/generators\/yield_from_by_reference[.]phpt$/
        }
        function ptn_supported_generator_fiber_lifecycle_row() {
            return ptn_path ~ /Zend\/tests\/generators\/bug74840[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/gh9916-008[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/gh15108-006[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/gh9735-008[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/generators\/bug66041[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/fatal-error-in-nested-fiber[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/generators\/bug71013[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/generators\/gh15330-005[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/resume[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/destructors_005[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/gh9916-003[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/fibers\/gh15108-001[.]phpt$/
        }
        function ptn_has_direct_assignment_yield(line) {
            return line ~ /(^|[^=!<>])[$][a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*yield([^[:alnum:]_]|$)/
        }
        function ptn_has_generator_resume_call(line) {
            return line ~ /->[[:space:]]*(send|next|throw)[[:space:]]*\(/
        }
        function ptn_supported_fiber_surface_line(line) {
            if (line ~ /(^|[^[:alnum:]_$])new[[:space:]]+fiber([^[:alnum:]_]|$)/) {
                return line !~ /(^|[^[:alnum:]_$])function[[:space:]]*&/
            }
            return line ~ /(^|[^[:alnum:]_$])fiber[[:space:]]*::[[:space:]]*getcurrent[[:space:]]*\(/
        }
        function ptn_spread_context(line,    i, ch, triple, prefix, stack_depth, stack) {
            stack_depth = 0
            for (i = 1; i <= length(line); i++) {
                triple = substr(line, i, 3)
                if (triple == "...") {
                    if (stack_depth > 0 && (stack[stack_depth] == "[" || stack[stack_depth] == "array(" || stack[stack_depth] == "list(")) {
                        return "array"
                    }
                    return "call"
                }

                ch = substr(line, i, 1)
                if (ch == "(") {
                    prefix = tolower(substr(line, 1, i - 1))
                    if (prefix ~ /(^|[^[:alnum:]_$])(array|list)[[:space:]]*$/) {
                        stack[++stack_depth] = "array("
                    } else {
                        stack[++stack_depth] = "("
                    }
                } else if (ch == "[") {
                    stack[++stack_depth] = "["
                } else if ((ch == ")" || ch == "]") && stack_depth > 0) {
                    stack_depth--
                }
            }
            return ""
        }
        {
            if (ptn_heredoc_label != "") {
                if (ptn_ends_heredoc($0, ptn_heredoc_label)) {
                    ptn_heredoc_label = ""
                    ptn_heredoc_nowdoc = 0
                    next
                }
                next
            }
            if (ptn_start_heredoc(ptn_php_code_line_raw($0))) {
                next
            }
            line = ptn_php_code_line($0)
            tmp = line
            ptn_line_open_braces = gsub(/\{/, "", tmp)
            tmp = line
            ptn_line_close_braces = gsub(/\}/, "", tmp)
            ptn_class_declaration_line = line ~ /(^|[^[:alnum:]_$])(class|interface|trait)[[:space:]]+[a-z_\\][a-z0-9_\\]*/ &&
                line !~ /(^|[^[:alnum:]_$])new[[:space:]]+class([^[:alnum:]_]|$)/
            if (ptn_class_declaration_line) {
                ptn_class_body_pending = 1
            }
            ptn_function_declaration_line = line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&?[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\(/
            ptn_function_declaration_by_ref = ptn_function_declaration_line &&
                line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\(/
            if (ptn_function_declaration_line) {
                ptn_function_body_pending = 1
            }
            if (ptn_class_body_pending && ptn_line_open_braces > 0) {
                ptn_class_body_depth += ptn_line_open_braces
                ptn_class_body_pending = 0
            } else if (ptn_class_body_pending && ptn_line_open_braces == 0 && line ~ /;/) {
                ptn_class_body_pending = 0
            }
            ptn_static_local_context = ptn_function_body_depth > 0
            if (ptn_function_body_pending && ptn_line_open_braces > 0) {
                ptn_function_body_depth += ptn_line_open_braces
                if (ptn_function_declaration_by_ref) {
                    ptn_by_ref_function_body_depth = ptn_function_body_depth
                }
                ptn_function_body_pending = 0
                ptn_static_local_context = 1
            } else if (ptn_function_body_pending && ptn_line_open_braces == 0 && line ~ /;/) {
                ptn_function_body_pending = 0
            }
            ptn_in_by_ref_function = ptn_by_ref_function_body_depth > 0 &&
                ptn_function_body_depth >= ptn_by_ref_function_body_depth
            if (!ptn_function_declaration_line && ptn_function_body_depth > 0 && ptn_line_open_braces > 0) {
                ptn_function_body_depth += ptn_line_open_braces
            }
            if (!ptn_class_declaration_line && ptn_class_body_depth > 0 && ptn_line_open_braces > 0) {
                ptn_class_body_depth += ptn_line_open_braces
            }
            if (line ~ /(^|[^[:alnum:]_$])interface[[:space:]]+[a-z_\\][a-z0-9_\\]*/) {
                saw_interface = 1
            }
            if (!saw_anonymous_class && saw_interface && match(line, /function[[:space:]]+([a-z_][a-z0-9_]*)[[:space:]]*[(]/, method_match)) {
                override_interface_methods[method_match[1]] = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])(new[[:space:]]+fiber|fiber[[:space:]]*::)/ &&
                !ptn_supported_fiber_surface_line(line) &&
                !ptn_supported_generator_fiber_lifecycle_row()) {
                print "unsupported-generator-runtime\trequires Fiber coroutine runtime and by-reference return/getReturn boundary, outside PTN execution model"
                found = 1
                exit
            }
            if (ptn_has_direct_assignment_yield(line)) {
                ptn_generator_assignment_yield_context = 1
            }
            if (ptn_has_generator_resume_call(line)) {
                ptn_generator_resume_call_context = 1
            }
            if (ptn_generator_assignment_yield_context &&
                ptn_generator_resume_call_context &&
                !ptn_supported_generator_fiber_lifecycle_row()) {
                print "unsupported-generator-runtime\trequires Generator::send/next/throw continuation for yielded assignment expressions, outside PTN collected generator runtime"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])spl_autoload_register[[:space:]]*\(/) {
                saw_spl_autoload_register = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])(__autoload|spl_autoload(_extensions)?)[[:space:]]*\(/ &&
                !ptn_supported_spl_autoload_helper_row()) {
                print "unsupported-autoload-metadata\trequires runtime class autoload symbol-table mutation, outside PTN static class metadata"
                found = 1
                exit
            }
            if (saw_spl_autoload_register && line ~ /(^|[^[:alnum:]_$])(require|include)(_once)?[[:space:]]+/ && !ptn_supported_autoload_include_class_declaration_row()) {
                print "unsupported-autoload-metadata\trequires autoload callback include-driven class declaration, outside PTN modeled autoload registry"
                found = 1
                exit
            }
            if (saw_spl_autoload_register && line ~ /function[[:space:]]*&?[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\([^)]*=[^)]*[a-z_\\][a-z0-9_\\]*[[:space:]]*::/ && !ptn_supported_autoload_parameter_default_class_constant_row()) {
                print "unsupported-autoload-metadata\trequires autoload during parameter default class-constant resolution, outside PTN modeled autoload registry"
                found = 1
                exit
            }
            if ((ptn_has_unmodeled_spl_symbol(line) &&
                    !ptn_supported_spl_fixed_array_surface_line(line) &&
                    !ptn_supported_recursive_iterator_iterator_surface_line(line) &&
                    !ptn_supported_append_iterator_surface_line(line) &&
                    !ptn_supported_spl_temp_file_object_surface_line(line) &&
                    !ptn_supported_directory_iterator_surface_line(line) &&
                    !ptn_supported_recursive_directory_iterator_surface_line(line) &&
                    !ptn_supported_generator_fiber_lifecycle_row()) ||
                (ptn_has_unmodeled_spl_function(line) &&
                    !ptn_supported_spl_helper_function_line(line) &&
                    !ptn_supported_spl_autoload_helper_function_line(line))) {
                print "unsupported-spl-surface\trequires SPL data structures, filesystem iterators, recursive iterator stacks, or SPL helper functions outside PTN bounded array-backed iterator wrapper surface"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])abstract[[:space:]]+class[[:space:]]+/) {
                saw_abstract_class = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])get_class[[:space:]]*[(]/) {
                saw_get_class = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])new[[:space:]]+class([^[:alnum:]_]|$)/) {
                saw_anonymous_class = 1
                if (saw_abstract_class && !ptn_supported_anonymous_inherited_abstract_row()) {
                    print "unsupported-anonymous-class\trequires anonymous class abstract parent implementation diagnostics, outside PTN modeled anonymous class subset"
                    found = 1
                    exit
                }
                if (saw_get_class && !ptn_supported_anonymous_get_class_row()) {
                    print "unsupported-anonymous-class\trequires PHP hidden-suffix anonymous class generated names, outside PTN modeled anonymous class subset"
                    found = 1
                    exit
                }
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])class_alias[[:space:]]*[(]/ && !ptn_supported_anonymous_class_alias_row()) {
                print "unsupported-anonymous-class\trequires anonymous class runtime class_alias metadata, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])closure[[:space:]]*::[[:space:]]*bind[[:space:]]*[(]/ && !ptn_supported_anonymous_closure_bind_row()) {
                print "unsupported-anonymous-class\trequires Closure::bind() scope binding for anonymous class instances, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])trigger_error[[:space:]]*[(]/ && !ptn_supported_anonymous_trigger_error_row()) {
                print "unsupported-anonymous-class\trequires trigger_error() diagnostics containing anonymous class generated names, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])get_class[[:space:]]*[(]/ && !ptn_supported_anonymous_get_class_row()) {
                print "unsupported-anonymous-class\trequires PHP hidden-suffix anonymous class generated names, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /[$][A-Za-z_][A-Za-z0-9_]*[[:space:]]*::/ && !ptn_supported_anonymous_dynamic_static_row()) {
                print "unsupported-anonymous-class\trequires dynamic static member access through anonymous class objects, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])abstract[[:space:]]+(public|protected|private|static|function)/ && !ptn_supported_anonymous_abstract_method_row()) {
                print "unsupported-anonymous-class\trequires anonymous class abstract method diagnostics, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (ptn_has_by_reference_parameter(line)) {
                ptn_call_unpack_by_reference_context = 1
            }
            if (ptn_deferred_generator_reason != "" &&
                line ~ /(^|[^[:alnum:]_$])foreach[[:space:]]*\([^)]*as[^)]*&[[:space:]]*\$[a-z_]/) {
                print "unsupported-generator-runtime\trequires generator foreach by-reference iteration boundary and generator reference diagnostics, outside PTN generator runtime"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])foreach[[:space:]]*\(/) {
                ptn_generator_foreach_context = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])yield[[:space:]]+from([^[:alnum:]_]|$)/) {
                if (ptn_in_by_ref_function && !ptn_supported_generator_by_reference_yield_from_diagnostic_row()) {
                    ptn_defer_generator_reason("requires generator yield-from by-reference rejection, outside PTN collected generator runtime")
                }
                if (ptn_generator_foreach_context && !ptn_supported_generator_foreach_cleanup_row()) {
                    if (!ptn_supported_generator_fiber_lifecycle_row()) {
                        ptn_defer_generator_reason("requires generator suspension cleanup for live foreach variables and premature close, outside PTN generator runtime")
                    }
                }
                next
            }
            if (line ~ /(^|[^[:alnum:]_$])yield([[:space:];(),]|$)/) {
                if (ptn_in_by_ref_function &&
                    line ~ /yield[^;]*[^=!<>]=([^=>]|$)/ &&
                    !ptn_supported_generator_by_reference_assignment_yield_row()) {
                    ptn_defer_generator_reason("requires generator suspension timing for by-reference yielded assignment expressions, outside PTN collected generator runtime")
                    next
                }
                if (ptn_generator_foreach_context && !ptn_supported_generator_foreach_cleanup_row()) {
                    if (!ptn_supported_generator_fiber_lifecycle_row()) {
                        ptn_defer_generator_reason("requires generator suspension cleanup for live foreach variables and premature close, outside PTN generator runtime")
                    }
                }
                next
            }
            if (ptn_deferred_generator_reason ~ /generator suspension cleanup/ &&
                ptn_spread_context(line) == "call" &&
                line ~ /[.][.][.][[:space:]]*[a-z_\\][a-z0-9_\\]*[[:space:]]*[(]/) {
                ptn_deferred_generator_reason = ""
            }
            if (ptn_function_body_depth > 0 && ptn_line_close_braces > 0) {
                ptn_function_body_depth -= ptn_line_close_braces
                if (ptn_function_body_depth < 0) {
                    ptn_function_body_depth = 0
                }
                if (ptn_by_ref_function_body_depth > 0 &&
                    ptn_function_body_depth < ptn_by_ref_function_body_depth) {
                    ptn_by_ref_function_body_depth = 0
                }
            }
            if (ptn_class_body_depth > 0 && ptn_line_close_braces > 0) {
                ptn_class_body_depth -= ptn_line_close_braces
                if (ptn_class_body_depth < 0) {
                    ptn_class_body_depth = 0
                }
            }
            if (line ~ /(^|[^[:alnum:]_$])eval[[:space:]]*\(/ &&
                !ptn_supported_eval_class_declaration_raw($0) &&
                !ptn_supported_eval_static_variable_dynamic_function_row() &&
                !ptn_supported_spl_autoload_eval_row() &&
                !ptn_supported_tokenizer_eval_row() &&
                !ptn_supported_zend_constexpr_lazy_eval_row()) {
                print "unsupported-dynamic-eval\trequires eval runtime fallback, outside PTN native dynamic-code boundary"
                found = 1
                exit
            }
            if (ptn_has_named_modeled_array_internal_call(line)) {
                print "unsupported-internal-call-binding\trequires named-argument binding for modeled array internal calls, outside PTN internal-call lowering"
                found = 1
                exit
            }
        }
        END {
            if (!found && ptn_deferred_generator_reason != "") {
                print "unsupported-generator-runtime\t" ptn_deferred_generator_reason
                found = 1
            }
            exit found ? 0 : 1
        }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
}

ptn_phpt_first_unsupported_class_metadata_surface() {
    local path=$1
    local ptn_has_override_attribute=0
    local ptn_reflection_property_simple_metadata_row=0
    local ptn_reflection_property_typed_metadata_row=0

    if ptn_phpt_section "$path" FILE | LC_ALL=C grep -E '#\[[^]]*\\?Override([^[:alnum:]_]|$)' >/dev/null; then
        ptn_has_override_attribute=1
    fi
    if {
        ptn_phpt_section "$path" FILE | LC_ALL=C grep -Eiq 'reflectionproperty' &&
            ptn_phpt_section "$path" FILE | LC_ALL=C grep -Eiq -- '->[[:space:]]*(getmodifiers|isfinal)[[:space:]]*\('
    } || {
        ptn_phpt_section "$path" FILE | LC_ALL=C grep -Eiq -- '->[[:space:]]*getproperties[[:space:]]*\(' &&
            ptn_phpt_section "$path" FILE | LC_ALL=C grep -Eiq -- '->[[:space:]]*isfinal[[:space:]]*\('
    }; then
        ptn_reflection_property_simple_metadata_row=1
    fi
    if ptn_phpt_section "$path" FILE | LC_ALL=C grep -Eiq 'reflectionproperty' &&
        ptn_phpt_section "$path" FILE | LC_ALL=C grep -Eiq -- '->[[:space:]]*isinitialized[[:space:]]*\('; then
        ptn_reflection_property_typed_metadata_row=1
    fi

    ptn_phpt_section "$path" FILE | LC_ALL=C awk -v ptn_path="$path" -v ptn_has_override_attribute="$ptn_has_override_attribute" -v ptn_reflection_property_simple_metadata_row="$ptn_reflection_property_simple_metadata_row" -v ptn_reflection_property_typed_metadata_row="$ptn_reflection_property_typed_metadata_row" '
        function ptn_php_code_line(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return tolower(out)
        }
        function ptn_mark_object_string_unsupported(reason) {
            if (object_string_unsupported_reason == "") {
                object_string_unsupported_reason = reason
            }
        }
        function ptn_track_reflection_metadata_var(assignment) {
            sub(/^[$]/, "", assignment)
            sub(/[[:space:]]*=.*/, "", assignment)
            reflection_metadata_vars[assignment] = 1
        }
        function ptn_has_reflection_source_metadata_method(line) {
            return 0
        }
        function ptn_has_reflection_constant_source_metadata_method(line) {
            return line ~ /->[[:space:]]*getfilename[[:space:]]*\(/
        }
        function ptn_supported_internal_attribute_metadata_row() {
            return ptn_path ~ /Zend\/tests\/attributes\/(001_placement|003_ast_nodes|005_objects|006_filter|009_doctrine_annotations_example|011_inheritance|013_class_scope|014_class_const_group|017_closure_scope|020_userland_attribute_validation|031_backtrace|gh8421)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/(021_attribute_flags_type_is_validated|022_attribute_flags_value_is_validated|023_ast_node_in_validation|ossfuzz371445205)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/constants\/(allow_named_parameters|constant_listed_as_target-userland|constant_redefined_(addition|change|removal)|multiple_attributes_(grouped|ungrouped)|must_target_const-userland|not_repeatable-userland|oss_fuzz_428053935|repeatable-userland|target_all_targets_const-(default|explicit))[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/delayed_target_validation\/(has_runtime_errors|validator_(AllowDynamicProperties|Attribute|Deprecated|NoDiscard|success)|with_Attribute)[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/ReflectionAttribute_(constructor_001|newInstance_(deprecated|exception))[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/007_self_reflect_attribute[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/002_rfcexample[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/004_name_resolution[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/015_property_group[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/027_trailing_comma_args[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/028_grouped[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/029_reflect_internal_symbols[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/032_attribute_validation_scope[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/034_target_values[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/deprecated\/property_readonly_00[123][.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/nodiscard\/property_readonly_00[12][.]phpt$/
        }
        function ptn_supported_reflection_property_named_type_metadata_row() {
            return ptn_path !~ /(^|\/)(lazy_objects|property_hooks)\// &&
                ptn_path !~ /ext\/reflection\/tests\/types\//
        }
        function ptn_supported_typed_static_property_metadata_row() {
            return ptn_reflection_property_typed_metadata_row ||
                ptn_path ~ /ext\/reflection\/tests\/(ReflectionClass_setStaticPropertyValue_003|ReflectionProperty_(isReadable_static|isWritable_static|typed_static)|gh12856)[.]phpt$/ ||
                ptn_path ~ /(^|\/)Zend\/tests\/type_declarations\/typed_properties_043[.]phpt$/
        }
        function ptn_supported_assertion_closure_source_row() {
            return ptn_path ~ /(^|\/)Zend\/tests\/type_declarations\/types_in_ast[.]phpt$/
        }
        function ptn_supported_lazy_reflection_property_raw_row() {
            return ptn_path ~ /(^|\/)Zend\/tests\/lazy_objects\/realize_proxy_overridden[.]phpt$/ ||
                ptn_path ~ /(^|\/)ext\/reflection\/tests\/property_hooks\/gh17713[.]phpt$/
        }
        function ptn_supported_property_hook_metadata_row() {
            return ptn_path ~ /Zend\/tests\/asymmetric_visibility\/gh19044[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/asymmetric_visibility\/virtual_(get|set)_only[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/delayed_target_validation\/with_Override_(okay|error_get|error_set)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/delayed_target_validation\/(has_runtime_errors|validator_NoDiscard|no_compile_errors|with_(AllowDynamicProperties|Attribute|Deprecated|NoDiscard|ReturnTypeWillChange|SensitiveParameter))[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/nodiscard\/unsupported_property_hook_(get|set)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/closures\/closure_0(49|51|53|55|62)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/clone\/clone_with_00[3-5][.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/clone\/clone_with_012[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/(direct_hook_call|explicit_set_value_parameter|get(_type_check)?|isset|object_in_hook|parameter_attributes|set|unset)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/get_by_ref_(backed|implemented_by_plain|implemented_by_val|virtual)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/interface_get_(by_ref_(backed|plain|virtual)|value_as_ref)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/(backed_implicit_(get|set)|bug005|default_on_hooks|explicit_(iter|set_value_parameter_type)|field_(assign|guard)|find_property_usage|foreach_002)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/(bug006|final|gh(15456|15644|16185|16725|17234|18000|18268|20270)|gh19044-[1-6]|inheritance|invalid_abstract|override_(add_(get|set|get_contravariant|set_covariant)|attribute_(fail|plain|virtual)|default_value)|protected_to_public|set_value_parameter_type_variance_00[12357]|traits(_conflict)?|type_compatibility(_invalid(_2)?)?)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/parent_(get(_ci)?|set_plain_zpp|superfluous_args|syntax)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/(magic_interaction|unserialize)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/override_(attribute_backed|by_plain_prop)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/lazy_objects\/(unset_hook|isset_hooked_may_not_initialize)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/property_hooks\/(abstract_get_set_readonly|abstract_hook(_in_non_abstract_class|_not_implemented)?|abstract_prop_(final|hooks|not_implemented|without_hooks)|backed_invariant|bug00[1248]|cpp|default_on_virtual(_with_inheritance)?|duplicate_hook|final_private_prop|final_prop(_2|_promoted_[1234])?|gh154(19|38)_[12]|interface(_explicit_abstract|_final_hook|_final_prop|_get_only|_get_set_readonly|_invalid_explicitly_abstract|_not_implemented|_not_public)?|invalid_(abstract_(body|final|indirect(_2)?|private)|empty_hooks|final_private|hook_visibility|static|static_prop)|no_get_parameters|parent_get_not_in_class|parent_outside_property|private_prop_final_hook|property_promotion|readonly|set_by_ref|set_shorthand|set_variadic|syntax|traits_abstract|unknown_hook(_private)?|var_property)[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/ReflectionClass_getProperties_003[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/ReflectionClass_isIterable_gh20217[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/property_hooks\/(ReflectionProperty_getSetRawValue|ReflectionProperty_isInitialized|gh17713)[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/property_hooks\/ReflectionClass_getMethods[.]phpt$/
        }
        function ptn_supported_enum_metadata_row() {
            return ptn_path ~ /Zend\/tests\/enum\/(__call|__callStatic|__class__|__clone|__function__|__get|__invoke|__isset|__method__|__serialize|__set|__set_state|__sleep|__toString|__unserialize|__unset|__wakeup|backed-cases-int|backed-cases-string|backed-duplicate-int|backed-duplicate-string|backed-from-invalid-int|backed-from-invalid-string)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/enum\/(ast-dumper|backed-from-invalid-type|backed-from-unknown-hash|backed-from|backed-implements-multiple|backed-implements|backed-int-case-without-value|backed-int-const-expr|backed-int-const-invalid-expr|backed-int|backed-negative-int|backed-string-heredoc|backed-string|backed-tryFrom-unknown-hash|backed-tryFrom|basic-methods|case-attributes|comparison|constant-aliases|constants|default-parameter|empty-from|enum-as-constant|enum-as-params|enum-attributes|enum-in-constant|enum-in-static-var|enum-in-var-export|enum_exists|enum_in_stack_trace|extending-user-error|final|gh16954|gh7821|gh8176|gh8418|implements|instanceof|keyword-no-bc-break|keyword-whitespace|magic-constants|namespaces|no-clone|no-constructors|no-destruct|no-dynamic-properties|no-enum-implements-backed-enum|no-enum-implements-unit-enum|no-from|no-implement-serializable|no-name-property|no-new-through-reflection|no-new|no-non-backed-enum-implements-backed-enum|no-pass-properties-by-ref|no-properties|no-return-properties-by-ref|no-static-properties|no-unset-propertes|no-value-property|no-write-properties-cache-slot|no-write-properties-through-foreach-reference|no-write-properties-through-references|no-write-properties|non-backed-enum-with-int-value|non-backed-enum-with-string-value|offsetGet-in-const-expr|print_r|reflectionclass|static-methods|traits-constants|traits-no-__construct|traits-no-cases-method|traits-no-forbidden-methods|traits-no-properties|traits|unit-cases|update-class-constant-failure|var_dump-nested|var_dump-reference|var_export|weak-map)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/enum\/(backed-mismatch|backed-tryFrom-casing|backed-type-no-union|enum_underscore_as_name|json_encode|name-property|value-property-type)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/enum\/debugInfo\/(backed_enum_value|magic_method|missing_magic|param_validation|return_validation|visibility_validation)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/Attribute\/Attribute_on_enum[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/allow_dynamic_properties_on_enum[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/delayed_target_validation\/validator_(AllowDynamicProperties|Attribute|Deprecated)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/deprecated\/(class_constants\/101|error_on_enum)[.]phpt$/ ||
                ptn_path ~ /Zend\/tests\/attributes\/override\/01[45][.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/ReflectionClass(Constant_isEnumCase|_isEnum)[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/ReflectionEnum_(construct|getBackingType|getCase|getCases|hasCase|isBacked|toString(_(backed_int|backed_string|unbacked))?)[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/ReflectionEnumUnitCase_(construct|getDocComment|getEnum|getValue)[.]phpt$/ ||
                ptn_path ~ /ext\/reflection\/tests\/ReflectionEnumBackedCase_getBackingValue[.]phpt$/ ||
                ptn_path ~ /ext\/soap\/tests\/gh15711[.]phpt$/ ||
                ptn_path ~ /ext\/spl\/tests\/ArrayObject\/ArrayObject_enum[.]phpt$/
        }
        function ptn_supported_magic_lifecycle_metadata_row() {
            return ptn_path ~ /Zend\/tests\/magic_methods\/magic_methods_(019|serialize|sleep|unserialize|wakeup)[.]phpt$/
        }
        function ptn_supported_date_timelib_relative_row() {
            return ptn_path ~ /ext\/date\/tests\/(bug20382-1|date_modify-2|gh9700|bug49585|bug70277|bug45543|bug54597|bug29150|bug62852_var2|bug33415-2|bug75851|bug41964)[.]phpt$/
        }
        function ptn_supported_class_constant_static_local_metadata_row() {
            return ptn_path ~ /ext\/reflection\/tests\/bug63614[.]phpt$/
        }
        function ptn_supported_static_local_new_constant_expression_row() {
            return ptn_path ~ /Zend\/tests\/constexpr\/new[.]phpt$/
        }
        function ptn_supported_readonly_indirect_mutation_row() {
            return ptn_path ~ /Zend\/tests\/readonly_props\/(cache_slot|readonly_clone_error[237]|readonly_clone_success1|readonly_modification)[.]phpt$/
        }
        {
            line = ptn_php_code_line($0)
            if (implemented_modifier_diagnostic_seen) {
                next
            }
            if (line ~ /(^|[^[:alnum:]_$])static[[:space:]]+[$][a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*new[[:space:]]+/ ||
                line ~ /function[[:space:]]*&?[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\([^)]*=[[:space:]]*new[[:space:]]+/) {
                if (ptn_supported_static_local_new_constant_expression_row()) {
                    next
                }
                print "unsupported-constant-expression\trequires object construction in constant-expression initializers, outside PTN modeled constant expression subset"
                found = 1
                exit
            }
            if (line ~ /=[[:space:]]*&[^;]*->[[:space:]]*getstaticvariables[[:space:]]*\([^)]*\)[[:space:]]*\[/) {
                print "unsupported-reference-lvalue\trequires temporary array-offset reference binding, outside PTN modeled reference targets"
                found = 1
                exit
            }
            readonly_class_context = readonly_class_depth > 0 || readonly_class_pending || line ~ /(^|[^[:alnum:]_$])readonly[[:space:]]+class[[:space:]]+[a-z_\\]/
            if (line ~ /(^|[[:space:]])(public|protected|private|var)?[[:space:]]*readonly[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/ ||
                line ~ /(^|[[:space:]])readonly[[:space:]]+(public|protected|private|var)?[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                readonly_property_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])enum[[:space:]]+[a-z_\\]/ &&
                !ptn_supported_enum_metadata_row()) {
                print "unsupported-enum-metadata\trequires enum declarations and case metadata, outside PTN modeled class metadata"
                found = 1
                exit
            }
            implemented_modifier_diagnostic = line ~ /(^|[^[:alnum:]_$])abstract[[:space:]]+abstract([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])final[[:space:]]+final([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])final[[:space:]]+abstract[[:space:]]+(class|function)([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])abstract[[:space:]]+final[[:space:]]+(class|function)([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])public[[:space:]][^;{}]*public([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])protected[[:space:]][^;{}]*protected([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])private[[:space:]][^;{}]*private([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])static[[:space:]][^;{}]*static([^[:alnum:]_$]|$)/
            if (implemented_modifier_diagnostic) {
                implemented_modifier_diagnostic_seen = 1
                next
            }
            final_class_constant_modifier = line ~ /(^|[^[:alnum:]_$])final[[:space:]]+((public|protected|private)[[:space:]]+)?const([^[:alnum:]_$]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])(public|protected|private)[[:space:]]+final[[:space:]]+const([^[:alnum:]_$]|$)/
            final_method_modifier = line ~ /(^|[^[:alnum:]_$])final[[:space:]][^;{}=]*function[[:space:]]/
            if (!implemented_modifier_diagnostic &&
                !final_class_constant_modifier &&
                !final_method_modifier &&
                !ptn_supported_property_hook_metadata_row() &&
                line ~ /(^|[^[:alnum:]_$])final[[:space:]]+(function|static|public|protected|private|abstract)([^[:alnum:]_$]|$)/) {
                print "unsupported-class-contract-metadata\trequires final class/method override metadata, outside PTN modeled class dispatch"
                found = 1
                exit
            }
            if (!ptn_supported_property_hook_metadata_row() &&
                !ptn_reflection_property_simple_metadata_row &&
                line !~ /function[[:space:]]/ &&
                line ~ /(^|[[:space:]])(public|protected|private|var|static|readonly)([[:space:]]|[(])/ &&
                line ~ /[$][a-z_][a-z0-9_]*[[:space:]]*([=][^;{]*)?[{][[:space:]]*(get|set)[[:space:]]*(=>|[{])/) {
                print "unsupported-property-hook-metadata\trequires property hook accessors, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (!ptn_supported_property_hook_metadata_row() &&
                !ptn_reflection_property_simple_metadata_row &&
                line !~ /function[[:space:]]/ &&
                line ~ /(^|[[:space:]])(public|protected|private|var|static|readonly)([[:space:]]|[(])/ &&
                line ~ /[$][a-z_][a-z0-9_]*[[:space:]]*([=][^;{]*)?[{][[:space:]]*$/ &&
                line !~ /[{][^}]*[[:space:]](get|set)[[:space:]]*;/) {
                print "unsupported-property-hook-metadata\trequires property hook accessors, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])readonly[[:space:]]+static([^[:alnum:]_]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])static[[:space:]]+readonly([^[:alnum:]_]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])readonly[[:space:]]+class[^{;]*\{[^}]*static[[:space:]][^;}]*\$/) {
                print "unsupported-readonly-property-metadata\trequires readonly static property diagnostics, outside PTN runnable readonly property subset"
                found = 1
                exit
            }
            if (readonly_class_context &&
                line ~ /(^|[[:space:]])(public|protected|private|var)?[[:space:]]*static[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)?[[:space:]]*\$[a-z_]/) {
                print "unsupported-readonly-property-metadata\trequires readonly static property diagnostics, outside PTN runnable readonly property subset"
                found = 1
                exit
            }
            if (line ~ /function[[:space:]]+__tostring[[:space:]]*\(/) {
                object_string_seen = 1
            }
            if (line ~ /function[[:space:]]+__get[[:space:]]*\(/) {
                magic_get_seen = 1
            }
            if (line ~ /function[[:space:]]+__isset[[:space:]]*\(/) {
                magic_isset_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])array_column[[:space:]]*\(/) {
                array_column_seen = 1
            }
            if (line ~ /function[[:space:]]+&?[[:space:]]*__(serialize|unserialize|sleep|wakeup)[[:space:]]*\(/ &&
                !ptn_supported_enum_metadata_row() &&
                !ptn_supported_magic_lifecycle_metadata_row() &&
                !ptn_supported_date_timelib_relative_row()) {
                print "unsupported-magic-method-metadata\trequires magic method dispatch/reflection metadata, outside PTN modeled object/class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(__autoload|spl_autoload(_extensions)?)[[:space:]]*\(/ &&
                !ptn_supported_spl_autoload_helper_row()) {
                print "unsupported-autoload-metadata\trequires runtime class autoload symbol-table mutation, outside PTN static class metadata"
                found = 1
                exit
            }
            if (match(line, /[$][a-z_][a-z0-9_]*[[:space:]]*=[^;]*new[[:space:]]+\\?reflection(class|function|method)[[:space:]]*\(/)) {
                ptn_track_reflection_metadata_var(substr(line, RSTART, RLENGTH))
            }
            if (match(line, /[$][a-z_][a-z0-9_]*[[:space:]]*=[^;]*new[[:space:]]+\\?reflectionconstant[[:space:]]*\(/)) {
                reflection_constant_assignment = substr(line, RSTART, RLENGTH)
                sub(/^[$]/, "", reflection_constant_assignment)
                sub(/[[:space:]]*=.*/, "", reflection_constant_assignment)
                reflection_constant_vars[reflection_constant_assignment] = 1
            }
            if (match(line, /[$][a-z_][a-z0-9_]*[[:space:]]*=[^;]*\\?reflectionmethod[[:space:]]*::[[:space:]]*createfrommethodname[[:space:]]*\(/)) {
                ptn_track_reflection_metadata_var(substr(line, RSTART, RLENGTH))
            }
            if (match(line, /foreach[[:space:]]*\([^)]*->[[:space:]]*getmethods[[:space:]]*\([^)]*\)[[:space:]]+as[[:space:]]*[$][a-z_][a-z0-9_]*/)) {
                reflection_method_foreach_var = substr(line, RSTART, RLENGTH)
                sub(/^.*[[:space:]]as[[:space:]]*[$]/, "", reflection_method_foreach_var)
                reflection_metadata_vars[reflection_method_foreach_var] = 1
            }
            for (reflection_metadata_var in reflection_metadata_vars) {
            }
            for (reflection_constant_var in reflection_constant_vars) {
                if (line ~ ("(^|[^[:alnum:]_$])[$]" reflection_constant_var "[[:space:]]*->[[:space:]]*getfilename[[:space:]]*\\(")) {
                    print "unsupported-internal-reflection-metadata\trequires ReflectionConstant source metadata beyond PTN modeled constant attributes"
                    found = 1
                    exit
                }
            }
            if (ptn_has_reflection_source_metadata_method(line) && line ~ /(^|[^[:alnum:]_$\\])reflection(class|function|method)([^[:alnum:]_]|$)/) {
                print "unsupported-internal-reflection-metadata\trequires reflection source/doc/static-variable metadata beyond PTN modeled reflection invocation and shape metadata"
                found = 1
                exit
            }
            if (line ~ /new[[:space:]]+\\?reflectionconstant[[:space:]]*\([^;]*\)[[:space:]]*->[[:space:]]*getfilename[[:space:]]*\(/ ||
                ptn_has_reflection_constant_source_metadata_method(line) && line ~ /(^|[^[:alnum:]_$\\])reflectionconstant([^[:alnum:]_]|$)/) {
                print "unsupported-internal-reflection-metadata\trequires ReflectionConstant source metadata beyond PTN modeled constant attributes"
                found = 1
                exit
            }
            if (match(line, /[$][a-z_][a-z0-9_]*[[:space:]]*=[^;]*new[[:space:]]+\\?reflectionproperty[[:space:]]*\(/)) {
                reflection_property_assignment = substr(line, RSTART, RLENGTH)
                sub(/^[$]/, "", reflection_property_assignment)
                sub(/[[:space:]]*=.*/, "", reflection_property_assignment)
                reflection_property_vars[reflection_property_assignment] = 1
            }
            for (reflection_property_var in reflection_property_vars) {
                if (line ~ ("(^|[^[:alnum:]_$])[$]" reflection_property_var "[[:space:]]*->[[:space:]]*getattributes[[:space:]]*\\(") &&
                    !ptn_supported_internal_attribute_metadata_row()) {
                    print "unsupported-internal-reflection-metadata\trequires ReflectionProperty dynamic/internal/property-hook metadata beyond the declared property subset"
                    found = 1
                    exit
                }
                if (line ~ ("(^|[^[:alnum:]_$])[$]" reflection_property_var "[[:space:]]*->[[:space:]]*(gethook|skiplazyinitialization|setrawvaluewithoutlazyinitialization)[[:space:]]*\\(") &&
                    !(ptn_supported_property_hook_metadata_row() && line ~ ("(^|[^[:alnum:]_$])[$]" reflection_property_var "[[:space:]]*->[[:space:]]*gethook[[:space:]]*\\(")) &&
                    !(ptn_supported_lazy_reflection_property_raw_row() && line ~ ("(^|[^[:alnum:]_$])[$]" reflection_property_var "[[:space:]]*->[[:space:]]*(skiplazyinitialization|setrawvaluewithoutlazyinitialization)[[:space:]]*\\("))) {
                    print "unsupported-internal-reflection-metadata\trequires ReflectionProperty dynamic/internal/property-hook metadata beyond the declared property subset"
                    found = 1
                    exit
                }
                if (line ~ ("(^|[^[:alnum:]_$])[$]" reflection_property_var "[[:space:]]*->[[:space:]]*(gettype|isreadonly)[[:space:]]*\\(") &&
                    !ptn_supported_reflection_property_named_type_metadata_row()) {
                    print "unsupported-internal-reflection-metadata\trequires ReflectionProperty dynamic/internal/property-hook metadata beyond the declared property subset"
                    found = 1
                    exit
                }
            }
            if (line ~ /(^|[^[:alnum:]_$\\])new[[:space:]]+\\?reflectionproperty[[:space:]]*\([[:space:]]*new[[:space:]]+/ ||
                (line ~ /new[[:space:]]+\\?reflectionproperty[[:space:]]*\([^;]*\)[[:space:]]*->[[:space:]]*getattributes[[:space:]]*\(/ &&
                    !ptn_supported_internal_attribute_metadata_row()) ||
                (line ~ /new[[:space:]]+\\?reflectionproperty[[:space:]]*\([^;]*\)[[:space:]]*->[[:space:]]*(gethook|skiplazyinitialization|setrawvaluewithoutlazyinitialization)[[:space:]]*\(/ &&
                    !(ptn_supported_property_hook_metadata_row() && line ~ /new[[:space:]]+\\?reflectionproperty[[:space:]]*\([^;]*\)[[:space:]]*->[[:space:]]*gethook[[:space:]]*\(/) &&
                    !(ptn_supported_lazy_reflection_property_raw_row() && line ~ /new[[:space:]]+\\?reflectionproperty[[:space:]]*\([^;]*\)[[:space:]]*->[[:space:]]*(skiplazyinitialization|setrawvaluewithoutlazyinitialization)[[:space:]]*\(/)) ||
                (line ~ /new[[:space:]]+\\?reflectionproperty[[:space:]]*\([^;]*\)[[:space:]]*->[[:space:]]*(gettype|isreadonly)[[:space:]]*\(/ &&
                    !ptn_supported_reflection_property_named_type_metadata_row()) ||
                line ~ /(^|[^[:alnum:]_$\\])reflectionproperty[[:space:]]*::[[:space:]]*(is_[a-z0-9_]*|export|setaccessible|getmodifiernames)[[:space:]]*[(]/) {
                print "unsupported-internal-reflection-metadata\trequires ReflectionProperty dynamic/internal/property-hook metadata beyond the declared property subset"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$\\])new[[:space:]]+\\?reflectionclass[[:space:]]*\([[:space:]]*\\?attribute[[:space:]]*::[[:space:]]*class[[:space:]]*\)/) {
                reflection_attribute_self_context = 1
            }
            if (line ~ /(^|[^[:alnum:]_$\\])new[[:space:]]+\\?(deprecated|nodiscard)([^[:alnum:]_]|$)/ &&
                !ptn_supported_internal_attribute_metadata_row()) {
                print "unsupported-internal-attribute-metadata\trequires direct Deprecated/NoDiscard fatal stack parity beyond modeled caught-object behavior"
                found = 1
                exit
            }
            if (line ~ /->[[:space:]]*getattributes[[:space:]]*\(/ &&
                !reflection_attribute_self_context &&
                !ptn_supported_internal_attribute_metadata_row()) {
                print "unsupported-internal-attribute-metadata\trequires internal attribute/reflection metadata beyond modeled Attribute self-reflection"
                found = 1
                exit
            }
            if ((readonly_class_context || readonly_property_seen) &&
                !ptn_supported_readonly_indirect_mutation_row() &&
                (line ~ /=[[:space:]]*&[[:space:]]*\$[a-z_][a-z0-9_]*->[a-z_][a-z0-9_]*/ ||
                    line ~ /->[a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*&/ ||
                    line ~ /->[a-z_][a-z0-9_]*[[:space:]]*(\[|\+\+|--)/)) {
                print "unsupported-readonly-property-metadata\trequires indirect readonly property mutation diagnostics, outside PTN modeled readonly property subset"
                found = 1
                exit
            }
            if (!ptn_has_override_attribute &&
                !ptn_supported_typed_static_property_metadata_row() &&
                line ~ /(^|[[:space:]])(public|protected|private)[[:space:]]+static[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/ &&
                line !~ /(^|[[:space:]])(public|protected|private)[[:space:]]+static[[:space:]]+final[[:space:]]+\$[a-z_]/) {
                print "unsupported-typed-property-metadata\trequires typed static property metadata, outside PTN modeled static property declarations"
                found = 1
                exit
            }
            if (line ~ /(^|[;{}])[[:space:]]*static[[:space:]]+[$][a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*(array[[:space:]]*[(]|\[)/) {
                static_local_array_initializer = 1
            }
            if (static_local_array_initializer &&
                !ptn_supported_class_constant_static_local_metadata_row() &&
                line ~ /(^|[^[:alnum:]_$])(self|static|parent)[[:space:]]*::[[:space:]]*[a-z_][a-z0-9_]*[[:space:]]*=>/) {
                print "unsupported-class-constant-metadata\trequires class-constant array keys in reflected static local initializers, outside PTN modeled static-variable metadata"
                found = 1
                exit
            }
            if (static_local_array_initializer && line ~ /;/) {
                static_local_array_initializer = 0
            }
            if (line ~ /(^|[^[:alnum:]_$])readonly[[:space:]]+class[[:space:]]+[a-z_\\]/) {
                readonly_class_pending = 1
            }
            if (readonly_class_pending || readonly_class_depth > 0) {
                tmp = line
                opens = gsub(/\{/, "", tmp)
                tmp = line
                closes = gsub(/\}/, "", tmp)
                if (opens > 0) {
                    readonly_class_depth += opens
                    readonly_class_pending = 0
                }
                if (readonly_class_depth > 0) {
                    readonly_class_depth -= closes
                    if (readonly_class_depth < 0) {
                        readonly_class_depth = 0
                    }
                }
            }
        }
        END {
            if (!found && object_string_seen && object_string_unsupported_reason != "") {
                print "unsupported-object-string-conversion-metadata\t" object_string_unsupported_reason
                found = 1
            }
            exit found ? 0 : 1
        }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
}

ptn_phpt_first_unsupported_runtime_diagnostics_surface() {
    local path=$1
    local display_path=${2:-$path}

    ptn_phpt_section "$path" FILE | LC_ALL=C awk -v ptn_path="$display_path" '
        function ptn_php_code_line(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return tolower(out)
        }
        {
            raw = tolower($0)
            line = ptn_php_code_line($0)
            if (line ~ /\)[[:space:]]*\[[^]]*\]([[:space:]]*\[[^]]*\])*[[:space:]]*([+*\/%.&|^-]=|<<=|>>=|\+\+|--)/) {
                print "unsupported-lvalue-runtime\trequires compound writable function-call array-dimension temporaries, outside PTN modeled assignment target set"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])assert[[:space:]]*\(/) {
                if (line ~ /[?][?]=/) {
                    print "unsupported-assertion-runtime\trequires assertion expression lvalue mode interaction, outside PTN modeled assertion lowering"
                    found = 1
                    exit
                }
                if (ptn_path !~ /(^|\/)Zend\/tests\/type_declarations\/types_in_ast[.]phpt$/ &&
                    line ~ /assert[[:space:]]*\([[:space:]]*0[[:space:]]*&&.*function[[:space:]]*\(/) {
                    print "unsupported-assertion-runtime\trequires assertion AST pretty-printing for closure expressions, outside PTN modeled assertion diagnostics"
                    found = 1
                    exit
                }
            }
        }
        END { exit found ? 0 : 1 }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
}

ptn_phpt_first_unsupported_internal_surface() {
    local path=$1

    ptn_phpt_section "$path" FILE | LC_ALL=C awk '
        function ptn_php_code_line(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return tolower(out)
        }
        {
            line = ptn_php_code_line($0)
            if (match(line, /\$[a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*(php_int_max|2147483647)/)) {
                huge_array_count_var = substr(line, RSTART, RLENGTH)
                sub(/[[:space:]]*=.*/, "", huge_array_count_var)
            }
            if (line ~ /(^|[^[:alnum:]_$])array_fill[[:space:]]*\(/) {
                call = line
                sub(/^.*array_fill[[:space:]]*\([[:space:]]*/, "", call)
                sub(/^[^,]*,[[:space:]]*/, "", call)
                count_arg = call
                sub(/,.*/, "", count_arg)
                if (count_arg ~ /(php_int_max|2147483647)/ ||
                    (huge_array_count_var != "" && index(count_arg, huge_array_count_var) > 0)) {
                    print "unsupported-resource-limit\trequires PHP memory allocation failure/resource-limit diagnostics for multi-billion element array_fill(), outside PTN safe PHPT execution bounds"
                    found = 1
                    exit
                }
            }
            if (line ~ /(^|[^[:alnum:]_$])(array_merge|array_diff)[[:space:]]*\([[:space:]]*\.\.\.[[:space:]]*array_fill[[:space:]]*\([^,]*,[[:space:]]*2[[:space:]]*\*\*[[:space:]]*\([[:space:]]*32/) {
                print "unsupported-resource-limit\trequires PHP max-array-size/resource-limit diagnostics for spread-expanded multi-billion element array calls, outside PTN safe PHPT execution bounds"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])memory_(get_peak_usage|reset_peak_usage)[[:space:]]*\(/) {
                print "unsupported-resource-limit\trequires Zend memory manager peak-usage accounting APIs, outside PTN safe PHPT execution bounds"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])global[[:space:]]+\$/ || line ~ /\$globals[[:space:]]*\[/) {
                global_state_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])stream_wrapper_(unregister|restore)[[:space:]]*\(/) {
                print "unsupported-internal\trequires user stream wrapper unregister/restore state transitions outside PTN modeled stream/resource runtime"
                found = 1
                exit
            }
        }
        END {
            exit found ? 0 : 1
        }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
}

ptn_phpt_first_unsupported_phar_archive_surface() {
    local rel=$1
    local path=$2

    [[ "$rel" == ext/phar/* ]] || return 1

    ptn_phpt_section "$path" FILE | LC_ALL=C awk '
        function ptn_php_code_line(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return tolower(out)
        }
        {
            raw = tolower($0)
            line = ptn_php_code_line($0)
            if (line ~ /(^|[^[:alnum:]_$])new[[:space:]]+\\?phar(data|fileinfo)?[[:space:]]*\(/) {
                print "unsupported-phar-archive-runtime\trequires Phar archive object parsing/mutation runtime, outside PTN modeled Phar metadata surface"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$\\])phar[[:space:]]*::[[:space:]]*(mapphar|webphar|mount|interceptfilefuncs|loadphar|unlinkarchive|mungserver|running|createstub|createdefaultstub)[[:space:]]*\(/) {
                print "unsupported-phar-archive-runtime\trequires Phar archive mapping/front-controller runtime, outside PTN modeled Phar metadata surface"
                found = 1
                exit
            }
            if (raw ~ /phar:\/\//) {
                print "unsupported-phar-archive-runtime\trequires phar:// stream wrapper archive access, outside PTN modeled stream runtime"
                found = 1
                exit
            }
        }
        END {
            exit found ? 0 : 1
        }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
}

ptn_phpt_supported_phar_archive_residual_row() {
    case "$1" in
        ext/phar/tests/002.phpt|\
        ext/phar/tests/004.phpt|\
        ext/phar/tests/005.phpt|\
        ext/phar/tests/006.phpt|\
        ext/phar/tests/007.phpt|\
        ext/phar/tests/008.phpt|\
        ext/phar/tests/009.phpt|\
        ext/phar/tests/010.phpt|\
        ext/phar/tests/011.phpt|\
        ext/phar/tests/012.phpt|\
        ext/phar/tests/013.phpt|\
        ext/phar/tests/014.phpt|\
        ext/phar/tests/016.phpt|\
        ext/phar/tests/include_path_advanced.phpt|\
        ext/phar/tests/phar_extract3.phpt|\
        ext/phar/tests/delete.phpt|\
        ext/phar/tests/rename_dir.phpt|\
        ext/phar/tests/gh20882.phpt|\
        ext/phar/tests/019c.phpt|\
        ext/phar/tests/bug54395.phpt|\
        ext/phar/tests/bug79503.phpt|\
        ext/phar/tests/zip/phar_buildfromiterator4.phpt|\
        ext/phar/tests/phar_buildfromiterator10.phpt|\
        ext/phar/tests/buildFromIterator_user_overrides/getMTime_errors.phpt|\
        ext/phar/tests/phar_oo_004.phpt|\
        ext/phar/tests/cached_manifest_1.phpt|\
        ext/phar/tests/stat.phpt|\
        ext/phar/tests/zip/frontcontroller20.phar.phpt|\
        ext/phar/tests/tar/phar_stub_error.phpt|\
        ext/phar/tests/zip/rmdir.phpt|\
        ext/phar/tests/zip/phar_oo_compressallgz.phpt|\
        ext/phar/tests/tar/phar_convert_phar4.phpt|\
        ext/phar/tests/zip/rename.phpt|\
        ext/phar/tests/tar/rmdir.phpt|\
        ext/phar/tests/tar/tar_nohash.phpt|\
        ext/phar/tests/tar/phar_setdefaultstub.phpt|\
        ext/phar/tests/tar/links3.phpt|\
        ext/phar/tests/zip/phar_stub_error.phpt|\
        ext/phar/tests/tar/delete_in_phar_b.phpt)
            return 0
            ;;
    esac
    return 1
}

ptn_phpt_first_unsupported_zip_archive_surface() {
    local rel=$1
    local path=$2

    [[ "$rel" == ext/zip/* ]] || return 1

    ptn_phpt_section "$path" FILE | LC_ALL=C awk '
        function ptn_php_code_line(raw,    i, ch, next_ch, out, quote, escaped) {
            quote = ""
            escaped = 0
            out = ""
            for (i = 1; i <= length(raw); i++) {
                ch = substr(raw, i, 1)
                next_ch = substr(raw, i + 1, 1)
                if (ptn_block_comment) {
                    if (ch == "*" && next_ch == "/") {
                        ptn_block_comment = 0
                        out = out "  "
                        i++
                    } else {
                        out = out " "
                    }
                    continue
                }
                if (quote != "") {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == quote) {
                        quote = ""
                    }
                    out = out " "
                    continue
                }
                if (ch == "\"" || ch == "\047") {
                    quote = ch
                    out = out " "
                    continue
                }
                if (ch == "/" && next_ch == "/") {
                    break
                }
                if (ch == "#") {
                    break
                }
                if (ch == "/" && next_ch == "*") {
                    ptn_block_comment = 1
                    out = out "  "
                    i++
                    continue
                }
                out = out ch
            }
            return tolower(out)
        }
        {
            line = ptn_php_code_line($0)
            if (line ~ /->[[:space:]]*(registerprogresscallback|setarchivecomment|unchange(all|archive)?)[[:space:]]*\(/) {
                print "unsupported-zip-archive-runtime\trequires unmodeled ZipArchive archive mutation/callback runtime"
                found = 1
                exit
            }
        }
        END {
            exit found ? 0 : 1
        }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
}

ptn_phpt_supported_curl_server_harness_row() {
    case "$1" in
        ext/curl/tests/bug79033.phpt|\
        ext/curl/tests/curl_copy_handle_variation4.phpt|\
        ext/curl/tests/curl_basic_013.phpt|\
        ext/curl/tests/curl_basic_023.phpt|\
        ext/curl/tests/curl_basic_003.phpt|\
        ext/curl/tests/bug48207.phpt|\
        ext/curl/tests/curl_handle_clone.phpt|\
        ext/curl/tests/bug45161.phpt|\
        ext/curl/tests/bug54798-unix.phpt|\
        ext/curl/tests/gh21023.phpt|\
        ext/curl/tests/curl_writeheader_callback.phpt)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

ptn_phpt_supported_ftp_server_harness_row() {
    [[ "$1" == "ext/standard/tests/streams/opendir-004.phpt" ]]
}

ptn_phpt_supported_php_cli_server_harness_row() {
    [[ "$1" == "ext/opcache/tests/issue0149.phpt" ]]
}

ptn_phpt_classify_row() {
    local row=$1
    local path=$2
    local php_src=${3:-}
    local rel=$row
    local sections
    local value
    local modeled_skipif_reason=""
    local unmodeled_skipif=0
    local supported_cli_self_probe=0

    if [[ -n "$php_src" ]]; then
        rel=$(ptn_phpt_manifest_row "$row" "$php_src")
    fi

    sections=$(ptn_phpt_sections_csv "$path")

    if value=$(ptn_phpt_first_unsupported_path_extension "$rel"); then
        printf 'unsupported-extension\trequires unavailable PTN extension %s; modeled extensions: %s\n' \
            "$value" "$(ptn_phpt_supported_extensions)"
        return 0
    fi

    if ptn_phpt_csv_contains_ci "EXTENSIONS" "$sections" || ptn_phpt_csv_contains_ci "SKIPIF" "$sections"; then
        if value=$(ptn_phpt_first_unsupported_extension "$rel" "$path"); then
            printf 'unsupported-extension\trequires unavailable PTN extension %s; modeled extensions: %s\n' \
                "$value" "$(ptn_phpt_supported_extensions)"
            return 0
        fi
    fi

    if ptn_phpt_requires_zend_test_native_helper "$rel" "$path"; then
        printf 'unsupported-zend-test-helper\trequires zend_test native helper API not modeled by the PTN runtime extension shim\n'
        return 0
    fi

    if ptn_phpt_has_external_service_harness "$path" &&
        ! ptn_phpt_supported_curl_server_harness_row "$rel" &&
        ! ptn_phpt_supported_ftp_server_harness_row "$rel" &&
        ! ptn_phpt_supported_php_cli_server_harness_row "$rel"; then
        printf 'external-service\trequires external service or php-src server harness\n'
        return 0
    fi

    if [[ "$rel" == sapi/cgi/* ]]; then
        printf 'cgi-sapi-executable\trequires php-cgi executable option/request behavior; PTN phpc currently models CLI scripts plus bounded CGI request context only\n'
        return 0
    fi

    if [[ "$rel" == sapi/fpm/* ]]; then
        printf 'fpm-sapi\trequires PHP-FPM master/worker, FastCGI, pool configuration, or signal behavior outside PTN native script execution\n'
        return 0
    fi

    if [[ "$rel" == sapi/phpdbg/* ]] || ptn_phpt_csv_contains_ci "PHPDBG" "$sections"; then
        printf 'phpdbg-sapi\trequires phpdbg debugger command SAPI behavior outside PTN native script execution\n'
        return 0
    fi

    if [[ "$rel" == tests/run-test/* ]]; then
        printf 'harness-self-test\texercises php-src run-tests.php harness behavior instead of PTN PHP semantics\n'
        return 0
    fi

    if [[ "$rel" == sapi/cli/* ]]; then
        if value=$(ptn_phpt_first_unsupported_language_surface "$path"); then
            printf '%s\n' "$value"
            return 0
        fi
        if ptn_phpt_supported_cli_self_probe "$path"; then
            supported_cli_self_probe=1
        elif ptn_phpt_has_unsupported_cli_option_probe "$path"; then
            printf 'unsupported-cli-option\trequires PHP CLI option behavior outside PTN phpc supported runner modes (-f, -r, -d, -v, -m, and bounded CGI -C)\n'
            return 0
        elif ptn_phpt_has_process_boundary "$path" && ! ptn_phpt_supported_process_boundary_row "$rel"; then
            printf 'process-boundary\trequires child-process execution/control and pipe semantics outside PTN native runtime boundary\n'
            return 0
        else
            printf 'sapi-behavior\trequires CLI SAPI executable behavior outside PTN phpc supported runner modes\n'
            return 0
        fi
    elif [[ "$rel" == sapi/* ]]; then
        printf 'sapi-behavior\texercises php-src SAPI executable behavior outside PTN script execution\n'
        return 0
    fi

    if [[ "$supported_cli_self_probe" -ne 1 ]] \
        && ptn_phpt_has_process_boundary "$path" \
        && ! ptn_phpt_supported_process_boundary_row "$rel"; then
        printf 'process-boundary\trequires child-process execution/control and pipe semantics outside PTN native runtime boundary\n'
        return 0
    fi

    if ptn_phpt_csv_contains_ci "INI" "$sections"; then
        if value=$(ptn_phpt_first_unsupported_standard_general_function_ini_surface "$rel" "$path"); then
            printf '%s\n' "$value"
            return 0
        fi
        if value=$(ptn_phpt_first_unsupported_ini "$path"); then
            if ! ptn_phpt_supported_zlib_output_ini_row "$rel" "$value"; then
                if ptn_phpt_unsupported_ini_blocker "$value"; then
                    return 0
                fi
                printf 'unsupported-ini\trequires unsupported ini setting %s; modeled ini keys: %s\n' \
                    "$value" "$(ptn_phpt_supported_ini)"
                return 0
            fi
        fi
    fi

    if [[ "$supported_cli_self_probe" -ne 1 ]] \
        && ptn_phpt_classify_harness_programs \
        && ptn_phpt_csv_contains_ci "SKIPIF" "$sections"; then
        if value=$(ptn_phpt_modeled_skipif_precondition "$path" "$rel"); then
            local skipif_category=${value%%$'\t'*}
            local skipif_reason=${value#*$'\t'}
            if [[ "$skipif_category" == "modeled-skipif" ]]; then
                modeled_skipif_reason=$skipif_reason
            else
                printf '%s\n' "$value"
                return 0
            fi
        else
            unmodeled_skipif=1
        fi
    fi

    if ptn_phpt_has_resource_limit_expectation "$path" \
        && ! ptn_phpt_has_modeled_string_allocation_limit_expectation "$path"; then
        printf 'unsupported-resource-limit-ini\trequires Zend memory manager allocation-failure/resource-limit diagnostics outside PTN safe PHPT execution bounds\n'
        return 0
    fi

    if ptn_phpt_supported_phar_archive_residual_row "$rel"; then
        printf 'runnable\timplemented PHAR tar/zip archive residual row pack\n'
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_phar_archive_surface "$rel" "$path"); then
        printf '%s\n' "$value"
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_zip_archive_surface "$rel" "$path"); then
        printf '%s\n' "$value"
        return 0
    fi

    if value=$(ptn_phpt_first_section_in_sections_csv "$sections" "$(ptn_phpt_unsupported_sections)"); then
        if [[ "$value" != "FILE_EXTERNAL" ]] || ! ptn_phpt_supported_file_external_row "$rel"; then
            printf 'sapi-behavior\trequires unsupported PHPT section --%s--\n' "$value"
            return 0
        fi
    fi

    if value=$(ptn_phpt_first_section_in_sections_csv "$sections" "$(ptn_phpt_environment_sections)"); then
        printf 'environment-assumption\trequires PHPT environment setup section --%s-- outside PTN script semantics\n' "$value"
        return 0
    fi

    if value=$(ptn_phpt_first_section_in_sections_csv "$sections" "$(ptn_phpt_harness_sections)"); then
        printf 'harness-cleanup\trequires PHPT harness post-test/setup section --%s-- executed outside measured program output\n' "$value"
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_language_surface "$path"); then
        printf '%s\n' "$value"
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_class_metadata_surface "$path"); then
        printf '%s\n' "$value"
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_runtime_diagnostics_surface "$path" "$rel"); then
        printf '%s\n' "$value"
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_internal_surface "$path"); then
        printf '%s\n' "$value"
        return 0
    fi

    if ptn_phpt_csv_contains_ci "XFAIL" "$sections"; then
        printf 'intentional-nongoal\tupstream XFAIL row is excluded from PTN pass-count campaign\n'
        return 0
    fi

    if value=$(ptn_phpt_first_section_in_sections_csv "$sections" "$(ptn_phpt_noisy_sections)"); then
        printf 'noisy-expectation\trequires noisy or external PHPT expectation mode --%s--\n' "$value"
        return 0
    fi

    if [[ "$unmodeled_skipif" -eq 1 ]]; then
        if value=$(ptn_phpt_first_section_in_sections_csv "$sections" "$(ptn_phpt_skipif_harness_sections)"); then
            printf 'harness-skipif\trequires PHPT harness precondition section --%s-- evaluated before measured program output\n' "$value"
            return 0
        fi
    fi

    if [[ -n "$modeled_skipif_reason" ]]; then
        printf 'runnable\tselected for PTN semantic measurement; %s\n' "$modeled_skipif_reason"
        return 0
    fi

    printf 'runnable\tselected for PTN semantic measurement\n'
}

ptn_phpt_category_slug() {
    local category=$1
    local slug
    slug=$(printf '%s' "$category" \
        | tr '[:upper:]' '[:lower:]' \
        | sed 's/[^a-z0-9._-]/-/g; s/--*/-/g; s/^-//; s/-$//')
    [[ -n "$slug" ]] || slug="category"
    printf '%s\n' "$slug"
}
