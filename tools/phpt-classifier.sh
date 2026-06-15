#!/usr/bin/env bash

# Shared PHPT preflight classification for PTN measurement runs.
#
# The classifier filters only PHPT harness/environment requirements that PTN
# does not currently model: extension availability, unsupported ini/runtime
# modes, SAPI/request sections, external service harnesses, process-boundary
# rows, broad harness cleanup/setup sections, opt-in harness preconditions,
# noisy upstream rows, broad unsupported language surfaces, source-level
# runtime diagnostic APIs, and upstream XFAILs. Generic PHP semantic gaps
# inside the modeled surface remain runnable and should surface as PTN failures.

PTN_PHPT_SUPPORTED_EXTENSIONS_DEFAULT="Core,date,pcre,standard,Reflection"
PTN_PHPT_SUPPORTED_INI_DEFAULT="assert.exception,date.timezone,display_errors,error_reporting,extension_dir,include_path,max_memory_limit,memory_limit,pcre.backtrack_limit,precision,zend.assertions,zend.exception_string_param_max_len"
PTN_PHPT_UNSUPPORTED_SECTIONS_DEFAULT="ARGS,CAPTURE_STDIO,CGI,COOKIE,COOKIE_RAW,EXPECTHEADERS,FILE_EXTERNAL,GET,HEADERS,PHPDBG,POST,POST_RAW,PUT,REDIRECTTEST,REQUEST,STDIN"
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

ptn_phpt_php_truthy() {
    local value=$1
    [[ -n "$value" && "$value" != "0" ]]
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
    local code
    local code_without_strings
    local code_for_identifiers
    code=$(ptn_phpt_skipif_code "$path")
    [[ -n "$code" ]] || return 1

    code_without_strings=$(printf '%s\n' "$code" | ptn_phpt_strip_php_strings | ptn_phpt_squash_ws)
    code_for_identifiers=$(printf '%s\n' "$code_without_strings" | sed -E 's/\$[A-Za-z_][A-Za-z0-9_]*/ /g')

    local identifier
    while IFS= read -r identifier; do
        [[ -n "$identifier" ]] || continue
        case "$identifier" in
            if|getenv|die|exit|echo|print|PHP_INT_SIZE|PHP_INT_MAX|PHP_OS_FAMILY|PHP_OS|PHP_DEBUG|PHP_ZTS|PHP_VERSION|version_compare|substr|setlocale|LC_ALL|LC_COLLATE|LC_CTYPE|LC_MESSAGES|LC_MONETARY|LC_NUMERIC|LC_TIME)
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
                SKIP_ASAN|SKIP_MSAN|SKIP_UBSAN|SKIP_PERF_SENSITIVE)
                    ;;
                SKIP_*|USE_ZEND_ALLOC|USE_TRACKED_ALLOC|RUN_RESOURCE_HEAVY_TESTS|STACK_LIMIT_DEFAULTS_CHECK)
                    env_family="environment"
                    ;;
                *)
                    return 1
                    ;;
            esac
            parsed_env_count=$((parsed_env_count + 1))
            local env_value=${!env_var:-}
            local env_is_truthy=0
            if ptn_phpt_php_truthy "$env_value"; then
                env_is_truthy=1
            fi
            if printf '%s\n' "$code" | grep -Eq "![[:space:]]*getenv[[:space:]]*\\([[:space:]]*['\"]$env_var['\"][[:space:]]*\\)"; then
                if [[ "$env_is_truthy" -eq 0 ]]; then
                    printf 'skipif-precondition\tmodeled static --SKIPIF-- environment gate requires %s set; current environment leaves it unset\n' "$env_var"
                    return 0
                fi
            elif [[ "$env_is_truthy" -eq 1 ]]; then
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

    local recognized_count=$((parsed_env_count + parsed_int_count + parsed_int_max_count + parsed_os_family_count + parsed_php_os_count + parsed_debug_count + parsed_zts_count + parsed_locale_count))
    [[ "$recognized_count" -gt 0 ]] || return 1
    [[ "$if_count" -eq "$recognized_count" ]] || return 1
    [[ "$output_count" -eq "$recognized_count" ]] || return 1

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
        register_argc_argv|variables_order|enable_post_data_reading|file_uploads|max_input_vars|max_input_nesting_level|post_max_size|always_populate_raw_post_data)
            printf 'unsupported-request-input-ini\trequires request/input/upload SAPI state controlled by %s; PTN native CLI currently has no request boundary\n' "$key"
            return 0
            ;;
        fatal_error_backtraces|error_log|report_memleaks)
            printf 'unsupported-diagnostics-ini\trequires engine diagnostic/logging mode %s; PTN diagnostics do not yet model that runtime channel\n' "$key"
            return 0
            ;;
        disable_functions)
            printf 'unsupported-function-disable-ini\trequires runtime function table mutation from disable_functions; PTN currently emits a fixed function registry\n'
            return 0
            ;;
        opcache.enable_cli|opcache.optimization_level)
            printf 'unsupported-opcache-ini\trequires Zend OPcache configuration; PTN native compiler has no OPcache runtime layer\n'
            return 0
            ;;
        default_charset|serialize_precision)
            printf 'unsupported-scalar-format-ini\trequires runtime scalar/string formatting default %s; PTN only models bounded precision/display ini state\n' "$key"
            return 0
            ;;
        sendmail_path|sys_temp_dir)
            printf 'unsupported-host-path-ini\trequires host path ini %s; PTN runtime does not yet model this process-global configuration\n' "$key"
            return 0
            ;;
    esac

    return 1
}

