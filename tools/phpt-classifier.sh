#!/usr/bin/env bash

# Shared PHPT preflight classification for PTN measurement runs.
#
# The classifier filters only PHPT harness/environment requirements that PTN
# does not currently model: extension availability, unsupported ini/runtime
# modes, SAPI/request sections, external service harnesses, and upstream XFAILs.
# Generic PHP semantic gaps remain runnable and should surface as PTN failures.

PTN_PHPT_SUPPORTED_EXTENSIONS_DEFAULT="Core,date,pcre,standard"
PTN_PHPT_SUPPORTED_INI_DEFAULT="date.timezone,display_errors,error_reporting,extension_dir,pcre.backtrack_limit,precision,zend.assertions"
PTN_PHPT_UNSUPPORTED_SECTIONS_DEFAULT="ARGS,CGI,COOKIE,COOKIE_RAW,EXPECTHEADERS,GET,HEADERS,POST,POST_RAW,PUT,REDIRECTTEST,REQUEST,STDIN"

ptn_phpt_supported_extensions() {
    printf '%s\n' "${PTN_PHPT_SUPPORTED_EXTENSIONS:-$PTN_PHPT_SUPPORTED_EXTENSIONS_DEFAULT}"
}

ptn_phpt_supported_ini() {
    printf '%s\n' "${PTN_PHPT_SUPPORTED_INI:-$PTN_PHPT_SUPPORTED_INI_DEFAULT}"
}

ptn_phpt_unsupported_sections() {
    printf '%s\n' "${PTN_PHPT_UNSUPPORTED_SECTIONS:-$PTN_PHPT_UNSUPPORTED_SECTIONS_DEFAULT}"
}

ptn_phpt_lower() {
    printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
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

ptn_phpt_has_external_service_harness() {
    local path=$1

    grep -Eiq \
        'http_server(_skipif)?|server\.inc|skipifconnectfailure|PHP_TEST_SHARED_EXTENSIONS|TEST_PHP_(MYSQL|PGSQL|LDAP|ODBC|FTP|SNMP)|getaddrinfo|localhost:[0-9]|127\.0\.0\.1|::1' \
        "$path"
}

ptn_phpt_first_unsupported_section() {
    local path=$1
    local unsupported
    unsupported=$(ptn_phpt_unsupported_sections)
    local section

    while IFS= read -r section; do
        if ptn_phpt_csv_contains_ci "$section" "$unsupported"; then
            printf '%s\n' "$section"
            return 0
        fi
    done < <(ptn_phpt_sections "$path")

    return 1
}

ptn_phpt_classify_row() {
    local row=$1
    local path=$2
    local php_src=${3:-}
    local rel=$row
    local value

    if [[ -n "$php_src" ]]; then
        rel=$(ptn_phpt_manifest_row "$row" "$php_src")
    fi

    if value=$(ptn_phpt_first_unsupported_extension "$rel" "$path"); then
        printf 'unsupported-extension\trequires unavailable PTN extension %s; modeled extensions: %s\n' \
            "$value" "$(ptn_phpt_supported_extensions)"
        return 0
    fi

    if ptn_phpt_has_external_service_harness "$path"; then
        printf 'external-service\trequires external service or php-src server harness\n'
        return 0
    fi

    if [[ "$rel" == sapi/* ]]; then
        printf 'sapi-behavior\texercises php-src SAPI executable behavior outside PTN script execution\n'
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_ini "$path"); then
        printf 'unsupported-ini\trequires unsupported ini setting %s; modeled ini keys: %s\n' \
            "$value" "$(ptn_phpt_supported_ini)"
        return 0
    fi

    if value=$(ptn_phpt_first_unsupported_section "$path"); then
        printf 'sapi-behavior\trequires unsupported PHPT section --%s--\n' "$value"
        return 0
    fi

    if ptn_phpt_has_section "$path" XFAIL; then
        printf 'intentional-nongoal\tupstream XFAIL row is excluded from PTN pass-count campaign\n'
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
