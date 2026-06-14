    if (argc > 0) {
        PtnArrayKey max_key = ptn_array_int_key(INT64_MAX);
        int max_key_occupied = ptn_array_find_key(array, max_key) < array->len;
        ptn_array_key_free(max_key);
        if (max_key_occupied) {
            ptn_throw_exception(runtime, "Error", "Cannot add element to the array as the next element is already occupied");
            return (int64_t)array->len;
        }
    }
    for (size_t i = 0; i < argc; i++) {
        PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
        ptn_array_set_entry(array, key, ptn_value_clone(values[i]));
    }
    return (int64_t)array->len;
}

static PTN_UNUSED PtnValue ptn_array_reindexing_internal_value(PtnValue value) {
    if (value.type == PTN_REFERENCE && value.as.reference->refcount == 1) {
        return ptn_value_deref(value);
    }
    return value;
}

static PTN_UNUSED PtnValue ptn_array_union(PtnArray *left, PtnArray *right) {
    PtnValue union_value = ptn_array_from_literal_entries(0, NULL);
    PtnArray *union_array = union_value.as.array;

    for (size_t i = 0; i < left->len; i++) {
        ptn_array_set_entry(
            union_array,
            ptn_array_key_clone(left->entries[i].key),
            ptn_value_clone(left->entries[i].value)
        );
    }

    for (size_t i = 0; i < right->len; i++) {
        if (ptn_array_find_key(union_array, right->entries[i].key) < union_array->len) {
            continue;
        }
        ptn_array_set_entry(
            union_array,
            ptn_array_key_clone(right->entries[i].key),
            ptn_value_clone(right->entries[i].value)
        );
    }

    return union_value;
}

