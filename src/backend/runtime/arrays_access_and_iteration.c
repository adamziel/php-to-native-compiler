        snprintf(buffer, buffer_len, "%lld", (long long)value.as.integer);
    } else {
        ptn_format_scalar_float(value.as.floating, buffer, buffer_len);
    }
}

static PTN_UNUSED void ptn_number_value_to_runtime_string(
    PtnRuntime *runtime,
    PtnValue value,
    char *buffer,
    size_t buffer_len
) {
    if (value.type == PTN_FLOAT) {
        ptn_format_runtime_scalar_float(runtime, value.as.floating, buffer, buffer_len);
        return;
    }
    ptn_number_value_to_string(value, buffer, buffer_len);
}

static PTN_UNUSED int ptn_compare_number_and_string(
    PtnRuntime *runtime,
    PtnValue number,
    PtnString string,
    int number_is_left
) {
    char number_string[128];
    ptn_number_value_to_runtime_string(runtime, number, number_string, sizeof(number_string));
    size_t number_len = strlen(number_string);
    int compared = ptn_compare_string_bytes(
        (const unsigned char *)number_string,
        number_len,
        string.data,
        string.len
    );
    return number_is_left ? compared : -compared;
}

static PTN_UNUSED int ptn_compare_number_types(PtnValue left, PtnValue right, int *compared) {
    if (left.type == PTN_INT) {
        if (right.type == PTN_INT) {
            *compared = ptn_compare_integers(left.as.integer, right.as.integer);
            return 1;
        }
        if (right.type == PTN_FLOAT) {
            *compared = ptn_compare_numbers((double)left.as.integer, right.as.floating);
            return 1;
        }
        return 0;
    }
    if (left.type == PTN_FLOAT) {
        if (right.type == PTN_INT) {
            *compared = ptn_compare_numbers(left.as.floating, (double)right.as.integer);
            return 1;
        }
        if (right.type == PTN_FLOAT) {
            *compared = ptn_compare_numbers(left.as.floating, right.as.floating);
            return 1;
        }
    }
    return 0;
}

typedef struct {
    int negative;
    const unsigned char *digits;
    size_t len;
} PtnDecimalIntegerString;

static PTN_UNUSED int ptn_parse_decimal_integer_string(
    PtnString string,
    PtnDecimalIntegerString *parsed
) {
    if (string.len == 0) {
        return 0;
    }

    const unsigned char *cursor = string.data;
    const unsigned char *limit = string.data + string.len;
    while (cursor < limit && isspace((unsigned char)*cursor)) {
        cursor++;
    }
    if (cursor >= limit) {
        return 0;
    }

    int negative = 0;
    if (*cursor == '+' || *cursor == '-') {
        negative = *cursor == '-';
        cursor++;
    }

    const unsigned char *digits_start = cursor;
    while (cursor < limit && isdigit((unsigned char)*cursor)) {
        cursor++;
    }
    if (cursor == digits_start) {
        return 0;
    }
    const unsigned char *digits_end = cursor;

    while (cursor < limit && isspace((unsigned char)*cursor)) {
        cursor++;
    }
    if (cursor != limit) {
        return 0;
    }

    const unsigned char *significant = digits_start;
    while (significant < digits_end && *significant == '0') {
        significant++;
    }

    parsed->negative = negative && significant < digits_end;
    parsed->digits = significant;
    parsed->len = (size_t)(digits_end - significant);
    return 1;
}

static PTN_UNUSED int ptn_compare_decimal_integer_strings(
    PtnString left,
    PtnString right,
    int *compared
) {
    PtnDecimalIntegerString left_integer;
    PtnDecimalIntegerString right_integer;
    if (!ptn_parse_decimal_integer_string(left, &left_integer) ||
        !ptn_parse_decimal_integer_string(right, &right_integer)) {
        return 0;
    }

    if (left_integer.len == 0 && right_integer.len == 0) {
        *compared = PTN_COMPARE_EQUAL;
    return 1;
    }
    if (left_integer.negative != right_integer.negative) {
        *compared = left_integer.negative ? PTN_COMPARE_LESS : PTN_COMPARE_GREATER;
        return 1;
    }
    if (left_integer.len != right_integer.len) {
        int ordered = left_integer.len < right_integer.len
            ? PTN_COMPARE_LESS
            : PTN_COMPARE_GREATER;
        *compared = left_integer.negative ? -ordered : ordered;
        return 1;
    }

    int byte_compare = memcmp(left_integer.digits, right_integer.digits, left_integer.len);
    if (byte_compare == 0) {
        *compared = PTN_COMPARE_EQUAL;
        return 1;
    }
    int ordered = byte_compare < 0 ? PTN_COMPARE_LESS : PTN_COMPARE_GREATER;
    *compared = left_integer.negative ? -ordered : ordered;
    return 1;
}

static PTN_UNUSED int ptn_compare_strings_loose(PtnString left, PtnString right) {
    int compared = PTN_COMPARE_EQUAL;
    if (ptn_compare_decimal_integer_strings(left, right, &compared)) {
        return compared;
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (!ptn_string_has_embedded_nul(left) &&
        !ptn_string_has_embedded_nul(right) &&
        ptn_is_numeric_string((const char *)left.data, &left_number) &&
        ptn_is_numeric_string((const char *)right.data, &right_number)) {
        return ptn_compare_numbers(left_number, right_number);
    }
    return ptn_compare_value_strings(left, right);
}

static PTN_UNUSED int ptn_runtime_memory_limit_bytes(PtnRuntime *runtime, size_t *limit_out) {
    if (runtime == NULL || limit_out == NULL) {
        return 0;
    }

    PtnRuntime *root = ptn_runtime_root(runtime);
    const char *text = root != NULL && root->memory_limit != NULL
        ? root->memory_limit
        : runtime->memory_limit;
    if (text == NULL) {
        return 0;
    }

    while (isspace((unsigned char)*text)) {
        text++;
    }
    errno = 0;
    char *end = NULL;
    long long parsed = strtoll(text, &end, 0);
    if (end == text || parsed <= 0 || errno == ERANGE) {
        return 0;
    }
    while (end != NULL && isspace((unsigned char)*end)) {
        end++;
    }

    uint64_t multiplier = 1;
    if (end != NULL && *end != '\0') {
        switch (tolower((unsigned char)*end)) {
            case 'g':
                multiplier = 1024ULL * 1024ULL * 1024ULL;
                break;
            case 'm':
                multiplier = 1024ULL * 1024ULL;
                break;
            case 'k':
                multiplier = 1024ULL;
                break;
            default:
                multiplier = 1;
                break;
        }
    }

    uint64_t magnitude = (uint64_t)parsed;
    if (magnitude > (uint64_t)SIZE_MAX / multiplier) {
        return 0;
    }
    *limit_out = (size_t)(magnitude * multiplier);
    return 1;
}

static PTN_UNUSED void ptn_array_enforce_memory_limit_for_entry_write(
    PtnRuntime *runtime,
    PtnArray *array,
    PtnArrayKey key,
    size_t line
) {
    if (runtime == NULL || array == NULL || array->len < array->capacity) {
        return;
    }
    if (ptn_array_find_key(array, key) < array->len) {
        return;
    }

    size_t new_capacity = array->capacity == 0 ? 8 : array->capacity * 2;
    if (new_capacity < array->capacity || new_capacity > SIZE_MAX / sizeof(PtnArrayEntry)) {
        ptn_emit_memory_allocation_overflow_error(runtime, new_capacity, sizeof(PtnArrayEntry), 0, line);
        return;
    }
    size_t allocation_size = new_capacity * sizeof(PtnArrayEntry);

    size_t limit = 0;
    if (!ptn_runtime_memory_limit_bytes(runtime, &limit) || limit == 0 || allocation_size <= limit) {
        return;
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Allowed memory size of %zu bytes exhausted (tried to allocate %zu bytes)",
        limit,
        allocation_size
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_fatal_error_at(runtime, message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_compare_equal(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line);
static PTN_UNUSED int ptn_compare_identical(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line);
static PTN_UNUSED int ptn_compare_not_identical(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line);
static PTN_UNUSED int ptn_compare_order(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line);
static PTN_UNUSED void ptn_clear_exception(PtnRuntime *runtime);
static PTN_UNUSED void ptn_rethrow_exception(PtnRuntime *runtime);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand(PtnValue value);
static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand_with_runtime(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
);
static PTN_UNUSED int ptn_try_object_to_string_operand(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnStringOperand *out
);
static PTN_UNUSED void ptn_string_operand_free(PtnStringOperand operand);
static PTN_UNUSED PtnNumber ptn_to_number(PtnValue value);
static PTN_UNUSED int ptn_float_to_int_loses_precision(double value);
static PTN_UNUSED int ptn_float_to_int_out_of_range(double value);
static PTN_UNUSED int ptn_runtime_has_active_exception(PtnRuntime *runtime);
static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    double value,
    const char *path,
    size_t line
);
static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    const char *value,
    const char *path,
    size_t line
);
static PTN_UNUSED void ptn_emit_bitwise_float_out_of_range_warning(
    PtnDiagnosticSink *diagnostics,
    double value,
    size_t line
);
static PTN_UNUSED PtnArray *ptn_runtime_array_detach_variable(PtnRuntime *runtime, const char *name);
static PTN_UNUSED PtnArray *ptn_value_replace_with_empty_array(PtnValue *value);
static PTN_UNUSED int ptn_arrayaccess_can_dispatch(
    PtnRuntime *runtime,
    PtnValue container,
    const char *method_name
);
static PTN_UNUSED PtnValue ptn_arrayaccess_read(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
);

static PTN_UNUSED PtnArrayEntry *ptn_array_entry_for_key(PtnArray *array, PtnArrayKey key) {
    size_t index = ptn_array_find_key(array, key);
    return index < array->len ? &array->entries[index] : NULL;
}

static PTN_UNUSED int ptn_internal_class_name_is_caching_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_recursive_caching_iterator(const char *class_name);
static PTN_UNUSED PtnValue ptn_caching_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_recursive_caching_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED int ptn_internal_class_name_is_recursive_regex_iterator(const char *class_name);
static PTN_UNUSED PtnValue ptn_recursive_regex_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED int ptn_internal_class_name_is_spl_file_object(const char *class_name);
static PtnValue ptn_spl_file_object_new_for_class(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_spl_file_object_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED int ptn_internal_class_name_is_intl_date_pattern_generator(const char *class_name);
static PTN_UNUSED PtnValue ptn_intl_date_pattern_generator_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_message_formatter_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_intl_number_formatter_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);

static PTN_UNUSED const char *ptn_offset_container_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return "null";
        case PTN_BOOL:
            return value.as.boolean ? "true" : "false";
        case PTN_INT:
            return "int";
        case PTN_FLOAT:
            return "float";
        case PTN_STRING:
            return "string";
        case PTN_RESOURCE:
            return "resource";
        case PTN_ARRAY:
            return "array";
        case PTN_OBJECT:
        case PTN_CLOSURE:
            return "object";
        case PTN_EXCEPTION:
            return "object";
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED int ptn_value_is_plain_object_for_array_offset(PtnRuntime *runtime, PtnValue value) {
    value = ptn_value_deref(value);
    return (value.type == PTN_OBJECT || value.type == PTN_CLOSURE || value.type == PTN_EXCEPTION) &&
        !ptn_arrayaccess_can_dispatch(runtime, value, "offsetGet") &&
        !ptn_arrayaccess_can_dispatch(runtime, value, "offsetSet") &&
        !ptn_arrayaccess_can_dispatch(runtime, value, "offsetExists") &&
        !ptn_arrayaccess_can_dispatch(runtime, value, "offsetUnset");
}

static PTN_UNUSED void ptn_throw_cannot_use_object_as_array(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    const char *class_name = "stdClass";
    if (value.type == PTN_OBJECT && value.as.object != NULL && value.as.object->class_name != NULL) {
        class_name = value.as.object->class_name;
    } else if (value.type == PTN_CLOSURE) {
        class_name = "Closure";
    } else if (value.type == PTN_EXCEPTION && value.as.exception != NULL && value.as.exception->class_name != NULL) {
        class_name = value.as.exception->class_name;
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot use object of type %s as array",
        class_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED const char *ptn_offset_key_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return "null";
        case PTN_BOOL:
            return "bool";
        case PTN_INT:
            return "int";
        case PTN_FLOAT:
            return "float";
        case PTN_STRING:
            return "string";
        case PTN_RESOURCE:
            return "resource";
        case PTN_ARRAY:
            return "array";
        case PTN_OBJECT:
            return value.as.object->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED int ptn_array_offset_key_is_invalid(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_ARRAY ||
        value.type == PTN_OBJECT ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION;
}

static PTN_UNUSED void ptn_throw_array_offset_key_type_error(
    PtnRuntime *runtime,
    PtnValue key_value,
    const char *format,
    size_t line
) {
    const char *type_name = ptn_offset_key_type_name(key_value);
    char message[256];
    int written = snprintf(message, sizeof(message), format, type_name);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_array_offset_key_from_value(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int quiet,
    PtnArrayKey *key_out
) {
    key_value = ptn_value_deref(key_value);
    switch (key_value.type) {
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION: {
            if (!quiet) {
                const char *type_name = ptn_offset_key_type_name(key_value);
                char message[256];
                int written = snprintf(
                    message,
                    sizeof(message),
                    "Cannot access offset of type %s on array",
                    type_name
                );
                if (written < 0 || (size_t)written >= sizeof(message)) {
                    ptn_abort_out_of_memory();
                }
                ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
            }
            return 0;
        }
        case PTN_REFERENCE:
            return ptn_array_offset_key_from_value(runtime, key_value, line, quiet, key_out);
        default:
            *key_out = ptn_array_key_from_value(key_value);
            return 1;
    }
}

static PTN_UNUSED void ptn_emit_array_runtime_diagnostic_at_path(
    PtnRuntime *runtime,
    const char *kind,
    const char *message,
    const char *path,
    size_t line
) {
    ptn_diagnostic_printf(
        runtime == NULL ? NULL : &runtime->diagnostics,
        "\n%s: %s in %s on line %zu\n",
        kind,
        message,
        path,
        line
    );
}

static PTN_UNUSED const char *ptn_array_runtime_diagnostic_path(PtnRuntime *runtime) {
    if (runtime != NULL && runtime->source_path != NULL) {
        return runtime->source_path;
    }
    return "ptn";
}

static PTN_UNUSED void ptn_emit_array_runtime_diagnostic(
    PtnRuntime *runtime,
    const char *kind,
    const char *message,
    size_t line
) {
    ptn_emit_array_runtime_diagnostic_at_path(
        runtime,
        kind,
        message,
        ptn_array_runtime_diagnostic_path(runtime),
        line
    );
}

static PTN_UNUSED void ptn_emit_array_runtime_warning(PtnRuntime *runtime, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    runtime->diagnostics.emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(
        &runtime->diagnostics,
        PTN_E_WARNING,
        message,
        runtime->source_path,
        line
    )) {
        return;
    }
    ptn_emit_array_runtime_diagnostic(runtime, "Warning", message, line);
}

static PTN_UNUSED void ptn_emit_resource_offset_warning(PtnRuntime *runtime, PtnResource *resource, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "Resource ID#%lld used as offset, casting to integer (%lld)",
        (long long)resource->id,
        (long long)resource->id
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    if (ptn_diagnostics_try_error_handler(
        &runtime->diagnostics,
        PTN_E_WARNING,
        message,
        runtime->source_path,
        line
    )) {
        return;
    }
    ptn_emit_array_runtime_diagnostic_at_path(
        runtime,
        "Warning",
        message,
        runtime->source_path == NULL ? "ptn" : runtime->source_path,
        line
    );
    runtime->diagnostics.emitted_warning = 1;
}

static PTN_UNUSED int ptn_class_name_is_stdclass(const char *class_name) {
    const char *stdclass = "stdClass";
    while (*class_name != '\0' && *stdclass != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*stdclass)) {
            return 0;
        }
        class_name++;
        stdclass++;
    }
    return *class_name == '\0' && *stdclass == '\0';
}

static PTN_UNUSED int ptn_class_name_is_datetime(const char *class_name) {
    const char *datetime = "DateTime";
    while (*class_name != '\0' && *datetime != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*datetime)) {
            return 0;
        }
        class_name++;
        datetime++;
    }
    return *class_name == '\0' && *datetime == '\0';
}

static PTN_UNUSED int ptn_class_name_is_datetime_immutable(const char *class_name) {
    const char *datetime = "DateTimeImmutable";
    while (*class_name != '\0' && *datetime != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*datetime)) {
            return 0;
        }
        class_name++;
        datetime++;
    }
    return *class_name == '\0' && *datetime == '\0';
}

static PTN_UNUSED int ptn_class_name_is_datetime_zone(const char *class_name) {
    const char *timezone = "DateTimeZone";
    while (*class_name != '\0' && *timezone != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*timezone)) {
            return 0;
        }
        class_name++;
        timezone++;
    }
    return *class_name == '\0' && *timezone == '\0';
}

static PTN_UNUSED int ptn_class_name_is_date_interval(const char *class_name) {
    const char *date_interval = "DateInterval";
    while (*class_name != '\0' && *date_interval != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*date_interval)) {
            return 0;
        }
        class_name++;
        date_interval++;
    }
    return *class_name == '\0' && *date_interval == '\0';
}

static PTN_UNUSED int ptn_class_name_is_generator(const char *class_name) {
    const char *generator = "Generator";
    while (*class_name != '\0' && *generator != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*generator)) {
            return 0;
        }
        class_name++;
        generator++;
    }
    return *class_name == '\0' && *generator == '\0';
}

static PTN_UNUSED char *ptn_dynamic_new_class_name_from_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_OBJECT:
            return ptn_duplicate_string(value.as.object->class_name);
        case PTN_EXCEPTION:
            return ptn_duplicate_string(value.as.exception->class_name);
        case PTN_CLOSURE:
            return ptn_duplicate_string("Closure");
        case PTN_STRING:
            return ptn_value_to_string(value);
        default:
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Class name must be a valid object or a string",
                runtime != NULL ? runtime->source_path : NULL,
                line
            );
            return NULL;
    }
}

static int ptn_invalid_url_exception_errors_arg_valid(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type != PTN_ARRAY || value.as.array == NULL) {
        return 0;
    }
    for (size_t i = 0; i < value.as.array->len; i++) {
        PtnArrayEntry *entry = &value.as.array->entries[i];
        PtnValue item = ptn_value_deref(entry->value);
        if (entry->key.type != PTN_ARRAY_KEY_INT ||
            entry->key.as.integer != (int64_t)i ||
            item.type != PTN_OBJECT ||
            item.as.object == NULL ||
            !ptn_ascii_case_equal(item.as.object->class_name, "Uri\\WhatWg\\UrlValidationError")) {
            return 0;
        }
    }
    return 1;
}

static void ptn_invalid_url_exception_throw_errors_arg_value_error(PtnRuntime *runtime) {
    ptn_throw_exception(
        runtime,
        "ValueError",
        "Uri\\WhatWg\\InvalidUrlException::__construct(): Argument #2 ($errors) must be a list of Uri\\WhatWg\\UrlValidationError"
    );
}

static PTN_UNUSED PtnValue ptn_new_exception_object(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    const char *declaring_class = ptn_exception_constructor_declaring_class(runtime, class_name);
    int is_error_exception = ptn_exception_name_equal(declaring_class, "ErrorException");
    int is_soap_fault = ptn_exception_name_equal(declaring_class, "SoapFault");
    int is_invalid_url_exception = ptn_exception_name_equal(declaring_class, "Uri\\WhatWg\\InvalidUrlException");
    size_t max_args = ptn_exception_constructor_max_args(declaring_class);
    if (argc > max_args) {
        char message[128];
        int written = snprintf(
            message,
            sizeof(message),
            "%s constructor expects at most %zu arguments",
            declaring_class,
            max_args
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(
            runtime,
            "ArgumentCountError",
            message
        );
        return ptn_null();
    }
    if (!ptn_exception_validate_soap_fault_code(runtime, declaring_class, argc, args, line)) {
        return ptn_null();
    }
    PtnStringOperand message = ptn_exception_constructor_message(
        runtime,
        declaring_class,
        argc,
        args,
        line
    );
    if (is_invalid_url_exception &&
        argc >= 2 &&
        !ptn_invalid_url_exception_errors_arg_valid(args[1])) {
        ptn_string_operand_free(message);
        ptn_invalid_url_exception_throw_errors_arg_value_error(runtime);
        return ptn_null();
    }
    int64_t code = 0;
    size_t code_index = is_invalid_url_exception ? 2 : 1;
    if (!is_soap_fault && argc > code_index) {
        PtnValue code_value = ptn_value_deref(args[code_index]);
        if (code_value.type == PTN_INT) {
            code = code_value.as.integer;
        } else if (code_value.type == PTN_BOOL) {
            code = code_value.as.boolean ? 1 : 0;
        } else if (code_value.type == PTN_FLOAT) {
            code = (int64_t)code_value.as.floating;
        }
    }
    int64_t severity = PTN_E_ERROR;
    if (is_error_exception && argc >= 3) {
        PtnValue severity_value = ptn_value_deref(args[2]);
        if (severity_value.type == PTN_INT) {
            severity = severity_value.as.integer;
        } else if (severity_value.type == PTN_BOOL) {
            severity = severity_value.as.boolean ? 1 : 0;
        } else if (severity_value.type == PTN_FLOAT) {
            severity = (int64_t)severity_value.as.floating;
        }
    }
    const char *exception_path = runtime->source_path;
    int has_error_exception_path_argument = 0;
    if (is_error_exception && argc >= 4 && ptn_value_deref(args[3]).type != PTN_NULL) {
        PtnStringOperand path_string = ptn_value_to_string_operand_with_runtime(runtime, args[3], line);
        if (runtime->exceptions->active_exception != NULL) {
            ptn_string_operand_free(message);
            ptn_string_operand_free(path_string);
            return ptn_null();
        }
        exception_path = ptn_duplicate_string_len(path_string.data, path_string.len);
        has_error_exception_path_argument = 1;
        ptn_string_operand_free(path_string);
    }
    size_t exception_line = has_error_exception_path_argument ? 0 : line;
    if (is_error_exception && argc >= 5 && ptn_value_deref(args[4]).type != PTN_NULL) {
        PtnValue line_value = ptn_value_deref(args[4]);
        if (line_value.type == PTN_INT && line_value.as.integer >= 0) {
            exception_line = (size_t)line_value.as.integer;
        } else if (line_value.type == PTN_FLOAT && line_value.as.floating >= 0.0) {
            exception_line = (size_t)line_value.as.floating;
        }
    }
    PtnValue previous = ptn_null();
    size_t previous_index = is_invalid_url_exception ? 3 : (is_error_exception ? 5 : 2);
    if (!is_soap_fault && argc > previous_index) {
        PtnValue previous_value = ptn_value_deref(args[previous_index]);
        if (previous_value.type == PTN_EXCEPTION ||
            (previous_value.type == PTN_OBJECT && ptn_object_is_declared_throwable(runtime, previous_value.as.object))) {
            previous = previous_value;
        }
    }
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        class_name,
        message.owned,
        message.len,
        code,
        previous,
        severity,
        exception_path,
        exception_line
    );
    if (is_invalid_url_exception) {
        ptn_value_destroy(&exception->errors);
        exception->errors = argc >= 2
            ? ptn_value_clone_deref(args[1])
            : ptn_array_from_literal_entries(0, NULL);
    }
    ptn_exception_set_soap_fault_headerfault(exception, argc, args);
    ptn_exception_set_soap_fault_properties(exception, argc, args);
    return ptn_exception_value(exception);
}

static PTN_UNUSED int ptn_runtime_autoloading_class(PtnRuntime *runtime, const char *class_name) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || class_name == NULL) {
        return 0;
    }
    for (size_t i = 0; i < root->autoloading_class_names_len; i++) {
        if (ptn_ascii_case_equal(root->autoloading_class_names[i], class_name)) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_runtime_push_autoloading_class(PtnRuntime *runtime, const char *class_name) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || class_name == NULL) {
        return;
    }
    if (root->autoloading_class_names_len == root->autoloading_class_names_capacity) {
        size_t new_capacity = root->autoloading_class_names_capacity == 0
            ? 4
            : root->autoloading_class_names_capacity * 2;
        if (new_capacity < root->autoloading_class_names_capacity ||
            new_capacity > SIZE_MAX / sizeof(char *)) {
            ptn_abort_out_of_memory();
        }
        char **new_names = realloc(root->autoloading_class_names, new_capacity * sizeof(char *));
        if (new_names == NULL) {
            ptn_abort_out_of_memory();
        }
        root->autoloading_class_names = new_names;
        root->autoloading_class_names_capacity = new_capacity;
    }
    root->autoloading_class_names[root->autoloading_class_names_len++] =
        ptn_duplicate_string(class_name);
}

static PTN_UNUSED void ptn_runtime_pop_autoloading_class(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || root->autoloading_class_names_len == 0) {
        return;
    }
    root->autoloading_class_names_len--;
    free(root->autoloading_class_names[root->autoloading_class_names_len]);
    root->autoloading_class_names[root->autoloading_class_names_len] = NULL;
}

static PTN_UNUSED void ptn_runtime_autoload_class_with_call_frame(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line,
    int suppress_user_call_frame_location
) {
#ifndef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    (void)runtime;
    (void)class_name;
    (void)line;
    (void)suppress_user_call_frame_location;
#else
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL ||
        class_name == NULL ||
        root->autoload_callbacks_len == 0 ||
        ptn_runtime_autoloading_class(root, class_name)) {
        return;
    }

    PtnValue active_callback = ptn_null();
    PtnTryFrame autoload_frame;
    int saved_suppress_user_call_frame_location =
        runtime->suppress_user_call_frame_location;
    const char *saved_current_class_name = runtime->current_class_name;
    const char *saved_current_called_class_name = runtime->current_called_class_name;
    ptn_runtime_push_autoloading_class(root, class_name);
    ptn_try_frame_push(runtime, &autoload_frame);
    if (setjmp(autoload_frame.jump) == 0) {
        for (size_t i = 0; i < root->autoload_callbacks_len; i++) {
            active_callback = ptn_value_clone(root->autoload_callbacks[i]);
            PtnValue callback_args[1] = { ptn_string(class_name) };
            runtime->current_class_name =
                root->autoload_callback_scope_class_names != NULL
                    ? root->autoload_callback_scope_class_names[i]
                    : NULL;
            runtime->current_called_class_name =
                root->autoload_callback_called_class_names != NULL
                    ? root->autoload_callback_called_class_names[i]
                    : runtime->current_class_name;
            runtime->suppress_user_call_frame_location =
                suppress_user_call_frame_location ? 1 : 0;
            PtnValue result =
                ptn_call_callable(runtime, active_callback, 1, callback_args, line, 0);
            runtime->suppress_user_call_frame_location =
                saved_suppress_user_call_frame_location;
            runtime->current_class_name = saved_current_class_name;
            runtime->current_called_class_name = saved_current_called_class_name;
            ptn_value_destroy(&result);
            ptn_value_destroy(&active_callback);
            active_callback = ptn_null();
            if (runtime->exceptions->active_exception != NULL) {
                break;
            }
        }
        ptn_try_frame_pop(runtime, &autoload_frame);
        ptn_runtime_pop_autoloading_class(root);
    } else {
        ptn_try_frame_pop(runtime, &autoload_frame);
        runtime->suppress_user_call_frame_location =
            saved_suppress_user_call_frame_location;
        runtime->current_class_name = saved_current_class_name;
        runtime->current_called_class_name = saved_current_called_class_name;
        ptn_value_destroy(&active_callback);
        ptn_runtime_pop_autoloading_class(root);
        ptn_rethrow_exception(runtime);
    }
#endif
}

static PTN_UNUSED void ptn_runtime_autoload_class(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
) {
    ptn_runtime_autoload_class_with_call_frame(runtime, class_name, line, 1);
}

static int ptn_class_name_autoload_char_is_valid(unsigned char ch) {
    return ch == '_' ||
        ch == '\\' ||
        (ch >= 'a' && ch <= 'z') ||
        (ch >= 'A' && ch <= 'Z') ||
        (ch >= '0' && ch <= '9') ||
        ch >= 0x80;
}

static int ptn_class_name_should_autoload(const char *name) {
    if (name == NULL || *name == '\0') {
        return 0;
    }
    for (const unsigned char *cursor = (const unsigned char *)name; *cursor != '\0'; cursor++) {
        if (!ptn_class_name_autoload_char_is_valid(*cursor)) {
            return 0;
        }
    }
    return 1;
}

static PtnValue ptn_declared_class_new_instance(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);

#if defined(PTN_HAS_INTERNAL_FUNCTION_DISPATCH) || defined(PTN_HAS_URI_INTERNAL_HELPERS)
static PTN_UNUSED int ptn_internal_class_name_is_uri_whatwg_url(const char *class_name);
static PTN_UNUSED PtnValue ptn_uri_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
#endif

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED PtnValue ptn_date_period_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_php_token_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_random_engine_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_random_engine_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED int ptn_internal_class_name_is_closure(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_curl_file(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_directory(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_phar(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_phar_data(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_php_token(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_random_engine(const char *class_name);
static PTN_UNUSED PtnValue ptn_phar_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_curl_file_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static int ptn_date_value_is_uninitialized_descendant(PtnValue value, const char *ancestor);
static void ptn_date_throw_uninitialized_named_object_error(PtnRuntime *runtime, const char *class_name);
#endif

static PTN_UNUSED PtnValue ptn_new_object(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    const char *lookup_class_name = ptn_runtime_resolve_class_alias(
        runtime,
        ptn_symbol_name_without_leading_slash(class_name)
    );
    if (ptn_class_name_is_stdclass(lookup_class_name)) {
        (void)argc;
        (void)args;
        return ptn_object_new_shell_at(runtime, "stdClass", line);
    }
    if (ptn_ascii_case_equal(lookup_class_name, "__PHP_Incomplete_Class")) {
        (void)argc;
        (void)args;
        return ptn_object_new_shell_at(runtime, "__PHP_Incomplete_Class", line);
    }
    if (!ptn_declared_runtime_class_exists(runtime, lookup_class_name)) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        if (!ptn_internal_class_exists_name(lookup_class_name) &&
            ptn_class_name_should_autoload(lookup_class_name)) {
            ptn_runtime_autoload_class(runtime, lookup_class_name, line);
            lookup_class_name = ptn_runtime_resolve_class_alias(
                runtime,
                ptn_symbol_name_without_leading_slash(class_name)
            );
        }
#else
        if (ptn_class_name_should_autoload(lookup_class_name)) {
            ptn_runtime_autoload_class(runtime, lookup_class_name, line);
            lookup_class_name = ptn_runtime_resolve_class_alias(
                runtime,
                ptn_symbol_name_without_leading_slash(class_name)
            );
        }
#endif
        if (runtime->exceptions->active_exception != NULL) {
            return ptn_null();
        }
    }
    if (ptn_runtime_dynamic_class_exists(runtime, lookup_class_name)) {
        (void)argc;
        (void)args;
        return ptn_object_new_shell_at(runtime, lookup_class_name, line);
    }
    if (ptn_declared_runtime_user_class_exists(runtime, lookup_class_name)) {
        return ptn_declared_class_new_instance(runtime, lookup_class_name, argc, args, line);
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_internal_class_name_is_reflection_class(lookup_class_name)) {
        return ptn_reflection_class_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_object(lookup_class_name)) {
        return ptn_reflection_object_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_enum(lookup_class_name)) {
        return ptn_reflection_enum_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_enum_unit_case(lookup_class_name)) {
        return ptn_reflection_enum_case_new(
            runtime,
            "ReflectionEnumUnitCase",
            0,
            argc,
            args,
            line
        );
    }
    if (ptn_internal_class_name_is_reflection_enum_backed_case(lookup_class_name)) {
        return ptn_reflection_enum_case_new(
            runtime,
            "ReflectionEnumBackedCase",
            1,
            argc,
            args,
            line
        );
    }
    if (ptn_internal_class_name_is_reflection_extension(lookup_class_name)) {
        return ptn_reflection_extension_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_zend_extension(lookup_class_name)) {
        return ptn_reflection_zend_extension_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_function(lookup_class_name)) {
        return ptn_reflection_function_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_generator(lookup_class_name)) {
        return ptn_reflection_generator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_fiber(lookup_class_name)) {
        return ptn_reflection_fiber_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_method(lookup_class_name)) {
        return ptn_reflection_method_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_parameter(lookup_class_name)) {
        return ptn_reflection_parameter_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_class_constant(lookup_class_name)) {
        return ptn_reflection_class_constant_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_constant(lookup_class_name)) {
        return ptn_reflection_constant_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_attribute(lookup_class_name)) {
        ptn_throw_exception(runtime, "Error", "Cannot directly instantiate ReflectionAttribute");
        return ptn_null();
    }
    if (ptn_internal_class_name_is_closure(lookup_class_name)) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Instantiation of class Closure is not allowed",
            runtime->source_path,
            line
        );
        return ptn_null();
    }
    if (ptn_internal_class_name_is_sensitive_parameter(lookup_class_name)) {
        return ptn_sensitive_parameter_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_sensitive_parameter_value(lookup_class_name)) {
        return ptn_sensitive_parameter_value_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_fiber(lookup_class_name)) {
        return ptn_fiber_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_weak_reference(lookup_class_name)) {
        return ptn_weak_reference_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_weak_map(lookup_class_name)) {
        return ptn_weak_map_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_attribute(lookup_class_name)) {
        return ptn_attribute_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_allow_dynamic_properties(lookup_class_name)) {
        return ptn_allow_dynamic_properties_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_delayed_target_validation(lookup_class_name)) {
        return ptn_delayed_target_validation_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_deprecated(lookup_class_name)) {
        return ptn_deprecated_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_no_discard(lookup_class_name)) {
        return ptn_no_discard_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_return_type_will_change(lookup_class_name)) {
        return ptn_return_type_will_change_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_pdo(lookup_class_name)) {
        return ptn_pdo_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_pdo_statement(lookup_class_name)) {
        return ptn_pdo_statement_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_pdo_exception(lookup_class_name)) {
        return ptn_new_exception_object(runtime, "PDOException", argc, args, line);
    }
    if (ptn_internal_class_name_is_pdo_row(lookup_class_name)) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "You may not create a PDORow object manually",
            runtime->source_path,
            line
        );
        return ptn_null();
    }
    if (ptn_internal_class_name_is_sqlite3(lookup_class_name)) {
        return ptn_sqlite3_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_sqlite3_stmt(lookup_class_name) ||
        ptn_internal_class_name_is_sqlite3_result(lookup_class_name)) {
        ptn_throw_exception(runtime, "Error", "Cannot directly instantiate internal class");
        return ptn_null();
    }
    if (ptn_internal_class_name_is_curl_file(lookup_class_name)) {
        return ptn_curl_file_new(runtime, argc, args, line);
    }
    if (ptn_ascii_case_equal(lookup_class_name, "IntlBreakIterator") ||
        ptn_ascii_case_equal(lookup_class_name, "IntlRuleBasedBreakIterator") ||
        ptn_ascii_case_equal(lookup_class_name, "IntlCodePointBreakIterator") ||
        ptn_ascii_case_equal(lookup_class_name, "IntlPartsIterator")) {
        return ptn_intl_break_iterator_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_intl_calendar(lookup_class_name) ||
        ptn_internal_class_name_is_intl_date_formatter(lookup_class_name) ||
        ptn_internal_class_name_is_intl_timezone(lookup_class_name) ||
        ptn_internal_class_name_is_intl_iterator(lookup_class_name) ||
        ptn_internal_class_name_is_message_formatter(lookup_class_name) ||
        ptn_internal_class_name_is_intl_list_formatter(lookup_class_name) ||
        ptn_internal_class_name_is_intl_date_pattern_generator(lookup_class_name) ||
        ptn_internal_class_name_is_locale(lookup_class_name) ||
        ptn_internal_class_name_is_number_formatter(lookup_class_name) ||
        ptn_internal_class_name_is_intl_number_range_formatter(lookup_class_name) ||
        ptn_internal_class_name_is_collator(lookup_class_name) ||
        ptn_internal_class_name_is_resource_bundle(lookup_class_name) ||
        ptn_internal_class_name_is_spoofchecker(lookup_class_name) ||
        ptn_internal_class_name_is_uconverter(lookup_class_name)) {
        return ptn_intl_plain_object_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_property(lookup_class_name)) {
        return ptn_reflection_property_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_reference(lookup_class_name)) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Call to private ReflectionReference::__construct() from global scope",
            runtime->source_path,
            line
        );
        return ptn_null();
    }
    if (ptn_internal_class_name_is_array_iterator(lookup_class_name)) {
        return ptn_array_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_empty_iterator(lookup_class_name)) {
        return ptn_empty_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_array_object(lookup_class_name)) {
        return ptn_array_object_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_spl_fixed_array(lookup_class_name)) {
        return ptn_spl_fixed_array_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_spl_object_storage(lookup_class_name)) {
        return ptn_spl_object_storage_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_spl_heap(lookup_class_name) ||
        ptn_internal_class_name_is_spl_max_heap(lookup_class_name) ||
        ptn_internal_class_name_is_spl_min_heap(lookup_class_name) ||
        ptn_internal_class_name_is_spl_priority_queue(lookup_class_name)) {
        return ptn_spl_heap_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_append_iterator(lookup_class_name)) {
        return ptn_append_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_recursive_caching_iterator(lookup_class_name)) {
        return ptn_recursive_caching_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_caching_iterator(lookup_class_name)) {
        return ptn_caching_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_callback_filter_iterator(lookup_class_name)) {
        return ptn_callback_filter_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_recursive_callback_filter_iterator(lookup_class_name)) {
        return ptn_recursive_callback_filter_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_filter_iterator(lookup_class_name)) {
        return ptn_filter_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_recursive_regex_iterator(lookup_class_name)) {
        return ptn_recursive_regex_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_regex_iterator(lookup_class_name)) {
        return ptn_regex_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_infinite_iterator(lookup_class_name)) {
        return ptn_infinite_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_iterator_iterator(lookup_class_name)) {
        return ptn_iterator_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_no_rewind_iterator(lookup_class_name)) {
        return ptn_no_rewind_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_multiple_iterator(lookup_class_name)) {
        return ptn_multiple_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_recursive_iterator_iterator(lookup_class_name)) {
        return ptn_recursive_iterator_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_limit_iterator(lookup_class_name)) {
        return ptn_limit_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_recursive_array_iterator(lookup_class_name)) {
        return ptn_recursive_array_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_spl_doubly_linked_list(lookup_class_name) ||
        ptn_internal_class_name_is_spl_queue(lookup_class_name) ||
        ptn_internal_class_name_is_spl_stack(lookup_class_name)) {
        return ptn_spl_doubly_linked_list_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_spl_file_object(lookup_class_name)) {
        return ptn_spl_file_object_new_for_class(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_directory_iterator(lookup_class_name)) {
        return ptn_directory_iterator_new_for_class(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_directory(lookup_class_name)) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Cannot directly construct Directory, use dir() instead",
            runtime->source_path,
            line
        );
        return ptn_null();
    }
    if (ptn_internal_class_name_is_spl_file_info(lookup_class_name)) {
        return ptn_spl_file_info_new(runtime, "SplFileInfo", argc, args, line);
    }
    if (ptn_internal_class_name_is_session_handler(lookup_class_name)) {
        return ptn_session_handler_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_php_token(lookup_class_name)) {
        return ptn_php_token_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_dom(lookup_class_name)) {
        return ptn_dom_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_simplexml(lookup_class_name)) {
        return ptn_simplexml_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_xml_reader(lookup_class_name)) {
        return ptn_xml_reader_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_xml_writer(lookup_class_name)) {
        return ptn_xmlwriter_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_xml_parser(lookup_class_name)) {
        return ptn_xml_parser_new(runtime, argc, args, line);
    }
#endif
    const char *exception_class_name = ptn_builtin_exception_class_name(lookup_class_name);
    if (exception_class_name != NULL) {
        return ptn_new_exception_object(runtime, exception_class_name, argc, args, line);
    }
    if (ptn_class_name_is_datetime(lookup_class_name)) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        return ptn_datetime_new(runtime, lookup_class_name, argc, args, line, 0);
#else
        if (argc > 1) {
            ptn_throw_exception(runtime, "ArgumentCountError", "DateTime constructor expects at most 1 argument");
            return ptn_null();
        }
        return ptn_object_new_shell_at(runtime, "DateTime", line);
#endif
    }
    if (ptn_class_name_is_datetime_immutable(lookup_class_name)) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        return ptn_datetime_new(runtime, lookup_class_name, argc, args, line, 0);
#else
        if (argc > 1) {
            ptn_throw_exception(runtime, "ArgumentCountError", "DateTimeImmutable constructor expects at most 1 argument");
            return ptn_null();
        }
        return ptn_object_new_shell_at(runtime, "DateTimeImmutable", line);
#endif
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_class_name_is_datetime_zone(lookup_class_name)) {
        return ptn_datetime_zone_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_class_name_is_date_interval(lookup_class_name)) {
        return ptn_date_interval_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_date_period(lookup_class_name)) {
        return ptn_date_period_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_bcmath_number(lookup_class_name)) {
        return ptn_bcmath_number_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_random_randomizer(lookup_class_name)) {
        return ptn_random_randomizer_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_random_engine(lookup_class_name)) {
        return ptn_random_engine_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_phar(lookup_class_name) ||
        ptn_internal_class_name_is_phar_data(lookup_class_name)) {
        return ptn_phar_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_phar_file_info(lookup_class_name)) {
        ptn_throw_exception(runtime, "Error", "Cannot directly instantiate internal class");
        return ptn_null();
    }
    if (ptn_internal_class_name_is_zip_archive(lookup_class_name)) {
        return ptn_zip_archive_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_soap_client(lookup_class_name)) {
        return ptn_soap_client_new(runtime, "SoapClient", argc, args, line);
    }
    if (ptn_internal_class_name_is_soap_server(lookup_class_name)) {
        return ptn_soap_client_new(runtime, "SoapServer", argc, args, line);
    }
    if (ptn_internal_class_name_is_soap_header(lookup_class_name)) {
        return ptn_soap_header_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_soap_var(lookup_class_name)) {
        return ptn_soap_var_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_soap_param(lookup_class_name)) {
        return ptn_soap_param_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_xml_writer(lookup_class_name)) {
        return ptn_xmlwriter_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_uri_rfc3986_uri(lookup_class_name) ||
        ptn_internal_class_name_is_uri_whatwg_url(lookup_class_name)) {
        return ptn_uri_new(runtime, lookup_class_name, argc, args, line);
    }
    if (ptn_internal_class_name_is_uri_whatwg_url(lookup_class_name)) {
        return ptn_uri_whatwg_url_new(runtime, argc, args, line);
    }
#endif
#ifdef PTN_HAS_URI_INTERNAL_HELPERS
    if (ptn_internal_class_name_is_uri_whatwg_url(lookup_class_name)) {
        return ptn_uri_new(runtime, lookup_class_name, argc, args, line);
    }
#endif
    if (ptn_class_name_is_generator(lookup_class_name)) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "The \"Generator\" class is reserved for internal use and cannot be manually instantiated",
            runtime->source_path,
            line
        );
        return ptn_null();
    }
    char message[192];
    int written = snprintf(message, sizeof(message), "Class \"%s\" not found", lookup_class_name);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_object_clone_property_value(PtnValue value) {
    if (value.type == PTN_REFERENCE &&
        value.as.reference != NULL &&
        value.as.reference->refcount > 1) {
        return ptn_value_clone(value);
    }
    return ptn_value_clone_deref(value);
}

static PTN_UNUSED void ptn_reference_adopt_property_type_clone_source(
    PtnReference *reference,
    const PtnObjectPropertyMetadata *metadata
);

static PTN_UNUSED void ptn_object_copy_storage_for_clone(PtnObject *cloned, PtnObject *source) {
    ptn_array_free(cloned->properties);
    PtnValue cloned_properties = ptn_array_from_literal_entries(0, NULL);
    cloned->properties = cloned_properties.as.array;
    for (size_t i = 0; i < source->properties->len; i++) {
        PtnArrayEntry *entry = &source->properties->entries[i];
        ptn_array_set_entry(
            cloned->properties,
            ptn_array_key_clone(entry->key),
            ptn_object_clone_property_value(entry->value)
        );
    }
    cloned->properties->current_index =
        source->properties->current_index <= source->properties->len
            ? source->properties->current_index
            : source->properties->len;
    cloned->properties->next_auto_key = source->properties->next_auto_key;
    for (size_t i = 0; i < source->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &source->property_metadata[i];
        ptn_object_register_property_metadata(
            cloned,
            metadata->display_name,
            metadata->declaring_class,
            metadata->read_visibility,
            metadata->set_visibility,
            metadata->is_readonly,
            metadata->has_hooks,
            metadata->is_virtual,
            metadata->hook_has_get,
            metadata->hook_get_returns_by_ref,
            metadata->hook_has_set,
            metadata->hook_get_declaring_class,
            metadata->hook_set_declaring_class,
            metadata->type_kind,
            metadata->type_class_name,
            metadata->type_text,
            metadata->type_allows_null
        );
        PtnObjectPropertyMetadata *cloned_metadata = NULL;
        for (size_t metadata_index = 0; metadata_index < cloned->property_metadata_len; metadata_index++) {
            if (strcmp(cloned->property_metadata[metadata_index].storage_name, metadata->storage_name) == 0) {
                cloned_metadata = &cloned->property_metadata[metadata_index];
                break;
            }
        }
        if (cloned_metadata != NULL) {
            cloned_metadata->is_unset = metadata->is_unset;
            cloned_metadata->lazy_skip = metadata->lazy_skip;
            if (metadata->last_type_name != NULL) {
                cloned_metadata->last_type_name = ptn_duplicate_string(metadata->last_type_name);
            }
            PtnArrayKey cloned_key = ptn_array_string_key(cloned_metadata->storage_name);
            PtnArrayEntry *cloned_entry = ptn_array_entry_for_key(cloned->properties, cloned_key);
            if (cloned_entry != NULL && cloned_entry->value.type == PTN_REFERENCE) {
                ptn_reference_adopt_property_type_clone_source(
                    cloned_entry->value.as.reference,
                    cloned_metadata
                );
            }
            ptn_array_key_free(cloned_key);
        }
    }
}

static PTN_UNUSED PtnValue ptn_object_clone_storage_without_magic(
    PtnRuntime *runtime,
    PtnObject *source
) {
    PtnValue clone = ptn_object_new_shell(runtime, source->class_name);
    ptn_object_copy_storage_for_clone(clone.as.object, source);
    return clone;
}

static PTN_UNUSED void ptn_lazy_object_copy_clone_state(
    PtnRuntime *runtime,
    PtnObject *cloned,
    PtnObject *source
) {
    if (cloned == NULL || source == NULL || !source->lazy_is_proxy) {
        return;
    }
    cloned->lazy_is_proxy = 1;
    cloned->lazy_uninitialized = source->lazy_uninitialized;
    cloned->lazy_options = source->lazy_options;
    cloned->lazy_initializing = 0;
    ptn_value_destroy(&cloned->lazy_initializer);
    ptn_value_destroy(&cloned->lazy_proxy_instance);
    cloned->lazy_initializer = ptn_value_clone_deref(source->lazy_initializer);
    PtnValue real = ptn_value_deref(source->lazy_proxy_instance);
    if (!source->lazy_uninitialized && real.type == PTN_OBJECT && real.as.object != NULL) {
        cloned->lazy_proxy_instance =
            ptn_object_clone_storage_without_magic(runtime, real.as.object);
    } else {
        cloned->lazy_proxy_instance = ptn_value_clone_deref(source->lazy_proxy_instance);
    }
}

static PTN_UNUSED PtnValue ptn_throw_clone_method_visibility_error(
    PtnRuntime *runtime,
    int visibility,
    const char *declaring_class,
    size_t line
);

static PTN_UNUSED PtnValue ptn_object_invoke_clone_magic(
    PtnRuntime *runtime,
    PtnValue clone,
    size_t line
) {
    PtnValue resolved_clone = ptn_value_deref(clone);
    if (resolved_clone.type != PTN_OBJECT || resolved_clone.as.object == NULL) {
        return clone;
    }
    PtnObject *cloned = resolved_clone.as.object;
    PtnRuntime *root = runtime == NULL || runtime->lifecycle_root == NULL
        ? runtime
        : runtime->lifecycle_root;
    PtnRuntime *dispatch_runtime = runtime;
    if (dispatch_runtime == NULL ||
        dispatch_runtime->method_dispatch == NULL ||
        dispatch_runtime->declared_method_exists == NULL) {
        dispatch_runtime = root;
    }
    if (dispatch_runtime == NULL ||
        dispatch_runtime->method_dispatch == NULL ||
        dispatch_runtime->declared_method_exists == NULL ||
        !dispatch_runtime->declared_method_exists(cloned->class_name, "__clone")) {
        return clone;
    }

    PtnTryFrame clone_frame;
    int clone_frame_active = 0;
    const char *previous_clone_scope = dispatch_runtime->current_class_name;
    int previous_clone_initializing = cloned->readonly_clone_initializing;
    if (dispatch_runtime->exceptions != NULL) {
        ptn_try_frame_push(dispatch_runtime, &clone_frame);
        clone_frame_active = 1;
        if (setjmp(clone_frame.jump) != 0) {
            ptn_try_frame_pop(dispatch_runtime, &clone_frame);
            dispatch_runtime->current_class_name = previous_clone_scope;
            cloned->readonly_clone_initializing = previous_clone_initializing;
            ptn_value_destroy(&clone);
            ptn_rethrow_exception(dispatch_runtime);
            return ptn_null();
        }
    }

    const char *declaring_class = cloned->class_name;
    int visibility = PTN_PROPERTY_PUBLIC;
    int is_abstract = 0;
    if (dispatch_runtime->declared_method_visibility_metadata != NULL &&
        dispatch_runtime->declared_method_visibility_metadata(
            cloned->class_name,
            "__clone",
            &declaring_class,
            &visibility,
            &is_abstract
        )) {
        if (is_abstract) {
            char message[512];
            int written = snprintf(message, sizeof(message), "Cannot call abstract method %s::__clone()", declaring_class);
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception_at(dispatch_runtime, "Error", message, dispatch_runtime->source_path, line);
            return clone;
        }
        int visible = dispatch_runtime->declared_method_visible != NULL
            ? dispatch_runtime->declared_method_visible(
                visibility,
                declaring_class,
                cloned->class_name,
                "__clone",
                dispatch_runtime->current_class_name
            )
            : visibility == PTN_PROPERTY_PUBLIC;
        if (!visible) {
            (void)ptn_throw_clone_method_visibility_error(
                dispatch_runtime,
                visibility,
                declaring_class,
                line
            );
            return clone;
        }
        if (dispatch_runtime->reflected_method_dispatch != NULL) {
            dispatch_runtime->current_class_name = declaring_class;
            cloned->readonly_clone_initializing = 1;
            PtnValue result = ptn_null();
            int handled = dispatch_runtime->reflected_method_dispatch(
                dispatch_runtime,
                clone,
                cloned->class_name,
                "__clone",
                cloned->class_name,
                0,
                NULL,
                line,
                &result
            );
            cloned->readonly_clone_initializing = previous_clone_initializing;
            dispatch_runtime->current_class_name = previous_clone_scope;
            if (ptn_runtime_has_active_exception(dispatch_runtime)) {
                ptn_value_destroy(&result);
                if (clone_frame_active) {
                    ptn_try_frame_pop(dispatch_runtime, &clone_frame);
                }
                ptn_value_destroy(&clone);
                ptn_rethrow_exception(dispatch_runtime);
                return ptn_null();
            }
            if (handled) {
                ptn_value_destroy(&result);
                if (clone_frame_active) {
                    ptn_try_frame_pop(dispatch_runtime, &clone_frame);
                }
                return clone;
            }
            ptn_value_destroy(&result);
        }
    }

    cloned->readonly_clone_initializing = 1;
    PtnValue result = dispatch_runtime->method_dispatch(
        dispatch_runtime,
        clone,
        "__clone",
        0,
        NULL,
        line
    );
    cloned->readonly_clone_initializing = previous_clone_initializing;
    ptn_value_destroy(&result);
    if (ptn_runtime_has_active_exception(dispatch_runtime)) {
        if (clone_frame_active) {
            ptn_try_frame_pop(dispatch_runtime, &clone_frame);
        }
        ptn_value_destroy(&clone);
        ptn_rethrow_exception(dispatch_runtime);
        return ptn_null();
    }
    if (clone_frame_active) {
        ptn_try_frame_pop(dispatch_runtime, &clone_frame);
    }
    return clone;
}

static PTN_UNUSED PtnValue ptn_dom_clone(PtnRuntime *runtime, PtnValue value, size_t line);

static PTN_UNUSED const char *ptn_clone_method_visibility_name(int visibility) {
    if (visibility == PTN_PROPERTY_PRIVATE) {
        return "private";
    }
    if (visibility == PTN_PROPERTY_PROTECTED) {
        return "protected";
    }
    return "public";
}

static PTN_UNUSED PtnValue ptn_throw_clone_method_visibility_error(
    PtnRuntime *runtime,
    int visibility,
    const char *declaring_class,
    size_t line
) {
    const char *visibility_name = ptn_clone_method_visibility_name(visibility);
    const char *scope = runtime == NULL ? NULL : runtime->current_class_name;
    int needed;
    if (scope == NULL) {
        needed = snprintf(
            NULL,
            0,
            "Call to %s method %s::__clone() from global scope",
            visibility_name,
            declaring_class
        );
    } else {
        needed = snprintf(
            NULL,
            0,
            "Call to %s method %s::__clone() from scope %s",
            visibility_name,
            declaring_class,
            scope
        );
    }
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (scope == NULL) {
        snprintf(
            message,
            (size_t)needed + 1,
            "Call to %s method %s::__clone() from global scope",
            visibility_name,
            declaring_class
        );
    } else {
        snprintf(
            message,
            (size_t)needed + 1,
            "Call to %s method %s::__clone() from scope %s",
            visibility_name,
            declaring_class,
            scope
        );
    }
    ptn_throw_exception_owned_message_at(runtime, "Error", message, runtime->source_path, line);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_clone_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_CLOSURE) {
        return ptn_closure_clone(runtime, resolved);
    }
    if (resolved.type != PTN_OBJECT) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "clone(): Argument #1 ($object) must be of type object, %s given",
            ptn_offset_container_type_name(resolved)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    PtnObject *source = resolved.as.object;
    if (source->lazy_uninitialized && !source->lazy_initializing) {
        if (!ptn_lazy_object_initialize(runtime, resolved, line)) {
            return ptn_null();
        }
        source = resolved.as.object;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_internal_class_name_is_sensitive_parameter_value(source->class_name)) {
        return ptn_sensitive_parameter_value_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_weak_map(source->class_name)) {
        return ptn_weak_map_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "ArrayObject")) {
        return ptn_array_object_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "SplFixedArray")) {
        return ptn_spl_fixed_array_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "SplObjectStorage")) {
        return ptn_spl_object_storage_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "SplHeap") ||
        ptn_declared_class_is_same_or_descendant(source->class_name, "SplPriorityQueue")) {
        return ptn_spl_heap_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "ArrayIterator") ||
        ptn_declared_class_is_same_or_descendant(source->class_name, "RecursiveArrayIterator")) {
        return ptn_array_iterator_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "IntlBreakIterator")) {
        return ptn_intl_break_iterator_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "IntlDateFormatter") &&
        source->native_data == NULL) {
        ptn_throw_exception(runtime, "Error", "Cannot clone uninitialized IntlDateFormatter");
        return ptn_null();
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "NumberFormatter") &&
        source->native_data == NULL) {
        ptn_throw_exception(runtime, "Error", "Cannot clone uninitialized NumberFormatter");
        return ptn_null();
    }
    if (ptn_internal_class_name_is_message_formatter(source->class_name)) {
        if (source->native_data == NULL) {
            ptn_throw_exception(runtime, "Error", "Cannot clone uninitialized MessageFormatter");
            return ptn_null();
        }
        return ptn_intl_message_formatter_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_number_formatter(source->class_name)) {
        return ptn_intl_number_formatter_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_intl_number_range_formatter(source->class_name)) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Trying to clone an uncloneable object of class %s",
            source->class_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return ptn_null();
    }
    if (ptn_internal_class_name_is_intl_date_pattern_generator(source->class_name)) {
        return ptn_intl_date_pattern_generator_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "SplDoublyLinkedList")) {
        return ptn_spl_doubly_linked_list_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "SplFileObject")) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Trying to clone an uncloneable object of class %s",
            source->class_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return ptn_null();
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "DirectoryIterator")) {
        return ptn_directory_iterator_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "SplFileInfo")) {
        return ptn_spl_file_info_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_uri_rfc3986_uri(source->class_name) ||
        ptn_internal_class_name_is_uri_whatwg_url(source->class_name)) {
        return ptn_uri_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "DateTime") ||
        ptn_declared_class_is_same_or_descendant(source->class_name, "DateTimeImmutable")) {
        return ptn_datetime_clone(runtime, resolved, line);
    }
    if (ptn_declared_class_is_same_or_descendant(source->class_name, "DateTimeZone")) {
        return ptn_datetime_zone_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_dom(source->class_name)) {
        return ptn_dom_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_simplexml(source->class_name)) {
        return ptn_simplexml_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_hash_context(source->class_name)) {
        return ptn_hash_context_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_random_engine(source->class_name)) {
        return ptn_random_engine_clone(runtime, resolved, line);
    }
    if (ptn_internal_class_name_is_directory(source->class_name)) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Trying to clone an uncloneable object of class %s",
            source->class_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return ptn_null();
    }
#endif
    if (source->enum_case_name != NULL || source->native_data != NULL) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Trying to clone an uncloneable object of class %s",
            source->class_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return ptn_null();
    }

    PtnValue clone = ptn_object_clone_storage_without_magic(runtime, source);
    PtnObject *cloned = clone.as.object;
    ptn_lazy_object_copy_clone_state(runtime, cloned, source);

    PtnRuntime *root = runtime == NULL || runtime->lifecycle_root == NULL
        ? runtime
        : runtime->lifecycle_root;
    PtnRuntime *dispatch_runtime = runtime;
    if (dispatch_runtime == NULL ||
        dispatch_runtime->method_dispatch == NULL ||
        dispatch_runtime->declared_method_exists == NULL) {
        dispatch_runtime = root;
    }
    if (dispatch_runtime != NULL &&
        dispatch_runtime->method_dispatch != NULL &&
        dispatch_runtime->declared_method_exists != NULL &&
        dispatch_runtime->declared_method_exists(cloned->class_name, "__clone")) {
        PtnTryFrame clone_frame;
        int clone_frame_active = 0;
        const char *previous_clone_scope = dispatch_runtime->current_class_name;
        int previous_clone_initializing = cloned->readonly_clone_initializing;
        if (dispatch_runtime->exceptions != NULL) {
            ptn_try_frame_push(dispatch_runtime, &clone_frame);
            clone_frame_active = 1;
            if (setjmp(clone_frame.jump) != 0) {
                ptn_try_frame_pop(dispatch_runtime, &clone_frame);
                dispatch_runtime->current_class_name = previous_clone_scope;
                cloned->readonly_clone_initializing = previous_clone_initializing;
                ptn_value_destroy(&clone);
                ptn_rethrow_exception(dispatch_runtime);
                return ptn_null();
            }
        }
        const char *declaring_class = cloned->class_name;
        int visibility = PTN_PROPERTY_PUBLIC;
        int is_abstract = 0;
        if (dispatch_runtime->declared_method_visibility_metadata != NULL &&
            dispatch_runtime->declared_method_visibility_metadata(
                cloned->class_name,
                "__clone",
                &declaring_class,
                &visibility,
                &is_abstract
            )) {
            if (is_abstract) {
                char message[512];
                int written = snprintf(
                    message,
                    sizeof(message),
                    "Cannot call abstract method %s::__clone()",
                    declaring_class
                );
                if (written < 0 || (size_t)written >= sizeof(message)) {
                    ptn_abort_out_of_memory();
                }
                ptn_throw_exception_at(
                    dispatch_runtime,
                    "Error",
                    message,
                    dispatch_runtime->source_path,
                    line
                );
                return clone;
            }
            int visible = dispatch_runtime->declared_method_visible != NULL
                ? dispatch_runtime->declared_method_visible(
                    visibility,
                    declaring_class,
                    cloned->class_name,
                    "__clone",
                    dispatch_runtime->current_class_name
                )
                : visibility == PTN_PROPERTY_PUBLIC;
            if (!visible) {
                (void)ptn_throw_clone_method_visibility_error(
                    dispatch_runtime,
                    visibility,
                    declaring_class,
                    line
                );
                return clone;
            }
            if (dispatch_runtime->reflected_method_dispatch != NULL) {
                dispatch_runtime->current_class_name = declaring_class;
                cloned->readonly_clone_initializing = 1;
                PtnValue result = ptn_null();
                int handled = dispatch_runtime->reflected_method_dispatch(
                    dispatch_runtime,
                    clone,
                    cloned->class_name,
                    "__clone",
                    cloned->class_name,
                    0,
                    NULL,
                    line,
                    &result
                );
                cloned->readonly_clone_initializing = previous_clone_initializing;
                dispatch_runtime->current_class_name = previous_clone_scope;
                if (ptn_runtime_has_active_exception(dispatch_runtime)) {
                    ptn_value_destroy(&result);
                    if (clone_frame_active) {
                        ptn_try_frame_pop(dispatch_runtime, &clone_frame);
                    }
                    ptn_value_destroy(&clone);
                    ptn_rethrow_exception(dispatch_runtime);
                    return ptn_null();
                }
                if (handled) {
                    ptn_value_destroy(&result);
                    if (clone_frame_active) {
                        ptn_try_frame_pop(dispatch_runtime, &clone_frame);
                    }
                    return clone;
                }
                ptn_value_destroy(&result);
            }
        }
        cloned->readonly_clone_initializing = 1;
        PtnValue result = dispatch_runtime->method_dispatch(
            dispatch_runtime,
            clone,
            "__clone",
            0,
            NULL,
            line
        );
        cloned->readonly_clone_initializing = previous_clone_initializing;
        ptn_value_destroy(&result);
        if (ptn_runtime_has_active_exception(dispatch_runtime)) {
            if (clone_frame_active) {
                ptn_try_frame_pop(dispatch_runtime, &clone_frame);
            }
            ptn_value_destroy(&clone);
            ptn_rethrow_exception(dispatch_runtime);
            return ptn_null();
        }
        if (clone_frame_active) {
            ptn_try_frame_pop(dispatch_runtime, &clone_frame);
        }
    }
    return clone;
}

static PTN_UNUSED PtnValue ptn_cast_object(PtnRuntime *runtime, PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT) {
        return ptn_value_clone(value);
    }

    PtnValue object_value = ptn_object_new_shell(runtime, "stdClass");
    PtnObject *object = object_value.as.object;
    if (value.type == PTN_NULL) {
        return object_value;
    }

    if (value.type == PTN_ARRAY) {
        for (size_t i = 0; i < value.as.array->len; i++) {
            PtnArrayEntry *entry = &value.as.array->entries[i];
            PtnArrayKey property_key;
            if (entry->key.type == PTN_ARRAY_KEY_INT) {
                char key_buffer[64];
                int written = snprintf(
                    key_buffer,
                    sizeof(key_buffer),
                    "%lld",
                    (long long)entry->key.as.integer
                );
                if (written < 0 || (size_t)written >= sizeof(key_buffer)) {
                    ptn_abort_out_of_memory();
                }
                property_key = ptn_array_string_key(key_buffer);
            } else {
                property_key = ptn_array_string_key_len(
                    entry->key.as.string,
                    entry->key.string_len
                );
            }
            ptn_array_set_entry(
                object->properties,
                property_key,
                ptn_value_clone_deref(entry->value)
            );
        }
        return object_value;
    }

    ptn_array_set_entry(
        object->properties,
        ptn_array_string_key("scalar"),
        ptn_value_clone(value)
    );
    return object_value;
}

static PTN_UNUSED PtnValue ptn_cast_object_with_runtime(PtnRuntime *runtime, PtnValue value, size_t line) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_FLOAT && isnan(resolved.as.floating)) {
        ptn_emit_nan_coercion_warning(runtime, "object", line);
    }
    return ptn_cast_object(runtime, resolved);
}

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED int ptn_internal_cast_array_object(PtnValue value, PtnValue *array_out);
#endif

static PtnArrayKey ptn_public_object_property_array_key_from_name_len(const char *name, size_t len) {
    int64_t integer = 0;
    if (ptn_string_is_integer_array_key_len(name, len, &integer)) {
        return ptn_array_int_key(integer);
    }
    return ptn_array_string_key_len(name, len);
}

static PtnArrayKey ptn_public_object_property_array_key(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_STRING) {
        return ptn_public_object_property_array_key_from_name_len(key.as.string, key.string_len);
    }
    return ptn_array_key_clone(key);
}

static PtnArrayKey ptn_cast_array_object_property_key(PtnObject *object, PtnArrayKey key) {
    if (object == NULL || key.type != PTN_ARRAY_KEY_STRING) {
        return ptn_array_key_clone(key);
    }
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(object, key.as.string);
    if (metadata == NULL || metadata->read_visibility == PTN_PROPERTY_PUBLIC) {
        return ptn_public_object_property_array_key(key);
    }

    size_t display_len = strlen(metadata->display_name);
    if (metadata->read_visibility == PTN_PROPERTY_PROTECTED) {
        if (display_len > SIZE_MAX - 3) {
            ptn_abort_out_of_memory();
        }
        size_t key_len = display_len + 3;
        char *storage = malloc(key_len);
        if (storage == NULL) {
            ptn_abort_out_of_memory();
        }
        storage[0] = '\0';
        storage[1] = '*';
        storage[2] = '\0';
        memcpy(storage + 3, metadata->display_name, display_len);
        PtnArrayKey result_key = ptn_array_string_key_len(storage, key_len);
        free(storage);
        return result_key;
    }

    size_t declaring_len = strlen(metadata->declaring_class);
    if (declaring_len > SIZE_MAX - display_len - 2) {
        ptn_abort_out_of_memory();
    }
    size_t key_len = declaring_len + display_len + 2;
    char *storage = malloc(key_len);
    if (storage == NULL) {
        ptn_abort_out_of_memory();
    }
    storage[0] = '\0';
    memcpy(storage + 1, metadata->declaring_class, declaring_len);
    storage[declaring_len + 1] = '\0';
    memcpy(storage + declaring_len + 2, metadata->display_name, display_len);
    PtnArrayKey result_key = ptn_array_string_key_len(storage, key_len);
    free(storage);
    return result_key;
}

static PTN_UNUSED PtnValue ptn_cast_array(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        return ptn_value_clone(value);
    }

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (value.type == PTN_OBJECT) {
        PtnValue specialized_array = ptn_null();
        if (ptn_internal_cast_array_object(value, &specialized_array)) {
            return specialized_array;
        }
    }
#endif

    PtnValue array_value = ptn_array_from_literal_entries(0, NULL);
    PtnArray *array = array_value.as.array;
    if (value.type == PTN_NULL) {
        return array_value;
    }

    if (value.type == PTN_OBJECT) {
        PtnArray *properties = value.as.object->properties;
        for (size_t i = 0; i < value.as.object->property_metadata_len; i++) {
            const PtnObjectPropertyMetadata *metadata = &value.as.object->property_metadata[i];
            if (value.as.object->lazy_uninitialized &&
                !value.as.object->lazy_initializing &&
                !metadata->lazy_skip) {
                continue;
            }
            PtnArrayKey object_key = ptn_array_string_key(metadata->storage_name);
            PtnArrayEntry *entry = ptn_array_entry_for_key(properties, object_key);
            if (entry != NULL) {
                PtnArrayKey array_key =
                    ptn_cast_array_object_property_key(value.as.object, object_key);
                ptn_array_set_entry(
                    array,
                    array_key,
                    ptn_value_clone_deref(entry->value)
                );
            }
            ptn_array_key_free(object_key);
        }
        for (size_t i = 0; i < properties->len; i++) {
            PtnArrayEntry *entry = &properties->entries[i];
            const PtnObjectPropertyMetadata *metadata =
                entry->key.type == PTN_ARRAY_KEY_STRING
                    ? ptn_object_property_metadata(value.as.object, entry->key.as.string)
                    : NULL;
            if (metadata != NULL) {
                continue;
            }
            if (value.as.object->lazy_uninitialized && !value.as.object->lazy_initializing) {
                continue;
            }
            PtnArrayKey array_key =
                ptn_cast_array_object_property_key(value.as.object, entry->key);
            ptn_array_set_entry(
                array,
                array_key,
                ptn_value_clone_deref(entry->value)
            );
        }
        return array_value;
    }

    if (value.type == PTN_CLOSURE) {
        ptn_array_set_entry(array, ptn_array_int_key(0), ptn_value_clone(value));
        return array_value;
    }

    if (value.type == PTN_EXCEPTION) {
        return array_value;
    }

    ptn_array_set_entry(array, ptn_array_int_key(0), ptn_value_clone(value));
    return array_value;
}

static PTN_UNUSED PtnValue ptn_cast_array_with_runtime(PtnRuntime *runtime, PtnValue value, size_t line) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_FLOAT && isnan(resolved.as.floating)) {
        ptn_emit_nan_coercion_warning(runtime, "array", line);
    }
    return ptn_cast_array(resolved);
}

static PTN_UNUSED const char *ptn_property_non_object_receiver_name(PtnValue receiver) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_BOOL) {
        return receiver.as.boolean ? "true" : "false";
    }
    return ptn_offset_container_type_name(receiver);
}

static PTN_UNUSED int ptn_value_is_from_string_offset(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_STRING && value.from_string_offset;
}

static PTN_UNUSED void ptn_throw_string_offset_as_object_error(
    PtnRuntime *runtime,
    size_t line
) {
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Cannot use string offset as an object",
        runtime == NULL ? NULL : runtime->source_path,
        line
    );
}

static PTN_UNUSED void ptn_emit_non_object_property_read_warning(
    PtnRuntime *runtime,
    const char *property,
    PtnValue receiver,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Attempt to read property \"%s\" on %s",
        property,
        ptn_property_non_object_receiver_name(receiver)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
}

static PTN_UNUSED void ptn_emit_closure_undefined_property_warning(
    PtnRuntime *runtime,
    const char *property,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Undefined property: Closure::$%s",
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    runtime->diagnostics.emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(
        &runtime->diagnostics,
        PTN_E_WARNING,
        message,
        runtime->source_path,
        line
    )) {
        return;
    }
    ptn_diagnostic_printf(
        &runtime->diagnostics,
        "\nWarning: %s in %s on line %zu\n",
        message,
        runtime->source_path != NULL ? runtime->source_path : "ptn",
        line
    );
}

static PTN_UNUSED void ptn_throw_closure_dynamic_property_error(
    PtnRuntime *runtime,
    const char *property,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot create dynamic property Closure::$%s",
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_property_assignment_on_non_object(
    PtnRuntime *runtime,
    const char *property,
    PtnValue receiver,
    size_t line
) {
    if (ptn_value_is_from_string_offset(receiver)) {
        ptn_throw_string_offset_as_object_error(runtime, line);
        return;
    }
    if (ptn_value_deref(receiver).type == PTN_CLOSURE) {
        ptn_throw_closure_dynamic_property_error(runtime, property, line);
        return;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Attempt to assign property \"%s\" on %s",
        property,
        ptn_property_non_object_receiver_name(receiver)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_property_increment_on_non_object(
    PtnRuntime *runtime,
    const char *property,
    PtnValue receiver,
    size_t line
) {
    if (ptn_value_is_from_string_offset(receiver)) {
        ptn_throw_string_offset_as_object_error(runtime, line);
        return;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Attempt to increment/decrement property \"%s\" on %s",
        property,
        ptn_property_non_object_receiver_name(receiver)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_property_modification_on_non_object(
    PtnRuntime *runtime,
    const char *property,
    PtnValue receiver,
    size_t line
) {
    if (ptn_value_is_from_string_offset(receiver)) {
        ptn_throw_string_offset_as_object_error(runtime, line);
        return;
    }
    if (ptn_value_deref(receiver).type == PTN_CLOSURE) {
        ptn_throw_closure_dynamic_property_error(runtime, property, line);
        return;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Attempt to modify property \"%s\" on %s",
        property,
        ptn_property_non_object_receiver_name(receiver)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_object_is_incomplete_class(PtnObject *object);
static PTN_UNUSED void ptn_throw_incomplete_object_property_modification(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
);
static PTN_UNUSED void ptn_throw_incomplete_object_method_call(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
);
static PTN_UNUSED void ptn_emit_incomplete_object_property_access_warning(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
);

static PTN_UNUSED void ptn_validate_property_write_receiver(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(receiver);
    if (resolved.type != PTN_OBJECT && resolved.type != PTN_EXCEPTION) {
        ptn_throw_property_assignment_on_non_object(runtime, property, receiver, line);
        return;
    }
    if (resolved.type == PTN_OBJECT && ptn_object_is_incomplete_class(resolved.as.object)) {
        ptn_throw_incomplete_object_property_modification(runtime, resolved.as.object, line);
    }
}

static PTN_UNUSED void ptn_validate_property_modify_receiver(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(receiver);
    if (resolved.type != PTN_OBJECT && resolved.type != PTN_EXCEPTION) {
        ptn_throw_property_modification_on_non_object(runtime, property, receiver, line);
        return;
    }
    if (resolved.type == PTN_OBJECT && ptn_object_is_incomplete_class(resolved.as.object)) {
        ptn_throw_incomplete_object_property_modification(runtime, resolved.as.object, line);
    }
}

static PTN_UNUSED void ptn_validate_property_increment_receiver(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(receiver);
    if (resolved.type != PTN_OBJECT && resolved.type != PTN_EXCEPTION) {
        ptn_throw_property_increment_on_non_object(runtime, property, receiver, line);
        return;
    }
    if (resolved.type == PTN_OBJECT && ptn_object_is_incomplete_class(resolved.as.object)) {
        ptn_throw_incomplete_object_property_modification(runtime, resolved.as.object, line);
    }
}

static PTN_UNUSED void ptn_emit_undefined_property_warning(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Undefined property: %s::$%s",
        object->class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    runtime->diagnostics.emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(
        &runtime->diagnostics,
        PTN_E_WARNING,
        message,
        runtime->source_path,
        line
    )) {
        return;
    }
    ptn_diagnostic_printf(
        &runtime->diagnostics,
        "\nWarning: %s in %s on line %zu\n",
        message,
        runtime->source_path != NULL ? runtime->source_path : "ptn",
        line
    );
}

static PTN_UNUSED int ptn_property_is_get_only_virtual(
    const PtnObjectPropertyMetadata *metadata
) {
    return metadata != NULL &&
        metadata->is_virtual &&
        metadata->hook_has_get &&
        !metadata->hook_has_set;
}

static PTN_UNUSED int ptn_property_is_set_only_virtual(
    const PtnObjectPropertyMetadata *metadata
) {
    return metadata != NULL &&
        metadata->is_virtual &&
        metadata->hook_has_set &&
        !metadata->hook_has_get;
}

static PTN_UNUSED const char *ptn_property_hook_get_declaring_class(
    const PtnObjectPropertyMetadata *metadata
) {
    if (metadata == NULL) {
        return NULL;
    }
    return metadata->hook_get_declaring_class != NULL
        ? metadata->hook_get_declaring_class
        : metadata->declaring_class;
}

static PTN_UNUSED const char *ptn_property_hook_set_declaring_class(
    const PtnObjectPropertyMetadata *metadata
) {
    if (metadata == NULL) {
        return NULL;
    }
    return metadata->hook_set_declaring_class != NULL
        ? metadata->hook_set_declaring_class
        : metadata->declaring_class;
}

static PTN_UNUSED int ptn_active_property_hook_matches(
    PtnRuntime *runtime,
    PtnObject *receiver_object,
    const PtnObjectPropertyMetadata *metadata,
    const char *hook_declaring_class,
    const char *access_scope,
    const char *property
) {
    if (runtime == NULL ||
        metadata == NULL ||
        runtime->active_property_hook_class == NULL ||
        runtime->active_property_hook_property == NULL ||
        runtime->active_property_hook_object != receiver_object) {
        return 0;
    }
    if ((property != NULL &&
            strcmp(runtime->active_property_hook_property, property) == 0) ||
        strcmp(runtime->active_property_hook_property, metadata->display_name) == 0) {
        return 1;
    }
    return (
        access_scope != NULL &&
        ptn_ascii_case_equal(runtime->active_property_hook_class, access_scope) &&
        strcmp(runtime->active_property_hook_property, property) == 0
    ) ||
        (
            hook_declaring_class != NULL &&
            ptn_ascii_case_equal(runtime->active_property_hook_class, hook_declaring_class) &&
            strcmp(runtime->active_property_hook_property, metadata->display_name) == 0
        );
}

static PTN_UNUSED void ptn_throw_get_only_virtual_property_write_error(
    PtnRuntime *runtime,
    const PtnObjectPropertyMetadata *metadata,
    const char *class_name,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot write to get-only virtual property %s::$%s",
        class_name != NULL ? class_name : metadata->declaring_class,
        metadata->display_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_set_only_virtual_property_read_error(
    PtnRuntime *runtime,
    const PtnObjectPropertyMetadata *metadata,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot read from set-only virtual property %s::$%s",
        metadata->declaring_class,
        metadata->display_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_hooked_property_indirect_modification_error(
    PtnRuntime *runtime,
    const PtnObjectPropertyMetadata *metadata,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Indirect modification of %s::$%s is not allowed",
        metadata->declaring_class,
        metadata->display_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_create_reference_to_property_error(
    PtnRuntime *runtime,
    const PtnObjectPropertyMetadata *metadata,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot create reference to property %s::$%s",
        metadata->declaring_class,
        metadata->display_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_hooked_property_unset_error(
    PtnRuntime *runtime,
    const PtnObjectPropertyMetadata *metadata,
    const char *class_name,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot unset hooked property %s::$%s",
        class_name != NULL ? class_name : metadata->declaring_class,
        metadata->display_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_private_property_for_scope(
    PtnObject *object,
    const char *property,
    const char *access_scope
) {
    if (object == NULL || property == NULL || access_scope == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &object->property_metadata[i];
        if (
            metadata->read_visibility == PTN_PROPERTY_PRIVATE
            && strcmp(metadata->display_name, property) == 0
            && ptn_property_class_names_equal(metadata->declaring_class, access_scope)
        ) {
            return metadata;
        }
    }
    return NULL;
}

static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_own_private_property(
    PtnObject *object,
    const char *property
) {
    if (object == NULL || property == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &object->property_metadata[i];
        if (
            metadata->read_visibility == PTN_PROPERTY_PRIVATE
            && strcmp(metadata->display_name, property) == 0
            && ptn_property_class_names_equal(metadata->declaring_class, object->class_name)
        ) {
            return metadata;
        }
    }
    return NULL;
}

static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_named_shared_property(
    PtnObject *object,
    const char *property
) {
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(object, property);
    if (metadata != NULL && metadata->read_visibility != PTN_PROPERTY_PRIVATE) {
        return metadata;
    }
    return NULL;
}

static PTN_UNUSED int ptn_object_public_property_slot_exists(
    PtnObject *object,
    const char *property
) {
    if (object == NULL || property == NULL) {
        return 0;
    }
    PtnArrayKey key = ptn_array_string_key(property);
    PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
    ptn_array_key_free(key);
    return entry != NULL;
}

static PTN_UNUSED int ptn_object_property_storage_initialized(
    PtnObject *object,
    const char *storage_name
) {
    if (object == NULL || storage_name == NULL) {
        return 0;
    }
    PtnArrayKey key = ptn_array_string_key(storage_name);
    PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
    ptn_array_key_free(key);
    return entry != NULL;
}

static PTN_UNUSED int ptn_readonly_property_storage_initialized(
    PtnObject *object,
    const PtnObjectPropertyMetadata *metadata
) {
    if (metadata == NULL || !metadata->is_readonly) {
        return 0;
    }
    return ptn_object_property_storage_initialized(object, metadata->storage_name) ||
        ptn_object_property_storage_initialized(object, metadata->display_name);
}

static PTN_UNUSED PtnObjectPropertyMetadata *ptn_object_mutable_property_metadata(
    PtnObject *object,
    const char *storage_name
) {
    if (object == NULL || storage_name == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        if (strcmp(object->property_metadata[i].storage_name, storage_name) == 0) {
            return &object->property_metadata[i];
        }
    }
    return NULL;
}

static PTN_UNUSED PtnObjectPropertyMetadata *ptn_object_metadata_for_display_name(
    PtnObject *object,
    const char *property
) {
    if (object == NULL || property == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        if (strcmp(object->property_metadata[i].display_name, property) == 0) {
            return &object->property_metadata[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_property_visibility_scope_class(
    const PtnObjectPropertyMetadata *metadata,
    PtnPropertyVisibility visibility
) {
    if (metadata == NULL) {
        return NULL;
    }
    if (visibility == PTN_PROPERTY_PROTECTED) {
        return ptn_declared_class_property_prototype_class(
            metadata->declaring_class,
            metadata->display_name
        );
    }
    return metadata->declaring_class;
}

static PTN_UNUSED PtnObjectPropertyMetadata *ptn_object_blocked_magic_metadata(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int for_write
) {
    PtnObjectPropertyMetadata *metadata =
        ptn_object_metadata_for_display_name(object, property);
    if (metadata == NULL) {
        return NULL;
    }
    PtnPropertyVisibility visibility =
        for_write ? metadata->set_visibility : metadata->read_visibility;
    if (ptn_property_visibility_allows(
        runtime,
        visibility,
        ptn_property_visibility_scope_class(metadata, visibility),
        access_scope
    )) {
        return NULL;
    }
    return metadata;
}

static PTN_UNUSED int ptn_blocked_property_write_should_call_magic_set(
    const PtnObjectPropertyMetadata *metadata
) {
    if (metadata == NULL) {
        return 0;
    }
    return metadata->is_unset || metadata->set_visibility == metadata->read_visibility;
}

static PTN_UNUSED const char *ptn_property_value_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return "null";
        case PTN_BOOL:
            return "bool";
        case PTN_INT:
            return "int";
        case PTN_FLOAT:
            return "float";
        case PTN_STRING:
            return "string";
        case PTN_RESOURCE:
            return "resource";
        case PTN_ARRAY:
            return "array";
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            return "object";
        case PTN_REFERENCE:
            return "reference";
    }
    return "mixed";
}

static PTN_UNUSED void ptn_object_metadata_remember_value_type(
    PtnObjectPropertyMetadata *metadata,
    PtnValue value
) {
    if (metadata == NULL) {
        return;
    }
    free(metadata->last_type_name);
    metadata->last_type_name = metadata->type_text == NULL
        ? ptn_duplicate_string(ptn_property_value_type_name(value))
        : ptn_duplicate_string(metadata->type_text);
}

static PTN_UNUSED int ptn_property_type_is_declared(PtnPropertyTypeKind kind) {
    return kind != PTN_PROPERTY_TYPE_NONE;
}

static PTN_UNUSED const char *ptn_property_assignment_given_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT) {
        return value.as.object->class_name;
    }
    if (value.type == PTN_EXCEPTION) {
        return value.as.exception->class_name;
    }
    if (value.type == PTN_CLOSURE) {
        return "Closure";
    }
    return ptn_offset_container_type_name(value);
}

static PTN_UNUSED void ptn_throw_property_type_assignment_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    const char *type_text,
    PtnValue value,
    int reference_context,
    size_t line
) {
    char message[384];
    const char *given = ptn_property_assignment_given_name(value);
    const char *declared_type = type_text == NULL ? "mixed" : type_text;
    int written;
    if (reference_context) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot assign %s to reference held by property %s::$%s of type %s",
            given,
            declaring_class,
            property,
            declared_type
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot assign %s to property %s::$%s of type %s",
            given,
            declaring_class,
            property,
            declared_type
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_object_released_while_assigning_property(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Object was released while assigning to property %s::$%s",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_property_string_is_numeric(PtnString string, double *number) {
    char *copy = ptn_duplicate_string_len((const char *)string.data, string.len);
    int is_numeric = ptn_is_numeric_string(copy, number);
    free(copy);
    return is_numeric;
}

static PTN_UNUSED int ptn_property_double_fits_int(double value) {
    return isfinite(value) && value >= -9223372036854775808.0 && value < 9223372036854775808.0;
}

static PTN_UNUSED int ptn_property_string_to_int_for_assignment(
    PtnRuntime *runtime,
    PtnString string,
    size_t line,
    int emit_precision_deprecation,
    int64_t *integer
) {
    char *copy = ptn_duplicate_string_len((const char *)string.data, string.len);
    double number = 0.0;
    int accepted = 0;
    if (ptn_is_numeric_string(copy, &number) && ptn_property_double_fits_int(number)) {
        if (emit_precision_deprecation && ptn_float_to_int_loses_precision(number)) {
            ptn_emit_float_string_to_int_precision_deprecation_at(
                runtime == NULL ? NULL : &runtime->diagnostics,
                copy,
                runtime == NULL || runtime->source_path == NULL ? "ptn" : runtime->source_path,
                line
            );
        }
        *integer = (int64_t)number;
        accepted = 1;
    }
    free(copy);
    return accepted;
}

static PTN_UNUSED void ptn_property_trim_type_span(
    const char **start,
    size_t *len
) {
    while (*len > 0 && isspace((unsigned char)(*start)[0])) {
        (*start)++;
        (*len)--;
    }
    while (*len > 0 && isspace((unsigned char)(*start)[*len - 1])) {
        (*len)--;
    }
    if (*len >= 2 && (*start)[0] == '(' && (*start)[*len - 1] == ')') {
        int depth = 0;
        int wraps = 1;
        for (size_t i = 0; i < *len; i++) {
            if ((*start)[i] == '(') {
                depth++;
            } else if ((*start)[i] == ')') {
                depth--;
                if (depth == 0 && i + 1 < *len) {
                    wraps = 0;
                    break;
                }
            }
        }
        if (wraps) {
            (*start)++;
            *len -= 2;
            ptn_property_trim_type_span(start, len);
        }
    }
    if (*len > 0 && (*start)[0] == '?') {
        (*start)++;
        (*len)--;
        ptn_property_trim_type_span(start, len);
    }
}

static PTN_UNUSED char *ptn_property_type_span_to_string(const char *start, size_t len) {
    char *copy = malloc(len + 1);
    if (copy == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(copy, start, len);
    copy[len] = '\0';
    return copy;
}

static PTN_UNUSED int ptn_property_type_text_allows_float(const char *start, size_t len) {
    ptn_property_trim_type_span(&start, &len);
    if (len == 0) {
        return 0;
    }
    int depth = 0;
    int saw_union = 0;
    for (size_t i = 0; i < len; i++) {
        char ch = start[i];
        if (ch == '(') {
            depth++;
            continue;
        }
        if (ch == ')') {
            depth--;
            continue;
        }
        if (ch == '|' && depth == 0) {
            saw_union = 1;
            break;
        }
    }
    if (saw_union) {
        size_t part_start = 0;
        depth = 0;
        for (size_t i = 0; i <= len; i++) {
            char ch = i < len ? start[i] : '|';
            if (i < len && ch == '(') {
                depth++;
                continue;
            }
            if (i < len && ch == ')') {
                depth--;
                continue;
            }
            if (i < len && (ch != '|' || depth != 0)) {
                continue;
            }
            if (ptn_property_type_text_allows_float(start + part_start, i - part_start)) {
                return 1;
            }
            part_start = i + 1;
        }
        return 0;
    }

    depth = 0;
    int saw_intersection = 0;
    for (size_t i = 0; i < len; i++) {
        char ch = start[i];
        if (ch == '(') {
            depth++;
            continue;
        }
        if (ch == ')') {
            depth--;
            continue;
        }
        if (ch == '&' && depth == 0) {
            saw_intersection = 1;
            break;
        }
    }
    if (saw_intersection) {
        size_t part_start = 0;
        depth = 0;
        for (size_t i = 0; i <= len; i++) {
            char ch = i < len ? start[i] : '&';
            if (i < len && ch == '(') {
                depth++;
                continue;
            }
            if (i < len && ch == ')') {
                depth--;
                continue;
            }
            if (i < len && (ch != '&' || depth != 0)) {
                continue;
            }
            if (!ptn_property_type_text_allows_float(start + part_start, i - part_start)) {
                return 0;
            }
            part_start = i + 1;
        }
        return 1;
    }

    return ptn_ascii_case_equal_span_to_string(start, len, "float") ||
        ptn_ascii_case_equal_span_to_string(start, len, "mixed");
}

static PTN_UNUSED int ptn_property_type_allows_float(
    PtnPropertyTypeKind kind,
    const char *type_text
) {
    switch (kind) {
        case PTN_PROPERTY_TYPE_NONE:
        case PTN_PROPERTY_TYPE_MIXED:
        case PTN_PROPERTY_TYPE_FLOAT:
            return 1;
        case PTN_PROPERTY_TYPE_TEXT:
            return type_text != NULL &&
                ptn_property_type_text_allows_float(type_text, strlen(type_text));
        case PTN_PROPERTY_TYPE_NULL:
        case PTN_PROPERTY_TYPE_ARRAY:
        case PTN_PROPERTY_TYPE_INT:
        case PTN_PROPERTY_TYPE_STRING:
        case PTN_PROPERTY_TYPE_BOOL:
        case PTN_PROPERTY_TYPE_OBJECT:
        case PTN_PROPERTY_TYPE_CLASS:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_reference_property_type_source_allows_float(
    const PtnReferencePropertyTypeSource *source
) {
    return source == NULL ||
        ptn_property_type_allows_float(source->kind, source->text);
}

static PTN_UNUSED int ptn_property_type_text_coerce_atom(
    PtnRuntime *runtime,
    const char *start,
    size_t len,
    PtnValue resolved,
    int allow_scalar_coercion,
    PtnValue *out
) {
    ptn_property_trim_type_span(&start, &len);
    if (len == 0) {
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "mixed")) {
        *out = ptn_value_clone(resolved);
        return 1;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "null")) {
        if (resolved.type == PTN_NULL) {
            *out = ptn_null();
            return 1;
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "array")) {
        if (resolved.type == PTN_ARRAY) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "object")) {
        if (resolved.type == PTN_OBJECT ||
            resolved.type == PTN_CLOSURE ||
            resolved.type == PTN_EXCEPTION) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "iterable")) {
        if (resolved.type == PTN_ARRAY) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        if (resolved.type == PTN_OBJECT ||
            resolved.type == PTN_CLOSURE ||
            resolved.type == PTN_EXCEPTION) {
            if (ptn_value_satisfies_class_type_hint(runtime, resolved, "Traversable") ||
                ptn_value_satisfies_class_type_hint(runtime, resolved, "Iterator")) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "int")) {
        if (resolved.type == PTN_INT) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        if (!allow_scalar_coercion) {
            return 0;
        }
        if (resolved.type == PTN_BOOL) {
            *out = ptn_cast_int(resolved);
            return 1;
        }
        if (resolved.type == PTN_FLOAT && ptn_property_double_fits_int(resolved.as.floating)) {
            *out = ptn_cast_int(resolved);
            return 1;
        }
        if (resolved.type == PTN_STRING) {
            double number = 0.0;
            if (ptn_property_string_is_numeric(resolved.as.string, &number) &&
                ptn_property_double_fits_int(number)) {
                *out = ptn_int((int64_t)number);
                return 1;
            }
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "float")) {
        if (resolved.type == PTN_FLOAT) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        if (!allow_scalar_coercion) {
            return 0;
        }
        if (resolved.type == PTN_INT || resolved.type == PTN_BOOL) {
            *out = ptn_cast_float(resolved);
            return 1;
        }
        if (resolved.type == PTN_STRING) {
            double number = 0.0;
            if (ptn_property_string_is_numeric(resolved.as.string, &number)) {
                *out = ptn_float(number);
                return 1;
            }
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "string")) {
        if (resolved.type == PTN_STRING) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        if (!allow_scalar_coercion) {
            return 0;
        }
        if (resolved.type == PTN_INT ||
            resolved.type == PTN_FLOAT ||
            resolved.type == PTN_BOOL) {
            *out = ptn_cast_string(resolved);
            return 1;
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "bool")) {
        if (resolved.type == PTN_BOOL) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        if (!allow_scalar_coercion) {
            return 0;
        }
        if (resolved.type == PTN_INT ||
            resolved.type == PTN_FLOAT ||
            resolved.type == PTN_STRING) {
            *out = ptn_bool(ptn_is_truthy(resolved));
            return 1;
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "true")) {
        if (resolved.type == PTN_BOOL && resolved.as.boolean) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        return 0;
    }
    if (ptn_ascii_case_equal_span_to_string(start, len, "false")) {
        if (resolved.type == PTN_BOOL && !resolved.as.boolean) {
            *out = ptn_value_clone(resolved);
            return 1;
        }
        return 0;
    }
    char *class_name = ptn_property_type_span_to_string(start, len);
    int matches = ptn_value_satisfies_class_type_hint(runtime, resolved, class_name);
    free(class_name);
    if (matches) {
        *out = ptn_value_clone(resolved);
        return 1;
    }
    return 0;
}

static PTN_UNUSED int ptn_property_type_text_coerce_intersection(
    PtnRuntime *runtime,
    const char *start,
    size_t len,
    PtnValue resolved,
    int allow_scalar_coercion,
    PtnValue *out
) {
    ptn_property_trim_type_span(&start, &len);
    int depth = 0;
    size_t part_start = 0;
    int saw_intersection = 0;
    for (size_t i = 0; i <= len; i++) {
        char ch = i < len ? start[i] : '&';
        if (i < len && ch == '(') {
            depth++;
            continue;
        }
        if (i < len && ch == ')') {
            depth--;
            continue;
        }
        if (i < len && (ch != '&' || depth != 0)) {
            continue;
        }
        size_t part_len = i - part_start;
        PtnValue ignored = ptn_null();
        if (!ptn_property_type_text_coerce_atom(
            runtime,
            start + part_start,
            part_len,
            resolved,
            allow_scalar_coercion,
            &ignored
        )) {
            return 0;
        }
        ptn_value_destroy(&ignored);
        if (i < len) {
            saw_intersection = 1;
        }
        part_start = i + 1;
    }
    if (saw_intersection) {
        *out = ptn_value_clone(resolved);
        return 1;
    }
    return ptn_property_type_text_coerce_atom(runtime, start, len, resolved, allow_scalar_coercion, out);
}

static PTN_UNUSED int ptn_property_type_text_coerce_assignment(
    PtnRuntime *runtime,
    const char *type_text,
    PtnValue resolved,
    PtnValue *out
) {
    if (type_text == NULL) {
        return 0;
    }
    const char *start = type_text;
    size_t len = strlen(type_text);
    ptn_property_trim_type_span(&start, &len);
    for (int allow_scalar_coercion = 0; allow_scalar_coercion <= 1; allow_scalar_coercion++) {
        if (allow_scalar_coercion && resolved.type == PTN_INT) {
            int depth = 0;
            size_t part_start = 0;
            for (size_t i = 0; i <= len; i++) {
                char ch = i < len ? start[i] : '|';
                if (i < len && ch == '(') {
                    depth++;
                    continue;
                }
                if (i < len && ch == ')') {
                    depth--;
                    continue;
                }
                if (i < len && (ch != '|' || depth != 0)) {
                    continue;
                }
                const char *part = start + part_start;
                size_t part_len = i - part_start;
                ptn_property_trim_type_span(&part, &part_len);
                if (ptn_ascii_case_equal_span_to_string(part, part_len, "float") &&
                    ptn_property_type_text_coerce_atom(
                        runtime,
                        part,
                        part_len,
                        resolved,
                        allow_scalar_coercion,
                        out
                    )) {
                    return 1;
                }
                part_start = i + 1;
            }
        }
        int depth = 0;
        size_t part_start = 0;
        for (size_t i = 0; i <= len; i++) {
            char ch = i < len ? start[i] : '|';
            if (i < len && ch == '(') {
                depth++;
                continue;
            }
            if (i < len && ch == ')') {
                depth--;
                continue;
            }
            if (i < len && (ch != '|' || depth != 0)) {
                continue;
            }
            if (ptn_property_type_text_coerce_intersection(
                runtime,
                start + part_start,
                i - part_start,
                resolved,
                allow_scalar_coercion,
                out
            )) {
                return 1;
            }
            part_start = i + 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_property_type_coerce_assignment(
    PtnRuntime *runtime,
    PtnPropertyTypeKind kind,
    const char *type_class_name,
    const char *type_text,
    int allows_null,
    const char *declaring_class,
    const char *property,
    PtnValue value,
    int reference_context,
    size_t line,
    PtnValue *out
) {
    PtnValue resolved = ptn_value_deref(value);
    int weak_scalar_coercion = runtime == NULL || !runtime->strict_types;
    if (kind == PTN_PROPERTY_TYPE_NONE) {
        *out = ptn_value_clone(resolved);
        return 1;
    }
    if (kind == PTN_PROPERTY_TYPE_MIXED) {
        *out = ptn_value_clone(resolved);
        return 1;
    }
    if (resolved.type == PTN_NULL) {
        if (allows_null || kind == PTN_PROPERTY_TYPE_NULL) {
            *out = ptn_null();
            return 1;
        }
        ptn_throw_property_type_assignment_error(
            runtime,
            declaring_class,
            property,
            type_text,
            resolved,
            reference_context,
            line
        );
        return 0;
    }
    switch (kind) {
        case PTN_PROPERTY_TYPE_NULL:
            break;
        case PTN_PROPERTY_TYPE_ARRAY:
            if (resolved.type == PTN_ARRAY) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            break;
        case PTN_PROPERTY_TYPE_INT:
            if (resolved.type == PTN_INT) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            if (weak_scalar_coercion && resolved.type == PTN_BOOL) {
                *out = ptn_cast_int(resolved);
                return 1;
            }
            if (weak_scalar_coercion &&
                resolved.type == PTN_FLOAT &&
                ptn_property_double_fits_int(resolved.as.floating)) {
                if (ptn_float_to_int_loses_precision(resolved.as.floating)) {
                    ptn_emit_float_to_int_precision_deprecation_at(
                        runtime == NULL ? NULL : &runtime->diagnostics,
                        resolved.as.floating,
                        runtime == NULL || runtime->source_path == NULL ? "ptn" : runtime->source_path,
                        line
                    );
                }
                *out = ptn_cast_int(resolved);
                return 1;
            }
            if (weak_scalar_coercion && resolved.type == PTN_STRING) {
                int64_t integer = 0;
                if (ptn_property_string_to_int_for_assignment(
                        runtime,
                        resolved.as.string,
                        line,
                        1,
                        &integer
                    )) {
                    *out = ptn_int(integer);
                    return 1;
                }
            }
            break;
        case PTN_PROPERTY_TYPE_FLOAT:
            if (resolved.type == PTN_FLOAT) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            if (resolved.type == PTN_INT ||
                (weak_scalar_coercion && resolved.type == PTN_BOOL)) {
                *out = ptn_cast_float(resolved);
                return 1;
            }
            if (weak_scalar_coercion && resolved.type == PTN_STRING) {
                double number = 0.0;
                if (ptn_property_string_is_numeric(resolved.as.string, &number)) {
                    *out = ptn_float(number);
                    return 1;
                }
            }
            break;
        case PTN_PROPERTY_TYPE_STRING:
            if (resolved.type == PTN_STRING ||
                (weak_scalar_coercion &&
                 (resolved.type == PTN_INT ||
                  resolved.type == PTN_FLOAT ||
                  resolved.type == PTN_BOOL))) {
                *out = ptn_cast_string(resolved);
                return 1;
            }
            if (weak_scalar_coercion && resolved.type == PTN_OBJECT) {
                PtnStringOperand object_string;
                if (ptn_try_object_to_string_operand(runtime, resolved, line, &object_string)) {
                    char *copy = ptn_duplicate_string_len(object_string.data, object_string.len);
                    size_t len = object_string.len;
                    ptn_string_operand_free(object_string);
                    *out = ptn_owned_string_len(copy, len);
                    return 1;
                }
            }
            break;
        case PTN_PROPERTY_TYPE_BOOL:
            if (resolved.type == PTN_BOOL) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            if (weak_scalar_coercion &&
                (resolved.type == PTN_INT ||
                 resolved.type == PTN_FLOAT ||
                 resolved.type == PTN_STRING)) {
                *out = ptn_bool(ptn_is_truthy(resolved));
                return 1;
            }
            break;
        case PTN_PROPERTY_TYPE_OBJECT:
            if (resolved.type == PTN_OBJECT ||
                resolved.type == PTN_CLOSURE ||
                resolved.type == PTN_EXCEPTION) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            break;
        case PTN_PROPERTY_TYPE_CLASS:
            if (type_class_name != NULL &&
                ptn_value_satisfies_class_type_hint(runtime, resolved, type_class_name)) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            break;
        case PTN_PROPERTY_TYPE_TEXT:
            if (ptn_property_type_text_coerce_assignment(runtime, type_text, resolved, out)) {
                return 1;
            }
            break;
        case PTN_PROPERTY_TYPE_NONE:
        case PTN_PROPERTY_TYPE_MIXED:
            *out = ptn_value_clone(resolved);
            return 1;
    }
    ptn_throw_property_type_assignment_error(
        runtime,
        declaring_class,
        property,
        type_text,
        resolved,
        reference_context,
        line
    );
    return 0;
}

static PTN_UNUSED int ptn_property_type_try_coerce_assignment(
    PtnRuntime *runtime,
    PtnPropertyTypeKind kind,
    const char *type_class_name,
    const char *type_text,
    int allows_null,
    PtnValue value,
    PtnValue *out
) {
    PtnValue resolved = ptn_value_deref(value);
    int weak_scalar_coercion = runtime == NULL || !runtime->strict_types;
    if (kind == PTN_PROPERTY_TYPE_NONE || kind == PTN_PROPERTY_TYPE_MIXED) {
        *out = ptn_value_clone(resolved);
        return 1;
    }
    if (resolved.type == PTN_NULL) {
        if (allows_null || kind == PTN_PROPERTY_TYPE_NULL) {
            *out = ptn_null();
            return 1;
        }
        return 0;
    }
    switch (kind) {
        case PTN_PROPERTY_TYPE_NULL:
            return 0;
        case PTN_PROPERTY_TYPE_ARRAY:
            if (resolved.type == PTN_ARRAY) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            return 0;
        case PTN_PROPERTY_TYPE_INT:
            if (resolved.type == PTN_INT) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            if (weak_scalar_coercion && resolved.type == PTN_BOOL) {
                *out = ptn_cast_int(resolved);
                return 1;
            }
            if (weak_scalar_coercion &&
                resolved.type == PTN_FLOAT &&
                ptn_property_double_fits_int(resolved.as.floating)) {
                *out = ptn_cast_int(resolved);
                return 1;
            }
            if (weak_scalar_coercion && resolved.type == PTN_STRING) {
                int64_t integer = 0;
                if (ptn_property_string_to_int_for_assignment(
                        runtime,
                        resolved.as.string,
                        0,
                        0,
                        &integer
                    )) {
                    *out = ptn_int(integer);
                    return 1;
                }
            }
            return 0;
        case PTN_PROPERTY_TYPE_FLOAT:
            if (resolved.type == PTN_FLOAT) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            if (resolved.type == PTN_INT ||
                (weak_scalar_coercion && resolved.type == PTN_BOOL)) {
                *out = ptn_cast_float(resolved);
                return 1;
            }
            if (weak_scalar_coercion && resolved.type == PTN_STRING) {
                double number = 0.0;
                if (ptn_property_string_is_numeric(resolved.as.string, &number)) {
                    *out = ptn_float(number);
                    return 1;
                }
            }
            return 0;
        case PTN_PROPERTY_TYPE_STRING:
            if (resolved.type == PTN_STRING ||
                (weak_scalar_coercion &&
                 (resolved.type == PTN_INT ||
                  resolved.type == PTN_FLOAT ||
                  resolved.type == PTN_BOOL))) {
                *out = ptn_cast_string(resolved);
                return 1;
            }
            return 0;
        case PTN_PROPERTY_TYPE_BOOL:
            if (resolved.type == PTN_BOOL) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            if (weak_scalar_coercion &&
                (resolved.type == PTN_INT ||
                 resolved.type == PTN_FLOAT ||
                 resolved.type == PTN_STRING)) {
                *out = ptn_bool(ptn_is_truthy(resolved));
                return 1;
            }
            return 0;
        case PTN_PROPERTY_TYPE_OBJECT:
            if (resolved.type == PTN_OBJECT ||
                resolved.type == PTN_CLOSURE ||
                resolved.type == PTN_EXCEPTION) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            return 0;
        case PTN_PROPERTY_TYPE_CLASS:
            if (type_class_name != NULL &&
                ptn_value_satisfies_class_type_hint(runtime, resolved, type_class_name)) {
                *out = ptn_value_clone(resolved);
                return 1;
            }
            return 0;
        case PTN_PROPERTY_TYPE_TEXT:
            return ptn_property_type_text_coerce_assignment(runtime, type_text, resolved, out);
        case PTN_PROPERTY_TYPE_NONE:
        case PTN_PROPERTY_TYPE_MIXED:
            *out = ptn_value_clone(resolved);
            return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_throw_unset_typed_property_magic_get_error(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnObjectPropertyMetadata *metadata,
    PtnValue value,
    size_t line
) {
    char message[512];
    PtnValue resolved = ptn_value_deref(value);
    const char *given = ptn_property_assignment_given_name(resolved);
    const char *getter_class = receiver.type == PTN_OBJECT
        ? receiver.as.object->class_name
        : metadata->declaring_class;
    const char *declared_type = metadata->type_text == NULL ? "mixed" : metadata->type_text;
    int written = snprintf(
        message,
        sizeof(message),
        "Value of type %s returned from %s::__get() must be compatible with unset property %s::$%s of type %s",
        given,
        getter_class,
        metadata->declaring_class,
        metadata->display_name,
        declared_type
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_coerce_unset_typed_property_magic_value(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnObjectPropertyMetadata *metadata,
    PtnValue value,
    size_t line,
    PtnValue *out
) {
    if (ptn_property_type_try_coerce_assignment(
        runtime,
        metadata->type_kind,
        metadata->type_class_name,
        metadata->type_text,
        metadata->type_allows_null,
        value,
        out
    )) {
        return 1;
    }
    ptn_throw_unset_typed_property_magic_get_error(runtime, receiver, metadata, value, line);
    return 0;
}

static PTN_UNUSED int ptn_reference_property_type_source_coerce_assignment(
    PtnRuntime *runtime,
    const PtnReferencePropertyTypeSource *source,
    PtnValue value,
    int reference_context,
    size_t line,
    PtnValue *out
) {
    return ptn_property_type_coerce_assignment(
        runtime,
        source->kind,
        source->class_name,
        source->text,
        source->allows_null,
        source->declaring_class,
        source->property_name,
        value,
        reference_context,
        line,
        out
    );
}

static PTN_UNUSED void ptn_throw_reference_inconsistent_assignment_error(
    PtnRuntime *runtime,
    PtnValue value,
    const PtnReferencePropertyTypeSource *first,
    const PtnReferencePropertyTypeSource *second
) {
    char message[768];
    const char *given = ptn_property_assignment_given_name(value);
    const char *first_type = first->text == NULL ? "mixed" : first->text;
    const char *second_type = second->text == NULL ? "mixed" : second->text;
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot assign %s to reference held by property %s::$%s of type %s and property %s::$%s of type %s, as this would result in an inconsistent type conversion",
        given,
        first->declaring_class,
        first->property_name,
        first_type,
        second->declaring_class,
        second->property_name,
        second_type
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static PTN_UNUSED PtnReferencePropertyTypeSource ptn_reference_primary_property_type_source(
    const PtnReference *reference
) {
    PtnReferencePropertyTypeSource source;
    source.kind = reference->property_type_kind;
    source.class_name = reference->property_type_class_name;
    source.text = reference->property_type_text;
    source.allows_null = reference->property_type_allows_null;
    source.declaring_class = reference->property_declaring_class;
    source.property_name = reference->property_name;
    return source;
}

static PTN_UNUSED int ptn_property_reference_coerce_assignment(
    PtnRuntime *runtime,
    const PtnReference *reference,
    PtnValue value,
    int reference_context,
    size_t line,
    PtnValue *out
) {
    if (reference == NULL || reference->property_type_kind == PTN_PROPERTY_TYPE_NONE) {
        *out = ptn_value_clone_deref(value);
        return 1;
    }
    PtnReferencePropertyTypeSource primary =
        ptn_reference_primary_property_type_source(reference);
    PtnValue coerced = ptn_null();
    if (!ptn_reference_property_type_source_coerce_assignment(
        runtime,
        &primary,
        value,
        reference_context,
        line,
        &coerced
    )) {
        return 0;
    }
    for (size_t i = 0; i < reference->property_type_source_len; i++) {
        PtnValue next = ptn_null();
        if (!ptn_reference_property_type_source_coerce_assignment(
            runtime,
            &reference->property_type_sources[i],
            value,
            reference_context,
            line,
            &next
        )) {
            ptn_value_destroy(&coerced);
            return 0;
        }
        if (!ptn_compare_identical(runtime, coerced, next, line)) {
            ptn_throw_reference_inconsistent_assignment_error(
                runtime,
                value,
                &primary,
                &reference->property_type_sources[i]
            );
            ptn_value_destroy(&coerced);
            ptn_value_destroy(&next);
            return 0;
        }
        ptn_value_destroy(&next);
    }
    *out = coerced;
    return 1;
}

static PTN_UNUSED int ptn_reference_property_type_source_accepts_array_auto_initialization(
    PtnRuntime *runtime,
    const PtnReferencePropertyTypeSource *source
) {
    return source != NULL &&
        ptn_property_type_accepts_array_auto_initialization(
            runtime,
            source->kind,
            source->class_name,
            source->text,
            source->allows_null
        );
}

static PTN_UNUSED int ptn_reference_property_types_accept_array_auto_initialization(
    PtnRuntime *runtime,
    const PtnReference *reference,
    PtnReferencePropertyTypeSource *blocking_source
) {
    if (reference == NULL || reference->property_type_kind == PTN_PROPERTY_TYPE_NONE) {
        return 1;
    }
    PtnReferencePropertyTypeSource primary =
        ptn_reference_primary_property_type_source(reference);
    if (!ptn_reference_property_type_source_accepts_array_auto_initialization(runtime, &primary)) {
        if (blocking_source != NULL) {
            *blocking_source = primary;
        }
        return 0;
    }
    for (size_t i = 0; i < reference->property_type_source_len; i++) {
        if (!ptn_reference_property_type_source_accepts_array_auto_initialization(
            runtime,
            &reference->property_type_sources[i]
        )) {
            if (blocking_source != NULL) {
                *blocking_source = reference->property_type_sources[i];
            }
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED int ptn_reference_property_source_matches(
    const PtnReferencePropertyTypeSource *source,
    const PtnObjectPropertyMetadata *metadata
) {
    return source != NULL &&
        metadata != NULL &&
        source->declaring_class != NULL &&
        source->property_name != NULL &&
        strcmp(source->declaring_class, metadata->declaring_class) == 0 &&
        strcmp(source->property_name, metadata->display_name) == 0;
}

static PTN_UNUSED PtnReferencePropertyTypeSource ptn_reference_property_source_from_metadata(
    const PtnObjectPropertyMetadata *metadata
) {
    PtnReferencePropertyTypeSource source;
    source.kind = metadata->type_kind;
    source.class_name = metadata->type_class_name == NULL
        ? NULL
        : ptn_duplicate_string(metadata->type_class_name);
    source.text = metadata->type_text == NULL
        ? NULL
        : ptn_duplicate_string(metadata->type_text);
    source.allows_null = metadata->type_allows_null;
    source.declaring_class = ptn_duplicate_string(metadata->declaring_class);
    source.property_name = ptn_duplicate_string(metadata->display_name);
    return source;
}

static PTN_UNUSED void ptn_reference_adopt_property_type(
    PtnReference *reference,
    const PtnObjectPropertyMetadata *metadata
) {
    if (reference == NULL ||
        metadata == NULL ||
        !ptn_property_type_is_declared(metadata->type_kind)) {
        return;
    }
    if (reference->property_type_kind != PTN_PROPERTY_TYPE_NONE) {
        PtnReferencePropertyTypeSource primary =
            ptn_reference_primary_property_type_source(reference);
        if (ptn_reference_property_source_matches(&primary, metadata)) {
            return;
        }
        for (size_t i = 0; i < reference->property_type_source_len; i++) {
            if (ptn_reference_property_source_matches(&reference->property_type_sources[i], metadata)) {
                return;
            }
        }
        if (reference->property_type_source_len == reference->property_type_source_cap) {
            size_t new_cap = reference->property_type_source_cap == 0
                ? 2
                : reference->property_type_source_cap * 2;
            PtnReferencePropertyTypeSource *new_sources = realloc(
                reference->property_type_sources,
                new_cap * sizeof(PtnReferencePropertyTypeSource)
            );
            if (new_sources == NULL) {
                ptn_abort_out_of_memory();
            }
            reference->property_type_sources = new_sources;
            reference->property_type_source_cap = new_cap;
        }
        reference->property_type_sources[reference->property_type_source_len++] =
            ptn_reference_property_source_from_metadata(metadata);
        return;
    }
    free(reference->property_type_class_name);
    free(reference->property_type_text);
    free(reference->property_declaring_class);
    free(reference->property_name);
    reference->property_type_kind = metadata->type_kind;
    reference->property_type_class_name =
        metadata->type_class_name == NULL ? NULL : ptn_duplicate_string(metadata->type_class_name);
    reference->property_type_text =
        metadata->type_text == NULL ? NULL : ptn_duplicate_string(metadata->type_text);
    reference->property_type_allows_null = metadata->type_allows_null;
    reference->property_declaring_class = ptn_duplicate_string(metadata->declaring_class);
    reference->property_name = ptn_duplicate_string(metadata->display_name);
}

static PTN_UNUSED void ptn_reference_remember_property_identity(
    PtnReference *reference,
    const PtnObjectPropertyMetadata *metadata
) {
    if (reference == NULL || metadata == NULL) {
        return;
    }
    if (reference->property_declaring_class != NULL || reference->property_name != NULL) {
        return;
    }
    reference->property_declaring_class = ptn_duplicate_string(metadata->declaring_class);
    reference->property_name = ptn_duplicate_string(metadata->display_name);
}

static PTN_UNUSED const char *ptn_reference_property_storage_key_for_object(
    PtnObject *object,
    PtnReference *reference
) {
    if (object == NULL ||
        reference == NULL ||
        reference->property_declaring_class == NULL ||
        reference->property_name == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        const PtnObjectPropertyMetadata *metadata = &object->property_metadata[i];
        if (strcmp(metadata->declaring_class, reference->property_declaring_class) == 0 &&
            strcmp(metadata->display_name, reference->property_name) == 0) {
            return metadata->storage_name;
        }
    }
    return NULL;
}

static PTN_UNUSED int ptn_reference_created_in_active_property_hook(
    PtnRuntime *runtime,
    PtnObject *receiver
) {
    return runtime != NULL &&
        receiver != NULL &&
        runtime->active_property_hook_object == receiver &&
        runtime->active_property_hook_class != NULL &&
        runtime->active_property_hook_property != NULL;
}

static PTN_UNUSED void ptn_reference_adopt_property_type_clone_source(
    PtnReference *reference,
    const PtnObjectPropertyMetadata *metadata
) {
    if (reference == NULL ||
        metadata == NULL ||
        !ptn_property_type_is_declared(metadata->type_kind)) {
        return;
    }
    if (reference->property_type_kind == PTN_PROPERTY_TYPE_NONE) {
        ptn_reference_adopt_property_type(reference, metadata);
        return;
    }
    if (reference->property_type_source_len == reference->property_type_source_cap) {
        size_t new_cap = reference->property_type_source_cap == 0
            ? 2
            : reference->property_type_source_cap * 2;
        PtnReferencePropertyTypeSource *new_sources = realloc(
            reference->property_type_sources,
            new_cap * sizeof(PtnReferencePropertyTypeSource)
        );
        if (new_sources == NULL) {
            ptn_abort_out_of_memory();
        }
        reference->property_type_sources = new_sources;
        reference->property_type_source_cap = new_cap;
    }
    reference->property_type_sources[reference->property_type_source_len++] =
        ptn_reference_property_source_from_metadata(metadata);
}

static PTN_UNUSED void ptn_throw_reference_property_bind_incompatibility(
    PtnRuntime *runtime,
    PtnValue value,
    const PtnReferencePropertyTypeSource *existing,
    const PtnObjectPropertyMetadata *metadata
) {
    char message[768];
    const char *given = ptn_property_assignment_given_name(value);
    const char *existing_type = existing->text == NULL ? "mixed" : existing->text;
    const char *new_type = metadata->type_text == NULL ? "mixed" : metadata->type_text;
    int written = snprintf(
        message,
        sizeof(message),
        "Reference with value of type %s held by property %s::$%s of type %s is not compatible with property %s::$%s of type %s",
        given,
        existing->declaring_class,
        existing->property_name,
        existing_type,
        metadata->declaring_class,
        metadata->display_name,
        new_type
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static PTN_UNUSED int ptn_magic_property_is_active_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnMagicPropertyOperation operation
);
static PTN_UNUSED size_t ptn_magic_property_push_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnMagicPropertyOperation operation
);
static PTN_UNUSED int ptn_magic_property_set_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnValue value,
    size_t line
);
static PTN_UNUSED int ptn_magic_property_unset_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    size_t line
);
static PTN_UNUSED int ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name
);

static PTN_UNUSED int ptn_magic_property_is_active(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnMagicPropertyOperation operation
) {
    return ptn_magic_property_is_active_len(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        operation
    );
}

static PTN_UNUSED int ptn_magic_property_is_active_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnMagicPropertyOperation operation
) {
    if (runtime == NULL || property == NULL) {
        return 0;
    }
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return 0;
    }
    size_t receiver_object_id = receiver.as.object->object_id;
    PtnValue effective_receiver = ptn_lazy_object_effective_initialized_proxy_receiver(receiver);
    size_t effective_object_id = effective_receiver.type == PTN_OBJECT &&
            effective_receiver.as.object != NULL
        ? effective_receiver.as.object->object_id
        : receiver_object_id;
    for (size_t i = 0; i < runtime->magic_property_frame_len; i++) {
        PtnMagicPropertyFrame *frame = &runtime->magic_property_frames[i];
        if (
            (frame->object_id == receiver_object_id ||
             frame->object_id == effective_object_id ||
             frame->effective_object_id == receiver_object_id ||
             frame->effective_object_id == effective_object_id) &&
            frame->operation == operation &&
            frame->property != NULL &&
            frame->property_len == property_len &&
            memcmp(frame->property, property, property_len) == 0
        ) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_magic_property_is_active_on_receiver_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnMagicPropertyOperation operation
) {
    if (runtime == NULL || property == NULL) {
        return 0;
    }
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return 0;
    }
    size_t receiver_object_id = receiver.as.object->object_id;
    for (size_t i = 0; i < runtime->magic_property_frame_len; i++) {
        PtnMagicPropertyFrame *frame = &runtime->magic_property_frames[i];
        if (frame->object_id == receiver_object_id &&
            frame->operation == operation &&
            frame->property != NULL &&
            frame->property_len == property_len &&
            memcmp(frame->property, property, property_len) == 0) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_magic_property_is_active_on_receiver(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnMagicPropertyOperation operation
) {
    return ptn_magic_property_is_active_on_receiver_len(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        operation
    );
}

static PTN_UNUSED size_t ptn_magic_property_push(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnMagicPropertyOperation operation
) {
    return ptn_magic_property_push_len(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        operation
    );
}

static PTN_UNUSED size_t ptn_magic_property_push_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnMagicPropertyOperation operation
) {
    size_t mark = runtime->magic_property_frame_len;
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT || property == NULL) {
        return mark;
    }
    size_t receiver_object_id = receiver.as.object->object_id;
    PtnValue effective_receiver = ptn_lazy_object_effective_initialized_proxy_receiver(receiver);
    size_t effective_object_id = effective_receiver.type == PTN_OBJECT &&
            effective_receiver.as.object != NULL
        ? effective_receiver.as.object->object_id
        : receiver_object_id;
    if (runtime->magic_property_frame_len == runtime->magic_property_frame_capacity) {
        size_t new_capacity = runtime->magic_property_frame_capacity == 0
            ? 4
            : runtime->magic_property_frame_capacity * 2;
        if (
            new_capacity < runtime->magic_property_frame_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnMagicPropertyFrame)
        ) {
            ptn_abort_out_of_memory();
        }
        PtnMagicPropertyFrame *new_frames = realloc(
            runtime->magic_property_frames,
            new_capacity * sizeof(PtnMagicPropertyFrame)
        );
        if (new_frames == NULL) {
            ptn_abort_out_of_memory();
        }
        runtime->magic_property_frames = new_frames;
        runtime->magic_property_frame_capacity = new_capacity;
    }
    PtnMagicPropertyFrame *frame =
        &runtime->magic_property_frames[runtime->magic_property_frame_len++];
    frame->object_id = receiver_object_id;
    frame->effective_object_id = effective_object_id;
    frame->property = ptn_duplicate_string_len(property, property_len);
    frame->property_len = property_len;
    frame->operation = operation;
    return mark;
}

static PTN_UNUSED void ptn_magic_property_pop(PtnRuntime *runtime, size_t mark) {
    if (runtime == NULL || mark > runtime->magic_property_frame_len) {
        return;
    }
    while (runtime->magic_property_frame_len > mark) {
        runtime->magic_property_frame_len--;
        free(runtime->magic_property_frames[runtime->magic_property_frame_len].property);
        runtime->magic_property_frames[runtime->magic_property_frame_len].property = NULL;
    }
}

static PTN_UNUSED int ptn_magic_property_get(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    PtnValue *value_out
) {
    if (runtime == NULL ||
        runtime->magic_property_get == NULL ||
        ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_GET)) {
        return 0;
    }
    return runtime->magic_property_get(runtime, receiver, property, line, value_out);
}

static PTN_UNUSED int ptn_magic_property_get_exists(PtnRuntime *runtime, PtnValue receiver) {
    (void)receiver;
    if (runtime == NULL || runtime->magic_property_get_exists == NULL) {
        return 0;
    }
    return runtime->magic_property_get_exists(runtime, receiver);
}

static PTN_UNUSED int ptn_magic_property_get_exists_inactive(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property
) {
    return ptn_magic_property_get_exists(runtime, receiver) &&
        !ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_GET);
}

static PTN_UNUSED int ptn_magic_property_get_has_active_frame(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return 0;
    }
    for (size_t i = 0; i < runtime->magic_property_frame_len; i++) {
        if (runtime->magic_property_frames[i].operation == PTN_MAGIC_PROPERTY_GET) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_magic_property_isset(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    int *isset_out
) {
    if (runtime == NULL ||
        runtime->magic_property_isset == NULL ||
        ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_ISSET)) {
        return 0;
    }
    return runtime->magic_property_isset(runtime, receiver, property, line, isset_out);
}

static PTN_UNUSED int ptn_magic_property_set(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnValue value,
    size_t line
) {
    return ptn_magic_property_set_len(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        value,
        line
    );
}

static PTN_UNUSED int ptn_magic_property_set_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    PtnValue value,
    size_t line
) {
    if (runtime == NULL ||
        runtime->magic_property_set == NULL ||
        ptn_magic_property_is_active_len(
            runtime,
            receiver,
            property,
            property_len,
            PTN_MAGIC_PROPERTY_SET
        )) {
        return 0;
    }
    return runtime->magic_property_set(runtime, receiver, property, property_len, value, line);
}

static PTN_UNUSED int ptn_magic_property_unset(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line
) {
    return ptn_magic_property_unset_len(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        line
    );
}

static PTN_UNUSED int ptn_magic_property_unset_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    size_t line
) {
    if (runtime == NULL ||
        runtime->magic_property_unset == NULL ||
        ptn_magic_property_is_active_len(
            runtime,
            receiver,
            property,
            property_len,
            PTN_MAGIC_PROPERTY_UNSET
        )) {
        return 0;
    }
    return runtime->magic_property_unset(runtime, receiver, property, property_len, line);
}

static PTN_UNUSED void ptn_throw_overloaded_property_reference_error(
    PtnRuntime *runtime,
    size_t line
) {
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Cannot assign by reference to overloaded object",
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
}

static PTN_UNUSED void ptn_emit_indirect_modification_overloaded_property_notice(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line
) {
    if (runtime == NULL) {
        return;
    }
    PtnValue resolved = ptn_value_deref(receiver);
    const char *class_name = resolved.type == PTN_OBJECT
        ? resolved.as.object->class_name
        : "stdClass";
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Indirect modification of overloaded property %s::$%s has no effect",
        class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    PtnRuntime *root = ptn_runtime_root(runtime);
    ptn_emit_notice_with_path(
        &runtime->diagnostics,
        message,
        runtime->source_path,
        line,
        root != NULL && root->output_has_started
    );
}

static PTN_UNUSED void ptn_call_magic_get_then_throw_overloaded_property_reference_error(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    int emit_indirect_notice
) {
    PtnValue magic_value = ptn_null();
    if (ptn_magic_property_get(runtime, receiver, property, line, &magic_value)) {
        PtnValue magic_resolved = ptn_value_deref(magic_value);
        if (
            emit_indirect_notice &&
            magic_value.type != PTN_REFERENCE &&
            magic_resolved.type != PTN_OBJECT &&
            magic_resolved.type != PTN_EXCEPTION
        ) {
            ptn_emit_indirect_modification_overloaded_property_notice(
                runtime,
                receiver,
                property,
                line
            );
        }
        ptn_value_destroy(&magic_value);
    }
    ptn_throw_overloaded_property_reference_error(runtime, line);
}

static PTN_UNUSED int ptn_object_reject_overloaded_property_reference_assignment(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line,
    int emit_indirect_notice
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return 0;
    }
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL &&
        ptn_magic_property_get_exists_inactive(runtime, receiver, property)) {
        ptn_call_magic_get_then_throw_overloaded_property_reference_error(
            runtime,
            receiver,
            property,
            line,
            emit_indirect_notice
        );
        return 1;
    }
    if (blocked_metadata == NULL &&
        ptn_object_metadata_for_display_name(receiver.as.object, property) == NULL &&
        ptn_magic_property_get_exists_inactive(runtime, receiver, property)) {
        ptn_call_magic_get_then_throw_overloaded_property_reference_error(
            runtime,
            receiver,
            property,
            line,
            emit_indirect_notice
        );
        return 1;
    }
    return 0;
}

static int ptn_reflection_internal_readonly_property_declaring_class(const char *declaring_class) {
    return ptn_ascii_case_equal(declaring_class, "ReflectionAttribute")
        || ptn_ascii_case_equal(declaring_class, "ReflectionClass")
        || ptn_ascii_case_equal(declaring_class, "ReflectionClassConstant")
        || ptn_ascii_case_equal(declaring_class, "ReflectionConstant")
        || ptn_ascii_case_equal(declaring_class, "ReflectionExtension")
        || ptn_ascii_case_equal(declaring_class, "ReflectionZendExtension")
        || ptn_ascii_case_equal(declaring_class, "ReflectionFunction")
        || ptn_ascii_case_equal(declaring_class, "ReflectionFunctionAbstract")
        || ptn_ascii_case_equal(declaring_class, "ReflectionMethod")
        || ptn_ascii_case_equal(declaring_class, "ReflectionObject")
        || ptn_ascii_case_equal(declaring_class, "ReflectionParameter")
        || ptn_ascii_case_equal(declaring_class, "ReflectionProperty")
        || ptn_ascii_case_equal(declaring_class, "ReflectionReference")
        || ptn_ascii_case_equal(declaring_class, "ReflectionType")
        || ptn_ascii_case_equal(declaring_class, "ReflectionNamedType")
        || ptn_ascii_case_equal(declaring_class, "ReflectionUnionType")
        || ptn_ascii_case_equal(declaring_class, "ReflectionIntersectionType");
}

static PTN_UNUSED void ptn_throw_readonly_property_error(
    PtnRuntime *runtime,
    const char *object_class_name,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    if (ptn_reflection_internal_readonly_property_declaring_class(declaring_class)) {
        const char *display_class = object_class_name == NULL ? declaring_class : object_class_name;
        int written = snprintf(
            message,
            sizeof(message),
            "Cannot set read-only property %s::$%s",
            display_class,
            property
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception_at(runtime, "ReflectionException", message, runtime->source_path, line);
        return;
    }
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot modify readonly property %s::$%s",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_readonly_property_unset_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot unset readonly property %s::$%s",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_date_period_internal_property_name(const char *property) {
    return property != NULL &&
        (strcmp(property, "start") == 0 ||
         strcmp(property, "current") == 0 ||
         strcmp(property, "end") == 0 ||
         strcmp(property, "interval") == 0 ||
         strcmp(property, "recurrences") == 0 ||
         strcmp(property, "include_start_date") == 0 ||
         strcmp(property, "include_end_date") == 0);
}

static PTN_UNUSED int ptn_object_property_is_date_period_internal(
    PtnRuntime *runtime,
    PtnObject *object,
    const PtnObjectPropertyMetadata *metadata
) {
    return object != NULL &&
        metadata != NULL &&
        metadata->is_readonly &&
        ptn_ascii_case_equal(metadata->declaring_class, "DatePeriod") &&
        ptn_date_period_internal_property_name(metadata->display_name) &&
        ptn_runtime_declared_class_is_same_or_descendant(
            runtime,
            object->class_name,
            "DatePeriod"
        );
}

static PTN_UNUSED void ptn_throw_date_period_internal_property_unset_error(
    PtnRuntime *runtime,
    const char *object_class_name,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot unset %s::$%s",
        object_class_name == NULL ? "DatePeriod" : object_class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_readonly_property_reference_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot acquire reference to readonly property %s::$%s",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_readonly_property_indirect_modification_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot indirectly modify readonly property %s::$%s",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_readonly_property_initialize_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    const char *access_scope
) {
    char message[320];
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot initialize readonly property %s::$%s from global scope",
            declaring_class,
            property
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot initialize readonly property %s::$%s from scope %s",
            declaring_class,
            property,
            access_scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_uninitialized_typed_property_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Typed property %s::$%s must not be accessed before initialization",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_uninitialized_typed_property_reference_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot access uninitialized non-nullable property %s::$%s by reference",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_dynamic_property_readonly_class_error(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot create dynamic property %s::$%s",
        class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_object_class_allows_dynamic_properties(
    PtnRuntime *runtime,
    const char *class_name
) {
    if (class_name == NULL) {
        return 0;
    }
    if (ptn_class_name_is_stdclass(class_name)) {
        return 1;
    }
    if (ptn_ascii_case_equal(class_name, "__PHP_Incomplete_Class")) {
        return 1;
    }
    if (ptn_runtime_dynamic_class_allows_dynamic_properties(runtime, class_name)) {
        return 1;
    }
    return runtime != NULL &&
        runtime->declared_class_allows_dynamic_properties != NULL &&
        runtime->declared_class_allows_dynamic_properties(class_name);
}

static PTN_UNUSED int ptn_object_is_incomplete_class(PtnObject *object) {
    return object != NULL &&
        object->class_name != NULL &&
        ptn_ascii_case_equal(object->class_name, "__PHP_Incomplete_Class");
}

static PTN_UNUSED char *ptn_incomplete_object_original_class_name(PtnObject *object) {
    if (object != NULL && object->properties != NULL) {
        PtnArrayKey key = ptn_array_string_key("__PHP_Incomplete_Class_Name");
        PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
        ptn_array_key_free(key);
        if (entry != NULL) {
            PtnValue class_name = ptn_value_deref(entry->value);
            if (class_name.type == PTN_STRING) {
                return ptn_duplicate_string_len(
                    (const char *)class_name.as.string.data,
                    class_name.as.string.len
                );
            }
        }
    }
    return ptn_duplicate_string("unknown");
}

static PTN_UNUSED void ptn_throw_incomplete_object_property_modification(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
) {
    char *class_name = ptn_incomplete_object_original_class_name(object);
    int needed = snprintf(
        NULL,
        0,
        "The script tried to modify a property on an incomplete object. Please ensure that the class definition \"%s\" of the object you are trying to operate on was loaded _before_ unserialize() gets called or provide an autoloader to load the class definition",
        class_name
    );
    if (needed < 0) {
        free(class_name);
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        free(class_name);
        ptn_abort_out_of_memory();
    }
    int written = snprintf(
        message,
        (size_t)needed + 1,
        "The script tried to modify a property on an incomplete object. Please ensure that the class definition \"%s\" of the object you are trying to operate on was loaded _before_ unserialize() gets called or provide an autoloader to load the class definition",
        class_name
    );
    free(class_name);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_owned_message_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_incomplete_object_method_call(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
) {
    char *class_name = ptn_incomplete_object_original_class_name(object);
    int needed = snprintf(
        NULL,
        0,
        "The script tried to call a method on an incomplete object. Please ensure that the class definition \"%s\" of the object you are trying to operate on was loaded _before_ unserialize() gets called or provide an autoloader to load the class definition",
        class_name
    );
    if (needed < 0) {
        free(class_name);
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        free(class_name);
        ptn_abort_out_of_memory();
    }
    int written = snprintf(
        message,
        (size_t)needed + 1,
        "The script tried to call a method on an incomplete object. Please ensure that the class definition \"%s\" of the object you are trying to operate on was loaded _before_ unserialize() gets called or provide an autoloader to load the class definition",
        class_name
    );
    free(class_name);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_owned_message_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_emit_incomplete_object_property_access_warning(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
) {
    char *class_name = ptn_incomplete_object_original_class_name(object);
    int needed = snprintf(
        NULL,
        0,
        "main(): The script tried to access a property on an incomplete object. Please ensure that the class definition \"%s\" of the object you are trying to operate on was loaded _before_ unserialize() gets called or provide an autoloader to load the class definition",
        class_name
    );
    if (needed < 0) {
        free(class_name);
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        free(class_name);
        ptn_abort_out_of_memory();
    }
    int written = snprintf(
        message,
        (size_t)needed + 1,
        "main(): The script tried to access a property on an incomplete object. Please ensure that the class definition \"%s\" of the object you are trying to operate on was loaded _before_ unserialize() gets called or provide an autoloader to load the class definition",
        class_name
    );
    free(class_name);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
    free(message);
}

static PTN_UNUSED void ptn_emit_dynamic_property_deprecation(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    size_t line
) {
    if (runtime == NULL ||
        object == NULL ||
        ptn_object_class_allows_dynamic_properties(runtime, object->class_name) ||
        !ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Creation of dynamic property %s::$%s is deprecated",
        object->class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_runtime_deprecation(runtime, message, line);
}

static PTN_UNUSED void ptn_runtime_clear_dynamic_property_deprecation_suppression(
    PtnRuntime *runtime
) {
    if (runtime == NULL) {
        return;
    }
    free(runtime->dynamic_property_deprecation_suppress_property);
    runtime->dynamic_property_deprecation_suppress_property = NULL;
    runtime->dynamic_property_deprecation_suppress_object = NULL;
}

static PTN_UNUSED void ptn_runtime_suppress_next_dynamic_property_deprecation(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property
) {
    if (runtime == NULL || object == NULL || property == NULL) {
        return;
    }
    ptn_runtime_clear_dynamic_property_deprecation_suppression(runtime);
    runtime->dynamic_property_deprecation_suppress_object = object;
    runtime->dynamic_property_deprecation_suppress_property = ptn_duplicate_string(property);
}

static PTN_UNUSED int ptn_runtime_consume_dynamic_property_deprecation_suppression(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property
) {
    if (runtime == NULL ||
        object == NULL ||
        property == NULL ||
        runtime->dynamic_property_deprecation_suppress_object != object ||
        runtime->dynamic_property_deprecation_suppress_property == NULL ||
        strcmp(runtime->dynamic_property_deprecation_suppress_property, property) != 0) {
        return 0;
    }
    ptn_runtime_clear_dynamic_property_deprecation_suppression(runtime);
    return 1;
}

#define PTN_PROPERTY_ACCESS_READ 0
#define PTN_PROPERTY_ACCESS_WRITE 1
#define PTN_PROPERTY_ACCESS_INDIRECT_WRITE 2
#define PTN_PROPERTY_ACCESS_UNSET 3

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED int ptn_internal_array_object_property_read(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line,
    PtnValue *value_out
);
static PTN_UNUSED int ptn_internal_array_object_property_write(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line,
    PtnValue *value_out
);
static PTN_UNUSED int ptn_internal_array_object_property_isset(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line,
    int *isset_out
);
static PTN_UNUSED int ptn_internal_array_object_property_unset(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
);
static PTN_UNUSED int ptn_internal_xml_property_read(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    PtnValue *value_out
);
static PTN_UNUSED int ptn_internal_date_interval_property_read(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    PtnValue *value_out
);
static PTN_UNUSED int ptn_internal_xml_property_write(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnValue value,
    size_t line,
    PtnValue *value_out
);
static PTN_UNUSED int ptn_internal_xml_property_write_indirect(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnValue value,
    size_t line,
    PtnValue *value_out
);
static PTN_UNUSED int ptn_internal_array_object_offset_reference(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnValue *offset_value,
    size_t line,
    int create_if_missing,
    PtnValue *reference_out
);
static PTN_UNUSED int ptn_internal_array_object_offset_reference_without_key_diagnostics(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnValue *offset_value,
    size_t line,
    int create_if_missing,
    PtnValue *reference_out
);
static PTN_UNUSED int ptn_internal_array_object_bind_offset_reference(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnValue *offset_value,
    PtnValue reference,
    size_t line
);
static PTN_UNUSED int ptn_internal_array_object_offset_reference_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnValue *offset_value,
    size_t line,
    PtnValue *reference_out
);
static PTN_UNUSED int ptn_internal_array_object_offset_lookup_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnValue *offset_value,
    size_t line,
    PtnLookupResult *result_out
);
static PTN_UNUSED int ptn_internal_array_object_offset_isset_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnValue *offset_value,
    size_t line,
    int *isset_out
);
static PTN_UNUSED int ptn_internal_array_object_offset_lookup_for_assign_op(
    PtnRuntime *runtime,
    PtnValue receiver,
    const PtnValue *offset_value,
    size_t line,
    PtnLookupResult *result_out
);
static PTN_UNUSED int ptn_internal_array_object_uses_builtin_offsets(
    PtnRuntime *runtime,
    PtnValue receiver
);
#endif

static PTN_UNUSED int ptn_object_static_property_visibility(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int access_mode,
    PtnPropertyVisibility *visibility_out,
    const char **declaring_class_out
) {
    (void)access_scope;
    if (runtime == NULL || object == NULL || property == NULL) {
        return 0;
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        object->class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        return 0;
    }
    PtnValue visibility_value;
    PtnPropertyVisibility read_visibility = PTN_PROPERTY_PUBLIC;
    PtnPropertyVisibility set_visibility = PTN_PROPERTY_PUBLIC;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_read_visibility_table(runtime),
            key,
            &visibility_value
        ) &&
        ptn_value_deref(visibility_value).type == PTN_INT
    ) {
        read_visibility = (PtnPropertyVisibility)ptn_value_deref(visibility_value).as.integer;
    }
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_set_visibility_table(runtime),
            key,
            &visibility_value
        ) &&
        ptn_value_deref(visibility_value).type == PTN_INT
    ) {
        set_visibility = (PtnPropertyVisibility)ptn_value_deref(visibility_value).as.integer;
    }
    free(key);
    PtnPropertyVisibility visibility = access_mode == PTN_PROPERTY_ACCESS_READ
        ? read_visibility
        : set_visibility;
    if (visibility_out != NULL) {
        *visibility_out = visibility;
    }
    if (declaring_class_out != NULL) {
        *declaring_class_out = declaring_class == NULL ? object->class_name : declaring_class;
    }
    return 1;
}

static PTN_UNUSED int ptn_object_static_property_inaccessible(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int access_mode,
    PtnPropertyVisibility *visibility_out,
    const char **declaring_class_out
) {
    PtnPropertyVisibility visibility = PTN_PROPERTY_PUBLIC;
    const char *declaring_class = NULL;
    if (!ptn_object_static_property_visibility(
        runtime,
        object,
        property,
        access_scope,
        access_mode,
        &visibility,
        &declaring_class
    )) {
        return 0;
    }
    if (ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
        return 0;
    }
    if (visibility_out != NULL) {
        *visibility_out = visibility;
    }
    if (declaring_class_out != NULL) {
        *declaring_class_out = declaring_class;
    }
    return 1;
}

static PTN_UNUSED int ptn_object_static_property_accessible(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int access_mode,
    const char **declaring_class_out
) {
    PtnPropertyVisibility visibility = PTN_PROPERTY_PUBLIC;
    const char *declaring_class = NULL;
    if (!ptn_object_static_property_visibility(
        runtime,
        object,
        property,
        access_scope,
        access_mode,
        &visibility,
        &declaring_class
    )) {
        return 0;
    }
    if (!ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
        return 0;
    }
    if (declaring_class_out != NULL) {
        *declaring_class_out = declaring_class;
    }
    return 1;
}

static PTN_UNUSED int ptn_object_instance_property_accessible(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int access_mode
) {
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_private_property_for_scope(object, property, access_scope);
    if (metadata == NULL) {
        metadata = ptn_object_named_shared_property(object, property);
    }
    if (metadata == NULL) {
        return 0;
    }
    PtnPropertyVisibility visibility = access_mode == PTN_PROPERTY_ACCESS_READ
        ? metadata->read_visibility
        : metadata->set_visibility;
    return ptn_property_visibility_allows(
        runtime,
        visibility,
        ptn_property_visibility_scope_class(metadata, visibility),
        access_scope
    );
}

static PTN_UNUSED void ptn_emit_static_property_non_static_notice(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    if (runtime == NULL ||
        !ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Accessing static property ", stdout);
    fputs(declaring_class, stdout);
    fputs("::$", stdout);
    fputs(property, stdout);
    fputs(" as non static in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_static_property_non_static_notice_if_accessible(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int access_mode,
    size_t line
) {
    if (ptn_object_instance_property_accessible(
        runtime,
        object,
        property,
        access_scope,
        access_mode
    )) {
        return;
    }
    const char *declaring_class = NULL;
    if (ptn_object_static_property_accessible(
        runtime,
        object,
        property,
        access_scope,
        access_mode,
        &declaring_class
    )) {
        ptn_emit_static_property_non_static_notice(
            runtime,
            declaring_class,
            property,
            line
        );
    }
}

static PTN_UNUSED char *ptn_object_resolve_property_storage_key(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int access_mode,
    int quiet,
    size_t line
) {
    int for_write = access_mode != PTN_PROPERTY_ACCESS_READ;
    int indirect_write = access_mode == PTN_PROPERTY_ACCESS_INDIRECT_WRITE;
    int unset_write = access_mode == PTN_PROPERTY_ACCESS_UNSET;
    const PtnObjectPropertyMetadata *scoped_private =
        ptn_object_private_property_for_scope(object, property, access_scope);
    if (scoped_private != NULL) {
        PtnPropertyVisibility visibility = (for_write || unset_write)
            ? scoped_private->set_visibility
            : scoped_private->read_visibility;
        if ((access_mode == PTN_PROPERTY_ACCESS_WRITE ||
                access_mode == PTN_PROPERTY_ACCESS_INDIRECT_WRITE) &&
            scoped_private->is_readonly &&
            scoped_private->set_visibility == PTN_PROPERTY_PROTECTED &&
            ptn_object_property_storage_initialized(object, scoped_private->storage_name)) {
            return ptn_duplicate_string(scoped_private->storage_name);
        }
        if (unset_write &&
            ptn_readonly_property_storage_initialized(object, scoped_private)) {
            return ptn_duplicate_string(scoped_private->storage_name);
        }
        if (!ptn_property_visibility_allows(
            runtime,
            visibility,
            ptn_property_visibility_scope_class(scoped_private, visibility),
            access_scope
        )) {
            if (quiet) {
                return NULL;
            }
            if (for_write && scoped_private->set_visibility != scoped_private->read_visibility) {
                if (indirect_write && scoped_private->is_readonly) {
                    ptn_throw_readonly_property_indirect_modification_error(
                        runtime,
                        scoped_private->declaring_class,
                        property,
                        line
                    );
                } else if (indirect_write) {
                    ptn_throw_property_indirect_set_visibility_error(
                        runtime,
                        scoped_private->set_visibility,
                        scoped_private->declaring_class,
                        property,
                        access_scope
                    );
                } else if (unset_write) {
                    ptn_throw_property_unset_visibility_error(
                        runtime,
                        scoped_private->set_visibility,
                        scoped_private->declaring_class,
                        property,
                        access_scope,
                        1,
                        scoped_private->is_readonly
                    );
                } else {
                    if (scoped_private->is_readonly &&
                        scoped_private->set_visibility == PTN_PROPERTY_PROTECTED) {
                        ptn_throw_readonly_property_set_visibility_error(
                            runtime,
                            scoped_private->set_visibility,
                            scoped_private->declaring_class,
                            property,
                            access_scope
                        );
                    } else {
                        ptn_throw_property_set_visibility_error(
                            runtime,
                            scoped_private->set_visibility,
                            scoped_private->declaring_class,
                            property,
                            access_scope
                        );
                    }
                }
            } else if (for_write && scoped_private->is_readonly) {
                ptn_throw_readonly_property_initialize_error(
                    runtime,
                    scoped_private->declaring_class,
                    property,
                    access_scope
                );
            } else if (unset_write && scoped_private->set_visibility != scoped_private->read_visibility) {
                ptn_throw_property_unset_visibility_error(
                    runtime,
                    scoped_private->set_visibility,
                    scoped_private->declaring_class,
                    property,
                    access_scope,
                    1,
                    scoped_private->is_readonly
                );
            } else {
                ptn_throw_property_visibility_error(
                    runtime,
                    visibility,
                    scoped_private->declaring_class,
                    property,
                    line
                );
            }
            return NULL;
        }
        return ptn_duplicate_string(scoped_private->storage_name);
    }
    const PtnObjectPropertyMetadata *shared_property =
        ptn_object_named_shared_property(object, property);
    if (shared_property != NULL) {
        PtnPropertyVisibility visibility = (for_write || unset_write)
            ? shared_property->set_visibility
            : shared_property->read_visibility;
        if ((access_mode == PTN_PROPERTY_ACCESS_WRITE ||
                access_mode == PTN_PROPERTY_ACCESS_INDIRECT_WRITE) &&
            shared_property->is_readonly &&
            shared_property->set_visibility == PTN_PROPERTY_PROTECTED &&
            ptn_object_property_storage_initialized(object, shared_property->storage_name)) {
            return ptn_duplicate_string(shared_property->storage_name);
        }
        if (unset_write &&
            ptn_readonly_property_storage_initialized(object, shared_property)) {
            return ptn_duplicate_string(shared_property->storage_name);
        }
        if (!ptn_property_visibility_allows(
            runtime,
            visibility,
            ptn_property_visibility_scope_class(shared_property, visibility),
            access_scope
        )) {
            if (quiet) {
                return NULL;
            }
            if (for_write && shared_property->set_visibility != shared_property->read_visibility) {
                if (indirect_write && shared_property->is_readonly) {
                    ptn_throw_readonly_property_indirect_modification_error(
                        runtime,
                        shared_property->declaring_class,
                        property,
                        line
                    );
                } else if (indirect_write) {
                    ptn_throw_property_indirect_set_visibility_error(
                        runtime,
                        shared_property->set_visibility,
                        shared_property->declaring_class,
                        property,
                        access_scope
                    );
                } else if (unset_write) {
                    ptn_throw_property_unset_visibility_error(
                        runtime,
                        shared_property->set_visibility,
                        shared_property->declaring_class,
                        property,
                        access_scope,
                        1,
                        shared_property->is_readonly
                    );
                } else {
                    if (shared_property->is_readonly &&
                        shared_property->set_visibility == PTN_PROPERTY_PROTECTED) {
                        ptn_throw_readonly_property_set_visibility_error(
                            runtime,
                            shared_property->set_visibility,
                            shared_property->declaring_class,
                            property,
                            access_scope
                        );
                    } else {
                        ptn_throw_property_set_visibility_error(
                            runtime,
                            shared_property->set_visibility,
                            shared_property->declaring_class,
                            property,
                            access_scope
                        );
                    }
                }
            } else if (for_write && shared_property->is_readonly) {
                ptn_throw_readonly_property_initialize_error(
                    runtime,
                    shared_property->declaring_class,
                    property,
                    access_scope
                );
            } else if (unset_write && shared_property->set_visibility != shared_property->read_visibility) {
                ptn_throw_property_unset_visibility_error(
                    runtime,
                    shared_property->set_visibility,
                    shared_property->declaring_class,
                    property,
                    access_scope,
                    1,
                    shared_property->is_readonly
                );
            } else {
                ptn_throw_property_visibility_error(
                    runtime,
                    visibility,
                    visibility == PTN_PROPERTY_PROTECTED
                        ? object->class_name
                        : shared_property->declaring_class,
                    property,
                    line
                );
            }
            return NULL;
        }
        return ptn_duplicate_string(shared_property->storage_name);
    }
    const PtnObjectPropertyMetadata *own_private =
        ptn_object_own_private_property(object, property);
    if (own_private == NULL) {
        if (access_mode == PTN_PROPERTY_ACCESS_READ &&
            ptn_object_static_property_visibility(
                runtime,
                object,
                property,
                access_scope,
                access_mode,
                NULL,
                NULL
            )) {
            PtnArrayKey dynamic_key = ptn_array_string_key(property);
            PtnArrayEntry *dynamic_entry =
                ptn_array_entry_for_key(object->properties, dynamic_key);
            ptn_array_key_free(dynamic_key);
            if (dynamic_entry != NULL || !quiet) {
                return ptn_duplicate_string(property);
            }
            return NULL;
        }
        PtnPropertyVisibility static_visibility = PTN_PROPERTY_PUBLIC;
        const char *static_declaring_class = NULL;
        if (ptn_object_static_property_inaccessible(
            runtime,
            object,
            property,
            access_scope,
            access_mode,
            &static_visibility,
            &static_declaring_class
        )) {
            if (!quiet) {
                ptn_throw_property_visibility_error(
                    runtime,
                    static_visibility,
                    static_declaring_class,
                    property,
                    line
                );
            }
            return NULL;
        }
        if (
            (access_mode == PTN_PROPERTY_ACCESS_WRITE ||
             access_mode == PTN_PROPERTY_ACCESS_INDIRECT_WRITE) &&
            (object->enum_case_name != NULL ||
             ptn_object_is_generator(object) ||
             ptn_ascii_case_equal(object->class_name, "BcMath\\Number") ||
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
             ptn_internal_class_name_is_sensitive_parameter_value(object->class_name) ||
             ptn_internal_class_name_is_weak_map(object->class_name) ||
             ptn_internal_class_name_is_weak_reference(object->class_name) ||
#endif
             0)
        ) {
            if (!quiet) {
                ptn_throw_dynamic_property_readonly_class_error(
                    runtime,
                    object->class_name,
                    property,
                    line
                );
            }
            return NULL;
        }
        if (
            for_write &&
            runtime != NULL &&
            runtime->declared_class_is_readonly != NULL &&
            runtime->declared_class_is_readonly(object->class_name)
        ) {
            if (!quiet) {
                ptn_throw_dynamic_property_readonly_class_error(
                    runtime,
                    object->class_name,
                    property,
                    line
                );
            }
            return NULL;
        }
        return ptn_duplicate_string(property);
    }
    if (quiet) {
        return NULL;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot access private property %s::$%s",
        own_private->declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
    return NULL;
}

static PTN_UNUSED int ptn_object_indirect_write_targets_overloaded_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT || !ptn_magic_property_get_exists(runtime, receiver)) {
        return 0;
    }
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL) {
        return 1;
    }
    if (ptn_object_metadata_for_display_name(receiver.as.object, property) != NULL) {
        return 0;
    }

    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
        1,
        line
    );
    if (storage_key == NULL) {
        return 1;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    return entry == NULL;
}

static PTN_UNUSED int ptn_object_indirect_write_should_emit_overloaded_property_notice(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return value.type != PTN_REFERENCE &&
        ptn_object_indirect_write_targets_overloaded_property(
            runtime,
            receiver,
            property,
            access_scope,
            line
        );
}

static PTN_UNUSED int ptn_object_emit_indirect_modification_overloaded_property_notice_for_value(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    if (!ptn_object_indirect_write_should_emit_overloaded_property_notice(
            runtime,
            receiver,
            property,
            access_scope,
            value,
            line
        )) {
        return 0;
    }
    ptn_emit_indirect_modification_overloaded_property_notice(
        runtime,
        receiver,
        property,
        line
    );
    return 1;
}

static PTN_UNUSED int ptn_object_missing_dynamic_property_for_creation(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    int access_mode,
    size_t line,
    PtnObject **object_out
) {
    receiver = ptn_value_deref(receiver);
    if (property == NULL ||
        receiver.type != PTN_OBJECT ||
        receiver.as.object == NULL ||
        ptn_object_is_incomplete_class(receiver.as.object) ||
        receiver.as.object->lazy_uninitialized ||
        ptn_object_metadata_for_display_name(receiver.as.object, property) != NULL) {
        return 0;
    }
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL) {
        return 0;
    }
    if (runtime != NULL &&
        runtime->magic_property_set != NULL &&
        !ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_SET)) {
        return 0;
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        access_mode,
        1,
        line
    );
    if (storage_key == NULL) {
        return 0;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    ptn_array_key_free(key);
    free(storage_key);
    if (metadata != NULL || entry != NULL) {
        return 0;
    }
    if (object_out != NULL) {
        *object_out = receiver.as.object;
    }
    return 1;
}

static PTN_UNUSED int ptn_object_emit_dynamic_property_creation_deprecation(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    size_t line,
    int suppress_following_write
) {
    if (object == NULL || property == NULL) {
        return 1;
    }
    size_t refcount_before = object->refcount;
    ptn_object_retain(object);
    ptn_emit_dynamic_property_deprecation(runtime, object, property, line);
    int receiver_invalidated = object->refcount <= refcount_before;
    int active_exception = runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL;
    if (receiver_invalidated && !active_exception) {
        ptn_throw_dynamic_property_readonly_class_error(
            runtime,
            object->class_name,
            property,
            line
        );
        active_exception = 1;
    }
    if (!active_exception && !receiver_invalidated && suppress_following_write) {
        ptn_runtime_suppress_next_dynamic_property_deprecation(runtime, object, property);
    }
    ptn_object_release(object);
    return !active_exception && !receiver_invalidated;
}

static PTN_UNUSED int ptn_object_preflight_dynamic_property_creation(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    int access_mode,
    size_t line
) {
    PtnObject *object = NULL;
    if (!ptn_object_missing_dynamic_property_for_creation(
            runtime,
            receiver,
            property,
            access_scope,
            access_mode,
            line,
            &object
        )) {
        return 1;
    }
    return ptn_object_emit_dynamic_property_creation_deprecation(
        runtime,
        object,
        property,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_undefined_exception_property_warning(
    PtnRuntime *runtime,
    PtnException *exception,
    const char *property,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Undefined property: %s::$%s",
        exception->class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    runtime->diagnostics.emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(
        &runtime->diagnostics,
        PTN_E_WARNING,
        message,
        runtime->source_path,
        line
    )) {
        return;
    }
    ptn_diagnostic_printf(
        &runtime->diagnostics,
        "\nWarning: %s in %s on line %zu\n",
        message,
        runtime->source_path != NULL ? runtime->source_path : "ptn",
        line
    );
}

static PTN_UNUSED void ptn_emit_dynamic_exception_property_deprecation(
    PtnRuntime *runtime,
    PtnException *exception,
    const char *property,
    size_t line
) {
    if (runtime == NULL ||
        exception == NULL ||
        ptn_object_class_allows_dynamic_properties(runtime, exception->class_name) ||
        !ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Creation of dynamic property %s::$%s is deprecated",
        exception->class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_runtime_deprecation(runtime, message, line);
}

static PTN_UNUSED int ptn_exception_property_read(
    PtnValue receiver,
    const char *property,
    PtnValue *out
) {
    receiver = ptn_value_deref(receiver);
    if (
        receiver.type == PTN_EXCEPTION &&
        ptn_ascii_case_equal(receiver.as.exception->class_name, "Uri\\WhatWg\\InvalidUrlException") &&
        ptn_ascii_case_equal(property, "errors")
    ) {
        *out = ptn_value_clone_deref(receiver.as.exception->errors);
        return 1;
    }
    if (
        receiver.type == PTN_EXCEPTION &&
        ptn_exception_is_soap_fault_class(receiver.as.exception->class_name) &&
        ptn_ascii_case_equal(property, "headerfault")
    ) {
        *out = ptn_value_clone_deref(receiver.as.exception->soap_fault_headerfault);
        return 1;
    }
    if (
        receiver.type == PTN_EXCEPTION &&
        receiver.as.exception->dynamic_properties.type == PTN_ARRAY
    ) {
        PtnArrayKey key = ptn_array_string_key(property);
        PtnArrayEntry *entry = ptn_array_entry_for_key(
            receiver.as.exception->dynamic_properties.as.array,
            key
        );
        ptn_array_key_free(key);
        if (entry != NULL) {
            *out = ptn_value_clone_deref(entry->value);
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_exception_write_dynamic_property(
    PtnRuntime *runtime,
    PtnException *exception,
    const char *property,
    PtnValue value,
    size_t line
) {
    if (exception == NULL || exception->dynamic_properties.type != PTN_ARRAY) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(property);
    PtnArrayEntry *entry = ptn_array_entry_for_key(
        exception->dynamic_properties.as.array,
        key
    );
    if (entry == NULL) {
        ptn_emit_dynamic_exception_property_deprecation(runtime, exception, property, line);
    }
    PtnValue stored = ptn_value_clone_deref(value);
    PtnValue result = ptn_value_clone(stored);
    ptn_array_set_entry_publish_first(exception->dynamic_properties.as.array, key, stored);
    return result;
}

static PTN_UNUSED int ptn_uninitialized_lazy_object_declares_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name
);

static PTN_UNUSED int ptn_lazy_object_property_reference_needs_initialization(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT ||
        !receiver.as.object->lazy_uninitialized ||
        receiver.as.object->lazy_initializing) {
        return 0;
    }
    if (ptn_uninitialized_lazy_object_declares_method(runtime, receiver, "__get") &&
        !ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_GET)) {
        return 0;
    }

    int local_lazy_slot = 0;
    char *lazy_storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
        1,
        line
    );
    if (lazy_storage_key != NULL) {
        PtnArrayKey lazy_key = ptn_array_string_key(lazy_storage_key);
        PtnArrayEntry *lazy_entry =
            ptn_array_entry_for_key(receiver.as.object->properties, lazy_key);
        const PtnObjectPropertyMetadata *lazy_metadata =
            ptn_object_property_metadata(receiver.as.object, lazy_storage_key);
        ptn_array_key_free(lazy_key);
        local_lazy_slot = lazy_metadata != NULL &&
            lazy_metadata->lazy_skip &&
            (lazy_entry != NULL ||
             (lazy_metadata->is_unset &&
              ptn_property_type_is_declared(lazy_metadata->type_kind)));
        free(lazy_storage_key);
    }

    if (!local_lazy_slot &&
        ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
            runtime,
            receiver,
            "__get"
        ) &&
        !ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_GET)) {
        return 0;
    }

    return !local_lazy_slot;
}

static PTN_UNUSED int ptn_lazy_object_property_access_uses_local_slot(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    int access_mode,
    size_t line,
    const PtnObjectPropertyMetadata **metadata_out
) {
    receiver = ptn_value_deref(receiver);
    if (metadata_out != NULL) {
        *metadata_out = NULL;
    }
    if (receiver.type != PTN_OBJECT ||
        !receiver.as.object->lazy_uninitialized ||
        receiver.as.object->lazy_initializing) {
        return 0;
    }

    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        access_mode,
        1,
        line
    );
    if (storage_key != NULL) {
        PtnArrayKey key = ptn_array_string_key(storage_key);
        PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(receiver.as.object, storage_key);
        ptn_array_key_free(key);
        free(storage_key);
        if (metadata_out != NULL) {
            *metadata_out = metadata;
        }
        if (access_mode == PTN_PROPERTY_ACCESS_UNSET && metadata != NULL && metadata->has_hooks) {
            return 1;
        }
        return metadata != NULL &&
            metadata->lazy_skip &&
            (entry != NULL ||
             metadata->is_unset ||
             ptn_property_type_is_declared(metadata->type_kind));
    }

    for (size_t i = 0; i < receiver.as.object->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &receiver.as.object->property_metadata[i];
        if (strcmp(metadata->display_name, property) == 0 && metadata->lazy_skip) {
            if (metadata_out != NULL) {
                *metadata_out = metadata;
            }
            return 1;
        }
    }

    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 0);
    if (metadata_out != NULL) {
        *metadata_out = blocked_metadata;
    }
    return blocked_metadata != NULL && blocked_metadata->lazy_skip;
}

static PTN_UNUSED int ptn_uninitialized_lazy_object_declares_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name
) {
    receiver = ptn_value_deref(receiver);
    return runtime != NULL &&
        runtime->declared_method_exists != NULL &&
        receiver.type == PTN_OBJECT &&
        receiver.as.object != NULL &&
        receiver.as.object->lazy_uninitialized &&
        !receiver.as.object->lazy_initializing &&
        runtime->declared_method_exists(receiver.as.object->class_name, method_name);
}

static PTN_UNUSED int ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name
) {
    receiver = ptn_value_deref(receiver);
    if (!ptn_uninitialized_lazy_object_declares_method(runtime, receiver, method_name)) {
        return 0;
    }
    if (!receiver.as.object->lazy_is_proxy) {
        return 1;
    }
    const char *class_name = receiver.as.object->class_name;
    const char *parent_name = ptn_declared_class_parent_name(class_name);
    if (parent_name == NULL) {
        return 1;
    }
    return !ptn_declared_class_direct_non_private_method_exists(class_name, method_name);
}

static PTN_UNUSED int ptn_initialized_lazy_proxy_should_forward_property_read(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT ||
        receiver.as.object == NULL ||
        !receiver.as.object->lazy_is_proxy ||
        receiver.as.object->lazy_uninitialized) {
        return 0;
    }
    if (ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_GET)) {
        return 1;
    }
    return !ptn_declared_class_direct_non_private_method_exists(receiver.as.object->class_name, "__get");
}

static PTN_UNUSED int ptn_lazy_object_property_read_needs_initialization(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    int allow_magic_get,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT ||
        !receiver.as.object->lazy_uninitialized ||
        receiver.as.object->lazy_initializing) {
        return 0;
    }

    const PtnObjectPropertyMetadata *metadata = NULL;
    if (ptn_lazy_object_property_access_uses_local_slot(
            runtime,
            receiver,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            line,
            &metadata
        )) {
        return 0;
    }

    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    if (metadata != NULL &&
        metadata->hook_has_get &&
        !metadata->lazy_skip &&
        !ptn_active_property_hook_matches(
            runtime,
            receiver.as.object,
            metadata,
            hook_declaring_class,
            access_scope,
            property
        ) &&
        runtime != NULL &&
        runtime->property_hook_get != NULL) {
        return 0;
    }

    if (allow_magic_get &&
        metadata == NULL &&
        ptn_magic_property_get_exists_inactive(runtime, receiver, property) &&
        ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
            runtime,
            receiver,
            "__get"
        )) {
        return 0;
    }

        return 1;
}

static PTN_UNUSED int ptn_lazy_object_property_isset_needs_initialization(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT ||
        !receiver.as.object->lazy_uninitialized ||
        receiver.as.object->lazy_initializing) {
        return 0;
    }

    const PtnObjectPropertyMetadata *metadata = NULL;
    if (ptn_lazy_object_property_access_uses_local_slot(
            runtime,
            receiver,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            line,
            &metadata
        )) {
        return 0;
    }

    if (metadata == NULL &&
        ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_ISSET) &&
        ptn_magic_property_get_exists_inactive(runtime, receiver, property)) {
        return 0;
    }

    if (metadata == NULL &&
        ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
            runtime,
            receiver,
            "__isset"
        ) &&
        !ptn_magic_property_is_active(runtime, receiver, property, PTN_MAGIC_PROPERTY_ISSET)) {
        return 0;
    }

    return 1;
}

static PTN_UNUSED PtnValue ptn_object_read_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    PtnValue exception_property = ptn_null();
    if (ptn_exception_property_read(receiver, property, &exception_property)) {
        return exception_property;
    }
    if (receiver.type == PTN_EXCEPTION) {
        ptn_emit_undefined_exception_property_warning(runtime, receiver.as.exception, property, line);
        return ptn_null();
    }
    if (receiver.type == PTN_CLOSURE) {
        ptn_emit_closure_undefined_property_warning(runtime, property, line);
        return ptn_null();
    }
    if (receiver.type != PTN_OBJECT) {
        ptn_emit_non_object_property_read_warning(runtime, property, receiver, line);
        return ptn_null();
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_emit_incomplete_object_property_access_warning(runtime, receiver.as.object, line);
        return ptn_null();
    }
    if (ptn_initialized_lazy_proxy_should_forward_property_read(runtime, receiver, property)) {
        receiver = ptn_lazy_object_effective_initialized_proxy_receiver_for_access(
            runtime,
            receiver,
            line
        );
        if (receiver.type != PTN_OBJECT || receiver.as.object == NULL) {
            return ptn_null();
        }
    }
    if (receiver.as.object->lazy_uninitialized && !receiver.as.object->lazy_initializing) {
        int receiver_was_lazy_proxy = receiver.as.object->lazy_is_proxy;
        if (ptn_lazy_object_property_read_needs_initialization(
                runtime,
                receiver,
                property,
                access_scope,
                1,
                line
            ) &&
            !ptn_lazy_object_initialize(runtime, receiver, line)) {
            return ptn_null();
        }
        if (receiver_was_lazy_proxy && !receiver.as.object->lazy_uninitialized) {
            receiver = ptn_lazy_object_effective_initialized_proxy_receiver_for_access(
                runtime,
                receiver,
                line
            );
            if (receiver.type != PTN_OBJECT || receiver.as.object == NULL) {
                return ptn_null();
            }
        }
    }
    if (ptn_initialized_lazy_proxy_should_forward_property_read(runtime, receiver, property)) {
        receiver = ptn_lazy_object_effective_initialized_proxy_receiver_for_access(
            runtime,
            receiver,
            line
        );
        if (receiver.type != PTN_OBJECT || receiver.as.object == NULL) {
            return ptn_null();
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue date_interval_value = ptn_null();
    if (ptn_internal_date_interval_property_read(
        runtime,
        receiver,
        property,
        line,
        &date_interval_value
    )) {
        return date_interval_value;
    }
    PtnValue internal_xml_value = ptn_null();
    if (ptn_internal_xml_property_read(
        runtime,
        receiver,
        property,
        line,
        &internal_xml_value
    )) {
        return internal_xml_value;
    }
    PtnValue array_object_value = ptn_null();
    if (ptn_internal_array_object_property_read(
        runtime,
        receiver,
        property,
        access_scope,
        line,
        &array_object_value
    )) {
        return array_object_value;
    }
#endif
    PtnPropertyVisibility static_visibility = PTN_PROPERTY_PUBLIC;
    const char *inaccessible_static_declaring_class = NULL;
    if (ptn_object_static_property_inaccessible(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        &static_visibility,
        &inaccessible_static_declaring_class
    )) {
        ptn_throw_property_visibility_error(
            runtime,
            static_visibility,
            ptn_static_property_visibility_error_class(
                static_visibility,
                receiver.as.object->class_name,
                inaccessible_static_declaring_class
            ),
            property,
            line
        );
        return ptn_null();
    }
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 0);
    if (blocked_metadata != NULL) {
        PtnValue magic_value = ptn_null();
        if (ptn_magic_property_get(runtime, receiver, property, line, &magic_value)) {
            PtnValue read_value = ptn_value_clone_deref(magic_value);
            ptn_value_destroy(&magic_value);
            return read_value;
        }
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1,
        line
    );
    if (storage_key == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                0,
                &magic_value
            )
        ) {
            PtnValue read_value = ptn_value_clone_deref(magic_value);
            ptn_value_destroy(&magic_value);
            return read_value;
        }
        storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            0,
            line
        );
    }
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    const char *static_declaring_class = NULL;
    int static_property_as_instance = metadata == NULL && ptn_object_static_property_visibility(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        NULL,
        &static_declaring_class
    );
    ptn_array_key_free(key);
    free(storage_key);
    if (ptn_property_is_set_only_virtual(metadata)) {
        ptn_throw_set_only_virtual_property_read_error(runtime, metadata, line);
        return ptn_null();
    }
    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    int active_same_property_hook = ptn_active_property_hook_matches(
        runtime,
        receiver.as.object,
        metadata,
        hook_declaring_class,
        access_scope,
        property
    );
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        !metadata->lazy_skip &&
        !active_same_property_hook &&
        runtime != NULL &&
        runtime->property_hook_get != NULL
    ) {
        PtnValue hook_value = ptn_null();
        if (runtime->property_hook_get(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            line,
            &hook_value
        )) {
            ptn_declared_class_property_hook_deprecation(
                runtime,
                hook_declaring_class,
                metadata->display_name,
                1,
                line
            );
            if (hook_value.type == PTN_REFERENCE) {
                PtnValue read_value = ptn_value_clone_deref(hook_value);
                ptn_value_destroy(&hook_value);
                return read_value;
            }
            return hook_value;
        }
    }
    if (entry == NULL) {
        if (metadata != NULL && metadata->lazy_skip) {
            if (ptn_property_type_is_declared(metadata->type_kind)) {
                ptn_throw_uninitialized_typed_property_error(
                    runtime,
                    metadata->declaring_class,
                    metadata->display_name,
                    line
                );
            }
            return ptn_null();
        }
        if (metadata != NULL &&
            metadata->type_kind == PTN_PROPERTY_TYPE_NONE &&
            !metadata->is_unset) {
            ptn_array_set_entry(
                receiver.as.object->properties,
                ptn_array_string_key(metadata->storage_name),
                ptn_null()
            );
            return ptn_null();
        }
        if (metadata != NULL && ptn_property_type_is_declared(metadata->type_kind)) {
            if (metadata->is_unset) {
                PtnValue magic_value = ptn_null();
                if (ptn_magic_property_get(runtime, receiver, property, line, &magic_value)) {
                    PtnValue coerced = ptn_null();
                    if (!ptn_coerce_unset_typed_property_magic_value(
                        runtime,
                        receiver,
                        metadata,
                        magic_value,
                        line,
                        &coerced
                    )) {
                        ptn_value_destroy(&magic_value);
                        return ptn_null();
                    }
                    ptn_value_destroy(&magic_value);
                    return coerced;
                }
            }
            ptn_throw_uninitialized_typed_property_error(
                runtime,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            return ptn_null();
        }
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                0,
                &magic_value
            )
        ) {
            PtnValue read_value = ptn_value_clone_deref(magic_value);
            ptn_value_destroy(&magic_value);
            return read_value;
        }
        if (static_property_as_instance) {
            ptn_emit_static_property_non_static_notice(
                runtime,
                static_declaring_class,
                property,
                line
            );
        }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        if (ptn_internal_class_name_is_pdo_row(receiver.as.object->class_name)) {
            return ptn_null();
        }
#endif
        ptn_emit_undefined_property_warning(runtime, receiver.as.object, property, line);
        if (runtime != NULL &&
            runtime->exceptions != NULL &&
            runtime->exceptions->active_exception != NULL) {
            return ptn_null();
        }
        char *updated_storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            1,
            line
        );
        if (updated_storage_key != NULL) {
            PtnArrayKey updated_key = ptn_array_string_key(updated_storage_key);
            PtnArrayEntry *updated_entry =
                ptn_array_entry_for_key(receiver.as.object->properties, updated_key);
            ptn_array_key_free(updated_key);
            free(updated_storage_key);
            if (updated_entry != NULL) {
                return ptn_value_clone_deref(updated_entry->value);
            }
        }
        return ptn_null();
    }
    if (static_property_as_instance) {
        ptn_emit_static_property_non_static_notice(
            runtime,
            static_declaring_class,
            property,
            line
        );
    }
    if (metadata != NULL && !metadata->lazy_skip) {
        ptn_declared_class_property_hook_deprecation(
            runtime,
            ptn_property_hook_get_declaring_class(metadata),
            metadata->display_name,
            1,
            line
        );
    }
    return ptn_value_clone_deref(entry->value);
}

static PTN_UNUSED PtnValue ptn_object_read_property_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    const char *access_scope,
    size_t line
) {
    if (property == NULL || property_len == strlen(property)) {
        return ptn_object_read_property(runtime, receiver, property, access_scope, line);
    }
    if (property_len == 0 || property[0] != '\0') {
        ptn_emit_type_error(
            &runtime->diagnostics,
            "Unsupported dynamic property name containing embedded NUL"
        );
        exit(255);
    }

    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        ptn_emit_undefined_exception_property_warning(runtime, receiver.as.exception, property, line);
        return ptn_null();
    }
    if (receiver.type == PTN_CLOSURE) {
        ptn_emit_closure_undefined_property_warning(runtime, property, line);
        return ptn_null();
    }
    if (receiver.type != PTN_OBJECT) {
        ptn_emit_non_object_property_read_warning(runtime, property, receiver, line);
        return ptn_null();
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_emit_incomplete_object_property_access_warning(runtime, receiver.as.object, line);
        return ptn_null();
    }

    PtnValue magic_value;
    if (
        runtime != NULL &&
        runtime->magic_property_read != NULL &&
        runtime->magic_property_read(
            runtime,
            receiver,
            property,
            property_len,
            line,
            0,
            &magic_value
        )
    ) {
        PtnValue read_value = ptn_value_clone_deref(magic_value);
        ptn_value_destroy(&magic_value);
        return read_value;
    }

    ptn_throw_exception_at(
        runtime,
        "Error",
        "Cannot access property starting with \"\\0\"",
        runtime == NULL ? NULL : runtime->source_path,
        line
    );
    return ptn_null();
}

static PTN_UNUSED int ptn_constant_expression_property_receiver_allowed(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(receiver);
    int forbidden_object = 0;
    if (resolved.type == PTN_OBJECT) {
        forbidden_object =
            resolved.as.object == NULL || resolved.as.object->enum_case_name == NULL;
    } else if (resolved.type == PTN_CLOSURE || resolved.type == PTN_EXCEPTION) {
        forbidden_object = 1;
    }
    if (forbidden_object) {
        const char *message =
            "Fetching properties on non-enums in constant expressions is not allowed";
        if (runtime != NULL) {
            ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
        } else {
            fprintf(stderr, "Fatal error: %s\n", message);
            exit(255);
        }
        return 0;
    }
    return 1;
}

static PTN_UNUSED PtnValue ptn_constant_expression_read_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    if (!ptn_constant_expression_property_receiver_allowed(runtime, receiver, line)) {
        return ptn_null();
    }
    return ptn_object_read_property(runtime, receiver, property, access_scope, line);
}

static PTN_UNUSED PtnValue ptn_constant_expression_read_property_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    const char *access_scope,
    size_t line
) {
    if (!ptn_constant_expression_property_receiver_allowed(runtime, receiver, line)) {
        return ptn_null();
    }
    return ptn_object_read_property_len(runtime, receiver, property, property_len, access_scope, line);
}

static PTN_UNUSED PtnValue ptn_object_read_property_for_indirect_write(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        ptn_throw_property_modification_on_non_object(runtime, property, receiver, line);
        return ptn_null();
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_throw_incomplete_object_property_modification(runtime, receiver.as.object, line);
        return ptn_null();
    }
    if (receiver.as.object->lazy_uninitialized && !receiver.as.object->lazy_initializing) {
        if (!ptn_lazy_object_initialize(runtime, receiver, line)) {
            return ptn_null();
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue date_interval_value = ptn_null();
    if (ptn_internal_date_interval_property_read(
        runtime,
        receiver,
        property,
        line,
        &date_interval_value
    )) {
        return date_interval_value;
    }
    PtnValue internal_xml_value = ptn_null();
    if (ptn_internal_xml_property_read(
        runtime,
        receiver,
        property,
        line,
        &internal_xml_value
    )) {
        return internal_xml_value;
    }
    PtnValue array_object_value = ptn_null();
    if (ptn_internal_array_object_property_read(
        runtime,
        receiver,
        property,
        access_scope,
        line,
        &array_object_value
    )) {
        return array_object_value;
    }
#endif
    int overloaded_property_read =
        ptn_magic_property_get_exists_inactive(runtime, receiver, property);
    if (!overloaded_property_read &&
        !ptn_object_preflight_dynamic_property_creation(
            runtime,
            receiver,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
            line
        )) {
        return ptn_null();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
        1,
        line
    );
    if (storage_key == NULL) {
        char *read_storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            1,
            line
        );
        if (read_storage_key != NULL) {
            PtnArrayKey read_key = ptn_array_string_key(read_storage_key);
            PtnArrayEntry *read_entry =
                ptn_array_entry_for_key(receiver.as.object->properties, read_key);
            ptn_array_key_free(read_key);
            if (read_entry != NULL) {
                PtnValue current = ptn_value_clone_deref(read_entry->value);
                if (current.type == PTN_OBJECT) {
                    free(read_storage_key);
                    return current;
                }
                ptn_value_destroy(&current);
            }
            free(read_storage_key);
        }
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                0,
                &magic_value
            )
        ) {
            return magic_value;
        }
        storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
            0,
            line
        );
        if (storage_key == NULL) {
            return ptn_null();
        }
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    int active_same_property_hook = ptn_active_property_hook_matches(
        runtime,
        receiver.as.object,
        metadata,
        hook_declaring_class,
        access_scope,
        property
    );
    if (metadata != NULL && metadata->is_readonly) {
        if (ptn_object_property_is_date_period_internal(runtime, receiver.as.object, metadata)) {
            ptn_array_key_free(key);
            free(storage_key);
            ptn_throw_readonly_property_error(
                runtime,
                receiver.as.object->class_name,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            return ptn_null();
        }
        if (entry != NULL) {
            PtnValue current = ptn_value_clone_deref(entry->value);
            if (current.type == PTN_OBJECT) {
                ptn_array_key_free(key);
                free(storage_key);
                return current;
            }
            ptn_value_destroy(&current);
        }
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_readonly_property_indirect_modification_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name,
            line
        );
        return ptn_null();
    }
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        !metadata->lazy_skip &&
        !active_same_property_hook &&
        runtime != NULL &&
        runtime->property_hook_get != NULL
    ) {
        PtnValue hook_value = ptn_null();
        if (runtime->property_hook_get(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            line,
            &hook_value
        )) {
            ptn_declared_class_property_hook_deprecation(
                runtime,
                hook_declaring_class,
                metadata->display_name,
                1,
                line
            );
            ptn_array_key_free(key);
            free(storage_key);
            if (hook_value.type == PTN_REFERENCE) {
                return hook_value;
            }
            if (ptn_value_deref(hook_value).type == PTN_OBJECT) {
                return hook_value;
            }
            ptn_value_destroy(&hook_value);
            ptn_throw_hooked_property_indirect_modification_error(runtime, metadata, line);
            return ptn_null();
        }
    }
    if (metadata != NULL && metadata->has_hooks && !metadata->hook_has_get) {
        if (entry != NULL) {
            PtnValue current = ptn_value_clone_deref(entry->value);
            if (current.type == PTN_OBJECT) {
                ptn_array_key_free(key);
                free(storage_key);
                return current;
            }
            ptn_value_destroy(&current);
        }
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_hooked_property_indirect_modification_error(runtime, metadata, line);
        return ptn_null();
    }
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        if (metadata != NULL &&
            metadata->set_visibility != metadata->read_visibility &&
            !ptn_property_visibility_allows(
                runtime,
                metadata->set_visibility,
                ptn_property_visibility_scope_class(metadata, metadata->set_visibility),
                access_scope
            )) {
            ptn_throw_property_indirect_set_visibility_error(
                runtime,
                metadata->set_visibility,
                metadata->declaring_class,
                metadata->display_name,
                access_scope
            );
            return ptn_null();
        }
        PtnValue magic_value;
        if (
            metadata == NULL &&
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                0,
                &magic_value
            )
        ) {
            return magic_value;
        }
        if (metadata != NULL && ptn_property_type_is_declared(metadata->type_kind)) {
            if (ptn_property_metadata_accepts_array_auto_initialization(runtime, metadata)) {
                return ptn_array_from_literal_entries(0, NULL);
            }
            ptn_throw_property_array_auto_initialization_error(
                runtime,
                metadata->declaring_class,
                metadata->display_name,
                metadata->type_text,
                0,
                line
            );
            return ptn_null();
        }
        return ptn_null();
    }
    if (metadata != NULL && ptn_property_type_is_declared(metadata->type_kind)) {
        PtnValue current = ptn_value_deref(entry->value);
        if (current.type == PTN_NULL &&
            !ptn_property_metadata_accepts_array_auto_initialization(runtime, metadata)) {
            ptn_throw_property_array_auto_initialization_error(
                runtime,
                metadata->declaring_class,
                metadata->display_name,
                metadata->type_text,
                0,
                line
            );
            return ptn_null();
        }
    }
    return ptn_value_clone_deref(entry->value);
}

static int ptn_lazy_object_compound_read_targets_dynamic_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (property == NULL ||
        receiver.type != PTN_OBJECT ||
        receiver.as.object == NULL ||
        !receiver.as.object->lazy_uninitialized ||
        receiver.as.object->lazy_initializing ||
        receiver.as.object->lazy_is_proxy) {
        return 0;
    }
    if (ptn_object_metadata_for_display_name(receiver.as.object, property) != NULL) {
        return 0;
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1,
        line
    );
    if (storage_key == NULL) {
        return 0;
    }
    int dynamic_property =
        ptn_object_property_metadata(receiver.as.object, storage_key) == NULL;
    free(storage_key);
    return dynamic_property;
}

static PTN_UNUSED PtnValue ptn_object_read_property_for_compound_assignment(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    if (ptn_lazy_object_compound_read_targets_dynamic_property(
            runtime,
            receiver,
            property,
            access_scope,
            line
        ) &&
        !ptn_lazy_object_initialize_for_dynamic_property_compound(runtime, receiver, line)) {
        return ptn_null();
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue date_interval_value = ptn_null();
    if (ptn_internal_date_interval_property_read(
        runtime,
        receiver,
        property,
        line,
        &date_interval_value
    )) {
        return date_interval_value;
    }
    PtnValue internal_xml_value = ptn_null();
    if (ptn_internal_xml_property_read(
        runtime,
        receiver,
        property,
        line,
        &internal_xml_value
    )) {
        return internal_xml_value;
    }
    PtnValue array_object_value = ptn_null();
    if (ptn_internal_array_object_property_read(
        runtime,
        receiver,
        property,
        access_scope,
        line,
        &array_object_value
    )) {
        return array_object_value;
    }
#endif
    PtnValue resolved_receiver = ptn_value_deref(receiver);
    int has_magic_get = resolved_receiver.type == PTN_OBJECT &&
        runtime != NULL &&
        runtime->declared_method_exists != NULL &&
        runtime->declared_method_exists(resolved_receiver.as.object->class_name, "__get");
    if (!has_magic_get) {
        if (!ptn_object_preflight_dynamic_property_creation(
                runtime,
                receiver,
                property,
                access_scope,
                PTN_PROPERTY_ACCESS_WRITE,
                line
            )) {
            return ptn_null();
        }
    }
    return ptn_object_read_property(runtime, receiver, property, access_scope, line);
}

static PTN_UNUSED PtnValue ptn_object_read_property_for_nested_write_receiver(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line,
    int read_for_compound
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        ptn_throw_property_modification_on_non_object(runtime, property, receiver, line);
        return ptn_null();
    }
    if (receiver.as.object->lazy_uninitialized && !receiver.as.object->lazy_initializing) {
        if (!ptn_lazy_object_initialize(runtime, receiver, line)) {
            return ptn_null();
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue date_interval_value = ptn_null();
    if (ptn_internal_date_interval_property_read(
        runtime,
        receiver,
        property,
        line,
        &date_interval_value
    )) {
        return date_interval_value;
    }
    PtnValue internal_xml_value = ptn_null();
    if (ptn_internal_xml_property_read(
        runtime,
        receiver,
        property,
        line,
        &internal_xml_value
    )) {
        return internal_xml_value;
    }
    PtnValue array_object_value = ptn_null();
    if (ptn_internal_array_object_property_read(
        runtime,
        receiver,
        property,
        access_scope,
        line,
        &array_object_value
    )) {
        return array_object_value;
    }
#endif
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 0);
    if (blocked_metadata != NULL) {
        PtnValue magic_value = ptn_null();
        if (ptn_magic_property_get(runtime, receiver, property, line, &magic_value)) {
            return magic_value;
        }
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1,
        line
    );
    if (storage_key == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                0,
                &magic_value
            )
        ) {
            return magic_value;
        }
        storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            0,
            line
        );
    }
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        if (metadata != NULL && ptn_property_type_is_declared(metadata->type_kind)) {
            if (metadata->is_readonly) {
                if (read_for_compound) {
                    ptn_throw_uninitialized_typed_property_error(
                        runtime,
                        metadata->declaring_class,
                        metadata->display_name,
                        line
                    );
                } else {
                    ptn_throw_readonly_property_indirect_modification_error(
                        runtime,
                        metadata->declaring_class,
                        metadata->display_name,
                        line
                    );
                }
                return ptn_null();
            }
            if (!ptn_property_metadata_accepts_array_auto_initialization(runtime, metadata)) {
                ptn_throw_uninitialized_typed_property_error(
                    runtime,
                    metadata->declaring_class,
                    metadata->display_name,
                    line
                );
                return ptn_null();
            }
            if (
                metadata->set_visibility != metadata->read_visibility &&
                !ptn_property_visibility_allows(
                    runtime,
                    metadata->set_visibility,
                    ptn_property_visibility_scope_class(metadata, metadata->set_visibility),
                    access_scope
                )
            ) {
                ptn_throw_property_indirect_set_visibility_error(
                    runtime,
                    metadata->set_visibility,
                    metadata->declaring_class,
                    metadata->display_name,
                    access_scope
                );
                return ptn_null();
            }
            return ptn_array_from_literal_entries(0, NULL);
        }
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                0,
                &magic_value
            )
        ) {
            return magic_value;
        }
        ptn_emit_undefined_property_warning(runtime, receiver.as.object, property, line);
        return ptn_null();
    }
    return ptn_value_clone_deref(entry->value);
}

static PTN_UNUSED PtnValue ptn_object_read_property_no_magic(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    PtnValue exception_property = ptn_null();
    if (ptn_exception_property_read(receiver, property, &exception_property)) {
        return exception_property;
    }
    if (receiver.type == PTN_EXCEPTION) {
        ptn_emit_undefined_exception_property_warning(runtime, receiver.as.exception, property, line);
        return ptn_null();
    }
    if (receiver.type != PTN_OBJECT) {
        ptn_emit_non_object_property_read_warning(runtime, property, receiver, line);
        return ptn_null();
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_emit_incomplete_object_property_access_warning(runtime, receiver.as.object, line);
        return ptn_null();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        0,
        line
    );
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    const char *static_declaring_class = NULL;
    int static_property_as_instance = metadata == NULL && ptn_object_static_property_accessible(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        &static_declaring_class
    );
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        if (metadata != NULL && ptn_property_type_is_declared(metadata->type_kind)) {
            ptn_throw_uninitialized_typed_property_error(
                runtime,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            return ptn_null();
        }
        if (metadata == NULL || metadata->is_unset) {
            PtnValue magic_value = ptn_null();
            if (ptn_magic_property_get(runtime, receiver, property, line, &magic_value)) {
                return magic_value;
            }
        }
        if (static_property_as_instance) {
            ptn_emit_static_property_non_static_notice(
                runtime,
                static_declaring_class,
                property,
                line
            );
        }
        ptn_emit_undefined_property_warning(runtime, receiver.as.object, property, line);
        return ptn_null();
    }
    if (static_property_as_instance) {
        ptn_emit_static_property_non_static_notice(
            runtime,
            static_declaring_class,
            property,
            line
        );
    }
    return ptn_value_clone_deref(entry->value);
}

static PTN_UNUSED PtnLookupResult ptn_object_property_lookup_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    PtnValue exception_property = ptn_null();
    if (ptn_exception_property_read(receiver, property, &exception_property)) {
        return ptn_lookup_found(exception_property);
    }
    if (receiver.type != PTN_OBJECT) {
        return ptn_lookup_missing();
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_emit_incomplete_object_property_access_warning(runtime, receiver.as.object, line);
        return ptn_lookup_missing();
    }
    if (receiver.as.object->lazy_uninitialized && !receiver.as.object->lazy_initializing) {
        if (ptn_lazy_object_property_read_needs_initialization(
                runtime,
                receiver,
                property,
                access_scope,
                1,
                line
            ) &&
            !ptn_lazy_object_initialize(runtime, receiver, line)) {
            return ptn_lookup_missing();
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue date_interval_value = ptn_null();
    if (ptn_internal_date_interval_property_read(
        runtime,
        receiver,
        property,
        line,
        &date_interval_value
    )) {
        return ptn_lookup_found(date_interval_value);
    }
    PtnValue internal_xml_value = ptn_null();
    if (ptn_internal_xml_property_read(
        runtime,
        receiver,
        property,
        line,
        &internal_xml_value
    )) {
        return ptn_lookup_found(internal_xml_value);
    }
#endif
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1,
        line
    );
    if (storage_key == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                1,
                &magic_value
            )
        ) {
            return ptn_lookup_found(magic_value);
        }
        return ptn_lookup_missing();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    ptn_array_key_free(key);
    free(storage_key);
    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    int active_same_property_hook = ptn_active_property_hook_matches(
        runtime,
        receiver.as.object,
        metadata,
        hook_declaring_class,
        access_scope,
        property
    );
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        !metadata->lazy_skip &&
        !active_same_property_hook &&
        runtime != NULL &&
        runtime->property_hook_get != NULL
    ) {
        PtnValue hook_value = ptn_null();
        if (runtime->property_hook_get(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            line,
            &hook_value
        )) {
            ptn_declared_class_property_hook_deprecation(
                runtime,
                hook_declaring_class,
                metadata->display_name,
                1,
                line
            );
            return ptn_lookup_found(hook_value);
        }
    }
    if (entry == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                1,
                &magic_value
            )
        ) {
            return ptn_lookup_found(magic_value);
        }
        return ptn_lookup_missing();
    }
    if (metadata != NULL) {
        ptn_declared_class_property_hook_deprecation(
            runtime,
            ptn_property_hook_get_declaring_class(metadata),
            metadata->display_name,
            1,
            line
        );
    }
    return ptn_lookup_found(ptn_value_clone_deref(entry->value));
}

static PTN_UNUSED PtnLookupResult ptn_object_property_probe_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    PtnValue stable_receiver = ptn_value_clone_deref(receiver);
    receiver = stable_receiver;
    PtnLookupResult result = ptn_lookup_missing();
    PtnValue exception_property = ptn_null();
    if (ptn_exception_property_read(receiver, property, &exception_property)) {
        result = ptn_lookup_found(exception_property);
        goto done;
    }
    if (receiver.type != PTN_OBJECT) {
        goto done;
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_emit_incomplete_object_property_access_warning(runtime, receiver.as.object, line);
        goto done;
    }
    if (receiver.as.object->lazy_uninitialized && !receiver.as.object->lazy_initializing) {
        if (ptn_lazy_object_property_isset_needs_initialization(
                runtime,
                receiver,
                property,
                access_scope,
                line
            ) &&
            !ptn_lazy_object_initialize(runtime, receiver, line)) {
            goto done;
        }
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1,
        line
    );
    if (storage_key == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                1,
                &magic_value
            )
        ) {
            result = ptn_lookup_found(magic_value);
            goto done;
        }
        goto done;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    int active_same_property_hook = ptn_active_property_hook_matches(
        runtime,
        receiver.as.object,
        metadata,
        hook_declaring_class,
        access_scope,
        property
    );
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        !metadata->lazy_skip &&
        !active_same_property_hook &&
        runtime != NULL &&
        runtime->property_hook_get != NULL
    ) {
        PtnValue hook_value = ptn_null();
        if (runtime->property_hook_get(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            line,
            &hook_value
        )) {
            ptn_declared_class_property_hook_deprecation(
                runtime,
                hook_declaring_class,
                metadata->display_name,
                1,
                line
            );
            ptn_array_key_free(key);
            free(storage_key);
            result = ptn_lookup_found(hook_value);
            goto done;
        }
    }
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(
                runtime,
                receiver,
                property,
                strlen(property),
                line,
                1,
                &magic_value
            )
        ) {
            result = ptn_lookup_found(magic_value);
            goto done;
        }
        goto done;
    }
    result = ptn_lookup_found(ptn_value_clone_deref(entry->value));

done:
    ptn_value_destroy(&stable_receiver);
    return result;
}

static PTN_UNUSED int ptn_object_property_is_set(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    PtnValue stable_receiver = ptn_value_clone_deref(receiver);
    receiver = stable_receiver;
    int result = 0;
    PtnValue exception_property = ptn_null();
    if (ptn_exception_property_read(receiver, property, &exception_property)) {
        result = ptn_value_deref(exception_property).type != PTN_NULL;
        ptn_value_destroy(&exception_property);
        goto done;
    }
    if (receiver.type != PTN_OBJECT) {
        goto done;
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_emit_incomplete_object_property_access_warning(runtime, receiver.as.object, line);
        goto done;
    }
    if (receiver.as.object->lazy_uninitialized && !receiver.as.object->lazy_initializing) {
        if (ptn_lazy_object_property_isset_needs_initialization(
                runtime,
                receiver,
                property,
                access_scope,
                line
            ) &&
            !ptn_lazy_object_initialize(runtime, receiver, line)) {
            goto done;
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    int simplexml_isset = 0;
    if (ptn_simplexml_property_is_set(receiver, property, &simplexml_isset)) {
        result = simplexml_isset;
        goto done;
    }
    PtnValue date_interval_value = ptn_null();
    if (ptn_internal_date_interval_property_read(runtime, receiver, property, line, &date_interval_value)) {
        result = ptn_value_deref(date_interval_value).type != PTN_NULL;
        ptn_value_destroy(&date_interval_value);
        goto done;
    }
    PtnValue internal_value = ptn_null();
    if (ptn_internal_xml_property_read(runtime, receiver, property, line, &internal_value)) {
        result = ptn_value_deref(internal_value).type != PTN_NULL;
        ptn_value_destroy(&internal_value);
        goto done;
    }
    int array_object_isset = 0;
    if (ptn_internal_array_object_property_isset(
        runtime,
        receiver,
        property,
        access_scope,
        line,
        &array_object_isset
    )) {
        result = array_object_isset;
        goto done;
    }
#endif
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1,
        line
    );
    if (storage_key == NULL) {
        int magic_isset = 0;
        if (ptn_magic_property_isset(runtime, receiver, property, line, &magic_isset)) {
            result = magic_isset;
            goto done;
        }
        goto done;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    int active_same_property_hook = ptn_active_property_hook_matches(
        runtime,
        receiver.as.object,
        metadata,
        hook_declaring_class,
        access_scope,
        property
    );
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        !metadata->lazy_skip &&
        !active_same_property_hook &&
        runtime != NULL &&
        runtime->property_hook_get != NULL
    ) {
        PtnValue hook_value = ptn_null();
        if (runtime->property_hook_get(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            line,
            &hook_value
        )) {
            ptn_declared_class_property_hook_deprecation(
                runtime,
                hook_declaring_class,
                metadata->display_name,
                1,
                line
            );
            result = ptn_value_deref(hook_value).type != PTN_NULL;
            ptn_value_destroy(&hook_value);
            ptn_array_key_free(key);
            free(storage_key);
            goto done;
        }
    }
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        if (metadata != NULL && metadata->lazy_skip) {
            goto done;
        }
        if (ptn_property_is_set_only_virtual(metadata)) {
            ptn_throw_set_only_virtual_property_read_error(runtime, metadata, line);
            goto done;
        }
        if (metadata == NULL || metadata->is_unset) {
            int magic_isset = 0;
            if (ptn_magic_property_isset(runtime, receiver, property, line, &magic_isset)) {
                result = magic_isset;
                goto done;
            }
        }
        goto done;
    }
    result = ptn_value_deref(entry->value).type != PTN_NULL;

done:
    ptn_value_destroy(&stable_receiver);
    return result;
}

static PTN_UNUSED PtnValue ptn_object_write_property_with_mode_len_impl(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    const char *access_scope,
    PtnValue value,
    size_t line,
    int indirect_write,
    int overloaded_notice_already_emitted
) {
    if (ptn_runtime_has_active_exception(runtime)) {
        return ptn_null();
    }
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_exception_write_dynamic_property(
            runtime,
            receiver.as.exception,
            property,
            value,
            line
        );
    }
    if (receiver.type != PTN_OBJECT) {
        if (indirect_write) {
            ptn_throw_property_modification_on_non_object(runtime, property, receiver, line);
        } else {
            ptn_throw_property_assignment_on_non_object(runtime, property, receiver, line);
        }
        return ptn_null();
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_throw_incomplete_object_property_modification(runtime, receiver.as.object, line);
        return ptn_null();
    }
    PtnValue preserved_lazy_write_value = ptn_null();
    int has_preserved_lazy_write_value = 0;
#define PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE() \
    do { \
        if (has_preserved_lazy_write_value) { \
            ptn_value_destroy(&preserved_lazy_write_value); \
            has_preserved_lazy_write_value = 0; \
        } \
    } while (0)
#define PTN_OBJECT_WRITE_RETURN(expr) \
    do { \
        PtnValue ptn_object_write_result__ = (expr); \
        PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE(); \
        return ptn_object_write_result__; \
    } while (0)
    if (receiver.as.object->lazy_uninitialized && !receiver.as.object->lazy_initializing) {
        int local_lazy_slot = 0;
        int lazy_set_hook_dispatch = 0;
        int lazy_magic_set_dispatch = 0;
        char *lazy_storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            indirect_write ? PTN_PROPERTY_ACCESS_INDIRECT_WRITE : PTN_PROPERTY_ACCESS_WRITE,
            1,
            line
        );
        if (lazy_storage_key != NULL) {
            PtnArrayKey lazy_key = ptn_array_string_key(lazy_storage_key);
            PtnArrayEntry *lazy_entry =
                ptn_array_entry_for_key(receiver.as.object->properties, lazy_key);
            const PtnObjectPropertyMetadata *lazy_metadata =
                ptn_object_property_metadata(receiver.as.object, lazy_storage_key);
            ptn_array_key_free(lazy_key);
            local_lazy_slot = lazy_metadata != NULL && lazy_metadata->lazy_skip;
            if (
                !indirect_write &&
                lazy_metadata != NULL &&
                lazy_metadata->hook_has_set &&
                !ptn_active_property_hook_matches(
                    runtime,
                    receiver.as.object,
                    lazy_metadata,
                    ptn_property_hook_set_declaring_class(lazy_metadata),
                    access_scope,
                    property
                )
            ) {
                lazy_set_hook_dispatch = 1;
            }
            lazy_magic_set_dispatch =
                !indirect_write &&
                lazy_metadata == NULL &&
                lazy_entry == NULL &&
                ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
                    runtime,
                    receiver,
                    "__set"
                ) &&
                !ptn_magic_property_is_active_len(
                    runtime,
                    receiver,
                    property,
                    property_len,
                    PTN_MAGIC_PROPERTY_SET
                );
            free(lazy_storage_key);
        } else {
            lazy_magic_set_dispatch =
                !indirect_write &&
                ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
                    runtime,
                    receiver,
                    "__set"
                ) &&
                !ptn_magic_property_is_active_len(
                    runtime,
                    receiver,
                    property,
                    property_len,
                    PTN_MAGIC_PROPERTY_SET
                );
        }
        if (!local_lazy_slot && !lazy_set_hook_dispatch && !lazy_magic_set_dispatch) {
            preserved_lazy_write_value = ptn_value_clone_deref(value);
            has_preserved_lazy_write_value = 1;
            value = preserved_lazy_write_value;
            if (!ptn_lazy_object_initialize(runtime, receiver, line)) {
                PTN_OBJECT_WRITE_RETURN(ptn_null());
            }
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (indirect_write) {
        PtnValue internal_xml_value = ptn_null();
        if (ptn_internal_xml_property_write_indirect(
            runtime,
            receiver,
            property,
            value,
            line,
            &internal_xml_value
        )) {
            PTN_OBJECT_WRITE_RETURN(internal_xml_value);
        }
    } else {
        PtnValue internal_xml_value = ptn_null();
        if (ptn_internal_xml_property_write(
            runtime,
            receiver,
            property,
            value,
            line,
            &internal_xml_value
        )) {
            PTN_OBJECT_WRITE_RETURN(internal_xml_value);
        }
        PtnValue array_object_value = ptn_null();
        if (ptn_internal_array_object_property_write(
            runtime,
            receiver,
            property,
            access_scope,
            value,
            line,
            &array_object_value
        )) {
            PTN_OBJECT_WRITE_RETURN(array_object_value);
        }
    }
#endif
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (
        indirect_write &&
        ptn_object_indirect_write_targets_overloaded_property(
            runtime,
            receiver,
            property,
            access_scope,
            line
        )
    ) {
        if (!overloaded_notice_already_emitted && value.type != PTN_REFERENCE) {
            ptn_emit_indirect_modification_overloaded_property_notice(
                runtime,
                receiver,
                property,
                line
            );
        }
        PTN_OBJECT_WRITE_RETURN(ptn_value_clone_deref(value));
    }
    if (
        blocked_metadata != NULL &&
        ptn_blocked_property_write_should_call_magic_set(blocked_metadata)
    ) {
        if (ptn_magic_property_set_len(runtime, receiver, property, property_len, value, line)) {
            PTN_OBJECT_WRITE_RETURN(ptn_value_clone_deref(value));
        }
    }
    if (blocked_metadata == NULL &&
        ptn_object_metadata_for_display_name(receiver.as.object, property) == NULL &&
        ptn_magic_property_set_len(runtime, receiver, property, property_len, value, line)) {
        PTN_OBJECT_WRITE_RETURN(ptn_value_clone_deref(value));
    }
    if (property_len > 0 && property[0] == '\0') {
        PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE();
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Cannot access property starting with \"\\0\"",
            runtime == NULL ? NULL : runtime->source_path,
            line
        );
        return ptn_null();
    }
    ptn_emit_static_property_non_static_notice_if_accessible(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        indirect_write ? PTN_PROPERTY_ACCESS_INDIRECT_WRITE : PTN_PROPERTY_ACCESS_WRITE,
        line
    );
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        indirect_write ? PTN_PROPERTY_ACCESS_INDIRECT_WRITE : PTN_PROPERTY_ACCESS_WRITE,
        0,
        line
    );
    if (storage_key == NULL) {
        PTN_OBJECT_WRITE_RETURN(ptn_null());
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    PtnObjectPropertyMetadata *mutable_metadata = metadata == NULL
        ? NULL
        : ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
    int suppress_dynamic_property_deprecation =
        ptn_runtime_consume_dynamic_property_deprecation_suppression(
            runtime,
            receiver.as.object,
            property
        );
    if (
        metadata != NULL &&
        metadata->is_unset &&
        ptn_magic_property_set_len(runtime, receiver, property, property_len, value, line)
    ) {
        ptn_array_key_free(key);
        free(storage_key);
        PTN_OBJECT_WRITE_RETURN(ptn_value_clone_deref(value));
    }
    if (
        indirect_write &&
        metadata != NULL &&
        metadata->has_hooks &&
        ptn_value_deref(value).type == PTN_OBJECT
    ) {
        ptn_array_key_free(key);
        free(storage_key);
        PTN_OBJECT_WRITE_RETURN(ptn_value_clone_deref(value));
    }
    if (ptn_property_is_get_only_virtual(metadata)) {
        ptn_array_key_free(key);
        free(storage_key);
        PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE();
        ptn_throw_get_only_virtual_property_write_error(
            runtime,
            metadata,
            receiver.as.object->class_name,
            line
        );
        return ptn_null();
    }
    int hook_set_deprecation_emitted = 0;
    const char *hook_declaring_class = ptn_property_hook_set_declaring_class(metadata);
    int active_same_property_hook = ptn_active_property_hook_matches(
        runtime,
        receiver.as.object,
        metadata,
        hook_declaring_class,
        access_scope,
        property
    );
    if (
        !indirect_write &&
        metadata != NULL &&
        metadata->hook_has_set &&
        !active_same_property_hook &&
        runtime != NULL &&
        runtime->property_hook_set != NULL
    ) {
        ptn_declared_class_property_hook_deprecation(
            runtime,
            hook_declaring_class,
            metadata->display_name,
            2,
            line
        );
        hook_set_deprecation_emitted = 1;
        if (runtime->property_hook_set(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            value,
            line
        )) {
            ptn_array_key_free(key);
            free(storage_key);
            PTN_OBJECT_WRITE_RETURN(ptn_value_clone_deref(value));
        }
    }
    int readonly_clone_reinit = 0;
    if (metadata != NULL && metadata->is_readonly && indirect_write) {
        if (ptn_object_property_is_date_period_internal(runtime, receiver.as.object, metadata)) {
            ptn_array_key_free(key);
            free(storage_key);
            PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE();
            ptn_throw_readonly_property_error(
                runtime,
                receiver.as.object->class_name,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            return ptn_null();
        }
        if (entry != NULL) {
            PtnValue current = ptn_value_deref(entry->value);
            PtnValue assigned = ptn_value_deref(value);
            if (current.type == PTN_OBJECT &&
                assigned.type == PTN_OBJECT &&
                current.as.object == assigned.as.object) {
                ptn_array_key_free(key);
                free(storage_key);
                PTN_OBJECT_WRITE_RETURN(ptn_value_clone(assigned));
            }
        }
        ptn_array_key_free(key);
        free(storage_key);
        PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE();
        ptn_throw_readonly_property_indirect_modification_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name,
            line
        );
        return ptn_null();
    }
    if (metadata != NULL && metadata->is_readonly && entry != NULL) {
        readonly_clone_reinit =
            receiver.as.object->readonly_clone_initializing &&
            mutable_metadata != NULL &&
            !mutable_metadata->readonly_clone_reinitialized;
        if (!readonly_clone_reinit) {
            if (receiver.as.object->lazy_initializing) {
                if (receiver.as.object->lazy_initializer_refcount_guards == SIZE_MAX) {
                    ptn_abort_out_of_memory();
                }
                ptn_object_retain(receiver.as.object);
                receiver.as.object->lazy_initializer_refcount_guards++;
            }
            ptn_array_key_free(key);
            free(storage_key);
            PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE();
            ptn_throw_readonly_property_error(
                runtime,
                receiver.as.object->class_name,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            return ptn_null();
        }
    }
    if (metadata == NULL && entry == NULL &&
        runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL) {
        ptn_array_key_free(key);
        free(storage_key);
        PTN_OBJECT_WRITE_RETURN(ptn_null());
    }
    if (metadata == NULL && entry == NULL && !suppress_dynamic_property_deprecation) {
        ptn_emit_dynamic_property_deprecation(runtime, receiver.as.object, property, line);
    }
    if (metadata != NULL && !hook_set_deprecation_emitted) {
        ptn_declared_class_property_hook_deprecation(
            runtime,
            ptn_property_hook_set_declaring_class(metadata),
            metadata->display_name,
            2,
            line
        );
    }
    PtnValue stored = ptn_null();
    if (metadata != NULL) {
        PtnObject *assignment_receiver = receiver.as.object;
        size_t refcount_before_coercion = assignment_receiver->refcount;
        ptn_object_retain(assignment_receiver);
        int coerced = ptn_property_type_coerce_assignment(
            runtime,
            metadata->type_kind,
            metadata->type_class_name,
            metadata->type_text,
            metadata->type_allows_null,
            metadata->declaring_class,
            metadata->display_name,
            value,
            0,
            line,
            &stored
        );
        int receiver_invalidated = assignment_receiver->refcount <= refcount_before_coercion;
        int active_exception = ptn_runtime_has_active_exception(runtime);
        if (receiver_invalidated && !active_exception) {
            PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE();
            ptn_throw_object_released_while_assigning_property(
                runtime,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            active_exception = 1;
        }
        ptn_object_release(assignment_receiver);
        if (!coerced || receiver_invalidated || active_exception) {
            ptn_value_destroy(&stored);
            ptn_array_key_free(key);
            free(storage_key);
            PTN_OBJECT_WRITE_RETURN(ptn_null());
        }
    }
    if (metadata == NULL) {
        stored = ptn_value_clone_deref(value);
    }
    PtnValue result = ptn_value_clone(stored);
    if (mutable_metadata != NULL) {
        mutable_metadata->is_unset = 0;
        ptn_object_metadata_remember_value_type(mutable_metadata, stored);
        if (metadata != NULL &&
            metadata->is_readonly &&
            receiver.as.object->readonly_clone_initializing) {
            mutable_metadata->readonly_clone_reinitialized = 1;
        }
    }
    if (entry != NULL && entry->value.type == PTN_REFERENCE) {
        ptn_array_update_next_auto_key(receiver.as.object->properties, key);
        ptn_reference_assign_publish_first(runtime, entry->value.as.reference, stored);
        ptn_value_destroy(&stored);
        ptn_array_key_free(key);
    } else {
        ptn_array_set_entry_publish_first(receiver.as.object->properties, key, stored);
    }
    ptn_lazy_object_sync_proxy_instance_properties(receiver.as.object);
    free(storage_key);
    PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE();
#undef PTN_OBJECT_WRITE_RETURN
#undef PTN_OBJECT_WRITE_CLEANUP_LAZY_VALUE
    return result;
}

static PTN_UNUSED PtnValue ptn_object_write_property_with_mode_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    const char *access_scope,
    PtnValue value,
    size_t line,
    int indirect_write
) {
    return ptn_object_write_property_with_mode_len_impl(
        runtime,
        receiver,
        property,
        property_len,
        access_scope,
        value,
        line,
        indirect_write,
        0
    );
}

static PTN_UNUSED PtnValue ptn_object_write_property_with_mode(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line,
    int indirect_write
) {
    return ptn_object_write_property_with_mode_len(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        access_scope,
        value,
        line,
        indirect_write
    );
}

static PTN_UNUSED PtnValue ptn_object_write_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_object_write_property_with_mode(
        runtime,
        receiver,
        property,
        access_scope,
        value,
        line,
        0
    );
}

static PTN_UNUSED PtnValue ptn_object_write_property_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_object_write_property_with_mode_len(
        runtime,
        receiver,
        property,
        property_len,
        access_scope,
        value,
        line,
        0
    );
}

static PTN_UNUSED int ptn_runtime_has_active_exception(PtnRuntime *runtime) {
    return runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL;
}

static PTN_UNUSED void ptn_object_reset_readonly_clone_reinitialization(PtnObject *object) {
    if (object == NULL) {
        return;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        object->property_metadata[i].readonly_clone_reinitialized = 0;
    }
}

static PTN_UNUSED char *ptn_clone_with_int_property_name(int64_t integer) {
    int needed = snprintf(NULL, 0, "%lld", (long long)integer);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *name = malloc((size_t)needed + 1);
    if (name == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(name, (size_t)needed + 1, "%lld", (long long)integer);
    return name;
}

static PTN_UNUSED char *ptn_clone_with_property_name_from_key(
    PtnRuntime *runtime,
    PtnArrayKey key,
    size_t line
) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_clone_with_int_property_name(key.as.integer);
    }
    if (key.string_len > 0 && key.as.string[0] == '\0') {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Cannot access property starting with \"\\0\"",
            runtime == NULL ? NULL : runtime->source_path,
            line
        );
        return NULL;
    }
    PtnString key_string;
    key_string.data = (const unsigned char *)key.as.string;
    key_string.len = key.string_len;
    key_string.payload = NULL;
    if (ptn_string_has_embedded_nul(key_string)) {
        if (runtime != NULL) {
            ptn_emit_type_error(
                &runtime->diagnostics,
                "Unsupported dynamic property name containing embedded NUL"
            );
        }
        exit(255);
    }
    return ptn_duplicate_string_len(key.as.string, key.string_len);
}

static PTN_UNUSED PtnValue ptn_clone_value_with_properties(
    PtnRuntime *runtime,
    PtnValue value,
    PtnValue with_properties,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type != PTN_OBJECT) {
        return ptn_clone_value(runtime, value, line);
    }
    PtnValue properties = ptn_value_deref(with_properties);
    if (properties.type != PTN_ARRAY) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "clone(): Argument #2 ($withProperties) must be of type array, %s given",
            ptn_offset_container_type_name(properties)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }

    PtnValue clone = ptn_clone_value(runtime, value, line);
    if (ptn_runtime_has_active_exception(runtime) || clone.type != PTN_OBJECT) {
        return clone;
    }

    PtnObject *cloned = clone.as.object;
    ptn_object_reset_readonly_clone_reinitialization(cloned);
    int previous_clone_initializing = cloned->readonly_clone_initializing;
    cloned->readonly_clone_initializing = 1;
    for (size_t i = 0; i < properties.as.array->len; i++) {
        PtnArrayEntry *entry = &properties.as.array->entries[i];
        char *property_name = ptn_clone_with_property_name_from_key(runtime, entry->key, line);
        if (ptn_runtime_has_active_exception(runtime)) {
            cloned->readonly_clone_initializing = previous_clone_initializing;
            ptn_value_destroy(&clone);
            free(property_name);
            return ptn_null();
        }
        if (entry->value.type == PTN_REFERENCE &&
            entry->value.as.reference != NULL &&
            entry->value.as.reference->refcount > 1) {
            cloned->readonly_clone_initializing = previous_clone_initializing;
            free(property_name);
            ptn_value_destroy(&clone);
            ptn_throw_exception(
                runtime,
                "Error",
                "Cannot assign by reference when cloning with updated properties"
            );
            return ptn_null();
        }
        PtnValue property_value = ptn_value_clone_deref(entry->value);
        PtnValue written = ptn_object_write_property(
            runtime,
            clone,
            property_name,
            runtime == NULL ? NULL : runtime->current_class_name,
            property_value,
            line
        );
        ptn_value_destroy(&written);
        ptn_value_destroy(&property_value);
        free(property_name);
        if (ptn_runtime_has_active_exception(runtime)) {
            cloned->readonly_clone_initializing = previous_clone_initializing;
            ptn_value_destroy(&clone);
            return ptn_null();
        }
    }
    cloned->readonly_clone_initializing = previous_clone_initializing;
    return clone;
}

static PTN_UNUSED PtnValue ptn_object_write_property_indirect(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_object_write_property_with_mode(
        runtime,
        receiver,
        property,
        access_scope,
        value,
        line,
        1
    );
}

static PTN_UNUSED PtnValue ptn_object_write_property_indirect_notice_state(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line,
    int overloaded_notice_already_emitted
) {
    return ptn_object_write_property_with_mode_len_impl(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        access_scope,
        value,
        line,
        1,
        overloaded_notice_already_emitted
    );
}

static PTN_UNUSED void ptn_object_bind_property_reference(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue reference,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        ptn_throw_property_modification_on_non_object(runtime, property, receiver, line);
        return;
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_throw_incomplete_object_property_modification(runtime, receiver.as.object, line);
        return;
    }
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    if (ptn_lazy_object_property_reference_needs_initialization(
            runtime,
            receiver,
            property,
            access_scope,
            line
        ) &&
        !ptn_lazy_object_initialize(runtime, receiver, line)) {
        return;
    }
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL &&
        ptn_magic_property_get_exists_inactive(runtime, receiver, property)) {
        ptn_call_magic_get_then_throw_overloaded_property_reference_error(
            runtime,
            receiver,
            property,
            line,
            1
        );
        return;
    }
    ptn_emit_static_property_non_static_notice_if_accessible(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_WRITE,
        line
    );
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_WRITE,
        0,
        line
    );
    if (storage_key == NULL) {
        return;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    if (metadata == NULL &&
        entry == NULL &&
        ptn_object_metadata_for_display_name(receiver.as.object, property) == NULL &&
        ptn_magic_property_get_exists_inactive(runtime, receiver, property)) {
        if (ptn_magic_property_get_has_active_frame(runtime)) {
            if (!ptn_object_emit_dynamic_property_creation_deprecation(
                    runtime,
                    receiver.as.object,
                    property,
                    line,
                    0
                )) {
                ptn_array_key_free(key);
                free(storage_key);
                return;
            }
        }
        ptn_array_key_free(key);
        free(storage_key);
        ptn_call_magic_get_then_throw_overloaded_property_reference_error(
            runtime,
            receiver,
            property,
            line,
            1
        );
        return;
    }
    if (metadata != NULL && metadata->is_readonly) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_readonly_property_indirect_modification_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name,
            line
        );
        return;
    }
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        runtime != NULL &&
        runtime->property_hook_get != NULL
    ) {
        PtnValue hook_value = ptn_null();
        if (runtime->property_hook_get(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            line,
            &hook_value
        )) {
            ptn_declared_class_property_hook_deprecation(
                runtime,
                hook_declaring_class,
                metadata->display_name,
                1,
                line
            );
            ptn_value_destroy(&hook_value);
            ptn_array_key_free(key);
            free(storage_key);
            ptn_throw_overloaded_property_reference_error(runtime, line);
            return;
        }
    }
    if (ptn_property_is_get_only_virtual(metadata)) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_get_only_virtual_property_write_error(
            runtime,
            metadata,
            receiver.as.object->class_name,
            line
        );
        return;
    }
    if (metadata != NULL && metadata->is_readonly && entry != NULL) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_readonly_property_indirect_modification_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name,
            line
        );
        return;
    }
    if (metadata != NULL && ptn_property_type_is_declared(metadata->type_kind)) {
        PtnValue coerced = ptn_null();
        if (!ptn_property_type_coerce_assignment(
            runtime,
            metadata->type_kind,
            metadata->type_class_name,
            metadata->type_text,
            metadata->type_allows_null,
            metadata->declaring_class,
            metadata->display_name,
            reference,
            0,
            line,
            &coerced
        )) {
            ptn_array_key_free(key);
            free(storage_key);
            return;
        }
        if (reference.as.reference->property_type_kind != PTN_PROPERTY_TYPE_NONE) {
            PtnValue existing_coerced = ptn_null();
            if (!ptn_property_reference_coerce_assignment(
                runtime,
                reference.as.reference,
                reference,
                1,
                line,
                &existing_coerced
            )) {
                ptn_value_destroy(&coerced);
                ptn_array_key_free(key);
                free(storage_key);
                return;
            }
            if (!ptn_compare_identical(runtime, existing_coerced, coerced, line)) {
                PtnReferencePropertyTypeSource existing =
                    ptn_reference_primary_property_type_source(reference.as.reference);
                ptn_throw_reference_property_bind_incompatibility(
                    runtime,
                    reference.as.reference->value,
                    &existing,
                    metadata
                );
                ptn_value_destroy(&existing_coerced);
                ptn_value_destroy(&coerced);
                ptn_array_key_free(key);
                free(storage_key);
                return;
            }
            ptn_value_destroy(&existing_coerced);
        }
        ptn_value_destroy(&reference.as.reference->value);
        reference.as.reference->value = coerced;
        ptn_reference_adopt_property_type(reference.as.reference, metadata);
    }
    ptn_array_set_entry_publish_first(receiver.as.object->properties, key, ptn_value_clone(reference));
    ptn_lazy_object_sync_proxy_instance_properties(receiver.as.object);
    PtnObjectPropertyMetadata *mutable_metadata =
        ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
    if (mutable_metadata != NULL) {
        mutable_metadata->is_unset = 0;
        ptn_object_metadata_remember_value_type(mutable_metadata, reference);
    }
    free(storage_key);
}

static PTN_UNUSED PtnValue ptn_object_reference_for_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        ptn_throw_property_modification_on_non_object(runtime, property, receiver, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (ptn_lazy_object_property_reference_needs_initialization(
            runtime,
            receiver,
            property,
            access_scope,
            line
        ) &&
        !ptn_lazy_object_initialize(runtime, receiver, line)) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    PtnValue original_receiver = receiver;
    if (receiver.as.object->lazy_is_proxy && !receiver.as.object->lazy_uninitialized) {
        receiver = ptn_lazy_object_effective_initialized_proxy_receiver_for_access(
            runtime,
            receiver,
            line
        );
        if (receiver.type != PTN_OBJECT || receiver.as.object == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }
    int reference_fetch_forwarded_from_initialized_proxy =
        original_receiver.type == PTN_OBJECT &&
        original_receiver.as.object != receiver.as.object &&
        original_receiver.as.object->lazy_is_proxy &&
        !original_receiver.as.object->lazy_uninitialized;
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_xml_value = ptn_null();
    if (ptn_internal_xml_property_read(
        runtime,
        receiver,
        property,
        line,
        &internal_xml_value
    )) {
        return ptn_reference_value(ptn_reference_new_owned(internal_xml_value));
    }
#endif
    PtnValue magic_receiver = ptn_lazy_object_effective_initialized_proxy_receiver(receiver);
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, magic_receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL &&
        ptn_magic_property_get_exists_inactive(runtime, magic_receiver, property)) {
        ptn_throw_overloaded_property_reference_error(runtime, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    ptn_emit_static_property_non_static_notice_if_accessible(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
        line
    );
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
        1,
        line
    );
    if (storage_key == NULL) {
        char *read_storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            1,
            line
        );
        if (read_storage_key != NULL) {
            PtnArrayKey read_key = ptn_array_string_key(read_storage_key);
            PtnArrayEntry *read_entry =
                ptn_array_entry_for_key(receiver.as.object->properties, read_key);
            ptn_array_key_free(read_key);
            if (read_entry != NULL) {
                const PtnObjectPropertyMetadata *read_metadata =
                    ptn_object_property_metadata(receiver.as.object, read_storage_key);
                if (
                    read_metadata != NULL &&
                    !read_metadata->is_readonly &&
                    ptn_property_visibility_allows(
                        runtime,
                        read_metadata->set_visibility,
                        ptn_property_visibility_scope_class(read_metadata, read_metadata->set_visibility),
                        access_scope
                    )
                ) {
                    if (read_entry->value.type != PTN_REFERENCE) {
                        PtnValue current = read_entry->value;
                        read_entry->value = ptn_reference_value(ptn_reference_new_owned(current));
                    }
                    ptn_reference_adopt_property_type(read_entry->value.as.reference, read_metadata);
                    if (ptn_reference_created_in_active_property_hook(runtime, receiver.as.object)) {
                        ptn_reference_remember_property_identity(
                            read_entry->value.as.reference,
                            read_metadata
                        );
                    }
                    PtnValue reference = ptn_value_clone(read_entry->value);
                    ptn_lazy_object_sync_forwarded_proxy_property_reference(
                        original_receiver,
                        receiver,
                        read_storage_key,
                        reference
                    );
                    free(read_storage_key);
                    return reference;
                }
                PtnValue current = ptn_value_clone_deref(read_entry->value);
                if (current.type == PTN_OBJECT) {
                    free(read_storage_key);
                    return ptn_reference_value(ptn_reference_new_owned(current));
                }
                ptn_value_destroy(&current);
            }
            free(read_storage_key);
        }
        storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
            0,
            line
        );
        if (storage_key == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    const char *hook_declaring_class = ptn_property_hook_get_declaring_class(metadata);
    int active_same_property_hook = ptn_active_property_hook_matches(
        runtime,
        receiver.as.object,
        metadata,
        hook_declaring_class,
        access_scope,
        property
    );
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        !metadata->hook_get_returns_by_ref &&
        !active_same_property_hook
    ) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_create_reference_to_property_error(runtime, metadata, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (
        metadata != NULL &&
        metadata->hook_has_get &&
        !active_same_property_hook &&
        runtime != NULL &&
        runtime->property_hook_get != NULL
    ) {
        PtnValue hook_value = ptn_null();
        if (runtime->property_hook_get(
            runtime,
            receiver,
            hook_declaring_class,
            metadata->display_name,
            line,
            &hook_value
        )) {
            ptn_declared_class_property_hook_deprecation(
                runtime,
                hook_declaring_class,
                metadata->display_name,
                1,
                line
            );
            ptn_array_key_free(key);
            free(storage_key);
            if (hook_value.type == PTN_REFERENCE) {
                const char *forwarded_storage_key =
                    ptn_reference_property_storage_key_for_object(
                        receiver.as.object,
                        hook_value.as.reference
                    );
                if (forwarded_storage_key != NULL) {
                    ptn_lazy_object_sync_forwarded_proxy_property_reference(
                        original_receiver,
                        receiver,
                        forwarded_storage_key,
                        hook_value
                    );
                }
                return hook_value;
            }
            ptn_value_destroy(&hook_value);
            ptn_throw_hooked_property_indirect_modification_error(runtime, metadata, line);
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }
    if (
        metadata != NULL &&
        metadata->has_hooks &&
        !metadata->hook_has_get &&
        !active_same_property_hook
    ) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_hooked_property_indirect_modification_error(runtime, metadata, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (ptn_property_is_get_only_virtual(metadata)) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_get_only_virtual_property_write_error(
            runtime,
            metadata,
            receiver.as.object->class_name,
            line
        );
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (entry == NULL &&
        metadata != NULL &&
        metadata->is_unset &&
        ptn_property_type_is_declared(metadata->type_kind)) {
        PtnValue magic_value = ptn_null();
        if (ptn_magic_property_get(runtime, magic_receiver, property, line, &magic_value)) {
            if (ptn_runtime_has_active_exception(runtime)) {
                ptn_value_destroy(&magic_value);
                ptn_array_key_free(key);
                free(storage_key);
                return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
            }
            if (magic_value.type == PTN_REFERENCE) {
                PtnValue coerced = ptn_null();
                if (!ptn_coerce_unset_typed_property_magic_value(
                    runtime,
                    magic_receiver,
                    metadata,
                    magic_value,
                    line,
                    &coerced
                )) {
                    ptn_value_destroy(&magic_value);
                    ptn_array_key_free(key);
                    free(storage_key);
                    return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
                }
                ptn_reference_assign_publish_first(runtime, magic_value.as.reference, coerced);
                ptn_value_destroy(&coerced);
                ptn_array_key_free(key);
                free(storage_key);
                return magic_value;
            }
            ptn_value_destroy(&magic_value);
        }
    }
    if (metadata == NULL && entry == NULL) {
        PtnValue magic_value = ptn_null();
        if (ptn_magic_property_get(runtime, magic_receiver, property, line, &magic_value)) {
            if (ptn_runtime_has_active_exception(runtime)) {
                ptn_value_destroy(&magic_value);
                ptn_array_key_free(key);
                free(storage_key);
                return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
            }
            if (magic_value.type == PTN_REFERENCE) {
                ptn_array_key_free(key);
                free(storage_key);
                return magic_value;
            }
            PtnValue current = ptn_value_clone_deref(magic_value);
            if (current.type != PTN_OBJECT && current.type != PTN_EXCEPTION) {
                ptn_emit_indirect_modification_overloaded_property_notice(
                    runtime,
                    magic_receiver,
                    property,
                    line
                );
            }
            ptn_value_destroy(&magic_value);
            ptn_array_key_free(key);
            free(storage_key);
            return ptn_reference_value(ptn_reference_new_owned(current));
        }
    }
    if (metadata != NULL && metadata->is_readonly && entry != NULL) {
        if (ptn_object_property_is_date_period_internal(runtime, receiver.as.object, metadata)) {
            ptn_array_key_free(key);
            free(storage_key);
            ptn_throw_readonly_property_error(
                runtime,
                receiver.as.object->class_name,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        if (ptn_ascii_case_equal(receiver.as.object->class_name, "BcMath\\Number")) {
            PtnValue current = ptn_value_clone_deref(entry->value);
            ptn_array_key_free(key);
            free(storage_key);
            return ptn_reference_value(ptn_reference_new_owned(current));
        }
        PtnValue current = ptn_value_clone_deref(entry->value);
        if (current.type == PTN_OBJECT) {
            ptn_array_key_free(key);
            free(storage_key);
            return ptn_reference_value(ptn_reference_new_owned(current));
        }
        ptn_value_destroy(&current);
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_readonly_property_indirect_modification_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name,
            line
        );
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (entry == NULL) {
        if (metadata == NULL &&
            reference_fetch_forwarded_from_initialized_proxy &&
            ptn_magic_property_is_active_on_receiver(
                runtime,
                receiver,
                property,
                PTN_MAGIC_PROPERTY_GET
            )) {
            ptn_emit_undefined_property_warning(runtime, receiver.as.object, property, line);
            ptn_array_key_free(key);
            free(storage_key);
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        if (metadata == NULL &&
            !ptn_object_emit_dynamic_property_creation_deprecation(
                runtime,
                receiver.as.object,
                property,
                line,
                0
            )) {
            ptn_array_key_free(key);
            free(storage_key);
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        if (metadata != NULL &&
            ptn_property_type_is_declared(metadata->type_kind) &&
            !metadata->type_allows_null) {
            ptn_array_key_free(key);
            free(storage_key);
            ptn_throw_uninitialized_typed_property_reference_error(
                runtime,
                metadata->declaring_class,
                metadata->display_name,
                line
            );
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        PtnValue reference = ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        if (metadata != NULL) {
            ptn_reference_adopt_property_type(reference.as.reference, metadata);
            if (ptn_reference_created_in_active_property_hook(runtime, receiver.as.object)) {
                ptn_reference_remember_property_identity(reference.as.reference, metadata);
            }
        }
        ptn_array_set_entry(receiver.as.object->properties, key, ptn_value_clone(reference));
        PtnObjectPropertyMetadata *mutable_metadata =
            ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
        if (mutable_metadata != NULL) {
            mutable_metadata->is_unset = 0;
            ptn_object_metadata_remember_value_type(mutable_metadata, reference);
        }
        ptn_lazy_object_sync_forwarded_proxy_property_reference(
            original_receiver,
            receiver,
            storage_key,
            reference
        );
        ptn_lazy_object_sync_proxy_instance_properties(receiver.as.object);
        free(storage_key);
        return reference;
    }
    ptn_array_key_free(key);
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    if (metadata != NULL) {
        ptn_reference_adopt_property_type(entry->value.as.reference, metadata);
        if (ptn_reference_created_in_active_property_hook(runtime, receiver.as.object)) {
            ptn_reference_remember_property_identity(entry->value.as.reference, metadata);
        }
    }
    ptn_lazy_object_sync_forwarded_proxy_property_reference(
        original_receiver,
        receiver,
        storage_key,
        entry->value
    );
    ptn_lazy_object_sync_proxy_instance_properties(receiver.as.object);
    free(storage_key);
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_object_unset_property_len(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t property_len,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        if (ptn_value_is_from_string_offset(receiver)) {
            ptn_throw_string_offset_as_object_error(runtime, line);
        }
        return;
    }
    if (ptn_object_is_incomplete_class(receiver.as.object)) {
        ptn_throw_incomplete_object_property_modification(runtime, receiver.as.object, line);
        return;
    }
    if (receiver.as.object->lazy_uninitialized &&
        !receiver.as.object->lazy_initializing) {
        int lazy_local_unset_slot = ptn_lazy_object_property_access_uses_local_slot(
            runtime,
            receiver,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_UNSET,
            line,
            NULL
        );
        int lazy_magic_unset_dispatch =
            ptn_uninitialized_lazy_object_magic_dispatch_can_skip_initialization(
                runtime,
                receiver,
                "__unset"
            ) &&
            !ptn_magic_property_is_active_len(
                runtime,
                receiver,
                property,
                property_len,
                PTN_MAGIC_PROPERTY_UNSET
            );
        if (!lazy_local_unset_slot &&
            !lazy_magic_unset_dispatch &&
            !ptn_lazy_object_initialize(runtime, receiver, line)) {
            return;
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_value = ptn_null();
    if (ptn_internal_xml_property_read(runtime, receiver, property, line, &internal_value)) {
        ptn_value_destroy(&internal_value);
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Cannot unset %s::$%s",
            receiver.as.object->class_name,
            property
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return;
    }
    if (ptn_internal_array_object_property_unset(runtime, receiver, property, access_scope, line)) {
        return;
    }
#endif
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL && blocked_metadata->is_unset) {
        if (ptn_magic_property_unset_len(runtime, receiver, property, property_len, line)) {
            return;
        }
    }
    if (blocked_metadata != NULL &&
        blocked_metadata->set_visibility != blocked_metadata->read_visibility &&
        !ptn_readonly_property_storage_initialized(receiver.as.object, blocked_metadata)) {
        ptn_throw_property_unset_visibility_error(
            runtime,
            blocked_metadata->set_visibility,
            blocked_metadata->declaring_class,
            property,
            access_scope,
            1,
            blocked_metadata->is_readonly
        );
        return;
    }
    if (blocked_metadata == NULL &&
        ptn_object_metadata_for_display_name(receiver.as.object, property) == NULL &&
        ptn_magic_property_unset_len(runtime, receiver, property, property_len, line)) {
        return;
    }
    if (property_len > 0 && property[0] == '\0') {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Cannot access property starting with \"\\0\"",
            runtime == NULL ? NULL : runtime->source_path,
            line
        );
        return;
    }
    ptn_emit_static_property_non_static_notice_if_accessible(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_UNSET,
        line
    );
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_UNSET,
        0,
        line
    );
    if (storage_key == NULL) {
        return;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    PtnObjectPropertyMetadata *mutable_metadata =
        ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
    if (mutable_metadata != NULL && mutable_metadata->has_hooks) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_hooked_property_unset_error(
            runtime,
            mutable_metadata,
            receiver.as.object->class_name,
            line
        );
        return;
    }
    if (ptn_object_property_is_date_period_internal(runtime, receiver.as.object, mutable_metadata)) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_date_period_internal_property_unset_error(
            runtime,
            receiver.as.object->class_name,
            mutable_metadata->display_name,
            line
        );
        return;
    }
    if (mutable_metadata != NULL && mutable_metadata->is_readonly && entry != NULL) {
        int readonly_clone_unset =
            receiver.as.object->readonly_clone_initializing &&
            !mutable_metadata->readonly_clone_reinitialized;
        if (!readonly_clone_unset) {
            ptn_array_key_free(key);
            free(storage_key);
            ptn_throw_readonly_property_unset_error(
                runtime,
                mutable_metadata->declaring_class,
                mutable_metadata->display_name,
                line
            );
            return;
        }
    }
    if (entry != NULL && entry->value.type == PTN_REFERENCE && mutable_metadata != NULL) {
        ptn_reference_forget_property_type(entry->value.as.reference, mutable_metadata);
    }
    int active_value_unset = entry != NULL;
    if (active_value_unset) {
        if (receiver.as.object->active_property_value_unsets == SIZE_MAX) {
            ptn_abort_out_of_memory();
        }
        receiver.as.object->active_property_value_unsets++;
    }
    ptn_array_unset_entry(receiver.as.object->properties, key);
    if (active_value_unset && receiver.as.object->active_property_value_unsets > 0) {
        receiver.as.object->active_property_value_unsets--;
    }
    if (mutable_metadata != NULL) {
        mutable_metadata->is_unset = 1;
        if (
            !mutable_metadata->is_readonly &&
            mutable_metadata->read_visibility == mutable_metadata->set_visibility
        ) {
            free(mutable_metadata->last_type_name);
            mutable_metadata->last_type_name = NULL;
        }
    }
    free(storage_key);
}

static PTN_UNUSED void ptn_object_unset_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    ptn_object_unset_property_len(
        runtime,
        receiver,
        property,
        property == NULL ? 0 : strlen(property),
        access_scope,
        line
    );
}

static PTN_UNUSED PtnValue ptn_object_declare_property_with_hooks(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    int is_readonly,
    int has_hooks,
    int is_virtual,
    int hook_has_get,
    int hook_get_returns_by_ref,
    int hook_has_set,
    const char *hook_get_declaring_class,
    const char *hook_set_declaring_class,
    PtnPropertyTypeKind type_kind,
    const char *type_class_name,
    const char *type_text,
    int type_allows_null,
    int has_value,
    PtnValue value,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_OBJECT) {
        ptn_object_register_property_metadata(
            receiver.as.object,
            property,
            declaring_class,
            read_visibility,
            set_visibility,
            is_readonly,
            has_hooks,
            is_virtual,
            hook_has_get,
            hook_get_returns_by_ref,
            hook_has_set,
            hook_get_declaring_class,
            hook_set_declaring_class,
            type_kind,
            type_class_name,
            type_text,
            type_allows_null
        );
    }
    if (!has_value) {
        if (receiver.type == PTN_OBJECT && !is_virtual) {
            char *storage_key = ptn_object_resolve_property_storage_key(
                runtime,
                receiver.as.object,
                property,
                declaring_class,
                PTN_PROPERTY_ACCESS_WRITE,
                1,
                line
            );
            if (storage_key != NULL) {
                PtnArrayKey key = ptn_array_string_key(storage_key);
                PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
                PtnObjectPropertyMetadata *mutable_metadata =
                    ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
                if (entry != NULL && entry->value.type == PTN_REFERENCE && mutable_metadata != NULL) {
                    ptn_reference_forget_property_type(entry->value.as.reference, mutable_metadata);
                }
                ptn_array_unset_entry(receiver.as.object->properties, key);
                if (mutable_metadata != NULL) {
                    mutable_metadata->is_unset = 0;
                    free(mutable_metadata->last_type_name);
                    mutable_metadata->last_type_name = NULL;
                }
                free(storage_key);
            }
        }
        return ptn_null();
    }
    const char *previous_active_property_hook_class =
        runtime == NULL ? NULL : runtime->active_property_hook_class;
    const char *previous_active_property_hook_property =
        runtime == NULL ? NULL : runtime->active_property_hook_property;
    PtnObject *previous_active_property_hook_object =
        runtime == NULL ? NULL : runtime->active_property_hook_object;
    if (runtime != NULL && has_hooks) {
        runtime->active_property_hook_class =
            hook_set_declaring_class != NULL ? hook_set_declaring_class : declaring_class;
        runtime->active_property_hook_property = property;
        runtime->active_property_hook_object =
            receiver.type == PTN_OBJECT ? receiver.as.object : NULL;
    }
    PtnValue declared = ptn_object_write_property(runtime, receiver, property, declaring_class, value, line);
    if (runtime != NULL && has_hooks) {
        runtime->active_property_hook_class = previous_active_property_hook_class;
        runtime->active_property_hook_property = previous_active_property_hook_property;
        runtime->active_property_hook_object = previous_active_property_hook_object;
    }
    return declared;
}

static PTN_UNUSED PtnValue ptn_object_declare_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    int is_readonly,
    PtnPropertyTypeKind type_kind,
    const char *type_class_name,
    const char *type_text,
    int type_allows_null,
    int has_value,
    PtnValue value,
    size_t line
) {
    return ptn_object_declare_property_with_hooks(
        runtime,
        receiver,
        property,
        declaring_class,
        read_visibility,
        set_visibility,
        is_readonly,
        0,
        0,
        0,
        0,
        0,
        NULL,
        NULL,
        type_kind,
        type_class_name,
        type_text,
        type_allows_null,
        has_value,
        value,
        line
    );
}

static PTN_UNUSED void ptn_emit_null_array_offset_deprecation(PtnRuntime *runtime, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    runtime->diagnostics.emitted_deprecation = 1;
    const char *message = "Using null as an array offset is deprecated, use an empty string instead";
    if (ptn_diagnostics_try_error_handler(
        &runtime->diagnostics,
        PTN_E_DEPRECATED,
        message,
        runtime->source_path,
        line
    )) {
        return;
    }
    ptn_diagnostic_printf(
        &runtime->diagnostics,
        "\nDeprecated: Using null as an array offset is deprecated, use an empty string instead in %s on line %zu\n",
        ptn_array_runtime_diagnostic_path(runtime),
        line
    );
}

static PTN_UNUSED void ptn_emit_array_offset_key_conversion_diagnostic(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int emit_null_key_deprecation
) {
    key_value = ptn_value_deref(key_value);
    if (key_value.type == PTN_NULL) {
        if (emit_null_key_deprecation) {
            ptn_emit_null_array_offset_deprecation(runtime, line);
        }
    } else if (key_value.type == PTN_RESOURCE) {
        ptn_emit_resource_offset_warning(runtime, key_value.as.resource, line);
    } else if (key_value.type == PTN_FLOAT) {
        if (ptn_float_to_int_out_of_range(key_value.as.floating)) {
            ptn_emit_bitwise_float_out_of_range_warning(&runtime->diagnostics, key_value.as.floating, line);
        } else if (ptn_float_to_int_loses_precision(key_value.as.floating)) {
            ptn_emit_float_to_int_precision_deprecation_at(
                &runtime->diagnostics,
                key_value.as.floating,
                runtime->source_path == NULL ? "ptn" : runtime->source_path,
                line
            );
        }
    }
}

static PTN_UNUSED void ptn_emit_false_array_conversion_deprecation(PtnRuntime *runtime, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    runtime->diagnostics.emitted_deprecation = 1;
    const char *message = "Automatic conversion of false to array is deprecated";
    if (ptn_diagnostics_try_error_handler(
        &runtime->diagnostics,
        PTN_E_DEPRECATED,
        message,
        runtime->source_path,
        line
    )) {
        return;
    }
    ptn_emit_array_runtime_diagnostic(runtime, "Deprecated", message, line);
}

static PTN_UNUSED PtnArray *ptn_array_convertible_scalar_for_write(
    PtnRuntime *runtime,
    PtnValue *value,
    size_t line
) {
    if (value->type == PTN_NULL) {
        return ptn_value_replace_with_empty_array(value);
    }
    if (value->type == PTN_BOOL && !value->as.boolean) {
        (void)ptn_value_replace_with_empty_array(value);
        ptn_emit_false_array_conversion_deprecation(runtime, line);
        return ptn_array_detach_value(value);
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_foreach_operand_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_BOOL) {
        return value.as.boolean ? "true" : "false";
    }
    return ptn_offset_container_type_name(value);
}

static PTN_UNUSED void ptn_emit_foreach_non_array_warning(
    PtnRuntime *runtime,
    PtnValue value,
    const char *path,
    size_t line
) {
    if (runtime != NULL && !ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    char message[128];
    snprintf(
        message,
        sizeof(message),
        "foreach() argument must be of type array|object, %s given",
        ptn_foreach_operand_type_name(value)
    );
    if (runtime != NULL) {
        runtime->diagnostics.emitted_warning = 1;
        if (ptn_diagnostics_try_error_handler(
            &runtime->diagnostics,
            PTN_E_WARNING,
            message,
            path,
            line
        )) {
            return;
        }
    }
    ptn_emit_array_runtime_diagnostic_at_path(runtime, "Warning", message, path, line);
}

static PTN_UNUSED PtnArray *ptn_runtime_array_for_reference_write(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    PtnValue *slot = ptn_symbols_value_slot(symbols, name);
    if (slot == NULL) {
        PtnValue array = ptn_array_from_literal_entries(0, NULL);
        ptn_runtime_write_variable(runtime, name, array);
        ptn_value_destroy(&array);
        slot = ptn_symbols_value_slot(symbols, name);
        if (slot == NULL) {
            return NULL;
        }
    }

    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_ARRAY) {
        PtnArray *array = ptn_runtime_array_detach_variable(runtime, name);
        return array != NULL ? array : value->as.array;
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(runtime, value, line);
    if (converted != NULL) {
        return converted;
    }
    if (value->type == PTN_STRING) {
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return NULL;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, *value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *value, line);
        return NULL;
    }

    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED int ptn_array_append_key_available(PtnRuntime *runtime, PtnArray *array) {
    if (ptn_array_find_key(array, ptn_array_int_key(INT64_MAX)) >= array->len) {
        return 1;
    }
    ptn_throw_exception(
        runtime,
        "Error",
        "Cannot add element to the array as the next element is already occupied"
    );
    return 0;
}

static PTN_UNUSED void ptn_array_literal_append_entry(
    PtnRuntime *runtime,
    PtnArray *array,
    size_t line,
    int has_key,
    PtnValue key_value,
    PtnValue value
) {
    if (has_key) {
        PtnValue key_value_deref = ptn_value_deref(key_value);
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, key_value_deref, line, 1);
        PtnArrayKey key = ptn_array_key_from_value(key_value_deref);
        ptn_array_set_entry(array, key, ptn_value_clone(value));
        return;
    }

    if (!ptn_array_append_key_available(runtime, array)) {
        return;
    }
    PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
    ptn_array_set_entry(array, key, ptn_value_clone(value));
}

static PTN_UNUSED void ptn_generator_data_free(void *data) {
    PtnGenerator *generator = (PtnGenerator *)data;
    if (generator == NULL) {
        return;
    }
    if (generator->values != NULL) {
        ptn_array_free(generator->values);
    }
    if (generator->keys != NULL) {
        ptn_array_free(generator->keys);
    }
    ptn_value_destroy(&generator->return_value);
    if (generator->reference_notice_lines != NULL) {
        ptn_array_free(generator->reference_notice_lines);
    }
    if (generator->yield_lines != NULL) {
        ptn_array_free(generator->yield_lines);
    }
    if (generator->delegate_sources != NULL) {
        ptn_array_free(generator->delegate_sources);
    }
    if (generator->force_close_yield_from_entries != NULL) {
        ptn_array_free(generator->force_close_yield_from_entries);
    }
    if (generator->output_chunks != NULL) {
        ptn_array_free(generator->output_chunks);
    }
    if (generator->send_call_positions != NULL) {
        ptn_array_free(generator->send_call_positions);
    }
    if (generator->send_call_kinds != NULL) {
        ptn_array_free(generator->send_call_kinds);
    }
    if (generator->send_call_names != NULL) {
        ptn_array_free(generator->send_call_names);
    }
    if (generator->send_call_receivers != NULL) {
        ptn_array_free(generator->send_call_receivers);
    }
    if (generator->send_call_arguments != NULL) {
        ptn_array_free(generator->send_call_arguments);
    }
    if (generator->send_call_yield_indexes != NULL) {
        ptn_array_free(generator->send_call_yield_indexes);
    }
    if (generator->send_call_lines != NULL) {
        ptn_array_free(generator->send_call_lines);
    }
    if (generator->send_yield_from_positions != NULL) {
        ptn_array_free(generator->send_yield_from_positions);
    }
    if (generator->send_yield_from_lines != NULL) {
        ptn_array_free(generator->send_yield_from_lines);
    }
    ptn_value_destroy(&generator->pending_exception);
    free(generator->pending_output.data);
    ptn_value_destroy(&generator->closure_owner);
    ptn_value_destroy(&generator->receiver);
    free(generator->function_name);
    free(generator->source_file);
    free(generator);
}

static PTN_UNUSED int ptn_object_is_generator(PtnObject *object) {
    return object != NULL &&
        object->native_data != NULL &&
        ptn_ascii_case_equal(object->class_name, "Generator");
}

static PTN_UNUSED PtnGenerator *ptn_generator_from_value(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT || !ptn_object_is_generator(value.as.object)) {
        return NULL;
    }
    return (PtnGenerator *)value.as.object->native_data;
}

static PTN_UNUSED PtnValue ptn_generator_new(PtnRuntime *runtime, int yields_by_ref) {
    PtnGenerator *generator = malloc(sizeof(PtnGenerator));
    if (generator == NULL) {
        ptn_abort_out_of_memory();
    }
    PtnValue values = ptn_array_from_literal_entries(0, NULL);
    PtnValue keys = ptn_array_from_literal_entries(0, NULL);
    PtnValue reference_notice_lines = ptn_array_from_literal_entries(0, NULL);
    PtnValue yield_lines = ptn_array_from_literal_entries(0, NULL);
    PtnValue delegate_sources = ptn_array_from_literal_entries(0, NULL);
    PtnValue force_close_yield_from_entries = ptn_array_from_literal_entries(0, NULL);
    PtnValue output_chunks = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_call_positions = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_call_kinds = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_call_names = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_call_receivers = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_call_arguments = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_call_yield_indexes = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_call_lines = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_yield_from_positions = ptn_array_from_literal_entries(0, NULL);
    PtnValue send_yield_from_lines = ptn_array_from_literal_entries(0, NULL);
    generator->values = values.as.array;
    generator->keys = keys.as.array;
    generator->object = NULL;
    generator->return_value = ptn_null();
    generator->reference_notice_lines = reference_notice_lines.as.array;
    generator->yield_lines = yield_lines.as.array;
    generator->delegate_sources = delegate_sources.as.array;
    generator->force_close_yield_from_entries = force_close_yield_from_entries.as.array;
    generator->output_chunks = output_chunks.as.array;
    generator->send_call_positions = send_call_positions.as.array;
    generator->send_call_kinds = send_call_kinds.as.array;
    generator->send_call_names = send_call_names.as.array;
    generator->send_call_receivers = send_call_receivers.as.array;
    generator->send_call_arguments = send_call_arguments.as.array;
    generator->send_call_yield_indexes = send_call_yield_indexes.as.array;
    generator->send_call_lines = send_call_lines.as.array;
    generator->send_yield_from_positions = send_yield_from_positions.as.array;
    generator->send_yield_from_lines = send_yield_from_lines.as.array;
    generator->pending_exception = ptn_null();
    generator->pending_exception_position = 0;
    generator->has_pending_exception = 0;
    generator->pending_exception_on_rewind = 0;
    generator->return_yield_position = 0;
    generator->has_return_yield_position = 0;
    ptn_string_buffer_init(&generator->pending_output);
    generator->closure_owner = ptn_null();
    generator->has_receiver = 0;
    generator->receiver = ptn_null();
    generator->function_name = NULL;
    generator->source_file = NULL;
    generator->source_line = 0;
    if (
        runtime != NULL &&
        runtime->owned_call_frame.has_current_closure &&
        ptn_value_deref(runtime->owned_call_frame.current_closure).type == PTN_CLOSURE
    ) {
        generator->closure_owner = ptn_value_share(runtime->owned_call_frame.current_closure);
    }
    if (runtime != NULL && runtime->has_current_receiver) {
        generator->has_receiver = 1;
        generator->receiver = ptn_value_clone_deref(runtime->current_receiver);
    }
    generator->position = 0;
    generator->next_auto_key = 0;
    generator->completed = 0;
    generator->started = 0;
    generator->executing = 0;
    generator->force_closing = 0;
    generator->yields_by_ref = yields_by_ref ? 1 : 0;

    PtnValue object = ptn_object_new_shell(runtime, "Generator");
    object.as.object->native_data = generator;
    object.as.object->native_data_free = ptn_generator_data_free;
    generator->object = object.as.object;
    const char *function_name = runtime == NULL || runtime->current_function_name == NULL
        ? "{unknown}"
        : runtime->current_function_name;
    char *owned_function_name = NULL;
    const char *prefix = "{closure:";
    size_t prefix_len = strlen(prefix);
    size_t function_name_len = strlen(function_name);
    if (
        runtime != NULL &&
        runtime->trace_frame != NULL &&
        runtime->trace_frame->previous != NULL &&
        runtime->trace_frame->previous->function_name != NULL &&
        function_name_len > prefix_len + 1 &&
        strncmp(function_name, prefix, prefix_len) == 0 &&
        function_name[function_name_len - 1] == '}'
    ) {
        const char *body_start = function_name + prefix_len;
        const char *body_end = function_name + function_name_len - 1;
        const char *line_start = body_end;
        while (line_start > body_start && line_start[-1] != ':') {
            line_start--;
        }
        size_t line = 0;
        int parsed_line = line_start > body_start;
        for (const char *cursor = line_start; parsed_line && cursor < body_end; cursor++) {
            if (*cursor < '0' || *cursor > '9') {
                parsed_line = 0;
                break;
            }
            size_t digit = (size_t)(*cursor - '0');
            if (line > (SIZE_MAX - digit) / 10) {
                parsed_line = 0;
                break;
            }
            line = line * 10 + digit;
        }
        if (parsed_line) {
            const char *caller_name = runtime->trace_frame->previous->function_name;
            int needed = snprintf(NULL, 0, "{closure:%s():%zu}", caller_name, line);
            if (needed < 0) {
                ptn_abort_out_of_memory();
            }
            owned_function_name = malloc((size_t)needed + 1);
            if (owned_function_name == NULL) {
                ptn_abort_out_of_memory();
            }
            snprintf(owned_function_name, (size_t)needed + 1, "{closure:%s():%zu}", caller_name, line);
            function_name = owned_function_name;
        }
    }
    generator->function_name = ptn_duplicate_string(function_name);
    if (runtime != NULL && runtime->trace_frame != NULL) {
        if (runtime->trace_frame->file != NULL) {
            generator->source_file = ptn_duplicate_string(runtime->trace_frame->file);
        }
        generator->source_line = runtime->trace_frame->line;
    } else if (runtime != NULL && runtime->source_path != NULL) {
        generator->source_file = ptn_duplicate_string(runtime->source_path);
    }
    PtnValue function_value = owned_function_name == NULL
        ? ptn_string(function_name)
        : ptn_owned_string(owned_function_name);
    PtnValue assigned = ptn_object_declare_property(
        runtime,
        object,
        "function",
        "Generator",
        PTN_PROPERTY_PUBLIC,
        PTN_PROPERTY_PUBLIC,
        0,
        PTN_PROPERTY_TYPE_NONE,
        NULL,
        NULL,
        0,
        1,
        function_value,
        0
    );
    ptn_value_destroy(&assigned);
    ptn_value_destroy(&function_value);
    return object;
}

static PTN_UNUSED int ptn_generator_guard_not_executing(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t line
) {
    if (generator == NULL || !generator->executing) {
        return 1;
    }
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Cannot resume an already running generator",
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
    return 0;
}

static PTN_UNUSED PtnValue ptn_generator_resume_receiver(
    PtnRuntime *runtime,
    PtnValue receiver
) {
    PtnGenerator *generator = runtime == NULL ? NULL : runtime->current_generator;
    PtnValue resolved = ptn_value_deref(receiver);
    if (
        resolved.type == PTN_NULL &&
        generator != NULL &&
        generator->executing &&
        generator->object != NULL
    ) {
        return ptn_value_clone_deref(ptn_object(generator->object));
    }
    return ptn_value_clone_deref(receiver);
}

static PTN_UNUSED void ptn_generator_flush_output_chunk(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t index
) {
    if (
        generator == NULL ||
        generator->output_chunks == NULL ||
        index >= generator->output_chunks->len
    ) {
        return;
    }
    PtnArrayEntry *entry = &generator->output_chunks->entries[index];
    PtnValue chunk = ptn_value_deref(entry->value);
    if (chunk.type != PTN_STRING || chunk.as.string.len == 0) {
        return;
    }
    PtnGenerator *saved_current_generator = runtime == NULL ? NULL : runtime->current_generator;
    if (runtime != NULL && saved_current_generator == generator) {
        runtime->current_generator = NULL;
    }
    ptn_output_write(runtime, (const char *)chunk.as.string.data, chunk.as.string.len);
    if (runtime != NULL && saved_current_generator == generator) {
        runtime->current_generator = saved_current_generator;
    }
    ptn_value_destroy(&entry->value);
    entry->value = ptn_null();
}

static PTN_UNUSED void ptn_generator_flush_pending_output(
    PtnRuntime *runtime,
    PtnGenerator *generator
) {
    if (
        generator == NULL ||
        generator->pending_output.data == NULL ||
        generator->pending_output.len == 0
    ) {
        return;
    }
    PtnGenerator *saved_current_generator = runtime == NULL ? NULL : runtime->current_generator;
    if (runtime != NULL && saved_current_generator == generator) {
        runtime->current_generator = NULL;
    }
    ptn_output_write(runtime, generator->pending_output.data, generator->pending_output.len);
    if (runtime != NULL && saved_current_generator == generator) {
        runtime->current_generator = saved_current_generator;
    }
    generator->pending_output.len = 0;
    generator->pending_output.data[0] = '\0';
}

static PTN_UNUSED void ptn_generator_flush_pending_output_before_value_drop(
    PtnRuntime *runtime,
    PtnValue value
) {
    PtnValue resolved = ptn_value_deref(value);
    if (
        !value.owned ||
        resolved.type != PTN_OBJECT ||
        resolved.as.object == NULL ||
        resolved.as.object->refcount != 1 ||
        !ptn_object_is_generator(resolved.as.object)
    ) {
        return;
    }
    ptn_generator_flush_pending_output(runtime, ptn_generator_from_value(resolved));
}

static PTN_UNUSED PtnValue ptn_generator_take_pending_output(PtnGenerator *generator) {
    if (generator == NULL || generator->pending_output.len == 0) {
        return ptn_null();
    }
    PtnValue output = ptn_owned_string_len(
        ptn_duplicate_string_len(generator->pending_output.data, generator->pending_output.len),
        generator->pending_output.len
    );
    generator->pending_output.len = 0;
    if (generator->pending_output.data != NULL) {
        generator->pending_output.data[0] = '\0';
    }
    return output;
}

static PTN_UNUSED PtnValue ptn_generator_yield(
    PtnRuntime *runtime,
    int has_key,
    PtnValue key_value,
    int has_value,
    PtnValue value,
    size_t line
) {
    PtnGenerator *generator = runtime == NULL ? NULL : runtime->current_generator;
    if (
        generator == NULL ||
        generator->values == NULL ||
        generator->keys == NULL ||
        generator->reference_notice_lines == NULL ||
        generator->yield_lines == NULL ||
        generator->delegate_sources == NULL ||
        generator->force_close_yield_from_entries == NULL ||
        generator->output_chunks == NULL
    ) {
        return ptn_null();
    }

    PtnValue stored;
    PtnValue reference_notice_line = ptn_int(0);
    if (generator->yields_by_ref) {
        if (value.type == PTN_REFERENCE) {
            stored = ptn_value_clone(value);
        } else {
            stored = ptn_value_clone_deref(value);
            if (has_value) {
                reference_notice_line = ptn_int((int64_t)line);
            }
        }
    } else {
        stored = ptn_value_clone_deref(value);
    }

    PtnValue raw_key;
    if (has_key) {
        raw_key = ptn_value_clone_deref(key_value);
        PtnValue resolved_key = ptn_value_deref(raw_key);
        if (
            resolved_key.type == PTN_INT &&
            resolved_key.as.integer >= generator->next_auto_key &&
            resolved_key.as.integer < INT64_MAX
        ) {
            generator->next_auto_key = resolved_key.as.integer + 1;
        }
    } else {
        raw_key = ptn_int(generator->next_auto_key);
        if (generator->next_auto_key < INT64_MAX) {
            generator->next_auto_key++;
        }
    }

    if (
        !ptn_array_append_key_available(runtime, generator->values) ||
        !ptn_array_append_key_available(runtime, generator->keys) ||
        !ptn_array_append_key_available(runtime, generator->reference_notice_lines) ||
        !ptn_array_append_key_available(runtime, generator->yield_lines) ||
        !ptn_array_append_key_available(runtime, generator->delegate_sources) ||
        !ptn_array_append_key_available(runtime, generator->force_close_yield_from_entries) ||
        !ptn_array_append_key_available(runtime, generator->output_chunks)
    ) {
        ptn_value_destroy(&stored);
        ptn_value_destroy(&raw_key);
        ptn_value_destroy(&reference_notice_line);
        return ptn_null();
    }
    PtnArrayKey value_key = ptn_array_int_key(generator->values->next_auto_key);
    PtnArrayKey raw_key_index = ptn_array_int_key(generator->keys->next_auto_key);
    PtnArrayKey notice_key = ptn_array_int_key(generator->reference_notice_lines->next_auto_key);
    PtnArrayKey line_key = ptn_array_int_key(generator->yield_lines->next_auto_key);
    PtnArrayKey delegate_key = ptn_array_int_key(generator->delegate_sources->next_auto_key);
    PtnArrayKey force_close_key = ptn_array_int_key(generator->force_close_yield_from_entries->next_auto_key);
    PtnArrayKey output_key = ptn_array_int_key(generator->output_chunks->next_auto_key);
    ptn_array_set_entry(generator->values, value_key, stored);
    ptn_array_set_entry(generator->keys, raw_key_index, raw_key);
    ptn_array_set_entry(generator->reference_notice_lines, notice_key, reference_notice_line);
    ptn_array_set_entry(generator->yield_lines, line_key, ptn_int((int64_t)line));
    ptn_array_set_entry(generator->delegate_sources, delegate_key, ptn_null());
    ptn_array_set_entry(generator->force_close_yield_from_entries, force_close_key, ptn_int(0));
    ptn_array_set_entry(generator->output_chunks, output_key, ptn_generator_take_pending_output(generator));
    return ptn_value_clone_deref(value);
}

static PTN_UNUSED PtnValue *ptn_generator_delegate_source_value(PtnGenerator *generator, size_t index) {
    if (
        generator == NULL ||
        generator->delegate_sources == NULL ||
        index >= generator->delegate_sources->len
    ) {
        return NULL;
    }
    PtnArrayEntry *entry = &generator->delegate_sources->entries[index];
    PtnValue resolved = ptn_value_deref(entry->value);
    if (resolved.type != PTN_OBJECT || !ptn_object_is_generator(resolved.as.object)) {
        return NULL;
    }
    return &entry->value;
}

static PTN_UNUSED int ptn_value_is_unpack_traversable(PtnValue value);
static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_value(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_iterator_current_key(PtnArrayIterator *iterator);
static PTN_UNUSED PtnValue ptn_array_iterator_current_value(PtnArrayIterator *iterator);
static PTN_UNUSED void ptn_array_iterator_advance(PtnArrayIterator *iterator);
static PTN_UNUSED void ptn_array_iterator_destroy(PtnArrayIterator *iterator);

static PTN_UNUSED PtnGenerator *ptn_generator_delegate_source(PtnGenerator *generator, size_t index) {
    PtnValue *source = ptn_generator_delegate_source_value(generator, index);
    return source == NULL ? NULL : ptn_generator_from_value(*source);
}

static PTN_UNUSED PtnValue *ptn_generator_traversable_delegate_source_value(
    PtnGenerator *generator,
    size_t index
) {
    if (
        generator == NULL ||
        generator->delegate_sources == NULL ||
        index >= generator->delegate_sources->len
    ) {
        return NULL;
    }
    PtnArrayEntry *entry = &generator->delegate_sources->entries[index];
    PtnValue resolved = ptn_value_deref(entry->value);
    if (
        resolved.type != PTN_OBJECT ||
        ptn_object_is_generator(resolved.as.object) ||
        !ptn_value_is_unpack_traversable(resolved)
    ) {
        return NULL;
    }
    return &entry->value;
}

static PTN_UNUSED int ptn_generator_validate_yield_from_delegate(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnGenerator *source_generator,
    size_t line
) {
    if (source_generator == NULL) {
        return 1;
    }
    if (source_generator == generator || source_generator->executing) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Impossible to yield from the Generator being currently run",
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
        return 0;
    }
    return 1;
}

static PTN_UNUSED int ptn_generator_position_valid(PtnGenerator *generator);
static PTN_UNUSED int ptn_generator_has_current_or_last_yield(PtnGenerator *generator);

static PTN_UNUSED void ptn_generator_skip_exhausted_delegates(PtnGenerator *generator) {
    while (
        generator != NULL &&
        generator->values != NULL &&
        generator->position < generator->values->len
    ) {
        if (ptn_generator_traversable_delegate_source_value(generator, generator->position) != NULL) {
            return;
        }
        PtnGenerator *source = ptn_generator_delegate_source(generator, generator->position);
        if (
            source != NULL &&
            generator->has_pending_exception &&
            generator->pending_exception_position == generator->position &&
            ptn_generator_has_current_or_last_yield(source)
        ) {
            return;
        }
        if (
            source == NULL ||
            source->executing ||
            source->has_pending_exception ||
            ptn_generator_position_valid(source)
        ) {
            return;
        }
        generator->position++;
    }
}

static PTN_UNUSED int ptn_generator_last_yield_index(PtnGenerator *generator, size_t *index) {
    if (
        generator == NULL ||
        generator->values == NULL ||
        generator->keys == NULL ||
        generator->values->len == 0 ||
        generator->keys->len < generator->values->len
    ) {
        return 0;
    }
    *index = generator->values->len - 1;
    return 1;
}

static PTN_UNUSED int ptn_generator_pending_exception_after_last_yield(
    PtnGenerator *generator,
    size_t *index
) {
    if (generator == NULL || !generator->started || !generator->has_pending_exception) {
        return 0;
    }
    size_t last_index = 0;
    if (!ptn_generator_last_yield_index(generator, &last_index)) {
        return 0;
    }
    if (generator->pending_exception_position != last_index) {
        return 0;
    }
    if (index != NULL) {
        *index = last_index;
    }
    return 1;
}

static PTN_UNUSED void ptn_generator_mark_return_yield(PtnGenerator *generator) {
    size_t index = 0;
    if (!ptn_generator_last_yield_index(generator, &index)) {
        return;
    }
    generator->return_yield_position = index;
    generator->has_return_yield_position = 1;
}

static PTN_UNUSED int ptn_generator_position_is_return_yield(
    PtnGenerator *generator,
    size_t position
) {
    return generator != NULL &&
        generator->has_return_yield_position &&
        generator->return_yield_position == position;
}

static PTN_UNUSED void ptn_generator_apply_resume_return_value(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnValue value
) {
    (void)runtime;
    if (
        generator == NULL ||
        !ptn_generator_position_is_return_yield(generator, generator->position)
    ) {
        return;
    }
    ptn_value_destroy(&generator->return_value);
    generator->return_value = ptn_value_clone_deref(value);
    generator->completed = 1;
    generator->has_return_yield_position = 0;
}

static PTN_UNUSED int ptn_generator_append_delegate_entry(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnValue resolved,
    size_t line
) {
    if (
        generator == NULL ||
        generator->values == NULL ||
        generator->keys == NULL ||
        generator->reference_notice_lines == NULL ||
        generator->yield_lines == NULL ||
        generator->delegate_sources == NULL ||
        generator->force_close_yield_from_entries == NULL ||
        generator->output_chunks == NULL ||
        resolved.type != PTN_OBJECT ||
        !ptn_value_is_unpack_traversable(resolved)
    ) {
        return 0;
    }

    if (
        !ptn_array_append_key_available(runtime, generator->values) ||
        !ptn_array_append_key_available(runtime, generator->keys) ||
        !ptn_array_append_key_available(runtime, generator->reference_notice_lines) ||
        !ptn_array_append_key_available(runtime, generator->yield_lines) ||
        !ptn_array_append_key_available(runtime, generator->delegate_sources) ||
        !ptn_array_append_key_available(runtime, generator->force_close_yield_from_entries) ||
        !ptn_array_append_key_available(runtime, generator->output_chunks)
    ) {
        return 0;
    }

    PtnArrayKey value_key = ptn_array_int_key(generator->values->next_auto_key);
    PtnArrayKey raw_key_index = ptn_array_int_key(generator->keys->next_auto_key);
    PtnArrayKey notice_key = ptn_array_int_key(generator->reference_notice_lines->next_auto_key);
    PtnArrayKey line_key = ptn_array_int_key(generator->yield_lines->next_auto_key);
    PtnArrayKey delegate_key = ptn_array_int_key(generator->delegate_sources->next_auto_key);
    PtnArrayKey force_close_key = ptn_array_int_key(generator->force_close_yield_from_entries->next_auto_key);
    PtnArrayKey output_key = ptn_array_int_key(generator->output_chunks->next_auto_key);
    ptn_array_set_entry(generator->values, value_key, ptn_null());
    ptn_array_set_entry(generator->keys, raw_key_index, ptn_null());
    ptn_array_set_entry(generator->reference_notice_lines, notice_key, ptn_int(0));
    ptn_array_set_entry(generator->yield_lines, line_key, ptn_int((int64_t)line));
    ptn_array_set_entry(generator->delegate_sources, delegate_key, ptn_value_clone_deref(resolved));
    ptn_array_set_entry(
        generator->force_close_yield_from_entries,
        force_close_key,
        ptn_int(runtime != NULL && runtime->generator_aborted_after_yield ? 1 : 0)
    );
    ptn_array_set_entry(generator->output_chunks, output_key, ptn_generator_take_pending_output(generator));
    return 1;
}

static PTN_UNUSED void ptn_generator_adopt_pending_yield_from_delegate(PtnRuntime *runtime, PtnValue source) {
    if (runtime == NULL || runtime->pending_yield_from_generator == NULL) {
        return;
    }
    PtnGenerator *parent = runtime->pending_yield_from_generator;
    size_t line = runtime->pending_yield_from_line;
    runtime->pending_yield_from_generator = NULL;
    runtime->pending_yield_from_line = 0;
    PtnValue resolved = ptn_value_deref(source);
    PtnGenerator *source_generator = ptn_generator_from_value(resolved);
    if (source_generator == NULL) {
        return;
    }
    if (!ptn_generator_validate_yield_from_delegate(runtime, parent, source_generator, line)) {
        return;
    }
    PtnGenerator *existing = ptn_generator_delegate_source(parent, parent->position);
    if (existing == source_generator) {
        return;
    }
    ptn_generator_append_delegate_entry(runtime, parent, resolved, line);
}

static PTN_UNUSED PtnValue ptn_generator_yield_delegate(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
) {
    PtnGenerator *generator = runtime == NULL ? NULL : runtime->current_generator;
    PtnValue resolved = ptn_value_deref(source);
    PtnGenerator *source_generator = ptn_generator_from_value(resolved);
    if (generator == NULL || source_generator == NULL) {
        return ptn_null();
    }

    if (!ptn_generator_validate_yield_from_delegate(runtime, generator, source_generator, line)) {
        return ptn_null();
    }
    PtnGenerator *existing = ptn_generator_delegate_source(generator, generator->position);
    if (existing != source_generator &&
        !ptn_generator_append_delegate_entry(runtime, generator, resolved, line)
    ) {
        return ptn_null();
    }
    return ptn_generator_get_return(runtime, resolved, line);
}

static PTN_UNUSED int ptn_generator_position_valid(PtnGenerator *generator) {
    return generator != NULL &&
        generator->values != NULL &&
        generator->keys != NULL &&
        generator->position < generator->values->len &&
        generator->position < generator->keys->len;
}

static PTN_UNUSED int ptn_generator_has_current_or_last_yield(PtnGenerator *generator) {
    if (ptn_generator_position_valid(generator)) {
        return 1;
    }
    size_t last_index = 0;
    return generator != NULL && ptn_generator_last_yield_index(generator, &last_index);
}

static PTN_UNUSED int ptn_generator_capture_pending_exception(PtnRuntime *runtime, PtnGenerator *generator) {
    if (
        runtime == NULL ||
        runtime->exceptions == NULL ||
        runtime->exceptions->active_exception == NULL ||
        generator == NULL ||
        generator->values == NULL
    ) {
        return 0;
    }
    ptn_value_destroy(&generator->pending_exception);
    generator->pending_exception = ptn_value_clone(ptn_exception_borrow(runtime->exceptions->active_exception));
    generator->pending_exception_position = generator->values->len == 0 ? 0 : generator->values->len - 1;
    generator->has_pending_exception = 1;
    generator->pending_exception_on_rewind =
        runtime->generator_aborted_after_yield && runtime->generator_aborted_rethrow_on_rewind ? 1 : 0;
    runtime->generator_aborted_after_yield = 0;
    runtime->generator_aborted_rethrow_on_rewind = 0;
    runtime->generator_chained_exception_during_unwind = 0;
    ptn_clear_exception(runtime);
    return 1;
}

static PTN_UNUSED void ptn_generator_trace_set_file_line(PtnValue frame, const char *file, size_t line) {
    if (file == NULL || line == 0) {
        return;
    }
    if (line > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_array_set_entry(
        frame.as.array,
        ptn_array_string_key("file"),
        ptn_owned_string(ptn_duplicate_string(file))
    );
    ptn_array_set_entry(
        frame.as.array,
        ptn_array_string_key("line"),
        ptn_int((int64_t)line)
    );
}

static PTN_UNUSED void ptn_generator_trace_set_empty_args(PtnValue frame) {
    ptn_array_set_entry(
        frame.as.array,
        ptn_array_string_key("args"),
        ptn_array_from_literal_entries(0, NULL)
    );
}

static PTN_UNUSED PtnValue ptn_generator_trace_function_frame(
    PtnGenerator *generator,
    const char *file,
    size_t line
) {
    PtnValue frame = ptn_array_from_literal_entries(0, NULL);
    ptn_generator_trace_set_file_line(frame, file, line);
    const char *function_name =
        generator != NULL && generator->function_name != NULL ? generator->function_name : "{unknown}";
    const char *object_separator = strstr(function_name, "->");
    const char *static_separator = strstr(function_name, "::");
    const char *separator = object_separator != NULL ? object_separator : static_separator;
    if (separator != NULL && separator != function_name && separator[2] != '\0') {
        size_t class_len = (size_t)(separator - function_name);
        const char *method_name = separator + 2;
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("class"),
            ptn_owned_string_len(ptn_duplicate_string_len(function_name, class_len), class_len)
        );
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("type"),
            ptn_string(object_separator != NULL || (generator != NULL && generator->has_receiver) ? "->" : "::")
        );
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("function"),
            ptn_owned_string(ptn_duplicate_string(method_name))
        );
    } else {
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("function"),
            ptn_owned_string(ptn_duplicate_string(function_name))
        );
    }
    ptn_generator_trace_set_empty_args(frame);
    return frame;
}

static PTN_UNUSED PtnValue ptn_generator_trace_resume_frame(
    PtnRuntime *runtime,
    const char *method_name,
    size_t line
) {
    PtnValue frame = ptn_array_from_literal_entries(0, NULL);
    ptn_generator_trace_set_file_line(frame, runtime != NULL ? runtime->source_path : NULL, line);
    ptn_array_set_entry(frame.as.array, ptn_array_string_key("class"), ptn_string("Generator"));
    ptn_array_set_entry(frame.as.array, ptn_array_string_key("type"), ptn_string("->"));
    ptn_array_set_entry(
        frame.as.array,
        ptn_array_string_key("function"),
        ptn_string(method_name == NULL ? "next" : method_name)
    );
    ptn_generator_trace_set_empty_args(frame);
    return frame;
}

static PTN_UNUSED void ptn_generator_trace_append(PtnValue trace, size_t *index, PtnValue frame) {
    if (*index > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_array_set_entry(trace.as.array, ptn_array_int_key((int64_t)*index), frame);
    (*index)++;
}

static PTN_UNUSED int ptn_generator_trace_function_equals_cstr(PtnValue value, const char *name) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type != PTN_STRING || name == NULL) {
        return 0;
    }
    size_t name_len = strlen(name);
    return resolved.as.string.len == name_len &&
        memcmp(resolved.as.string.data, name, name_len) == 0;
}

static PTN_UNUSED int ptn_generator_trace_frame_matches_generator(
    PtnValue frame,
    PtnGenerator *generator
) {
    if (generator == NULL || generator->function_name == NULL) {
        return 0;
    }
    PtnValue resolved = ptn_value_deref(frame);
    PtnValue *function_slot = ptn_trace_array_string_slot(resolved, "function");
    if (function_slot == NULL) {
        return 0;
    }
    if (ptn_generator_trace_function_equals_cstr(*function_slot, generator->function_name)) {
        return 1;
    }

    const char *object_separator = strstr(generator->function_name, "->");
    const char *static_separator = strstr(generator->function_name, "::");
    const char *separator = object_separator != NULL ? object_separator : static_separator;
    if (separator != NULL && separator[2] != '\0') {
        return ptn_generator_trace_function_equals_cstr(*function_slot, separator + 2);
    }
    return 0;
}

static PTN_UNUSED size_t ptn_generator_yield_line_at(PtnGenerator *generator, size_t position) {
    if (
        generator == NULL ||
        generator->yield_lines == NULL ||
        position >= generator->yield_lines->len
    ) {
        return 0;
    }
    PtnValue line_value = ptn_value_deref(generator->yield_lines->entries[position].value);
    if (line_value.type != PTN_INT || line_value.as.integer <= 0) {
        return 0;
    }
    return (size_t)line_value.as.integer;
}

static PTN_UNUSED int ptn_generator_resume_uses_iterator_helper_frame(PtnRuntime *runtime) {
    if (
        runtime == NULL ||
        runtime->trace_frame == NULL ||
        runtime->trace_frame->function_name == NULL
    ) {
        return 0;
    }
    return strcmp(runtime->trace_frame->function_name, "iterator_count") == 0 ||
        strcmp(runtime->trace_frame->function_name, "iterator_to_array") == 0 ||
        strcmp(runtime->trace_frame->function_name, "iterator_apply") == 0;
}

static PTN_UNUSED int ptn_generator_trace_frame_is_generator_method(PtnValue frame, const char *method_name) {
    PtnValue resolved = ptn_value_deref(frame);
    if (resolved.type != PTN_ARRAY || method_name == NULL) {
        return 0;
    }
    PtnValue *class_slot = ptn_trace_array_string_slot(resolved, "class");
    PtnValue *function_slot = ptn_trace_array_string_slot(resolved, "function");
    PtnValue resolved_class = class_slot == NULL ? ptn_null() : ptn_value_deref(*class_slot);
    PtnValue resolved_function = function_slot == NULL ? ptn_null() : ptn_value_deref(*function_slot);
    size_t method_len = strlen(method_name);
    return resolved_class.type == PTN_STRING &&
        resolved_function.type == PTN_STRING &&
        resolved_class.as.string.len == strlen("Generator") &&
        memcmp(resolved_class.as.string.data, "Generator", strlen("Generator")) == 0 &&
        resolved_function.as.string.len == method_len &&
        memcmp(resolved_function.as.string.data, method_name, method_len) == 0;
}

static PTN_UNUSED int ptn_generator_trace_frame_is_generator_rewind(PtnValue frame) {
    return ptn_generator_trace_frame_is_generator_method(frame, "rewind");
}

static PTN_UNUSED int ptn_generator_trace_frame_is_generator_resume(PtnValue frame) {
    return ptn_generator_trace_frame_is_generator_method(frame, "current") ||
        ptn_generator_trace_frame_is_generator_method(frame, "key") ||
        ptn_generator_trace_frame_is_generator_method(frame, "next") ||
        ptn_generator_trace_frame_is_generator_method(frame, "rewind") ||
        ptn_generator_trace_frame_is_generator_method(frame, "send") ||
        ptn_generator_trace_frame_is_generator_method(frame, "throw");
}

static PTN_UNUSED size_t ptn_generator_trace_frame_line(PtnValue frame) {
    PtnValue resolved = ptn_value_deref(frame);
    if (resolved.type != PTN_ARRAY) {
        return 0;
    }
    PtnValue *line_slot = ptn_trace_array_string_slot(resolved, "line");
    if (line_slot == NULL) {
        return 0;
    }
    PtnValue line_value = ptn_value_deref(*line_slot);
    if (line_value.type != PTN_INT || line_value.as.integer <= 0) {
        return 0;
    }
    return (size_t)line_value.as.integer;
}

static PTN_UNUSED void ptn_generator_trace_normalize_get_iterator_frame(PtnValue frame) {
    PtnValue resolved = ptn_value_deref(frame);
    if (resolved.type != PTN_ARRAY) {
        return;
    }
    PtnValue *function_slot = ptn_trace_array_string_slot(resolved, "function");
    PtnValue *type_slot = ptn_trace_array_string_slot(resolved, "type");
    if (function_slot == NULL) {
        return;
    }
    PtnValue resolved_function = ptn_value_deref(*function_slot);
    if (resolved_function.type != PTN_STRING) {
        return;
    }
    const unsigned char *separator = NULL;
    for (size_t i = 0; i + 1 < resolved_function.as.string.len; i++) {
        if (
            resolved_function.as.string.data[i] == ':' &&
            resolved_function.as.string.data[i + 1] == ':'
        ) {
            separator = resolved_function.as.string.data + i;
            break;
        }
    }
    if (separator != NULL) {
        const unsigned char *method = separator + 2;
        size_t method_len =
            resolved_function.as.string.data + resolved_function.as.string.len - method;
        if (method_len != strlen("getIterator") || memcmp(method, "getIterator", method_len) != 0) {
            return;
        }
        size_t class_len = (size_t)(separator - resolved_function.as.string.data);
        ptn_array_set_entry(
            resolved.as.array,
            ptn_array_string_key("class"),
            ptn_owned_string_len(
                ptn_duplicate_string_len((const char *)resolved_function.as.string.data, class_len),
                class_len
            )
        );
        ptn_array_set_entry(resolved.as.array, ptn_array_string_key("type"), ptn_string("->"));
        ptn_array_set_entry(resolved.as.array, ptn_array_string_key("function"), ptn_string("getIterator"));
        return;
    }
    if (
        type_slot == NULL ||
        !ptn_generator_trace_function_equals_cstr(*function_slot, "getIterator")
    ) {
        return;
    }
    PtnValue resolved_type = ptn_value_deref(*type_slot);
    if (
        resolved_type.type == PTN_STRING &&
        resolved_type.as.string.len == strlen("::") &&
        memcmp(resolved_type.as.string.data, "::", strlen("::")) == 0
    ) {
        ptn_array_set_entry(resolved.as.array, ptn_array_string_key("type"), ptn_string("->"));
    }
}

static PTN_UNUSED void ptn_generator_rewrite_pending_exception_trace_with_parent(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnGenerator *resume_parent,
    size_t resume_parent_position,
    PtnException *exception,
    size_t position,
    size_t line,
    const char *resume_method_name,
    int preserve_internal_resume_frame
) {
    if (exception == NULL) {
        return;
    }
    PtnValue trace = ptn_array_from_literal_entries(0, NULL);
    size_t index = 0;
    if (resume_parent != NULL && resume_parent != generator) {
        PtnValue existing = ptn_value_deref(exception->trace);
        size_t yielded_from_line = ptn_generator_yield_line_at(resume_parent, resume_parent_position);
        if (existing.type == PTN_ARRAY && existing.as.array != NULL && existing.as.array->len > 0) {
            PtnValue first_frame = ptn_generator_trace_function_frame(
                generator,
                runtime != NULL ? runtime->source_path : NULL,
                yielded_from_line != 0 ? yielded_from_line : line
            );
            ptn_generator_trace_append(trace, &index, first_frame);
            for (size_t i = 1; i < existing.as.array->len; i++) {
                PtnValue copied_frame = ptn_value_clone_deref(existing.as.array->entries[i].value);
                if (ptn_generator_trace_frame_is_generator_resume(copied_frame)) {
                    ptn_value_destroy(&copied_frame);
                    continue;
                }
                ptn_generator_trace_append(trace, &index, copied_frame);
            }
        } else {
            ptn_generator_trace_append(
                trace,
                &index,
                ptn_generator_trace_function_frame(
                    generator,
                    runtime != NULL ? runtime->source_path : NULL,
                    yielded_from_line != 0 ? yielded_from_line : line
                )
            );
        }
        ptn_generator_trace_append(
            trace,
            &index,
            ptn_generator_trace_function_frame(resume_parent, NULL, 0)
        );
        if (resume_method_name != NULL) {
            ptn_generator_trace_append(
                trace,
                &index,
                ptn_generator_trace_resume_frame(runtime, resume_method_name, line)
            );
        }
        ptn_value_destroy(&exception->trace);
        exception->trace = trace;
        return;
    }
    if (
        runtime != NULL &&
        runtime->current_generator != NULL &&
        runtime->current_generator != generator
    ) {
        PtnValue existing = ptn_value_deref(exception->trace);
        int existing_trace_already_has_current_generator = 0;
        if (existing.type == PTN_ARRAY && existing.as.array != NULL && existing.as.array->len > 0) {
            for (size_t i = 0; i < existing.as.array->len; i++) {
                if (
                    i + 1 == existing.as.array->len &&
                    ptn_generator_trace_frame_matches_generator(
                        existing.as.array->entries[i].value,
                        runtime->current_generator
                    )
                ) {
                    existing_trace_already_has_current_generator = 1;
                }
                ptn_generator_trace_append(
                    trace,
                    &index,
                    ptn_value_clone_deref(existing.as.array->entries[i].value)
                );
            }
        } else {
            ptn_generator_trace_append(
                trace,
                &index,
                ptn_generator_trace_function_frame(generator, runtime->source_path, line)
            );
        }
        if (!existing_trace_already_has_current_generator) {
            ptn_generator_trace_append(
                trace,
                &index,
                ptn_generator_trace_function_frame(runtime->current_generator, NULL, 0)
            );
        }
    } else {
        PtnValue existing = ptn_value_deref(exception->trace);
        int append_resume_frame = 0;
        int replaced_internal_resume_frame = 0;
        int use_iterator_helper_frame =
            ptn_generator_resume_uses_iterator_helper_frame(runtime);
        if (existing.type == PTN_ARRAY && existing.as.array != NULL) {
            size_t yielded_from_line = ptn_generator_yield_line_at(generator, position);
            if (existing.as.array->len >= 2) {
                PtnValue last_frame =
                    ptn_value_deref(existing.as.array->entries[existing.as.array->len - 1].value);
                append_resume_frame = last_frame.type == PTN_ARRAY &&
                    ptn_trace_array_string_slot(last_frame, "file") == NULL;
            }
            size_t skipped_internal_resume_line = 0;
            if (
                append_resume_frame &&
                resume_method_name != NULL &&
                !use_iterator_helper_frame &&
                !preserve_internal_resume_frame
            ) {
                for (size_t i = 0; i < existing.as.array->len; i++) {
                    PtnValue frame = existing.as.array->entries[i].value;
                    if (ptn_generator_trace_frame_is_generator_rewind(frame)) {
                        skipped_internal_resume_line = ptn_generator_trace_frame_line(frame);
                        break;
                    }
                }
            }
            int apply_yielded_from_line_to_next_frame = 0;
            for (size_t i = 0; i < existing.as.array->len; i++) {
                PtnValue copied_frame = ptn_value_clone_deref(existing.as.array->entries[i].value);
                if (
                    use_iterator_helper_frame &&
                    i > 0 &&
                    ptn_generator_trace_frame_is_generator_rewind(copied_frame)
                ) {
                    ptn_value_destroy(&copied_frame);
                    continue;
                }
                if (use_iterator_helper_frame && i == 0) {
                    ptn_value_destroy(&copied_frame);
                    copied_frame = ptn_generator_trace_function_frame(generator, NULL, 0);
                }
                if (
                    resume_method_name != NULL &&
                    !use_iterator_helper_frame &&
                    !preserve_internal_resume_frame &&
                    ptn_generator_trace_frame_is_generator_rewind(copied_frame)
                ) {
                    ptn_value_destroy(&copied_frame);
                    if (append_resume_frame) {
                        apply_yielded_from_line_to_next_frame = 1;
                        continue;
                    }
                    copied_frame = ptn_generator_trace_resume_frame(runtime, resume_method_name, line);
                    replaced_internal_resume_frame = 1;
                }
                if (
                    apply_yielded_from_line_to_next_frame &&
                    yielded_from_line != 0 &&
                    ptn_value_deref(copied_frame).type == PTN_ARRAY &&
                    ptn_trace_array_string_slot(ptn_value_deref(copied_frame), "file") == NULL
                ) {
                    ptn_generator_trace_set_file_line(
                        copied_frame,
                        runtime != NULL ? runtime->source_path : NULL,
                        yielded_from_line
                    );
                    apply_yielded_from_line_to_next_frame = 0;
                }
                if (
                    i == 0 &&
                    !use_iterator_helper_frame &&
                    !preserve_internal_resume_frame &&
                    (yielded_from_line != 0 || skipped_internal_resume_line != 0) &&
                    ptn_value_deref(copied_frame).type == PTN_ARRAY &&
                    ptn_trace_array_string_slot(ptn_value_deref(copied_frame), "file") == NULL
                ) {
                    size_t frame_line = skipped_internal_resume_line != 0
                        ? skipped_internal_resume_line
                        : yielded_from_line;
                    ptn_generator_trace_set_file_line(
                        copied_frame,
                        runtime != NULL ? runtime->source_path : NULL,
                        frame_line
                    );
                }
                if (i == 0) {
                    ptn_generator_trace_normalize_get_iterator_frame(copied_frame);
                }
                ptn_generator_trace_append(
                    trace,
                    &index,
                    copied_frame
                );
            }
        } else {
            ptn_generator_trace_append(
                trace,
                &index,
                ptn_generator_trace_function_frame(generator, NULL, 0)
            );
        }
        if (resume_method_name != NULL && use_iterator_helper_frame) {
            ptn_exception_trace_append_frame(trace, runtime->trace_frame, &index);
        } else if (resume_method_name != NULL && append_resume_frame && !replaced_internal_resume_frame) {
            ptn_generator_trace_append(
                trace,
                &index,
                ptn_generator_trace_resume_frame(runtime, resume_method_name, line)
            );
        }
    }
    ptn_value_destroy(&exception->trace);
    exception->trace = trace;
}

static PTN_UNUSED void ptn_generator_rewrite_pending_exception_trace(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnException *exception,
    size_t position,
    size_t line,
    const char *resume_method_name,
    int preserve_internal_resume_frame
) {
    ptn_generator_rewrite_pending_exception_trace_with_parent(
        runtime,
        generator,
        NULL,
        0,
        exception,
        position,
        line,
        resume_method_name,
        preserve_internal_resume_frame
    );
}

static PTN_UNUSED void ptn_generator_rewrite_throw_unwind_exception_trace(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnException *exception,
    size_t line
) {
    if (exception == NULL) {
        return;
    }
    PtnValue existing = ptn_value_deref(exception->trace);
    if (existing.type != PTN_ARRAY || existing.as.array == NULL || existing.as.array->len == 0) {
        return;
    }

    PtnValue trace = ptn_array_from_literal_entries(0, NULL);
    size_t index = 0;
    ptn_generator_trace_append(
        trace,
        &index,
        ptn_value_clone_deref(existing.as.array->entries[0].value)
    );
    ptn_generator_trace_append(
        trace,
        &index,
        ptn_generator_trace_function_frame(generator, NULL, 0)
    );
    PtnValue resume_frame = ptn_generator_trace_resume_frame(runtime, "throw", line);
    PtnValue resume_args = ptn_array_from_literal_entries(0, NULL);
    PtnValue previous = ptn_value_deref(exception->previous);
    if (previous.type == PTN_EXCEPTION) {
        ptn_array_set_entry(
            resume_args.as.array,
            ptn_array_int_key(0),
            ptn_value_clone_deref(exception->previous)
        );
    }
    ptn_array_set_entry(
        resume_frame.as.array,
        ptn_array_string_key("args"),
        resume_args
    );
    ptn_generator_trace_append(
        trace,
        &index,
        resume_frame
    );
    for (size_t i = 1; i < existing.as.array->len; i++) {
        ptn_generator_trace_append(
            trace,
            &index,
            ptn_value_clone_deref(existing.as.array->entries[i].value)
        );
    }
    ptn_value_destroy(&exception->trace);
    exception->trace = trace;
}

static PTN_UNUSED void ptn_generator_throw_pending_exception_value(
    PtnRuntime *runtime,
    PtnValue pending,
    const char *path,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(pending);
    if (
        runtime == NULL ||
        runtime->exceptions == NULL ||
        runtime->exceptions->try_frame != NULL ||
        resolved.type != PTN_EXCEPTION
    ) {
        ptn_throw_value(runtime, pending, path, line);
        return;
    }

    PtnException *pending_exception = resolved.as.exception;
    PtnException *active = runtime->exceptions->active_exception;
    if (active != pending_exception) {
        ptn_exception_chain_previous_if_missing(pending_exception, active);
        ptn_exception_retain(pending_exception);
        ptn_exception_free(active);
        runtime->exceptions->active_exception = pending_exception;
    }
    ptn_value_destroy(&pending);

    PtnRuntime *root = ptn_runtime_root(runtime);
    int previous_defer_uncaught_exception_emit = root == NULL
        ? 0
        : root->defer_uncaught_exception_emit;
    if (root != NULL) {
        root->defer_uncaught_exception_emit = 1;
    }
    ptn_runtime_shutdown_before_exit(runtime);
    if (root != NULL) {
        root->defer_uncaught_exception_emit = previous_defer_uncaught_exception_emit;
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    exit(255);
}

static PTN_UNUSED int ptn_generator_throw_pending_exception_at_position(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t position,
    size_t line,
    const char *resume_method_name,
    int preserve_internal_resume_frame
) {
    if (
        generator == NULL ||
        !generator->has_pending_exception ||
        generator->pending_exception_position != position
    ) {
        return 0;
    }
    ptn_generator_flush_pending_output(runtime, generator);
    PtnValue pending = ptn_value_clone_deref(generator->pending_exception);
    PtnValue resolved_pending = ptn_value_deref(pending);
    if (resolved_pending.type == PTN_EXCEPTION) {
        ptn_generator_rewrite_pending_exception_trace(
            runtime,
            generator,
            resolved_pending.as.exception,
            position,
            line,
            resume_method_name,
            preserve_internal_resume_frame
        );
    }
    ptn_value_destroy(&generator->pending_exception);
    generator->pending_exception = ptn_null();
    generator->has_pending_exception = 0;
    if (generator->values != NULL) {
        generator->position = generator->values->len;
    }
    ptn_generator_throw_pending_exception_value(
        runtime,
        pending,
        runtime != NULL ? runtime->source_path : NULL,
        generator->source_line
    );
    return 1;
}

static PTN_UNUSED int ptn_generator_throw_pending_exception_at_position_with_parent(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnGenerator *resume_parent,
    size_t resume_parent_position,
    size_t position,
    size_t line,
    const char *resume_method_name,
    int preserve_internal_resume_frame
) {
    if (
        generator == NULL ||
        !generator->has_pending_exception ||
        generator->pending_exception_position != position
    ) {
        return 0;
    }
    ptn_generator_flush_pending_output(runtime, generator);
    PtnValue pending = ptn_value_clone_deref(generator->pending_exception);
    PtnValue resolved_pending = ptn_value_deref(pending);
    if (resolved_pending.type == PTN_EXCEPTION) {
        ptn_generator_rewrite_pending_exception_trace_with_parent(
            runtime,
            generator,
            resume_parent,
            resume_parent_position,
            resolved_pending.as.exception,
            position,
            line,
            resume_method_name,
            preserve_internal_resume_frame
        );
    }
    ptn_value_destroy(&generator->pending_exception);
    generator->pending_exception = ptn_null();
    generator->has_pending_exception = 0;
    if (generator->values != NULL) {
        generator->position = generator->values->len;
    }
    ptn_generator_throw_pending_exception_value(
        runtime,
        pending,
        runtime != NULL ? runtime->source_path : NULL,
        generator->source_line
    );
    return 1;
}

static PTN_UNUSED int ptn_generator_throw_pending_delegate_exception_at_position(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t position,
    size_t line,
    const char *resume_method_name,
    int preserve_internal_resume_frame
) {
    if (
        generator == NULL ||
        !generator->has_pending_exception ||
        generator->pending_exception_position != position
    ) {
        return 0;
    }

    PtnGenerator *source = ptn_generator_delegate_source(generator, position);
    if (source == NULL || source->executing || ptn_generator_position_valid(source)) {
        return 0;
    }

    return ptn_generator_throw_pending_exception_at_position(
        runtime,
        generator,
        position,
        line,
        resume_method_name,
        preserve_internal_resume_frame
    );
}

static PTN_UNUSED void ptn_generator_emit_pending_reference_notice(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t index
) {
    if (
        runtime == NULL ||
        generator == NULL ||
        generator->reference_notice_lines == NULL ||
        index >= generator->reference_notice_lines->len
    ) {
        return;
    }
    PtnArrayEntry *entry = &generator->reference_notice_lines->entries[index];
    PtnValue line_value = ptn_value_deref(entry->value);
    if (line_value.type != PTN_INT || line_value.as.integer <= 0) {
        return;
    }
    ptn_emit_only_variable_references_yielded_by_reference_notice(
        &runtime->diagnostics,
        (size_t)line_value.as.integer
    );
    PtnValue replacement = ptn_int(0);
    ptn_value_destroy(&entry->value);
    entry->value = replacement;
}

static PTN_UNUSED void ptn_generator_release_consumed_reference(PtnGenerator *generator, size_t index) {
    if (generator == NULL || generator->values == NULL || index >= generator->values->len) {
        return;
    }
    PtnArrayEntry *entry = &generator->values->entries[index];
    if (entry->value.type != PTN_REFERENCE) {
        return;
    }
    PtnValue replacement = ptn_value_clone_deref(entry->value);
    ptn_value_destroy(&entry->value);
    entry->value = replacement;
}

static PTN_UNUSED int ptn_generator_force_close_yield_from_entry(
    PtnGenerator *generator,
    size_t index
) {
    if (
        generator == NULL ||
        generator->force_close_yield_from_entries == NULL ||
        index >= generator->force_close_yield_from_entries->len
    ) {
        return 0;
    }
    PtnValue value = ptn_value_deref(generator->force_close_yield_from_entries->entries[index].value);
    return value.type == PTN_INT && value.as.integer != 0;
}

static PTN_UNUSED PtnValue ptn_generator_current_at_position(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t position,
    size_t line,
    int use_delegate_last_value
);

static PTN_UNUSED void ptn_generator_throw_force_closed_yield_from(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t line
) {
    char *message = ptn_duplicate_string("Cannot use \"yield from\" in a force-closed generator");
    const char *path = generator != NULL && generator->source_file != NULL
        ? generator->source_file
        : (runtime != NULL ? runtime->source_path : NULL);
    size_t frame_line = runtime != NULL && runtime->call_site_line != 0
        ? runtime->call_site_line
        : line;
    ptn_throw_exception_owned_message_at_with_trace_frame(
        runtime,
        "Error",
        message,
        path,
        line,
        generator != NULL && generator->function_name != NULL ? generator->function_name : "{unknown}",
        runtime != NULL && runtime->source_path != NULL ? runtime->source_path : path,
        frame_line,
        0,
        NULL
    );
}

static PTN_UNUSED void ptn_generator_force_close(PtnRuntime *runtime, PtnGenerator *generator) {
    if (
        generator == NULL ||
        generator->values == NULL ||
        !generator->started ||
        generator->force_closing ||
        generator->position >= generator->values->len
    ) {
        return;
    }
    generator->force_closing = 1;
    size_t index = generator->position;
    size_t yield_line = ptn_generator_yield_line_at(generator, index);
    PtnValue current = ptn_generator_current_at_position(runtime, generator, index, yield_line, 0);
    ptn_value_destroy(&current);
    PtnGenerator *source = ptn_generator_delegate_source(generator, index);
    if (source != NULL) {
        ptn_generator_force_close(runtime, source);
    }
    ptn_generator_release_consumed_reference(generator, index);
    index++;
    for (; index < generator->values->len; index++) {
        if (!ptn_generator_force_close_yield_from_entry(generator, index)) {
            continue;
        }
        yield_line = ptn_generator_yield_line_at(generator, index);
        ptn_generator_flush_output_chunk(runtime, generator, index);
        generator->position = index;
        generator->force_closing = 0;
        ptn_generator_throw_force_closed_yield_from(runtime, generator, yield_line);
        return;
    }
    generator->position = generator->values->len;
    generator->force_closing = 0;
}

static PTN_UNUSED void ptn_generator_close(PtnGenerator *generator) {
    if (generator == NULL || generator->values == NULL) {
        return;
    }
    generator->started = 1;
    if (generator->position < generator->values->len) {
        ptn_generator_release_consumed_reference(generator, generator->position);
    }
    if (generator->delegate_sources != NULL) {
        for (size_t i = 0; i < generator->delegate_sources->len; i++) {
            PtnGenerator *source = ptn_generator_delegate_source(generator, i);
            if (source != NULL) {
                ptn_generator_close(source);
            }
        }
    }
    generator->position = generator->values->len;
}

static PTN_UNUSED void ptn_generator_close_for_throw(PtnRuntime *runtime, PtnGenerator *generator) {
    if (generator == NULL || generator->values == NULL) {
        return;
    }
    generator->started = 1;
    size_t start = generator->position;
    if (generator->delegate_sources != NULL) {
        for (size_t i = 0; i < generator->delegate_sources->len; i++) {
            PtnGenerator *source = ptn_generator_delegate_source(generator, i);
            if (source != NULL) {
                ptn_generator_close_for_throw(runtime, source);
            }
        }
    }
    for (size_t i = start; i < generator->values->len; i++) {
        size_t yield_line = ptn_generator_yield_line_at(generator, i);
        ptn_value_destroy_with_runtime_scope_at(
            runtime,
            &generator->values->entries[i].value,
            yield_line
        );
    }
    generator->position = generator->values->len;
}

static PTN_UNUSED const char *ptn_generator_throw_given_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT && value.as.object != NULL && value.as.object->class_name != NULL) {
        return value.as.object->class_name;
    }
    if (value.type == PTN_CLOSURE) {
        return "Closure";
    }
    if (value.type == PTN_EXCEPTION && value.as.exception != NULL && value.as.exception->class_name != NULL) {
        return value.as.exception->class_name;
    }
    return ptn_offset_container_type_name(value);
}

static PTN_UNUSED PtnValue ptn_generator_throw(
    PtnRuntime *runtime,
    PtnValue receiver,
    PtnValue exception,
    size_t line
) {
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (!ptn_generator_guard_not_executing(runtime, generator, line)) {
        return ptn_null();
    }
    PtnValue resolved_exception = ptn_value_deref(exception);
    int is_throwable = resolved_exception.type == PTN_EXCEPTION ||
        (resolved_exception.type == PTN_OBJECT &&
            ptn_object_is_declared_throwable(runtime, resolved_exception.as.object));
    if (!is_throwable) {
        const char *given = ptn_generator_throw_given_name(resolved_exception);
        int needed = snprintf(
            NULL,
            0,
            "Generator::throw(): Argument #1 ($exception) must be of type Throwable, %s given",
            given
        );
        if (needed < 0) {
            ptn_abort_out_of_memory();
        }
        char *message = malloc((size_t)needed + 1);
        if (message == NULL) {
            ptn_abort_out_of_memory();
        }
        snprintf(
            message,
            (size_t)needed + 1,
            "Generator::throw(): Argument #1 ($exception) must be of type Throwable, %s given",
            given
        );
        PtnValue trace_args[] = { exception };
        ptn_throw_exception_owned_message_at_with_trace_frame(
            runtime,
            "TypeError",
            message,
            runtime != NULL ? runtime->source_path : NULL,
            line,
            "Generator->throw",
            runtime != NULL ? runtime->source_path : NULL,
            line,
            1,
            trace_args
        );
        return ptn_null();
    }

    PtnValue thrown = ptn_value_clone_deref(resolved_exception);
    PtnValue resolved_thrown = ptn_value_deref(thrown);
    if (
        runtime != NULL &&
        runtime->exceptions != NULL &&
        resolved_thrown.type == PTN_EXCEPTION
    ) {
        ptn_exception_free(runtime->exceptions->active_exception);
        runtime->exceptions->active_exception = resolved_thrown.as.exception;
        ptn_exception_retain(runtime->exceptions->active_exception);
        if (runtime->exceptions->active_exception->path == NULL) {
            runtime->exceptions->active_exception->path = runtime->source_path;
            runtime->exceptions->active_exception->line = line;
        }
        ptn_value_destroy(&thrown);

        PtnRuntime *root = ptn_runtime_root(runtime);
        int previous_defer_uncaught_exception_emit = root == NULL
            ? 0
            : root->defer_uncaught_exception_emit;
        if (root != NULL) {
            root->defer_uncaught_exception_emit = 1;
        }
        PtnTryFrame close_frame;
        int close_frame_active = 0;
        if (runtime->exceptions != NULL) {
            ptn_try_frame_push(runtime, &close_frame);
            close_frame_active = 1;
            if (setjmp(close_frame.jump) != 0) {
                ptn_try_frame_pop(runtime, &close_frame);
                if (root != NULL) {
                    root->defer_uncaught_exception_emit = previous_defer_uncaught_exception_emit;
                }
                ptn_generator_rewrite_throw_unwind_exception_trace(
                    runtime,
                    generator,
                    runtime->exceptions->active_exception,
                    line
                );
                ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
                ptn_runtime_shutdown_before_exit(runtime);
                exit(255);
                return ptn_null();
            }
        }
        ptn_generator_close_for_throw(runtime, generator);
        if (close_frame_active) {
            ptn_try_frame_pop(runtime, &close_frame);
        }
        if (root != NULL) {
            root->defer_uncaught_exception_emit = previous_defer_uncaught_exception_emit;
        }
        ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
        ptn_runtime_shutdown_before_exit(runtime);
        exit(255);
        return ptn_null();
    }
    ptn_generator_close(generator);
    return ptn_throw_value(
        runtime,
        thrown,
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
}

static PTN_UNUSED PtnValue ptn_generator_current_or_last(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
);

static PTN_UNUSED PtnValue ptn_generator_key_or_last(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
);

static PTN_UNUSED PtnValue ptn_generator_current_at_position(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t position,
    size_t line,
    int use_delegate_last_value
) {
    ptn_generator_flush_output_chunk(runtime, generator, position);
    PtnValue *delegate_source = ptn_generator_delegate_source_value(generator, position);
    PtnGenerator *source_generator = delegate_source == NULL
        ? NULL
        : ptn_generator_from_value(*delegate_source);
    int delegate_can_provide_current = delegate_source != NULL &&
        use_delegate_last_value &&
        ptn_generator_has_current_or_last_yield(source_generator);
    if (
        !delegate_can_provide_current &&
        (
            delegate_source == NULL ||
            !use_delegate_last_value ||
            !ptn_generator_pending_exception_after_last_yield(source_generator, NULL)
        )
    ) {
        if (ptn_generator_throw_pending_delegate_exception_at_position(
            runtime,
            generator,
            position,
            line,
            NULL,
            1
        )) {
            return ptn_null();
        }
    }
    if (delegate_source != NULL) {
        PtnValue source_receiver = ptn_value_clone_deref(*delegate_source);
        PtnValue current = use_delegate_last_value
            ? ptn_generator_current_or_last(runtime, source_receiver, line)
            : ptn_generator_current(runtime, source_receiver, line);
        ptn_value_destroy(&source_receiver);
        return current;
    }
    PtnValue *traversable_source =
        ptn_generator_traversable_delegate_source_value(generator, position);
    if (traversable_source != NULL) {
        PtnValue source_receiver = ptn_value_clone_deref(*traversable_source);
        PtnArrayIterator iterator = ptn_array_iterator_from_value(
            runtime,
            ptn_value_deref(source_receiver),
            NULL,
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
        PtnValue current = iterator.valid
            ? ptn_array_iterator_current_value(&iterator)
            : ptn_null();
        PtnValue result = ptn_value_clone_deref(current);
        ptn_value_destroy(&current);
        ptn_array_iterator_destroy(&iterator);
        ptn_value_destroy(&source_receiver);
        return result;
    }
    ptn_generator_emit_pending_reference_notice(runtime, generator, position);
    return ptn_value_clone_deref(generator->values->entries[position].value);
}

static PTN_UNUSED PtnValue ptn_generator_key_at_position(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    size_t position,
    size_t line,
    int use_delegate_last_value
) {
    ptn_generator_flush_output_chunk(runtime, generator, position);
    PtnValue *delegate_source = ptn_generator_delegate_source_value(generator, position);
    PtnGenerator *source_generator = delegate_source == NULL
        ? NULL
        : ptn_generator_from_value(*delegate_source);
    int delegate_can_provide_key = delegate_source != NULL &&
        use_delegate_last_value &&
        ptn_generator_has_current_or_last_yield(source_generator);
    if (
        !delegate_can_provide_key &&
        (
            delegate_source == NULL ||
            !use_delegate_last_value ||
            !ptn_generator_pending_exception_after_last_yield(source_generator, NULL)
        )
    ) {
        if (ptn_generator_throw_pending_delegate_exception_at_position(
            runtime,
            generator,
            position,
            line,
            NULL,
            1
        )) {
            return ptn_null();
        }
    }
    if (delegate_source != NULL) {
        PtnValue source_receiver = ptn_value_clone_deref(*delegate_source);
        PtnValue key = use_delegate_last_value
            ? ptn_generator_key_or_last(runtime, source_receiver, line)
            : ptn_generator_key(runtime, source_receiver, line);
        ptn_value_destroy(&source_receiver);
        return key;
    }
    PtnValue *traversable_source =
        ptn_generator_traversable_delegate_source_value(generator, position);
    if (traversable_source != NULL) {
        PtnValue source_receiver = ptn_value_clone_deref(*traversable_source);
        PtnArrayIterator iterator = ptn_array_iterator_from_value(
            runtime,
            ptn_value_deref(source_receiver),
            NULL,
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
        PtnValue key = iterator.valid
            ? ptn_array_iterator_current_key(&iterator)
            : ptn_null();
        PtnValue result = ptn_value_clone_deref(key);
        ptn_value_destroy(&key);
        ptn_array_iterator_destroy(&iterator);
        ptn_value_destroy(&source_receiver);
        return result;
    }
    return ptn_value_clone_deref(generator->keys->entries[position].value);
}

static PTN_UNUSED PtnValue ptn_generator_current_or_last(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
) {
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (ptn_generator_position_valid(generator)) {
        return ptn_generator_current(runtime, receiver, line);
    }
    size_t last_index = 0;
    if (generator != NULL && ptn_generator_last_yield_index(generator, &last_index)) {
        return ptn_generator_current_at_position(runtime, generator, last_index, line, 1);
    }
    ptn_generator_flush_pending_output(runtime, generator);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_generator_key_or_last(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
) {
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (ptn_generator_position_valid(generator)) {
        return ptn_generator_key(runtime, receiver, line);
    }
    size_t last_index = 0;
    if (generator != NULL && ptn_generator_last_yield_index(generator, &last_index)) {
        return ptn_generator_key_at_position(runtime, generator, last_index, line, 1);
    }
    ptn_generator_flush_pending_output(runtime, generator);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_generator_current(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)line;
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (generator != NULL) {
        if (!generator->started) {
            ptn_generator_skip_exhausted_delegates(generator);
        }
        generator->started = 1;
    }
    if (!ptn_generator_position_valid(generator)) {
        if (generator != NULL && generator->has_pending_exception) {
            ptn_generator_throw_pending_exception_at_position(
                runtime,
                generator,
                generator->pending_exception_position,
                line,
                "next",
                1
            );
            return ptn_null();
        }
        ptn_generator_flush_pending_output(runtime, generator);
        return ptn_null();
    }
    return ptn_generator_current_at_position(runtime, generator, generator->position, line, 1);
}

static PTN_UNUSED PtnValue ptn_generator_get_return(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)line;
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (generator != NULL && generator->has_pending_exception) {
        ptn_generator_throw_pending_exception_at_position(
            runtime,
            generator,
            generator->pending_exception_position,
            line,
            "rewind",
            1
        );
        return ptn_null();
    }
    if (generator == NULL || !generator->completed) {
        ptn_throw_exception(
            runtime,
            "Exception",
            "Cannot get return value of a generator that hasn't returned"
        );
        return ptn_null();
    }
    if (runtime != NULL && runtime->current_generator == generator) {
        generator->executing = 0;
        runtime->current_generator = NULL;
    }
    return ptn_value_clone_deref(generator->return_value);
}

static PTN_UNUSED PtnValue ptn_generator_key(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)line;
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (generator != NULL) {
        if (!generator->started) {
            ptn_generator_skip_exhausted_delegates(generator);
        }
        generator->started = 1;
    }
    if (!ptn_generator_position_valid(generator)) {
        if (generator != NULL && generator->has_pending_exception) {
            ptn_generator_throw_pending_exception_at_position(
                runtime,
                generator,
                generator->pending_exception_position,
                line,
                "rewind",
                1
            );
            return ptn_null();
        }
        ptn_generator_flush_pending_output(runtime, generator);
        return ptn_null();
    }
    return ptn_generator_key_at_position(runtime, generator, generator->position, line, 1);
}

static PTN_UNUSED PtnValue ptn_generator_next(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)runtime;
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (!ptn_generator_guard_not_executing(runtime, generator, line)) {
        return ptn_null();
    }
    if (generator != NULL) {
        generator->started = 1;
    }
    if (ptn_generator_position_valid(generator)) {
        PtnValue *delegate_source = ptn_generator_delegate_source_value(generator, generator->position);
        if (delegate_source != NULL) {
            PtnValue source_receiver = ptn_value_clone_deref(*delegate_source);
            PtnValue advanced = ptn_generator_next(runtime, source_receiver, line);
            ptn_value_destroy(&advanced);
            PtnGenerator *source_generator = ptn_generator_from_value(source_receiver);
            int source_still_valid = ptn_generator_position_valid(source_generator);
            ptn_value_destroy(&source_receiver);
            if (source_still_valid) {
                return ptn_null();
            }
        }
        PtnValue *traversable_source =
            ptn_generator_traversable_delegate_source_value(generator, generator->position);
        if (traversable_source != NULL) {
            PtnValue source_receiver = ptn_value_clone_deref(*traversable_source);
            PtnArrayIterator iterator = ptn_array_iterator_from_value(
                runtime,
                ptn_value_deref(source_receiver),
                NULL,
                runtime != NULL ? runtime->source_path : NULL,
                line
            );
            if (iterator.valid) {
                PtnGenerator *iterator_generator = iterator.generator;
                size_t last_index = 0;
                if (
                    iterator_generator != NULL &&
                    ptn_generator_pending_exception_after_last_yield(iterator_generator, &last_index) &&
                    iterator.index == last_index
                ) {
                    ptn_generator_throw_pending_exception_at_position_with_parent(
                        runtime,
                        iterator_generator,
                        generator,
                        generator->position,
                        last_index,
                        line,
                        "next",
                        0
                    );
                }
                ptn_array_iterator_advance(&iterator);
            }
            int source_still_valid = iterator.valid;
            ptn_array_iterator_destroy(&iterator);
            ptn_value_destroy(&source_receiver);
            if (source_still_valid) {
                return ptn_null();
            }
            ptn_generator_flush_output_chunk(runtime, generator, generator->position);
            ptn_generator_release_consumed_reference(generator, generator->position);
            generator->position++;
            ptn_generator_skip_exhausted_delegates(generator);
            if (!ptn_generator_position_valid(generator)) {
                ptn_generator_flush_pending_output(runtime, generator);
            }
            return ptn_null();
        }
        ptn_generator_flush_output_chunk(runtime, generator, generator->position);
        if (ptn_generator_throw_pending_exception_at_position(
                runtime,
                generator,
                generator->position,
                line,
                "next",
                0
        )) {
            return ptn_null();
        }
        ptn_generator_apply_resume_return_value(runtime, generator, ptn_null());
        ptn_generator_release_consumed_reference(generator, generator->position);
        generator->position++;
        ptn_generator_skip_exhausted_delegates(generator);
        if (ptn_generator_position_valid(generator)) {
            ptn_generator_flush_output_chunk(runtime, generator, generator->position);
        } else {
            ptn_generator_flush_pending_output(runtime, generator);
        }
    } else if (generator != NULL) {
        if (generator->has_pending_exception) {
            ptn_generator_throw_pending_exception_at_position(
                runtime,
                generator,
                generator->pending_exception_position,
                line,
                "rewind",
                1
            );
            return ptn_null();
        }
        ptn_generator_flush_pending_output(runtime, generator);
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_generator_array_entry_value(PtnArray *array, size_t index) {
    if (array == NULL || index >= array->len) {
        return ptn_null();
    }
    return array->entries[index].value;
}

static PTN_UNUSED int ptn_generator_entry_matches_position(PtnArray *positions, size_t index, size_t position) {
    if (positions == NULL || index >= positions->len) {
        return 0;
    }
    PtnValue value = ptn_value_deref(positions->entries[index].value);
    return value.type == PTN_INT && value.as.integer >= 0 && (size_t)value.as.integer == position;
}

enum {
    PTN_GENERATOR_SEND_CALL_FUNCTION = 0,
    PTN_GENERATOR_SEND_CALL_CALLABLE = 1,
    PTN_GENERATOR_SEND_CALL_METHOD = 2,
    PTN_GENERATOR_SEND_CALL_NESTED_FUNCTION = 3
};

static PTN_UNUSED void ptn_generator_register_send_call_entry(
    PtnRuntime *runtime,
    int kind,
    const char *function_name,
    PtnValue receiver,
    size_t argc,
    const PtnValue *args,
    size_t yield_argc,
    const size_t *yield_indexes,
    size_t line
) {
    PtnGenerator *generator = runtime == NULL ? NULL : runtime->current_generator;
    if (
        generator == NULL ||
        generator->values == NULL ||
        generator->send_call_positions == NULL ||
        generator->send_call_kinds == NULL ||
        generator->send_call_names == NULL ||
        generator->send_call_receivers == NULL ||
        generator->send_call_arguments == NULL ||
        generator->send_call_yield_indexes == NULL ||
        generator->send_call_lines == NULL ||
        generator->values->len == 0
    ) {
        return;
    }
    if (
        !ptn_array_append_key_available(runtime, generator->send_call_positions) ||
        !ptn_array_append_key_available(runtime, generator->send_call_kinds) ||
        !ptn_array_append_key_available(runtime, generator->send_call_names) ||
        !ptn_array_append_key_available(runtime, generator->send_call_receivers) ||
        !ptn_array_append_key_available(runtime, generator->send_call_arguments) ||
        !ptn_array_append_key_available(runtime, generator->send_call_yield_indexes) ||
        !ptn_array_append_key_available(runtime, generator->send_call_lines)
    ) {
        return;
    }

    PtnValue arguments = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < argc; i++) {
        ptn_array_set_entry(arguments.as.array, ptn_array_int_key((int64_t)i), ptn_value_clone(args[i]));
    }
    PtnValue indexes = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < yield_argc; i++) {
        ptn_array_set_entry(indexes.as.array, ptn_array_int_key((int64_t)i), ptn_int((int64_t)yield_indexes[i]));
    }

    PtnArrayKey key = ptn_array_int_key(generator->send_call_positions->next_auto_key);
    ptn_array_set_entry(
        generator->send_call_positions,
        key,
        ptn_int((int64_t)(generator->values->len - 1))
    );
    ptn_array_set_entry(
        generator->send_call_kinds,
        ptn_array_int_key(generator->send_call_kinds->next_auto_key),
        ptn_int((int64_t)kind)
    );
    ptn_array_set_entry(
        generator->send_call_names,
        ptn_array_int_key(generator->send_call_names->next_auto_key),
        ptn_string(function_name == NULL ? "" : function_name)
    );
    ptn_array_set_entry(
        generator->send_call_receivers,
        ptn_array_int_key(generator->send_call_receivers->next_auto_key),
        ptn_value_clone(receiver)
    );
    ptn_array_set_entry(
        generator->send_call_arguments,
        ptn_array_int_key(generator->send_call_arguments->next_auto_key),
        arguments
    );
    ptn_array_set_entry(
        generator->send_call_yield_indexes,
        ptn_array_int_key(generator->send_call_yield_indexes->next_auto_key),
        indexes
    );
    ptn_array_set_entry(
        generator->send_call_lines,
        ptn_array_int_key(generator->send_call_lines->next_auto_key),
        ptn_int((int64_t)line)
    );
}

static PTN_UNUSED void ptn_generator_register_send_call(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    size_t yield_argc,
    const size_t *yield_indexes,
    size_t line
) {
    ptn_generator_register_send_call_entry(
        runtime,
        PTN_GENERATOR_SEND_CALL_FUNCTION,
        function_name,
        ptn_null(),
        argc,
        args,
        yield_argc,
        yield_indexes,
        line
    );
}

static PTN_UNUSED void ptn_generator_register_send_callable(
    PtnRuntime *runtime,
    PtnValue callable,
    size_t argc,
    const PtnValue *args,
    size_t yield_argc,
    const size_t *yield_indexes,
    size_t line
) {
    ptn_generator_register_send_call_entry(
        runtime,
        PTN_GENERATOR_SEND_CALL_CALLABLE,
        "",
        callable,
        argc,
        args,
        yield_argc,
        yield_indexes,
        line
    );
}

static PTN_UNUSED void ptn_generator_register_send_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name,
    size_t argc,
    const PtnValue *args,
    size_t yield_argc,
    const size_t *yield_indexes,
    size_t line
) {
    ptn_generator_register_send_call_entry(
        runtime,
        PTN_GENERATOR_SEND_CALL_METHOD,
        method_name,
        receiver,
        argc,
        args,
        yield_argc,
        yield_indexes,
        line
    );
}

static PTN_UNUSED void ptn_generator_register_send_nested_call(
    PtnRuntime *runtime,
    const char *outer_function_name,
    const char *inner_function_name,
    size_t argc,
    const PtnValue *args,
    size_t yield_argc,
    const size_t *yield_indexes,
    size_t line
) {
    PtnValue inner_name = ptn_string(inner_function_name == NULL ? "" : inner_function_name);
    ptn_generator_register_send_call_entry(
        runtime,
        PTN_GENERATOR_SEND_CALL_NESTED_FUNCTION,
        outer_function_name,
        inner_name,
        argc,
        args,
        yield_argc,
        yield_indexes,
        line
    );
    ptn_value_destroy(&inner_name);
}

static PTN_UNUSED int ptn_value_is_unpack_traversable(PtnValue value);

static PTN_UNUSED void ptn_generator_register_send_yield_from(PtnRuntime *runtime, size_t line) {
    PtnGenerator *generator = runtime == NULL ? NULL : runtime->current_generator;
    if (
        generator == NULL ||
        generator->values == NULL ||
        generator->send_yield_from_positions == NULL ||
        generator->send_yield_from_lines == NULL ||
        generator->values->len == 0
    ) {
        return;
    }
    if (
        !ptn_array_append_key_available(runtime, generator->send_yield_from_positions) ||
        !ptn_array_append_key_available(runtime, generator->send_yield_from_lines)
    ) {
        return;
    }
    ptn_array_set_entry(
        generator->send_yield_from_positions,
        ptn_array_int_key(generator->send_yield_from_positions->next_auto_key),
        ptn_int((int64_t)(generator->values->len - 1))
    );
    ptn_array_set_entry(
        generator->send_yield_from_lines,
        ptn_array_int_key(generator->send_yield_from_lines->next_auto_key),
        ptn_int((int64_t)line)
    );
}

static PTN_UNUSED void ptn_generator_throw_yield_from_running_delegate(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnValue sent_value,
    size_t throw_line,
    size_t resume_line
) {
    const char *message = "Impossible to yield from the Generator being currently run";
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        "Error",
        ptn_duplicate_string(message),
        strlen(message),
        0,
        ptn_null(),
        PTN_E_ERROR,
        runtime != NULL ? runtime->source_path : NULL,
        throw_line
    );

    PtnValue trace = ptn_array_from_literal_entries(0, NULL);
    size_t trace_index = 0;
    ptn_generator_trace_append(
        trace,
        &trace_index,
        ptn_generator_trace_function_frame(generator, NULL, 0)
    );
    PtnValue resume_frame = ptn_generator_trace_resume_frame(runtime, "send", resume_line);
    PtnValue args = ptn_array_from_literal_entries(0, NULL);
    ptn_array_set_entry(args.as.array, ptn_array_int_key(0), ptn_value_clone_deref(sent_value));
    ptn_array_set_entry(resume_frame.as.array, ptn_array_string_key("args"), args);
    ptn_generator_trace_append(trace, &trace_index, resume_frame);
    ptn_value_destroy(&exception->trace);
    exception->trace = trace;

    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED int ptn_generator_bind_sent_yield_from(
    PtnRuntime *runtime,
    PtnGenerator *generator,
    PtnValue sent_value,
    size_t fallback_line
) {
    if (
        generator == NULL ||
        generator->delegate_sources == NULL ||
        generator->send_yield_from_positions == NULL ||
        generator->send_yield_from_lines == NULL ||
        generator->position >= generator->delegate_sources->len
    ) {
        return 0;
    }
    for (size_t i = 0; i < generator->send_yield_from_positions->len; i++) {
        if (!ptn_generator_entry_matches_position(generator->send_yield_from_positions, i, generator->position)) {
            continue;
        }
        size_t line = fallback_line;
        PtnValue line_value = ptn_value_deref(ptn_generator_array_entry_value(generator->send_yield_from_lines, i));
        if (line_value.type == PTN_INT && line_value.as.integer >= 0) {
            line = (size_t)line_value.as.integer;
        }
        PtnValue resolved = ptn_value_deref(sent_value);
        if (
            resolved.type != PTN_ARRAY &&
            !(resolved.type == PTN_OBJECT && ptn_value_is_unpack_traversable(resolved))
        ) {
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Can use \"yield from\" only with arrays and Traversables",
                runtime != NULL ? runtime->source_path : NULL,
                line
            );
            return 1;
        }
        PtnGenerator *source_generator = ptn_generator_from_value(resolved);
        if (source_generator == NULL) {
            return 0;
        }
        if (source_generator == generator || source_generator->executing) {
            ptn_generator_throw_yield_from_running_delegate(
                runtime,
                generator,
                sent_value,
                line,
                fallback_line
            );
            return 1;
        }
        if (!ptn_generator_validate_yield_from_delegate(runtime, generator, source_generator, line)) {
            return 1;
        }
        PtnArrayEntry *entry = &generator->delegate_sources->entries[generator->position];
        ptn_value_destroy(&entry->value);
        entry->value = ptn_value_clone_deref(resolved);
        return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_generator_apply_sent_value_to_call_args(
    PtnArray *arguments,
    PtnArray *yield_indexes,
    PtnValue sent_value
) {
    if (arguments == NULL || yield_indexes == NULL) {
        return;
    }
    for (size_t i = 0; i < yield_indexes->len; i++) {
        PtnValue index_value = ptn_value_deref(yield_indexes->entries[i].value);
        if (index_value.type != PTN_INT || index_value.as.integer < 0) {
            continue;
        }
        size_t argument_index = (size_t)index_value.as.integer;
        if (argument_index >= arguments->len) {
            continue;
        }
        ptn_value_destroy(&arguments->entries[argument_index].value);
        arguments->entries[argument_index].value = ptn_value_clone_deref(sent_value);
    }
}

static PTN_UNUSED int ptn_generator_yield_indexes_contains(PtnArray *yield_indexes, size_t argument_index) {
    if (yield_indexes == NULL) {
        return 0;
    }
    for (size_t i = 0; i < yield_indexes->len; i++) {
        PtnValue index_value = ptn_value_deref(yield_indexes->entries[i].value);
        if (
            index_value.type == PTN_INT &&
            index_value.as.integer >= 0 &&
            (size_t)index_value.as.integer == argument_index
        ) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_generator_send(PtnRuntime *runtime, PtnValue receiver, PtnValue sent_value, size_t line) {
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (!ptn_generator_guard_not_executing(runtime, generator, line)) {
        return ptn_null();
    }
    if (generator != NULL) {
        generator->started = 1;
    }
    if (!ptn_generator_position_valid(generator)) {
        ptn_generator_flush_pending_output(runtime, generator);
        return ptn_null();
    }

    PtnValue *delegate_source = ptn_generator_delegate_source_value(generator, generator->position);
    if (delegate_source != NULL) {
        PtnValue source_receiver = ptn_value_clone_deref(*delegate_source);
        PtnValue result = ptn_generator_send(runtime, source_receiver, sent_value, line);
        PtnGenerator *source_generator = ptn_generator_from_value(source_receiver);
        int source_still_valid = ptn_generator_position_valid(source_generator);
        ptn_value_destroy(&source_receiver);
        if (source_still_valid) {
            return result;
        }
        ptn_value_destroy(&result);
    }

    if (ptn_generator_bind_sent_yield_from(runtime, generator, sent_value, line)) {
        if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
            return ptn_null();
        }
        return ptn_generator_current(runtime, receiver, line);
    }

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (generator->send_call_positions != NULL) {
        for (size_t i = 0; i < generator->send_call_positions->len; i++) {
            if (!ptn_generator_entry_matches_position(generator->send_call_positions, i, generator->position)) {
                continue;
            }
            PtnValue kind_value = ptn_value_deref(ptn_generator_array_entry_value(generator->send_call_kinds, i));
            PtnValue name_value = ptn_value_deref(ptn_generator_array_entry_value(generator->send_call_names, i));
            PtnValue receiver_value = ptn_value_deref(ptn_generator_array_entry_value(generator->send_call_receivers, i));
            PtnValue arguments_value = ptn_value_deref(ptn_generator_array_entry_value(generator->send_call_arguments, i));
            PtnValue indexes_value = ptn_value_deref(ptn_generator_array_entry_value(generator->send_call_yield_indexes, i));
            PtnValue line_value = ptn_value_deref(ptn_generator_array_entry_value(generator->send_call_lines, i));
            if (name_value.type != PTN_STRING || arguments_value.type != PTN_ARRAY || indexes_value.type != PTN_ARRAY) {
                continue;
            }
            int kind = PTN_GENERATOR_SEND_CALL_FUNCTION;
            if (kind_value.type == PTN_INT) {
                kind = (int)kind_value.as.integer;
            }
            size_t call_line = line;
            if (line_value.type == PTN_INT && line_value.as.integer >= 0) {
                call_line = (size_t)line_value.as.integer;
            }
            PtnArray *arguments = arguments_value.as.array;
            PtnArray *yield_indexes = indexes_value.as.array;
            size_t argc = arguments == NULL ? 0 : arguments->len;
            PtnValue *call_args = NULL;
            if (argc > 0) {
                call_args = malloc(sizeof(PtnValue) * argc);
                if (call_args == NULL) {
                    ptn_abort_out_of_memory();
                }
                for (size_t arg_index = 0; arg_index < argc; arg_index++) {
                    call_args[arg_index] = ptn_generator_yield_indexes_contains(yield_indexes, arg_index)
                        ? ptn_value_share(sent_value)
                        : ptn_value_share(arguments->entries[arg_index].value);
                }
            }
            char *function_name = ptn_value_to_string(name_value);
            PtnValue call_result = ptn_null();
            if (kind == PTN_GENERATOR_SEND_CALL_CALLABLE) {
                call_result = ptn_call_callable(runtime, receiver_value, argc, call_args, call_line, 0);
            } else if (kind == PTN_GENERATOR_SEND_CALL_METHOD) {
                call_result = ptn_call_declared_method(runtime, receiver_value, function_name, argc, call_args, call_line);
            } else if (kind == PTN_GENERATOR_SEND_CALL_NESTED_FUNCTION) {
                char *inner_function_name = ptn_value_to_string(receiver_value);
                PtnValue inner_result = ptn_call_function(runtime, inner_function_name, argc, call_args, call_line);
                free(inner_function_name);
                if (runtime == NULL || runtime->exceptions->active_exception == NULL) {
                    PtnValue outer_args[] = { ptn_value_share(inner_result) };
                    call_result = ptn_call_function(runtime, function_name, 1, outer_args, call_line);
                    ptn_value_drop(&outer_args[0]);
                }
                ptn_value_destroy(&inner_result);
            } else {
                call_result = ptn_call_function(runtime, function_name, argc, call_args, call_line);
            }
            free(function_name);
            ptn_value_destroy(&call_result);
            for (size_t arg_index = 0; arg_index < argc; arg_index++) {
                ptn_value_drop(&call_args[arg_index]);
            }
            free(call_args);
            if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
                return ptn_null();
            }
        }
    }
#endif

    ptn_generator_apply_resume_return_value(runtime, generator, sent_value);
    PtnValue advanced = ptn_generator_next(runtime, receiver, line);
    ptn_value_destroy(&advanced);
    return ptn_generator_current(runtime, receiver, line);
}

static PTN_UNUSED PtnValue ptn_generator_rewind(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)runtime;
    (void)line;
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (!ptn_generator_guard_not_executing(runtime, generator, line)) {
        return ptn_null();
    }
    if (generator != NULL) {
        if (
            generator->started &&
            !ptn_generator_position_valid(generator) &&
            !generator->has_pending_exception &&
            !generator->pending_exception_on_rewind
        ) {
            ptn_throw_exception(runtime, "Exception", "Cannot traverse an already closed generator");
            return ptn_null();
        }
        generator->started = 1;
        generator->position = 0;
        if (generator->delegate_sources != NULL) {
            for (size_t i = 0; i < generator->delegate_sources->len; i++) {
                PtnGenerator *source = ptn_generator_delegate_source(generator, i);
                if (source != NULL) {
                    source->position = 0;
                }
            }
        }
        if (generator->pending_exception_on_rewind) {
            generator->pending_exception_on_rewind = 0;
            ptn_generator_flush_output_chunk(runtime, generator, generator->pending_exception_position);
            if (ptn_generator_throw_pending_exception_at_position(
                    runtime,
                    generator,
                    generator->pending_exception_position,
                    line,
                    "rewind",
                    1
                )) {
                return ptn_null();
            }
        }
    }
    return ptn_null();
}

static PTN_UNUSED void ptn_generator_set_return_value(PtnRuntime *runtime, PtnGenerator *generator, PtnValue value) {
    if (generator == NULL) {
        return;
    }
    ptn_value_destroy(&generator->return_value);
    generator->return_value = ptn_value_clone_deref(value);
    generator->completed = 1;
    if (!ptn_generator_position_valid(generator)) {
        ptn_generator_flush_pending_output(runtime, generator);
    }
}

static PTN_UNUSED PtnValue ptn_generator_valid(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)line;
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (generator != NULL) {
        if (!generator->started) {
            ptn_generator_skip_exhausted_delegates(generator);
        }
        generator->started = 1;
    }
    int valid = ptn_generator_position_valid(generator);
    if (!valid) {
        ptn_generator_flush_pending_output(runtime, generator);
    }
    return ptn_bool(valid);
}

static PTN_UNUSED void ptn_emit_unpack_traversable_by_ref_warning(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argument_index,
    size_t line
) {
    if (runtime == NULL || !ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Warning: Cannot pass by-reference argument ", stdout);
    fprintf(stdout, "%zu", argument_index);
    fputs(" of ", stdout);
    fputs(function_name != NULL ? function_name : "{closure}", stdout);
    fputs("() by unpacking a Traversable, passing by-value instead in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_array_unpack_invalid_operand_message(
    PtnValue value,
    char *message,
    size_t message_len
) {
    const char *given = ptn_offset_container_type_name(value);
    PtnValue deref_value = ptn_value_deref(value);
    if (deref_value.type == PTN_OBJECT) {
        given = deref_value.as.object->class_name;
    } else if (deref_value.type == PTN_EXCEPTION) {
        given = deref_value.as.exception->class_name;
    } else if (deref_value.type == PTN_CLOSURE) {
        given = "Closure";
    }
    snprintf(
        message,
        message_len,
        "Only arrays and Traversables can be unpacked, %s given",
        given
    );
}

static PTN_UNUSED void ptn_array_unpack_invalid_operand_fatal(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    char message[160];
    ptn_array_unpack_invalid_operand_message(value, message, sizeof(message));
    if (runtime->diagnostics.display_errors) {
        FILE *stream = runtime->diagnostics.stream == NULL ? stderr : runtime->diagnostics.stream;
        fputs("Fatal error: ", stream);
        fputs(message, stream);
        fputs(" in ", stream);
        fputs(runtime->source_path, stream);
        fputs(" on line ", stream);
        fprintf(stream, "%zu", line);
        fputc('\n', stream);
    }
    exit(255);
}

static PTN_UNUSED void ptn_array_unpack_invalid_operand_throw(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    char message[160];
    ptn_array_unpack_invalid_operand_message(value, message, sizeof(message));
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_object_implements_builtin_interface(PtnObject *object, const char *interface_name);
static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_traversable_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line,
    size_t depth
);
static PTN_UNUSED PtnValue ptn_array_iterator_current_key(PtnArrayIterator *iterator);
static PTN_UNUSED PtnValue ptn_array_iterator_current_value(PtnArrayIterator *iterator);
static PTN_UNUSED void ptn_array_iterator_advance(PtnArrayIterator *iterator);
static PTN_UNUSED void ptn_array_iterator_destroy(PtnArrayIterator *iterator);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED int ptn_internal_array_iterator_current_reference(
    PtnRuntime *runtime,
    PtnValue iterator_object,
    size_t line,
    PtnValue *out
);
static PTN_UNUSED PtnArrayIterator ptn_pdo_statement_array_iterator(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
);
#endif

static PTN_UNUSED int ptn_value_is_unpack_traversable(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT || value.as.object == NULL) {
        return 0;
    }
    return ptn_object_is_generator(value.as.object) ||
        ptn_object_implements_builtin_interface(value.as.object, "Iterator") ||
        ptn_object_implements_builtin_interface(value.as.object, "IteratorAggregate");
}

static PTN_UNUSED int ptn_array_unpack_key_from_iterator_value(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    const char *message,
    PtnArrayKey *key_out
) {
    PtnValue key = ptn_value_deref(key_value);
    if (key.type != PTN_INT && key.type != PTN_STRING) {
        ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
        return 0;
    }
    *key_out = ptn_array_key_from_value(key);
    return 1;
}

static PTN_UNUSED void ptn_array_unpack_iterator_into(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnArrayIterator *iterator,
    size_t line
) {
    while (iterator->valid) {
        if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
            return;
        }

        PtnValue key_value = ptn_array_iterator_current_key(iterator);
        PtnValue value = ptn_array_iterator_current_value(iterator);
        PtnArrayKey source_key;
        if (!ptn_array_unpack_key_from_iterator_value(
            runtime,
            key_value,
            line,
            "Keys must be of type int|string during array unpacking",
            &source_key
        )) {
            ptn_value_destroy(&key_value);
            ptn_value_destroy(&value);
            return;
        }

        PtnArrayKey target_key;
        if (source_key.type == PTN_ARRAY_KEY_INT) {
            if (!ptn_array_append_key_available(runtime, target)) {
                ptn_array_key_free(source_key);
                ptn_value_destroy(&key_value);
                ptn_value_destroy(&value);
                return;
            }
            target_key = ptn_array_int_key(target->next_auto_key);
            ptn_array_key_free(source_key);
        } else {
            target_key = source_key;
        }

        ptn_array_set_entry(target, target_key, ptn_value_clone_deref(value));
        ptn_value_destroy(&key_value);
        ptn_value_destroy(&value);
        ptn_array_iterator_advance(iterator);
    }
}

static PTN_UNUSED void ptn_array_unpack_array_into(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnArray *source
) {
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        PtnArrayKey key = entry->key.type == PTN_ARRAY_KEY_INT
            ? ptn_array_int_key(target->next_auto_key)
            : ptn_array_key_clone(entry->key);
        if (entry->key.type == PTN_ARRAY_KEY_INT &&
            !ptn_array_append_key_available(runtime, target)) {
            ptn_array_key_free(key);
            return;
        }
        ptn_array_set_entry(target, key, ptn_value_clone_deref(entry->value));
    }
}

static PTN_UNUSED void ptn_array_unpack_into_common(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line,
    int fatal_on_invalid_operand
) {
    PtnValue source = ptn_value_deref(value);
    if (source.type != PTN_ARRAY) {
        if (ptn_value_is_unpack_traversable(source)) {
            PtnArrayIterator iterator = ptn_array_iterator_from_traversable_object(
                runtime,
                source,
                NULL,
                runtime != NULL ? runtime->source_path : NULL,
                line,
                0
            );
            ptn_array_unpack_iterator_into(runtime, target, &iterator, line);
            ptn_array_iterator_destroy(&iterator);
            return;
        }
        if (fatal_on_invalid_operand) {
            ptn_array_unpack_invalid_operand_fatal(runtime, source, line);
        } else {
            ptn_array_unpack_invalid_operand_throw(runtime, source, line);
        }
        return;
    }

    ptn_array_unpack_array_into(runtime, target, source.as.array);
}

static PTN_UNUSED void ptn_array_unpack_into(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line
) {
    ptn_array_unpack_into_common(runtime, target, value, line, 0);
}

static PTN_UNUSED void ptn_array_unpack_const_invalid(PtnRuntime *runtime, size_t line) {
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Only arrays can be unpacked in constant expression",
        runtime->source_path,
        line
    );
}

static PTN_UNUSED void ptn_array_unpack_const_into(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line
) {
    PtnValue source = ptn_value_deref(value);
    if (source.type != PTN_ARRAY) {
        ptn_array_unpack_const_invalid(runtime, line);
        return;
    }
    ptn_array_unpack_array_into(runtime, target, source.as.array);
}

static PTN_UNUSED void ptn_array_unpack_into_or_fatal(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line
) {
    ptn_array_unpack_into_common(runtime, target, value, line, 1);
}

typedef struct {
    size_t len;
    size_t capacity;
    PtnValue *values;
    char **names;
    int saw_named;
    int use_plain_positional_after_named_message;
} PtnCallArguments;

static PTN_UNUSED void ptn_call_arguments_init(PtnCallArguments *arguments) {
    arguments->len = 0;
    arguments->capacity = 0;
    arguments->values = NULL;
    arguments->names = NULL;
    arguments->saw_named = 0;
    arguments->use_plain_positional_after_named_message = 0;
}

static PTN_UNUSED void ptn_call_arguments_reserve(PtnCallArguments *arguments, size_t additional) {
    if (additional > SIZE_MAX - arguments->len) {
        ptn_abort_out_of_memory();
    }
    size_t needed = arguments->len + additional;
    if (needed <= arguments->capacity) {
        return;
    }
    size_t next_capacity = arguments->capacity == 0 ? 8 : arguments->capacity;
    while (next_capacity < needed) {
        if (next_capacity > SIZE_MAX / 2) {
            next_capacity = needed;
            break;
        }
        next_capacity *= 2;
    }
    PtnValue *values = realloc(arguments->values, next_capacity * sizeof(PtnValue));
    if (values == NULL) {
        ptn_abort_out_of_memory();
    }
    char **names = realloc(arguments->names, next_capacity * sizeof(char *));
    if (names == NULL) {
        ptn_abort_out_of_memory();
    }
    arguments->values = values;
    arguments->names = names;
    arguments->capacity = next_capacity;
}

static PTN_UNUSED int ptn_call_arguments_can_append(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    const char *name,
    size_t line
) {
    if (name == NULL && arguments->saw_named) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            arguments->use_plain_positional_after_named_message
                ? "Cannot use positional argument after named argument"
                : "Cannot use positional argument after named argument during unpacking",
            runtime->source_path,
            line
        );
        return 0;
    }
    return 1;
}

static PTN_UNUSED void ptn_call_arguments_append_named_owned(PtnCallArguments *arguments, const char *name, PtnValue value) {
    ptn_call_arguments_reserve(arguments, 1);
    arguments->values[arguments->len] = value;
    arguments->names[arguments->len] = name == NULL ? NULL : ptn_duplicate_string(name);
    if (name != NULL) {
        arguments->saw_named = 1;
    }
    arguments->len++;
}

static PTN_UNUSED int ptn_call_arguments_append_checked_owned(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    const char *name,
    PtnValue value,
    size_t line
) {
    if (!ptn_call_arguments_can_append(runtime, arguments, name, line)) {
        ptn_value_destroy(&value);
        return 0;
    }
    ptn_call_arguments_append_named_owned(arguments, name, value);
    return 1;
}

static PTN_UNUSED void ptn_call_arguments_append_owned(PtnCallArguments *arguments, PtnValue value) {
    ptn_call_arguments_append_named_owned(arguments, NULL, value);
}

static PTN_UNUSED int ptn_call_argument_index_is_by_ref(
    size_t index,
    const char *name,
    const size_t *by_ref_indices,
    const char *const *by_ref_names,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index,
    size_t *parameter_position_out
) {
    if (name != NULL && by_ref_names != NULL) {
        for (size_t i = 0; i < by_ref_indices_len; i++) {
            if (by_ref_names[i] != NULL && strcmp(by_ref_names[i], name) == 0) {
                if (parameter_position_out != NULL) {
                    *parameter_position_out = by_ref_indices[i] + 1;
                }
                return 1;
            }
        }
    }
    if (has_by_ref_variadic && index >= by_ref_variadic_index) {
        if (parameter_position_out != NULL) {
            *parameter_position_out = index + 1;
        }
        return 1;
    }
    for (size_t i = 0; i < by_ref_indices_len; i++) {
        if (by_ref_indices[i] == index) {
            if (parameter_position_out != NULL) {
                *parameter_position_out = by_ref_indices[i] + 1;
            }
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_call_arguments_unpack_name_from_key_value(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    PtnValue key_value,
    size_t line,
    char **name_out
) {
    *name_out = NULL;
    PtnValue key = ptn_value_deref(key_value);
    if (key.type != PTN_INT && key.type != PTN_STRING) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Keys must be of type int|string during argument unpacking",
            runtime->source_path,
            line
        );
        return 0;
    }

    PtnArrayKey array_key = ptn_array_key_from_value(key);
    if (array_key.type == PTN_ARRAY_KEY_STRING) {
        *name_out = ptn_duplicate_string_len(array_key.as.string, array_key.string_len);
        ptn_array_key_free(array_key);
        return 1;
    }
    ptn_array_key_free(array_key);
    return ptn_call_arguments_can_append(runtime, arguments, NULL, line);
}

static PTN_UNUSED int ptn_call_arguments_unpack_name_from_array_key(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    PtnArrayKey key,
    size_t line,
    char **name_out
) {
    *name_out = NULL;
    if (key.type == PTN_ARRAY_KEY_STRING) {
        *name_out = ptn_duplicate_string_len(key.as.string, key.string_len);
        return 1;
    }
    return ptn_call_arguments_can_append(runtime, arguments, NULL, line);
}

static PTN_UNUSED void ptn_call_arguments_unpack(PtnRuntime *runtime, PtnCallArguments *arguments, PtnValue value, size_t line) {
    PtnValue source = ptn_value_deref(value);
    PtnArray *array = NULL;
    if (source.type == PTN_ARRAY) {
        array = source.as.array;
    }
    if (array == NULL) {
        if (ptn_value_is_unpack_traversable(source)) {
            PtnArrayIterator iterator = ptn_array_iterator_from_traversable_object(
                runtime,
                source,
                NULL,
                runtime != NULL ? runtime->source_path : NULL,
                line,
                0
            );
            while (iterator.valid) {
                if (runtime->exceptions->active_exception != NULL) {
                    break;
                }
                PtnValue key = ptn_array_iterator_current_key(&iterator);
                char *argument_name = NULL;
                if (!ptn_call_arguments_unpack_name_from_key_value(runtime, arguments, key, line, &argument_name)) {
                    ptn_value_destroy(&key);
                    break;
                }
                ptn_value_destroy(&key);

                PtnValue current = ptn_array_iterator_current_value(&iterator);
                ptn_call_arguments_append_named_owned(arguments, argument_name, ptn_value_clone_deref(current));
                free(argument_name);
                ptn_value_destroy(&current);
                ptn_array_iterator_advance(&iterator);
            }
            ptn_array_iterator_destroy(&iterator);
            return;
        }
        char message[160];
        ptn_array_unpack_invalid_operand_message(source, message, sizeof(message));
        ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
        return;
    }

    ptn_call_arguments_reserve(arguments, array->len);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        char *argument_name = NULL;
        if (!ptn_call_arguments_unpack_name_from_array_key(runtime, arguments, entry->key, line, &argument_name)) {
            return;
        }
        ptn_call_arguments_append_named_owned(arguments, argument_name, ptn_value_clone_deref(entry->value));
        free(argument_name);
    }
}

static PTN_UNUSED void ptn_call_arguments_unpack_array_with_parameter_modes(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    PtnValue *source_value,
    const size_t *by_ref_indices,
    const char *const *by_ref_names,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index,
    const char *function_name,
    size_t line
) {
    PtnValue *storage = source_value != NULL && source_value->type == PTN_REFERENCE
        ? &source_value->as.reference->value
        : source_value;
    PtnValue source = storage != NULL ? ptn_value_deref(*storage) : ptn_null();
    PtnArray *array = NULL;
    if (source.type == PTN_ARRAY) {
        array = source.as.array;
    }
    if (array == NULL) {
        if (ptn_value_is_unpack_traversable(source)) {
            PtnArrayIterator iterator = ptn_array_iterator_from_traversable_object(
                runtime,
                source,
                NULL,
                runtime != NULL ? runtime->source_path : NULL,
                line,
                0
            );
            while (iterator.valid) {
                if (runtime->exceptions->active_exception != NULL) {
                    break;
                }
                PtnValue key = ptn_array_iterator_current_key(&iterator);
                char *argument_name = NULL;
                if (!ptn_call_arguments_unpack_name_from_key_value(runtime, arguments, key, line, &argument_name)) {
                    ptn_value_destroy(&key);
                    break;
                }
                ptn_value_destroy(&key);

                PtnValue current = ptn_array_iterator_current_value(&iterator);
                size_t by_ref_parameter_position = arguments->len + 1;
                if (ptn_call_argument_index_is_by_ref(
                    arguments->len,
                    argument_name,
                    by_ref_indices,
                    by_ref_names,
                    by_ref_indices_len,
                    has_by_ref_variadic,
                    by_ref_variadic_index,
                    &by_ref_parameter_position
                )) {
                    ptn_emit_unpack_traversable_by_ref_warning(
                        runtime,
                        function_name,
                        by_ref_parameter_position,
                        line
                    );
                    ptn_call_arguments_append_named_owned(
                        arguments,
                        argument_name,
                        ptn_reference_value(ptn_reference_new_owned(ptn_value_clone_deref(current)))
                    );
                } else {
                    ptn_call_arguments_append_named_owned(arguments, argument_name, ptn_value_clone_deref(current));
                }
                free(argument_name);
                ptn_value_destroy(&current);
                ptn_array_iterator_advance(&iterator);
            }
            ptn_array_iterator_destroy(&iterator);
            return;
        }
        char message[160];
        ptn_array_unpack_invalid_operand_message(source, message, sizeof(message));
        ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
        return;
    }

    int needs_by_ref_entry = 0;
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        char *argument_name = NULL;
        if (!ptn_call_arguments_unpack_name_from_array_key(runtime, arguments, entry->key, line, &argument_name)) {
            free(argument_name);
            return;
        }
        if (ptn_call_argument_index_is_by_ref(
            arguments->len + i,
            argument_name,
            by_ref_indices,
            by_ref_names,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index,
            NULL
        )) {
            needs_by_ref_entry = 1;
            free(argument_name);
            break;
        }
        free(argument_name);
    }
    if (needs_by_ref_entry && storage != NULL) {
        PtnArray *detached = ptn_array_detach_value(storage);
        if (detached != NULL) {
            array = detached;
        } else {
            source = ptn_value_deref(*storage);
            if (source.type == PTN_ARRAY) {
                array = source.as.array;
            }
        }
    }

    ptn_call_arguments_reserve(arguments, array->len);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        char *argument_name = NULL;
        if (!ptn_call_arguments_unpack_name_from_array_key(runtime, arguments, entry->key, line, &argument_name)) {
            return;
        }

        if (ptn_call_argument_index_is_by_ref(
            arguments->len,
            argument_name,
            by_ref_indices,
            by_ref_names,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index,
            NULL
        )) {
            if (entry->value.type != PTN_REFERENCE) {
                PtnValue current = entry->value;
                entry->value = ptn_reference_value(ptn_reference_new_owned(current));
                entry->by_ref_argument_eligible = 1;
            }
            ptn_call_arguments_append_named_owned(arguments, argument_name, ptn_value_clone(entry->value));
        } else {
            ptn_call_arguments_append_named_owned(arguments, argument_name, ptn_value_clone_deref(entry->value));
        }
        free(argument_name);
    }
}

static PTN_UNUSED void ptn_call_arguments_unpack_value_with_parameter_modes(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    PtnValue *value,
    const size_t *by_ref_indices,
    const char *const *by_ref_names,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index,
    const char *function_name,
    size_t line
) {
    ptn_call_arguments_unpack_array_with_parameter_modes(
        runtime,
        arguments,
        value,
        by_ref_indices,
        by_ref_names,
        by_ref_indices_len,
        has_by_ref_variadic,
        by_ref_variadic_index,
        function_name,
        line
    );
}

static PTN_UNUSED void ptn_call_arguments_unpack_variable_with_parameter_modes(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    const char *name,
    const size_t *by_ref_indices,
    const char *const *by_ref_names,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index,
    const char *function_name,
    size_t line
) {
    if (strcmp(name, "GLOBALS") == 0) {
        PtnValue globals = ptn_runtime_globals_snapshot(runtime);
        ptn_call_arguments_unpack_array_with_parameter_modes(
            runtime,
            arguments,
            &globals,
            by_ref_indices,
            by_ref_names,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index,
            function_name,
            line
        );
        ptn_value_destroy(&globals);
        return;
    }

    PtnValue *slot = ptn_symbols_get_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot == NULL) {
        if (ptn_runtime_is_auto_global_symbol_name(name)) {
            ptn_emit_undefined_global_variable_warning(
                &runtime->diagnostics,
                name,
                runtime->source_path,
                line
            );
        } else {
            ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, runtime->source_path, line);
        }
        PtnValue missing = ptn_null();
        ptn_call_arguments_unpack_array_with_parameter_modes(
            runtime,
            arguments,
            &missing,
            by_ref_indices,
            by_ref_names,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index,
            function_name,
            line
        );
        return;
    }

    ptn_call_arguments_unpack_array_with_parameter_modes(
        runtime,
        arguments,
        slot,
        by_ref_indices,
        by_ref_names,
        by_ref_indices_len,
        has_by_ref_variadic,
        by_ref_variadic_index,
        function_name,
        line
    );
}

static PTN_UNUSED void ptn_call_arguments_destroy(PtnCallArguments *arguments) {
    for (size_t i = 0; i < arguments->len; i++) {
        ptn_value_destroy(&arguments->values[i]);
        free(arguments->names[i]);
    }
    free(arguments->values);
    free(arguments->names);
    arguments->values = NULL;
    arguments->names = NULL;
    arguments->len = 0;
    arguments->capacity = 0;
    arguments->saw_named = 0;
}

static PTN_UNUSED PtnArrayEntry *ptn_array_reference_entry(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnValue *key_value,
    size_t line
) {
    if (key_value == NULL) {
        PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
        size_t index = array->len;
        ptn_array_set_entry(array, key, ptn_reference_value(ptn_reference_new_owned(ptn_null())));
        return &array->entries[index];
    }

    ptn_emit_array_offset_key_conversion_diagnostic(runtime, *key_value, line, 1);
    PtnArrayKey key;
    if (!ptn_array_offset_key_from_value(runtime, *key_value, line, 0, &key)) {
        return NULL;
    }
    PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
    if (entry != NULL) {
        ptn_array_key_free(key);
        return entry;
    }

    size_t index = array->len;
    ptn_array_set_entry(array, key, ptn_reference_value(ptn_reference_new_owned(ptn_null())));
    return &array->entries[index];
}

static PTN_UNUSED void ptn_emit_indirect_modification_overloaded_element_notice(
    PtnRuntime *runtime,
    PtnValue container,
    size_t line
);
static PTN_UNUSED PtnValue ptn_arrayaccess_call(
    PtnRuntime *runtime,
    PtnValue container,
    const char *method_name,
    size_t argc,
    PtnValue *args,
    size_t line
);
static PTN_UNUSED void ptn_emit_illegal_string_offset_warning(
    PtnRuntime *runtime,
    const char *key,
    size_t line
);
static PTN_UNUSED int ptn_string_to_offset(const char *string, int64_t *offset, int *warn_illegal);
static PTN_UNUSED int ptn_string_offset_from_value(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int quiet,
    int64_t *offset
);

static PTN_UNUSED int ptn_warn_illegal_string_reference_key(
    PtnRuntime *runtime,
    PtnValue container,
    const PtnValue *key_value,
    size_t line
) {
    PtnValue value = ptn_value_deref(container);
    if (value.type != PTN_STRING || key_value == NULL) {
        return 1;
    }

    int64_t offset = 0;
    return ptn_string_offset_from_value(runtime, *key_value, line, 0, &offset);
}

static PTN_UNUSED PtnFunctionMetadata ptn_arrayaccess_declared_method_metadata(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name
) {
    receiver = ptn_value_deref(receiver);
    if (
        runtime == NULL ||
        runtime->declared_method_metadata == NULL ||
        receiver.type != PTN_OBJECT ||
        receiver.as.object == NULL ||
        receiver.as.object->class_name == NULL ||
        method_name == NULL
    ) {
        return ptn_function_metadata_not_found();
    }
    return runtime->declared_method_metadata(receiver.as.object->class_name, method_name);
}

static PTN_UNUSED void ptn_arrayaccess_throw_missing_offset_get_argument(
    PtnRuntime *runtime,
    PtnValue receiver,
    PtnFunctionMetadata metadata,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    const char *class_name = receiver.type == PTN_OBJECT && receiver.as.object != NULL
        ? receiver.as.object->class_name
        : "ArrayAccess";
    const char *parameter_name = "offset";
    if (
        metadata.found &&
        metadata.parameter_count > 0 &&
        metadata.parameters != NULL &&
        metadata.parameters[0].name != NULL
    ) {
        parameter_name = metadata.parameters[0].name;
    }

    int needed = snprintf(
        NULL,
        0,
        "%s::offsetGet(): Argument #1 ($%s) not passed",
        class_name,
        parameter_name
    );
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(
        message,
        (size_t)needed + 1,
        "%s::offsetGet(): Argument #1 ($%s) not passed",
        class_name,
        parameter_name
    );
    ptn_throw_exception_owned_message_at(
        runtime,
        "ArgumentCountError",
        message,
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
}

static PTN_UNUSED int ptn_arrayaccess_append_reference_temporary(
    PtnRuntime *runtime,
    PtnValue container,
    size_t line,
    PtnValue *reference_out
) {
    PtnValue value = ptn_value_deref(container);
    if (
        value.type == PTN_OBJECT &&
        value.as.object != NULL &&
        (ptn_ascii_case_equal(value.as.object->class_name, "DOMNamedNodeMap") ||
            ptn_ascii_case_equal(value.as.object->class_name, "Dom\\NamedNodeMap") ||
            ptn_ascii_case_equal(value.as.object->class_name, "DOMTokenList") ||
            ptn_ascii_case_equal(value.as.object->class_name, "Dom\\TokenList"))
    ) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "Cannot append to %s",
            value.as.object->class_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        if (reference_out != NULL) {
            *reference_out = ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        return 1;
    }
    if (
        value.type == PTN_OBJECT &&
        value.as.object != NULL &&
        ptn_object_is_internal_or_descendant(value, "SplFixedArray")
    ) {
        ptn_throw_exception(runtime, "Error", "[] operator not supported for SplFixedArray");
        if (reference_out != NULL) {
            *reference_out = ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        return 1;
    }
    if (
        value.type != PTN_OBJECT ||
        value.as.object == NULL ||
        !ptn_object_is_internal_or_descendant(value, "ArrayObject")
    ) {
        return 0;
    }

    int exact_array_object = ptn_ascii_case_equal(value.as.object->class_name, "ArrayObject");
    int has_declared_offset_get =
        !exact_array_object &&
        runtime != NULL &&
        runtime->declared_method_exists != NULL &&
        runtime->declared_method_exists(value.as.object->class_name, "offsetGet");
    if (has_declared_offset_get) {
        PtnFunctionMetadata metadata =
            ptn_arrayaccess_declared_method_metadata(runtime, value, "offsetGet");
        if (metadata.found && metadata.return_by_ref) {
            PtnValue arg = ptn_null();
            PtnValue result = ptn_arrayaccess_call(runtime, value, "offsetGet", 1, &arg, line);
            if (reference_out != NULL) {
                *reference_out = result.type == PTN_REFERENCE
                    ? result
                    : ptn_reference_value(ptn_reference_new_owned(result));
            } else {
                ptn_value_destroy(&result);
            }
            return 1;
        }
        if (metadata.found && metadata.required_parameter_count > 0) {
            ptn_emit_indirect_modification_overloaded_element_notice(runtime, value, line);
            ptn_arrayaccess_throw_missing_offset_get_argument(runtime, value, metadata, line);
            if (reference_out != NULL) {
                *reference_out = ptn_reference_value(ptn_reference_new_owned(ptn_null()));
            }
            return 1;
        }
        PtnValue result = ptn_arrayaccess_call(runtime, value, "offsetGet", 0, NULL, line);
        if (result.type == PTN_REFERENCE) {
            if (reference_out != NULL) {
                *reference_out = result;
            } else {
                ptn_value_destroy(&result);
            }
            return 1;
        }
        ptn_emit_indirect_modification_overloaded_element_notice(runtime, value, line);
        if (reference_out != NULL) {
            *reference_out = ptn_reference_value(ptn_reference_new_owned(result));
        } else {
            ptn_value_destroy(&result);
        }
        return 1;
    }

    ptn_emit_indirect_modification_overloaded_element_notice(runtime, value, line);
    if (reference_out != NULL) {
        *reference_out = ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    return 1;
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_array_dim(
    PtnRuntime *runtime,
    const char *name,
    const PtnValue *key_value,
    const char *path,
    size_t line
) {
    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot != NULL) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            PtnValue append_reference = ptn_null();
            if (
                key_value == NULL &&
                ptn_arrayaccess_append_reference_temporary(runtime, slot_value, line, &append_reference)
            ) {
                return append_reference;
            }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            PtnValue reference = ptn_null();
            if (ptn_internal_array_object_offset_reference(
                runtime,
                slot_value,
                key_value,
                line,
                1,
                &reference
            )) {
                return reference;
            }
#endif
            PtnValue key = key_value == NULL ? ptn_null() : *key_value;
            PtnValue value = ptn_arrayaccess_read(runtime, slot_value, key, line);
            if (value.type == PTN_REFERENCE) {
                return value;
            }
            ptn_emit_indirect_modification_overloaded_element_notice(runtime, slot_value, line);
            return ptn_reference_value(ptn_reference_new_owned(value));
        }
        if (slot_value.type == PTN_STRING && key_value == NULL) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", path, line);
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        if (!ptn_warn_illegal_string_reference_key(runtime, slot_value, key_value, line)) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }

    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    if (key_value == NULL && !ptn_array_append_key_available(runtime, array)) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    PtnArrayEntry *entry = ptn_array_reference_entry(runtime, array, key_value, line);
    if (entry == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
        entry->by_ref_argument_eligible = 1;
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_array_value_dim(
    PtnRuntime *runtime,
    PtnValue *container,
    const PtnValue *key_value,
    const char *path,
    size_t line,
    int warn_non_referenceable
) {
    if (container == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    if (warn_non_referenceable && container->type != PTN_REFERENCE) {
        ptn_emit_attempting_to_set_reference_to_non_referenceable_value_notice(
            &runtime->diagnostics,
            line
        );
    }

    PtnValue *value = container->type == PTN_REFERENCE
        ? &container->as.reference->value
        : container;
    if (ptn_arrayaccess_can_dispatch(runtime, *value, "offsetGet")) {
        PtnValue append_reference = ptn_null();
        if (
            key_value == NULL &&
            ptn_arrayaccess_append_reference_temporary(runtime, *value, line, &append_reference)
        ) {
            return append_reference;
        }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        PtnValue reference = ptn_null();
        if (ptn_internal_array_object_offset_reference(
            runtime,
            *value,
            key_value,
            line,
            1,
            &reference
        )) {
            return reference;
        }
#endif
        PtnValue key = key_value == NULL ? ptn_null() : *key_value;
        PtnValue result = ptn_arrayaccess_read(runtime, *value, key, line);
        if (result.type == PTN_REFERENCE) {
            return result;
        }
        if (warn_non_referenceable) {
            ptn_emit_indirect_modification_overloaded_element_notice(runtime, *value, line);
        }
        return ptn_reference_value(ptn_reference_new_owned(result));
    }

    PtnArray *array = NULL;
    if (value->type == PTN_ARRAY) {
        array = ptn_array_detach_value(value);
    } else if (value->type == PTN_STRING) {
        if (key_value == NULL) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", path, line);
        } else {
            if (ptn_warn_illegal_string_reference_key(runtime, *value, key_value, line)) {
                ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
            }
        }
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    } else if ((array = ptn_array_convertible_scalar_for_write(runtime, value, line)) != NULL) {
        /* false/null conversion handled by shared lvalue write semantics. */
    } else if (ptn_value_is_plain_object_for_array_offset(runtime, *value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *value, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    } else {
        ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    if (key_value == NULL && !ptn_array_append_key_available(runtime, array)) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    PtnArrayEntry *entry = ptn_array_reference_entry(runtime, array, key_value, line);
    if (entry == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
        entry->by_ref_argument_eligible = 1;
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_runtime_bind_array_dim_reference(
    PtnRuntime *runtime,
    const char *name,
    const PtnValue *key_value,
    PtnValue reference,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return;
    }
    if (key_value == NULL && !ptn_array_append_key_available(runtime, array)) {
        return;
    }
    PtnArrayKey key;
    if (key_value == NULL) {
        key = ptn_array_int_key(array->next_auto_key);
    } else {
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, *key_value, line, 1);
        if (!ptn_array_offset_key_from_value(runtime, *key_value, line, 0, &key)) {
            return;
        }
    }
    ptn_array_set_entry(array, key, ptn_value_clone(reference));
}

static PTN_UNUSED int ptn_object_property_visible_for_foreach(
    PtnRuntime *runtime,
    PtnObject *object,
    PtnArrayKey key,
    const char *access_scope
) {
    if (object == NULL || key.type != PTN_ARRAY_KEY_STRING) {
        return 1;
    }
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(object, key.as.string);
    const char *display_name = metadata != NULL ? metadata->display_name : key.as.string;
    const PtnObjectPropertyMetadata *scoped_private =
        ptn_object_private_property_for_scope(object, display_name, access_scope);
    if (scoped_private != NULL && strcmp(scoped_private->storage_name, key.as.string) != 0) {
        return 0;
    }
    if (metadata == NULL || metadata->read_visibility == PTN_PROPERTY_PUBLIC) {
        return 1;
    }
    return ptn_property_visibility_allows(
        runtime,
        metadata->read_visibility,
        ptn_property_visibility_scope_class(metadata, metadata->read_visibility),
        access_scope
    );
}

static PTN_UNUSED int ptn_object_property_iterable_for_foreach(
    PtnRuntime *runtime,
    PtnObject *object,
    PtnArrayKey key,
    const char *access_scope
) {
    if (!ptn_object_property_visible_for_foreach(runtime, object, key, access_scope)) {
        return 0;
    }
    if (object == NULL || key.type != PTN_ARRAY_KEY_STRING) {
        return 1;
    }
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(object, key.as.string);
    if (metadata != NULL) {
        if (ptn_property_is_set_only_virtual(metadata)) {
            return 0;
        }
        return ptn_object_property_storage_initialized(object, metadata->storage_name) ||
            (metadata->has_hooks && metadata->hook_has_get);
    }
    if (object->properties == NULL) {
        return 0;
    }
    return ptn_array_find_key(object->properties, key) < object->properties->len;
}

static PTN_UNUSED PtnValue ptn_object_foreach_key_value(
    PtnObject *object,
    PtnArrayKey key
) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    if (object != NULL) {
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(object, key.as.string);
        if (metadata != NULL) {
            return ptn_string(metadata->display_name);
        }
    }
    if (key.string_len >= 3 && key.as.string[0] == '\0') {
        const char *second_nul = memchr(key.as.string + 1, '\0', key.string_len - 1);
        if (second_nul != NULL) {
            size_t prefix_len = (size_t)(second_nul - key.as.string) + 1;
            if (prefix_len <= key.string_len) {
                size_t display_len = key.string_len - prefix_len;
                return ptn_owned_string_len(
                    ptn_duplicate_string_len(key.as.string + prefix_len, display_len),
                    display_len
                );
            }
        }
    }
    return ptn_owned_string_len(ptn_duplicate_string_len(key.as.string, key.string_len), key.string_len);
}

static PTN_UNUSED char *ptn_object_foreach_property_name(
    PtnObject *object,
    PtnArrayKey key
) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        int needed = snprintf(NULL, 0, "%lld", (long long)key.as.integer);
        if (needed < 0) {
            ptn_abort_out_of_memory();
        }
        char *name = malloc((size_t)needed + 1);
        if (name == NULL) {
            ptn_abort_out_of_memory();
        }
        snprintf(name, (size_t)needed + 1, "%lld", (long long)key.as.integer);
        return name;
    }
    if (object != NULL) {
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(object, key.as.string);
        if (metadata != NULL) {
            return ptn_duplicate_string(metadata->display_name);
        }
    }
    if (key.string_len >= 3 && key.as.string[0] == '\0') {
        const char *second_nul = memchr(key.as.string + 1, '\0', key.string_len - 1);
        if (second_nul != NULL) {
            size_t prefix_len = (size_t)(second_nul - key.as.string) + 1;
            if (prefix_len <= key.string_len) {
                size_t display_len = key.string_len - prefix_len;
                return ptn_duplicate_string_len(key.as.string + prefix_len, display_len);
            }
        }
    }
    return ptn_duplicate_string_len(key.as.string, key.string_len);
}

static PTN_UNUSED void ptn_array_iterator_skip_invisible_object_properties(PtnArrayIterator *iterator) {
    if (!iterator->object_property_iterator || iterator->object == NULL || iterator->array == NULL) {
        return;
    }
    size_t limit = iterator->live ? iterator->array->len : iterator->length;
    while (iterator->index < limit &&
        !ptn_object_property_iterable_for_foreach(
            iterator->runtime,
            iterator->object,
            iterator->array->entries[iterator->index].key,
            iterator->access_scope
        )) {
        iterator->index++;
        limit = iterator->live ? iterator->array->len : iterator->length;
    }
    iterator->valid = iterator->index < limit;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_empty(void) {
    PtnArrayIterator iterator;
    iterator.array = NULL;
    iterator.object = NULL;
    iterator.generator = NULL;
    iterator.runtime = NULL;
    iterator.access_scope = NULL;
    iterator.iterator_object = ptn_null();
    iterator.index = 0;
    iterator.length = 0;
    iterator.current_key = ptn_array_int_key(0);
    iterator.current_reference = NULL;
    iterator.watched_slot = NULL;
    iterator.line = 0;
    iterator.seen_mutation_epoch = 0;
    iterator.has_current_key = 0;
    iterator.has_iterator_object = 0;
    iterator.protocol_iterator = 0;
    iterator.spl_dllist_delete = 0;
    iterator.spl_dllist_reverse = 0;
    iterator.object_property_iterator = 0;
    iterator.valid = 0;
    iterator.live = 0;
    return iterator;
}

static PTN_UNUSED size_t ptn_array_iterator_effective_index(PtnArrayIterator *iterator) {
    if (iterator == NULL || iterator->array == NULL) {
        return 0;
    }
    if (!iterator->spl_dllist_reverse) {
        return iterator->index;
    }
    if (iterator->index >= iterator->array->len) {
        return iterator->array->len;
    }
    return iterator->array->len - 1 - iterator->index;
}

static PTN_UNUSED void ptn_array_iterator_clear_current_key(PtnArrayIterator *iterator) {
    if (!iterator->has_current_key) {
        return;
    }
    ptn_array_key_free(iterator->current_key);
    iterator->current_key = ptn_array_int_key(0);
    iterator->current_reference = NULL;
    iterator->has_current_key = 0;
}

static PTN_UNUSED void ptn_array_iterator_remember_current_key(PtnArrayIterator *iterator) {
    ptn_array_iterator_clear_current_key(iterator);
    if (iterator->array != NULL) {
        iterator->seen_mutation_epoch = iterator->array->iterator_mutation_epoch;
    }
    if (
        iterator->array == NULL ||
        !iterator->valid ||
        ptn_array_iterator_effective_index(iterator) >= iterator->array->len
    ) {
        return;
    }
    size_t physical_index = ptn_array_iterator_effective_index(iterator);
    iterator->current_key = ptn_array_key_clone(iterator->array->entries[physical_index].key);
    iterator->current_reference = iterator->array->entries[physical_index].value.type == PTN_REFERENCE
        ? iterator->array->entries[physical_index].value.as.reference
        : NULL;
    iterator->has_current_key = 1;
    if (iterator->live) {
        iterator->array->has_iterator_current_index = 1;
        iterator->array->iterator_current_index = iterator->index;
    }
}

static PTN_UNUSED int ptn_object_implements_builtin_interface(PtnObject *object, const char *interface_name) {
    return object != NULL &&
        (ptn_declared_class_implements_interface(object->class_name, interface_name) ||
         ptn_builtin_class_implements_interface(object->class_name, interface_name));
}

static PTN_UNUSED int ptn_object_supports_foreach_by_reference(PtnObject *object) {
    return object != NULL &&
        object->class_name != NULL &&
        (
            ptn_ascii_case_equal(object->class_name, "ArrayObject") ||
            ptn_ascii_case_equal(object->class_name, "ArrayIterator") ||
            ptn_ascii_case_equal(object->class_name, "RecursiveArrayIterator") ||
            ptn_ascii_case_equal(object->class_name, "WeakMap")
        );
}

static PTN_UNUSED int ptn_object_has_iterator_method(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *method_name
) {
    if (runtime == NULL || runtime->method_dispatch == NULL || object == NULL) {
        return 0;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (
        ptn_internal_class_exists_name(object->class_name) &&
        ptn_internal_class_method_exists(object->class_name, method_name)
    ) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "ArrayObject") &&
        ptn_internal_class_method_exists("ArrayObject", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "ArrayIterator") &&
        ptn_internal_class_method_exists("ArrayIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "EmptyIterator") &&
        ptn_internal_class_method_exists("EmptyIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "SplFixedArray") &&
        ptn_internal_class_method_exists("SplFixedArray", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "CachingIterator") &&
        ptn_internal_class_method_exists("CachingIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "RegexIterator") &&
        ptn_internal_class_method_exists("RegexIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "FilterIterator") &&
        ptn_internal_class_method_exists("FilterIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "IteratorIterator") &&
        ptn_internal_class_method_exists("IteratorIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "InfiniteIterator") &&
        ptn_internal_class_method_exists("InfiniteIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "NoRewindIterator") &&
        ptn_internal_class_method_exists("NoRewindIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "RecursiveArrayIterator") &&
        ptn_internal_class_method_exists("RecursiveArrayIterator", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "SplDoublyLinkedList") &&
        ptn_internal_class_method_exists("SplDoublyLinkedList", method_name)) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "SplObjectStorage") &&
        ptn_internal_class_method_exists("SplObjectStorage", method_name)) {
        return 1;
    }
    if ((ptn_declared_class_is_same_or_descendant(object->class_name, "SplHeap") ||
         ptn_declared_class_is_same_or_descendant(object->class_name, "SplPriorityQueue")) &&
        (ptn_internal_class_method_exists("SplHeap", method_name) ||
         ptn_internal_class_method_exists("SplPriorityQueue", method_name))) {
        return 1;
    }
    if (ptn_declared_class_is_same_or_descendant(object->class_name, "SplFileObject") &&
        ptn_internal_class_method_exists("SplFileObject", method_name)) {
        return 1;
    }
#endif
    return runtime->declared_method_exists != NULL &&
        runtime->declared_method_exists(object->class_name, method_name);
}

static PTN_UNUSED int ptn_object_declares_iterator_method(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *method_name
) {
    return runtime != NULL &&
        runtime->declared_method_exists != NULL &&
        object != NULL &&
        object->class_name != NULL &&
        runtime->declared_method_exists(object->class_name, method_name);
}

static PTN_UNUSED PtnValue ptn_protocol_iterator_call(
    PtnArrayIterator *iterator,
    const char *method_name
) {
    if (
        iterator->runtime == NULL ||
        iterator->runtime->method_dispatch == NULL ||
        !iterator->has_iterator_object
    ) {
        return ptn_null();
    }
    PtnValue result = iterator->runtime->method_dispatch(
        iterator->runtime,
        iterator->iterator_object,
        method_name,
        0,
        NULL,
        iterator->line
    );
    if (iterator->runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&result);
        ptn_rethrow_exception(iterator->runtime);
        return ptn_null();
    }
    return result;
}

static PTN_UNUSED void ptn_protocol_iterator_refresh_valid(PtnArrayIterator *iterator) {
    PtnValue valid = ptn_protocol_iterator_call(iterator, "valid");
    iterator->valid = ptn_is_truthy(valid);
    ptn_value_destroy(&valid);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_array_snapshot(PtnArray *array) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (array == NULL) {
        return iterator;
    }
    iterator.array = ptn_array_clone(array);
    ptn_array_iterator_retain(iterator.array);
    iterator.length = iterator.array->len;
    iterator.valid = iterator.length != 0;
    ptn_array_iterator_remember_current_key(&iterator);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_generator(
    PtnRuntime *runtime,
    PtnObject *object,
    int by_ref,
    const char *path,
    size_t line
) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (!ptn_object_is_generator(object)) {
        return iterator;
    }
    PtnGenerator *generator = (PtnGenerator *)object->native_data;
    if (by_ref && !generator->yields_by_ref) {
        ptn_throw_exception_at(
            runtime,
            "Exception",
            "You can only iterate a generator by-reference if it declared that it yields by-reference",
            path,
            line
        );
        return iterator;
    }
    iterator.array = generator->values;
    iterator.object = object;
    iterator.generator = generator;
    iterator.runtime = runtime;
    generator->started = 1;
    ptn_generator_skip_exhausted_delegates(generator);
    iterator.index = generator->position;
    iterator.valid = iterator.array != NULL &&
        generator->keys != NULL &&
        iterator.index < iterator.array->len &&
        generator->keys->len >= iterator.array->len;
    if (!iterator.valid && generator->has_pending_exception) {
        size_t last_index = 0;
        if (ptn_generator_pending_exception_after_last_yield(generator, &last_index)) {
            iterator.index = last_index;
            iterator.valid = iterator.array != NULL &&
                generator->keys != NULL &&
                iterator.index < iterator.array->len &&
                generator->keys->len >= iterator.array->len;
        }
        if (!iterator.valid) {
            ptn_generator_throw_pending_exception_at_position(
                runtime,
                generator,
                generator->pending_exception_position,
                line,
                "rewind",
                1
            );
        }
    }
    iterator.live = 1;
    ptn_object_retain(object);
    ptn_array_iterator_retain(iterator.array);
    ptn_array_iterator_remember_current_key(&iterator);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_protocol_iterator(
    PtnRuntime *runtime,
    PtnValue iterator_value,
    const char *access_scope,
    size_t line
) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    iterator.runtime = runtime;
    iterator.access_scope = access_scope;
    iterator.iterator_object = ptn_value_clone_deref(iterator_value);
    iterator.has_iterator_object = 1;
    iterator.protocol_iterator = 1;
    iterator.line = line;

    PtnTryFrame iterator_frame;
    int iterator_frame_active = 0;
    if (runtime != NULL && runtime->exceptions != NULL) {
        ptn_try_frame_push(runtime, &iterator_frame);
        iterator_frame_active = 1;
        if (setjmp(iterator_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &iterator_frame);
            ptn_array_iterator_destroy(&iterator);
            ptn_rethrow_exception(runtime);
            return ptn_array_iterator_empty();
        }
    }
    if (ptn_object_has_iterator_method(runtime, iterator.iterator_object.as.object, "rewind")) {
        PtnValue rewind = ptn_protocol_iterator_call(&iterator, "rewind");
        ptn_value_destroy(&rewind);
    }
    ptn_protocol_iterator_refresh_valid(&iterator);
    if (iterator_frame_active) {
        ptn_try_frame_pop(runtime, &iterator_frame);
    }
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_traversable_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line,
    size_t depth
);

static PTN_UNUSED PtnObject *ptn_lazy_object_foreach_effective_object(
    PtnRuntime *runtime,
    PtnObject *object,
    size_t line
) {
    for (size_t depth = 0; object != NULL && depth < 64; depth++) {
        if (object->lazy_uninitialized && !object->lazy_initializing) {
            if (!ptn_lazy_object_initialize(runtime, ptn_value_borrow(ptn_object(object)), line)) {
                return NULL;
            }
        }
        if (!object->lazy_is_proxy || object->lazy_uninitialized) {
            return object;
        }
        PtnValue real = ptn_value_deref(object->lazy_proxy_instance);
        if (real.type != PTN_OBJECT || real.as.object == NULL || real.as.object == object) {
            return object;
        }
        object = real.as.object;
    }
    return object;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_traversable_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line,
    size_t depth
);

static PTN_UNUSED void ptn_iteratoraggregate_invalid_result_throw(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *path,
    size_t line
) {
    const char *class_name = object != NULL && object->class_name != NULL
        ? object->class_name
        : "IteratorAggregate";
    char message[256];
    snprintf(
        message,
        sizeof(message),
        "Objects returned by %s::getIterator() must be traversable or implement interface Iterator",
        class_name
    );
    ptn_throw_exception_at(runtime, "Exception", message, path, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_object_properties(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *access_scope,
    size_t line
) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (object == NULL) {
        return iterator;
    }
    PtnValue keys_value = ptn_array_from_literal_entries(0, NULL);
    PtnArray *keys = keys_value.as.array;
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        const PtnObjectPropertyMetadata *metadata = &object->property_metadata[i];
        PtnArrayKey key = ptn_array_string_key(metadata->storage_name);
        if (ptn_object_property_iterable_for_foreach(runtime, object, key, access_scope)) {
            ptn_array_set_entry(keys, key, ptn_null());
        } else {
            ptn_array_key_free(key);
        }
    }
    if (object->properties != NULL) {
        for (size_t i = 0; i < object->properties->len; i++) {
            PtnArrayEntry *entry = &object->properties->entries[i];
            const PtnObjectPropertyMetadata *metadata =
                entry->key.type == PTN_ARRAY_KEY_STRING
                    ? ptn_object_property_metadata(object, entry->key.as.string)
                    : NULL;
            if (metadata != NULL) {
                continue;
            }
            if (ptn_object_property_visible_for_foreach(runtime, object, entry->key, access_scope)) {
                ptn_array_set_entry(keys, ptn_array_key_clone(entry->key), ptn_null());
            }
        }
    }
    iterator.array = keys;
    ptn_array_iterator_retain(iterator.array);
    iterator.object = object;
    iterator.runtime = runtime;
    iterator.access_scope = access_scope;
    iterator.line = line;
    iterator.valid = iterator.array->len != 0;
    iterator.length = iterator.array->len;
    iterator.object_property_iterator = 1;
    iterator.live = 0;
    ptn_object_retain(object);
    ptn_array_iterator_remember_current_key(&iterator);
    return iterator;
}

static PTN_UNUSED void ptn_array_iterator_sync_object_property_keys(PtnArrayIterator *iterator) {
    if (iterator == NULL ||
        !iterator->object_property_iterator ||
        iterator->object == NULL ||
        iterator->generator != NULL ||
        iterator->array == NULL ||
        iterator->object->properties == NULL) {
        return;
    }
    for (size_t i = 0; i < iterator->object->properties->len; i++) {
        PtnArrayEntry *entry = &iterator->object->properties->entries[i];
        const PtnObjectPropertyMetadata *metadata =
            entry->key.type == PTN_ARRAY_KEY_STRING
                ? ptn_object_property_metadata(iterator->object, entry->key.as.string)
                : NULL;
        if (!ptn_object_property_iterable_for_foreach(
                iterator->runtime,
                iterator->object,
                entry->key,
                iterator->access_scope
            )) {
            continue;
        }
        if (ptn_array_find_key(iterator->array, entry->key) < iterator->array->len) {
            continue;
        }
        ptn_array_set_entry(iterator->array, ptn_array_key_clone(entry->key), ptn_null());
    }
    iterator->length = iterator->array->len;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_traversable_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line,
    size_t depth
) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT) {
        return ptn_array_iterator_empty();
    }
    PtnObject *effective_object =
        ptn_lazy_object_foreach_effective_object(runtime, value.as.object, line);
    if (effective_object == NULL) {
        return ptn_array_iterator_empty();
    }
    if (effective_object != value.as.object) {
        value = ptn_value_borrow(ptn_object(effective_object));
    }
    if (ptn_object_is_generator(value.as.object)) {
        return ptn_array_iterator_from_generator(runtime, value.as.object, 0, path, line);
    }

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_date_value_is_uninitialized_descendant(value, "DatePeriod")) {
        ptn_date_throw_uninitialized_named_object_error(runtime, "DatePeriod");
        return ptn_array_iterator_empty();
    }
#endif

    int has_declared_get_iterator =
        ptn_object_declares_iterator_method(runtime, value.as.object, "getIterator");

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (!has_declared_get_iterator &&
        ptn_internal_class_name_is_spl_fixed_array(value.as.object->class_name)) {
        PtnArrayIterator iterator = ptn_array_iterator_empty();
        if (ptn_spl_fixed_array_iterator_from_object(
            runtime,
            value,
            access_scope,
            line,
            &iterator
        )) {
            return iterator;
        }
        if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
            return iterator;
        }
    }

    if (ptn_object_is_internal_or_descendant(value, "SplDoublyLinkedList")) {
        PtnArrayIterator iterator = ptn_array_iterator_empty();
        if (ptn_spl_doubly_linked_list_iterator_from_object(
            runtime,
            value,
            access_scope,
            line,
            &iterator
        )) {
            return iterator;
        }
        if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
            return iterator;
        }
    }

    if (ptn_object_is_internal_or_descendant(value, "PDOStatement")) {
        return ptn_pdo_statement_array_iterator(runtime, value, line);
    }
#endif

    if (
        ptn_object_implements_builtin_interface(value.as.object, "IteratorAggregate") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "getIterator")
    ) {
        if (depth > 16) {
            ptn_throw_exception(runtime, "Exception", "IteratorAggregate recursion limit exceeded");
            return ptn_array_iterator_empty();
        }
        PtnValue result = runtime->method_dispatch(runtime, value, "getIterator", 0, NULL, line);
        if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
            ptn_value_destroy(&result);
            ptn_rethrow_exception(runtime);
            return ptn_array_iterator_empty();
        }
        PtnValue resolved = ptn_value_deref(result);
        PtnArrayIterator iterator = ptn_array_iterator_empty();
        if (resolved.type == PTN_OBJECT && ptn_object_is_generator(resolved.as.object)) {
            iterator = ptn_array_iterator_from_generator(runtime, resolved.as.object, 0, path, line);
            iterator.iterator_object = ptn_value_clone_deref(value);
            iterator.has_iterator_object = 1;
        } else if (
            resolved.type == PTN_OBJECT &&
            (
                ptn_object_implements_builtin_interface(resolved.as.object, "Iterator") ||
                ptn_object_implements_builtin_interface(resolved.as.object, "IteratorAggregate")
            )
        ) {
            iterator = ptn_array_iterator_from_traversable_object(
                runtime,
                resolved,
                access_scope,
                path,
                line,
                depth + 1
            );
        } else {
            ptn_iteratoraggregate_invalid_result_throw(runtime, value.as.object, path, line);
        }
        if (
            iterator.has_iterator_object &&
            iterator.iterator_object.type == PTN_OBJECT &&
            resolved.type == PTN_OBJECT &&
            iterator.iterator_object.as.object == resolved.as.object
        ) {
            iterator.iterator_object.as.object->defer_object_id_release_once = 1;
        }
        ptn_value_destroy(&result);
        return iterator;
    }

    if (
        ptn_object_implements_builtin_interface(value.as.object, "Iterator") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "valid") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "current") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "next")
    ) {
        return ptn_array_iterator_from_protocol_iterator(runtime, value, access_scope, line);
    }

    return ptn_array_iterator_from_object_properties(runtime, value.as.object, access_scope, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_traversable_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line,
    size_t depth
) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT) {
        return ptn_array_iterator_empty();
    }
    PtnObject *effective_object =
        ptn_lazy_object_foreach_effective_object(runtime, value.as.object, line);
    if (effective_object == NULL) {
        return ptn_array_iterator_empty();
    }
    if (effective_object != value.as.object) {
        value = ptn_value_borrow(ptn_object(effective_object));
    }
    if (ptn_object_is_generator(value.as.object)) {
        return ptn_array_iterator_from_generator(runtime, value.as.object, 1, path, line);
    }

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_object_is_internal_or_descendant(value, "PDOStatement")) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "An iterator cannot be used with foreach by reference",
            path,
            line
        );
        return ptn_array_iterator_empty();
    }
#endif

    if (
        ptn_object_implements_builtin_interface(value.as.object, "IteratorAggregate") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "getIterator")
    ) {
        if (depth > 16) {
            ptn_throw_exception(runtime, "Exception", "IteratorAggregate recursion limit exceeded");
            return ptn_array_iterator_empty();
        }
        PtnValue result = runtime->method_dispatch(runtime, value, "getIterator", 0, NULL, line);
        if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
            ptn_value_destroy(&result);
            ptn_rethrow_exception(runtime);
            return ptn_array_iterator_empty();
        }
        PtnValue resolved = ptn_value_deref(result);
        PtnArrayIterator iterator = ptn_array_iterator_empty();
        if (resolved.type == PTN_OBJECT && ptn_object_is_generator(resolved.as.object)) {
            iterator = ptn_array_iterator_from_generator(runtime, resolved.as.object, 1, path, line);
            iterator.iterator_object = ptn_value_clone_deref(value);
            iterator.has_iterator_object = 1;
        } else if (
            resolved.type == PTN_OBJECT &&
            ptn_object_implements_builtin_interface(resolved.as.object, "IteratorAggregate")
        ) {
            iterator = ptn_array_iterator_by_ref_from_traversable_object(
                runtime,
                resolved,
                access_scope,
                path,
                line,
                depth + 1
            );
        } else if (
            resolved.type == PTN_OBJECT &&
            ptn_object_implements_builtin_interface(resolved.as.object, "Iterator") &&
            ptn_object_supports_foreach_by_reference(resolved.as.object)
        ) {
            iterator = ptn_array_iterator_from_traversable_object(
                runtime,
                resolved,
                access_scope,
                path,
                line,
                depth + 1
            );
        } else if (
            resolved.type == PTN_OBJECT &&
            ptn_object_implements_builtin_interface(resolved.as.object, "Iterator")
        ) {
            ptn_throw_exception_at(
                runtime,
                "Error",
                "An iterator cannot be used with foreach by reference",
                path,
                line
            );
        } else {
            ptn_iteratoraggregate_invalid_result_throw(runtime, value.as.object, path, line);
        }
        ptn_value_destroy(&result);
        return iterator;
    }

    if (
        ptn_object_implements_builtin_interface(value.as.object, "Iterator") &&
        ptn_object_supports_foreach_by_reference(value.as.object)
    ) {
        return ptn_array_iterator_from_traversable_object(runtime, value, access_scope, path, line, depth);
    }

    if (
        ptn_object_implements_builtin_interface(value.as.object, "Iterator") ||
        ptn_object_implements_builtin_interface(value.as.object, "IteratorAggregate")
    ) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "An iterator cannot be used with foreach by reference",
            path,
            line
        );
        return ptn_array_iterator_empty();
    }

    return ptn_array_iterator_from_object_properties(runtime, value.as.object, access_scope, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_value(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line
) {
    value = ptn_value_deref(value);
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (value.type != PTN_ARRAY) {
        if (value.type == PTN_CLOSURE) {
            return iterator;
        }
        if (value.type == PTN_OBJECT) {
            if (value.as.object->lazy_uninitialized && !value.as.object->lazy_initializing) {
                if (!ptn_lazy_object_initialize(runtime, value, line)) {
                    return iterator;
                }
            }
            if (ptn_object_is_generator(value.as.object)) {
                return ptn_array_iterator_from_generator(
                    runtime,
                    value.as.object,
                    0,
                    path,
                    line
                );
            }
            return ptn_array_iterator_from_traversable_object(runtime, value, access_scope, path, line, 0);
        }
        ptn_emit_foreach_non_array_warning(runtime, value, path, line);
        return iterator;
    }
    return ptn_array_iterator_from_array_snapshot(value.as.array);
}

static PTN_UNUSED void ptn_array_iterator_destroy_with_runtime_scope_at(
    PtnArrayIterator *iterator,
    PtnRuntime *runtime,
    size_t line
);

static PTN_UNUSED PtnValue ptn_generator_yield_from(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(source);
    if (
        resolved.type != PTN_ARRAY &&
        !(resolved.type == PTN_OBJECT && ptn_value_is_unpack_traversable(resolved))
    ) {
        ptn_throw_exception(
            runtime,
            "Error",
            "Can use \"yield from\" only with arrays and Traversables"
        );
        return ptn_null();
    }

    if (resolved.type == PTN_OBJECT && ptn_object_is_generator(resolved.as.object)) {
        return ptn_generator_yield_delegate(runtime, resolved, line);
    }
    int has_user_try_frame = 0;
    if (runtime != NULL && runtime->exceptions != NULL) {
        for (
            PtnTryFrame *frame = runtime->exceptions->try_frame;
            frame != NULL;
            frame = frame->previous
        ) {
            if (frame->is_user_try) {
                has_user_try_frame = 1;
                break;
            }
        }
    }
    if (
        runtime != NULL &&
        runtime->current_generator != NULL &&
        !has_user_try_frame &&
        resolved.type == PTN_OBJECT &&
        ptn_value_is_unpack_traversable(resolved) &&
        ptn_generator_append_delegate_entry(runtime, runtime->current_generator, resolved, line)
    ) {
        return ptn_null();
    }

    PtnArrayIterator iterator = ptn_array_iterator_empty();
    PtnTryFrame yield_from_frame;
    int yield_from_frame_active = 0;
    if (runtime != NULL && runtime->exceptions != NULL) {
        ptn_try_frame_push(runtime, &yield_from_frame);
        yield_from_frame_active = 1;
        if (setjmp(yield_from_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &yield_from_frame);
            ptn_value_destroy(&runtime->deferred_yield_from_iterator_object);
            if (iterator.has_iterator_object) {
                runtime->deferred_yield_from_iterator_object =
                    ptn_value_clone_deref(iterator.iterator_object);
            } else if (iterator.object != NULL) {
                runtime->deferred_yield_from_iterator_object =
                    ptn_value_clone_deref(ptn_object(iterator.object));
            } else {
                runtime->deferred_yield_from_iterator_object = ptn_null();
            }
            ptn_gc_attach_value_runtime(runtime, runtime->deferred_yield_from_iterator_object, 0);
            runtime->defer_unreferenced_destructors_for_catch = 1;
            PtnRuntime *root = ptn_runtime_root(runtime);
            if (root != NULL) {
                root->defer_unreferenced_destructors_for_catch = 1;
            }
            ptn_array_iterator_destroy(&iterator);
            ptn_rethrow_exception(runtime);
            return ptn_null();
        }
    }
    iterator = ptn_array_iterator_from_value(
        runtime,
        resolved,
        NULL,
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
    while (iterator.valid) {
        if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
            break;
        }
        PtnGenerator *generator = runtime == NULL ? NULL : runtime->current_generator;
        int64_t saved_next_auto_key = generator == NULL ? 0 : generator->next_auto_key;
        PtnValue key = ptn_array_iterator_current_key(&iterator);
        PtnValue value = ptn_array_iterator_current_value(&iterator);
        PtnValue yielded = ptn_generator_yield(runtime, 1, key, 1, value, line);
        if (generator != NULL) {
            generator->next_auto_key = saved_next_auto_key;
        }
        ptn_value_destroy(&yielded);
        ptn_value_destroy(&key);
        ptn_value_destroy(&value);
        ptn_array_iterator_advance(&iterator);
    }
    ptn_array_iterator_destroy_with_runtime_scope_at(&iterator, runtime, line);
    if (yield_from_frame_active) {
        ptn_try_frame_pop(runtime, &yield_from_frame);
    }
    if (resolved.type == PTN_OBJECT && ptn_object_is_generator(resolved.as.object)) {
        return ptn_generator_get_return(runtime, resolved, line);
    }
    return ptn_null();
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_slot(
    PtnRuntime *runtime,
    PtnValue *slot,
    const char *access_scope,
    const char *path,
    size_t line
) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (slot == NULL) {
        ptn_emit_foreach_non_array_warning(runtime, ptn_null(), path, line);
        return iterator;
    }

    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_OBJECT) {
        if (value->as.object->lazy_uninitialized && !value->as.object->lazy_initializing) {
            if (!ptn_lazy_object_initialize(runtime, *value, line)) {
                return iterator;
            }
        }
        if (ptn_object_is_generator(value->as.object)) {
            return ptn_array_iterator_from_generator(
                runtime,
                value->as.object,
                1,
                path,
                line
            );
        }
        if (
            ptn_object_implements_builtin_interface(value->as.object, "IteratorAggregate") &&
            ptn_object_has_iterator_method(runtime, value->as.object, "getIterator")
        ) {
            return ptn_array_iterator_by_ref_from_traversable_object(
                runtime,
                *value,
                access_scope,
                path,
                line,
                0
            );
        }
        if (
            ptn_object_implements_builtin_interface(value->as.object, "Iterator") &&
            !ptn_object_supports_foreach_by_reference(value->as.object)
        ) {
            ptn_throw_exception_at(
                runtime,
                "Error",
                "An iterator cannot be used with foreach by reference",
                path,
                line
            );
            return iterator;
        }
        return ptn_array_iterator_from_traversable_object(runtime, *value, access_scope, path, line, 0);
    }
    if (value->type == PTN_CLOSURE) {
        return iterator;
    }
    if (value->type != PTN_ARRAY) {
        ptn_emit_foreach_non_array_warning(runtime, ptn_value_deref(*value), path, line);
        return iterator;
    }

    PtnArray *array = ptn_array_detach_value(value);
    if (array == NULL) {
        ptn_emit_foreach_non_array_warning(runtime, ptn_value_deref(*value), path, line);
        return iterator;
    }

    iterator.array = array;
    iterator.index = 0;
    iterator.length = 0;
    iterator.valid = array->len != 0;
    iterator.live = 1;
    iterator.runtime = runtime;
    iterator.access_scope = access_scope;
    iterator.line = line;
    ptn_array_iterator_remember_current_key(&iterator);
    ptn_array_iterator_retain(array);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_variable(
    PtnRuntime *runtime,
    const char *name,
    const char *access_scope,
    const char *path,
    size_t line
) {
    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot == NULL) {
        if (ptn_runtime_is_auto_global_symbol_name(name)) {
            ptn_emit_undefined_global_variable_warning(&runtime->diagnostics, name, path, line);
            ptn_emit_foreach_non_array_warning(runtime, ptn_null(), path, line);
            return ptn_array_iterator_empty();
        }
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
        ptn_emit_foreach_non_array_warning(runtime, ptn_null(), path, line);
        return ptn_array_iterator_empty();
    }
    PtnArrayIterator iterator = ptn_array_iterator_by_ref_from_slot(runtime, slot, access_scope, path, line);
    if (
        iterator.array != NULL &&
        (iterator.object == NULL || iterator.object_property_iterator)
    ) {
        iterator.watched_slot = slot;
    }
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_reference(
    PtnRuntime *runtime,
    PtnValue reference,
    const char *access_scope,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    return ptn_array_iterator_by_ref_from_slot(runtime, &reference, access_scope, path, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_value(
    PtnRuntime *runtime,
    PtnValue *value,
    const char *access_scope,
    const char *path,
    size_t line
) {
    return ptn_array_iterator_by_ref_from_slot(runtime, value, access_scope, path, line);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_key(PtnArrayIterator *iterator) {
    if (iterator->protocol_iterator) {
        return ptn_protocol_iterator_call(iterator, "key");
    }
    if (iterator->generator != NULL) {
        ptn_generator_flush_output_chunk(iterator->runtime, iterator->generator, iterator->index);
        if (ptn_generator_throw_pending_delegate_exception_at_position(
                iterator->runtime,
                iterator->generator,
                iterator->index,
                iterator->line,
                NULL,
                1
            )) {
            return ptn_null();
        }
        PtnValue *delegate_source =
            ptn_generator_delegate_source_value(iterator->generator, iterator->index);
        if (delegate_source != NULL) {
            PtnValue source_receiver = ptn_value_clone_deref(*delegate_source);
            PtnValue key = ptn_generator_key(iterator->runtime, source_receiver, iterator->line);
            ptn_value_destroy(&source_receiver);
            return key;
        }
    }
    if (
        iterator->generator != NULL &&
        iterator->generator->keys != NULL &&
        iterator->index < iterator->generator->keys->len
    ) {
        return ptn_value_clone_deref(iterator->generator->keys->entries[iterator->index].value);
    }
    size_t physical_index = ptn_array_iterator_effective_index(iterator);
    PtnArrayKey key = iterator->array->entries[physical_index].key;
    if (iterator->object_property_iterator && iterator->object != NULL && iterator->generator == NULL) {
        return ptn_object_foreach_key_value(iterator->object, key);
    }
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_owned_string_len(ptn_duplicate_string_len(key.as.string, key.string_len), key.string_len);
}

static PTN_UNUSED void ptn_array_iterator_observe_current_key(PtnArrayIterator *iterator) {
    if (iterator == NULL || !iterator->protocol_iterator) {
        return;
    }
    PtnValue key = ptn_array_iterator_current_key(iterator);
    ptn_value_destroy(&key);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_value(PtnArrayIterator *iterator) {
    if (iterator->protocol_iterator) {
        return ptn_protocol_iterator_call(iterator, "current");
    }
    if (iterator->generator != NULL) {
        ptn_generator_flush_output_chunk(iterator->runtime, iterator->generator, iterator->index);
        if (ptn_generator_throw_pending_delegate_exception_at_position(
                iterator->runtime,
                iterator->generator,
                iterator->index,
                iterator->line,
                NULL,
                1
            )) {
            return ptn_null();
        }
        PtnValue *delegate_source =
            ptn_generator_delegate_source_value(iterator->generator, iterator->index);
        if (delegate_source != NULL) {
            PtnValue source_receiver = ptn_value_clone_deref(*delegate_source);
            PtnValue current = ptn_generator_current(iterator->runtime, source_receiver, iterator->line);
            ptn_value_destroy(&source_receiver);
            return current;
        }
    }
    if (iterator->generator != NULL) {
        ptn_generator_emit_pending_reference_notice(iterator->runtime, iterator->generator, iterator->index);
    }
    size_t physical_index = ptn_array_iterator_effective_index(iterator);
    PtnArrayEntry *entry = &iterator->array->entries[physical_index];
    if (iterator->object_property_iterator && iterator->object != NULL && iterator->generator == NULL) {
        const PtnObjectPropertyMetadata *metadata =
            entry->key.type == PTN_ARRAY_KEY_STRING
                ? ptn_object_property_metadata(iterator->object, entry->key.as.string)
                : NULL;
        if (metadata == NULL && iterator->object->properties != NULL) {
            PtnArrayEntry *property_entry =
                ptn_array_entry_for_key(iterator->object->properties, entry->key);
            if (property_entry != NULL) {
                return ptn_value_borrow(property_entry->value);
            }
        }
        char *property_name = ptn_object_foreach_property_name(iterator->object, entry->key);
        PtnValue receiver = ptn_value_borrow(ptn_object(iterator->object));
        PtnValue value = ptn_object_read_property(
            iterator->runtime,
            receiver,
            property_name,
            iterator->access_scope,
            iterator->line
        );
        free(property_name);
        return value;
    }
    return ptn_value_borrow(entry->value);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_reference(PtnArrayIterator *iterator) {
    if (iterator->protocol_iterator) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        PtnValue reference = ptn_null();
        if (ptn_internal_array_iterator_current_reference(
            iterator->runtime,
            iterator->iterator_object,
            iterator->line,
            &reference
        )) {
            return reference;
        }
#endif
        PtnValue current = ptn_protocol_iterator_call(iterator, "current");
        if (current.type == PTN_REFERENCE) {
            return current;
        }
        return ptn_reference_value(ptn_reference_new_owned(current));
    }
    if (iterator->generator != NULL) {
        ptn_generator_emit_pending_reference_notice(iterator->runtime, iterator->generator, iterator->index);
    }
    size_t physical_index = ptn_array_iterator_effective_index(iterator);
    PtnArrayEntry *entry = &iterator->array->entries[physical_index];
    if (iterator->object_property_iterator && iterator->object != NULL && iterator->generator == NULL) {
        const PtnObjectPropertyMetadata *metadata =
            entry->key.type == PTN_ARRAY_KEY_STRING
                ? ptn_object_property_metadata(iterator->object, entry->key.as.string)
                : NULL;
        if (metadata == NULL && iterator->object->properties != NULL) {
            PtnArrayEntry *property_entry =
                ptn_array_entry_for_key(iterator->object->properties, entry->key);
            if (property_entry != NULL) {
                if (property_entry->value.type != PTN_REFERENCE) {
                    PtnValue current = property_entry->value;
                    property_entry->value = ptn_reference_value(ptn_reference_new_owned(current));
                }
                iterator->current_reference = property_entry->value.as.reference;
                return ptn_value_clone(property_entry->value);
            }
        }
        if (entry->key.type == PTN_ARRAY_KEY_STRING) {
            if (metadata != NULL && metadata->is_readonly) {
                ptn_throw_readonly_property_reference_error(
                    iterator->runtime,
                    metadata->declaring_class,
                    metadata->display_name,
                    iterator->line
                );
                return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
            }
        }
        char *property_name = ptn_object_foreach_property_name(iterator->object, entry->key);
        PtnValue receiver = ptn_value_borrow(ptn_object(iterator->object));
        PtnValue reference = ptn_object_reference_for_property(
            iterator->runtime,
            receiver,
            property_name,
            iterator->access_scope,
            iterator->line
        );
        if (
            reference.type == PTN_REFERENCE &&
            iterator->watched_slot != NULL &&
            entry->key.type == PTN_ARRAY_KEY_STRING
        ) {
            PtnValue watched = ptn_value_deref(*iterator->watched_slot);
            const char *forwarded_storage_key =
                ptn_reference_property_storage_key_for_object(
                    receiver.as.object,
                    reference.as.reference
                );
            ptn_lazy_object_sync_forwarded_proxy_property_reference(
                watched,
                receiver,
                forwarded_storage_key != NULL ? forwarded_storage_key : entry->key.as.string,
                reference
            );
        }
        free(property_name);
        return reference;
    }
    if (
        iterator->object_property_iterator &&
        iterator->object != NULL &&
        iterator->generator == NULL &&
        entry->key.type == PTN_ARRAY_KEY_STRING
    ) {
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(iterator->object, entry->key.as.string);
        if (metadata != NULL && metadata->is_readonly) {
            ptn_throw_readonly_property_reference_error(
                iterator->runtime,
                metadata->declaring_class,
                metadata->display_name,
                iterator->line
            );
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    iterator->current_reference = entry->value.as.reference;
    if (
        iterator->object_property_iterator &&
        iterator->object != NULL &&
        iterator->generator == NULL &&
        entry->key.type == PTN_ARRAY_KEY_STRING
    ) {
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(iterator->object, entry->key.as.string);
        if (metadata != NULL) {
            if (metadata->is_readonly) {
                ptn_throw_readonly_property_reference_error(
                    iterator->runtime,
                    metadata->declaring_class,
                    metadata->display_name,
                    iterator->line
                );
                return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
            }
            ptn_reference_adopt_property_type(entry->value.as.reference, metadata);
        }
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED PtnValue ptn_value_reference_for_array_path(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
);

static PTN_UNUSED PtnValue ptn_array_iterator_current_reference_for_array_path(
    PtnArrayIterator *iterator,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
) {
    if (segment_count == 0 || iterator->protocol_iterator || iterator->generator != NULL) {
        PtnValue current = ptn_array_iterator_current_reference(iterator);
        if (segment_count == 0) {
            return current;
        }
        PtnValue nested = ptn_value_reference_for_array_path(
            iterator->runtime,
            &current,
            segments,
            segment_count,
            path,
            line
        );
        ptn_value_destroy(&current);
        return nested;
    }

    size_t physical_index = ptn_array_iterator_effective_index(iterator);
    if (
        iterator->array == NULL ||
        !iterator->valid ||
        physical_index >= iterator->array->len ||
        iterator->object_property_iterator
    ) {
        PtnValue current = ptn_array_iterator_current_reference(iterator);
        PtnValue nested = ptn_value_reference_for_array_path(
            iterator->runtime,
            &current,
            segments,
            segment_count,
            path,
            line
        );
        ptn_value_destroy(&current);
        return nested;
    }

    PtnArrayEntry *entry = &iterator->array->entries[physical_index];
    return ptn_value_reference_for_array_path(
        iterator->runtime,
        &entry->value,
        segments,
        segment_count,
        path,
        line
    );
}

static PTN_UNUSED void ptn_array_iterator_release(PtnArray *array);

static PTN_UNUSED int ptn_array_iterator_refresh_watched_object(PtnArrayIterator *iterator) {
    if (
        iterator == NULL ||
        !iterator->object_property_iterator ||
        iterator->watched_slot == NULL
    ) {
        return 0;
    }

    PtnValue watched = ptn_value_deref(*iterator->watched_slot);
    if (watched.type != PTN_OBJECT || watched.as.object == NULL) {
        ptn_array_iterator_clear_current_key(iterator);
        iterator->valid = 0;
        return 1;
    }

    PtnObject *effective_object = ptn_lazy_object_foreach_effective_object(
        iterator->runtime,
        watched.as.object,
        iterator->line
    );
    if (effective_object == NULL) {
        ptn_array_iterator_clear_current_key(iterator);
        iterator->valid = 0;
        return 1;
    }
    if (effective_object == iterator->object) {
        return 0;
    }

    PtnRuntime *runtime = iterator->runtime;
    const char *access_scope = iterator->access_scope;
    size_t line = iterator->line;
    PtnValue *watched_slot = iterator->watched_slot;
    PtnArray *old_array = iterator->array;
    PtnObject *old_object = iterator->object;
    PtnArrayIterator replacement =
        ptn_array_iterator_from_object_properties(runtime, effective_object, access_scope, line);

    ptn_array_iterator_clear_current_key(iterator);
    if (old_array != NULL) {
        ptn_array_iterator_release(old_array);
        ptn_array_free(old_array);
    }
    if (old_object != NULL) {
        PtnValue old_value = ptn_object(old_object);
        ptn_value_destroy_with_runtime_scope_at(runtime, &old_value, line);
    }

    *iterator = replacement;
    iterator->watched_slot = watched_slot;
    return 1;
}

static PTN_UNUSED PtnArray *ptn_array_iterator_watched_slot_array(PtnArrayIterator *iterator) {
    if (iterator->watched_slot == NULL) {
        return NULL;
    }
    PtnValue *value = iterator->watched_slot->type == PTN_REFERENCE
        ? &iterator->watched_slot->as.reference->value
        : iterator->watched_slot;
    if (value->type != PTN_ARRAY) {
        return NULL;
    }
    return value->as.array;
}

static PTN_UNUSED int ptn_array_iterator_refresh_watched_array(PtnArrayIterator *iterator) {
    if (!iterator->live || iterator->watched_slot == NULL) {
        return 1;
    }
    PtnValue watched = ptn_value_deref(*iterator->watched_slot);
    if (watched.type != PTN_ARRAY || watched.as.array == NULL) {
        ptn_emit_foreach_non_array_warning(
            iterator->runtime,
            watched,
            iterator->runtime == NULL ? NULL : iterator->runtime->source_path,
            iterator->line
        );
        if (iterator->array != NULL) {
            ptn_array_iterator_release(iterator->array);
            iterator->array = NULL;
        }
        iterator->valid = 0;
        ptn_array_iterator_clear_current_key(iterator);
        return 0;
    }
    PtnArray *array = watched.as.array;
    if (array == iterator->array) {
        return 1;
    }
    ptn_array_iterator_retain(array);
    if (iterator->array != NULL) {
        ptn_array_iterator_release(iterator->array);
    }
    iterator->array = array;
    iterator->object = NULL;
    return 1;
}

static PTN_UNUSED void ptn_array_iterator_advance(PtnArrayIterator *iterator) {
    if (iterator->protocol_iterator) {
        PtnValue next = ptn_protocol_iterator_call(iterator, "next");
        ptn_value_destroy(&next);
        ptn_protocol_iterator_refresh_valid(iterator);
        return;
    }
    if (iterator->array == NULL) {
        iterator->valid = 0;
        ptn_array_iterator_clear_current_key(iterator);
        return;
    }

    if (iterator->generator != NULL) {
        PtnValue *delegate_source =
            ptn_generator_delegate_source_value(iterator->generator, iterator->index);
        if (delegate_source != NULL) {
            PtnValue source_receiver = ptn_value_clone_deref(*delegate_source);
            PtnValue advanced = ptn_generator_next(iterator->runtime, source_receiver, iterator->line);
            ptn_value_destroy(&advanced);
            PtnGenerator *source_generator = ptn_generator_from_value(source_receiver);
            int source_still_valid = ptn_generator_position_valid(source_generator);
            ptn_value_destroy(&source_receiver);
            if (source_still_valid) {
                ptn_array_iterator_clear_current_key(iterator);
                ptn_array_iterator_remember_current_key(iterator);
                return;
            }
            ptn_array_iterator_clear_current_key(iterator);
        }
        ptn_generator_flush_output_chunk(iterator->runtime, iterator->generator, iterator->index);
        if (ptn_generator_throw_pending_exception_at_position(
                iterator->runtime,
                iterator->generator,
                iterator->index,
                iterator->line,
                "next",
                0
            )) {
            return;
        }
    }

    if (iterator->spl_dllist_delete && iterator->valid) {
        size_t physical_index = ptn_array_iterator_effective_index(iterator);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        PtnArray *replacement = NULL;
        if (physical_index < iterator->array->len) {
            replacement = ptn_spl_doubly_linked_list_iterator_remove_index(
                iterator->runtime,
                iterator->object,
                physical_index,
                iterator->line
            );
            if (iterator->runtime != NULL && iterator->runtime->exceptions->active_exception != NULL) {
                iterator->valid = 0;
                ptn_array_iterator_clear_current_key(iterator);
                return;
            }
            if (replacement != NULL && replacement != iterator->array) {
                ptn_array_iterator_retain(replacement);
                ptn_array_iterator_release(iterator->array);
                iterator->array = replacement;
            }
        }
        if (replacement == NULL) {
            if (!ptn_array_iterator_refresh_watched_array(iterator)) {
                return;
            }
        }
#else
        iterator->valid = 0;
        ptn_array_iterator_clear_current_key(iterator);
        return;
#endif
        ptn_array_iterator_clear_current_key(iterator);
        size_t limit = iterator->array == NULL ? 0 : iterator->array->len;
        iterator->valid = iterator->index < limit;
        ptn_array_iterator_remember_current_key(iterator);
        return;
    }

    if (ptn_array_iterator_refresh_watched_object(iterator)) {
        return;
    }

    int switched_from_mutated_array = 0;
    size_t mutation_resume_index = 0;
    if (
        iterator->live &&
        iterator->watched_slot != NULL &&
        iterator->array != NULL &&
        iterator->array->iterator_mutation_epoch != iterator->seen_mutation_epoch
    ) {
        PtnArray *watched_array = ptn_array_iterator_watched_slot_array(iterator);
        if (watched_array != NULL && watched_array != iterator->array) {
            switched_from_mutated_array = 1;
            mutation_resume_index = iterator->array->iterator_mutation_resume_index;
        }
    }

    if (!ptn_array_iterator_refresh_watched_array(iterator)) {
        return;
    }

    size_t next_index = iterator->index + 1;
    if (switched_from_mutated_array) {
        next_index = mutation_resume_index;
    } else if (iterator->has_current_key && !iterator->spl_dllist_reverse) {
        size_t current_index = ptn_array_find_key(iterator->array, iterator->current_key);
        int current_identity_matches = current_index < iterator->array->len &&
            (iterator->object_property_iterator ||
             iterator->current_reference == NULL ||
             (iterator->array->entries[current_index].value.type == PTN_REFERENCE &&
              iterator->array->entries[current_index].value.as.reference == iterator->current_reference));
        if (!current_identity_matches && iterator->current_reference != NULL) {
            for (size_t i = 0; i < iterator->array->len; i++) {
                if (
                    iterator->array->entries[i].value.type == PTN_REFERENCE &&
                    iterator->array->entries[i].value.as.reference == iterator->current_reference
                ) {
                    current_index = i;
                    current_identity_matches = 1;
                    break;
                }
            }
        }
        if (current_identity_matches) {
            next_index = current_index + 1;
        } else if (
            iterator->live &&
            iterator->array->iterator_mutation_epoch != iterator->seen_mutation_epoch
        ) {
            next_index = iterator->array->iterator_mutation_resume_index;
        } else {
            next_index = iterator->index;
        }
    }

    if (iterator->generator != NULL) {
        ptn_generator_release_consumed_reference(iterator->generator, iterator->index);
    }

    ptn_array_iterator_sync_object_property_keys(iterator);
    ptn_array_iterator_clear_current_key(iterator);
    size_t limit = iterator->live ? iterator->array->len : iterator->length;
    if (limit > iterator->array->len) {
        limit = iterator->array->len;
    }
    iterator->index = next_index;
    iterator->valid = iterator->index < limit;
    if (iterator->generator != NULL) {
        iterator->generator->position = iterator->index;
    }
    ptn_array_iterator_skip_invisible_object_properties(iterator);
    if (!iterator->valid && iterator->generator != NULL) {
        ptn_generator_flush_pending_output(iterator->runtime, iterator->generator);
    }
    ptn_array_iterator_remember_current_key(iterator);
}

static PTN_UNUSED void ptn_array_iterator_release(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    if (array->iterator_refcount == 0) {
        return;
    }
    array->iterator_refcount--;
    if (array->iterator_refcount == 0 && array->refcount == 0) {
        ptn_runtime_unregister_array(array->lifecycle_runtime, array);
        ptn_array_destroy_storage(array);
    }
}

static PTN_UNUSED void ptn_array_iterator_destroy_with_runtime_scope_at(
    PtnArrayIterator *iterator,
    PtnRuntime *runtime,
    size_t line
) {
    ptn_array_iterator_clear_current_key(iterator);
    if (iterator->array != NULL) {
        if (iterator->live) {
            ptn_array_iterator_release(iterator->array);
        } else {
            ptn_array_iterator_release(iterator->array);
            ptn_array_free(iterator->array);
        }
        iterator->array = NULL;
    }
    if (iterator->object != NULL) {
        PtnValue object = ptn_object(iterator->object);
        ptn_value_destroy_with_runtime_scope_at(runtime, &object, line);
        iterator->object = NULL;
    }
    iterator->generator = NULL;
    iterator->runtime = NULL;
    iterator->access_scope = NULL;
    if (iterator->has_iterator_object) {
        ptn_value_destroy_with_runtime_scope_at(runtime, &iterator->iterator_object, line);
        iterator->iterator_object = ptn_null();
        iterator->has_iterator_object = 0;
    }
    iterator->watched_slot = NULL;
    iterator->index = 0;
    iterator->length = 0;
    iterator->line = 0;
    iterator->valid = 0;
    iterator->protocol_iterator = 0;
    iterator->spl_dllist_delete = 0;
    iterator->spl_dllist_reverse = 0;
    iterator->object_property_iterator = 0;
    iterator->live = 0;
}

static PTN_UNUSED void ptn_array_iterator_destroy(PtnArrayIterator *iterator) {
    ptn_array_iterator_destroy_with_runtime_scope_at(iterator, NULL, 0);
}

static PTN_UNUSED char *ptn_array_key_diagnostic_name(PtnArrayKey key) {
    char buffer[64];
    if (key.type == PTN_ARRAY_KEY_INT) {
        int written = snprintf(buffer, sizeof(buffer), "%lld", (long long)key.as.integer);
        if (written < 0 || (size_t)written >= sizeof(buffer)) {
            ptn_abort_out_of_memory();
        }
        return ptn_duplicate_string(buffer);
    }

    size_t key_len = key.string_len;
    if (key_len > SIZE_MAX - 3) {
        ptn_abort_out_of_memory();
    }
    char *display = malloc(key_len + 3);
    if (display == NULL) {
        ptn_abort_out_of_memory();
    }
    display[0] = '"';
    memcpy(display + 1, key.as.string, key_len);
    display[key_len + 1] = '"';
    display[key_len + 2] = '\0';
    return display;
}

static PTN_UNUSED void ptn_emit_undefined_array_key_warning(PtnRuntime *runtime, PtnArrayKey key, size_t line) {
    const char *prefix = "Undefined array key ";
    char *display = ptn_array_key_diagnostic_name(key);
    size_t prefix_len = strlen(prefix);
    size_t display_len = strlen(display);
    if (prefix_len > SIZE_MAX - display_len - 1) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc(prefix_len + display_len + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(message, prefix, prefix_len);
    memcpy(message + prefix_len, display, display_len + 1);
    ptn_emit_array_runtime_warning(runtime, message, line);
    free(message);
    free(display);
}

static PTN_UNUSED void ptn_emit_string_offset_cast_warning(PtnRuntime *runtime, size_t line) {
    ptn_emit_array_runtime_warning(runtime, "String offset cast occurred", line);
}

static PTN_UNUSED void ptn_emit_illegal_string_offset_warning(PtnRuntime *runtime, const char *key, size_t line) {
    const char *prefix = "Illegal string offset \"";
    size_t prefix_len = strlen(prefix);
    size_t key_len = strlen(key);
    if (key_len > SIZE_MAX - prefix_len - 2) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc(prefix_len + key_len + 2);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(message, prefix, prefix_len);
    memcpy(message + prefix_len, key, key_len);
    message[prefix_len + key_len] = '"';
    message[prefix_len + key_len + 1] = '\0';
    ptn_emit_array_runtime_warning(runtime, message, line);
    free(message);
}

static PTN_UNUSED void ptn_emit_uninitialized_string_offset_warning(PtnRuntime *runtime, int64_t offset, size_t line) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Uninitialized string offset %lld", (long long)offset);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_array_runtime_warning(runtime, message, line);
}

static PTN_UNUSED void ptn_emit_illegal_string_offset_integer_warning(PtnRuntime *runtime, int64_t offset, size_t line) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Illegal string offset %lld", (long long)offset);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_array_runtime_warning(runtime, message, line);
}

static PTN_UNUSED void ptn_emit_string_offset_assignment_byte_warning(PtnRuntime *runtime, size_t line) {
    ptn_emit_array_runtime_warning(
        runtime,
        "Only the first byte will be assigned to the string offset",
        line
    );
}

static PTN_UNUSED int ptn_string_to_offset(const char *string, int64_t *offset, int *warn_illegal) {
    const char *cursor = string;
    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }

    const char *number_start = cursor;
    if (*cursor == '-' || *cursor == '+') {
        cursor++;
    }
    if (!isdigit((unsigned char)*cursor)) {
        return 0;
    }
    while (isdigit((unsigned char)*cursor)) {
        cursor++;
    }

    char *end = NULL;
    errno = 0;
    long long parsed = strtoll(number_start, &end, 10);
    if (errno == ERANGE || end == number_start) {
        return 0;
    }

    cursor = end;
    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }
    if (*cursor == '\0') {
        *offset = (int64_t)parsed;
        return 1;
    }
    if (*cursor == '.') {
        return 0;
    }

    *offset = (int64_t)parsed;
    *warn_illegal = 1;
    return 1;
}

static PTN_UNUSED int64_t ptn_string_offset_float_to_int(double value) {
    return ptn_float_to_php_integer(value);
}

static PTN_UNUSED int ptn_string_offset_from_value(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int quiet,
    int64_t *offset
) {
    key_value = ptn_value_deref(key_value);
    switch (key_value.type) {
        case PTN_INT:
            *offset = key_value.as.integer;
            return 1;
        case PTN_BOOL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(runtime, line);
            }
            *offset = key_value.as.boolean ? 1 : 0;
            return 1;
        case PTN_NULL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(runtime, line);
            }
            *offset = 0;
            return 1;
        case PTN_FLOAT:
            if (quiet) {
                if (ptn_float_to_int_out_of_range(key_value.as.floating)) {
                    ptn_emit_bitwise_float_out_of_range_warning(
                        &runtime->diagnostics,
                        key_value.as.floating,
                        line
                    );
                }
                if (ptn_float_to_int_loses_precision(key_value.as.floating)) {
                    ptn_emit_float_to_int_precision_deprecation_at(
                        &runtime->diagnostics,
                        key_value.as.floating,
                        runtime->source_path == NULL ? "ptn" : runtime->source_path,
                        line
                    );
                }
            } else {
                ptn_emit_string_offset_cast_warning(runtime, line);
            }
            *offset = ptn_string_offset_float_to_int(key_value.as.floating);
            return 1;
        case PTN_RESOURCE:
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type resource on string");
            return 0;
        case PTN_STRING: {
            int warn_illegal = 0;
            const char *key_string = (const char *)key_value.as.string.data;
            if (ptn_string_to_offset(key_string, offset, &warn_illegal)) {
                if (warn_illegal) {
                    if (quiet) {
                        return 0;
                    }
                    ptn_emit_illegal_string_offset_warning(runtime, key_string, line);
                }
                return 1;
            }
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type string on string");
            return 0;
        }
        case PTN_ARRAY:
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type array on string");
            return 0;
        case PTN_OBJECT:
            if (quiet) {
                return 0;
            }
            {
                const char *type_name = ptn_offset_key_type_name(key_value);
                char message[256];
                int written = snprintf(
                    message,
                    sizeof(message),
                    "Cannot access offset of type %s on string",
                    type_name
                );
                if (written < 0 || (size_t)written >= sizeof(message)) {
                    ptn_abort_out_of_memory();
                }
                ptn_throw_exception(runtime, "TypeError", message);
            }
            return 0;
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            if (quiet) {
                return 0;
            }
            {
                const char *type_name = ptn_offset_key_type_name(key_value);
                char message[256];
                int written = snprintf(
                    message,
                    sizeof(message),
                    "Cannot access offset of type %s on string",
                    type_name
                );
                if (written < 0 || (size_t)written >= sizeof(message)) {
                    ptn_abort_out_of_memory();
                }
                ptn_throw_exception(runtime, "TypeError", message);
            }
            return 0;
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_string_offset_from_value_for_quiet_lookup(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int64_t *offset
) {
    key_value = ptn_value_deref(key_value);
    switch (key_value.type) {
        case PTN_INT:
            *offset = key_value.as.integer;
            return 1;
        case PTN_BOOL:
            *offset = key_value.as.boolean ? 1 : 0;
            return 1;
        case PTN_NULL:
            *offset = 0;
            return 1;
        case PTN_FLOAT:
            *offset = ptn_string_offset_float_to_int(key_value.as.floating);
            return 1;
        case PTN_STRING: {
            int warn_illegal = 0;
            const char *key_string = (const char *)key_value.as.string.data;
            if (ptn_string_to_offset(key_string, offset, &warn_illegal)) {
                if (warn_illegal) {
                    ptn_emit_illegal_string_offset_warning(runtime, key_string, line);
                }
                return 1;
            }
            return 0;
        }
        case PTN_RESOURCE:
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type resource on string");
            return 0;
        case PTN_ARRAY:
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type array on string");
            return 0;
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION: {
            const char *type_name = ptn_offset_key_type_name(key_value);
            char message[256];
            int written = snprintf(
                message,
                sizeof(message),
                "Cannot access offset of type %s on string",
                type_name
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "TypeError", message);
            return 0;
        }
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_string_offset_index(size_t string_len, int64_t offset, size_t *index) {
    if (offset >= 0) {
        uint64_t positive = (uint64_t)offset;
        if (positive >= string_len) {
            return 0;
        }
        *index = (size_t)positive;
        return 1;
    }

    uint64_t distance = (uint64_t)(-(offset + 1)) + 1;
    if (distance > string_len) {
        return 0;
    }
    *index = string_len - (size_t)distance;
    return 1;
}

static PTN_UNUSED PtnLookupResult ptn_string_offset_lookup(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line,
    int quiet
) {
    int64_t offset = 0;
    int offset_available = quiet
        ? ptn_string_offset_from_value_for_quiet_lookup(runtime, key_value, line, &offset)
        : ptn_string_offset_from_value(runtime, key_value, line, quiet, &offset);
    if (!offset_available) {
        return ptn_lookup_missing();
    }
    size_t index = 0;
    if (!ptn_string_offset_index(container.as.string.len, offset, &index)) {
        if (!quiet) {
            ptn_emit_uninitialized_string_offset_warning(runtime, offset, line);
            return ptn_lookup_found(ptn_value_from_string_offset(ptn_string("")));
        }
        return ptn_lookup_missing();
    }

    char *result = malloc(2);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = (char)container.as.string.data[index];
    result[1] = '\0';
    return ptn_lookup_found(ptn_value_from_string_offset(ptn_owned_string_len(result, 1)));
}

static PTN_UNUSED int ptn_string_offset_assignment_index(
    PtnRuntime *runtime,
    size_t string_len,
    int64_t offset,
    size_t line,
    size_t *index,
    size_t *new_len
) {
    if (offset >= 0) {
        uint64_t positive = (uint64_t)offset;
        if (positive >= (uint64_t)PTRDIFF_MAX) {
            ptn_emit_illegal_string_offset_integer_warning(runtime, offset, line);
            return 0;
        }
        *index = (size_t)positive;
        *new_len = *index >= string_len ? *index + 1 : string_len;
        return 1;
    }

    if (ptn_string_offset_index(string_len, offset, index)) {
        *new_len = string_len;
        return 1;
    }

    ptn_emit_illegal_string_offset_integer_warning(runtime, offset, line);
    return 0;
}

static PTN_UNUSED unsigned char ptn_string_offset_assignment_byte(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
    }

    PtnStringOperand string = ptn_value_to_string_operand_with_runtime(runtime, value, line);
    if (string.len == 0) {
        ptn_string_operand_free(string);
        ptn_throw_exception(runtime, "Error", "Cannot assign an empty string to a string offset");
        return 0;
    }
    if (string.len > 1) {
        ptn_emit_string_offset_assignment_byte_warning(runtime, line);
    }

    unsigned char byte = (unsigned char)string.data[0];
    ptn_string_operand_free(string);
    return byte;
}

static PTN_UNUSED PtnValue ptn_runtime_string_offset_set_result(
    PtnRuntime *runtime,
    PtnValue *target,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    if (target == NULL || target->type != PTN_STRING) {
        return ptn_null();
    }

    int64_t offset = 0;
    if (!ptn_string_offset_from_value(runtime, key_value, line, 0, &offset)) {
        return ptn_null();
    }
    if (target->type != PTN_STRING) {
        return ptn_null();
    }

    size_t index = 0;
    size_t new_len = 0;
    if (!ptn_string_offset_assignment_index(runtime, target->as.string.len, offset, line, &index, &new_len)) {
        return ptn_null();
    }

    unsigned char byte = ptn_string_offset_assignment_byte(runtime, value, line);
    ptn_cow_debug_note_string_detach();
    ptn_value_detach_for_write(target);
    if (target->as.string.len != new_len) {
        ptn_string_value_resize(target, new_len);
    }
    target->as.string.payload->data[index] = byte;
    target->as.string.payload->interned = 0;
    ptn_string_value_refresh(target);

    char *result = malloc(2);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = (char)byte;
    result[1] = '\0';
    return ptn_owned_string_len(result, 1);
}

static PTN_UNUSED void ptn_runtime_string_offset_set(
    PtnRuntime *runtime,
    PtnValue *target,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    PtnValue result = ptn_runtime_string_offset_set_result(runtime, target, key_value, value, line);
    ptn_value_destroy(&result);
}

static PTN_UNUSED void ptn_reject_nested_string_offset_array_access(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line
) {
    int64_t offset = 0;
    if (!ptn_string_offset_from_value(runtime, key_value, line, 0, &offset)) {
        return;
    }
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Cannot use string offset as an array",
        runtime == NULL ? NULL : runtime->source_path,
        line
    );
}

static PTN_UNUSED void ptn_reject_nested_string_offset_unset(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line
) {
    key_value = ptn_value_deref(key_value);
    int valid_offset = 0;
    switch (key_value.type) {
        case PTN_INT:
            valid_offset = 1;
            break;
        case PTN_BOOL:
        case PTN_NULL:
        case PTN_FLOAT: {
            int64_t offset = 0;
            valid_offset = ptn_string_offset_from_value(runtime, key_value, line, 0, &offset);
            break;
        }
        case PTN_STRING: {
            int64_t offset = 0;
            int warn_illegal = 0;
            valid_offset = ptn_string_to_offset(
                (const char *)key_value.as.string.data,
                &offset,
                &warn_illegal
            );
            break;
        }
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            valid_offset = 0;
            break;
    }

    ptn_throw_exception_at(
        runtime,
        "Error",
        valid_offset ? "Cannot use string offset as an array" : "Cannot unset string offsets",
        runtime == NULL ? NULL : runtime->source_path,
        line
    );
}

static PTN_UNUSED uint64_t ptn_runtime_symbol_table_epoch_for_name(PtnRuntime *runtime, const char *name) {
    if (runtime == NULL || name == NULL) {
        return 0;
    }
    return ptn_runtime_variable_symbol_table(runtime, name)->mutation_epoch;
}

static PTN_UNUSED int ptn_runtime_is_globals_name(const char *name);

static PTN_UNUSED int ptn_array_path_root_values_match(PtnValue left, PtnValue right) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type != right.type) {
        return 0;
    }
    switch (left.type) {
        case PTN_NULL:
            return 1;
        case PTN_BOOL:
            return left.as.boolean == right.as.boolean;
        case PTN_INT:
            return left.as.integer == right.as.integer;
        case PTN_FLOAT:
            return left.as.floating == right.as.floating;
        case PTN_STRING:
            return left.as.string.len == right.as.string.len &&
                memcmp(left.as.string.data, right.as.string.data, left.as.string.len) == 0;
        case PTN_ARRAY:
            return left.as.array == right.as.array;
        case PTN_OBJECT:
        case PTN_CLOSURE:
            return left.as.object == right.as.object;
        case PTN_EXCEPTION:
            return left.as.exception == right.as.exception;
        case PTN_RESOURCE:
            return left.as.resource == right.as.resource;
        case PTN_REFERENCE:
            return left.as.reference == right.as.reference;
    }
    return 0;
}

static PTN_UNUSED int ptn_runtime_array_path_root_matches_snapshot(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root
) {
    if (runtime == NULL || name == NULL || ptn_runtime_is_globals_name(name)) {
        return 1;
    }
    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    PtnValue current = slot == NULL ? ptn_null() : ptn_value_deref(*slot);
    return ptn_array_path_root_values_match(current, pre_eval_root);
}

static PTN_UNUSED size_t ptn_array_path_root_snapshot_array_refcount(PtnValue root) {
    root = ptn_value_deref(root);
    if (root.type != PTN_ARRAY || root.as.array == NULL) {
        return 0;
    }
    return root.as.array->refcount;
}

static PTN_UNUSED int ptn_runtime_array_path_root_matches_guard_snapshot(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root,
    size_t guarded_array_refcount
) {
    if (!ptn_runtime_array_path_root_matches_snapshot(runtime, name, pre_eval_root)) {
        return 0;
    }
    PtnValue root = ptn_value_deref(pre_eval_root);
    if (root.type != PTN_ARRAY || root.as.array == NULL) {
        return 1;
    }
    return root.as.array->refcount == guarded_array_refcount;
}

static PTN_UNUSED void ptn_emit_invalidated_array_path_write_diagnostics(
    PtnRuntime *runtime,
    PtnValue root,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return;
    }

    PtnValue container = ptn_value_deref(root);
    PtnValue owned_container = ptn_null();
    int has_owned_container = 0;
    for (size_t i = 0; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            break;
        }
        if (container.type == PTN_ARRAY) {
            PtnArrayKey key;
            if (!ptn_array_offset_key_from_value(runtime, segment->value, line, 1, &key)) {
                break;
            }
            PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
            ptn_array_key_free(key);
            if (entry == NULL) {
                break;
            }
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
                has_owned_container = 0;
            }
            owned_container = ptn_value_clone_deref(entry->value);
            has_owned_container = 1;
            container = ptn_value_deref(owned_container);
            continue;
        }
        break;
    }

    if (container.type == PTN_STRING) {
        const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
        if (!leaf->append) {
            int64_t offset = 0;
            (void)ptn_string_offset_from_value(runtime, leaf->value, line, 0, &offset);
        }
    }

    if (has_owned_container) {
        ptn_value_destroy(&owned_container);
    }
}

static PTN_UNUSED int ptn_arrayaccess_can_dispatch(
    PtnRuntime *runtime,
    PtnValue container,
    const char *method_name
) {
    container = ptn_value_deref(container);
    if (container.type != PTN_OBJECT || runtime->method_dispatch == NULL) {
        return 0;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (
        ptn_internal_class_exists_name(container.as.object->class_name) &&
        ptn_internal_class_method_exists(container.as.object->class_name, method_name)
    ) {
        return 1;
    }
    if (
        ptn_object_is_internal_or_descendant(container, "ArrayObject") &&
        ptn_internal_class_method_exists("ArrayObject", method_name)
    ) {
        return 1;
    }
    if (
        ptn_object_is_internal_or_descendant(container, "SplFixedArray") &&
        ptn_internal_class_method_exists("SplFixedArray", method_name)
    ) {
        return 1;
    }
    if (
        ptn_object_is_internal_or_descendant(container, "SplObjectStorage") &&
        ptn_internal_class_method_exists("SplObjectStorage", method_name)
    ) {
        return 1;
    }
    if (
        (ptn_object_is_internal_or_descendant(container, "ArrayIterator") ||
         ptn_object_is_internal_or_descendant(container, "RecursiveArrayIterator")) &&
        ptn_internal_class_method_exists("ArrayIterator", method_name)
    ) {
        return 1;
    }
    if (
        ptn_object_is_internal_or_descendant(container, "SplDoublyLinkedList") &&
        ptn_internal_class_method_exists("SplDoublyLinkedList", method_name)
    ) {
        return 1;
    }
#endif
    return runtime->declared_method_exists != NULL &&
        runtime->declared_method_exists(container.as.object->class_name, method_name);
}

static PTN_UNUSED PtnValue ptn_arrayaccess_call(
    PtnRuntime *runtime,
    PtnValue container,
    const char *method_name,
    size_t argc,
    PtnValue *args,
    size_t line
) {
    PtnValue receiver = ptn_value_clone_deref(container);
    PtnValue result = runtime->method_dispatch(runtime, receiver, method_name, argc, args, line);
    ptn_value_destroy(&receiver);
    if (runtime != NULL && runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&result);
        ptn_rethrow_exception(runtime);
        return ptn_null();
    }
    return result;
}

static PTN_UNUSED PtnValue ptn_arrayaccess_read(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    PtnValue args[1] = { ptn_value_clone_deref(key_value) };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetGet", 1, args, line);
    ptn_value_destroy(&args[0]);
    return result;
}

static PTN_UNUSED void ptn_emit_indirect_modification_overloaded_element_notice(
    PtnRuntime *runtime,
    PtnValue container,
    size_t line
) {
    container = ptn_value_deref(container);
    const char *type_name = container.type == PTN_OBJECT
        ? container.as.object->class_name
        : ptn_offset_container_type_name(container);
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Indirect modification of overloaded element of %s has no effect",
        type_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_notice_with_path(
        runtime == NULL ? NULL : &runtime->diagnostics,
        message,
        runtime != NULL && runtime->source_path != NULL ? runtime->source_path : "ptn",
        line,
        1
    );
}

static PTN_UNUSED int ptn_arrayaccess_nested_write_should_apply(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue nested,
    size_t line
) {
    PtnValue nested_value = ptn_value_deref(nested);
    if (nested.type == PTN_REFERENCE ||
        ptn_arrayaccess_can_dispatch(runtime, nested_value, "offsetSet") ||
        ptn_arrayaccess_can_dispatch(runtime, nested_value, "offsetGet")) {
        return 1;
    }

    if (nested_value.type != PTN_OBJECT) {
        ptn_emit_indirect_modification_overloaded_element_notice(runtime, container, line);
    }
    return nested_value.type != PTN_ARRAY && nested_value.type != PTN_NULL;
}

static PTN_UNUSED void ptn_arrayaccess_write(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    PtnValue args[2] = {
        ptn_value_clone_deref(key_value),
        ptn_value_clone_deref(value)
    };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetSet", 2, args, line);
    ptn_value_destroy(&result);
    ptn_value_destroy(&args[0]);
    ptn_value_destroy(&args[1]);
}

static PTN_UNUSED int ptn_arrayaccess_value_is_weak_map(PtnValue value) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    value = ptn_value_deref(value);
    return value.type == PTN_OBJECT &&
        value.as.object != NULL &&
        ptn_internal_class_name_is_weak_map(value.as.object->class_name);
#else
    (void)value;
    return 0;
#endif
}

static PTN_UNUSED int ptn_weak_map_reject_append_offset(
    PtnRuntime *runtime,
    PtnValue container,
    const PtnArrayPathSegment *segment
) {
    if (segment == NULL || !segment->append || !ptn_arrayaccess_value_is_weak_map(container)) {
        return 0;
    }
    ptn_throw_exception(runtime, "Error", "Cannot append to WeakMap");
    return 1;
}

static PTN_UNUSED int ptn_arrayaccess_exists(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    PtnValue args[1] = { ptn_value_clone_deref(key_value) };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetExists", 1, args, line);
    int exists = ptn_is_truthy(ptn_value_deref(result));
    ptn_value_destroy(&result);
    ptn_value_destroy(&args[0]);
    return exists;
}

static PTN_UNUSED void ptn_arrayaccess_unset(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    PtnValue args[1] = { ptn_value_clone_deref(key_value) };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetUnset", 1, args, line);
    ptn_value_destroy(&result);
    ptn_value_destroy(&args[0]);
}

static PTN_UNUSED PtnLookupResult ptn_offset_lookup(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line, int quiet) {
    PtnValue stable_container = ptn_value_clone_deref(container);
    PtnValue stable_key = ptn_value_clone_deref(key_value);
    container = stable_container;
    key_value = stable_key;
    PtnLookupResult result = ptn_lookup_missing();
    if (container.type == PTN_STRING) {
        result = ptn_string_offset_lookup(runtime, container, key_value, line, quiet);
        goto done;
    }

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (quiet) {
        PtnLookupResult internal_result = ptn_lookup_missing();
        if (ptn_internal_array_object_offset_lookup_quiet(
                runtime,
                container,
                &key_value,
                line,
                &internal_result
            )) {
            result = internal_result;
            goto done;
        }
    }
#endif

    if (quiet && ptn_arrayaccess_can_dispatch(runtime, container, "offsetExists") &&
        !ptn_arrayaccess_exists(runtime, container, key_value, line)) {
        goto done;
    }

    if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetGet")) {
        result = ptn_lookup_found(ptn_arrayaccess_read(runtime, container, key_value, line));
        goto done;
    }

    if (ptn_value_is_plain_object_for_array_offset(runtime, container)) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        if (ptn_internal_class_name_is_pdo_row(container.as.object->class_name)) {
            goto done;
        }
#endif
        ptn_throw_cannot_use_object_as_array(runtime, container, line);
        goto done;
    }

    if (container.type != PTN_ARRAY) {
        if (!quiet) {
            const char *prefix = "Trying to access array offset on ";
            const char *type_name = ptn_offset_container_type_name(container);
            char message[128];
            int written = snprintf(message, sizeof(message), "%s%s", prefix, type_name);
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_array_runtime_warning(runtime, message, line);
        }
        goto done;
    }

    ptn_emit_array_offset_key_conversion_diagnostic(runtime, key_value, line, 1);

    PtnArrayKey key;
    if (quiet && ptn_array_offset_key_is_invalid(key_value)) {
        ptn_throw_array_offset_key_type_error(
            runtime,
            key_value,
            "Cannot access offset of type %s on array",
            line
        );
        goto done;
    }
    if (!ptn_array_offset_key_from_value(runtime, key_value, line, quiet, &key)) {
        goto done;
    }
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    if (entry == NULL) {
        if (!quiet) {
            ptn_emit_undefined_array_key_warning(runtime, key, line);
        }
        ptn_array_key_free(key);
        goto done;
    }
    PtnValue value = ptn_value_clone_deref(entry->value);
    ptn_array_key_free(key);
    result = ptn_lookup_found(value);

done:
    ptn_value_destroy(&stable_key);
    ptn_value_destroy(&stable_container);
    return result;
}

static PTN_UNUSED PtnValue ptn_array_read(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    PtnLookupResult result = ptn_offset_lookup(runtime, container, key_value, line, 0);
    if (!result.exists) {
        return ptn_null();
    }
    return result.value;
}

static PTN_UNUSED PtnValue ptn_constant_expression_array_read(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(container);
    if (
        resolved.type == PTN_OBJECT ||
        resolved.type == PTN_CLOSURE ||
        resolved.type == PTN_EXCEPTION
    ) {
        const char *message = "Cannot use [] on objects in constant expression";
        if (runtime != NULL) {
            ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
        } else {
            fprintf(stderr, "Fatal error: %s\n", message);
            exit(255);
        }
        return ptn_null();
    }
    return ptn_array_read(runtime, container, key_value, line);
}

static PTN_UNUSED PtnValue ptn_array_read_for_list_destructure(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    container = ptn_value_deref(container);
    key_value = ptn_value_deref(key_value);
    if (container.type == PTN_NULL) {
        return ptn_null();
    }
    if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetGet")) {
        return ptn_arrayaccess_read(runtime, container, key_value, line);
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, container)) {
        ptn_throw_cannot_use_object_as_array(runtime, container, line);
        return ptn_null();
    }
    if (container.type != PTN_ARRAY) {
        char message[128];
        const char *type_name = container.type == PTN_BOOL
            ? "bool"
            : ptn_offset_container_type_name(container);
        int written = snprintf(
            message,
            sizeof(message),
            "Cannot use %s as array",
            type_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_array_runtime_diagnostic(runtime, "Warning", message, line);
        return ptn_null();
    }

    ptn_emit_array_offset_key_conversion_diagnostic(runtime, key_value, line, 1);

    PtnArrayKey key;
    if (!ptn_array_offset_key_from_value(runtime, key_value, line, 0, &key)) {
        return ptn_null();
    }
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    if (entry == NULL) {
        ptn_emit_undefined_array_key_warning(runtime, key, line);
        ptn_array_key_free(key);
        return ptn_null();
    }
    PtnValue value = ptn_value_clone_deref(entry->value);
    ptn_array_key_free(key);
    return value;
}

static PTN_UNUSED PtnValue ptn_value_array_path_read_for_list_destructure(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnValue current = ptn_value_clone_deref(target);
    for (size_t i = 0; i < segment_count; i++) {
        if (segments[i].append) {
            ptn_value_destroy(&current);
            return ptn_null();
        }
        PtnValue next = ptn_array_read_for_list_destructure(
            runtime,
            current,
            segments[i].value,
            line
        );
        ptn_value_destroy(&current);
        current = next;
    }
    return current;
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_read_for_list_destructure(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
) {
    PtnValue root = ptn_runtime_read_variable(runtime, name, path, line);
    PtnValue result = ptn_value_array_path_read_for_list_destructure(
        runtime,
        root,
        segments,
        segment_count,
        line
    );
    ptn_value_destroy(&root);
    return result;
}

static PTN_UNUSED int ptn_offset_is_set(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line,
    int emit_array_key_diagnostic
) {
    PtnValue stable_container = ptn_value_clone_deref(container);
    PtnValue stable_key = ptn_value_clone_deref(key_value);
    container = stable_container;
    key_value = stable_key;
    int result = 0;
    if (container.type == PTN_STRING) {
        int64_t offset = 0;
        size_t index = 0;
        result = ptn_string_offset_from_value(runtime, key_value, line, 1, &offset) &&
            ptn_string_offset_index(container.as.string.len, offset, &index);
        goto done;
    }

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_arrayaccess_value_is_weak_map(container)) {
        result = ptn_weak_map_offset_isset(runtime, container, key_value, line);
        goto done;
    }
    if (ptn_internal_array_object_offset_isset_quiet(
            runtime,
            container,
            &key_value,
            line,
            &result
        )) {
        goto done;
    }
#endif

    if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetExists")) {
        result = ptn_arrayaccess_exists(runtime, container, key_value, line);
        goto done;
    }

    if (ptn_value_is_plain_object_for_array_offset(runtime, container)) {
        ptn_throw_cannot_use_object_as_array(runtime, container, line);
        goto done;
    }

    if (container.type != PTN_ARRAY) {
        goto done;
    }
    if (emit_array_key_diagnostic) {
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, key_value, line, 1);
    }

    PtnArrayKey key;
    if (ptn_array_offset_key_is_invalid(key_value)) {
        ptn_throw_array_offset_key_type_error(
            runtime,
            key_value,
            "Cannot access offset of type %s in isset or empty",
            line
        );
        goto done;
    }
    if (!ptn_array_offset_key_from_value(runtime, key_value, line, 1, &key)) {
        goto done;
    }
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    result = entry != NULL && ptn_value_deref(entry->value).type != PTN_NULL;
    ptn_array_key_free(key);

done:
    ptn_value_destroy(&stable_key);
    ptn_value_destroy(&stable_container);
    return result;
}

static PTN_UNUSED int ptn_offset_is_empty(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    PtnValue stable_container = ptn_value_clone_deref(container);
    PtnValue stable_key = ptn_value_clone_deref(key_value);
    container = stable_container;
    key_value = stable_key;
    int result = 1;
    if (container.type == PTN_STRING) {
        int64_t offset = 0;
        size_t index = 0;
        if (!ptn_string_offset_from_value(runtime, key_value, line, 1, &offset) ||
            !ptn_string_offset_index(container.as.string.len, offset, &index)) {
            goto done;
        }
        result = container.as.string.data[index] == '0';
        goto done;
    }

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    {
        PtnLookupResult internal_result = ptn_lookup_missing();
        if (ptn_internal_array_object_offset_lookup_quiet(
                runtime,
                container,
                &key_value,
                line,
                &internal_result
            )) {
            if (!internal_result.exists) {
                goto done;
            }
            result = !ptn_is_truthy(ptn_value_deref(internal_result.value));
            ptn_value_destroy(&internal_result.value);
            goto done;
        }
    }
#endif

    if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetExists")) {
        if (!ptn_arrayaccess_exists(runtime, container, key_value, line)) {
            goto done;
        }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        PtnValue resolved_container = ptn_value_deref(container);
        if (resolved_container.type == PTN_OBJECT &&
            (ptn_ascii_case_equal(resolved_container.as.object->class_name, "DOMNodeList") ||
             ptn_ascii_case_equal(resolved_container.as.object->class_name, "Dom\\NodeList") ||
             ptn_ascii_case_equal(resolved_container.as.object->class_name, "DOMNamedNodeMap") ||
             ptn_ascii_case_equal(resolved_container.as.object->class_name, "Dom\\NamedNodeMap"))) {
            result = 0;
            goto done;
        }
        if (ptn_object_is_internal_or_descendant(resolved_container, "SplFixedArray")) {
            result = 0;
            goto done;
        }
#endif
        if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetGet")) {
            PtnValue value = ptn_arrayaccess_read(runtime, container, key_value, line);
            result = !ptn_is_truthy(ptn_value_deref(value));
            ptn_value_destroy(&value);
            goto done;
        }
        result = 0;
        goto done;
    }

    if (ptn_value_is_plain_object_for_array_offset(runtime, container)) {
        ptn_throw_cannot_use_object_as_array(runtime, container, line);
        goto done;
    }

    if (container.type != PTN_ARRAY) {
        goto done;
    }
    ptn_emit_array_offset_key_conversion_diagnostic(runtime, key_value, line, 1);

    PtnArrayKey key;
    if (ptn_array_offset_key_is_invalid(key_value)) {
        ptn_throw_array_offset_key_type_error(
            runtime,
            key_value,
            "Cannot access offset of type %s in isset or empty",
            line
        );
        goto done;
    }
    if (!ptn_array_offset_key_from_value(runtime, key_value, line, 1, &key)) {
        goto done;
    }
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    result = entry == NULL || !ptn_is_truthy(ptn_value_deref(entry->value));
    ptn_array_key_free(key);

done:
    ptn_value_destroy(&stable_key);
    ptn_value_destroy(&stable_container);
    return result;
}

static PTN_UNUSED PtnValue ptn_array_key_value(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_owned_string_len(ptn_duplicate_string_len(key.as.string, key.string_len), key.string_len);
}

static PTN_UNUSED void ptn_emit_assign_op_missing_array_key(PtnRuntime *runtime, PtnValue key_value, size_t line) {
    ptn_emit_array_offset_key_conversion_diagnostic(runtime, key_value, line, 1);
    key_value = ptn_value_deref(key_value);
    PtnArrayKey key;
    if (!ptn_array_offset_key_from_value(runtime, key_value, line, 0, &key)) {
        return;
    }
    ptn_emit_undefined_array_key_warning(runtime, key, line);
    ptn_array_key_free(key);
}

static PTN_UNUSED void ptn_runtime_array_warn_missing_base_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    if (strcmp(name, "GLOBALS") == 0) {
        return;
    }
    PtnValue container;
    if (!ptn_symbols_get(ptn_runtime_variable_symbol_table(runtime, name), name, &container)) {
        PtnTryFrame warning_frame;
        ptn_try_frame_push(runtime, &warning_frame);
        if (setjmp(warning_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &warning_frame);
            return;
        }
        if (ptn_runtime_is_auto_global_symbol_name(name)) {
            ptn_emit_undefined_global_variable_warning(&runtime->diagnostics, name, path, line);
            ptn_try_frame_pop(runtime, &warning_frame);
            return;
        }
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
        ptn_try_frame_pop(runtime, &warning_frame);
    }
}

static PTN_UNUSED PtnArray *ptn_runtime_array_detach_variable(PtnRuntime *runtime, const char *name) {
    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    size_t index = ptn_symbols_find(symbols, name);
    if (index >= symbols->len) {
        return NULL;
    }
    PtnValue *value = &symbols->items[index].value;
    if (value->type == PTN_REFERENCE) {
        return ptn_array_detach_value(&value->as.reference->value);
    }
    if (value->type != PTN_ARRAY) {
        return NULL;
    }
    return ptn_array_detach_value(value);
}

static PTN_UNUSED void ptn_runtime_separate_array_variable(PtnRuntime *runtime, const char *name) {
    (void)ptn_runtime_array_detach_variable(runtime, name);
}

static PTN_UNUSED PtnArray *ptn_value_replace_with_empty_array(PtnValue *value) {
    ptn_value_destroy(value);
    *value = ptn_array_from_literal_entries(0, NULL);
    return value->as.array;
}

static PTN_UNUSED PtnArray *ptn_array_root_slot_for_write(
    PtnRuntime *runtime,
    PtnValue *slot,
    size_t line
) {
    if (slot == NULL) {
        return NULL;
    }
    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_ARRAY) {
        return ptn_array_detach_value(value);
    }
    if (slot->type == PTN_REFERENCE &&
        value->type == PTN_NULL &&
        slot->as.reference->property_type_kind != PTN_PROPERTY_TYPE_NONE) {
        PtnReferencePropertyTypeSource blocking_source;
        if (!ptn_reference_property_types_accept_array_auto_initialization(
            runtime,
            slot->as.reference,
            &blocking_source
        )) {
            ptn_throw_property_array_auto_initialization_error(
                runtime,
                blocking_source.declaring_class,
                blocking_source.property_name,
                blocking_source.text,
                1,
                line
            );
            return NULL;
        }
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(runtime, value, line);
    if (converted != NULL) {
        return converted;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, *value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *value, line);
        return NULL;
    }
    (void)runtime;
    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED PtnArray *ptn_runtime_array_root_for_write(
    PtnRuntime *runtime,
    const char *name,
    size_t line
) {
    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    PtnValue *slot = ptn_symbols_value_slot(symbols, name);
    if (slot != NULL) {
        return ptn_array_root_slot_for_write(runtime, slot, line);
    }

    PtnValue array = ptn_array_from_literal_entries(0, NULL);
    ptn_runtime_write_variable(runtime, name, array);
    ptn_value_destroy(&array);
    slot = ptn_symbols_value_slot(symbols, name);
    if (slot == NULL) {
        return NULL;
    }
    return ptn_array_root_slot_for_write(runtime, slot, line);
}

static PTN_UNUSED int ptn_array_path_segment_key(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    size_t line,
    PtnArrayKey *key_out
) {
    if (segment->append) {
        if (!ptn_array_append_key_available(runtime, array)) {
            return 0;
        }
        *key_out = ptn_array_int_key(array->next_auto_key);
        return 1;
    }
    return ptn_array_offset_key_from_value(runtime, segment->value, line, 0, key_out);
}

static PTN_UNUSED PtnArrayPathSegment *ptn_array_path_segments_clone_values(
    const PtnArrayPathSegment *segments,
    size_t segment_count
) {
    if (segment_count == 0) {
        return NULL;
    }
    if (segment_count > SIZE_MAX / sizeof(PtnArrayPathSegment)) {
        ptn_abort_out_of_memory();
    }
    PtnArrayPathSegment *cloned = malloc(segment_count * sizeof(PtnArrayPathSegment));
    if (cloned == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < segment_count; i++) {
        cloned[i].append = segments[i].append;
        cloned[i].value = segments[i].append
            ? ptn_null()
            : ptn_value_clone_deref(segments[i].value);
        cloned[i].deferred_missing_variable_name = segments[i].deferred_missing_variable_name;
        cloned[i].deferred_missing_variable_line = segments[i].deferred_missing_variable_line;
    }
    return cloned;
}

static PTN_UNUSED void ptn_array_path_segments_free_cloned_values(
    PtnArrayPathSegment *segments,
    size_t segment_count
) {
    if (segments == NULL) {
        return;
    }
    for (size_t i = 0; i < segment_count; i++) {
        ptn_value_drop(&segments[i].value);
    }
    free(segments);
}

static PTN_UNUSED int ptn_runtime_is_globals_name(const char *name) {
    return strcmp(name, "GLOBALS") == 0;
}

static PTN_UNUSED char *ptn_runtime_global_name_from_segment(
    const PtnArrayPathSegment *segment
) {
    if (segment->append) {
        return NULL;
    }
    return ptn_value_to_string(segment->value);
}

static PTN_UNUSED PtnLookupResult ptn_runtime_globals_array_path_lookup_quiet(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_lookup_found(ptn_runtime_globals_snapshot(runtime));
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return ptn_lookup_missing();
    }

    PtnLookupResult root = ptn_runtime_read_global_variable_quiet(runtime, global_name);
    free(global_name);
    if (!root.exists || segment_count == 1) {
        return root;
    }

    PtnValue container = ptn_value_deref(root.value);
    PtnValue owned_container = ptn_null();
    int has_owned_container = 0;
    for (size_t i = 1; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return ptn_lookup_missing();
        }

        PtnLookupResult result = ptn_offset_lookup(runtime, container, segment->value, line, 1);
        if (!result.exists || i + 1 == segment_count) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return result;
        }

        if (has_owned_container) {
            ptn_value_destroy(&owned_container);
        }
        owned_container = result.value;
        has_owned_container = 1;
        container = ptn_value_deref(owned_container);
    }

    if (has_owned_container) {
        ptn_value_destroy(&owned_container);
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED PtnValue ptn_runtime_globals_array_path_read(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_runtime_globals_snapshot(runtime);
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return ptn_null();
    }

    PtnValue root_value;
    if (!ptn_symbols_get(ptn_runtime_global_symbol_table(runtime), global_name, &root_value)) {
        ptn_emit_undefined_global_variable_warning(
            &runtime->diagnostics,
            global_name,
            runtime->source_path,
            line
        );
        free(global_name);
        return ptn_null();
    }
    free(global_name);

    if (segment_count == 1) {
        return ptn_value_clone_deref(root_value);
    }

    PtnValue container = ptn_value_deref(root_value);
    PtnValue owned_container = ptn_null();
    int has_owned_container = 0;
    for (size_t i = 1; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return ptn_null();
        }

        PtnLookupResult result = ptn_offset_lookup(runtime, container, segment->value, line, 0);
        if (!result.exists) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return ptn_null();
        }
        if (i + 1 == segment_count) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return result.value;
        }

        if (has_owned_container) {
            ptn_value_destroy(&owned_container);
        }
        owned_container = result.value;
        has_owned_container = 1;
        container = ptn_value_deref(owned_container);
    }

    if (has_owned_container) {
        ptn_value_destroy(&owned_container);
    }
    return ptn_null();
}

static PTN_UNUSED PtnLookupResult ptn_runtime_array_path_lookup_quiet(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (ptn_runtime_is_globals_name(name)) {
        return ptn_runtime_globals_array_path_lookup_quiet(runtime, segments, segment_count, line);
    }
    if (segment_count == 0) {
        return ptn_lookup_missing();
    }

    PtnLookupResult root = ptn_runtime_read_variable_quiet(runtime, name);
    if (!root.exists) {
        return ptn_lookup_missing();
    }

    PtnValue container = ptn_value_deref(root.value);
    PtnValue owned_container = ptn_null();
    int has_owned_container = 0;
    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return ptn_lookup_missing();
        }

        PtnLookupResult result = ptn_offset_lookup(runtime, container, segment->value, line, 1);
        if (!result.exists || i + 1 == segment_count) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return result;
        }

        if (has_owned_container) {
            ptn_value_destroy(&owned_container);
        }
        owned_container = result.value;
        has_owned_container = 1;
        container = ptn_value_deref(owned_container);
    }

    if (has_owned_container) {
        ptn_value_destroy(&owned_container);
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED PtnLookupResult ptn_value_array_path_lookup_quiet(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_lookup_found(ptn_value_clone_deref(target));
    }

    PtnValue container = ptn_value_deref(target);
    PtnValue owned_container = ptn_null();
    int has_owned_container = 0;
    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return ptn_lookup_missing();
        }

        PtnLookupResult result = ptn_offset_lookup(runtime, container, segment->value, line, 1);
        if (!result.exists || i + 1 == segment_count) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return result;
        }

        if (has_owned_container) {
            ptn_value_destroy(&owned_container);
        }
        owned_container = result.value;
        has_owned_container = 1;
        container = ptn_value_deref(owned_container);
    }

    if (has_owned_container) {
        ptn_value_destroy(&owned_container);
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED void ptn_array_path_emit_key_conversion_diagnostic(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_key_conversion_diagnostic
) {
    if (segment->deferred_missing_variable_name != NULL) {
        size_t warning_line = segment->deferred_missing_variable_line == 0
            ? line
            : segment->deferred_missing_variable_line;
        ptn_emit_undefined_variable_warning(
            &runtime->diagnostics,
            segment->deferred_missing_variable_name,
            runtime->source_path,
            warning_line
        );
    }
    if (!emit_key_conversion_diagnostic || segment->append) {
        return;
    }
    ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 1);
}

static PTN_UNUSED void ptn_array_path_emit_deferred_undefined_variable_warning(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segment,
    size_t line
) {
    ptn_array_path_emit_key_conversion_diagnostic(runtime, segment, line, 0);
}

static PTN_UNUSED int ptn_array_path_emit_write_diagnostic_changed_array(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_key_conversion_diagnostic
) {
    if (array == NULL) {
        ptn_array_path_emit_key_conversion_diagnostic(
            runtime,
            segment,
            line,
            emit_key_conversion_diagnostic
        );
        return 0;
    }

    size_t refcount = array->refcount;
    uint64_t mutation_epoch = array->mutation_epoch;
    ptn_array_retain(array);
    ptn_array_debug_hide_ref(array);
    ptn_array_path_emit_key_conversion_diagnostic(
        runtime,
        segment,
        line,
        emit_key_conversion_diagnostic
    );
    int changed = array->refcount != refcount + 1 || array->mutation_epoch != mutation_epoch;
    ptn_array_debug_unhide_ref(array);
    ptn_array_free(array);
    return changed;
}

static PTN_UNUSED PtnArray *ptn_array_descend_for_reference_write(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    const char *path,
    size_t line
) {
    PtnArrayKey key;
    if (!ptn_array_path_segment_key(runtime, array, segment, line, &key)) {
        return NULL;
    }
    PtnArrayEntry *entry = segment->append ? NULL : ptn_array_entry_for_key(array, key);

    if (entry == NULL) {
        PtnValue child = ptn_array_from_literal_entries(0, NULL);
        ptn_array_set_entry(array, key, child);
        return array->entries[array->len - 1].value.as.array;
    }

    ptn_array_key_free(key);
    PtnValue *entry_value = entry->value.type == PTN_REFERENCE
        ? &entry->value.as.reference->value
        : &entry->value;
    if (entry_value->type == PTN_ARRAY) {
        return ptn_array_detach_value(entry_value);
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(runtime, entry_value, line);
    if (converted != NULL) {
        return converted;
    }
    if (entry_value->type == PTN_STRING) {
        if (!segment->append) {
            if (!ptn_warn_illegal_string_reference_key(runtime, *entry_value, &segment->value, line)) {
                return NULL;
            }
        }
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return NULL;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, *entry_value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *entry_value, line);
        return NULL;
    }

    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED PtnValue ptn_value_reference_for_array_path(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
);

static PTN_UNUSED void ptn_value_array_path_set_impl(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
);
static PTN_UNUSED PtnValue ptn_value_array_path_set_result(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
);
static PTN_UNUSED PtnValue ptn_value_array_path_read_for_assign_op(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
);
static PTN_UNUSED PtnValue ptn_value_array_path_read_for_inc_dec(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
);
static PTN_UNUSED PtnValue ptn_value_array_path_read_for_overloaded_assign_op(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
);

static PTN_UNUSED PtnValue ptn_value_reference_for_array_path(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
);

static PTN_UNUSED void ptn_value_bind_array_path_reference(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue reference,
    const char *path,
    size_t line
);

static PTN_UNUSED PtnValue ptn_runtime_reference_for_array_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
) {
    if (ptn_runtime_is_globals_name(name)) {
        if (segment_count == 0) {
            return ptn_runtime_reference_for_variable(runtime, name);
        }

        char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
        if (global_name == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        PtnValue *slot = ptn_runtime_global_variable_slot_for_write(runtime, global_name);
        free(global_name);
        if (slot == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        return ptn_value_reference_for_array_path(
            runtime,
            slot,
            segments + 1,
            segment_count - 1,
            path,
            line
        );
    }

    if (segment_count == 0) {
        return ptn_runtime_reference_for_variable(runtime, name);
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot != NULL) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            PtnValue append_reference = ptn_null();
            if (
                segments[0].append &&
                ptn_arrayaccess_append_reference_temporary(runtime, slot_value, line, &append_reference)
            ) {
                return append_reference;
            }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
            PtnValue array_object_reference = ptn_null();
            if (ptn_internal_array_object_offset_reference(
                runtime,
                slot_value,
                offset_value,
                line,
                1,
                &array_object_reference
            )) {
                if (segment_count == 1) {
                    return array_object_reference;
                }
                PtnValue nested_reference = ptn_value_reference_for_array_path(
                    runtime,
                    &array_object_reference,
                    segments + 1,
                    segment_count - 1,
                    path,
                    line
                );
                ptn_value_destroy(&array_object_reference);
                return nested_reference;
            }
#endif
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            PtnValue value = ptn_arrayaccess_read(runtime, slot_value, key, line);
            if (segment_count == 1 && value.type == PTN_REFERENCE) {
                return value;
            }
            if (segment_count == 1) {
                (void)ptn_arrayaccess_nested_write_should_apply(runtime, slot_value, value, line);
                return ptn_reference_value(ptn_reference_new_owned(value));
            }
            if (value.type == PTN_REFERENCE) {
                PtnValue nested_reference = ptn_value_reference_for_array_path(
                    runtime,
                    &value,
                    segments + 1,
                    segment_count - 1,
                    path,
                    line
                );
                ptn_value_destroy(&value);
                return nested_reference;
            }
            ptn_value_destroy(&value);
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        if (slot_value.type == PTN_STRING && segments[0].append) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", path, line);
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }

    if (slot != NULL && !segments[0].append) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (!ptn_warn_illegal_string_reference_key(runtime, slot_value, &segments[0].value, line)) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }

    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_reference_write(runtime, array, &segments[i], path, line);
        if (array == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    PtnArrayEntry *entry = ptn_array_reference_entry(
        runtime,
        array,
        leaf->append ? NULL : &leaf->value,
        line
    );
    if (entry == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
        entry->by_ref_argument_eligible = 1;
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_runtime_bind_array_path_reference(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue reference,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    if (ptn_runtime_is_globals_name(name)) {
        if (segment_count == 0) {
            ptn_runtime_bind_variable_reference(runtime, name, reference);
            return;
        }

        char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
        if (global_name == NULL) {
            return;
        }
        if (segment_count == 1) {
            ptn_gc_attach_value_runtime(runtime, reference, 0);
            ptn_symbols_bind_reference(ptn_runtime_global_symbol_table(runtime), global_name, reference);
            free(global_name);
            return;
        }

        PtnValue *slot = ptn_runtime_global_variable_slot_for_write(runtime, global_name);
        free(global_name);
        if (slot == NULL) {
            return;
        }
        ptn_value_bind_array_path_reference(
            runtime,
            slot,
            segments + 1,
            segment_count - 1,
            reference,
            path,
            line
        );
        return;
    }
    if (segment_count == 0) {
        ptn_runtime_bind_variable_reference(runtime, name, reference);
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot != NULL) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (!segments[0].append) {
            if (!ptn_warn_illegal_string_reference_key(runtime, slot_value, &segments[0].value, line)) {
                return;
            }
        }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        if (
            !segments[0].append &&
            segment_count == 1 &&
            slot_value.type == PTN_OBJECT &&
            ptn_internal_class_name_is_weak_map(slot_value.as.object->class_name)
        ) {
            PtnValue key = segments[0].value;
            if (ptn_weak_map_bind_reference(runtime, slot_value, key, reference, line)) {
                return;
            }
        }
        if (!segments[0].append) {
            if (segment_count == 1) {
                if (ptn_internal_array_object_bind_offset_reference(
                        runtime,
                        slot_value,
                        &segments[0].value,
                        reference,
                        line
                    )) {
                    return;
                }
            } else {
                PtnValue array_object_reference = ptn_null();
                if (ptn_internal_array_object_offset_reference(
                        runtime,
                        slot_value,
                        &segments[0].value,
                        line,
                        1,
                        &array_object_reference
                    )) {
                    ptn_value_bind_array_path_reference(
                        runtime,
                        &array_object_reference,
                        segments + 1,
                        segment_count - 1,
                        reference,
                        path,
                        line
                    );
                    ptn_value_destroy(&array_object_reference);
                    return;
                }
            }
        }
#endif
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            if (segments[0].append) {
                ptn_emit_indirect_modification_overloaded_element_notice(runtime, slot_value, line);
                ptn_throw_exception_at(
                    runtime,
                    "Error",
                    "Cannot assign by reference to an array dimension of an object",
                    path,
                    line
                );
                return;
            }
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            PtnValue value = ptn_arrayaccess_read(runtime, slot_value, key, line);
            if (value.type == PTN_REFERENCE) {
                if (segment_count == 1) {
                    ptn_reference_assign(runtime, value.as.reference, reference);
                } else {
                    ptn_value_bind_array_path_reference(
                        runtime,
                        &value,
                        segments + 1,
                        segment_count - 1,
                        reference,
                        path,
                        line
                    );
                }
                ptn_value_destroy(&value);
                return;
            }
            ptn_emit_indirect_modification_overloaded_element_notice(runtime, slot_value, line);
            ptn_value_destroy(&value);
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Cannot assign by reference to an array dimension of an object",
                path,
                line
            );
            return;
        }
        if (slot_value.type == PTN_STRING && segments[0].append) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", path, line);
            return;
        }
        if (!segments[0].append) {
            if (!ptn_warn_illegal_string_reference_key(runtime, slot_value, &segments[0].value, line)) {
                return;
            }
        }
    }

    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return;
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_reference_write(runtime, array, &segments[i], path, line);
        if (array == NULL) {
            return;
        }
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return;
    }
    PtnArrayKey key;
    if (leaf->append) {
        key = ptn_array_int_key(array->next_auto_key);
    } else if (!ptn_array_offset_key_from_value(runtime, leaf->value, line, 0, &key)) {
        return;
    }
    ptn_array_set_entry(array, key, ptn_value_clone(reference));
}

static PTN_UNUSED PtnValue ptn_value_reference_for_array_path(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
) {
    if (target == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (segment_count == 0) {
        if (target->type != PTN_REFERENCE) {
            PtnValue current = *target;
            *target = ptn_reference_value(ptn_reference_new_owned(current));
        }
        return ptn_value_clone(*target);
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetGet")) {
        PtnValue append_reference = ptn_null();
        if (
            segments[0].append &&
            ptn_arrayaccess_append_reference_temporary(runtime, *target_value, line, &append_reference)
        ) {
            return append_reference;
        }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
        PtnValue array_object_reference = ptn_null();
        if (ptn_internal_array_object_offset_reference(
            runtime,
            *target_value,
            offset_value,
            line,
            1,
            &array_object_reference
        )) {
            if (segment_count == 1) {
                return array_object_reference;
            }
            PtnValue nested_reference = ptn_value_reference_for_array_path(
                runtime,
                &array_object_reference,
                segments + 1,
                segment_count - 1,
                path,
                line
            );
            ptn_value_destroy(&array_object_reference);
            return nested_reference;
        }
#endif
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue value = ptn_arrayaccess_read(runtime, *target_value, key, line);
        if (segment_count == 1 && value.type == PTN_REFERENCE) {
            return value;
        }
        if (segment_count == 1) {
            (void)ptn_arrayaccess_nested_write_should_apply(runtime, *target_value, value, line);
            return ptn_reference_value(ptn_reference_new_owned(value));
        }
        if (value.type == PTN_REFERENCE) {
            PtnValue nested_reference = ptn_value_reference_for_array_path(
                runtime,
                &value,
                segments + 1,
                segment_count - 1,
                path,
                line
            );
            ptn_value_destroy(&value);
            return nested_reference;
        }
        ptn_value_destroy(&value);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    PtnArray *array = NULL;
    if (target_value->type == PTN_ARRAY) {
        array = ptn_array_detach_value(target_value);
    } else if (target_value->type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", path, line);
        } else {
            if (ptn_warn_illegal_string_reference_key(runtime, *target_value, &segments[0].value, line)) {
                ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
            }
        }
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    } else if ((array = ptn_array_convertible_scalar_for_write(runtime, target_value, line)) != NULL) {
        /* false/null conversion handled by shared lvalue write semantics. */
    } else if (ptn_value_is_plain_object_for_array_offset(runtime, *target_value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *target_value, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    } else {
        ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_reference_write(runtime, array, &segments[i], path, line);
        if (array == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    PtnArrayEntry *entry = ptn_array_reference_entry(
        runtime,
        array,
        leaf->append ? NULL : &leaf->value,
        line
    );
    if (entry == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
        entry->by_ref_argument_eligible = 1;
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_value_bind_array_path_reference(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue reference,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    if (target == NULL) {
        return;
    }
    if (segment_count == 0) {
        ptn_value_destroy(target);
        *target = ptn_value_clone(reference);
        return;
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (!segments[0].append) {
        if (segment_count == 1) {
            if (ptn_internal_array_object_bind_offset_reference(
                    runtime,
                    *target_value,
                    &segments[0].value,
                    reference,
                    line
                )) {
                return;
            }
        } else {
            PtnValue array_object_reference = ptn_null();
            if (ptn_internal_array_object_offset_reference(
                    runtime,
                    *target_value,
                    &segments[0].value,
                    line,
                    1,
                    &array_object_reference
                )) {
                ptn_value_bind_array_path_reference(
                    runtime,
                    &array_object_reference,
                    segments + 1,
                    segment_count - 1,
                    reference,
                    path,
                    line
                );
                ptn_value_destroy(&array_object_reference);
                return;
            }
        }
    }
#endif
    PtnArray *array = NULL;
    if (target_value->type == PTN_ARRAY) {
        array = ptn_array_detach_value(target_value);
    } else if (target_value->type == PTN_STRING) {
        if (!segments[0].append) {
            if (!ptn_warn_illegal_string_reference_key(runtime, *target_value, &segments[0].value, line)) {
                return;
            }
        }
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return;
    } else if ((array = ptn_array_convertible_scalar_for_write(runtime, target_value, line)) != NULL) {
        /* false/null conversion handled by shared lvalue write semantics. */
    } else if (ptn_value_is_plain_object_for_array_offset(runtime, *target_value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *target_value, line);
        return;
    } else {
        ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
        return;
    }
    if (array == NULL) {
        return;
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_reference_write(runtime, array, &segments[i], path, line);
        if (array == NULL) {
            return;
        }
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return;
    }
    PtnArrayKey key;
    if (leaf->append) {
        key = ptn_array_int_key(array->next_auto_key);
    } else if (!ptn_array_offset_key_from_value(runtime, leaf->value, line, 0, &key)) {
        return;
    }
    ptn_array_set_entry(array, key, ptn_value_clone(reference));
}

static PTN_UNUSED PtnArray *ptn_array_descend_for_write(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_array_path_emit_key_conversion_diagnostic(runtime, segment, line, emit_null_key_deprecation);
    PtnArrayKey key;
    if (!ptn_array_path_segment_key(runtime, array, segment, line, &key)) {
        return NULL;
    }
    PtnArrayEntry *entry = segment->append ? NULL : ptn_array_entry_for_key(array, key);

    if (entry == NULL) {
        PtnValue child = ptn_array_from_literal_entries(0, NULL);
        ptn_array_set_entry(array, key, child);
        return array->entries[array->len - 1].value.as.array;
    }

    ptn_array_key_free(key);
    PtnValue *entry_value = entry->value.type == PTN_REFERENCE
        ? &entry->value.as.reference->value
        : &entry->value;
    if (entry_value->type == PTN_ARRAY) {
        return ptn_array_detach_value(entry_value);
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(runtime, entry_value, line);
    if (converted != NULL) {
        return converted;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, *entry_value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *entry_value, line);
        return NULL;
    }

    (void)runtime;
    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED PtnValue ptn_array_set_path_leaf_result(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
);

static PTN_UNUSED int ptn_array_path_write_guard_invalidated(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root,
    uint64_t guarded_epoch,
    size_t guarded_array_refcount,
    int guard_enabled,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (!guard_enabled) {
        return 0;
    }
    (void)guarded_epoch;
    if (ptn_runtime_array_path_root_matches_guard_snapshot(
        runtime,
        name,
        pre_eval_root,
        guarded_array_refcount
    )) {
        return 0;
    }
    ptn_emit_invalidated_array_path_write_diagnostics(
        runtime,
        pre_eval_root,
        segments,
        segment_count,
        line
    );
    return 1;
}

static PTN_UNUSED int ptn_array_path_emit_write_diagnostic_changed_array_guarded(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_key_conversion_diagnostic,
    const char *guarded_name,
    PtnValue pre_eval_root,
    uint64_t guarded_epoch,
    size_t guarded_array_refcount,
    int guard_enabled,
    const PtnArrayPathSegment *segments,
    size_t segment_count
) {
    if (!guard_enabled || segment->deferred_missing_variable_name == NULL) {
        return ptn_array_path_emit_write_diagnostic_changed_array(
            runtime,
            array,
            segment,
            line,
            emit_key_conversion_diagnostic
        );
    }

    if (array == NULL) {
        ptn_array_path_emit_deferred_undefined_variable_warning(runtime, segment, line);
        if (ptn_array_path_write_guard_invalidated(
            runtime,
            guarded_name,
            pre_eval_root,
            guarded_epoch,
            guarded_array_refcount,
            guard_enabled,
            segments,
            segment_count,
            line
        )) {
            return 1;
        }
        if (emit_key_conversion_diagnostic && !segment->append) {
            ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 1);
        }
        return 0;
    }

    size_t refcount = array->refcount;
    uint64_t mutation_epoch = array->mutation_epoch;
    ptn_array_retain(array);
    ptn_array_debug_hide_ref(array);
    ptn_array_path_emit_deferred_undefined_variable_warning(runtime, segment, line);
    size_t retained_guarded_array_refcount = guarded_array_refcount == 0
        ? 0
        : guarded_array_refcount + 1;
    if (ptn_array_path_write_guard_invalidated(
        runtime,
        guarded_name,
        pre_eval_root,
        guarded_epoch,
        retained_guarded_array_refcount,
        guard_enabled,
        segments,
        segment_count,
        line
    )) {
        ptn_array_debug_unhide_ref(array);
        ptn_array_free(array);
        return 1;
    }
    if (array->refcount != refcount + 1 || array->mutation_epoch != mutation_epoch) {
        ptn_array_debug_unhide_ref(array);
        ptn_array_free(array);
        return 1;
    }
    if (emit_key_conversion_diagnostic && !segment->append) {
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 1);
    }
    int changed = array->refcount != refcount + 1 || array->mutation_epoch != mutation_epoch;
    ptn_array_debug_unhide_ref(array);
    ptn_array_free(array);
    return changed;
}

static PTN_UNUSED PtnValue ptn_array_path_set_result_from_root_impl(
    PtnRuntime *runtime,
    const char *guarded_name,
    PtnValue pre_eval_root,
    uint64_t guarded_epoch,
    int guard_enabled,
    PtnArray *array,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (array == NULL) {
        return ptn_null();
    }
    if (segment_count == 0) {
        return ptn_value_clone_deref(value);
    }

    size_t guarded_array_refcount = guard_enabled
        ? ptn_array_path_root_snapshot_array_refcount(pre_eval_root)
        : 0;
    PtnArray *current = array;
    for (size_t i = 0; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (ptn_array_path_emit_write_diagnostic_changed_array_guarded(
            runtime,
            current,
            segment,
            line,
            emit_null_key_deprecation,
            guarded_name,
            pre_eval_root,
            guarded_epoch,
            guarded_array_refcount,
            guard_enabled,
            segments,
            segment_count
        )) {
            return ptn_null();
        }
        if (ptn_array_path_write_guard_invalidated(
            runtime,
            guarded_name,
            pre_eval_root,
            guarded_epoch,
            guarded_array_refcount,
            guard_enabled,
            segments,
            segment_count,
            line
        )) {
            return ptn_null();
        }
        PtnArrayKey key;
        if (!ptn_array_path_segment_key(runtime, current, segment, line, &key)) {
            return ptn_null();
        }
        PtnArrayEntry *entry = segment->append ? NULL : ptn_array_entry_for_key(current, key);

        if (entry == NULL) {
            PtnValue child = ptn_array_from_literal_entries(0, NULL);
            ptn_array_set_entry(current, key, child);
            current = current->entries[current->len - 1].value.as.array;
            continue;
        }

        ptn_array_key_free(key);
        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        PtnValue resolved_entry = ptn_value_deref(*entry_value);
        if (ptn_arrayaccess_can_dispatch(runtime, resolved_entry, "offsetSet") ||
            ptn_arrayaccess_can_dispatch(runtime, resolved_entry, "offsetGet")) {
            return ptn_value_array_path_set_result(
                runtime,
                &entry->value,
                segments + i + 1,
                segment_count - i - 1,
                value,
                line
            );
        }
        if (entry_value->type == PTN_ARRAY) {
            current = ptn_array_detach_value(entry_value);
            continue;
        }
        if (guard_enabled && entry_value->type == PTN_BOOL && !entry_value->as.boolean) {
            (void)ptn_value_replace_with_empty_array(entry_value);
            ptn_emit_false_array_conversion_deprecation(runtime, line);
            if (ptn_array_path_write_guard_invalidated(
                runtime,
                guarded_name,
                pre_eval_root,
                guarded_epoch,
                guarded_array_refcount,
                guard_enabled,
                segments,
                segment_count,
                line
            )) {
                return ptn_null();
            }
            current = ptn_array_detach_value(entry_value);
            if (current == NULL) {
                return ptn_null();
            }
            continue;
        }
        PtnArray *converted = ptn_array_convertible_scalar_for_write(runtime, entry_value, line);
        if (converted != NULL) {
            if (ptn_array_path_write_guard_invalidated(
                runtime,
                guarded_name,
                pre_eval_root,
                guarded_epoch,
                guarded_array_refcount,
                guard_enabled,
                segments,
                segment_count,
                line
            )) {
                return ptn_null();
            }
            current = converted;
            continue;
        }
        if (entry_value->type == PTN_STRING) {
            const PtnArrayPathSegment *next_segment = &segments[i + 1];
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, next_segment, line);
            if (ptn_array_path_write_guard_invalidated(
                runtime,
                guarded_name,
                pre_eval_root,
                guarded_epoch,
                guarded_array_refcount,
                guard_enabled,
                segments,
                segment_count,
                line
            )) {
                return ptn_null();
            }
            if (next_segment->append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            } else if (i + 2 == segment_count) {
                return ptn_runtime_string_offset_set_result(
                    runtime,
                    entry_value,
                    next_segment->value,
                    value,
                    line
                );
            } else {
                ptn_reject_nested_string_offset_array_access(runtime, next_segment->value, line);
            }
            return ptn_null();
        }
        if (ptn_value_is_plain_object_for_array_offset(runtime, *entry_value)) {
            ptn_throw_cannot_use_object_as_array(runtime, *entry_value, line);
            return ptn_null();
        }

        ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
        return ptn_null();
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (ptn_array_path_emit_write_diagnostic_changed_array_guarded(
        runtime,
        current,
        leaf,
        line,
        emit_null_key_deprecation,
        guarded_name,
        pre_eval_root,
        guarded_epoch,
        guarded_array_refcount,
        guard_enabled,
        segments,
        segment_count
    )) {
        return ptn_null();
    }
    if (ptn_array_path_write_guard_invalidated(
        runtime,
        guarded_name,
        pre_eval_root,
        guarded_epoch,
        guarded_array_refcount,
        guard_enabled,
        segments,
        segment_count,
        line
    )) {
        return ptn_null();
    }
    PtnArrayKey key;
    if (!ptn_array_path_segment_key(runtime, current, leaf, line, &key)) {
        return ptn_null();
    }
    ptn_array_enforce_memory_limit_for_entry_write(runtime, current, key, line);
    return ptn_array_write_entry_result(runtime, current, key, value);
}

static PTN_UNUSED PtnValue ptn_array_path_set_result_from_root(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    return ptn_array_path_set_result_from_root_impl(
        runtime,
        NULL,
        ptn_null(),
        0,
        0,
        array,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED PtnValue ptn_array_path_set_result_from_root_after_dimension_eval(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root,
    uint64_t guarded_epoch,
    PtnArray *array,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    return ptn_array_path_set_result_from_root_impl(
        runtime,
        name,
        pre_eval_root,
        guarded_epoch,
        1,
        array,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED void ptn_array_path_set_from_root(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    PtnValue result = ptn_array_path_set_result_from_root(
        runtime,
        array,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
    ptn_value_destroy(&result);
}

static PTN_UNUSED void ptn_array_path_set_from_root_after_dimension_eval(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root,
    uint64_t guarded_epoch,
    PtnArray *array,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    PtnValue result = ptn_array_path_set_result_from_root_after_dimension_eval(
        runtime,
        name,
        pre_eval_root,
        guarded_epoch,
        array,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
    ptn_value_destroy(&result);
}

static PTN_UNUSED void ptn_array_set_path_leaf(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_array_path_emit_key_conversion_diagnostic(runtime, segment, line, emit_null_key_deprecation);
    PtnArrayKey key;
    if (!ptn_array_path_segment_key(runtime, array, segment, line, &key)) {
        return;
    }
    ptn_array_enforce_memory_limit_for_entry_write(runtime, array, key, line);
    ptn_array_write_entry(runtime, array, key, ptn_array_value_clone_for_write(array, value));
}

static PTN_UNUSED PtnValue ptn_array_set_path_leaf_result(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_array_path_emit_key_conversion_diagnostic(runtime, segment, line, emit_null_key_deprecation);
    PtnArrayKey key;
    if (!ptn_array_path_segment_key(runtime, array, segment, line, &key)) {
        return ptn_null();
    }
    ptn_array_enforce_memory_limit_for_entry_write(runtime, array, key, line);
    return ptn_array_write_entry_result(runtime, array, key, value);
}

static PTN_UNUSED void ptn_runtime_globals_array_path_set_impl(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (segment_count == 0) {
        return;
    }
    if (segments[0].append) {
        ptn_emit_fatal_error_at(
            runtime,
            "Cannot append to $GLOBALS",
            runtime->source_path,
            line
        );
        return;
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return;
    }

    if (segment_count == 1) {
        ptn_runtime_write_global_variable(runtime, global_name, value);
        free(global_name);
        return;
    }

    PtnValue *slot = ptn_runtime_global_variable_slot_for_write(runtime, global_name);
    free(global_name);
    if (slot == NULL) {
        return;
    }

    if (segment_count == 2) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[1].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
                return;
            }
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[1], line);
            ptn_runtime_string_offset_set(runtime, slot_value, segments[1].value, value, line);
            return;
        }
    }
    if (segment_count > 2) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[1].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            } else {
                ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[1], line);
                ptn_reject_nested_string_offset_array_access(runtime, segments[1].value, line);
            }
            return;
        }
    }

    PtnArray *array = ptn_array_root_slot_for_write(runtime, slot, line);
    ptn_array_path_set_from_root(
        runtime,
        array,
        segments + 1,
        segment_count - 1,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED PtnValue ptn_runtime_globals_array_path_set_result_impl(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (segment_count == 0) {
        return ptn_value_clone_deref(value);
    }
    if (segments[0].append) {
        ptn_emit_fatal_error_at(
            runtime,
            "Cannot append to $GLOBALS",
            runtime->source_path,
            line
        );
        return ptn_null();
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return ptn_null();
    }

    if (segment_count == 1) {
        PtnValue result = ptn_runtime_write_global_variable_result(runtime, global_name, value);
        free(global_name);
        return result;
    }

    PtnValue *slot = ptn_runtime_global_variable_slot_for_write(runtime, global_name);
    free(global_name);
    if (slot == NULL) {
        return ptn_null();
    }

    if (segment_count == 2) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[1].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
                return ptn_null();
            }
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[1], line);
            return ptn_runtime_string_offset_set_result(runtime, slot_value, segments[1].value, value, line);
        }
    }
    if (segment_count > 2) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[1].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            } else {
                ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[1], line);
                ptn_reject_nested_string_offset_array_access(runtime, segments[1].value, line);
            }
            return ptn_null();
        }
    }

    PtnArray *array = ptn_array_root_slot_for_write(runtime, slot, line);
    return ptn_array_path_set_result_from_root(
        runtime,
        array,
        segments + 1,
        segment_count - 1,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED PtnValue ptn_runtime_globals_array_path_set_result(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    return ptn_runtime_globals_array_path_set_result_impl(
        runtime,
        segments,
        segment_count,
        value,
        line,
        1
    );
}

static PTN_UNUSED void ptn_runtime_array_path_set_after_dimension_eval(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root,
    uint64_t pre_eval_epoch,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
);

static PTN_UNUSED PtnValue ptn_runtime_array_path_set_result_impl(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
);

static PTN_UNUSED void ptn_runtime_array_path_set_impl(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (ptn_runtime_is_globals_name(name)) {
        ptn_runtime_globals_array_path_set_impl(
            runtime,
            segments,
            segment_count,
            value,
            line,
            emit_null_key_deprecation
        );
        return;
    }
    if (segment_count == 0) {
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot != NULL && segments[0].append) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            return;
        }
    }
    if (slot != NULL && segment_count == 1) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type != PTN_STRING) {
            slot_value = NULL;
        }
        if (slot_value != NULL) {
            if (segments[0].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
                return;
            }
            uint64_t pre_key_diagnostic_epoch = ptn_runtime_symbol_table_epoch_for_name(runtime, name);
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
            if (ptn_runtime_symbol_table_epoch_for_name(runtime, name) == pre_key_diagnostic_epoch) {
                slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
                slot_value = slot == NULL ? NULL : (slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot);
                if (slot_value != NULL && slot_value->type == PTN_STRING) {
                    ptn_runtime_string_offset_set(runtime, slot_value, segments[0].value, value, line);
                    return;
                }
            }
            int64_t offset = 0;
            (void)ptn_string_offset_from_value(runtime, segments[0].value, line, 0, &offset);
            return;
        }
    }
    if (slot != NULL && segment_count > 1) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[0].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            } else {
                ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
                ptn_reject_nested_string_offset_array_access(runtime, segments[0].value, line);
            }
            return;
        }
    }
    if (slot != NULL && segment_count == 1) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_weak_map_reject_append_offset(runtime, slot_value, &segments[0])) {
            return;
        }
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetSet")) {
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
            ptn_arrayaccess_write(runtime, slot_value, key, value, line);
            return;
        }
    }
    if (slot != NULL && segment_count > 1) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_weak_map_reject_append_offset(runtime, slot_value, &segments[0])) {
            return;
        }
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            if (segments[0].append) {
                PtnValue append_reference = ptn_null();
                if (ptn_arrayaccess_append_reference_temporary(runtime, slot_value, line, &append_reference)) {
                    ptn_value_array_path_set_impl(
                        runtime,
                        &append_reference,
                        segments + 1,
                        segment_count - 1,
                        value,
                        line,
                        emit_null_key_deprecation
                    );
                    ptn_value_destroy(&append_reference);
                    return;
                }
            }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
            PtnValue nested_reference = ptn_null();
            if (ptn_internal_array_object_offset_reference(
                runtime,
                slot_value,
                offset_value,
                line,
                1,
                &nested_reference
            )) {
                ptn_value_array_path_set_impl(
                    runtime,
                    &nested_reference,
                    segments + 1,
                    segment_count - 1,
                    value,
                    line,
                    emit_null_key_deprecation
                );
                ptn_value_destroy(&nested_reference);
                return;
            }
#endif
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
            PtnValue nested = ptn_arrayaccess_read(runtime, slot_value, key, line);
            if (ptn_arrayaccess_nested_write_should_apply(runtime, slot_value, nested, line)) {
                ptn_value_array_path_set_impl(
                    runtime,
                    &nested,
                    segments + 1,
                    segment_count - 1,
                    value,
                    line,
                    emit_null_key_deprecation
                );
            }
            ptn_value_destroy(&nested);
            return;
        }
    }

    PtnArray *array = ptn_runtime_array_root_for_write(runtime, name, line);
    ptn_array_path_set_from_root(
        runtime,
        array,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED void ptn_runtime_array_path_set(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_path_set_impl(runtime, name, segments, segment_count, value, line, 1);
}

static PTN_UNUSED void ptn_runtime_array_path_set_with_key_diagnostics(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_runtime_array_path_set_impl(
        runtime,
        name,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED int ptn_runtime_array_path_prepare_guarded_root_after_dimension_eval(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line,
    PtnArray **array_out,
    PtnValue *guard_root_out,
    uint64_t *guarded_epoch_out
) {
    *array_out = NULL;
    *guard_root_out = ptn_null();
    *guarded_epoch_out = 0;
    if (runtime == NULL ||
        name == NULL ||
        ptn_runtime_is_globals_name(name) ||
        segment_count == 0) {
        return 0;
    }

    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    PtnValue *slot = ptn_symbols_value_slot(symbols, name);
    if (slot == NULL) {
        PtnValue array_value = ptn_array_from_literal_entries(0, NULL);
        ptn_runtime_write_variable(runtime, name, array_value);
        ptn_value_destroy(&array_value);
        slot = ptn_symbols_value_slot(symbols, name);
        if (slot == NULL) {
            return 2;
        }
    }

    PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    PtnValue resolved = ptn_value_deref(*slot_value);
    int emitted_false_conversion_diagnostic = 0;
    PtnArray *array = NULL;
    if (resolved.type == PTN_ARRAY) {
        array = ptn_array_detach_value(slot_value);
    } else if (resolved.type == PTN_NULL) {
        array = ptn_array_root_slot_for_write(runtime, slot, line);
    } else if (resolved.type == PTN_BOOL && !resolved.as.boolean) {
        (void)ptn_value_replace_with_empty_array(slot_value);
        array = ptn_array_detach_value(slot_value);
        emitted_false_conversion_diagnostic = 1;
    } else {
        return 0;
    }
    if (array == NULL) {
        return 2;
    }

    *array_out = array;
    *guard_root_out = ptn_value_clone_deref(*slot_value);
    *guarded_epoch_out = ptn_runtime_symbol_table_epoch_for_name(runtime, name);

    if (emitted_false_conversion_diagnostic) {
        size_t guarded_array_refcount =
            ptn_array_path_root_snapshot_array_refcount(*guard_root_out);
        ptn_emit_false_array_conversion_deprecation(runtime, line);
        if (ptn_array_path_write_guard_invalidated(
            runtime,
            name,
            *guard_root_out,
            *guarded_epoch_out,
            guarded_array_refcount,
            1,
            segments,
            segment_count,
            line
        )) {
            ptn_value_destroy(guard_root_out);
            *guard_root_out = ptn_null();
            *array_out = NULL;
            return 2;
        }
    }

    return 1;
}

static PTN_UNUSED void ptn_runtime_array_path_set_after_dimension_eval(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root,
    uint64_t pre_eval_epoch,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    (void)pre_eval_epoch;
    if (!ptn_runtime_array_path_root_matches_snapshot(runtime, name, pre_eval_root)) {
        ptn_emit_invalidated_array_path_write_diagnostics(
            runtime,
            pre_eval_root,
            segments,
            segment_count,
            line
        );
        return;
    }
    PtnArray *array = NULL;
    PtnValue guard_root = ptn_null();
    uint64_t guarded_epoch = 0;
    int guard_state = ptn_runtime_array_path_prepare_guarded_root_after_dimension_eval(
        runtime,
        name,
        segments,
        segment_count,
        line,
        &array,
        &guard_root,
        &guarded_epoch
    );
    if (guard_state == 1) {
        ptn_array_path_set_from_root_after_dimension_eval(
            runtime,
            name,
            guard_root,
            guarded_epoch,
            array,
            segments,
            segment_count,
            value,
            line,
            emit_null_key_deprecation
        );
        ptn_value_destroy(&guard_root);
        return;
    }
    if (guard_state == 2) {
        return;
    }
    ptn_runtime_array_path_set_impl(
        runtime,
        name,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_set_result_after_dimension_eval(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root,
    uint64_t pre_eval_epoch,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    (void)pre_eval_epoch;
    if (!ptn_runtime_array_path_root_matches_snapshot(runtime, name, pre_eval_root)) {
        ptn_emit_invalidated_array_path_write_diagnostics(
            runtime,
            pre_eval_root,
            segments,
            segment_count,
            line
        );
        return ptn_null();
    }
    PtnArray *array = NULL;
    PtnValue guard_root = ptn_null();
    uint64_t guarded_epoch = 0;
    int guard_state = ptn_runtime_array_path_prepare_guarded_root_after_dimension_eval(
        runtime,
        name,
        segments,
        segment_count,
        line,
        &array,
        &guard_root,
        &guarded_epoch
    );
    if (guard_state == 1) {
        PtnValue result = ptn_array_path_set_result_from_root_after_dimension_eval(
            runtime,
            name,
            guard_root,
            guarded_epoch,
            array,
            segments,
            segment_count,
            value,
            line,
            emit_null_key_deprecation
        );
        ptn_value_destroy(&guard_root);
        return result;
    }
    if (guard_state == 2) {
        return ptn_null();
    }
    return ptn_runtime_array_path_set_result_impl(
        runtime,
        name,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_set_result_impl(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (ptn_runtime_is_globals_name(name)) {
        return ptn_runtime_globals_array_path_set_result_impl(
            runtime,
            segments,
            segment_count,
            value,
            line,
            emit_null_key_deprecation
        );
    }
    if (segment_count == 0) {
        return ptn_value_clone_deref(value);
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot != NULL && segments[0].append) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            return ptn_null();
        }
    }
    if (slot != NULL && segment_count == 1) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[0].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
                return ptn_null();
            }
            uint64_t pre_key_diagnostic_epoch = ptn_runtime_symbol_table_epoch_for_name(runtime, name);
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
            if (ptn_runtime_symbol_table_epoch_for_name(runtime, name) == pre_key_diagnostic_epoch) {
                slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
                slot_value = slot == NULL ? NULL : (slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot);
                if (slot_value != NULL && slot_value->type == PTN_STRING) {
                    return ptn_runtime_string_offset_set_result(runtime, slot_value, segments[0].value, value, line);
                }
            }
            int64_t offset = 0;
            (void)ptn_string_offset_from_value(runtime, segments[0].value, line, 0, &offset);
            return ptn_null();
        }
    }
    if (slot != NULL && segment_count > 1) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[0].append) {
                ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            } else {
                ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
                ptn_reject_nested_string_offset_array_access(runtime, segments[0].value, line);
            }
            return ptn_null();
        }
    }
    if (slot != NULL && segment_count == 1) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_weak_map_reject_append_offset(runtime, slot_value, &segments[0])) {
            return ptn_null();
        }
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetSet")) {
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
            ptn_arrayaccess_write(runtime, slot_value, key, value, line);
            return ptn_value_clone_deref(value);
        }
    }
    if (slot != NULL && segment_count > 1) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_weak_map_reject_append_offset(runtime, slot_value, &segments[0])) {
            return ptn_null();
        }
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            if (segments[0].append) {
                PtnValue append_reference = ptn_null();
                if (ptn_arrayaccess_append_reference_temporary(runtime, slot_value, line, &append_reference)) {
                    PtnValue result = ptn_value_array_path_set_result(
                        runtime,
                        &append_reference,
                        segments + 1,
                        segment_count - 1,
                        value,
                        line
                    );
                    ptn_value_destroy(&append_reference);
                    return result;
                }
            }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
            PtnValue nested_reference = ptn_null();
            if (ptn_internal_array_object_offset_reference(
                runtime,
                slot_value,
                offset_value,
                line,
                1,
                &nested_reference
            )) {
                PtnValue result = ptn_value_array_path_set_result(
                    runtime,
                    &nested_reference,
                    segments + 1,
                    segment_count - 1,
                    value,
                    line
                );
                ptn_value_destroy(&nested_reference);
                return result;
            }
#endif
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
            PtnValue nested = ptn_arrayaccess_read(runtime, slot_value, key, line);
            PtnValue result = ptn_value_clone_deref(value);
            if (ptn_arrayaccess_nested_write_should_apply(runtime, slot_value, nested, line)) {
                ptn_value_destroy(&result);
                result = ptn_value_array_path_set_result(
                    runtime,
                    &nested,
                    segments + 1,
                    segment_count - 1,
                    value,
                    line
                );
            }
            ptn_value_destroy(&nested);
            return result;
        }
    }

    PtnArray *array = ptn_runtime_array_root_for_write(runtime, name, line);
    return ptn_array_path_set_result_from_root(
        runtime,
        array,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_set_result(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    return ptn_runtime_array_path_set_result_impl(
        runtime,
        name,
        segments,
        segment_count,
        value,
        line,
        1
    );
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_set_result_with_key_diagnostics(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    return ptn_runtime_array_path_set_result_impl(
        runtime,
        name,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED void ptn_runtime_array_path_set_from_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    if (ptn_runtime_has_active_exception(runtime)) {
        return;
    }
    ptn_runtime_array_path_set_impl(runtime, name, segments, segment_count, value, line, 0);
}

static PTN_UNUSED void ptn_runtime_array_path_set_from_inc_dec(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue current,
    PtnValue value,
    size_t line
) {
    if (segment_count == 0 || ptn_runtime_is_globals_name(name)) {
        ptn_runtime_array_path_set_from_assign_op(runtime, name, segments, segment_count, value, line);
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot != NULL && segment_count == 1) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
            PtnValue reference = ptn_null();
            if (ptn_internal_array_object_offset_reference_without_key_diagnostics(
                    runtime,
                    slot_value,
                    offset_value,
                    line,
                    1,
                    &reference
                )) {
                if (reference.type == PTN_REFERENCE) {
                    ptn_reference_assign(runtime, reference.as.reference, value);
                }
                ptn_value_destroy(&reference);
                return;
            }
#endif
            if (current.type == PTN_REFERENCE) {
                ptn_reference_assign(runtime, current.as.reference, value);
            } else {
                ptn_emit_indirect_modification_overloaded_element_notice(runtime, slot_value, line);
            }
            return;
        }
    }

    ptn_runtime_array_path_set_from_assign_op(runtime, name, segments, segment_count, value, line);
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_read_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_null();
    }
    if (ptn_runtime_is_globals_name(name)) {
        char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
        if (global_name == NULL) {
            return ptn_null();
        }

        PtnValue root_value;
        if (!ptn_symbols_get(ptn_runtime_global_symbol_table(runtime), global_name, &root_value)) {
            ptn_emit_undefined_global_variable_warning(
                &runtime->diagnostics,
                global_name,
                runtime->source_path,
                line
            );
            free(global_name);
            return ptn_null();
        }
        free(global_name);

        if (segment_count == 1) {
            return ptn_value_clone(root_value);
        }
        return ptn_value_array_path_read_for_assign_op(
            runtime,
            root_value,
            segments + 1,
            segment_count - 1,
            line
        );
    }

    if (ptn_runtime_is_globals_name(name)) {
        char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
        if (global_name == NULL) {
            return ptn_null();
        }
        PtnLookupResult root = ptn_runtime_read_global_variable_quiet(runtime, global_name);
        free(global_name);
        if (!root.exists) {
            ptn_emit_assign_op_missing_array_key(runtime, segments[0].value, line);
            return ptn_null();
        }
        if (segment_count == 1) {
            return ptn_value_clone(root.value);
        }
        return ptn_value_array_path_read_for_assign_op(
            runtime,
            root.value,
            segments + 1,
            segment_count - 1,
            line
        );
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot == NULL) {
        if (!segments[0].append) {
            ptn_emit_assign_op_missing_array_key(runtime, segments[0].value, line);
        }
        return ptn_null();
    }
    PtnValue slot_value = ptn_value_deref(*slot);
    if (slot_value.type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            return ptn_null();
        }
        int64_t offset = 0;
        if (!ptn_string_offset_from_value(runtime, segments[0].value, line, 0, &offset)) {
            return ptn_null();
        }
        (void)offset;
        if (segment_count > 1) {
            fflush(stdout);
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Cannot use string offset as an array",
                runtime->source_path,
                line
            );
            return ptn_null();
        }
        ptn_throw_exception_at(runtime, "Error", "Cannot use assign-op operators with string offsets", runtime->source_path, line);
        return ptn_null();
    }
    if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
        if (segment_count == 1) {
            PtnLookupResult lookup = ptn_lookup_missing();
            if (ptn_internal_array_object_offset_lookup_for_assign_op(
                    runtime,
                    slot_value,
                    offset_value,
                    line,
                    &lookup
                )) {
                return lookup.exists ? lookup.value : ptn_null();
            }
        }
        PtnValue reference = ptn_null();
        if (ptn_internal_array_object_offset_reference(
                runtime,
                slot_value,
                offset_value,
                line,
                0,
                &reference
            )) {
            if (segment_count == 1) {
                return reference;
            }
            PtnValue result = ptn_value_array_path_read_for_assign_op(
                runtime,
                reference,
                segments + 1,
                segment_count - 1,
                line
            );
            ptn_value_destroy(&reference);
            return result;
        }
#endif
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue nested = ptn_arrayaccess_read(runtime, slot_value, key, line);
        if (segment_count == 1) {
            return nested;
        }
        PtnValue result = ptn_value_array_path_read_for_assign_op(
            runtime,
            nested,
            segments + 1,
            segment_count - 1,
            line
        );
        ptn_value_destroy(&nested);
        return result;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, slot_value)) {
        ptn_throw_cannot_use_object_as_array(runtime, slot_value, line);
        return ptn_null();
    }
    if (slot_value.type == PTN_BOOL && !slot_value.as.boolean) {
        PtnValue *slot_target = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        (void)ptn_value_replace_with_empty_array(slot_target);
        ptn_emit_false_array_conversion_deprecation(runtime, line);
        slot_value = ptn_value_deref(*slot);
    }

    PtnValue container = ptn_value_borrow(slot_value);
    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return ptn_null();
        }
        if (container.type != PTN_ARRAY && container.type != PTN_NULL) {
            return ptn_null();
        }
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 1);
        PtnArrayKey key;
        if (!ptn_array_offset_key_from_value(runtime, segment->value, line, 0, &key)) {
            return ptn_null();
        }
        if (container.type == PTN_ARRAY) {
            PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
            if (entry == NULL) {
                ptn_emit_undefined_array_key_warning(runtime, key, line);
                ptn_array_key_free(key);
                if (i + 1 == segment_count) {
                    return ptn_null();
                }
                container = ptn_null();
                continue;
            }
            ptn_array_key_free(key);
            if (i + 1 == segment_count) {
                return ptn_value_clone(entry->value);
            }
            container = ptn_value_deref(entry->value);
            continue;
        }
        if (container.type == PTN_NULL) {
            ptn_emit_undefined_array_key_warning(runtime, key, line);
            ptn_array_key_free(key);
            if (i + 1 == segment_count) {
                return ptn_null();
            }
            container = ptn_null();
            continue;
        }
        ptn_array_key_free(key);
        return ptn_null();
    }
    return ptn_null();
}

static PTN_UNUSED int ptn_runtime_array_path_indirect_receiver_uses_arrayaccess_get(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count
) {
    if (segment_count == 0) {
        return 0;
    }

    if (!ptn_runtime_is_globals_name(name)) {
        PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
        PtnValue *slot = ptn_symbols_value_slot(symbols, name);
        if (slot == NULL) {
            return 0;
        }
        PtnValue slot_value = ptn_value_deref(*slot);
        return ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet");
    }

    if (segment_count < 2) {
        return 0;
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return 0;
    }
    PtnValue *slot = ptn_runtime_global_variable_slot(runtime, global_name);
    free(global_name);
    if (slot == NULL) {
        return 0;
    }

    PtnValue slot_value = ptn_value_deref(*slot);
    return ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet");
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_read_for_indirect_write_receiver(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (ptn_runtime_array_path_indirect_receiver_uses_arrayaccess_get(
            runtime,
            name,
            segments,
            segment_count
        )) {
        return ptn_runtime_array_path_read_for_assign_op(
            runtime,
            name,
            segments,
            segment_count,
            line
        );
    }

    PtnLookupResult lookup = ptn_runtime_array_path_lookup_quiet(
        runtime,
        name,
        segments,
        segment_count,
        line
    );
    if (lookup.exists) {
        return lookup.value;
    }

    ptn_value_destroy(&lookup.value);
    return ptn_runtime_array_path_set_result(
        runtime,
        name,
        segments,
        segment_count,
        ptn_null(),
        line
    );
}

static PTN_UNUSED int ptn_runtime_array_path_root_false_converted_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    PtnValue pre_eval_root
) {
    PtnValue root = ptn_value_deref(pre_eval_root);
    if (root.type != PTN_BOOL || root.as.boolean) {
        return 0;
    }
    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot == NULL) {
        return 0;
    }
    PtnValue current = ptn_value_deref(*slot);
    return current.type == PTN_ARRAY;
}

static PTN_UNUSED int ptn_runtime_array_path_read_overloaded_base_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line,
    PtnValue *base_out,
    PtnValue *container_out
) {
    *container_out = ptn_null();
    if (segment_count > 1 && !ptn_runtime_is_globals_name(name)) {
        PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
        if (slot != NULL) {
            PtnValue slot_value = ptn_value_deref(*slot);
            if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
                *container_out = ptn_value_clone(slot_value);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
                const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
                if (ptn_internal_array_object_offset_reference(
                        runtime,
                        slot_value,
                        offset_value,
                        line,
                        0,
                        base_out
                    )) {
                    return 1;
                }
#endif
                PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
                *base_out = ptn_arrayaccess_read(runtime, slot_value, key, line);
                return 1;
            }
        }
    }

    *base_out = ptn_runtime_array_path_read_for_assign_op(
        runtime,
        name,
        segments,
        segment_count,
        line
    );
    return 0;
}

static PTN_UNUSED int ptn_value_array_path_read_overloaded_base_for_assign_op(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line,
    PtnValue *base_out,
    PtnValue *container_out
) {
    *container_out = ptn_null();
    PtnValue target_value = ptn_value_deref(target);
    if (segment_count > 1 && ptn_arrayaccess_can_dispatch(runtime, target_value, "offsetGet")) {
        *container_out = ptn_value_clone(target_value);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
        if (ptn_internal_array_object_offset_reference(
                runtime,
                target_value,
                offset_value,
                line,
                0,
                base_out
            )) {
            return 1;
        }
#endif
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        *base_out = ptn_arrayaccess_read(runtime, target_value, key, line);
        return 1;
    }

    *base_out = ptn_value_array_path_read_for_assign_op(
        runtime,
        target,
        segments,
        segment_count,
        line
    );
    return 0;
}

static PTN_UNUSED void ptn_value_array_path_set_impl(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (target == NULL || segment_count == 0) {
        return;
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (target_value->type == PTN_STRING && segments[0].append) {
        ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
        return;
    }
    if (target_value->type == PTN_STRING && segment_count > 1) {
        ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
        ptn_reject_nested_string_offset_array_access(runtime, segments[0].value, line);
        return;
    }
    if (segment_count == 1 && target_value->type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            return;
        }
        ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
        ptn_runtime_string_offset_set(runtime, target_value, segments[0].value, value, line);
        return;
    }
    if (ptn_weak_map_reject_append_offset(runtime, *target_value, &segments[0])) {
        return;
    }
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetSet")) {
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        ptn_arrayaccess_write(runtime, *target_value, key, value, line);
        return;
    }
    if (segment_count > 1 && ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetGet")) {
        if (segments[0].append) {
            PtnValue append_reference = ptn_null();
            if (ptn_arrayaccess_append_reference_temporary(runtime, *target_value, line, &append_reference)) {
                ptn_value_array_path_set_impl(
                    runtime,
                    &append_reference,
                    segments + 1,
                    segment_count - 1,
                    value,
                    line,
                    emit_null_key_deprecation
                );
                ptn_value_destroy(&append_reference);
                return;
            }
        }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
        PtnValue nested_reference = ptn_null();
        if (ptn_internal_array_object_offset_reference(
            runtime,
            *target_value,
            offset_value,
            line,
            1,
            &nested_reference
        )) {
            ptn_value_array_path_set_impl(
                runtime,
                &nested_reference,
                segments + 1,
                segment_count - 1,
                value,
                line,
                emit_null_key_deprecation
            );
            ptn_value_destroy(&nested_reference);
            return;
        }
#endif
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue nested = ptn_arrayaccess_read(runtime, *target_value, key, line);
        if (ptn_arrayaccess_nested_write_should_apply(runtime, *target_value, nested, line)) {
            ptn_value_array_path_set_impl(
                runtime,
                &nested,
                segments + 1,
                segment_count - 1,
                value,
                line,
                emit_null_key_deprecation
            );
        }
        ptn_value_destroy(&nested);
        return;
    }

    PtnArray *array = ptn_array_root_slot_for_write(runtime, target, line);
    ptn_array_path_set_from_root(
        runtime,
        array,
        segments,
        segment_count,
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED void ptn_value_array_path_set(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    ptn_value_array_path_set_impl(runtime, target, segments, segment_count, value, line, 1);
}

static PTN_UNUSED PtnValue ptn_value_array_path_set_result(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    if (target == NULL || segment_count == 0) {
        return ptn_value_clone_deref(value);
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (target_value->type == PTN_STRING && segments[0].append) {
        ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
        return ptn_null();
    }
    if (target_value->type == PTN_STRING && segment_count > 1) {
        ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
        ptn_reject_nested_string_offset_array_access(runtime, segments[0].value, line);
        return ptn_null();
    }
    if (segment_count == 1 && target_value->type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            return ptn_null();
        }
        ptn_array_path_emit_deferred_undefined_variable_warning(runtime, &segments[0], line);
        return ptn_runtime_string_offset_set_result(runtime, target_value, segments[0].value, value, line);
    }
    if (ptn_weak_map_reject_append_offset(runtime, *target_value, &segments[0])) {
        return ptn_null();
    }
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetSet")) {
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        ptn_arrayaccess_write(runtime, *target_value, key, value, line);
        return ptn_value_clone_deref(value);
    }
    if (segment_count > 1 && ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetGet")) {
        if (segments[0].append) {
            PtnValue append_reference = ptn_null();
            if (ptn_arrayaccess_append_reference_temporary(runtime, *target_value, line, &append_reference)) {
                PtnValue result = ptn_value_array_path_set_result(
                    runtime,
                    &append_reference,
                    segments + 1,
                    segment_count - 1,
                    value,
                    line
                );
                ptn_value_destroy(&append_reference);
                return result;
            }
        }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
        PtnValue nested_reference = ptn_null();
        if (ptn_internal_array_object_offset_reference(
            runtime,
            *target_value,
            offset_value,
            line,
            1,
            &nested_reference
        )) {
            PtnValue result = ptn_value_array_path_set_result(
                runtime,
                &nested_reference,
                segments + 1,
                segment_count - 1,
                value,
                line
            );
            ptn_value_destroy(&nested_reference);
            return result;
        }
#endif
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue nested = ptn_arrayaccess_read(runtime, *target_value, key, line);
        PtnValue result = ptn_value_clone_deref(value);
        if (ptn_arrayaccess_nested_write_should_apply(runtime, *target_value, nested, line)) {
            ptn_value_destroy(&result);
            result = ptn_value_array_path_set_result(
                runtime,
                &nested,
                segments + 1,
                segment_count - 1,
                value,
                line
            );
        }
        ptn_value_destroy(&nested);
        return result;
    }

    PtnArray *array = ptn_array_root_slot_for_write(runtime, target, line);
    return ptn_array_path_set_result_from_root(
        runtime,
        array,
        segments,
        segment_count,
        value,
        line,
        1
    );
}

static PTN_UNUSED void ptn_value_array_path_set_from_assign_op(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    if (ptn_runtime_has_active_exception(runtime)) {
        return;
    }
    ptn_value_array_path_set_impl(runtime, target, segments, segment_count, value, line, 0);
}

static PTN_UNUSED void ptn_value_array_path_set_from_overloaded_assign_op(
    PtnRuntime *runtime,
    PtnValue *target,
    PtnValue container,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    if (target == NULL) {
        return;
    }
    if (ptn_runtime_has_active_exception(runtime)) {
        return;
    }
    if (ptn_arrayaccess_nested_write_should_apply(runtime, container, *target, line)) {
        ptn_value_array_path_set_from_assign_op(runtime, target, segments, segment_count, value, line);
    }
}

static PTN_UNUSED void ptn_value_array_path_set_from_inc_dec(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue current,
    PtnValue value,
    size_t line
) {
    if (target == NULL || segment_count == 0) {
        ptn_value_array_path_set_from_assign_op(runtime, target, segments, segment_count, value, line);
        return;
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetGet")) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
        PtnValue reference = ptn_null();
        if (ptn_internal_array_object_offset_reference_without_key_diagnostics(
                runtime,
                *target_value,
                offset_value,
                line,
                1,
                &reference
            )) {
            if (reference.type == PTN_REFERENCE) {
                ptn_reference_assign(runtime, reference.as.reference, value);
            }
            ptn_value_destroy(&reference);
            return;
        }
#endif
        if (current.type == PTN_REFERENCE) {
            ptn_reference_assign(runtime, current.as.reference, value);
        } else {
            ptn_emit_indirect_modification_overloaded_element_notice(runtime, *target_value, line);
        }
        return;
    }

    ptn_value_array_path_set_from_assign_op(runtime, target, segments, segment_count, value, line);
}

static PTN_UNUSED PtnValue ptn_value_array_path_read_for_assign_op_impl(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line,
    const char *string_offset_write_message,
    int emit_key_conversion_diagnostics
) {
    if (segment_count == 0) {
        return ptn_null();
    }

    PtnValue slot_value = ptn_value_deref(target);
    if (slot_value.type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception_at(runtime, "Error", "[] operator not supported for strings", runtime->source_path, line);
            return ptn_null();
        }
        int64_t offset = 0;
        if (!ptn_string_offset_from_value(runtime, segments[0].value, line, 0, &offset)) {
            return ptn_null();
        }
        (void)offset;
        if (segment_count > 1) {
            fflush(stdout);
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Cannot use string offset as an array",
                runtime->source_path,
                line
            );
            return ptn_null();
        }
        ptn_throw_exception_at(runtime, "Error", string_offset_write_message, runtime->source_path, line);
        return ptn_null();
    }
    if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        const PtnValue *offset_value = segments[0].append ? NULL : &segments[0].value;
        if (segment_count == 1) {
            PtnLookupResult lookup = ptn_lookup_missing();
            if (ptn_internal_array_object_offset_lookup_for_assign_op(
                    runtime,
                    slot_value,
                    offset_value,
                    line,
                    &lookup
                )) {
                return lookup.exists ? lookup.value : ptn_null();
            }
        }
        PtnValue reference = ptn_null();
        if (ptn_internal_array_object_offset_reference(
                runtime,
                slot_value,
                offset_value,
                line,
                0,
                &reference
            )) {
            if (segment_count == 1) {
                return reference;
            }
            PtnValue result = ptn_value_array_path_read_for_assign_op_impl(
                runtime,
                reference,
                segments + 1,
                segment_count - 1,
                line,
                string_offset_write_message,
                emit_key_conversion_diagnostics
            );
            ptn_value_destroy(&reference);
            return result;
        }
#endif
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue nested = ptn_arrayaccess_read(runtime, slot_value, key, line);
        if (segment_count == 1) {
            return nested;
        }
        PtnValue result = ptn_value_array_path_read_for_assign_op_impl(
            runtime,
            nested,
            segments + 1,
            segment_count - 1,
            line,
            string_offset_write_message,
            emit_key_conversion_diagnostics
        );
        ptn_value_destroy(&nested);
        return result;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, slot_value)) {
        ptn_throw_cannot_use_object_as_array(runtime, slot_value, line);
        return ptn_null();
    }

    PtnValue container = ptn_value_borrow(slot_value);
    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return ptn_null();
        }
        if (container.type != PTN_ARRAY && container.type != PTN_NULL) {
            return ptn_null();
        }
        if (emit_key_conversion_diagnostics) {
            ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 1);
        }
        PtnArrayKey key;
        if (!ptn_array_offset_key_from_value(
            runtime,
            segment->value,
            line,
            !emit_key_conversion_diagnostics,
            &key
        )) {
            return ptn_null();
        }
        if (container.type == PTN_NULL) {
            ptn_emit_undefined_array_key_warning(runtime, key, line);
            ptn_array_key_free(key);
            if (i + 1 == segment_count) {
                return ptn_null();
            }
            container = ptn_null();
            continue;
        }
        PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
        if (entry == NULL) {
            ptn_emit_undefined_array_key_warning(runtime, key, line);
            ptn_array_key_free(key);
            if (i + 1 == segment_count) {
                return ptn_null();
            }
            container = ptn_null();
            continue;
        }
        ptn_array_key_free(key);
        if (i + 1 == segment_count) {
            return ptn_value_clone(entry->value);
        }
        container = ptn_value_deref(entry->value);
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_value_array_path_read_for_assign_op(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    return ptn_value_array_path_read_for_assign_op_impl(
        runtime,
        target,
        segments,
        segment_count,
        line,
        "Cannot use assign-op operators with string offsets",
        1
    );
}

static PTN_UNUSED PtnValue ptn_value_array_path_read_for_inc_dec(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    return ptn_value_array_path_read_for_assign_op_impl(
        runtime,
        target,
        segments,
        segment_count,
        line,
        "Cannot increment/decrement string offsets",
        1
    );
}

static PTN_UNUSED PtnValue ptn_value_array_path_read_for_overloaded_assign_op(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    return ptn_value_array_path_read_for_assign_op_impl(
        runtime,
        target,
        segments,
        segment_count,
        line,
        "Cannot use assign-op operators with string offsets",
        1
    );
}

static PTN_UNUSED int ptn_array_unset_non_array_value(PtnRuntime *runtime, PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_NULL) {
        return 0;
    }
    ptn_throw_exception(runtime, "Error", "Cannot unset offset in a non-array variable");
    return 1;
}

static PTN_UNUSED void ptn_value_array_path_unset(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (target == NULL || segment_count == 0) {
        return;
    }

    PtnValue *value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (value->type == PTN_STRING) {
        if (segment_count == 1) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        } else if (segments[0].append) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        } else {
            ptn_reject_nested_string_offset_unset(runtime, segments[0].value, line);
        }
        return;
    }
    if (value->type == PTN_BOOL && !value->as.boolean) {
        ptn_emit_false_array_conversion_deprecation(runtime, line);
        return;
    }
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetUnset")) {
        if (segments[0].append) {
            return;
        }
        ptn_arrayaccess_unset(runtime, *value, segments[0].value, line);
        return;
    }
    if (segment_count > 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetGet")) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        if (!segments[0].append) {
            if (ptn_internal_array_object_uses_builtin_offsets(runtime, *value) &&
                ptn_array_offset_key_is_invalid(segments[0].value)) {
                ptn_emit_indirect_modification_overloaded_element_notice(runtime, *value, line);
                ptn_arrayaccess_unset(runtime, *value, segments[0].value, line);
                return;
            }
            PtnValue nested_reference = ptn_null();
            if (ptn_internal_array_object_offset_reference_quiet(
                    runtime,
                    *value,
                    &segments[0].value,
                    line,
                    &nested_reference
                )) {
                if (ptn_arrayaccess_nested_write_should_apply(runtime, *value, nested_reference, line)) {
                    ptn_value_array_path_unset(runtime, &nested_reference, segments + 1, segment_count - 1, line);
                }
                ptn_value_destroy(&nested_reference);
                return;
            }
        }
#endif
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue nested = ptn_arrayaccess_read(runtime, *value, key, line);
        if (ptn_arrayaccess_nested_write_should_apply(runtime, *value, nested, line)) {
            ptn_value_array_path_unset(runtime, &nested, segments + 1, segment_count - 1, line);
        }
        ptn_value_destroy(&nested);
        return;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, *value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *value, line);
        return;
    }
    if (value->type != PTN_ARRAY) {
        (void)ptn_array_unset_non_array_value(runtime, *value);
        return;
    }

    PtnArray *array = ptn_array_detach_value(value);
    for (size_t i = 0; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return;
        }
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 0);
        if (ptn_array_offset_key_is_invalid(segment->value)) {
            ptn_throw_array_offset_key_type_error(
                runtime,
                segment->value,
                "Cannot access offset of type %s on array",
                line
            );
            return;
        }
        PtnArrayKey key;
        if (!ptn_array_offset_key_from_value(runtime, segment->value, line, 1, &key)) {
            return;
        }
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
        ptn_array_key_free(key);
        if (entry == NULL) {
            return;
        }
        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        if (entry_value->type == PTN_STRING) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
            return;
        }
        if (entry_value->type == PTN_BOOL && !entry_value->as.boolean) {
            ptn_emit_false_array_conversion_deprecation(runtime, line);
            return;
        }
        if (ptn_value_is_plain_object_for_array_offset(runtime, *entry_value)) {
            ptn_throw_cannot_use_object_as_array(runtime, *entry_value, line);
            return;
        }
        if (entry_value->type != PTN_ARRAY) {
            (void)ptn_array_unset_non_array_value(runtime, *entry_value);
            return;
        }
        array = ptn_array_detach_value(entry_value);
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append) {
        return;
    }
    ptn_emit_array_offset_key_conversion_diagnostic(runtime, leaf->value, line, segment_count > 1);
    if (ptn_array_offset_key_is_invalid(leaf->value)) {
        ptn_throw_array_offset_key_type_error(
            runtime,
            leaf->value,
            segment_count == 1
                ? "Cannot unset offset of type %s on array"
                : "Cannot access offset of type %s on array",
            line
        );
        return;
    }
    PtnArrayKey key;
    if (!ptn_array_offset_key_from_value(runtime, leaf->value, line, 1, &key)) {
        return;
    }
    (void)ptn_array_unset_entry(array, key);
}

static PTN_UNUSED void ptn_object_array_path_unset(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return;
    }
    char *read_storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1,
        line
    );
    if (read_storage_key == NULL) {
        return;
    }
    PtnArrayKey read_key = ptn_array_string_key(read_storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, read_key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, read_storage_key);
    if (metadata != NULL && metadata->has_hooks) {
        ptn_array_key_free(read_key);
        free(read_storage_key);
        PtnValue current = ptn_object_read_property_for_indirect_write(
            runtime,
            receiver,
            property,
            access_scope,
            line
        );
        if (ptn_runtime_has_active_exception(runtime)) {
            ptn_value_destroy(&current);
            return;
        }
        ptn_value_array_path_unset(runtime, &current, segments, segment_count, line);
        PtnValue assigned = ptn_object_write_property_indirect(
            runtime,
            receiver,
            property,
            access_scope,
            current,
            line
        );
        ptn_value_destroy(&assigned);
        ptn_value_destroy(&current);
        return;
    }
    ptn_array_key_free(read_key);
    free(read_storage_key);
    if (entry == NULL) {
        if (metadata != NULL && metadata->type_kind == PTN_PROPERTY_TYPE_ARRAY) {
            return;
        }
        PtnValue current = ptn_object_read_property_for_indirect_write(
            runtime,
            receiver,
            property,
            access_scope,
            line
        );
        if (runtime != NULL &&
            runtime->exceptions != NULL &&
            runtime->exceptions->active_exception != NULL) {
            ptn_value_destroy(&current);
            return;
        }
        PtnValue resolved = ptn_value_deref(current);
        if (current.type == PTN_REFERENCE || resolved.type == PTN_OBJECT) {
            ptn_value_array_path_unset(runtime, &current, segments, segment_count, line);
        } else if (
            metadata != NULL &&
            metadata->type_kind == PTN_PROPERTY_TYPE_ARRAY &&
            resolved.type == PTN_ARRAY
        ) {
            ptn_value_array_path_unset(runtime, &current, segments, segment_count, line);
        } else if (resolved.type != PTN_NULL) {
            ptn_emit_indirect_modification_overloaded_property_notice(
                runtime,
                receiver,
                property,
                line
            );
        }
        ptn_value_destroy(&current);
        return;
    }
    char *write_storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
        0,
        line
    );
    if (write_storage_key == NULL) {
        return;
    }
    free(write_storage_key);
    PtnValue current = ptn_value_clone_deref(entry->value);
    ptn_value_array_path_unset(runtime, &current, segments, segment_count, line);
    PtnValue assigned = ptn_object_write_property_indirect(
        runtime,
        receiver,
        property,
        access_scope,
        current,
        line
    );
    ptn_value_destroy(&assigned);
    ptn_value_destroy(&current);
}

static PTN_UNUSED PtnValue ptn_runtime_array_read_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    const PtnValue *key_value,
    size_t line
) {
    (void)path;
    if (key_value == NULL) {
        return ptn_null();
    }
    PtnArrayPathSegment segment = { 0, *key_value };
    return ptn_runtime_array_path_read_for_assign_op(runtime, name, &segment, 1, line);
}

static PTN_UNUSED void ptn_runtime_array_set_impl(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    PtnArrayPathSegment segment = { 0, key_value };
    ptn_runtime_array_path_set_impl(runtime, name, &segment, 1, value, line, emit_null_key_deprecation);
}

static PTN_UNUSED void ptn_runtime_array_set(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_set_impl(runtime, name, key_value, value, line, 1);
}

static PTN_UNUSED void ptn_runtime_array_set_from_assign_op(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_set_impl(runtime, name, key_value, value, line, 0);
}

static PTN_UNUSED void ptn_runtime_array_append(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t line
) {
    PtnArrayPathSegment segment = { 1, ptn_null() };
    ptn_runtime_array_path_set(runtime, name, &segment, 1, value, line);
}

static PTN_UNUSED void ptn_runtime_array_path_unset(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
);

static PTN_UNUSED void ptn_runtime_globals_array_path_unset(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    (void)line;
    if (segment_count == 0) {
        return;
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return;
    }

    if (segment_count == 1) {
        ptn_runtime_unset_global_variable(runtime, global_name);
        free(global_name);
        return;
    }

    PtnValue *slot = ptn_runtime_global_variable_slot(runtime, global_name);
    free(global_name);
    if (slot == NULL) {
        return;
    }

    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_STRING) {
        if (segment_count == 2) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        } else if (segments[1].append) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        } else {
            ptn_reject_nested_string_offset_unset(runtime, segments[1].value, line);
        }
        return;
    }
    if (value->type == PTN_BOOL && !value->as.boolean) {
        ptn_emit_false_array_conversion_deprecation(runtime, line);
        return;
    }
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetUnset")) {
        if (segments[0].append) {
            return;
        }
        ptn_arrayaccess_unset(runtime, *value, segments[0].value, line);
        return;
    }
    if (segment_count > 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetGet")) {
        ptn_value_array_path_unset(runtime, value, segments, segment_count, line);
        return;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, *value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *value, line);
        return;
    }
    if (value->type != PTN_ARRAY) {
        (void)ptn_array_unset_non_array_value(runtime, *value);
        return;
    }

    PtnArray *array = ptn_array_detach_value(value);
    for (size_t i = 1; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return;
        }
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 0);
        if (ptn_array_offset_key_is_invalid(segment->value)) {
            ptn_throw_array_offset_key_type_error(
                runtime,
                segment->value,
                "Cannot access offset of type %s on array",
                line
            );
            return;
        }
        PtnArrayKey key;
        if (!ptn_array_offset_key_from_value(runtime, segment->value, line, 1, &key)) {
            return;
        }
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
        ptn_array_key_free(key);
        if (entry == NULL) {
            return;
        }
        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        if (entry_value->type == PTN_STRING) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
            return;
        }
        if (entry_value->type == PTN_BOOL && !entry_value->as.boolean) {
            ptn_emit_false_array_conversion_deprecation(runtime, line);
            return;
        }
        if (ptn_value_is_plain_object_for_array_offset(runtime, *entry_value)) {
            ptn_throw_cannot_use_object_as_array(runtime, *entry_value, line);
            return;
        }
        if (entry_value->type != PTN_ARRAY) {
            (void)ptn_array_unset_non_array_value(runtime, *entry_value);
            return;
        }
        array = ptn_array_detach_value(entry_value);
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append) {
        return;
    }
    ptn_emit_array_offset_key_conversion_diagnostic(runtime, leaf->value, line, segment_count > 1);
    if (ptn_array_offset_key_is_invalid(leaf->value)) {
        ptn_throw_array_offset_key_type_error(
            runtime,
            leaf->value,
            segment_count == 1
                ? "Cannot unset offset of type %s on array"
                : "Cannot access offset of type %s on array",
            line
        );
        return;
    }
    PtnArrayKey key;
    if (!ptn_array_offset_key_from_value(runtime, leaf->value, line, 1, &key)) {
        return;
    }
    (void)ptn_array_unset_entry(array, key);
}

static PTN_UNUSED void ptn_runtime_array_unset(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    size_t line
) {
    (void)line;
    PtnArrayPathSegment segment = { 0, key_value };
    ptn_runtime_array_path_unset(runtime, name, &segment, 1, line);
}

static PTN_UNUSED void ptn_runtime_array_path_unset(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (ptn_runtime_is_globals_name(name)) {
        ptn_runtime_globals_array_path_unset(runtime, segments, segment_count, line);
        return;
    }
    (void)line;
    if (segment_count == 0) {
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot == NULL) {
        if (ptn_runtime_is_auto_global_symbol_name(name)) {
            ptn_emit_undefined_global_variable_warning(&runtime->diagnostics, name, runtime->source_path, line);
            return;
        }
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, runtime->source_path, line);
        return;
    }
    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_STRING) {
        if (segment_count == 1) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        } else if (segments[0].append) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        } else {
            ptn_reject_nested_string_offset_unset(runtime, segments[0].value, line);
        }
        return;
    }
    if (value->type == PTN_BOOL && !value->as.boolean) {
        ptn_emit_false_array_conversion_deprecation(runtime, line);
        return;
    }
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetUnset")) {
        if (segments[0].append) {
            return;
        }
        ptn_arrayaccess_unset(runtime, *value, segments[0].value, line);
        return;
    }
    if (segment_count > 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetGet")) {
        ptn_value_array_path_unset(runtime, value, segments, segment_count, line);
        return;
    }
    if (ptn_value_is_plain_object_for_array_offset(runtime, *value)) {
        ptn_throw_cannot_use_object_as_array(runtime, *value, line);
        return;
    }
    if (value->type != PTN_ARRAY) {
        (void)ptn_array_unset_non_array_value(runtime, *value);
        return;
    }

    PtnArray *array = ptn_array_detach_value(value);
    for (size_t i = 0; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return;
        }
        ptn_emit_array_offset_key_conversion_diagnostic(runtime, segment->value, line, 0);
        if (ptn_array_offset_key_is_invalid(segment->value)) {
            ptn_throw_array_offset_key_type_error(
                runtime,
                segment->value,
                "Cannot access offset of type %s on array",
                line
            );
            return;
        }
        PtnArrayKey key;
        if (!ptn_array_offset_key_from_value(runtime, segment->value, line, 1, &key)) {
            return;
        }
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
        ptn_array_key_free(key);
        if (entry == NULL) {
            return;
        }
        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        if (entry_value->type == PTN_STRING) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
            return;
        }
        if (entry_value->type == PTN_BOOL && !entry_value->as.boolean) {
            ptn_emit_false_array_conversion_deprecation(runtime, line);
            return;
        }
        if (ptn_value_is_plain_object_for_array_offset(runtime, *entry_value)) {
            ptn_throw_cannot_use_object_as_array(runtime, *entry_value, line);
            return;
        }
        if (entry_value->type != PTN_ARRAY) {
            (void)ptn_array_unset_non_array_value(runtime, *entry_value);
            return;
        }
        array = ptn_array_detach_value(entry_value);
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append) {
        return;
    }
    ptn_emit_array_offset_key_conversion_diagnostic(runtime, leaf->value, line, segment_count > 1);
    if (ptn_array_offset_key_is_invalid(leaf->value)) {
        ptn_throw_array_offset_key_type_error(
            runtime,
            leaf->value,
            segment_count == 1
                ? "Cannot unset offset of type %s on array"
                : "Cannot access offset of type %s on array",
            line
        );
        return;
    }
    PtnArrayKey key;
    if (!ptn_array_offset_key_from_value(runtime, leaf->value, line, 1, &key)) {
        return;
    }
    (void)ptn_array_unset_entry(array, key);
}

static PTN_UNUSED PtnValue ptn_array_current_value(PtnArray *array) {
    if (array->current_index >= array->len) {
        return ptn_bool(0);
    }
    return ptn_value_share(array->entries[array->current_index].value);
}

static PTN_UNUSED PtnValue ptn_array_current_key_value(PtnArray *array) {
    if (array->current_index >= array->len) {
        return ptn_null();
    }
    return ptn_array_key_value(array->entries[array->current_index].key);
}

static PTN_UNUSED PtnValue ptn_array_next_value(PtnArray *array) {
    if (array->len == 0 || array->current_index + 1 >= array->len) {
        array->current_index = array->len;
        return ptn_bool(0);
    }
    array->current_index++;
    return ptn_value_share(array->entries[array->current_index].value);
}

static PTN_UNUSED PtnValue ptn_array_reset_value(PtnArray *array) {
    array->current_index = 0;
    return ptn_array_current_value(array);
}

static PTN_UNUSED PtnValue ptn_array_end_value(PtnArray *array) {
    if (array->len == 0) {
        array->current_index = 0;
        return ptn_bool(0);
    }
    array->current_index = array->len - 1;
    return ptn_array_current_value(array);
}

static PTN_UNUSED PtnValue ptn_array_prev_value(PtnArray *array) {
    if (array->len == 0 || array->current_index == 0 || array->current_index >= array->len) {
        array->current_index = array->len;
        return ptn_bool(0);
    }
    array->current_index--;
    return ptn_array_current_value(array);
}

static PTN_UNUSED PtnValue ptn_array_pop_value(PtnArray *array) {
    if (array->len == 0) {
        array->current_index = 0;
        return ptn_null();
    }

    size_t removed_index = array->len - 1;
    PtnArrayKey removed_key = array->entries[removed_index].key;
    int64_t old_next_auto_key = array->next_auto_key;
    PtnValue removed = ptn_value_clone_deref(array->entries[removed_index].value);
    ptn_array_index_remove(array, removed_key);
    ptn_value_destroy(&array->entries[removed_index].value);
    ptn_array_key_free(removed_key);
    array->len--;
    array->current_index = 0;
    if (removed_key.type == PTN_ARRAY_KEY_INT &&
        ptn_array_next_auto_key_after_integer(removed_key.as.integer) == old_next_auto_key) {
        array->next_auto_key = removed_key.as.integer;
    } else {
        array->next_auto_key = old_next_auto_key;
    }
    return removed;
}

static PTN_UNUSED PtnValue ptn_array_shift_value(PtnArray *array) {
    if (array->len == 0) {
        array->current_index = 0;
        return ptn_null();
    }

    PtnValue removed = ptn_value_clone_deref(array->entries[0].value);
    ptn_value_destroy(&array->entries[0].value);
    ptn_array_key_free(array->entries[0].key);
    for (size_t i = 1; i < array->len; i++) {
        array->entries[i - 1] = array->entries[i];
    }
    array->len--;

    int64_t next_integer_key = 0;
    for (size_t i = 0; i < array->len; i++) {
        if (array->entries[i].key.type == PTN_ARRAY_KEY_INT) {
            array->entries[i].key.as.integer = next_integer_key;
            if (next_integer_key < INT64_MAX) {
                next_integer_key++;
            }
        }
    }

    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    PtnValue result = ptn_value_clone_deref(removed);
    ptn_value_destroy(&removed);
    return result;
}

static PTN_UNUSED int64_t ptn_array_unshift_values(PtnArray *array, size_t argc, const PtnValue *values) {
    if (argc > SIZE_MAX - array->len) {
        ptn_abort_out_of_memory();
    }
    size_t new_len = array->len + argc;
    if (new_len > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }

    PtnArrayEntry *new_entries = NULL;
    if (new_len != 0) {
        if (new_len > SIZE_MAX / sizeof(PtnArrayEntry)) {
            ptn_abort_out_of_memory();
        }
        new_entries = malloc(new_len * sizeof(PtnArrayEntry));
        if (new_entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }

    size_t out = 0;
    int64_t next_integer_key = 0;
    for (size_t i = 0; i < argc; i++) {
        new_entries[out].key = ptn_array_int_key(next_integer_key);
        new_entries[out].value = ptn_value_clone_deref(values[i]);
        new_entries[out].by_ref_argument_eligible = 0;
        out++;
        if (next_integer_key < INT64_MAX) {
            next_integer_key++;
        }
    }

    for (size_t i = 0; i < array->len; i++) {
        if (array->entries[i].key.type == PTN_ARRAY_KEY_INT) {
            new_entries[out].key = ptn_array_int_key(next_integer_key);
            ptn_array_key_free(array->entries[i].key);
            if (next_integer_key < INT64_MAX) {
                next_integer_key++;
            }
        } else {
            new_entries[out].key = array->entries[i].key;
        }
        new_entries[out].value = array->entries[i].value;
        new_entries[out].by_ref_argument_eligible =
            array->entries[i].by_ref_argument_eligible;
        out++;
    }

    free(array->entries);
    array->entries = new_entries;
    array->len = new_len;
    array->capacity = new_len;
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    return (int64_t)array->len;
}

typedef struct {
    uint32_t state[624];
    uint32_t count;
    int mode;
    int seeded;
} PtnMt19937State;

#define PTN_MT_RAND_MT19937 0
#define PTN_MT_RAND_PHP 1
#define PTN_MT19937_N 624
#define PTN_MT19937_M 397

static PtnMt19937State ptn_global_mt19937_state = { { 0 }, PTN_MT19937_N, PTN_MT_RAND_MT19937, 0 };

static uint32_t ptn_mt19937_twist(uint32_t m, uint32_t u, uint32_t v) {
    uint32_t mix = (u & 0x80000000U) | (v & 0x7fffffffU);
    return m ^ (mix >> 1) ^ ((uint32_t)(-(int32_t)(v & 1U)) & 0x9908b0dfU);
}

static uint32_t ptn_mt19937_twist_php(uint32_t m, uint32_t u, uint32_t v) {
    uint32_t mix = (u & 0x80000000U) | (v & 0x7fffffffU);
    return m ^ (mix >> 1) ^ ((uint32_t)(-(int32_t)(u & 1U)) & 0x9908b0dfU);
}

static void ptn_mt19937_reload(PtnMt19937State *state) {
    uint32_t *p = state->state;
    if (state->mode == PTN_MT_RAND_MT19937) {
        for (uint32_t i = PTN_MT19937_N - PTN_MT19937_M; i--; ++p) {
            *p = ptn_mt19937_twist(p[PTN_MT19937_M], p[0], p[1]);
        }
        for (uint32_t i = PTN_MT19937_M; --i; ++p) {
            *p = ptn_mt19937_twist(p[PTN_MT19937_M - PTN_MT19937_N], p[0], p[1]);
        }
        *p = ptn_mt19937_twist(p[PTN_MT19937_M - PTN_MT19937_N], p[0], state->state[0]);
    } else {
        for (uint32_t i = PTN_MT19937_N - PTN_MT19937_M; i--; ++p) {
            *p = ptn_mt19937_twist_php(p[PTN_MT19937_M], p[0], p[1]);
        }
        for (uint32_t i = PTN_MT19937_M; --i; ++p) {
            *p = ptn_mt19937_twist_php(p[PTN_MT19937_M - PTN_MT19937_N], p[0], p[1]);
        }
        *p = ptn_mt19937_twist_php(p[PTN_MT19937_M - PTN_MT19937_N], p[0], state->state[0]);
    }
    state->count = 0;
}

static PTN_UNUSED void ptn_mt19937_seed32(PtnMt19937State *state, uint32_t seed, int mode) {
    state->mode = mode == PTN_MT_RAND_PHP ? PTN_MT_RAND_PHP : PTN_MT_RAND_MT19937;
    state->state[0] = seed;
    for (uint32_t i = 1; i < PTN_MT19937_N; i++) {
        uint32_t prev = state->state[i - 1];
        state->state[i] = (1812433253U * (prev ^ (prev >> 30)) + i) & 0xffffffffU;
    }
    state->count = PTN_MT19937_N;
    state->seeded = 1;
    ptn_mt19937_reload(state);
}

static void ptn_mt19937_seed_default(PtnMt19937State *state) {
    uint64_t seed = (uint64_t)time(NULL) ^ ((uint64_t)(uintptr_t)state << 1);
#if defined(_WIN32)
    seed ^= (uint64_t)_getpid();
#else
    seed ^= (uint64_t)getpid();
#endif
    ptn_mt19937_seed32(state, (uint32_t)seed, PTN_MT_RAND_MT19937);
}

static uint32_t ptn_mt19937_generate(PtnMt19937State *state) {
    if (!state->seeded) {
        ptn_mt19937_seed_default(state);
    }
    if (state->count >= PTN_MT19937_N) {
        ptn_mt19937_reload(state);
    }

    uint32_t value = state->state[state->count++];
    value ^= value >> 11;
    value ^= (value << 7) & 0x9d2c5680U;
    value ^= (value << 15) & 0xefc60000U;
    return value ^ (value >> 18);
}

static uint32_t ptn_mt19937_range32(PtnMt19937State *state, uint32_t upper_inclusive) {
    uint32_t result = ptn_mt19937_generate(state);
    if (upper_inclusive == UINT32_MAX) {
        return result;
    }

    uint32_t range = upper_inclusive + 1U;
    if ((range & (range - 1U)) == 0) {
        return result & (range - 1U);
    }

    uint32_t limit = UINT32_MAX - (UINT32_MAX % range) - 1U;
    for (uint32_t attempts = 0; result > limit && attempts <= 50; attempts++) {
        result = ptn_mt19937_generate(state);
    }
    return result % range;
}

static uint64_t ptn_mt19937_u64(PtnMt19937State *state) {
    uint64_t low = (uint64_t)ptn_mt19937_generate(state);
    uint64_t high = (uint64_t)ptn_mt19937_generate(state);
    return low | (high << 32);
}

static uint64_t ptn_mt19937_range64(PtnMt19937State *state, uint64_t upper_inclusive) {
    uint64_t result = ptn_mt19937_u64(state);
    if (upper_inclusive == UINT64_MAX) {
        return result;
    }

    uint64_t range = upper_inclusive + 1ULL;
    if ((range & (range - 1ULL)) == 0) {
        return result & (range - 1ULL);
    }

    uint64_t limit = UINT64_MAX - (UINT64_MAX % range) - 1ULL;
    for (uint32_t attempts = 0; result > limit && attempts <= 50; attempts++) {
        result = ptn_mt19937_u64(state);
    }
    return result % range;
}

static PTN_UNUSED uint64_t ptn_mt19937_range_php_legacy(
    PtnMt19937State *state,
    uint64_t upper_inclusive
) {
    uint64_t result = (uint64_t)(ptn_mt19937_generate(state) >> 1);
    if (upper_inclusive == 2147483647ULL) {
        return result;
    }
    __uint128_t range = (__uint128_t)upper_inclusive + 1U;
    return (uint64_t)(((__uint128_t)result * range) / 2147483648ULL);
}

static PTN_UNUSED size_t ptn_mt19937_bounded_index(PtnMt19937State *state, size_t upper_inclusive) {
    if (upper_inclusive <= (size_t)UINT32_MAX) {
        return (size_t)ptn_mt19937_range32(state, (uint32_t)upper_inclusive);
    }
    return (size_t)ptn_mt19937_range64(state, (uint64_t)upper_inclusive);
}

static PTN_UNUSED void ptn_random_seed(uint64_t seed, int mode) {
    ptn_mt19937_seed32(&ptn_global_mt19937_state, (uint32_t)seed, mode);
}

static PTN_UNUSED size_t ptn_random_bounded_index(size_t upper_inclusive) {
    return ptn_mt19937_bounded_index(&ptn_global_mt19937_state, upper_inclusive);
}

static int ptn_array_key_compare_ascending(PtnArrayKey left, PtnArrayKey right) {
    if (left.type == PTN_ARRAY_KEY_INT && right.type == PTN_ARRAY_KEY_INT) {
        if (left.as.integer < right.as.integer) {
            return -1;
        }
        if (left.as.integer > right.as.integer) {
            return 1;
        }
        return 0;
    }
    if (left.type == PTN_ARRAY_KEY_STRING && right.type == PTN_ARRAY_KEY_STRING) {
        PtnString left_string = { (unsigned char *)left.as.string, left.string_len, 0 };
        PtnString right_string = { (unsigned char *)right.as.string, right.string_len, 0 };
        int compared = ptn_compare_strings_loose(left_string, right_string);
        if (compared == PTN_COMPARE_LESS) {
            return -1;
        }
        if (compared == PTN_COMPARE_GREATER) {
            return 1;
        }
        return 0;
    }
    PtnValue left_value = ptn_array_key_value(left);
    PtnValue right_value = ptn_array_key_value(right);
    int compared = ptn_compare_order(NULL, left_value, right_value, 0);
    ptn_value_destroy(&left_value);
    ptn_value_destroy(&right_value);
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_GREATER) {
        return 1;
    }
    return compared == PTN_COMPARE_UNORDERED ? 1 : 0;
}

static PTN_UNUSED void ptn_array_ksort_entries(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_key_compare_ascending(array->entries[j - 1].key, moving.key) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static int ptn_array_key_compare_descending(PtnArrayKey left, PtnArrayKey right) {
    return -ptn_array_key_compare_ascending(left, right);
}

static PTN_UNUSED void ptn_array_krsort_entries(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_key_compare_descending(array->entries[j - 1].key, moving.key) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static int ptn_array_value_compare_ascending_with_context(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    PtnTryFrame compare_frame;
    int guard_exceptions = runtime != NULL && runtime->exceptions != NULL;
    if (guard_exceptions) {
        ptn_try_frame_push(runtime, &compare_frame);
        if (setjmp(compare_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &compare_frame);
            PtnException *exception = runtime->exceptions->active_exception;
            if (
                exception != NULL &&
                ptn_ascii_case_equal(exception->class_name, "Error") &&
                strcmp(exception->message, "Nesting level too deep - recursive dependency?") == 0
            ) {
                ptn_clear_exception(runtime);
                return 0;
            }
            ptn_rethrow_exception(runtime);
            return 0;
        }
    }
    int compared = ptn_compare_order(runtime, left, right, line);
    if (guard_exceptions) {
        ptn_try_frame_pop(runtime, &compare_frame);
    }
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_GREATER) {
        return 1;
    }
    return compared == PTN_COMPARE_UNORDERED ? 1 : 0;
}

static int ptn_array_value_compare_ascending(PtnValue left, PtnValue right) {
    return ptn_array_value_compare_ascending_with_context(NULL, left, right, 0);
}

static int ptn_array_value_compare_numeric(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        if (left_number.integer < right_number.integer) {
            return -1;
        }
        if (left_number.integer > right_number.integer) {
            return 1;
        }
        return 0;
    }
    if (left_number.floating < right_number.floating) {
        return -1;
    }
    if (left_number.floating > right_number.floating) {
        return 1;
    }
    return 0;
}

static PtnStringOperand ptn_array_sort_string_operand(PtnRuntime *runtime, PtnValue value, size_t line) {
    PtnValue deref = ptn_value_deref(value);
    if (runtime != NULL && deref.type == PTN_ARRAY) {
        ptn_emit_spaced_warning(&runtime->diagnostics, "Array to string conversion", line);
    }
    return runtime != NULL
        ? ptn_value_to_string_operand_with_runtime(runtime, value, line)
        : ptn_value_to_string_operand(value);
}

static int ptn_array_value_compare_string(PtnValue left, PtnValue right, PtnRuntime *runtime, size_t line) {
    PtnStringOperand left_string = ptn_array_sort_string_operand(runtime, left, line);
    PtnStringOperand right_string = ptn_array_sort_string_operand(runtime, right, line);
    int compared = ptn_compare_string_bytes(
        (const unsigned char *)left_string.data,
        left_string.len,
        (const unsigned char *)right_string.data,
        right_string.len
    );
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_GREATER) {
        return 1;
    }
    return 0;
}

static int ptn_array_value_compare_string_case(PtnValue left, PtnValue right, PtnRuntime *runtime, size_t line) {
    PtnStringOperand left_string = ptn_array_sort_string_operand(runtime, left, line);
    PtnStringOperand right_string = ptn_array_sort_string_operand(runtime, right, line);
    const unsigned char *left_bytes = (const unsigned char *)left_string.data;
    const unsigned char *right_bytes = (const unsigned char *)right_string.data;
    size_t shared_len = left_string.len < right_string.len ? left_string.len : right_string.len;
    int compared = 0;
    for (size_t i = 0; i < shared_len; i++) {
        unsigned char left_byte = left_bytes[i];
        unsigned char right_byte = right_bytes[i];
        if (left_byte >= (unsigned char)'A' && left_byte <= (unsigned char)'Z') {
            left_byte = (unsigned char)(left_byte + ((unsigned char)'a' - (unsigned char)'A'));
        }
        if (right_byte >= (unsigned char)'A' && right_byte <= (unsigned char)'Z') {
            right_byte = (unsigned char)(right_byte + ((unsigned char)'a' - (unsigned char)'A'));
        }
        if (left_byte < right_byte) {
            compared = -1;
            break;
        }
        if (left_byte > right_byte) {
            compared = 1;
            break;
        }
    }
    if (compared == 0) {
        if (left_string.len < right_string.len) {
            compared = -1;
        } else if (left_string.len > right_string.len) {
            compared = 1;
        }
    }
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return compared;
}

static int ptn_ascii_is_natural_digit(unsigned char byte) {
    return byte >= (unsigned char)'0' && byte <= (unsigned char)'9';
}

static int ptn_ascii_is_natural_space(unsigned char byte) {
    return byte == (unsigned char)' ' ||
        byte == (unsigned char)'\t' ||
        byte == (unsigned char)'\n' ||
        byte == (unsigned char)'\r' ||
        byte == (unsigned char)'\v' ||
        byte == (unsigned char)'\f';
}

static unsigned char ptn_ascii_natural_case_fold(unsigned char byte) {
    if (byte >= (unsigned char)'A' && byte <= (unsigned char)'Z') {
        return (unsigned char)(byte + ((unsigned char)'a' - (unsigned char)'A'));
    }
    return byte;
}

static int ptn_compare_natural_digit_run_left(
    const unsigned char *left,
    size_t left_len,
    size_t left_offset,
    const unsigned char *right,
    size_t right_len,
    size_t right_offset
) {
    while (1) {
        int left_digit = left_offset < left_len && ptn_ascii_is_natural_digit(left[left_offset]);
        int right_digit = right_offset < right_len && ptn_ascii_is_natural_digit(right[right_offset]);
        if (!left_digit && !right_digit) {
            return 0;
        }
        if (!left_digit) {
            return -1;
        }
        if (!right_digit) {
            return 1;
        }
        if (left[left_offset] < right[right_offset]) {
            return -1;
        }
        if (left[left_offset] > right[right_offset]) {
            return 1;
        }
        left_offset++;
        right_offset++;
    }
}

static int ptn_compare_natural_digit_run_right(
    const unsigned char *left,
    size_t left_len,
    size_t left_offset,
    const unsigned char *right,
    size_t right_len,
    size_t right_offset
) {
    int bias = 0;
    while (1) {
        int left_digit = left_offset < left_len && ptn_ascii_is_natural_digit(left[left_offset]);
        int right_digit = right_offset < right_len && ptn_ascii_is_natural_digit(right[right_offset]);
        if (!left_digit && !right_digit) {
            return bias;
        }
        if (!left_digit) {
            return -1;
        }
        if (!right_digit) {
            return 1;
        }
        if (left[left_offset] < right[right_offset]) {
            if (bias == 0) {
                bias = -1;
            }
        } else if (left[left_offset] > right[right_offset]) {
            if (bias == 0) {
                bias = 1;
            }
        }
        left_offset++;
        right_offset++;
    }
}

static int ptn_compare_natural_string_operands(PtnStringOperand left_operand, PtnStringOperand right_operand, int case_insensitive) {
    const unsigned char *left = (const unsigned char *)left_operand.data;
    const unsigned char *right = (const unsigned char *)right_operand.data;
    size_t left_offset = 0;
    size_t right_offset = 0;

    while (
        left_offset < left_operand.len &&
        left[left_offset] == (unsigned char)'0' &&
        left_offset + 1 < left_operand.len &&
        ptn_ascii_is_natural_digit(left[left_offset + 1])
    ) {
        left_offset++;
    }
    while (
        right_offset < right_operand.len &&
        right[right_offset] == (unsigned char)'0' &&
        right_offset + 1 < right_operand.len &&
        ptn_ascii_is_natural_digit(right[right_offset + 1])
    ) {
        right_offset++;
    }

    while (1) {
        while (left_offset < left_operand.len && ptn_ascii_is_natural_space(left[left_offset])) {
            left_offset++;
        }
        while (right_offset < right_operand.len && ptn_ascii_is_natural_space(right[right_offset])) {
            right_offset++;
        }

        int left_digit = left_offset < left_operand.len && ptn_ascii_is_natural_digit(left[left_offset]);
        int right_digit = right_offset < right_operand.len && ptn_ascii_is_natural_digit(right[right_offset]);
        if (left_digit && right_digit) {
            int compared = left[left_offset] == (unsigned char)'0' || right[right_offset] == (unsigned char)'0'
                ? ptn_compare_natural_digit_run_left(
                    left,
                    left_operand.len,
                    left_offset,
                    right,
                    right_operand.len,
                    right_offset
                )
                : ptn_compare_natural_digit_run_right(
                    left,
                    left_operand.len,
                    left_offset,
                    right,
                    right_operand.len,
                    right_offset
                );
            if (compared != 0) {
                return compared;
            }
        }

        if (left_offset >= left_operand.len && right_offset >= right_operand.len) {
            return 0;
        }
        if (left_offset >= left_operand.len) {
            return -1;
        }
        if (right_offset >= right_operand.len) {
            return 1;
        }
        unsigned char left_byte = left[left_offset];
        unsigned char right_byte = right[right_offset];
        if (case_insensitive) {
            left_byte = ptn_ascii_natural_case_fold(left_byte);
            right_byte = ptn_ascii_natural_case_fold(right_byte);
        }
        if (left_byte < right_byte) {
            return -1;
        }
        if (left_byte > right_byte) {
            return 1;
        }
        left_offset++;
        right_offset++;
    }
}

static int ptn_array_value_compare_natural(PtnValue left, PtnValue right, PtnRuntime *runtime, size_t line) {
    PtnStringOperand left_string = ptn_array_sort_string_operand(runtime, left, line);
    PtnStringOperand right_string = ptn_array_sort_string_operand(runtime, right, line);
    int compared = ptn_compare_natural_string_operands(left_string, right_string, 0);
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return compared;
}

static int ptn_array_value_compare_natural_case(PtnValue left, PtnValue right, PtnRuntime *runtime, size_t line) {
    PtnStringOperand left_string = ptn_array_sort_string_operand(runtime, left, line);
    PtnStringOperand right_string = ptn_array_sort_string_operand(runtime, right, line);
    int compared = ptn_compare_natural_string_operands(left_string, right_string, 1);
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return compared;
}

static int ptn_array_sort_flags_case_insensitive(int64_t flags) {
    return flags == (PTN_SORT_STRING | PTN_SORT_FLAG_CASE) ||
        flags == (PTN_SORT_NATURAL | PTN_SORT_FLAG_CASE);
}

static int64_t ptn_array_sort_flags_base(int64_t flags) {
    return ptn_array_sort_flags_case_insensitive(flags)
        ? (flags & ~PTN_SORT_FLAG_CASE)
        : flags;
}

static int ptn_array_value_compare_by_sort_flags(
    PtnValue left,
    PtnValue right,
    int64_t flags,
    PtnRuntime *runtime,
    size_t line
) {
    int case_insensitive = ptn_array_sort_flags_case_insensitive(flags);
    switch (ptn_array_sort_flags_base(flags)) {
        case PTN_SORT_NUMERIC:
            return ptn_array_value_compare_numeric(left, right);
        case PTN_SORT_STRING:
        case PTN_SORT_LOCALE_STRING:
            return case_insensitive
                ? ptn_array_value_compare_string_case(left, right, runtime, line)
                : ptn_array_value_compare_string(left, right, runtime, line);
        case PTN_SORT_NATURAL:
            return case_insensitive
                ? ptn_array_value_compare_natural_case(left, right, runtime, line)
                : ptn_array_value_compare_natural(left, right, runtime, line);
        case PTN_SORT_REGULAR:
        default:
            return ptn_array_value_compare_ascending_with_context(runtime, left, right, line);
    }
}

static int ptn_array_key_compare_by_sort_flags(PtnArrayKey left, PtnArrayKey right, int64_t flags) {
    if (ptn_array_sort_flags_base(flags) == PTN_SORT_REGULAR) {
        return ptn_array_key_compare_ascending(left, right);
    }
    PtnValue left_value = ptn_array_key_value(left);
    PtnValue right_value = ptn_array_key_value(right);
    int compared = ptn_array_value_compare_by_sort_flags(left_value, right_value, flags, NULL, 0);
    ptn_value_destroy(&left_value);
    ptn_value_destroy(&right_value);
    return compared;
}

static void ptn_array_reindex_after_sort(PtnArray *array) {
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_key_free(array->entries[i].key);
        array->entries[i].key = ptn_array_int_key((int64_t)i);
    }
    ptn_array_recompute_next_auto_key(array);
}

typedef struct {
    PtnArrayEntry entry;
    size_t original_index;
} PtnArraySortItem;

typedef struct {
    int compare_keys;
    int descending;
    int64_t flags;
    PtnRuntime *runtime;
    size_t line;
} PtnArraySortContext;

static int ptn_array_sort_item_compare(
    const PtnArraySortItem *left,
    const PtnArraySortItem *right,
    const PtnArraySortContext *context
) {
    int compared = context->compare_keys
        ? ptn_array_key_compare_by_sort_flags(left->entry.key, right->entry.key, context->flags)
        : ptn_array_value_compare_by_sort_flags(
            left->entry.value,
            right->entry.value,
            context->flags,
            context->runtime,
            context->line
        );
    if (context->descending) {
        compared = -compared;
    }
    if (compared != 0) {
        return compared;
    }
    if (left->original_index < right->original_index) {
        return -1;
    }
    if (left->original_index > right->original_index) {
        return 1;
    }
    return 0;
}

static void ptn_array_sort_item_swap(PtnArraySortItem *left, PtnArraySortItem *right) {
    PtnArraySortItem temporary = *left;
    *left = *right;
    *right = temporary;
}

static PtnArrayEntry ptn_array_sort_entry_clone(PtnArrayEntry entry) {
    PtnArrayEntry cloned;
    cloned.key = ptn_array_key_clone(entry.key);
    cloned.value = ptn_value_clone(entry.value);
    cloned.by_ref_argument_eligible = entry.by_ref_argument_eligible;
    return cloned;
}

static void ptn_array_sort_entry_destroy(PtnArrayEntry *entry) {
    ptn_array_key_free(entry->key);
    ptn_value_destroy(&entry->value);
}

static void ptn_array_destroy_entries(PtnArrayEntry *entries, size_t len) {
    for (size_t i = 0; i < len; i++) {
        ptn_array_sort_entry_destroy(&entries[i]);
    }
}

static void ptn_array_sort_items_destroy_entries(PtnArraySortItem *items, size_t len) {
    for (size_t i = 0; i < len; i++) {
        ptn_array_sort_entry_destroy(&items[i].entry);
    }
}

static void ptn_array_zend_sort_2(
    PtnArraySortItem *a,
    PtnArraySortItem *b,
    const PtnArraySortContext *context
) {
    if (ptn_array_sort_item_compare(a, b, context) > 0) {
        ptn_array_sort_item_swap(a, b);
    }
}

static void ptn_array_zend_sort_3(
    PtnArraySortItem *a,
    PtnArraySortItem *b,
    PtnArraySortItem *c,
    const PtnArraySortContext *context
) {
    if (!(ptn_array_sort_item_compare(a, b, context) > 0)) {
        if (!(ptn_array_sort_item_compare(b, c, context) > 0)) {
            return;
        }
        ptn_array_sort_item_swap(b, c);
        if (ptn_array_sort_item_compare(a, b, context) > 0) {
            ptn_array_sort_item_swap(a, b);
        }
        return;
    }
    if (!(ptn_array_sort_item_compare(c, b, context) > 0)) {
        ptn_array_sort_item_swap(a, c);
        return;
    }
    ptn_array_sort_item_swap(a, b);
    if (ptn_array_sort_item_compare(b, c, context) > 0) {
        ptn_array_sort_item_swap(b, c);
    }
}

static void ptn_array_zend_sort_4(
    PtnArraySortItem *a,
    PtnArraySortItem *b,
    PtnArraySortItem *c,
    PtnArraySortItem *d,
    const PtnArraySortContext *context
) {
    ptn_array_zend_sort_3(a, b, c, context);
    if (ptn_array_sort_item_compare(c, d, context) > 0) {
        ptn_array_sort_item_swap(c, d);
        if (ptn_array_sort_item_compare(b, c, context) > 0) {
            ptn_array_sort_item_swap(b, c);
            if (ptn_array_sort_item_compare(a, b, context) > 0) {
                ptn_array_sort_item_swap(a, b);
            }
        }
    }
}

static void ptn_array_zend_sort_5(
    PtnArraySortItem *a,
    PtnArraySortItem *b,
    PtnArraySortItem *c,
    PtnArraySortItem *d,
    PtnArraySortItem *e,
    const PtnArraySortContext *context
) {
    ptn_array_zend_sort_4(a, b, c, d, context);
    if (ptn_array_sort_item_compare(d, e, context) > 0) {
        ptn_array_sort_item_swap(d, e);
        if (ptn_array_sort_item_compare(c, d, context) > 0) {
            ptn_array_sort_item_swap(c, d);
            if (ptn_array_sort_item_compare(b, c, context) > 0) {
                ptn_array_sort_item_swap(b, c);
                if (ptn_array_sort_item_compare(a, b, context) > 0) {
                    ptn_array_sort_item_swap(a, b);
                }
            }
        }
    }
}

static void ptn_array_zend_insert_sort(
    PtnArraySortItem *base,
    size_t len,
    const PtnArraySortContext *context
) {
    switch (len) {
        case 0:
        case 1:
            return;
        case 2:
            ptn_array_zend_sort_2(base, base + 1, context);
            return;
        case 3:
            ptn_array_zend_sort_3(base, base + 1, base + 2, context);
            return;
        case 4:
            ptn_array_zend_sort_4(base, base + 1, base + 2, base + 3, context);
            return;
        case 5:
            ptn_array_zend_sort_5(base, base + 1, base + 2, base + 3, base + 4, context);
            return;
        default:
            break;
    }

    PtnArraySortItem *start = base;
    PtnArraySortItem *end = start + len;
    PtnArraySortItem *sentry = start + 6;
    for (PtnArraySortItem *i = start + 1; i < sentry; i++) {
        PtnArraySortItem *j = i - 1;
        if (!(ptn_array_sort_item_compare(j, i, context) > 0)) {
            continue;
        }
        while (j != start) {
            j--;
            if (!(ptn_array_sort_item_compare(j, i, context) > 0)) {
                j++;
                break;
            }
        }
        for (PtnArraySortItem *k = i; k > j; k--) {
            ptn_array_sort_item_swap(k, k - 1);
        }
    }

    for (PtnArraySortItem *i = sentry; i < end; i++) {
        PtnArraySortItem *j = i - 1;
        if (!(ptn_array_sort_item_compare(j, i, context) > 0)) {
            continue;
        }
        do {
            j -= 2;
            if (!(ptn_array_sort_item_compare(j, i, context) > 0)) {
                j++;
                if (!(ptn_array_sort_item_compare(j, i, context) > 0)) {
                    j++;
                }
                break;
            }
            if (j == start) {
                break;
            }
            if (j == start + 1) {
                j--;
                if (ptn_array_sort_item_compare(i, j, context) > 0) {
                    j++;
                }
                break;
            }
        } while (1);
        for (PtnArraySortItem *k = i; k > j; k--) {
            ptn_array_sort_item_swap(k, k - 1);
        }
    }
}

static void ptn_array_zend_sort_items(
    PtnArraySortItem *base,
    size_t len,
    const PtnArraySortContext *context
) {
    while (1) {
        if (len <= 16) {
            ptn_array_zend_insert_sort(base, len, context);
            return;
        }

        PtnArraySortItem *start = base;
        PtnArraySortItem *end = start + len;
        size_t offset = len >> 1;
        PtnArraySortItem *pivot = start + offset;

        if (len >> 10) {
            size_t delta = offset >> 1;
            ptn_array_zend_sort_5(start, start + delta, pivot, pivot + delta, end - 1, context);
        } else {
            ptn_array_zend_sort_3(start, pivot, end - 1, context);
        }

        ptn_array_sort_item_swap(start + 1, pivot);
        pivot = start + 1;
        PtnArraySortItem *i = pivot + 1;
        PtnArraySortItem *j = end - 1;
        while (1) {
            while (ptn_array_sort_item_compare(pivot, i, context) > 0) {
                i++;
                if (i == j) {
                    goto done;
                }
            }
            j--;
            if (j == i) {
                goto done;
            }
            while (ptn_array_sort_item_compare(j, pivot, context) > 0) {
                j--;
                if (j == i) {
                    goto done;
                }
            }
            ptn_array_sort_item_swap(i, j);
            i++;
            if (i == j) {
                goto done;
            }
        }

done:
        ptn_array_sort_item_swap(pivot, i - 1);
        if ((size_t)((i - 1) - start) < (size_t)(end - i)) {
            ptn_array_zend_sort_items(start, (size_t)(i - start) - 1, context);
            base = i;
            len = (size_t)(end - i);
        } else {
            ptn_array_zend_sort_items(i, (size_t)(end - i), context);
            len = (size_t)(i - start) - 1;
        }
    }
}

static void ptn_array_sort_entries_by_flags_with_context(
    PtnRuntime *runtime,
    PtnArray *array,
    int compare_keys,
    int descending,
    int reindex,
    int64_t flags,
    size_t line
) {
    if (array->len > 1) {
        size_t original_len = array->len;
        uint64_t mutation_epoch_before_compare = array->mutation_epoch;
        PtnArraySortItem *items = malloc(sizeof(PtnArraySortItem) * original_len);
        if (items == NULL) {
            ptn_abort_out_of_memory();
        }
        for (size_t i = 0; i < original_len; i++) {
            items[i].entry = ptn_array_sort_entry_clone(array->entries[i]);
            items[i].original_index = i;
        }
        PtnArraySortContext context = { compare_keys, descending, flags, runtime, line };
        ptn_array_zend_sort_items(items, original_len, &context);
        if (array->mutation_epoch != mutation_epoch_before_compare) {
            ptn_array_sort_items_destroy_entries(items, original_len);
            free(items);
            array->current_index = 0;
            ptn_array_rebuild_index(array);
            return;
        }
        ptn_array_destroy_entries(array->entries, original_len);
        for (size_t i = 0; i < original_len; i++) {
            array->entries[i] = items[i].entry;
        }
        free(items);
    }
    if (reindex) {
        ptn_array_reindex_after_sort(array);
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static void ptn_array_sort_entries_by_flags(PtnArray *array, int compare_keys, int descending, int reindex, int64_t flags) {
    ptn_array_sort_entries_by_flags_with_context(NULL, array, compare_keys, descending, reindex, flags, 0);
}

static PTN_UNUSED void ptn_array_sort_values_with_flags_context(
    PtnRuntime *runtime,
    PtnArray *array,
    int64_t flags,
    size_t line
) {
    ptn_array_sort_entries_by_flags_with_context(runtime, array, 0, 0, 1, flags, line);
}

static PTN_UNUSED void ptn_array_rsort_values_with_flags_context(
    PtnRuntime *runtime,
    PtnArray *array,
    int64_t flags,
    size_t line
) {
    ptn_array_sort_entries_by_flags_with_context(runtime, array, 0, 1, 1, flags, line);
}

static PTN_UNUSED void ptn_array_asort_values_with_flags_context(
    PtnRuntime *runtime,
    PtnArray *array,
    int64_t flags,
    size_t line
) {
    ptn_array_sort_entries_by_flags_with_context(runtime, array, 0, 0, 0, flags, line);
}

static PTN_UNUSED void ptn_array_arsort_values_with_flags_context(
    PtnRuntime *runtime,
    PtnArray *array,
    int64_t flags,
    size_t line
) {
    ptn_array_sort_entries_by_flags_with_context(runtime, array, 0, 1, 0, flags, line);
}

static PTN_UNUSED void ptn_array_sort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 0, 1, flags);
}

static PTN_UNUSED void ptn_array_rsort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 1, 1, flags);
}

static PTN_UNUSED void ptn_array_asort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 0, 0, flags);
}

static PTN_UNUSED void ptn_array_arsort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 1, 0, flags);
}

static PTN_UNUSED void ptn_array_ksort_entries_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 1, 0, 0, flags);
}

static PTN_UNUSED void ptn_array_krsort_entries_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 1, 1, 0, flags);
}

static PTN_UNUSED void ptn_array_sort_values(PtnArray *array) {
    ptn_array_sort_values_with_flags(array, PTN_SORT_REGULAR);
}

static PTN_UNUSED void ptn_array_rsort_values(PtnArray *array) {
    ptn_array_rsort_values_with_flags(array, PTN_SORT_REGULAR);
}

static PTN_UNUSED void ptn_array_asort_values(PtnArray *array) {
    ptn_array_asort_values_with_flags(array, PTN_SORT_REGULAR);
}

static PTN_UNUSED void ptn_array_arsort_values(PtnArray *array) {
    ptn_array_arsort_values_with_flags(array, PTN_SORT_REGULAR);
}

static PTN_UNUSED void ptn_array_natsort_values(PtnRuntime *runtime, PtnArray *array, size_t line) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_natural(array->entries[j - 1].value, moving.value, runtime, line) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_natcasesort_values(PtnRuntime *runtime, PtnArray *array, size_t line) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_natural_case(array->entries[j - 1].value, moving.value, runtime, line) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_shuffle_values_with_state(PtnArray *array, PtnMt19937State *state) {
    if (state == NULL) {
        state = &ptn_global_mt19937_state;
    }
    if (array->len > 1) {
        for (size_t i = array->len - 1; i > 0; i--) {
            size_t j = ptn_mt19937_bounded_index(state, i);
            PtnArrayEntry tmp = array->entries[i];
            array->entries[i] = array->entries[j];
            array->entries[j] = tmp;
        }
    }
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_key_free(array->entries[i].key);
        array->entries[i].key = ptn_array_int_key((int64_t)i);
    }
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_shuffle_values(PtnArray *array) {
    ptn_array_shuffle_values_with_state(array, &ptn_global_mt19937_state);
}

static PTN_UNUSED int64_t ptn_array_push_values(PtnRuntime *runtime, PtnArray *array, size_t argc, const PtnValue *values) {
    if (argc > SIZE_MAX - array->len) {
        ptn_abort_out_of_memory();
    }
    if (array->len + argc > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }

    for (size_t i = 0; i < argc; i++) {
        if (!ptn_array_append_key_available(runtime, array)) {
            return (int64_t)array->len;
        }
        PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
        ptn_array_set_entry(array, key, ptn_value_clone_deref(values[i]));
    }

    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    return (int64_t)array->len;
}