ptn_phpt_has_external_service_harness() {
    local path=$1

    grep -Eiq \
        'http_server(_skipif)?|server\.inc|skipifconnectfailure|PHP_TEST_SHARED_EXTENSIONS|TEST_PHP_(MYSQL|PGSQL|LDAP|ODBC|FTP|SNMP)|getaddrinfo|localhost:[0-9]|127\.0\.0\.1|::1' \
        "$path"
}

ptn_phpt_has_process_boundary() {
    local path=$1

    awk '
        /^--[A-Z0-9_]+--[[:space:]]*$/ {
            section = $0
            sub(/^--/, "", section)
            sub(/--[[:space:]]*$/, "", section)
            active = section == "FILE" || section == "CLEAN" || section == "SKIPIF"
            next
        }
        active && /(^|[^[:alnum:]_\$])(proc_open|proc_close|proc_get_status|proc_terminate|proc_nice|popen|pclose|exec|system|passthru|shell_exec)[[:space:]]*\(/ {
            found = 1
            exit
        }
        END { exit found ? 0 : 1 }
    ' "$path"
}

ptn_phpt_first_unsupported_section() {
    local path=$1
    local unsupported
    unsupported=$(ptn_phpt_unsupported_sections)
    ptn_phpt_first_section_in_csv "$path" "$unsupported"
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
            return line ~ /(^|[^[:alnum:]_$\\])array_(all|any|change_key_case|chunk|column|combine|count_values|diff|diff_assoc|diff_key|diff_uassoc|diff_ukey|fill|fill_keys|find|find_key|first|flip|intersect|intersect_assoc|intersect_key|intersect_uassoc|intersect_ukey|is_list|key_exists|key_first|key_last|keys|last|map|merge|merge_recursive|pad|pop|product|push|reduce|replace|replace_recursive|reverse|search|shift|slice|splice|sum|udiff|udiff_assoc|udiff_uassoc|uintersect|uintersect_assoc|uintersect_uassoc|unique|unshift|values|walk|walk_recursive)[[:space:]]*\([^)]*[(,][[:space:]]*[a-z_][a-z0-9_]*[[:space:]]*:[^:]/
        }
        function ptn_has_by_reference_parameter(line) {
            return line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&?[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\([^)]*&[[:space:]]*(\.\.\.)?[[:space:]]*\$[a-z_]/
        }
        function ptn_is_function_declaration(line) {
            return line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&?[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\(/
        }
        function ptn_has_spl_iterator_object(line) {
            return line ~ /(^|[^[:alnum:]_$])new[[:space:]]+(arrayiterator|arrayobject|splfixedarray|limititerator|iteratoriterator|regexiterator|callbackfilteriterator)([^[:alnum:]_]|$)/
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
                if (!ptn_heredoc_nowdoc && $0 ~ /[$]([$A-Za-z_{])/) {
                    print "unsupported-string-parser\trequires heredoc interpolation inside `<<<` string bodies, outside PTN modeled string parser"
                    found = 1
                    exit
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
                ptn_function_body_pending = 0
                ptn_static_local_context = 1
            } else if (ptn_function_body_pending && ptn_line_open_braces == 0 && line ~ /;/) {
                ptn_function_body_pending = 0
            }
            if (line ~ /(^|[^[:alnum:]_$])interface[[:space:]]+[a-z_\\][a-z0-9_\\]*/) {
                saw_interface = 1
            }
            if (!saw_anonymous_class && saw_interface && match(line, /function[[:space:]]+([a-z_][a-z0-9_]*)[[:space:]]*[(]/, method_match)) {
                override_interface_methods[method_match[1]] = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])(new[[:space:]]+fiber|fiber[[:space:]]*::)/) {
                print "unsupported-generator-runtime\trequires Fiber coroutine runtime and by-reference return/getReturn boundary, outside PTN execution model"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(spl_autoload_[a-z0-9_]*|__autoload)[[:space:]]*\(/) {
                print "unsupported-autoload-metadata\trequires runtime class autoload symbol-table mutation, outside PTN static class metadata"
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
                if (saw_abstract_class) {
                    print "unsupported-anonymous-class\trequires anonymous class abstract parent implementation diagnostics, outside PTN modeled anonymous class subset"
                    found = 1
                    exit
                }
                if (saw_get_class) {
                    print "unsupported-anonymous-class\trequires PHP hidden-suffix anonymous class generated names, outside PTN modeled anonymous class subset"
                    found = 1
                    exit
                }
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])class_alias[[:space:]]*[(]/) {
                print "unsupported-anonymous-class\trequires anonymous class runtime class_alias metadata, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])closure[[:space:]]*::[[:space:]]*bind[[:space:]]*[(]/) {
                print "unsupported-anonymous-class\trequires Closure::bind() scope binding for anonymous class instances, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])trigger_error[[:space:]]*[(]/) {
                print "unsupported-anonymous-class\trequires trigger_error() diagnostics containing anonymous class generated names, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])get_class[[:space:]]*[(]/) {
                print "unsupported-anonymous-class\trequires PHP hidden-suffix anonymous class generated names, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /[$][A-Za-z_][A-Za-z0-9_]*[[:space:]]*::/) {
                print "unsupported-anonymous-class\trequires dynamic static member access through anonymous class objects, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (saw_anonymous_class && line ~ /(^|[^[:alnum:]_$])abstract[[:space:]]+(public|protected|private|static|function)/) {
                print "unsupported-anonymous-class\trequires anonymous class abstract method diagnostics, outside PTN modeled anonymous class subset"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])use[[:space:]]+[a-z_\\][a-z0-9_\\]*(.*[,][[:space:]]*[a-z_\\][a-z0-9_\\]*)?[[:space:]]*\{/) {
                print "unsupported-trait-declaration\trequires trait adaptation aliases, precedence, and conflict diagnostics"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\(/) {
                ptn_by_ref_function_context = 1
            }
            if (ptn_has_by_reference_parameter(line)) {
                ptn_call_unpack_by_reference_context = 1
            }
            if (ptn_has_spl_iterator_object(line) && ptn_spread_context(line) == "call") {
                print "unsupported-call-unpacking-traversable\trequires Traversable/SPL iterator argument unpacking, outside PTN array-only call unpacking runtime"
                found = 1
                exit
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
                print "unsupported-generator-runtime\trequires generator yield-from delegation diagnostics, return-value propagation, and by-reference rejection, outside PTN generator runtime"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])yield([[:space:];(),]|$)/) {
                if (ptn_by_ref_function_context && line ~ /yield[^;]*[^=!<>]=([^=>]|$)/) {
                    ptn_defer_generator_reason("requires generator suspension timing for by-reference yielded assignment expressions, outside PTN collected generator runtime")
                    next
                }
                if (ptn_generator_foreach_context) {
                    ptn_defer_generator_reason("requires generator suspension cleanup for live foreach variables and premature close, outside PTN generator runtime")
                }
                next
            }
            if (ptn_deferred_generator_reason ~ /generator suspension cleanup/ &&
                ptn_spread_context(line) == "call" &&
                line ~ /[.][.][.][[:space:]]*[a-z_\\][a-z0-9_\\]*[[:space:]]*[(]/) {
                ptn_deferred_generator_reason = ""
            }
            if (!ptn_static_local_context && ptn_class_body_depth == 0 &&
                line ~ /(^|[;{}])[[:space:]]*static[[:space:]]+\$[a-z_]/) {
                print "unsupported-function-state\trequires top-level static variable diagnostics, outside PTN function-local static storage model"
                found = 1
                exit
            }
            if (ptn_function_body_depth > 0 && ptn_line_close_braces > 0) {
                ptn_function_body_depth -= ptn_line_close_braces
                if (ptn_function_body_depth < 0) {
                    ptn_function_body_depth = 0
                }
            }
            if (ptn_class_body_depth > 0 && ptn_line_close_braces > 0) {
                ptn_class_body_depth -= ptn_line_close_braces
                if (ptn_class_body_depth < 0) {
                    ptn_class_body_depth = 0
                }
            }
            if (line ~ /(^|[^[:alnum:]_$])foreach[[:space:]]*\([^)]*\$[a-z_][a-z0-9_]*[[:space:]]*\[[[:space:]]*\][^)]*as([^[:alnum:]_]|$)/) {
                print "unsupported-expression-diagnostics\trequires array-append read diagnostics (`[]` in read context), outside PTN expression diagnostics"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])foreach[[:space:]]*\([^)]*as[^)]*\$this([^[:alnum:]_]|$)/) {
                print "unsupported-expression-diagnostics\trequires foreach assignment diagnostics for `$this`, outside PTN special-variable assignment diagnostics"
                found = 1
                exit
            }
            if (line ~ /[$][{][[:space:]]*[$]/ || line ~ /(^|[;{}])[[:space:]]*global[[:space:]][^;]*[$][$]/ || line ~ /[$][$][a-z_][a-z0-9_]*([[:space:]]*\[[^]]*\])*[[:space:]]*([+*\/%.&|^-]?=|[?][?]=)/) {
                print "unsupported-dynamic-symbol\trequires variable-variable symbol-table mutation, outside PTN modeled dynamic reads"
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

    if ptn_phpt_section "$path" FILE | LC_ALL=C grep -E '#\[[^]]*\\?Override([^[:alnum:]_]|$)' >/dev/null; then
        ptn_has_override_attribute=1
    fi

    ptn_phpt_section "$path" FILE | LC_ALL=C awk -v ptn_path="$path" -v ptn_has_override_attribute="$ptn_has_override_attribute" '
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
        {
            line = ptn_php_code_line($0)
            if (implemented_modifier_diagnostic_seen) {
                next
            }
            readonly_class_context = readonly_class_depth > 0 || readonly_class_pending || line ~ /(^|[^[:alnum:]_$])readonly[[:space:]]+class[[:space:]]+[a-z_\\]/
            if (line ~ /(^|[[:space:]])(public|protected|private|var)?[[:space:]]*readonly[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/ ||
                line ~ /(^|[[:space:]])readonly[[:space:]]+(public|protected|private|var)?[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                readonly_property_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])enum[[:space:]]+[a-z_\\]/) {
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
            if (!implemented_modifier_diagnostic &&
                line ~ /(^|[^[:alnum:]_$])final[[:space:]]+(function|static|public|protected|private|abstract)([^[:alnum:]_$]|$)/) {
                print "unsupported-class-contract-metadata\trequires final class/method override metadata, outside PTN modeled class dispatch"
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
            if (line ~ /(^|[[:space:]])((private|protected)[[:space:]]+static|static[[:space:]]+(private|protected))[[:space:]]+\$[a-z_]/) {
                print "unsupported-property-visibility-metadata\trequires non-public static property visibility metadata, outside PTN modeled property visibility"
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
            if (line ~ /function[[:space:]]+&?[[:space:]]*__(serialize|unserialize|sleep|wakeup)[[:space:]]*\(/) {
                print "unsupported-magic-method-metadata\trequires magic method dispatch/reflection metadata, outside PTN modeled object/class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(spl_autoload_[a-z0-9_]*|__autoload)[[:space:]]*\(/) {
                print "unsupported-autoload-metadata\trequires runtime class autoload symbol-table mutation, outside PTN static class metadata"
                found = 1
                exit
            }
            if (line ~ /->[[:space:]]*getclosurethis[[:space:]]*\(/) {
                print "unsupported-reflection-metadata\trequires ReflectionFunction closure binding metadata (`getClosureThis()`), outside PTN modeled reflection metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$\\])new[[:space:]]+\\?reflectionproperty[[:space:]]*\(/ ||
                line ~ /(^|[^[:alnum:]_$\\])reflectionproperty[[:space:]]*::/) {
                print "unsupported-internal-reflection-metadata\trequires ReflectionProperty metadata/mutation support, outside PTN modeled reflection metadata"
                found = 1
                exit
            }
            if (line ~ /->[[:space:]]*getattributes[[:space:]]*\(/ ||
                line ~ /(^|[^[:alnum:]_$\\])attribute[[:space:]]*::/ ||
                line ~ /(^|[^[:alnum:]_$\\])new[[:space:]]+\\?(deprecated|nodiscard)([^[:alnum:]_]|$)/) {
                print "unsupported-internal-attribute-metadata\trequires internal attribute/reflection metadata such as Reflection*::getAttributes(), Attribute constants, Deprecated, or NoDiscard"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(get_defined_functions|get_declared_classes)[[:space:]]*\(/ ||
                line ~ /->[[:space:]]*newinstancewithoutconstructor[[:space:]]*\(/) {
                print "unsupported-internal-reflection-metadata\trequires complete internal arginfo/class registry reflection, outside PTN modeled metadata"
                found = 1
                exit
            }
            if (!ptn_has_override_attribute &&
                line ~ /function[[:space:]]+__construct[[:space:]]*\([^)]*(public|protected|private|readonly)[[:space:]]+/) {
                print "unsupported-property-promotion-metadata\trequires constructor property promotion metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (!ptn_has_override_attribute &&
                line ~ /(^|[,([:space:]])(public|protected|private|readonly)[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/ &&
                line !~ /;/) {
                print "unsupported-property-promotion-metadata\trequires constructor property promotion metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if ((readonly_class_context || readonly_property_seen) &&
                (line ~ /=[[:space:]]*&[[:space:]]*\$[a-z_][a-z0-9_]*->[a-z_][a-z0-9_]*/ ||
                    line ~ /->[a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*&/ ||
                    line ~ /->[a-z_][a-z0-9_]*[[:space:]]*(\[|\+\+|--)/)) {
                print "unsupported-readonly-property-metadata\trequires indirect readonly property mutation diagnostics, outside PTN modeled readonly property subset"
                found = 1
                exit
            }
            if (!ptn_has_override_attribute &&
                !readonly_class_context &&
                line !~ /(^|[[:space:]])(public|protected|private)[[:space:]]+readonly[[:space:]]+/ &&
                line !~ /(^|[[:space:]])readonly[[:space:]]+(public|protected|private)[[:space:]]+/ &&
                line !~ /(^|[[:space:]])(public|protected|private|var)[[:space:]]+static[[:space:]]+[$][a-z_]/ &&
                line !~ /(^|[[:space:]])static[[:space:]]+(public|protected|private|var)?[[:space:]]*[$][a-z_]/ &&
                line ~ /(^|[[:space:]])(public|protected|private|var|static|readonly)[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                print "unsupported-typed-property-metadata\trequires typed property metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (!ptn_has_override_attribute &&
                line ~ /(^|[[:space:]])(public|protected|private)[[:space:]]+static[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                print "unsupported-typed-property-metadata\trequires typed static property metadata, outside PTN modeled static property declarations"
                found = 1
                exit
            }
            if (line ~ /[.][.][.]/ &&
                (line ~ /(^|[[:space:]])(public|protected|private)?[[:space:]]*const[[:space:]]+[a-z_][a-z0-9_]*[[:space:]]*=/ ||
                 line ~ /(^|[[:space:]])(public|protected|private|var)?[[:space:]]*static[[:space:]]+\$[a-z_][a-z0-9_]*[[:space:]]*=/)) {
                print "unsupported-class-constant-metadata\trequires class-scope constant/static-property default unpack evaluation, outside PTN modeled class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])(private|protected)[[:space:]]+const[[:space:]]+/) {
                print "unsupported-class-constant-metadata\trequires non-public class constant metadata, outside PTN modeled class constants"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])const[[:space:]]+[a-z_\\][a-z0-9_\\|?]*[[:space:]]+[a-z_][a-z0-9_]*[[:space:]]*=/) {
                print "unsupported-class-constant-metadata\trequires typed class constant metadata, outside PTN modeled class constants"
                found = 1
                exit
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
            if (line ~ /(^|[^[:alnum:]_$])(debug_backtrace|debug_print_backtrace)[[:space:]]*\(/) {
                print "unsupported-diagnostics-runtime\trequires debug_backtrace()/debug_print_backtrace() stack-frame snapshots, outside PTN modeled call-frame diagnostics"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(set_error_handler|restore_error_handler|set_exception_handler|restore_exception_handler)[[:space:]]*\(/) {
                print "unsupported-diagnostics-runtime\trequires user error/exception handler state and fallback dispatch, outside PTN modeled diagnostic channel"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])assert_options[[:space:]]*\(/) {
                print "unsupported-assertion-runtime\trequires assert_options() mode/callback state, outside PTN modeled catchable AssertionError subset"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])namespace[[:space:]]+[a-z_\\][a-z0-9_\\]*[[:space:]]*;/) {
                namespace_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])assert[[:space:]]*\(/) {
                if (namespace_seen) {
                    print "unsupported-assertion-runtime\trequires namespace-aware assertion function resolution and diagnostic rendering, outside PTN modeled assertion state"
                    found = 1
                    exit
                }
                if (line ~ /[?][?]=/) {
                    print "unsupported-assertion-runtime\trequires assertion expression lvalue mode interaction, outside PTN modeled assertion lowering"
                    found = 1
                    exit
                }
                if (line ~ /assert[[:space:]]*\([[:space:]]*0[[:space:]]*&&.*function[[:space:]]*\(/) {
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
            if (line ~ /(^|[^[:alnum:]_$])array_splice[[:space:]]*\(/) {
                array_splice_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])function[[:space:]]+__destruct[[:space:]]*\(/) {
                destructor_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])global[[:space:]]+\$/ || line ~ /\$globals[[:space:]]*\[/) {
                global_state_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])stream_wrapper_(register|unregister|restore)[[:space:]]*\(/) {
                print "unsupported-internal\trequires user stream wrapper registration and stream callback dispatch, outside PTN modeled stream/resource runtime"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])foreach[[:space:]]*\([^)]*as[^)]*&/) {
                byref_foreach = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])(array_shift|array_unshift)[[:space:]]*\(/) {
                positional_mutator = 1
            }
        }
        END {
            if (!found && array_splice_seen && destructor_seen && global_state_seen) {
                print "unsupported-internal\trequires array_splice() destructor reentrancy detection when element destruction mutates global array state, outside PTN modeled array_splice helper"
                found = 1
            }
            if (!found && byref_foreach && positional_mutator) {
                print "unsupported-internal\trequires by-reference foreach iterator-pointer preservation under positional array mutation, outside PTN foreach iterator model"
                found = 1
            }
            exit found ? 0 : 1
        }
    '
    local -a ptn_status=("${PIPESTATUS[@]}")
    return "${ptn_status[1]}"
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

    if ptn_phpt_has_external_service_harness "$path"; then
        printf 'external-service\trequires external service or php-src server harness\n'
        return 0
    fi

    if [[ "$rel" == sapi/* ]]; then
        printf 'sapi-behavior\texercises php-src SAPI executable behavior outside PTN script execution\n'
        return 0
    fi

    if [[ "$rel" == tests/run-test/* ]]; then
        printf 'harness-self-test\texercises php-src run-tests.php harness behavior instead of PTN PHP semantics\n'
        return 0
    fi

    if ptn_phpt_has_process_boundary "$path"; then
        printf 'process-boundary\trequires child-process execution/control and pipe semantics outside PTN native runtime boundary\n'
        return 0
    fi

    if ptn_phpt_csv_contains_ci "INI" "$sections"; then
        if value=$(ptn_phpt_first_unsupported_ini "$path"); then
            if ptn_phpt_unsupported_ini_blocker "$value"; then
                return 0
            fi
            printf 'unsupported-ini\trequires unsupported ini setting %s; modeled ini keys: %s\n' \
                "$value" "$(ptn_phpt_supported_ini)"
            return 0
        fi
    fi

    if ptn_phpt_classify_harness_programs && ptn_phpt_csv_contains_ci "SKIPIF" "$sections"; then
        if value=$(ptn_phpt_modeled_skipif_precondition "$path"); then
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

    if value=$(ptn_phpt_first_section_in_sections_csv "$sections" "$(ptn_phpt_unsupported_sections)"); then
        printf 'sapi-behavior\trequires unsupported PHPT section --%s--\n' "$value"
        return 0
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

    if value=$(ptn_phpt_first_unsupported_runtime_diagnostics_surface "$path"); then
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