static PTN_UNUSED int ptn_compare_arrays_equal(PtnArray *left, PtnArray *right) {
    if (left->len != right->len) {
        return 0;
    }
    for (size_t i = 0; i < left->len; i++) {
        PtnArrayEntry *right_entry = ptn_array_entry_for_key(right, left->entries[i].key);
        if (right_entry == NULL || !ptn_compare_equal(left->entries[i].value, right_entry->value)) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED int ptn_compare_arrays_identical(PtnArray *left, PtnArray *right) {
    if (left->len != right->len) {
        return 0;
    }
    for (size_t i = 0; i < left->len; i++) {
        if (!ptn_array_keys_equal(left->entries[i].key, right->entries[i].key) ||
            !ptn_compare_identical(left->entries[i].value, right->entries[i].value)) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED int ptn_compare_objects_equal(PtnObject *left, PtnObject *right) {
    if (left == right) {
        return 1;
    }
    if (strcmp(left->class_name, right->class_name) != 0) {
        return 0;
    }
    return ptn_compare_arrays_equal(left->properties, right->properties);
}

static PTN_UNUSED int ptn_compare_arrays_order(PtnArray *left, PtnArray *right) {
    if (left->len < right->len) {
        return PTN_COMPARE_LESS;
    }
    if (left->len > right->len) {
        return PTN_COMPARE_GREATER;
    }
    for (size_t i = 0; i < left->len; i++) {
        PtnArrayEntry *right_entry = ptn_array_entry_for_key(right, left->entries[i].key);
        if (right_entry == NULL) {
            return PTN_COMPARE_UNORDERED;
        }
        int compared = ptn_compare_order(left->entries[i].value, right_entry->value);
        if (compared != PTN_COMPARE_EQUAL) {
            return compared;
        }
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_equal(PtnValue left, PtnValue right) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == right.type) {
        switch (left.type) {
            case PTN_NULL:
                return 1;
            case PTN_BOOL:
                return left.as.boolean == right.as.boolean;
            case PTN_INT:
                return left.as.integer == right.as.integer;
            case PTN_FLOAT:
                return ptn_compare_numbers(left.as.floating, right.as.floating) == PTN_COMPARE_EQUAL;
            case PTN_STRING:
                return ptn_compare_strings_loose(left.as.string, right.as.string) == PTN_COMPARE_EQUAL;
            case PTN_ARRAY:
                return ptn_compare_arrays_equal(left.as.array, right.as.array);
            case PTN_OBJECT:
                return ptn_compare_objects_equal(left.as.object, right.as.object);
            case PTN_CLOSURE:
                return left.as.closure == right.as.closure;
            case PTN_EXCEPTION:
                return left.as.exception == right.as.exception;
            case PTN_RESOURCE:
                return left.as.resource == right.as.resource;
            case PTN_REFERENCE:
                return 0;
        }
    }

    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_is_truthy(left) == ptn_is_truthy(right);
    }
    if (left.type == PTN_NULL || right.type == PTN_NULL) {
        if (left.type == PTN_NULL && right.type == PTN_NULL) {
            return 1;
        }
        PtnValue other = left.type == PTN_NULL ? right : left;
        switch (other.type) {
            case PTN_NULL:
                return 1;
            case PTN_BOOL:
                return ptn_is_truthy(other) == 0;
            case PTN_INT:
                return other.as.integer == 0;
            case PTN_FLOAT:
                return other.as.floating == 0.0;
            case PTN_STRING:
                return other.as.string.len == 0;
            case PTN_ARRAY:
                return other.as.array->len == 0;
            case PTN_OBJECT:
            case PTN_CLOSURE:
                return 0;
            case PTN_EXCEPTION:
                return 0;
            case PTN_RESOURCE:
                return 0;
            case PTN_REFERENCE:
                return 0;
        }
    }

    if (left.type == PTN_ARRAY || right.type == PTN_ARRAY) {
        if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
            return ptn_compare_arrays_equal(left.as.array, right.as.array);
        }
        return 0;
    }

    int compared = 0;
    if (ptn_compare_number_types(left, right, &compared)) {
        return compared == PTN_COMPARE_EQUAL;
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number) == PTN_COMPARE_EQUAL;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_identical(PtnValue left, PtnValue right) {
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
            return ptn_compare_value_strings(left.as.string, right.as.string) == PTN_COMPARE_EQUAL;
        case PTN_ARRAY:
            if (left.as.array == right.as.array) {
                return 1;
            }
            return ptn_compare_arrays_identical(left.as.array, right.as.array);
        case PTN_OBJECT:
            return left.as.object == right.as.object;
        case PTN_CLOSURE:
            return left.as.closure == right.as.closure;
        case PTN_EXCEPTION:
            return left.as.exception == right.as.exception;
        case PTN_RESOURCE:
            return left.as.resource == right.as.resource;
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_not_identical(PtnValue left, PtnValue right) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type != right.type) {
        return 1;
    }
    switch (left.type) {
        case PTN_NULL:
            return 0;
        case PTN_BOOL:
            return left.as.boolean != right.as.boolean;
        case PTN_INT:
            return left.as.integer != right.as.integer;
        case PTN_FLOAT:
            return left.as.floating != right.as.floating;
        case PTN_STRING:
            return ptn_compare_value_strings(left.as.string, right.as.string) != PTN_COMPARE_EQUAL;
        case PTN_ARRAY:
            if (left.as.array == right.as.array) {
                return 0;
            }
            return !ptn_compare_arrays_identical(left.as.array, right.as.array);
        case PTN_OBJECT:
            return left.as.object != right.as.object;
        case PTN_CLOSURE:
            return left.as.closure != right.as.closure;
        case PTN_EXCEPTION:
            return left.as.exception != right.as.exception;
        case PTN_RESOURCE:
            return left.as.resource != right.as.resource;
        case PTN_REFERENCE:
            return 1;
    }
    return 1;
}

static PTN_UNUSED int ptn_value_is_nan(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_FLOAT && isnan(value.as.floating);
}

static PTN_UNUSED int ptn_compare_order(PtnValue left, PtnValue right) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == right.type) {
        switch (left.type) {
            case PTN_NULL:
                return PTN_COMPARE_EQUAL;
            case PTN_BOOL:
                return ptn_compare_integers(left.as.boolean, right.as.boolean);
            case PTN_INT:
                return ptn_compare_integers(left.as.integer, right.as.integer);
            case PTN_FLOAT:
                return ptn_compare_numbers(left.as.floating, right.as.floating);
            case PTN_STRING:
                return ptn_compare_strings_loose(left.as.string, right.as.string);
            case PTN_ARRAY:
                return ptn_compare_arrays_order(left.as.array, right.as.array);
            case PTN_OBJECT:
                return left.as.object == right.as.object ? PTN_COMPARE_EQUAL : PTN_COMPARE_GREATER;
            case PTN_CLOSURE:
                return left.as.closure == right.as.closure ? PTN_COMPARE_EQUAL : PTN_COMPARE_GREATER;
            case PTN_EXCEPTION:
                return left.as.exception == right.as.exception ? PTN_COMPARE_EQUAL : PTN_COMPARE_GREATER;
            case PTN_RESOURCE:
                return ptn_compare_integers(left.as.resource->id, right.as.resource->id);
            case PTN_REFERENCE:
                return PTN_COMPARE_UNORDERED;
        }
    }

    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
    }
    if (left.type == PTN_NULL && right.type == PTN_NULL) {
        return 0;
    }
    if (left.type == PTN_NULL) {
        if (ptn_value_is_nan(right)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (ptn_is_number_type(right)) {
            double right_number = right.type == PTN_INT ? (double)right.as.integer : right.as.floating;
            return ptn_compare_numbers(0.0, right_number);
        }
        if (right.type == PTN_STRING) {
            return ptn_compare_string_bytes((const unsigned char *)"", 0, right.as.string.data, right.as.string.len);
        }
        if (right.type == PTN_ARRAY) {
            return ptn_compare_numbers(0.0, (double)ptn_is_truthy(right));
        }
    }
    if (right.type == PTN_NULL) {
        if (ptn_value_is_nan(left)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (ptn_is_number_type(left)) {
            double left_number = left.type == PTN_INT ? (double)left.as.integer : left.as.floating;
            return ptn_compare_numbers(left_number, 0.0);
        }
        if (left.type == PTN_STRING) {
            return ptn_compare_string_bytes(left.as.string.data, left.as.string.len, (const unsigned char *)"", 0);
        }
        if (left.type == PTN_ARRAY) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), 0.0);
        }
    }

    if (left.type == PTN_ARRAY || right.type == PTN_ARRAY) {
        if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
            return ptn_compare_arrays_order(left.as.array, right.as.array);
        }
        return left.type == PTN_ARRAY ? PTN_COMPARE_GREATER : PTN_COMPARE_LESS;
    }

    int compared = 0;
    if (ptn_compare_number_types(left, right, &compared)) {
        return compared;
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number);
    }
    if (ptn_is_number_type(left) && right.type == PTN_STRING) {
        if (ptn_value_is_nan(left)) {
            return PTN_COMPARE_UNORDERED;
        }
        return ptn_compare_number_and_string(left, right.as.string, 1);
    }
    if (left.type == PTN_STRING && ptn_is_number_type(right)) {
        if (ptn_value_is_nan(right)) {
            return PTN_COMPARE_UNORDERED;
        }
        return ptn_compare_number_and_string(right, left.as.string, 0);
    }
    return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
}

