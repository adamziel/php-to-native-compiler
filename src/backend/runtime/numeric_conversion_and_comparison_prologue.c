static PTN_UNUSED void ptn_runtime_init_function_frame(PtnRuntime *runtime, PtnRuntime *caller_runtime) {
    ptn_symbols_init(&runtime->symbols);
    ptn_symbols_init(&runtime->owned_constants);
    runtime->constants = caller_runtime->constants;
    ptn_diagnostics_init(&runtime->diagnostics, stderr);
}

static void ptn_runtime_free(PtnRuntime *runtime) {
    ptn_symbols_free(&runtime->owned_constants);
    ptn_symbols_free(&runtime->symbols);
}

static PTN_UNUSED void ptn_runtime_write_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    ptn_symbols_set(&runtime->symbols, name, value);
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnValue value;
    if (ptn_symbols_get(&runtime->symbols, name, &value)) {
        return value;
    }
    ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
    return ptn_null();
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_variable_quiet(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_symbols_get(&runtime->symbols, name, &value)) {
        return ptn_lookup_found(value);
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED int ptn_runtime_variable_is_set(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    return ptn_symbols_get(&runtime->symbols, name, &value) && value.type != PTN_NULL;
}

static PTN_UNUSED int ptn_runtime_variable_is_empty(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    return !ptn_symbols_get(&runtime->symbols, name, &value) || !ptn_is_truthy(value);
}

static PTN_UNUSED void ptn_runtime_define_constant(PtnRuntime *runtime, const char *name, PtnValue value) {
    ptn_symbols_set(runtime->constants, name, value);
}

static PTN_UNUSED PtnNumber ptn_number_int(int64_t integer) {
    PtnNumber number;
    number.type = PTN_NUMBER_INT;
    number.integer = integer;
    number.floating = (double)integer;
    return number;
}

static PTN_UNUSED PtnNumber ptn_number_float(double floating) {
    PtnNumber number;
    number.type = PTN_NUMBER_FLOAT;
    number.integer = 0;
    number.floating = floating;
    return number;
}

static PTN_UNUSED int ptn_contains_float_marker(const char *start, const char *end) {
    for (const char *cursor = start; cursor < end; cursor++) {
        if (*cursor == '.' || *cursor == 'e' || *cursor == 'E') {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED PtnNumber ptn_string_to_number(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return ptn_number_int(0);
    }

    char *int_end = NULL;
    errno = 0;
    long long integer = strtoll(start, &int_end, 10);
    int int_errno = errno;

    char *float_end = NULL;
    errno = 0;
    double floating = strtod(start, &float_end);
    if (float_end == start) {
        return ptn_number_int(0);
    }

    if (int_end == float_end && int_errno != ERANGE && !ptn_contains_float_marker(start, int_end)) {
        return ptn_number_int((int64_t)integer);
    }
    return ptn_number_float(floating);
}

static PTN_UNUSED PtnNumber ptn_to_number(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return ptn_number_int(0);
        case PTN_BOOL:
            return ptn_number_int(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_number_int(value.as.integer);
        case PTN_FLOAT:
            return ptn_number_float(value.as.floating);
        case PTN_STRING:
            return ptn_string_to_number(value.as.string);
        case PTN_ARRAY:
            return ptn_number_int(value.as.array->len == 0 ? 0 : 1);
    }
    return ptn_number_int(0);
}

static PTN_UNUSED int ptn_fast_integer_value(PtnValue value, int64_t *integer) {
    switch (value.type) {
        case PTN_NULL:
            *integer = 0;
            return 1;
        case PTN_BOOL:
            *integer = value.as.boolean ? 1 : 0;
            return 1;
        case PTN_INT:
            *integer = value.as.integer;
            return 1;
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_ARRAY:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_fast_scalar_double(PtnValue value, double *number) {
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        *number = (double)integer;
        return 1;
    }
    if (value.type == PTN_FLOAT) {
        *number = value.as.floating;
        return 1;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_negate(PtnValue value) {
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        if (integer == INT64_MIN) {
            return ptn_float(-(double)integer);
        }
        return ptn_int(-integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(-value.as.floating);
    }

    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(-number.floating);
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(-(double)number.integer);
    }
    return ptn_int(-number.integer);
}

static PTN_UNUSED PtnValue ptn_positive(PtnValue value) {
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_int(integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(value.as.floating);
    }

    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED int ptn_is_truthy(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return 0;
        case PTN_BOOL:
            return value.as.boolean != 0;
        case PTN_INT:
            return value.as.integer != 0;
        case PTN_FLOAT:
            return value.as.floating != 0.0;
        case PTN_STRING:
            return value.as.string[0] != '\0' && strcmp(value.as.string, "0") != 0;
        case PTN_ARRAY:
            return value.as.array->len != 0;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_not(PtnValue value) {
    return ptn_bool(!ptn_is_truthy(value));
}

static PTN_UNUSED PtnValue ptn_cast_int(PtnValue value) {
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_int(integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_int((int64_t)value.as.floating);
    }

    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_int((int64_t)number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED PtnValue ptn_cast_float(PtnValue value) {
    double fast_number = 0.0;
    if (ptn_fast_scalar_double(value, &fast_number)) {
        return ptn_float(fast_number);
    }

    PtnNumber number = ptn_to_number(value);
    return ptn_float(number.floating);
}

static PTN_UNUSED void ptn_abort_arithmetic_error(const char *message) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(1);
}

static PTN_UNUSED void ptn_abort_type_error_at(const char *message, const char *path, size_t line) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputs(" in ", stderr);
    fputs(path, stderr);
    fputs(" on line ", stderr);
    fprintf(stderr, "%zu", line);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED void ptn_abort_control_error(const char *message, const char *path, size_t line) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputs(" in ", stderr);
    fputs(path, stderr);
    fputs(" on line ", stderr);
    fprintf(stderr, "%zu", line);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED int ptn_is_number_type(PtnValue value) {
    return value.type == PTN_INT || value.type == PTN_FLOAT;
}

static PTN_UNUSED int ptn_string_may_be_numeric(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }
    if (isdigit((unsigned char)*start) || *start == '+' || *start == '-' || *start == '.') {
        return 1;
    }
    return *start == 'i' || *start == 'I' || *start == 'n' || *start == 'N';
}

static PTN_UNUSED int ptn_is_numeric_string(const char *string, double *number) {
    if (!ptn_string_may_be_numeric(string)) {
        return 0;
    }

    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    char *end = NULL;
    double parsed = strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    if (*end != '\0') {
        return 0;
    }
    *number = parsed;
    return 1;
}

static PTN_UNUSED int ptn_comparison_numeric_value(PtnValue value, double *number) {
    switch (value.type) {
        case PTN_INT:
            *number = (double)value.as.integer;
            return 1;
        case PTN_FLOAT:
            *number = value.as.floating;
            return 1;
        case PTN_STRING:
            return ptn_is_numeric_string(value.as.string, number);
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_ARRAY:
            return 0;
    }
    return 0;
}

enum {
    PTN_COMPARE_LESS = -1,
    PTN_COMPARE_EQUAL = 0,
    PTN_COMPARE_GREATER = 1,
    PTN_COMPARE_UNORDERED = 2
};

static PTN_UNUSED int ptn_compare_numbers(double left, double right) {
    if (isnan(left) || isnan(right)) {
        return PTN_COMPARE_UNORDERED;
    }
    if (left < right) {
        return PTN_COMPARE_LESS;
    }
    if (left > right) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_integers(int64_t left, int64_t right) {
    if (left < right) {
        return PTN_COMPARE_LESS;
    }
    if (left > right) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_strings(const char *left, const char *right) {
    int compared = strcmp(left, right);
    return compared < 0 ? -1 : (compared > 0 ? 1 : 0);
}

static PTN_UNUSED void ptn_number_value_to_string(PtnValue value, char *buffer, size_t buffer_len) {
    if (value.type == PTN_INT) {
