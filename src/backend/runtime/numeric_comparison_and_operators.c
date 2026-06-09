    for (size_t i = 0; i < argc; i++) {
        PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
        ptn_array_set_entry(array, key, ptn_value_clone(values[i]));
    }
    return (int64_t)array->len;
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
            case PTN_EXCEPTION:
                return left.as.exception == right.as.exception;
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
                return other.as.string[0] == '\0';
            case PTN_ARRAY:
                return other.as.array->len == 0;
            case PTN_EXCEPTION:
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
            if (left.as.string == right.as.string) {
                return 1;
            }
            return strcmp(left.as.string, right.as.string) == 0;
        case PTN_ARRAY:
            if (left.as.array == right.as.array) {
                return 1;
            }
            return ptn_compare_arrays_identical(left.as.array, right.as.array);
        case PTN_EXCEPTION:
            return left.as.exception == right.as.exception;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_not_identical(PtnValue left, PtnValue right) {
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
            if (left.as.string == right.as.string) {
                return 0;
            }
            return strcmp(left.as.string, right.as.string) != 0;
        case PTN_ARRAY:
            if (left.as.array == right.as.array) {
                return 0;
            }
            return !ptn_compare_arrays_identical(left.as.array, right.as.array);
        case PTN_EXCEPTION:
            return left.as.exception != right.as.exception;
    }
    return 1;
}

static PTN_UNUSED int ptn_value_is_nan(PtnValue value) {
    return value.type == PTN_FLOAT && isnan(value.as.floating);
}

static PTN_UNUSED int ptn_compare_order(PtnValue left, PtnValue right) {
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
            case PTN_EXCEPTION:
                return left.as.exception == right.as.exception ? PTN_COMPARE_EQUAL : PTN_COMPARE_GREATER;
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
            return ptn_compare_strings("", right.as.string);
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
            return ptn_compare_strings(left.as.string, "");
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

static PTN_UNUSED PtnValue ptn_add(PtnValue left, PtnValue right) {
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

    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
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

static PTN_UNUSED PtnValue ptn_subtract(PtnValue left, PtnValue right) {
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

    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
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

static PTN_UNUSED PtnValue ptn_multiply(PtnValue left, PtnValue right) {
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

    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
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

static PTN_UNUSED PtnValue ptn_power(PtnValue left, PtnValue right) {
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

    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        int64_t integer_result = 0;
        if (ptn_integer_power_fits(left_number.integer, right_number.integer, &integer_result)) {
            return ptn_int(integer_result);
        }
    }
    return ptn_float(pow(left_number.floating, right_number.floating));
}

static PTN_UNUSED PtnValue ptn_divide(PtnValue left, PtnValue right) {
    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        if (right_integer == 0) {
            ptn_abort_arithmetic_error("Division by zero");
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
            ptn_abort_arithmetic_error("Division by zero");
        }
        return ptn_float(left_fast_number / right_fast_number);
    }

    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (right_number.floating == 0.0) {
        ptn_abort_arithmetic_error("Division by zero");
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

static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation(double value) {
    printf(
        "\nDeprecated: Implicit conversion from float %.14g to int loses precision in ptn-generated-code on line 0\n",
        value
    );
}

static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation(const char *value) {
    printf(
        "\nDeprecated: Implicit conversion from float-string \"%s\" to int loses precision in ptn-generated-code on line 0\n",
        value
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

static PTN_UNUSED void ptn_emit_non_numeric_value_warning(void) {
    printf("\nWarning: A non-numeric value encountered in ptn-generated-code on line 0\n");
}

static PTN_UNUSED int64_t ptn_number_to_integer(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return (int64_t)number.floating;
    }
    return number.integer;
}

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation(PtnValue value) {
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return integer;
    }
    if (value.type == PTN_FLOAT) {
        if (ptn_float_to_int_loses_precision(value.as.floating)) {
            ptn_emit_float_to_int_precision_deprecation(value.as.floating);
        }
        return (int64_t)value.as.floating;
    }

    PtnNumber number = ptn_to_number(value);
    if (value.type == PTN_STRING && ptn_string_has_trailing_non_numeric_data(value.as.string)) {
        ptn_emit_non_numeric_value_warning();
    }
    if (number.type == PTN_NUMBER_FLOAT && ptn_float_to_int_loses_precision(number.floating)) {
        if (value.type == PTN_STRING) {
            ptn_emit_float_string_to_int_precision_deprecation(value.as.string);
        } else {
            ptn_emit_float_to_int_precision_deprecation(number.floating);
        }
    }
    return ptn_number_to_integer(number);
}

static PTN_UNUSED PtnValue ptn_modulo(PtnValue left, PtnValue right) {
    int64_t left_fast_integer = 0;
    int64_t right_fast_integer = 0;
    if (ptn_fast_integer_value(left, &left_fast_integer) &&
        ptn_fast_integer_value(right, &right_fast_integer)) {
        if (right_fast_integer == 0) {
            ptn_abort_arithmetic_error("Modulo by zero");
        }
        if (left_fast_integer == INT64_MIN && right_fast_integer == -1) {
            return ptn_int(0);
        }
        return ptn_int(left_fast_integer % right_fast_integer);
    }

    int64_t left_integer = ptn_value_to_integer_with_precision_deprecation(left);
    int64_t right_integer = ptn_value_to_integer_with_precision_deprecation(right);
    if (right_integer == 0) {
        ptn_abort_arithmetic_error("Modulo by zero");
    }
    if (left_integer == INT64_MIN && right_integer == -1) {
        return ptn_int(0);
    }
    return ptn_int(left_integer % right_integer);
}

static PTN_UNUSED PtnValue ptn_increment(PtnValue value) {
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_add_integers(integer, 1);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(value.as.floating + 1.0);
    }
    return ptn_add(value, ptn_int(1));
}

static PTN_UNUSED PtnValue ptn_decrement(PtnValue value) {
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_subtract_integers(integer, 1);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(value.as.floating - 1.0);
    }
    return ptn_subtract(value, ptn_int(1));
}

static PTN_UNUSED PtnValue ptn_bitwise_string_and(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left[i] & (unsigned char)right[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_or(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len > right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        unsigned char left_byte = i < left_len ? (unsigned char)left[i] : 0;
        unsigned char right_byte = i < right_len ? (unsigned char)right[i] : 0;
        result[i] = (char)(left_byte | right_byte);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_xor(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left[i] ^ (unsigned char)right[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_not(const char *value) {
    size_t len = strlen(value);
    char *result = malloc(len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        result[i] = (char)(~(unsigned char)value[i]);
    }
    result[len] = '\0';
    return ptn_owned_string(result);
}

static PTN_UNUSED int64_t ptn_value_to_integer(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(value);
}

static PTN_UNUSED int64_t ptn_bitwise_integer_operand(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(value);
}

static PTN_UNUSED PtnValue ptn_bitwise_and(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_and(left.as.string, right.as.string);
    }
    return ptn_int(ptn_bitwise_integer_operand(left) & ptn_bitwise_integer_operand(right));
}

static PTN_UNUSED PtnValue ptn_bitwise_or(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_or(left.as.string, right.as.string);
    }
    return ptn_int(ptn_bitwise_integer_operand(left) | ptn_bitwise_integer_operand(right));
}

static PTN_UNUSED PtnValue ptn_bitwise_xor(PtnValue left, PtnValue right) {
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_bitwise_string_xor(left.as.string, right.as.string);
    }
    return ptn_int(ptn_bitwise_integer_operand(left) ^ ptn_bitwise_integer_operand(right));