static PTN_UNUSED int ptn_compare_less(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) == PTN_COMPARE_LESS;
}

static PTN_UNUSED int ptn_compare_less_equal(PtnValue left, PtnValue right) {
    int compared = ptn_compare_order(left, right);
    return compared == PTN_COMPARE_LESS || compared == PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_greater(PtnValue left, PtnValue right) {
    return ptn_compare_order(left, right) == PTN_COMPARE_GREATER;
}

static PTN_UNUSED int ptn_compare_greater_equal(PtnValue left, PtnValue right) {
    int compared = ptn_compare_order(left, right);
    return compared == PTN_COMPARE_GREATER || compared == PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_spaceship(PtnValue left, PtnValue right) {
    int compared = ptn_compare_order(left, right);
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_EQUAL) {
        return 0;
    }
    return 1;
}

static PTN_UNUSED void ptn_emit_arithmetic_non_numeric_value_warning(PtnRuntime *runtime, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Warning: A non-numeric value encountered in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED int ptn_arithmetic_string_to_number(
    PtnString string,
    PtnNumber *number,
    int *has_trailing_non_numeric_data
) {
    const char *data = (const char *)string.data;
    const char *limit = data + string.len;
    const char *start = data;
    while (start < limit && isspace((unsigned char)*start)) {
        start++;
    }
    if (start >= limit) {
        return 0;
    }

    char *int_end = NULL;
    errno = 0;
    long long integer = strtoll(start, &int_end, 10);
    int int_errno = errno;

    char *float_end = NULL;
    errno = 0;
    double floating = strtod(start, &float_end);
    if (float_end == start) {
        return 0;
    }

    const char *end = float_end;
    while (end < limit && isspace((unsigned char)*end)) {
        end++;
    }
    *has_trailing_non_numeric_data = end < limit;

    if (int_end == float_end && int_errno != ERANGE && !ptn_contains_float_marker(start, int_end)) {
        *number = ptn_number_int((int64_t)integer);
    } else {
        *number = ptn_number_float(floating);
    }
    return 1;
}

static PTN_UNUSED int ptn_arithmetic_number(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnNumber *number
) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            *number = ptn_number_int(0);
            return 1;
        case PTN_BOOL:
            *number = ptn_number_int(value.as.boolean ? 1 : 0);
            return 1;
        case PTN_INT:
            *number = ptn_number_int(value.as.integer);
            return 1;
        case PTN_FLOAT:
            *number = ptn_number_float(value.as.floating);
            return 1;
        case PTN_STRING: {
            int has_trailing_non_numeric_data = 0;
            if (!ptn_arithmetic_string_to_number(value.as.string, number, &has_trailing_non_numeric_data)) {
                return 0;
            }
            if (has_trailing_non_numeric_data) {
                ptn_emit_arithmetic_non_numeric_value_warning(runtime, line);
            }
            return 1;
        }
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED const char *ptn_arithmetic_operand_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_OBJECT:
            return value.as.object->class_name;
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            return ptn_offset_container_type_name(value);
    }
    return ptn_offset_container_type_name(value);
}

