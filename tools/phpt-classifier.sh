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
PTN_PHPT_SUPPORTED_INI_DEFAULT="date.timezone,display_errors,error_reporting,extension_dir,include_path,pcre.backtrack_limit,precision,zend.assertions"
PTN_PHPT_UNSUPPORTED_SECTIONS_DEFAULT="ARGS,CAPTURE_STDIO,CGI,COOKIE,COOKIE_RAW,EXPECTHEADERS,FILE_EXTERNAL,GET,HEADERS,PHPDBG,POST,POST_RAW,PUT,REDIRECTTEST,REQUEST,STDIN"
PTN_PHPT_ENVIRONMENT_SECTIONS_DEFAULT="ENV"
PTN_PHPT_HARNESS_SECTIONS_DEFAULT="CLEAN"
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
    code=$(ptn_phpt_skipif_code "$path")
    [[ -n "$code" ]] || return 1

    code_without_strings=$(printf '%s\n' "$code" | ptn_phpt_strip_php_strings | ptn_phpt_squash_ws)

    local identifier
    while IFS= read -r identifier; do
        [[ -n "$identifier" ]] || continue
        case "$identifier" in
            if|getenv|die|exit|echo|print|PHP_INT_SIZE|setlocale|LC_ALL|LC_COLLATE|LC_CTYPE|LC_MESSAGES|LC_MONETARY|LC_NUMERIC|LC_TIME)
                ;;
            *)
                return 1
                ;;
        esac
    done < <(printf '%s\n' "$code_without_strings" | grep -Eo '[A-Za-z_][A-Za-z0-9_]*' || true)

    local if_count
    local output_count
    local getenv_count
    local php_int_count
    local setlocale_count
    if_count=$(ptn_phpt_count_matches '(^|[^A-Za-z0-9_])if[[:space:]]*\(' "$code_without_strings")
    output_count=$(ptn_phpt_count_matches '(^|[^A-Za-z0-9_])(die|exit|echo|print)([[:space:]]*\(|[[:space:]])' "$code_without_strings")
    getenv_count=$(ptn_phpt_count_matches 'getenv[[:space:]]*\(' "$code_without_strings")
    php_int_count=$(ptn_phpt_count_matches 'PHP_INT_SIZE' "$code_without_strings")
    setlocale_count=$(ptn_phpt_count_matches 'setlocale[[:space:]]*\(' "$code_without_strings")

    local env_probe_lines
    local parsed_env_count=0
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
                *)
                    return 1
                    ;;
            esac
            parsed_env_count=$((parsed_env_count + 1))
            if [[ -n "${!env_var:-}" ]]; then
                printf 'skipif-precondition\tmodeled static --SKIPIF-- sanitizer/environment gate requires %s unset; current environment sets it\n' "$env_var"
                return 0
            fi
        done <<< "$env_probe_lines"
        modeled_families+=("sanitizer-env")
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

    local recognized_count=$((parsed_env_count + parsed_int_count + parsed_locale_count))
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
        assert.exception)
            printf 'unsupported-assertion-ini\trequires configurable assert.exception assertion mode; PTN currently models catchable AssertionError but not assertion INI/runtime mode switching\n'
            return 0
            ;;
        memory_limit)
            printf 'unsupported-resource-limit-ini\trequires PHP memory_limit parsing/enforcement; PTN has no Zend memory manager/resource limit boundary\n'
            return 0
            ;;
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

    ptn_phpt_section "$path" FILE | LC_ALL=C awk '
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
            return line ~ /(^|[^[:alnum:]_$\\])array_(all|any|change_key_case|chunk|column|combine|count_values|diff|diff_assoc|diff_key|diff_uassoc|diff_ukey|fill|fill_keys|filter|find|find_key|first|flip|intersect|intersect_assoc|intersect_key|intersect_uassoc|intersect_ukey|is_list|key_exists|key_first|key_last|keys|last|map|merge|merge_recursive|pad|pop|product|push|reduce|replace|replace_recursive|reverse|search|shift|slice|splice|sum|udiff|udiff_assoc|udiff_uassoc|uintersect|uintersect_assoc|uintersect_uassoc|unique|unshift|values|walk|walk_recursive)[[:space:]]*\([^)]*[(,][[:space:]]*[a-z_][a-z0-9_]*[[:space:]]*:[^:]/
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
            if (ptn_has_php_attribute_syntax($0)) {
                print "unsupported-attribute-syntax-metadata\trequires PHP attribute syntax (`#[...]`) plus declaration/reflection metadata, outside PTN parser/metadata model"
                found = 1
                exit
            }
            if (ptn_start_heredoc(ptn_php_code_line_raw($0))) {
                next
            }
            line = ptn_php_code_line($0)
            if (line ~ /(^|[^[:alnum:]_$])(new[[:space:]]+fiber|fiber[[:space:]]*::)/) {
                print "unsupported-generator-runtime\trequires Fiber coroutine runtime and by-reference return/getReturn boundary, outside PTN execution model"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])new[[:space:]]+class([^[:alnum:]_]|$)/) {
                print "unsupported-anonymous-class\trequires anonymous class syntax (`new class`), generated class metadata, constructor dispatch, and reflection naming"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])interface[[:space:]]+[a-z_\\]/) {
                print "unsupported-interface-declaration\trequires interface declarations, constants, method contracts, and interface metadata tables"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])implements[[:space:]]+[a-z_\\]/) {
                print "unsupported-interface-implementation\trequires interface implementation checks, method compatibility validation, and runtime interface metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])trait[[:space:]]+[a-z_\\]/) {
                print "unsupported-trait-declaration\trequires trait declarations, composition, aliases, precedence, and conflict diagnostics"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])function[[:space:]]*&[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\(/) {
                ptn_by_ref_function_context = 1
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
                if (ptn_by_ref_function_context) {
                    ptn_defer_generator_reason("requires by-reference generator yield boundary, reference identity, and only-variable-yield diagnostics, outside PTN generator runtime")
                } else if (ptn_generator_foreach_context) {
                    ptn_defer_generator_reason("requires generator suspension cleanup for live foreach variables and premature close, outside PTN generator runtime")
                } else {
                    ptn_defer_generator_reason("requires generator/yield lowering, outside PTN function and iterator runtime")
                }
                next
            }
            if (line ~ /(^|[,(])[[:space:]]*\?[[:space:]]*([a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:].&]*\$[a-z_]/ ||
                line ~ /\)[[:space:]]*:[[:space:]]*\?[[:space:]]*([a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)([^[:alnum:]_]|$)/) {
                print "unsupported-type-hint\trequires nullable type-hint metadata and coercion (`?T`), outside PTN modeled type hints"
                found = 1
                exit
            }
            if (line ~ /\)[[:space:]]*:[[:space:]]*never([^[:alnum:]_]|$)/) {
                print "unsupported-type-hint\trequires `never` return type control-flow validation, outside PTN modeled type hints"
                found = 1
                exit
            }
            if (line ~ /(^|[;{}])[[:space:]]*static[[:space:]]+\$[a-z_]/) {
                print "unsupported-function-state\trequires static local variables, outside PTN function-local static storage model"
                found = 1
                exit
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
            if (line ~ /[$][$]|[$][{][[:space:]]*[$]/) {
                print "unsupported-dynamic-symbol\trequires variable variables and runtime symbol-table lookup/mutation, outside PTN static variable model"
                found = 1
                exit
            }
            if (ptn_has_named_modeled_array_internal_call(line)) {
                print "unsupported-internal-call-binding\trequires named-argument binding for modeled array internal calls, outside PTN internal-call lowering"
                found = 1
                exit
            }
            if ($0 ~ /\.\.\./) {
                declaration = line ~ /(^|[^[:alnum:]_$])(function|fn)[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\([^)]*\.\.\./
                first_class_callable = line ~ /\([[:space:]]*\.\.\.[[:space:]]*\)/
                if (!declaration && !first_class_callable) {
                    spread_context = ptn_spread_context(line)
                    if (spread_context == "array") {
                        print "unsupported-array-unpacking\trequires array literal/destructuring unpacking (`...`), outside PTN modeled array lowering"
                    } else {
                        print "unsupported-call-unpacking\trequires call-site argument unpacking (`...`), outside PTN modeled call lowering"
                    }
                    found = 1
                    exit
                }
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
            if (line ~ /(^|[^[:alnum:]_$])abstract[[:space:]]+(class|function|static|public|protected|private|[a-z_\\])/) {
                print "unsupported-class-contract-metadata\trequires abstract class/method contract metadata, outside PTN modeled class dispatch"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])final[[:space:]]+(class|function|static|public|protected|private|[a-z_\\])/) {
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
            if (line ~ /(^|[[:space:]])(private|protected)[[:space:]]+(static[[:space:]]+)?function[[:space:]]+[a-z_]/) {
                print "unsupported-method-visibility-metadata\trequires non-public method visibility dispatch and diagnostics, outside PTN modeled method visibility"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])(private|protected)[[:space:]]+(static[[:space:]]+)?\$[a-z_]/) {
                print "unsupported-property-visibility-metadata\trequires non-public property visibility metadata, outside PTN modeled property visibility"
                found = 1
                exit
            }
            if (line ~ /function[[:space:]]+__tostring[[:space:]]*\(/) {
                print "unsupported-object-string-conversion-metadata\trequires object-to-string magic conversion metadata shared by comparisons, array helpers, diagnostics, and reflection"
                found = 1
                exit
            }
            if (line ~ /function[[:space:]]+__(call|callstatic|get|set|isset|unset|debuginfo|serialize|unserialize|sleep|wakeup)[[:space:]]*\(/) {
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
            if (line ~ /(^|[^[:alnum:]_$])get_object_vars[[:space:]]*\(/) {
                print "unsupported-object-property-metadata\trequires get_object_vars() object property-table export and property array-dimension lvalues, outside PTN modeled object/property metadata"
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
            if (line ~ /function[[:space:]]+__construct[[:space:]]*\([^)]*(public|protected|private|readonly)[[:space:]]+/) {
                print "unsupported-property-promotion-metadata\trequires constructor property promotion metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (line ~ /(^|[,([:space:]])(public|protected|private|readonly)[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/ &&
                line !~ /;/) {
                print "unsupported-property-promotion-metadata\trequires constructor property promotion metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (((readonly_class_context || readonly_property_seen) &&
                (line ~ /=[[:space:]]*&[[:space:]]*\$[a-z_][a-z0-9_]*->[a-z_][a-z0-9_]*/ ||
                    line ~ /->[a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*&/)) ||
                line ~ /->[a-z_][a-z0-9_]*[[:space:]]*(\[|\+\+|--)/) {
                print "unsupported-readonly-property-metadata\trequires indirect readonly property mutation diagnostics, outside PTN modeled readonly property subset"
                found = 1
                exit
            }
            if (!readonly_class_context &&
                line !~ /(^|[[:space:]])(public|protected|private)[[:space:]]+readonly[[:space:]]+/ &&
                line !~ /(^|[[:space:]])readonly[[:space:]]+(public|protected|private)[[:space:]]+/ &&
                line ~ /(^|[[:space:]])(public|protected|private|var|static|readonly)[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                print "unsupported-typed-property-metadata\trequires typed property metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])(public|protected|private)[[:space:]]+static[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                print "unsupported-typed-property-metadata\trequires typed static property metadata, outside PTN modeled static property declarations"
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
        END { exit found ? 0 : 1 }
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
            if (line ~ /(^|[^[:alnum:]_$])(debug_backtrace|debug_print_backtrace)[[:space:]]*\(/ ||
                line ~ /->[[:space:]]*gettraceasstring[[:space:]]*\(/) {
                print "unsupported-diagnostics-runtime\trequires debug_backtrace()/debug_print_backtrace() stack-frame snapshots, outside PTN modeled call-frame diagnostics"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(set_error_handler|restore_error_handler|set_exception_handler|restore_exception_handler)[[:space:]]*\(/) {
                print "unsupported-diagnostics-runtime\trequires user error/exception handler state and fallback dispatch, outside PTN modeled diagnostic channel"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])new[[:space:]]+errorexception[[:space:]]*\(/ ||
                line ~ /->[[:space:]]*getseverity[[:space:]]*\(/) {
                print "unsupported-diagnostics-runtime\trequires ErrorException severity and trace metadata, outside PTN modeled built-in exception values"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])assert_options[[:space:]]*\(/) {
                print "unsupported-assertion-runtime\trequires assert_options() mode/callback state, outside PTN modeled catchable AssertionError subset"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])ini_set[[:space:]]*\(/ && raw ~ /zend[.]assertions/) {
                print "unsupported-assertion-runtime\trequires runtime zend.assertions mode switching, outside PTN modeled assertion state"
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
            if (line ~ /(^|[^[:alnum:]_$])array_splice[[:space:]]*\(/) {
                array_splice_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])function[[:space:]]+__destruct[[:space:]]*\(/) {
                destructor_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])global[[:space:]]+\$/ || line ~ /\$globals[[:space:]]*\[/) {
                global_state_seen = 1
            }
            if (line ~ /(^|[^[:alnum:]_$])array_multisort[[:space:]]*\(/) {
                print "unsupported-internal\trequires array_multisort() multi-array by-reference sorting and flag/cursor mutation semantics, outside PTN modeled sort helpers"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(u|ua|uk)sort[[:space:]]*\(/) {
                print "unsupported-internal\trequires usort()/uasort()/uksort() user-comparator by-reference sort helpers and COW separation, outside PTN modeled sort helpers"
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
