#!/usr/bin/env bash

# Shared PHPT preflight classification for PTN measurement runs.
#
# The classifier filters only PHPT harness/environment requirements that PTN
# does not currently model: extension availability, unsupported ini/runtime
# modes, SAPI/request sections, external service harnesses, process-boundary
# rows, broad harness cleanup/setup sections, opt-in harness preconditions,
# noisy upstream rows, broad unsupported language surfaces, and upstream XFAILs.
# Generic PHP semantic gaps inside the modeled surface remain runnable and
# should surface as PTN failures.

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
        {
            if (ptn_has_php_attribute_syntax($0)) {
                print "unsupported-language\trequires PHP attribute syntax (`#[...]`) and reflection metadata, outside PTN parser/metadata model"
                found = 1
                exit
            }
            line = ptn_php_code_line($0)
            if (line ~ /<<<[[:space:]]*["'\''"]?[a-z_][a-z0-9_]*/) {
                print "unsupported-language\trequires heredoc/nowdoc string syntax (`<<<`), outside PTN modeled string parser"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])new[[:space:]]+class([^[:alnum:]_]|$)/) {
                print "unsupported-language\trequires anonymous class syntax (`new class`), outside PTN modeled class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])interface[[:space:]]+[a-z_\\]/) {
                print "unsupported-language\trequires interface declarations, outside PTN modeled class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])implements[[:space:]]+[a-z_\\]/) {
                print "unsupported-language\trequires interface implementation checks, outside PTN modeled class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])trait[[:space:]]+[a-z_\\]/) {
                print "unsupported-language\trequires trait declarations, outside PTN modeled class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])yield([[:space:];(),]|$)/) {
                print "unsupported-language\trequires generator/yield lowering, outside PTN function and iterator runtime"
                found = 1
                exit
            }
            if (line ~ /(^|[,(])[[:space:]]*\?[[:space:]]*([a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:].&]*\$[a-z_]/ ||
                line ~ /\)[[:space:]]*:[[:space:]]*\?[[:space:]]*([a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)([^[:alnum:]_]|$)/) {
                print "unsupported-language\trequires nullable type-hint metadata and coercion (`?T`), outside PTN modeled type hints"
                found = 1
                exit
            }
            if (line ~ /\)[[:space:]]*:[[:space:]]*never([^[:alnum:]_]|$)/) {
                print "unsupported-language\trequires `never` return type control-flow validation, outside PTN modeled type hints"
                found = 1
                exit
            }
            if (line ~ /(^|[;{}])[[:space:]]*static[[:space:]]+\$[a-z_]/) {
                print "unsupported-language\trequires static local variables, outside PTN function-local static storage model"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])foreach[[:space:]]*\([^)]*\$[a-z_][a-z0-9_]*[[:space:]]*\[[[:space:]]*\][^)]*as([^[:alnum:]_]|$)/) {
                print "unsupported-language\trequires array-append read diagnostics (`[]` in read context), outside PTN expression diagnostics"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])foreach[[:space:]]*\([^)]*as[^)]*\$this([^[:alnum:]_]|$)/) {
                print "unsupported-language\trequires foreach assignment diagnostics for `$this`, outside PTN special-variable assignment diagnostics"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])unset[[:space:]]*\([[:space:]]*[$][$]/) {
                print "unsupported-language\trequires plain variable-variable unset, outside PTN modeled dynamic-root array unset/write support"
                found = 1
                exit
            }
            if ($0 ~ /\.\.\./) {
                declaration = line ~ /(^|[^[:alnum:]_$])(function|fn)[[:space:]]*([a-z_\\][a-z0-9_\\]*)?[[:space:]]*\([^)]*\.\.\./
                if (!declaration) {
                    print "unsupported-language\trequires call-site or array unpacking (`...`), outside PTN modeled call/array lowering"
                    found = 1
                    exit
                }
            }
        }
        END { exit found ? 0 : 1 }
    '
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
                print "unsupported-class-metadata\trequires enum declarations and case metadata, outside PTN modeled class metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])abstract[[:space:]]+(class|function|static|public|protected|private|[a-z_\\])/) {
                print "unsupported-class-metadata\trequires abstract class/method contract metadata, outside PTN modeled class dispatch"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])final[[:space:]]+(class|function|static|public|protected|private|[a-z_\\])/) {
                print "unsupported-class-metadata\trequires final class/method override metadata, outside PTN modeled class dispatch"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])readonly[[:space:]]+static([^[:alnum:]_]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])static[[:space:]]+readonly([^[:alnum:]_]|$)/ ||
                line ~ /(^|[^[:alnum:]_$])readonly[[:space:]]+class[^{;]*\{[^}]*static[[:space:]][^;}]*\$/) {
                print "unsupported-class-metadata\trequires readonly static property diagnostics, outside PTN runnable readonly property subset"
                found = 1
                exit
            }
            if (readonly_class_context &&
                line ~ /(^|[[:space:]])(public|protected|private|var)?[[:space:]]*static[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)?[[:space:]]*\$[a-z_]/) {
                print "unsupported-class-metadata\trequires readonly static property diagnostics, outside PTN runnable readonly property subset"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])(private|protected)[[:space:]]+(static[[:space:]]+)?function[[:space:]]+[a-z_]/) {
                print "unsupported-class-metadata\trequires non-public method visibility dispatch and diagnostics, outside PTN modeled method visibility"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])(private|protected)[[:space:]]+(static[[:space:]]+)?\$[a-z_]/) {
                print "unsupported-class-metadata\trequires non-public property visibility metadata, outside PTN modeled property visibility"
                found = 1
                exit
            }
            if (line ~ /function[[:space:]]+__(call|callstatic|get|set|isset|unset|debuginfo|serialize|unserialize|sleep|wakeup|tostring)[[:space:]]*\(/) {
                print "unsupported-class-metadata\trequires unsupported magic method dispatch/reflection metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])(spl_autoload_[a-z0-9_]*|__autoload)[[:space:]]*\(/) {
                print "unsupported-class-metadata\trequires runtime class autoload symbol-table mutation, outside PTN static class metadata"
                found = 1
                exit
            }
            if (line ~ /->[[:space:]]*getclosurethis[[:space:]]*\(/) {
                print "unsupported-class-metadata\trequires ReflectionFunction closure binding metadata (`getClosureThis()`), outside PTN modeled reflection metadata"
                found = 1
                exit
            }
            if (line ~ /(^|[^[:alnum:]_$])get_object_vars[[:space:]]*\(/) {
                print "unsupported-class-metadata\trequires get_object_vars() object property-table export and property array-dimension lvalues, outside PTN modeled object/property metadata"
                found = 1
                exit
            }
            if (line ~ /function[[:space:]]+__construct[[:space:]]*\([^)]*(public|protected|private|readonly)[[:space:]]+/) {
                print "unsupported-class-metadata\trequires constructor property promotion metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (line ~ /(^|[,([:space:]])(public|protected|private|readonly)[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/ &&
                line !~ /;/) {
                print "unsupported-class-metadata\trequires constructor property promotion metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (((readonly_class_context || readonly_property_seen) &&
                (line ~ /=[[:space:]]*&[[:space:]]*\$[a-z_][a-z0-9_]*->[a-z_][a-z0-9_]*/ ||
                    line ~ /->[a-z_][a-z0-9_]*[[:space:]]*=[[:space:]]*&/)) ||
                line ~ /->[a-z_][a-z0-9_]*[[:space:]]*(\[|\+\+|--)/) {
                print "unsupported-class-metadata\trequires indirect readonly property mutation diagnostics, outside PTN modeled readonly property subset"
                found = 1
                exit
            }
            if (!readonly_class_context &&
                line !~ /(^|[[:space:]])(public|protected|private)[[:space:]]+readonly[[:space:]]+/ &&
                line !~ /(^|[[:space:]])readonly[[:space:]]+(public|protected|private)[[:space:]]+/ &&
                line ~ /(^|[[:space:]])(public|protected|private|var|static|readonly)[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                print "unsupported-class-metadata\trequires typed property metadata, outside PTN modeled property declarations"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])(public|protected|private)[[:space:]]+static[[:space:]]+([?]?[a-z_\\][a-z0-9_\\]*|int|float|string|bool|array|object|mixed|iterable)[[:space:]]+\$[a-z_]/) {
                print "unsupported-class-metadata\trequires typed static property metadata, outside PTN modeled static property declarations"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])(private|protected)[[:space:]]+const[[:space:]]+/) {
                print "unsupported-class-metadata\trequires non-public class constant metadata, outside PTN modeled class constants"
                found = 1
                exit
            }
            if (line ~ /(^|[[:space:]])const[[:space:]]+[a-z_\\][a-z0-9_\\|?]*[[:space:]]+[a-z_][a-z0-9_]*[[:space:]]*=/) {
                print "unsupported-class-metadata\trequires typed class constant metadata, outside PTN modeled class constants"
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
}

ptn_phpt_classify_row() {
    local row=$1
    local path=$2
    local php_src=${3:-}
    local rel=$row
    local sections
    local value

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

    if ptn_phpt_classify_harness_programs; then
        if value=$(ptn_phpt_first_section_in_sections_csv "$sections" "$(ptn_phpt_skipif_harness_sections)"); then
            printf 'harness-skipif\trequires PHPT harness precondition section --%s-- evaluated before measured program output\n' "$value"
            return 0
        fi
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