static PTN_UNUSED void ptn_throw_unsupported_operand_types(
    PtnRuntime *runtime,
    PtnValue left,
    const char *operator,
    PtnValue right,
    size_t line
) {
    const char *left_type = ptn_arithmetic_operand_type_name(left);
    const char *right_type = ptn_arithmetic_operand_type_name(right);
    int needed = snprintf(
        NULL,
        0,
        "Unsupported operand types: %s %s %s",
        left_type,
        operator,
        right_type
    );
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(
        message,
        (size_t)needed + 1,
        "Unsupported operand types: %s %s %s",
        left_type,
        operator,
        right_type
    );
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
    free(message);
}

static PTN_UNUSED void ptn_arithmetic_operands(
    PtnRuntime *runtime,
    PtnValue left,
    const char *operator,
    PtnValue right,
    size_t line,
    PtnNumber *left_number,
    PtnNumber *right_number
) {
    if (!ptn_arithmetic_number(runtime, left, line, left_number)) {
        ptn_throw_unsupported_operand_types(runtime, left, operator, right, line);
    }
    if (!ptn_arithmetic_number(runtime, right, line, right_number)) {
        ptn_throw_unsupported_operand_types(runtime, left, operator, right, line);
    }
}

static PTN_UNUSED int ptn_fast_numeric_pair(PtnValue left, PtnValue right, double *left_number, double *right_number) {
    return ptn_fast_scalar_double(left, left_number) && ptn_fast_scalar_double(right, right_number);
}

static PTN_UNUSED PtnValue ptn_add_integers(int64_t left, int64_t right) {
    if ((right > 0 && left > INT64_MAX - right) ||
        (right < 0 && left < INT64_MIN - right)) {
        return ptn_float((double)left + (double)right);
    }
    return ptn_int(left + right);
}

static PTN_UNUSED PtnValue ptn_add(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
        return ptn_array_union(left.as.array, right.as.array);
    }

    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        return ptn_add_integers(left_integer, right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        return ptn_float(left_fast_number + right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "+", right, line, &left_number, &right_number);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating + right_number.floating);
    }

    return ptn_add_integers(left_number.integer, right_number.integer);
}

static PTN_UNUSED PtnValue ptn_subtract_integers(int64_t left, int64_t right) {
    if ((right < 0 && left > INT64_MAX + right) ||
        (right > 0 && left < INT64_MIN + right)) {
        return ptn_float((double)left - (double)right);
    }
    return ptn_int(left - right);
}

static PTN_UNUSED PtnValue ptn_subtract(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        return ptn_subtract_integers(left_integer, right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        return ptn_float(left_fast_number - right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "-", right, line, &left_number, &right_number);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating - right_number.floating);
    }

    return ptn_subtract_integers(left_number.integer, right_number.integer);
}

static PTN_UNUSED int ptn_multiply_overflows(int64_t left, int64_t right) {
    if (left == 0 || right == 0) {
        return 0;
    }
    if (left > 0) {
        if (right > 0) {
            return left > INT64_MAX / right;
        }
        return right < INT64_MIN / left;
    }
    if (right > 0) {
        return left < INT64_MIN / right;
    }
    return right < INT64_MAX / left;
}

