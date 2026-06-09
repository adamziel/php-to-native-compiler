static PTN_UNUSED void ptn_runtime_init_function_frame(PtnRuntime *runtime, PtnRuntime *caller_runtime) {
    ptn_symbols_init(&runtime->symbols);
    ptn_symbols_init(&runtime->owned_constants);
    runtime->constants = caller_runtime->constants;
    ptn_diagnostics_init(&runtime->diagnostics, stderr);
    runtime->owned_exceptions.active_exception = NULL;
    runtime->owned_exceptions.try_frame = NULL;
    runtime->exceptions = caller_runtime->exceptions;
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

static PTN_UNUSED void ptn_runtime_unset_variable(PtnRuntime *runtime, const char *name) {
    ptn_symbols_unset(&runtime->symbols, name);
}

static PTN_UNUSED PtnException *ptn_exception_new(const char *class_name, const char *message) {
    PtnException *exception = malloc(sizeof(PtnException));
    if (exception == NULL) {
        ptn_abort_out_of_memory();
    }
    exception->class_name = class_name;
    exception->message = ptn_duplicate_string(message);
    return exception;
}

static PTN_UNUSED int ptn_exception_name_equal(const char *left, const char *right) {
    while (*left != '\0' && *right != '\0') {
        int left_byte = tolower((unsigned char)*left);
        int right_byte = tolower((unsigned char)*right);
        if (left_byte != right_byte) {
            return 0;
        }
        left++;
        right++;
    }
    return *left == '\0' && *right == '\0';
}

static PTN_UNUSED int ptn_exception_type_matches_name(const char *class_name, const char *type_name) {
    if (type_name[0] == '\\') {
        type_name++;
    }
    if (ptn_exception_name_equal(class_name, type_name)) {
        return 1;
    }
    if (ptn_exception_name_equal(type_name, "Throwable")) {
        return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_try_frame_push(PtnRuntime *runtime, PtnTryFrame *frame) {
    frame->previous = runtime->exceptions->try_frame;
    runtime->exceptions->try_frame = frame;
}

static PTN_UNUSED void ptn_try_frame_pop(PtnRuntime *runtime, PtnTryFrame *frame) {
    if (runtime->exceptions->try_frame == frame) {
        runtime->exceptions->try_frame = frame->previous;
    }
}

static PTN_UNUSED void ptn_throw_exception(PtnRuntime *runtime, const char *class_name, const char *message) {
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = ptn_exception_new(class_name, message);
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED int ptn_exception_matches(PtnRuntime *runtime, const char *type_name) {
    return runtime->exceptions->active_exception != NULL &&
        ptn_exception_type_matches_name(runtime->exceptions->active_exception->class_name, type_name);
}

static PTN_UNUSED PtnValue ptn_current_exception_value(PtnRuntime *runtime) {
    if (runtime->exceptions->active_exception == NULL) {
        return ptn_null();
    }
    return ptn_exception_borrow(runtime->exceptions->active_exception);
}

static PTN_UNUSED void ptn_clear_exception(PtnRuntime *runtime) {
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = NULL;
}

static PTN_UNUSED void ptn_rethrow_exception(PtnRuntime *runtime) {
    PtnException *exception = runtime->exceptions->active_exception;
    if (exception == NULL) {
        return;
    }
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    fputs("Fatal error: ", stderr);
    fputs(exception->message, stderr);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED PtnValue ptn_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    (void)args;
    (void)line;
    if (receiver.type == PTN_EXCEPTION && ptn_exception_name_equal(name, "getMessage")) {
        if (argc != 0) {
            ptn_throw_exception(
                runtime,
                "ArgumentCountError",
                "Too many arguments to exception method getMessage()"
            );
        }
        return ptn_string(receiver.as.exception->message);
    }
    ptn_throw_exception(runtime, "Error", "Call to undefined method");
    return ptn_null();
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

static PTN_UNUSED int ptn_string_has_embedded_nul(PtnString string) {
    return memchr(string.data, '\0', string.len) != NULL;
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
            if (ptn_string_has_embedded_nul(value.as.string)) {
                return ptn_number_int(0);
            }
            return ptn_string_to_number((const char *)value.as.string.data);
        case PTN_ARRAY:
            return ptn_number_int(value.as.array->len == 0 ? 0 : 1);
        case PTN_EXCEPTION:
            return ptn_number_int(1);
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
        case PTN_EXCEPTION:
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
            return value.as.string.len != 0 &&
                !(value.as.string.len == 1 && value.as.string.data[0] == '0');
        case PTN_ARRAY:
            return value.as.array->len != 0;
        case PTN_EXCEPTION:
            return 1;
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
            if (ptn_string_has_embedded_nul(value.as.string)) {
                return 0;
            }
            return ptn_is_numeric_string((const char *)value.as.string.data, number);
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_ARRAY:
        case PTN_EXCEPTION:
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

static PTN_UNUSED int ptn_compare_string_bytes(
    const unsigned char *left,
    size_t left_len,
    const unsigned char *right,
    size_t right_len
) {
    size_t shared_len = left_len < right_len ? left_len : right_len;
    int compared = shared_len == 0 ? 0 : memcmp(left, right, shared_len);
    if (compared < 0) {
        return PTN_COMPARE_LESS;
    }
    if (compared > 0) {
        return PTN_COMPARE_GREATER;
    }
    if (left_len < right_len) {
        return PTN_COMPARE_LESS;
    }
    if (left_len > right_len) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_value_strings(PtnString left, PtnString right) {
    return ptn_compare_string_bytes(left.data, left.len, right.data, right.len);
}

static PTN_UNUSED void ptn_number_value_to_string(PtnValue value, char *buffer, size_t buffer_len) {
    if (value.type == PTN_INT) {