static PTN_UNUSED PtnValue ptn_multiply_integers(int64_t left, int64_t right) {
    if (ptn_multiply_overflows(left, right)) {
        return ptn_float((double)left * (double)right);
    }
    return ptn_int(left * right);
}

static PTN_UNUSED PtnValue ptn_multiply(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        return ptn_multiply_integers(left_integer, right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        return ptn_float(left_fast_number * right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "*", right, line, &left_number, &right_number);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating * right_number.floating);
    }

    return ptn_multiply_integers(left_number.integer, right_number.integer);
}

static PTN_UNUSED int ptn_integer_power_fits(int64_t base, int64_t exponent, int64_t *out) {
    if (exponent < 0) {
        return 0;
    }

    int64_t result = 1;
    int64_t factor = base;
    int64_t remaining = exponent;
    while (remaining > 0) {
        if ((remaining & 1) != 0) {
            if (ptn_multiply_overflows(result, factor)) {
                return 0;
            }
            result *= factor;
        }
        remaining >>= 1;
        if (remaining > 0) {
            if (ptn_multiply_overflows(factor, factor)) {
                return 0;
            }
            factor *= factor;
        }
    }

    *out = result;
    return 1;
}

static PTN_UNUSED PtnValue ptn_power(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        int64_t integer_result = 0;
        if (ptn_integer_power_fits(left_integer, right_integer, &integer_result)) {
            return ptn_int(integer_result);
        }
        return ptn_float(pow((double)left_integer, (double)right_integer));
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        return ptn_float(pow(left_fast_number, right_fast_number));
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "**", right, line, &left_number, &right_number);
    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        int64_t integer_result = 0;
        if (ptn_integer_power_fits(left_number.integer, right_number.integer, &integer_result)) {
            return ptn_int(integer_result);
        }
    }
    return ptn_float(pow(left_number.floating, right_number.floating));
}

static PTN_UNUSED PtnValue ptn_divide(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        if (right_integer == 0) {
            ptn_throw_exception_at(runtime, "DivisionByZeroError", "Division by zero", runtime->source_path, line);
            return ptn_null();
        }
        if (left_integer == INT64_MIN && right_integer == -1) {
            return ptn_float((double)left_integer / (double)right_integer);
        }
        if (left_integer % right_integer == 0) {
            return ptn_int(left_integer / right_integer);
        }
        return ptn_float((double)left_integer / (double)right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        if (right_fast_number == 0.0) {
            ptn_throw_exception_at(runtime, "DivisionByZeroError", "Division by zero", runtime->source_path, line);
            return ptn_null();
        }
        return ptn_float(left_fast_number / right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "/", right, line, &left_number, &right_number);
    if (right_number.floating == 0.0) {
        ptn_throw_exception_at(runtime, "DivisionByZeroError", "Division by zero", runtime->source_path, line);
        return ptn_null();
    }

    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        if (left_number.integer == INT64_MIN && right_number.integer == -1) {
            return ptn_float((double)left_number.integer / (double)right_number.integer);
        }
        if (left_number.integer % right_number.integer == 0) {
            return ptn_int(left_number.integer / right_number.integer);
        }
    }
    return ptn_float(left_number.floating / right_number.floating);
}

static PTN_UNUSED int ptn_float_to_int_loses_precision(double value) {
    if (value < -9223372036854775808.0 || value >= 9223372036854775808.0) {
        return 1;
    }
    int64_t integer = (int64_t)value;
    return (double)integer != value;
}

static PTN_UNUSED int ptn_float_to_int_out_of_range(double value) {
    return value < -9223372036854775808.0 || value >= 9223372036854775808.0;
}

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

static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation(
    PtnDiagnosticSink *diagnostics,
    double value
) {
    ptn_emit_float_to_int_precision_deprecation_at(
        diagnostics,
        value,
        "ptn-generated-code",
        0
    );
}

static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    double value,
    const char *path,
    size_t line
) {
    if (diagnostics != NULL && !ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    if (diagnostics != NULL) {
        diagnostics->emitted_deprecation = 1;
    }
    printf(
        "\nDeprecated: Implicit conversion from float %.16G to int loses precision in %s on line %zu\n",
        value,
        path,
        line
    );
}

static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation(
    PtnDiagnosticSink *diagnostics,
    const char *value
) {
    ptn_emit_float_string_to_int_precision_deprecation_at(
        diagnostics,
        value,
        "ptn-generated-code",
        0
    );
}

static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    const char *value,
    const char *path,
    size_t line
) {
    if (diagnostics != NULL && !ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    if (diagnostics != NULL) {
        diagnostics->emitted_deprecation = 1;
    }
    printf(
        "\nDeprecated: Implicit conversion from float-string \"%s\" to int loses precision in %s on line %zu\n",
        value,
        path,
        line
    );
}

static PTN_UNUSED int ptn_string_has_trailing_non_numeric_data(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    char *end = NULL;
    (void)strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    return *end != '\0';
}

static PTN_UNUSED void ptn_emit_non_numeric_value_warning(PtnDiagnosticSink *diagnostics) {
    if (diagnostics != NULL && !ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    printf("\nWarning: A non-numeric value encountered in ptn-generated-code on line 0\n");
}

static PTN_UNUSED int64_t ptn_number_to_integer(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return (int64_t)number.floating;
    }
    return number.integer;
}

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    PtnValue value,
    const char *path,
    size_t line
);

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation(
    PtnDiagnosticSink *diagnostics,
    PtnValue value
) {
    return ptn_value_to_integer_with_precision_deprecation_at(
        diagnostics,
        value,
        "ptn-generated-code",
        0
    );
}

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    PtnValue value,
    const char *path,
    size_t line
) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return integer;
    }
    if (value.type == PTN_FLOAT) {
        if (ptn_float_to_int_loses_precision(value.as.floating)) {
            ptn_emit_float_to_int_precision_deprecation_at(
                diagnostics,
                value.as.floating,
                path,
                line
            );
        }
        return (int64_t)value.as.floating;
    }

    PtnNumber number = ptn_to_number(value);
    const char *string_data = value.type == PTN_STRING ? (const char *)value.as.string.data : "";
    if (value.type == PTN_STRING && ptn_string_has_trailing_non_numeric_data(string_data)) {
        ptn_emit_non_numeric_value_warning(diagnostics);
    }
    if (number.type == PTN_NUMBER_FLOAT && ptn_float_to_int_loses_precision(number.floating)) {
        if (value.type == PTN_STRING) {
            ptn_emit_float_string_to_int_precision_deprecation_at(
                diagnostics,
                string_data,
                path,
                line
            );
        } else {
            ptn_emit_float_to_int_precision_deprecation_at(
                diagnostics,
                number.floating,
                path,
                line
            );
        }
    }
    return ptn_number_to_integer(number);
}

static PTN_UNUSED PtnValue ptn_modulo(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int64_t left_fast_integer = 0;
    int64_t right_fast_integer = 0;
    if (ptn_fast_integer_value(left, &left_fast_integer) &&
        ptn_fast_integer_value(right, &right_fast_integer)) {
        if (right_fast_integer == 0) {
            ptn_throw_exception_at(runtime, "DivisionByZeroError", "Modulo by zero", runtime->source_path, line);
            return ptn_null();
        }
        if (left_fast_integer == INT64_MIN && right_fast_integer == -1) {
            return ptn_int(0);
        }
        return ptn_int(left_fast_integer % right_fast_integer);
    }

    int64_t left_integer = ptn_value_to_integer_with_precision_deprecation_at(
        &runtime->diagnostics,
        left,
        runtime->source_path,
        line
    );
    int64_t right_integer = ptn_value_to_integer_with_precision_deprecation_at(
        &runtime->diagnostics,
        right,
        runtime->source_path,
        line
    );
    if (right_integer == 0) {
        ptn_throw_exception_at(runtime, "DivisionByZeroError", "Modulo by zero", runtime->source_path, line);
        return ptn_null();
    }
    if (left_integer == INT64_MIN && right_integer == -1) {
        return ptn_int(0);
    }
    return ptn_int(left_integer % right_integer);
}

static PTN_UNUSED PtnValue ptn_increment_number(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating + 1.0);
    }
    return ptn_add_integers(number.integer, 1);
}

static PTN_UNUSED PtnValue ptn_decrement_number(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating - 1.0);
    }
    return ptn_subtract_integers(number.integer, 1);
}

static PTN_UNUSED int ptn_increment_string_byte_is_alnum(unsigned char byte) {
    return (byte >= '0' && byte <= '9') ||
        (byte >= 'a' && byte <= 'z') ||
        (byte >= 'A' && byte <= 'Z');
}

static PTN_UNUSED PtnValue ptn_increment_string(PtnString string) {
    if (string.len == 0) {
        return ptn_string_literal("1", 1);
    }

    PtnNumber number;
    int has_trailing_non_numeric_data = 0;
    if (ptn_arithmetic_string_to_number(string, &number, &has_trailing_non_numeric_data) &&
        !has_trailing_non_numeric_data) {
        return ptn_increment_number(number);
    }

    if (!ptn_increment_string_byte_is_alnum(string.data[string.len - 1])) {
        return ptn_owned_string_len(
            ptn_duplicate_string_len((const char *)string.data, string.len),
            string.len
        );
    }

    char *result = ptn_duplicate_string_len((const char *)string.data, string.len);
    int carry = 0;
    char carry_prefix = '\0';
    for (size_t offset = string.len; offset > 0; offset--) {
        size_t index = offset - 1;
        unsigned char byte = (unsigned char)result[index];
        if (byte >= '0' && byte <= '8') {
            result[index] = (char)(byte + 1);
            carry = 0;
            break;
        }
        if (byte == '9') {
            result[index] = '0';
            carry = 1;
            carry_prefix = '1';
            continue;
        }
        if (byte >= 'a' && byte <= 'y') {
            result[index] = (char)(byte + 1);
            carry = 0;
            break;
        }
        if (byte == 'z') {
            result[index] = 'a';
            carry = 1;
            carry_prefix = 'a';
            continue;
        }
        if (byte >= 'A' && byte <= 'Y') {
            result[index] = (char)(byte + 1);
            carry = 0;
            break;
        }
        if (byte == 'Z') {
            result[index] = 'A';
            carry = 1;
            carry_prefix = 'A';
            continue;
        }
        carry = 0;
        break;
    }

    if (carry) {
        if (string.len == SIZE_MAX) {
            free(result);
            ptn_abort_out_of_memory();
        }
        char *prefixed = malloc(string.len + 2);
        if (prefixed == NULL) {
            free(result);
            ptn_abort_out_of_memory();
        }
        prefixed[0] = carry_prefix;
        memcpy(prefixed + 1, result, string.len + 1);
        free(result);
        return ptn_owned_string_len(prefixed, string.len + 1);
    }
    return ptn_owned_string_len(result, string.len);
}

static PTN_UNUSED PtnValue ptn_decrement_string(PtnString string) {
    if (string.len == 0) {
        return ptn_subtract_integers(0, 1);
    }

    PtnNumber number;
    int has_trailing_non_numeric_data = 0;
    if (ptn_arithmetic_string_to_number(string, &number, &has_trailing_non_numeric_data) &&
        !has_trailing_non_numeric_data) {
        return ptn_decrement_number(number);
    }

    return ptn_owned_string_len(
        ptn_duplicate_string_len((const char *)string.data, string.len),
        string.len
    );
}

static PTN_UNUSED void ptn_throw_invalid_increment_decrement(
    PtnRuntime *runtime,
    const char *operation,
    PtnValue value,
    size_t line
) {
    const char *type_name = ptn_arithmetic_operand_type_name(value);
    int needed = snprintf(NULL, 0, "Cannot %s %s", operation, type_name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(message, (size_t)needed + 1, "Cannot %s %s", operation, type_name);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
    free(message);
}

static PTN_UNUSED PtnValue ptn_increment_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return ptn_int(1);
        case PTN_BOOL:
            return ptn_bool(value.as.boolean);
        case PTN_INT:
            return ptn_add_integers(value.as.integer, 1);
        case PTN_FLOAT:
            return ptn_float(value.as.floating + 1.0);
        case PTN_STRING:
            return ptn_increment_string(value.as.string);
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            ptn_throw_invalid_increment_decrement(runtime, "increment", value, line);
            return ptn_null();
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_decrement_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return ptn_null();
        case PTN_BOOL:
            return ptn_bool(value.as.boolean);
        case PTN_INT:
            return ptn_subtract_integers(value.as.integer, 1);
        case PTN_FLOAT:
            return ptn_float(value.as.floating - 1.0);
        case PTN_STRING:
            return ptn_decrement_string(value.as.string);
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            ptn_throw_invalid_increment_decrement(runtime, "decrement", value, line);
            return ptn_null();
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_bitwise_string_and(PtnStringOperand left, PtnStringOperand right) {
    size_t left_len = left.len;
    size_t right_len = right.len;
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left.data[i] & (unsigned char)right.data[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string_len(result, result_len);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_or(PtnStringOperand left, PtnStringOperand right) {
    size_t left_len = left.len;
    size_t right_len = right.len;
    size_t result_len = left_len > right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        unsigned char left_byte = i < left_len ? (unsigned char)left.data[i] : 0;
        unsigned char right_byte = i < right_len ? (unsigned char)right.data[i] : 0;
        result[i] = (char)(left_byte | right_byte);
    }
    result[result_len] = '\0';
    return ptn_owned_string_len(result, result_len);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_xor(PtnStringOperand left, PtnStringOperand right) {
    size_t left_len = left.len;
    size_t right_len = right.len;
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left.data[i] ^ (unsigned char)right.data[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string_len(result, result_len);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_not(PtnStringOperand value) {
    size_t len = value.len;
    char *result = malloc(len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        result[i] = (char)(~(unsigned char)value.data[i]);
    }
    result[len] = '\0';
    return ptn_owned_string_len(result, len);
}

static PTN_UNUSED int64_t ptn_value_to_integer(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(NULL, value);
}

static PTN_UNUSED int64_t ptn_bitwise_integer_operand(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(NULL, value);
}

static PTN_UNUSED void ptn_format_bitwise_float_diagnostic(double value, char *buffer, size_t buffer_size) {
    int written = snprintf(buffer, buffer_size, "%.16G", value);
    if (written < 0 || (size_t)written >= buffer_size) {
        ptn_abort_out_of_memory();
    }
}

static PTN_UNUSED void ptn_emit_bitwise_float_out_of_range_warning(
    PtnDiagnosticSink *diagnostics,
    double value,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    char formatted[64];
    ptn_format_bitwise_float_diagnostic(value, formatted, sizeof(formatted));
    fputc('\n', stdout);
    fputs("Warning: The float ", stdout);
    fputs(formatted, stdout);
    fputs(" is not representable as an int, cast occurred in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED int64_t ptn_bitwise_integer_operand_checked(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_FLOAT) {
        if (ptn_float_to_int_out_of_range(value.as.floating)) {
            ptn_emit_bitwise_float_out_of_range_warning(&runtime->diagnostics, value.as.floating, line);
            return INT64_MIN;
        }
        if (ptn_float_to_int_loses_precision(value.as.floating)) {
            ptn_emit_float_to_int_precision_deprecation(&runtime->diagnostics, value.as.floating);
        }
        return (int64_t)value.as.floating;
    }
    return ptn_value_to_integer_with_precision_deprecation(&runtime->diagnostics, value);
}

static PTN_UNUSED PtnValue ptn_bitwise_and(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        PtnStringOperand left_string = {
            (const char *)left.as.string.data,
            NULL,
            left.as.string.len
        };
        PtnStringOperand right_string = {
            (const char *)right.as.string.data,
            NULL,
            right.as.string.len
        };
        return ptn_bitwise_string_and(left_string, right_string);
    }
    return ptn_int(
        ptn_bitwise_integer_operand_checked(runtime, left, line) &
        ptn_bitwise_integer_operand_checked(runtime, right, line)
    );
}

static PTN_UNUSED PtnValue ptn_bitwise_or(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        PtnStringOperand left_string = {
            (const char *)left.as.string.data,
            NULL,
            left.as.string.len
        };
        PtnStringOperand right_string = {
            (const char *)right.as.string.data,
            NULL,
            right.as.string.len
        };
        return ptn_bitwise_string_or(left_string, right_string);
    }
    return ptn_int(
        ptn_bitwise_integer_operand_checked(runtime, left, line) |
        ptn_bitwise_integer_operand_checked(runtime, right, line)
    );
}

static PTN_UNUSED PtnValue ptn_bitwise_xor(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        PtnStringOperand left_string = {
            (const char *)left.as.string.data,
            NULL,
            left.as.string.len
        };
        PtnStringOperand right_string = {
            (const char *)right.as.string.data,
            NULL,
            right.as.string.len
        };
        return ptn_bitwise_string_xor(left_string, right_string);
    }
    return ptn_int(
        ptn_bitwise_integer_operand_checked(runtime, left, line) ^
        ptn_bitwise_integer_operand_checked(runtime, right, line)
    );
}
