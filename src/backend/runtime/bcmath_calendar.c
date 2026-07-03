typedef struct {
    int sign;
    char *digits;
    size_t len;
    size_t scale;
} PtnBcNumber;

static PtnStringOperand ptn_internal_expect_string_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);
static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);
static PtnRuntime *ptn_runtime_config_root(PtnRuntime *runtime);
static void ptn_get_defined_constants_add_int(PtnValue table, const char *name, int64_t value);
static int ptn_constant_name_matches_any(const char *name, const char *const *names, size_t count);

static char *ptn_bc_duplicate_range(const char *data, size_t len) {
    char *copy = malloc(len + 1);
    if (copy == NULL) {
        ptn_abort_out_of_memory();
    }
    if (len != 0) {
        memcpy(copy, data, len);
    }
    copy[len] = '\0';
    return copy;
}

static char *ptn_bc_zero_digits(void) {
    return ptn_bc_duplicate_range("0", 1);
}

static char *ptn_bc_normalize_digits_owned(char *digits) {
    size_t len = strlen(digits);
    size_t first = 0;
    while (first + 1 < len && digits[first] == '0') {
        first++;
    }
    if (first == 0) {
        return digits;
    }
    size_t next_len = len - first;
    memmove(digits, digits + first, next_len + 1);
    return digits;
}

static int ptn_bc_digits_is_zero(const char *digits) {
    for (const char *p = digits; *p != '\0'; p++) {
        if (*p != '0') {
            return 0;
        }
    }
    return 1;
}

static void ptn_bc_number_free(PtnBcNumber *number) {
    if (number == NULL) {
        return;
    }
    free(number->digits);
    number->digits = NULL;
    number->len = 0;
    number->scale = 0;
    number->sign = 0;
}

static int ptn_bc_parse_number_operand(PtnStringOperand input, PtnBcNumber *out) {
    out->sign = 0;
    out->digits = NULL;
    out->len = 0;
    out->scale = 0;
    if (memchr(input.data, '\0', input.len) != NULL) {
        return 0;
    }

    size_t index = 0;
    int negative = 0;
    if (index < input.len && (input.data[index] == '+' || input.data[index] == '-')) {
        negative = input.data[index] == '-';
        index++;
    }

    int seen_dot = 0;
    size_t digit_count = 0;
    size_t fractional_digits = 0;
    for (size_t i = index; i < input.len; i++) {
        unsigned char ch = (unsigned char)input.data[i];
        if (ch == '.') {
            if (seen_dot) {
                return 0;
            }
            seen_dot = 1;
            continue;
        }
        if (!isdigit(ch)) {
            return 0;
        }
        digit_count++;
        if (seen_dot) {
            fractional_digits++;
        }
    }

    if (digit_count == 0) {
        out->digits = ptn_bc_zero_digits();
        out->len = 1;
        out->scale = 0;
        out->sign = 0;
        return 1;
    }

    char *digits = malloc(digit_count + 1);
    if (digits == NULL) {
        ptn_abort_out_of_memory();
    }
    size_t pos = 0;
    for (size_t i = index; i < input.len; i++) {
        unsigned char ch = (unsigned char)input.data[i];
        if (isdigit(ch)) {
            digits[pos++] = (char)ch;
        }
    }
    digits[pos] = '\0';
    digits = ptn_bc_normalize_digits_owned(digits);
    out->digits = digits;
    out->len = strlen(digits);
    out->scale = fractional_digits;
    out->sign = ptn_bc_digits_is_zero(digits) ? 0 : (negative ? -1 : 1);
    return 1;
}

static int ptn_bc_expect_number(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line,
    PtnBcNumber *out
) {
    PtnStringOperand operand =
        ptn_internal_expect_string_arg(runtime, function_name, position, argument_name, value, line);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_string_operand_free(operand);
        return 0;
    }
    int ok = ptn_bc_parse_number_operand(operand, out);
    ptn_string_operand_free(operand);
    if (!ok) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) is not well-formed",
            function_name,
            position,
            argument_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    return 1;
}

static int ptn_bc_current_scale(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_config_root(runtime);
    return root == NULL ? 0 : root->bcmath_scale;
}

static void ptn_bc_set_current_scale(PtnRuntime *runtime, int scale) {
    PtnRuntime *root = ptn_runtime_config_root(runtime);
    if (root != NULL) {
        root->bcmath_scale = scale;
    }
}

static int ptn_bc_expect_scale(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line,
    int default_scale,
    int *out
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_NULL) {
        *out = default_scale;
        return 1;
    }
    int64_t scale = ptn_internal_expect_integer_arg(
        runtime,
        function_name,
        position,
        argument_name,
        value,
        line
    );
    if (runtime->exceptions->active_exception != NULL) {
        return 0;
    }
    if (scale < 0 || scale > INT_MAX) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) must be between 0 and 2147483647",
            function_name,
            position,
            argument_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    *out = (int)scale;
    return 1;
}

static int ptn_bc_optional_scale(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    PtnValue value,
    size_t line,
    int *out
) {
    return ptn_bc_expect_scale(
        runtime,
        function_name,
        position,
        "scale",
        value,
        line,
        ptn_bc_current_scale(runtime),
        out
    );
}

static char *ptn_bc_append_zeros(const char *digits, size_t zeros) {
    if (ptn_bc_digits_is_zero(digits)) {
        return ptn_bc_zero_digits();
    }
    size_t len = strlen(digits);
    char *out = malloc(len + zeros + 1);
    if (out == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(out, digits, len);
    memset(out + len, '0', zeros);
    out[len + zeros] = '\0';
    return out;
}

static int ptn_bc_cmp_digits(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    if (left_len != right_len) {
        return left_len < right_len ? -1 : 1;
    }
    int cmp = strcmp(left, right);
    if (cmp == 0) {
        return 0;
    }
    return cmp < 0 ? -1 : 1;
}

static char *ptn_bc_add_digits(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t max_len = left_len > right_len ? left_len : right_len;
    char *out = malloc(max_len + 2);
    if (out == NULL) {
        ptn_abort_out_of_memory();
    }
    out[max_len + 1] = '\0';
    int carry = 0;
    for (size_t i = 0; i < max_len; i++) {
        int l = left_len > i ? left[left_len - 1 - i] - '0' : 0;
        int r = right_len > i ? right[right_len - 1 - i] - '0' : 0;
        int sum = l + r + carry;
        out[max_len - i] = (char)('0' + (sum % 10));
        carry = sum / 10;
    }
    out[0] = (char)('0' + carry);
    return ptn_bc_normalize_digits_owned(out);
}

static char *ptn_bc_sub_digits(const char *left, const char *right) {
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    char *out = malloc(left_len + 1);
    if (out == NULL) {
        ptn_abort_out_of_memory();
    }
    out[left_len] = '\0';
    int borrow = 0;
    for (size_t i = 0; i < left_len; i++) {
        int l = left[left_len - 1 - i] - '0' - borrow;
        int r = right_len > i ? right[right_len - 1 - i] - '0' : 0;
        if (l < r) {
            l += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[left_len - 1 - i] = (char)('0' + (l - r));
    }
    return ptn_bc_normalize_digits_owned(out);
}

static char *ptn_bc_mul_digits(const char *left, const char *right) {
    if (ptn_bc_digits_is_zero(left) || ptn_bc_digits_is_zero(right)) {
        return ptn_bc_zero_digits();
    }
    size_t left_len = strlen(left);
    size_t right_len = strlen(right);
    size_t out_len = left_len + right_len;
    int *acc = calloc(out_len, sizeof(int));
    if (acc == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < left_len; i++) {
        int l = left[left_len - 1 - i] - '0';
        for (size_t j = 0; j < right_len; j++) {
            int r = right[right_len - 1 - j] - '0';
            acc[out_len - 1 - (i + j)] += l * r;
        }
    }
    for (size_t i = out_len; i > 1; i--) {
        acc[i - 2] += acc[i - 1] / 10;
        acc[i - 1] %= 10;
    }
    char *out = malloc(out_len + 1);
    if (out == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < out_len; i++) {
        out[i] = (char)('0' + acc[i]);
    }
    out[out_len] = '\0';
    free(acc);
    return ptn_bc_normalize_digits_owned(out);
}

static char *ptn_bc_mul_digit(const char *digits, int digit) {
    if (digit == 0 || ptn_bc_digits_is_zero(digits)) {
        return ptn_bc_zero_digits();
    }
    char scalar[2] = { (char)('0' + digit), '\0' };
    return ptn_bc_mul_digits(digits, scalar);
}

static char *ptn_bc_div2_digits(const char *digits) {
    size_t len = strlen(digits);
    char *out = malloc(len + 1);
    if (out == NULL) {
        ptn_abort_out_of_memory();
    }
    int carry = 0;
    for (size_t i = 0; i < len; i++) {
        int value = carry * 10 + (digits[i] - '0');
        out[i] = (char)('0' + (value / 2));
        carry = value % 2;
    }
    out[len] = '\0';
    return ptn_bc_normalize_digits_owned(out);
}

static char *ptn_bc_add_small(const char *digits, int small) {
    char scalar[32];
    snprintf(scalar, sizeof(scalar), "%d", small);
    return ptn_bc_add_digits(digits, scalar);
}

static char *ptn_bc_sub_small(const char *digits, int small) {
    char scalar[32];
    snprintf(scalar, sizeof(scalar), "%d", small);
    return ptn_bc_sub_digits(digits, scalar);
}

static void ptn_bc_divmod_digits(
    const char *numerator,
    const char *denominator,
    char **quotient_out,
    char **remainder_out
) {
    if (ptn_bc_digits_is_zero(denominator)) {
        *quotient_out = ptn_bc_zero_digits();
        *remainder_out = ptn_bc_zero_digits();
        return;
    }
    if (ptn_bc_cmp_digits(numerator, denominator) < 0) {
        *quotient_out = ptn_bc_zero_digits();
        *remainder_out = ptn_bc_duplicate_range(numerator, strlen(numerator));
        return;
    }
    size_t len = strlen(numerator);
    char *quotient = malloc(len + 1);
    if (quotient == NULL) {
        ptn_abort_out_of_memory();
    }
    char *remainder = ptn_bc_zero_digits();
    for (size_t i = 0; i < len; i++) {
        size_t rem_len = strlen(remainder);
        char *next;
        if (ptn_bc_digits_is_zero(remainder) && numerator[i] == '0') {
            next = ptn_bc_zero_digits();
        } else if (ptn_bc_digits_is_zero(remainder)) {
            next = ptn_bc_duplicate_range(&numerator[i], 1);
        } else {
            next = malloc(rem_len + 2);
            if (next == NULL) {
                ptn_abort_out_of_memory();
            }
            memcpy(next, remainder, rem_len);
            next[rem_len] = numerator[i];
            next[rem_len + 1] = '\0';
            next = ptn_bc_normalize_digits_owned(next);
        }
        free(remainder);
        remainder = next;
        int q = 0;
        for (int candidate = 9; candidate >= 1; candidate--) {
            char *product = ptn_bc_mul_digit(denominator, candidate);
            int cmp = ptn_bc_cmp_digits(product, remainder);
            if (cmp <= 0) {
                q = candidate;
                char *new_remainder = ptn_bc_sub_digits(remainder, product);
                free(remainder);
                remainder = new_remainder;
                free(product);
                break;
            }
            free(product);
        }
        quotient[i] = (char)('0' + q);
    }
    quotient[len] = '\0';
    *quotient_out = ptn_bc_normalize_digits_owned(quotient);
    *remainder_out = remainder;
}

static char *ptn_bc_truncate_or_pad_abs(const char *digits, size_t current_scale, size_t target_scale) {
    size_t len = strlen(digits);
    if (ptn_bc_digits_is_zero(digits)) {
        return ptn_bc_zero_digits();
    }
    if (current_scale == target_scale) {
        return ptn_bc_duplicate_range(digits, len);
    }
    if (current_scale < target_scale) {
        return ptn_bc_append_zeros(digits, target_scale - current_scale);
    }
    size_t drop = current_scale - target_scale;
    if (drop >= len) {
        return ptn_bc_zero_digits();
    }
    char *out = ptn_bc_duplicate_range(digits, len - drop);
    return ptn_bc_normalize_digits_owned(out);
}

static size_t ptn_bc_format_output_length(const char *digits, int sign, size_t current_scale, size_t out_scale) {
    int zero = ptn_bc_digits_is_zero(digits);
    size_t scaled_len = zero ? 1 : strlen(digits);
    if (!zero && current_scale < out_scale) {
        size_t padding = out_scale - current_scale;
        if (scaled_len > SIZE_MAX - padding) {
            return SIZE_MAX;
        }
        scaled_len += padding;
    } else if (!zero && current_scale > out_scale) {
        size_t drop = current_scale - out_scale;
        scaled_len = drop >= scaled_len ? 1 : scaled_len - drop;
    }

    size_t integer_len = scaled_len > out_scale ? scaled_len - out_scale : 0;
    size_t out_len = 0;
    if (sign < 0 && !zero) {
        out_len++;
    }
    out_len += integer_len == 0 ? 1 : integer_len;
    if (out_scale != 0) {
        if (out_len > SIZE_MAX - 1 || out_len + 1 > SIZE_MAX - out_scale) {
            return SIZE_MAX;
        }
        out_len += 1 + out_scale;
    }
    return out_len;
}

static void ptn_bc_enforce_format_memory_limit(
    PtnRuntime *runtime,
    const char *digits,
    int sign,
    size_t current_scale,
    size_t out_scale,
    size_t line
) {
    if (runtime == NULL) {
        return;
    }
    size_t output_len = ptn_bc_format_output_length(digits, sign, current_scale, out_scale);
    ptn_string_result_enforce_memory_limit(runtime, output_len, line);
}

static PtnValue ptn_bc_format_digits_value(
    const char *digits,
    int sign,
    size_t current_scale,
    size_t out_scale
) {
    char *scaled = ptn_bc_truncate_or_pad_abs(digits, current_scale, out_scale);
    size_t len = strlen(scaled);
    int zero = ptn_bc_digits_is_zero(scaled);
    size_t integer_len = len > out_scale ? len - out_scale : 0;
    size_t fractional_zero_prefix = out_scale > len ? out_scale - len : 0;
    size_t out_len = (sign < 0 && !zero ? 1 : 0)
        + (integer_len == 0 ? 1 : integer_len)
        + (out_scale == 0 ? 0 : 1 + out_scale);
    char *out = malloc(out_len + 1);
    if (out == NULL) {
        ptn_abort_out_of_memory();
    }
    size_t pos = 0;
    if (sign < 0 && !zero) {
        out[pos++] = '-';
    }
    if (integer_len == 0) {
        out[pos++] = '0';
    } else {
        size_t first = 0;
        while (first + 1 < integer_len && scaled[first] == '0') {
            first++;
        }
        memcpy(out + pos, scaled + first, integer_len - first);
        pos += integer_len - first;
    }
    if (out_scale != 0) {
        out[pos++] = '.';
        if (fractional_zero_prefix != 0) {
            memset(out + pos, '0', fractional_zero_prefix);
            pos += fractional_zero_prefix;
        }
        size_t available_fraction = len > integer_len ? len - integer_len : 0;
        if (available_fraction != 0) {
            memcpy(out + pos, scaled + integer_len, available_fraction);
            pos += available_fraction;
        }
        while (pos < out_len) {
            out[pos++] = '0';
        }
    }
    out[pos] = '\0';
    free(scaled);
    return ptn_owned_string_len(out, out_len);
}

static PtnValue ptn_bc_format_digits_value_checked(
    PtnRuntime *runtime,
    const char *digits,
    int sign,
    size_t current_scale,
    size_t out_scale,
    size_t line
) {
    ptn_bc_enforce_format_memory_limit(runtime, digits, sign, current_scale, out_scale, line);
    return ptn_bc_format_digits_value(digits, sign, current_scale, out_scale);
}

static void ptn_bc_align_numbers(
    const PtnBcNumber *left,
    const PtnBcNumber *right,
    char **left_out,
    char **right_out,
    size_t *scale_out
) {
    size_t scale = left->scale > right->scale ? left->scale : right->scale;
    *left_out = ptn_bc_append_zeros(left->digits, scale - left->scale);
    *right_out = ptn_bc_append_zeros(right->digits, scale - right->scale);
    *scale_out = scale;
}

static PtnValue ptn_bc_add_or_sub_values(
    PtnRuntime *runtime,
    const PtnBcNumber *left,
    const PtnBcNumber *right,
    int negate_right,
    int scale,
    size_t line
) {
    char *left_digits;
    char *right_digits;
    size_t aligned_scale;
    ptn_bc_align_numbers(left, right, &left_digits, &right_digits, &aligned_scale);
    int right_sign = negate_right ? -right->sign : right->sign;
    char *result_digits = NULL;
    int result_sign = 0;
    if (left->sign == 0) {
        result_digits = ptn_bc_duplicate_range(right_digits, strlen(right_digits));
        result_sign = right_sign;
    } else if (right_sign == 0) {
        result_digits = ptn_bc_duplicate_range(left_digits, strlen(left_digits));
        result_sign = left->sign;
    } else if (left->sign == right_sign) {
        result_digits = ptn_bc_add_digits(left_digits, right_digits);
        result_sign = left->sign;
    } else {
        int cmp = ptn_bc_cmp_digits(left_digits, right_digits);
        if (cmp == 0) {
            result_digits = ptn_bc_zero_digits();
            result_sign = 0;
        } else if (cmp > 0) {
            result_digits = ptn_bc_sub_digits(left_digits, right_digits);
            result_sign = left->sign;
        } else {
            result_digits = ptn_bc_sub_digits(right_digits, left_digits);
            result_sign = right_sign;
        }
    }
    if (ptn_bc_digits_is_zero(result_digits)) {
        result_sign = 0;
    }
    PtnValue result = ptn_bc_format_digits_value_checked(
        runtime,
        result_digits,
        result_sign,
        aligned_scale,
        (size_t)scale,
        line
    );
    free(left_digits);
    free(right_digits);
    free(result_digits);
    return result;
}

static PtnValue ptn_bc_mul_value(PtnRuntime *runtime, const PtnBcNumber *left, const PtnBcNumber *right, int scale, size_t line) {
    char *digits = ptn_bc_mul_digits(left->digits, right->digits);
    int sign = (left->sign == 0 || right->sign == 0) ? 0 : left->sign * right->sign;
    PtnValue result = ptn_bc_format_digits_value_checked(
        runtime,
        digits,
        sign,
        left->scale + right->scale,
        (size_t)scale,
        line
    );
    free(digits);
    return result;
}

static char *ptn_bc_div_abs_digits(const PtnBcNumber *left, const PtnBcNumber *right, size_t scale) {
    char *numerator = ptn_bc_append_zeros(left->digits, scale + right->scale);
    char *denominator = ptn_bc_append_zeros(right->digits, left->scale);
    char *quotient;
    char *remainder;
    ptn_bc_divmod_digits(numerator, denominator, &quotient, &remainder);
    free(numerator);
    free(denominator);
    free(remainder);
    return quotient;
}

static PtnValue ptn_bc_div_value(
    PtnRuntime *runtime,
    const char *function_name,
    const PtnBcNumber *left,
    const PtnBcNumber *right,
    int scale,
    size_t line
) {
    if (right->sign == 0) {
        ptn_throw_exception(runtime, "DivisionByZeroError", "Division by zero");
        return ptn_null();
    }
    char *quotient = ptn_bc_div_abs_digits(left, right, (size_t)scale);
    int sign = (left->sign == 0 || ptn_bc_digits_is_zero(quotient)) ? 0 : left->sign * right->sign;
    (void)function_name;
    PtnValue result = ptn_bc_format_digits_value_checked(runtime, quotient, sign, (size_t)scale, (size_t)scale, line);
    free(quotient);
    return result;
}

static PtnBcNumber ptn_bc_number_from_owned(char *digits, int sign, size_t scale) {
    PtnBcNumber number;
    digits = ptn_bc_normalize_digits_owned(digits);
    number.sign = ptn_bc_digits_is_zero(digits) ? 0 : sign;
    number.digits = digits;
    number.len = strlen(digits);
    number.scale = scale;
    return number;
}

static PtnBcNumber ptn_bc_number_clone_abs_scaled(const PtnBcNumber *number, size_t scale) {
    char *digits = ptn_bc_truncate_or_pad_abs(number->digits, number->scale, scale);
    return ptn_bc_number_from_owned(digits, number->sign, scale);
}

static PtnBcNumber ptn_bc_number_mul_exact(const PtnBcNumber *left, const PtnBcNumber *right) {
    char *digits = ptn_bc_mul_digits(left->digits, right->digits);
    int sign = (left->sign == 0 || right->sign == 0) ? 0 : left->sign * right->sign;
    return ptn_bc_number_from_owned(digits, sign, left->scale + right->scale);
}

static PtnBcNumber ptn_bc_number_sub_exact(const PtnBcNumber *left, const PtnBcNumber *right) {
    char *left_digits;
    char *right_digits;
    size_t scale;
    ptn_bc_align_numbers(left, right, &left_digits, &right_digits, &scale);
    PtnBcNumber result;
    if (left->sign == right->sign) {
        int cmp = ptn_bc_cmp_digits(left_digits, right_digits);
        if (cmp == 0) {
            result = ptn_bc_number_from_owned(ptn_bc_zero_digits(), 0, scale);
        } else if (cmp > 0) {
            result = ptn_bc_number_from_owned(ptn_bc_sub_digits(left_digits, right_digits), left->sign, scale);
        } else {
            result = ptn_bc_number_from_owned(ptn_bc_sub_digits(right_digits, left_digits), -right->sign, scale);
        }
    } else {
        result = ptn_bc_number_from_owned(ptn_bc_add_digits(left_digits, right_digits), left->sign, scale);
    }
    free(left_digits);
    free(right_digits);
    return result;
}

static PtnValue ptn_bc_mod_value(
    PtnRuntime *runtime,
    const PtnBcNumber *left,
    const PtnBcNumber *right,
    int scale,
    size_t line
) {
    if (right->sign == 0) {
        ptn_throw_exception(runtime, "DivisionByZeroError", "Modulo by zero");
        return ptn_null();
    }
    char *quot_digits = ptn_bc_div_abs_digits(left, right, 0);
    PtnBcNumber quotient = ptn_bc_number_from_owned(quot_digits, left->sign == 0 ? 0 : left->sign * right->sign, 0);
    PtnBcNumber product = ptn_bc_number_mul_exact(&quotient, right);
    PtnBcNumber remainder = ptn_bc_number_sub_exact(left, &product);
    PtnValue result = ptn_bc_format_digits_value_checked(
        runtime,
        remainder.digits,
        remainder.sign,
        remainder.scale,
        (size_t)scale,
        line
    );
    ptn_bc_number_free(&quotient);
    ptn_bc_number_free(&product);
    ptn_bc_number_free(&remainder);
    return result;
}

static int ptn_bc_compare_at_scale(const PtnBcNumber *left, const PtnBcNumber *right, size_t scale) {
    PtnBcNumber a = ptn_bc_number_clone_abs_scaled(left, scale);
    PtnBcNumber b = ptn_bc_number_clone_abs_scaled(right, scale);
    int result;
    if (a.sign != b.sign) {
        result = a.sign < b.sign ? -1 : 1;
    } else if (a.sign == 0) {
        result = 0;
    } else {
        int cmp = ptn_bc_cmp_digits(a.digits, b.digits);
        result = a.sign > 0 ? cmp : -cmp;
    }
    ptn_bc_number_free(&a);
    ptn_bc_number_free(&b);
    return result;
}

static int ptn_bc_parse_exponent(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    const PtnBcNumber *number,
    int allow_negative,
    int64_t *out
) {
    if (number->scale != 0) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) cannot have a fractional part",
            function_name,
            position,
            argument_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    if (!allow_negative && number->sign < 0) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) must be greater than or equal to 0",
            function_name,
            position,
            argument_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    if (strlen(number->digits) > 18) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) is too large",
            function_name,
            position,
            argument_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    int64_t value = 0;
    for (const char *p = number->digits; *p != '\0'; p++) {
        value = value * 10 + (*p - '0');
    }
    if (number->sign < 0) {
        value = -value;
    }
    *out = value;
    return 1;
}

static int ptn_bc_number_has_nonzero_fraction(const PtnBcNumber *number) {
    if (number->scale == 0) {
        return 0;
    }
    size_t len = strlen(number->digits);
    size_t start = len > number->scale ? len - number->scale : 0;
    for (size_t i = start; i < len; i++) {
        if (number->digits[i] != '0') {
            return 1;
        }
    }
    return 0;
}

static PtnBcNumber ptn_bc_pow_nonnegative(const PtnBcNumber *base, int64_t exponent) {
    PtnBcNumber result = ptn_bc_number_from_owned(ptn_bc_duplicate_range("1", 1), 1, 0);
    PtnBcNumber factor = ptn_bc_number_from_owned(ptn_bc_duplicate_range(base->digits, strlen(base->digits)), base->sign, base->scale);
    int64_t exp = exponent;
    while (exp > 0) {
        if ((exp & 1) != 0) {
            PtnBcNumber next = ptn_bc_number_mul_exact(&result, &factor);
            ptn_bc_number_free(&result);
            result = next;
        }
        exp >>= 1;
        if (exp != 0) {
            PtnBcNumber next_factor = ptn_bc_number_mul_exact(&factor, &factor);
            ptn_bc_number_free(&factor);
            factor = next_factor;
        }
    }
    ptn_bc_number_free(&factor);
    return result;
}

static PtnValue ptn_bc_pow_value(PtnRuntime *runtime, const PtnBcNumber *base, int64_t exponent, int scale, size_t line) {
    if (exponent >= 0) {
        PtnBcNumber result = ptn_bc_pow_nonnegative(base, exponent);
        PtnValue value = ptn_bc_format_digits_value_checked(
            runtime,
            result.digits,
            result.sign,
            result.scale,
            (size_t)scale,
            line
        );
        ptn_bc_number_free(&result);
        return value;
    }
    if (base->sign == 0) {
        ptn_throw_exception(runtime, "DivisionByZeroError", "Negative power of zero");
        return ptn_null();
    }
    PtnBcNumber positive = ptn_bc_pow_nonnegative(base, -exponent);
    PtnBcNumber one = ptn_bc_number_from_owned(ptn_bc_duplicate_range("1", 1), 1, 0);
    PtnValue value = ptn_bc_div_value(runtime, "bcpow", &one, &positive, scale, line);
    ptn_bc_number_free(&positive);
    ptn_bc_number_free(&one);
    return value;
}

static char *ptn_bc_mod_abs_digits(const char *left, const char *right) {
    char *quotient;
    char *remainder;
    ptn_bc_divmod_digits(left, right, &quotient, &remainder);
    free(quotient);
    return remainder;
}

static PtnValue ptn_bc_powmod_value(
    PtnRuntime *runtime,
    const PtnBcNumber *base,
    const PtnBcNumber *exponent,
    const PtnBcNumber *modulus,
    int scale,
    size_t line
) {
    if (modulus->sign == 0) {
        ptn_throw_exception(runtime, "DivisionByZeroError", "Modulo by zero");
        return ptn_null();
    }
    if (ptn_bc_number_has_nonzero_fraction(base)) {
        ptn_throw_exception(runtime, "ValueError", "bcpowmod(): Argument #1 ($num) cannot have a fractional part");
        return ptn_null();
    }
    if (ptn_bc_number_has_nonzero_fraction(modulus)) {
        ptn_throw_exception(runtime, "ValueError", "bcpowmod(): Argument #3 ($modulus) cannot have a fractional part");
        return ptn_null();
    }
    if (exponent->scale != 0) {
        ptn_throw_exception(runtime, "ValueError", "bcpowmod(): Argument #2 ($exponent) cannot have a fractional part");
        return ptn_null();
    }
    if (ptn_bc_cmp_digits(modulus->digits, "1") == 0) {
        return ptn_bc_format_digits_value_checked(runtime, "0", 0, 0, (size_t)scale, line);
    }
    int64_t exp_value = 0;
    if (!ptn_bc_parse_exponent(runtime, "bcpowmod", 2, "exponent", exponent, 0, &exp_value)) {
        return ptn_null();
    }
    char *mod_abs = ptn_bc_duplicate_range(modulus->digits, strlen(modulus->digits));
    char *result = ptn_bc_duplicate_range("1", 1);
    char *factor = ptn_bc_mod_abs_digits(base->digits, mod_abs);
    int64_t exp = exp_value;
    while (exp > 0) {
        if ((exp & 1) != 0) {
            char *product = ptn_bc_mul_digits(result, factor);
            free(result);
            result = ptn_bc_mod_abs_digits(product, mod_abs);
            free(product);
        }
        exp >>= 1;
        if (exp != 0) {
            char *product = ptn_bc_mul_digits(factor, factor);
            free(factor);
            factor = ptn_bc_mod_abs_digits(product, mod_abs);
            free(product);
        }
    }
    int sign = 0;
    if (!ptn_bc_digits_is_zero(result)) {
        sign = base->sign < 0 && (exp_value % 2) != 0 ? -1 : 1;
    }
    PtnValue value = ptn_bc_format_digits_value_checked(runtime, result, sign, 0, (size_t)scale, line);
    free(mod_abs);
    free(result);
    free(factor);
    return value;
}

static char *ptn_bc_isqrt_digits(const char *digits) {
    if (ptn_bc_digits_is_zero(digits)) {
        return ptn_bc_zero_digits();
    }
    size_t len = strlen(digits);
    size_t high_digits = (len + 1) / 2 + 1;
    char *low = ptn_bc_zero_digits();
    char *high = malloc(high_digits + 1);
    if (high == NULL) {
        ptn_abort_out_of_memory();
    }
    high[0] = '1';
    memset(high + 1, '0', high_digits - 1);
    high[high_digits] = '\0';
    char *answer = ptn_bc_zero_digits();
    while (ptn_bc_cmp_digits(low, high) <= 0) {
        char *sum = ptn_bc_add_digits(low, high);
        char *mid = ptn_bc_div2_digits(sum);
        free(sum);
        char *mid_square = ptn_bc_mul_digits(mid, mid);
        int cmp = ptn_bc_cmp_digits(mid_square, digits);
        free(mid_square);
        if (cmp <= 0) {
            free(answer);
            answer = ptn_bc_duplicate_range(mid, strlen(mid));
            char *next_low = ptn_bc_add_small(mid, 1);
            free(low);
            low = next_low;
        } else {
            if (ptn_bc_digits_is_zero(mid)) {
                free(mid);
                break;
            }
            char *next_high = ptn_bc_sub_small(mid, 1);
            free(high);
            high = next_high;
        }
        free(mid);
    }
    free(low);
    free(high);
    return answer;
}

static PtnValue ptn_bc_sqrt_value(PtnRuntime *runtime, const PtnBcNumber *number, int scale, size_t line) {
    if (number->sign < 0) {
        ptn_throw_exception(runtime, "ValueError", "bcsqrt(): Argument #1 ($num) must be greater than or equal to 0");
        return ptn_null();
    }
    int64_t exponent = (int64_t)scale * 2 - (int64_t)number->scale;
    char *radicand;
    if (exponent >= 0) {
        radicand = ptn_bc_append_zeros(number->digits, (size_t)exponent);
    } else {
        size_t drop = (size_t)(-exponent);
        size_t len = strlen(number->digits);
        radicand = drop >= len ? ptn_bc_zero_digits() : ptn_bc_duplicate_range(number->digits, len - drop);
        radicand = ptn_bc_normalize_digits_owned(radicand);
    }
    char *root = ptn_bc_isqrt_digits(radicand);
    PtnValue value = ptn_bc_format_digits_value_checked(
        runtime,
        root,
        ptn_bc_digits_is_zero(root) ? 0 : 1,
        (size_t)scale,
        (size_t)scale,
        line
    );
    free(radicand);
    free(root);
    return value;
}

static int ptn_bc_has_fractional_nonzero(const PtnBcNumber *number) {
    if (number->scale == 0) {
        return 0;
    }
    size_t len = strlen(number->digits);
    if (number->scale >= len) {
        for (size_t i = 0; i < len; i++) {
            if (number->digits[i] != '0') {
                return 1;
            }
        }
        return 0;
    }
    for (size_t i = len - number->scale; i < len; i++) {
        if (number->digits[i] != '0') {
            return 1;
        }
    }
    return 0;
}

static PtnValue ptn_bc_ceil_floor_value(const PtnBcNumber *number, int ceiling) {
    char *integer_digits = ptn_bc_truncate_or_pad_abs(number->digits, number->scale, 0);
    int sign = ptn_bc_digits_is_zero(integer_digits) ? 0 : number->sign;
    if (ptn_bc_has_fractional_nonzero(number)) {
        if (ceiling && number->sign > 0) {
            char *next = ptn_bc_add_small(integer_digits, 1);
            free(integer_digits);
            integer_digits = next;
            sign = 1;
        } else if (!ceiling && number->sign < 0) {
            char *next = ptn_bc_add_small(integer_digits, 1);
            free(integer_digits);
            integer_digits = next;
            sign = -1;
        }
    }
    PtnValue value = ptn_bc_format_digits_value(integer_digits, sign, 0, 0);
    free(integer_digits);
    return value;
}

static const char *ptn_bc_rounding_mode_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT && ptn_ascii_case_equal(value.as.object->class_name, "RoundingMode")) {
        return value.as.object->enum_case_name == NULL ? "HalfAwayFromZero" : value.as.object->enum_case_name;
    }
    return "HalfAwayFromZero";
}

static int ptn_bc_round_should_increment(const char *kept_digits, const char *dropped, int sign, const char *mode) {
    int any_dropped = 0;
    for (const char *p = dropped; *p != '\0'; p++) {
        if (*p != '0') {
            any_dropped = 1;
            break;
        }
    }
    if (!any_dropped) {
        return 0;
    }
    int first = dropped[0] - '0';
    int beyond_half = first > 5;
    int exactly_half = first == 5;
    if (exactly_half) {
        for (const char *p = dropped + 1; *p != '\0'; p++) {
            if (*p != '0') {
                beyond_half = 1;
                exactly_half = 0;
                break;
            }
        }
    }
    if (ptn_ascii_case_equal(mode, "TowardsZero")) {
        return 0;
    }
    if (ptn_ascii_case_equal(mode, "AwayFromZero")) {
        return 1;
    }
    if (ptn_ascii_case_equal(mode, "PositiveInfinity")) {
        return sign > 0;
    }
    if (ptn_ascii_case_equal(mode, "NegativeInfinity")) {
        return sign < 0;
    }
    if (beyond_half) {
        return 1;
    }
    if (!exactly_half) {
        return 0;
    }
    if (ptn_ascii_case_equal(mode, "HalfTowardsZero")) {
        return 0;
    }
    if (ptn_ascii_case_equal(mode, "HalfEven") || ptn_ascii_case_equal(mode, "HalfOdd")) {
        size_t len = strlen(kept_digits);
        int last = len == 0 ? 0 : kept_digits[len - 1] - '0';
        int is_even = (last % 2) == 0;
        return ptn_ascii_case_equal(mode, "HalfEven") ? !is_even : is_even;
    }
    return 1;
}

static PtnValue ptn_bc_round_value(const PtnBcNumber *number, int precision, const char *mode) {
    size_t keep_scale = precision >= 0 ? (size_t)precision : 0;
    size_t shift_left = precision < 0 ? (size_t)(-precision) : 0;
    size_t len = strlen(number->digits);
    char *expanded = NULL;
    const char *digits = number->digits;
    if (number->scale > len) {
        size_t expanded_len = number->scale;
        expanded = malloc(expanded_len + 1);
        if (expanded == NULL) {
            ptn_abort_out_of_memory();
        }
        size_t prefix = number->scale - len;
        memset(expanded, '0', prefix);
        memcpy(expanded + prefix, number->digits, len + 1);
        digits = expanded;
        len = expanded_len;
    }
    size_t integer_digits = len > number->scale ? len - number->scale : 0;
    size_t keep_total = integer_digits + keep_scale;
    if (shift_left != 0) {
        keep_total = integer_digits > shift_left ? integer_digits - shift_left : 0;
    }
    if (keep_total >= len) {
        PtnValue early = ptn_bc_format_digits_value(digits, number->sign, number->scale, keep_scale);
        free(expanded);
        return early;
    }
    char *kept = keep_total == 0 ? ptn_bc_zero_digits() : ptn_bc_duplicate_range(digits, keep_total);
    const char *dropped = digits + keep_total;
    if (ptn_bc_round_should_increment(kept, dropped, number->sign, mode)) {
        char *next = ptn_bc_add_small(kept, 1);
        free(kept);
        kept = next;
    }
    size_t current_scale = keep_scale + shift_left;
    if (shift_left != 0 && !ptn_bc_digits_is_zero(kept)) {
        char *with_zeros = ptn_bc_append_zeros(kept, shift_left);
        free(kept);
        kept = with_zeros;
        current_scale = 0;
    }
    PtnValue value = ptn_bc_format_digits_value(kept, ptn_bc_digits_is_zero(kept) ? 0 : number->sign, current_scale, keep_scale);
    free(kept);
    free(expanded);
    return value;
}

#define PTN_BCMATH_NUMBER_CLASS "BcMath\\Number"

static const char *ptn_bcmath_number_arg_type_name(PtnValue value) {
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

static int ptn_bcmath_number_is_object(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_OBJECT &&
        ptn_internal_class_name_is_bcmath_number(value.as.object->class_name);
}

static PtnArrayEntry *ptn_bcmath_number_property_entry(PtnObject *object, const char *name) {
    if (object == NULL || object->properties == NULL) {
        return NULL;
    }
    PtnArrayKey key = ptn_array_string_key(name);
    PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
    ptn_array_key_free(key);
    return entry;
}

static int ptn_bcmath_number_read(PtnValue value, PtnBcNumber *out) {
    value = ptn_value_deref(value);
    if (!ptn_bcmath_number_is_object(value)) {
        return 0;
    }
    PtnArrayEntry *entry = ptn_bcmath_number_property_entry(value.as.object, "value");
    if (entry == NULL) {
        return 0;
    }
    PtnValue property = ptn_value_deref(entry->value);
    if (property.type != PTN_STRING) {
        return 0;
    }
    PtnStringOperand operand = ptn_string_operand_borrowed_len(
        (const char *)property.as.string.data,
        property.as.string.len
    );
    return ptn_bc_parse_number_operand(operand, out);
}

static PtnValue ptn_bcmath_number_value_clone(PtnValue value) {
    value = ptn_value_deref(value);
    if (!ptn_bcmath_number_is_object(value)) {
        return ptn_null();
    }
    PtnArrayEntry *entry = ptn_bcmath_number_property_entry(value.as.object, "value");
    if (entry == NULL) {
        return ptn_null();
    }
    return ptn_value_clone_deref(entry->value);
}

static void ptn_bcmath_number_throw_invalid_serialization(PtnRuntime *runtime) {
    ptn_throw_exception(runtime, "Exception", "Invalid serialization data for BcMath\\Number object");
}

static int ptn_bcmath_number_parse_serialization_data(
    PtnRuntime *runtime,
    PtnValue data,
    PtnBcNumber *out
) {
    data = ptn_value_deref(data);
    if (data.type != PTN_ARRAY || data.as.array->len != 1) {
        ptn_bcmath_number_throw_invalid_serialization(runtime);
        return 0;
    }

    PtnArrayKey key = ptn_array_string_key("value");
    PtnArrayEntry *entry = ptn_array_entry_for_key(data.as.array, key);
    ptn_array_key_free(key);
    if (entry == NULL) {
        ptn_bcmath_number_throw_invalid_serialization(runtime);
        return 0;
    }

    PtnValue value = ptn_value_deref(entry->value);
    if (value.type != PTN_STRING || value.as.string.len == 0) {
        ptn_bcmath_number_throw_invalid_serialization(runtime);
        return 0;
    }

    PtnStringOperand operand = ptn_string_operand_borrowed_len(
        (const char *)value.as.string.data,
        value.as.string.len
    );
    if (!ptn_bc_parse_number_operand(operand, out)) {
        ptn_bcmath_number_throw_invalid_serialization(runtime);
        return 0;
    }
    return 1;
}

static void ptn_bcmath_number_throw_value_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) is not well-formed",
        function_name,
        position,
        argument_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "ValueError", message);
}

static void ptn_bcmath_number_throw_type_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be of type int, string, or BcMath\\Number, %s given",
        function_name,
        position,
        argument_name,
        ptn_bcmath_number_arg_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static void ptn_bcmath_number_emit_null_deprecation(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Passing null to parameter #%zu ($%s) of type BcMath\\Number|string|int is deprecated",
        function_name,
        position,
        argument_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_deprecation(&runtime->diagnostics, message, line);
}

static int ptn_bcmath_number_parse_text(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnStringOperand operand,
    PtnBcNumber *out
) {
    if (!ptn_bc_parse_number_operand(operand, out)) {
        ptn_bcmath_number_throw_value_error(runtime, function_name, position, argument_name);
        return 0;
    }
    return 1;
}

static int ptn_bcmath_number_parse_int64(int64_t value, PtnBcNumber *out) {
    char buffer[32];
    int written = snprintf(buffer, sizeof(buffer), "%lld", (long long)value);
    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    PtnStringOperand operand = ptn_string_operand_borrowed_len(buffer, (size_t)written);
    return ptn_bc_parse_number_operand(operand, out);
}

static int ptn_bcmath_number_expect_operand(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line,
    int allow_number_object,
    PtnBcNumber *out
) {
    value = ptn_value_deref(value);
    if (allow_number_object && ptn_bcmath_number_is_object(value)) {
        if (!ptn_bcmath_number_read(value, out)) {
            ptn_throw_exception(runtime, "Error", "Invalid BcMath\\Number object");
            return 0;
        }
        return 1;
    }
    switch (value.type) {
        case PTN_NULL:
            ptn_bcmath_number_emit_null_deprecation(runtime, function_name, position, argument_name, line);
            return ptn_bcmath_number_parse_int64(0, out);
        case PTN_BOOL:
            return ptn_bcmath_number_parse_int64(value.as.boolean ? 1 : 0, out);
        case PTN_INT:
            return ptn_bcmath_number_parse_int64(value.as.integer, out);
        case PTN_FLOAT: {
            int64_t integer = ptn_internal_expect_integer_arg(
                runtime,
                function_name,
                position,
                argument_name,
                value,
                line
            );
            if (runtime->exceptions->active_exception != NULL) {
                return 0;
            }
            return ptn_bcmath_number_parse_int64(integer, out);
        }
        case PTN_STRING: {
            PtnStringOperand operand = ptn_string_operand_borrowed_len(
                (const char *)value.as.string.data,
                value.as.string.len
            );
            return ptn_bcmath_number_parse_text(runtime, function_name, position, argument_name, operand, out);
        }
        case PTN_OBJECT:
        case PTN_ARRAY:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            ptn_bcmath_number_throw_type_error(runtime, function_name, position, argument_name, value);
            return 0;
    }
    ptn_bcmath_number_throw_type_error(runtime, function_name, position, argument_name, value);
    return 0;
}

static int ptn_bcmath_number_expect_constructor_operand(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnBcNumber *out
) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL: {
            char message[192];
            int written = snprintf(
                message,
                sizeof(message),
                "BcMath\\Number::__construct(): Passing null to parameter #1 ($num) of type string|int is deprecated"
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_deprecation(&runtime->diagnostics, message, line);
            return ptn_bcmath_number_parse_int64(0, out);
        }
        case PTN_BOOL:
            return ptn_bcmath_number_parse_int64(value.as.boolean ? 1 : 0, out);
        case PTN_INT:
            return ptn_bcmath_number_parse_int64(value.as.integer, out);
        case PTN_FLOAT: {
            int64_t integer = ptn_internal_expect_integer_arg(
                runtime,
                "BcMath\\Number::__construct",
                1,
                "num",
                value,
                line
            );
            if (runtime->exceptions->active_exception != NULL) {
                return 0;
            }
            return ptn_bcmath_number_parse_int64(integer, out);
        }
        case PTN_STRING: {
            PtnStringOperand operand = ptn_string_operand_borrowed_len(
                (const char *)value.as.string.data,
                value.as.string.len
            );
            return ptn_bcmath_number_parse_text(
                runtime,
                "BcMath\\Number::__construct",
                1,
                "num",
                operand,
                out
            );
        }
        case PTN_OBJECT:
        case PTN_ARRAY:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_RESOURCE:
        case PTN_REFERENCE: {
            char message[224];
            int written = snprintf(
                message,
                sizeof(message),
                "BcMath\\Number::__construct(): Argument #1 ($num) must be of type string|int, %s given",
                ptn_bcmath_number_arg_type_name(value)
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "TypeError", message);
            return 0;
        }
    }
    return 0;
}

static void ptn_bcmath_number_declare_readonly_property(
    PtnRuntime *runtime,
    PtnValue object,
    const char *property,
    PtnPropertyTypeKind type_kind,
    PtnValue value,
    int has_value,
    size_t line
) {
    PtnValue assigned = ptn_object_declare_property(
        runtime,
        object,
        property,
        PTN_BCMATH_NUMBER_CLASS,
        PTN_PROPERTY_PUBLIC,
        PTN_PROPERTY_PUBLIC,
        1,
        type_kind,
        NULL,
        NULL,
        0,
        has_value,
        value,
        line
    );
    ptn_value_destroy(&assigned);
}

static PtnValue ptn_bcmath_number_initialize_from_value(
    PtnRuntime *runtime,
    PtnValue object,
    PtnValue value_string,
    int scale,
    size_t line
) {
    ptn_bcmath_number_declare_readonly_property(
        runtime,
        object,
        "value",
        PTN_PROPERTY_TYPE_STRING,
        value_string,
        1,
        line
    );
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    PtnValue scale_value = ptn_int(scale);
    ptn_bcmath_number_declare_readonly_property(
        runtime,
        object,
        "scale",
        PTN_PROPERTY_TYPE_INT,
        scale_value,
        1,
        line
    );
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    return object;
}

static PtnValue ptn_bcmath_number_object_from_value(
    PtnRuntime *runtime,
    PtnValue value_string,
    int scale,
    size_t line
) {
    PtnValue object = ptn_object_new_shell(runtime, PTN_BCMATH_NUMBER_CLASS);
    PtnValue initialized = ptn_bcmath_number_initialize_from_value(runtime, object, value_string, scale, line);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&object);
        return ptn_null();
    }
    return initialized;
}

static PtnValue ptn_bcmath_number_object_from_parsed(
    PtnRuntime *runtime,
    const PtnBcNumber *number,
    size_t line
) {
    PtnValue value = ptn_bc_format_digits_value(number->digits, number->sign, number->scale, number->scale);
    PtnValue object = ptn_bcmath_number_object_from_value(runtime, value, (int)number->scale, line);
    ptn_value_destroy(&value);
    return object;
}

static PtnValue ptn_bcmath_number_object_from_result(
    PtnRuntime *runtime,
    PtnValue value,
    int scale,
    size_t line
) {
    PtnValue object = ptn_bcmath_number_object_from_value(runtime, value, scale, line);
    return object;
}

static PtnValue ptn_bcmath_number_format_trimmed(
    const PtnBcNumber *number,
    size_t min_scale
) {
    size_t scale = number->scale;
    char *digits = ptn_bc_truncate_or_pad_abs(number->digits, number->scale, scale);
    while (scale > min_scale) {
        size_t len = strlen(digits);
        if (len == 0 || digits[len - 1] != '0') {
            break;
        }
        if (len == 1) {
            digits[0] = '0';
            digits[1] = '\0';
        } else {
            digits[len - 1] = '\0';
        }
        scale--;
    }
    PtnValue result = ptn_bc_format_digits_value(
        digits,
        ptn_bc_digits_is_zero(digits) ? 0 : number->sign,
        scale,
        scale
    );
    free(digits);
    return result;
}

static PtnValue ptn_bcmath_number_trim_value(PtnValue value, size_t min_scale, int *scale_out) {
    PtnValue deref = ptn_value_deref(value);
    PtnStringOperand operand = ptn_string_operand_borrowed_len(
        (const char *)deref.as.string.data,
        deref.as.string.len
    );
    PtnBcNumber parsed;
    if (!ptn_bc_parse_number_operand(operand, &parsed)) {
        *scale_out = 0;
        return ptn_string("0");
    }
    PtnValue result = ptn_bcmath_number_format_trimmed(&parsed, min_scale);
    PtnStringOperand result_operand = ptn_string_operand_borrowed_len(
        (const char *)result.as.string.data,
        result.as.string.len
    );
    PtnBcNumber trimmed;
    if (ptn_bc_parse_number_operand(result_operand, &trimmed)) {
        *scale_out = (int)trimmed.scale;
        ptn_bc_number_free(&trimmed);
    } else {
        *scale_out = 0;
    }
    ptn_bc_number_free(&parsed);
    return result;
}

static int ptn_bcmath_number_expect_scale(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    PtnValue value,
    size_t line,
    int default_scale,
    int *out
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_NULL) {
        *out = default_scale;
        return 1;
    }
    if (value.type == PTN_ARRAY ||
        value.type == PTN_OBJECT ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION ||
        value.type == PTN_RESOURCE) {
        char message[224];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($scale) must be of type ?int, %s given",
            function_name,
            position,
            ptn_bcmath_number_arg_type_name(value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return 0;
    }
    int64_t scale = ptn_internal_expect_integer_arg(runtime, function_name, position, "scale", value, line);
    if (runtime->exceptions->active_exception != NULL) {
        return 0;
    }
    if (scale < 0 || scale > INT_MAX) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($scale) must be between 0 and 2147483647",
            function_name,
            position
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    *out = (int)scale;
    return 1;
}

static int ptn_bcmath_number_default_scale_binary(const char *operator, const PtnBcNumber *left, const PtnBcNumber *right) {
    if (strcmp(operator, "+") == 0 || strcmp(operator, "-") == 0) {
        return (int)(left->scale > right->scale ? left->scale : right->scale);
    }
    if (strcmp(operator, "*") == 0) {
        return (int)(left->scale + right->scale);
    }
    if (strcmp(operator, "%") == 0) {
        return (int)left->scale;
    }
    return 0;
}

static PtnValue ptn_bcmath_number_binary_result(
    PtnRuntime *runtime,
    const char *operator,
    const char *function_name,
    const PtnBcNumber *left,
    const PtnBcNumber *right,
    int explicit_scale,
    int scale,
    size_t line
) {
    PtnValue value = ptn_null();
    int object_scale = scale;
    if (strcmp(operator, "+") == 0) {
        object_scale = explicit_scale ? scale : ptn_bcmath_number_default_scale_binary(operator, left, right);
        value = ptn_bc_add_or_sub_values(runtime, left, right, 0, object_scale, line);
    } else if (strcmp(operator, "-") == 0) {
        object_scale = explicit_scale ? scale : ptn_bcmath_number_default_scale_binary(operator, left, right);
        value = ptn_bc_add_or_sub_values(runtime, left, right, 1, object_scale, line);
    } else if (strcmp(operator, "*") == 0) {
        object_scale = explicit_scale ? scale : ptn_bcmath_number_default_scale_binary(operator, left, right);
        value = ptn_bc_mul_value(runtime, left, right, object_scale, line);
    } else if (strcmp(operator, "/") == 0) {
        if (!explicit_scale) {
            int high_scale = (int)left->scale + 10;
            value = ptn_bc_div_value(runtime, function_name, left, right, high_scale, line);
            if (runtime->exceptions->active_exception != NULL) {
                return ptn_null();
            }
            PtnValue trimmed = ptn_bcmath_number_trim_value(value, left->scale, &object_scale);
            ptn_value_destroy(&value);
            value = trimmed;
        } else {
            object_scale = scale;
            value = ptn_bc_div_value(runtime, function_name, left, right, object_scale, line);
        }
    } else if (strcmp(operator, "%") == 0) {
        object_scale = explicit_scale ? scale : (int)left->scale;
        value = ptn_bc_mod_value(runtime, left, right, object_scale, line);
    } else if (strcmp(operator, "**") == 0) {
        int64_t exponent = 0;
        if (!ptn_bc_parse_exponent(runtime, function_name, 1, "exponent", right, 1, &exponent)) {
            return ptn_null();
        }
        if (!explicit_scale) {
            if (exponent >= 0) {
                object_scale = (int)(left->scale * (size_t)exponent);
                value = ptn_bc_pow_value(runtime, left, exponent, object_scale, line);
            } else {
                int high_scale = (int)left->scale + 10;
                value = ptn_bc_pow_value(runtime, left, exponent, high_scale, line);
                if (runtime->exceptions->active_exception != NULL) {
                    return ptn_null();
                }
                PtnValue trimmed = ptn_bcmath_number_trim_value(value, left->scale, &object_scale);
                ptn_value_destroy(&value);
                value = trimmed;
            }
        } else {
            object_scale = scale;
            value = ptn_bc_pow_value(runtime, left, exponent, object_scale, line);
        }
    }
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&value);
        return ptn_null();
    }
    PtnValue object = ptn_bcmath_number_object_from_result(runtime, value, object_scale, line);
    ptn_value_destroy(&value);
    return object;
}

static int ptn_bcmath_number_operator_operand(
    PtnRuntime *runtime,
    PtnValue value,
    int is_left,
    size_t line,
    PtnBcNumber *out
) {
    value = ptn_value_deref(value);
    if (ptn_bcmath_number_is_object(value)) {
        if (!ptn_bcmath_number_read(value, out)) {
            ptn_throw_exception(runtime, "Error", "Invalid BcMath\\Number object");
            return 0;
        }
        return 1;
    }
    switch (value.type) {
        case PTN_BOOL:
            return ptn_bcmath_number_parse_int64(value.as.boolean ? 1 : 0, out);
        case PTN_INT:
            return ptn_bcmath_number_parse_int64(value.as.integer, out);
        case PTN_FLOAT: {
            int64_t integer = ptn_internal_expect_integer_arg(
                runtime,
                "BcMath\\Number operator",
                1,
                "num",
                value,
                line
            );
            if (runtime->exceptions->active_exception != NULL) {
                return 0;
            }
            return ptn_bcmath_number_parse_int64(integer, out);
        }
        case PTN_STRING: {
            PtnStringOperand operand = ptn_string_operand_borrowed_len(
                (const char *)value.as.string.data,
                value.as.string.len
            );
            if (!ptn_bc_parse_number_operand(operand, out)) {
                ptn_throw_exception(
                    runtime,
                    "ValueError",
                    is_left
                        ? "Left string operand cannot be converted to BcMath\\Number"
                        : "Right string operand cannot be converted to BcMath\\Number"
                );
                return 0;
            }
            return 1;
        }
        case PTN_NULL:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PtnValue ptn_bcmath_number_divmod_result(
    PtnRuntime *runtime,
    const PtnBcNumber *left,
    const PtnBcNumber *right,
    int scale,
    size_t line
) {
    if (right->sign == 0) {
        ptn_throw_exception(runtime, "DivisionByZeroError", "Division by zero");
        return ptn_null();
    }
    char *quotient = ptn_bc_div_abs_digits(left, right, 0);
    int q_sign = left->sign == 0 || ptn_bc_digits_is_zero(quotient) ? 0 : left->sign * right->sign;
    PtnValue quotient_value = ptn_bc_format_digits_value(quotient, q_sign, 0, 0);
    PtnValue quotient_object = ptn_bcmath_number_object_from_result(runtime, quotient_value, 0, line);
    ptn_value_destroy(&quotient_value);
    free(quotient);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    PtnValue remainder_value = ptn_bc_mod_value(runtime, left, right, scale, line);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&quotient_object);
        return ptn_null();
    }
    PtnValue remainder_object = ptn_bcmath_number_object_from_result(runtime, remainder_value, scale, line);
    ptn_value_destroy(&remainder_value);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&quotient_object);
        return ptn_null();
    }
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    ptn_array_set_entry(result.as.array, ptn_array_int_key(0), quotient_object);
    ptn_array_set_entry(result.as.array, ptn_array_int_key(1), remainder_object);
    return result;
}

static PTN_UNUSED PtnValue ptn_bcmath_number_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    if (argc != 1) {
        char message[128];
        int written = snprintf(
            message,
            sizeof(message),
            "BcMath\\Number::__construct() expects exactly 1 argument, %zu given",
            argc
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ArgumentCountError", message);
        return ptn_null();
    }
    PtnBcNumber number;
    if (!ptn_bcmath_number_expect_constructor_operand(runtime, args[0], line, &number)) {
        return ptn_null();
    }
    PtnValue object = ptn_bcmath_number_object_from_parsed(runtime, &number, line);
    ptn_bc_number_free(&number);
    return object;
}

static PtnValue ptn_bcmath_number_initialize_from_serialization_data(
    PtnRuntime *runtime,
    PtnValue object,
    PtnValue data,
    size_t line
) {
    PtnBcNumber number;
    if (!ptn_bcmath_number_parse_serialization_data(runtime, data, &number)) {
        return ptn_null();
    }
    PtnValue value = ptn_bc_format_digits_value(number.digits, number.sign, number.scale, number.scale);
    PtnValue result = ptn_bcmath_number_initialize_from_value(runtime, object, value, (int)number.scale, line);
    ptn_value_destroy(&value);
    ptn_bc_number_free(&number);
    return runtime->exceptions->active_exception != NULL ? ptn_null() : result;
}

static int ptn_bcmath_number_argc_between(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    size_t min,
    size_t max
) {
    if (argc >= min && argc <= max) {
        return 1;
    }
    char message[160];
    int written = min == max
        ? snprintf(message, sizeof(message), "%s() expects exactly %zu argument%s, %zu given", function_name, min, min == 1 ? "" : "s", argc)
        : snprintf(message, sizeof(message), "%s() expects between %zu and %zu arguments, %zu given", function_name, min, max, argc);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "ArgumentCountError", message);
    return 0;
}

static PTN_UNUSED PtnValue ptn_bcmath_number_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (ptn_ascii_case_equal(name, "__construct")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::__construct", argc, 1, 1)) {
            return ptn_null();
        }
        PtnBcNumber number;
        if (!ptn_bcmath_number_expect_constructor_operand(runtime, args[0], line, &number)) {
            return ptn_null();
        }
        PtnValue value = ptn_bc_format_digits_value(number.digits, number.sign, number.scale, number.scale);
        PtnValue result = ptn_bcmath_number_initialize_from_value(runtime, receiver, value, (int)number.scale, line);
        ptn_value_destroy(&value);
        ptn_bc_number_free(&number);
        return runtime->exceptions->active_exception != NULL ? ptn_null() : result;
    }
    if (ptn_ascii_case_equal(name, "__toString")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::__toString", argc, 0, 0)) {
            return ptn_null();
        }
        return ptn_bcmath_number_value_clone(receiver);
    }
    if (ptn_ascii_case_equal(name, "__serialize")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::__serialize", argc, 0, 0)) {
            return ptn_null();
        }
        PtnValue result = ptn_array_from_literal_entries(0, NULL);
        ptn_array_set_entry(result.as.array, ptn_array_string_key("value"), ptn_bcmath_number_value_clone(receiver));
        return result;
    }
    if (ptn_ascii_case_equal(name, "__unserialize")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::__unserialize", argc, 1, 1)) {
            return ptn_null();
        }
        PtnValue data = ptn_value_deref(args[0]);
        if (data.type != PTN_ARRAY) {
            char message[192];
            int written = snprintf(
                message,
                sizeof(message),
                "BcMath\\Number::__unserialize(): Argument #1 ($data) must be of type array, %s given",
                ptn_bcmath_number_arg_type_name(data)
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "TypeError", message);
            return ptn_null();
        }
        return ptn_bcmath_number_initialize_from_serialization_data(runtime, receiver, data, line);
    }

    PtnBcNumber left;
    if (!ptn_bcmath_number_read(receiver, &left)) {
        ptn_throw_exception(runtime, "Error", "Invalid BcMath\\Number object");
        return ptn_null();
    }

    if (ptn_ascii_case_equal(name, "add") ||
        ptn_ascii_case_equal(name, "sub") ||
        ptn_ascii_case_equal(name, "mul") ||
        ptn_ascii_case_equal(name, "div") ||
        ptn_ascii_case_equal(name, "mod") ||
        ptn_ascii_case_equal(name, "pow")) {
        char function_name[48];
        int written = snprintf(function_name, sizeof(function_name), "BcMath\\Number::%s", name);
        if (written < 0 || (size_t)written >= sizeof(function_name)) {
            ptn_abort_out_of_memory();
        }
        if (!ptn_bcmath_number_argc_between(runtime, function_name, argc, 1, 2)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        const char *argument_name = ptn_ascii_case_equal(name, "pow") ? "exponent" : "num";
        PtnBcNumber right;
        if (!ptn_bcmath_number_expect_operand(runtime, function_name, 1, argument_name, args[0], line, 1, &right)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        int scale = 0;
        int explicit_scale = argc >= 2 && ptn_value_deref(args[1]).type != PTN_NULL;
        if (argc >= 2 &&
            !ptn_bcmath_number_expect_scale(runtime, function_name, 2, args[1], line, 0, &scale)) {
            ptn_bc_number_free(&left);
            ptn_bc_number_free(&right);
            return ptn_null();
        }
        const char *operator = ptn_ascii_case_equal(name, "add") ? "+"
            : (ptn_ascii_case_equal(name, "sub") ? "-"
            : (ptn_ascii_case_equal(name, "mul") ? "*"
            : (ptn_ascii_case_equal(name, "div") ? "/"
            : (ptn_ascii_case_equal(name, "mod") ? "%" : "**"))));
        PtnValue result = ptn_bcmath_number_binary_result(
            runtime,
            operator,
            function_name,
            &left,
            &right,
            explicit_scale,
            scale,
            line
        );
        ptn_bc_number_free(&left);
        ptn_bc_number_free(&right);
        return result;
    }
    if (ptn_ascii_case_equal(name, "divmod")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::divmod", argc, 1, 2)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        PtnBcNumber right;
        if (!ptn_bcmath_number_expect_operand(runtime, "BcMath\\Number::divmod", 1, "num", args[0], line, 1, &right)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        int scale = (int)left.scale;
        if (argc >= 2 &&
            !ptn_bcmath_number_expect_scale(runtime, "BcMath\\Number::divmod", 2, args[1], line, scale, &scale)) {
            ptn_bc_number_free(&left);
            ptn_bc_number_free(&right);
            return ptn_null();
        }
        PtnValue result = ptn_bcmath_number_divmod_result(runtime, &left, &right, scale, line);
        ptn_bc_number_free(&left);
        ptn_bc_number_free(&right);
        return result;
    }
    if (ptn_ascii_case_equal(name, "powmod")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::powmod", argc, 2, 3)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        PtnBcNumber exponent;
        if (!ptn_bcmath_number_expect_operand(runtime, "BcMath\\Number::powmod", 1, "exponent", args[0], line, 1, &exponent)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        PtnBcNumber modulus;
        if (!ptn_bcmath_number_expect_operand(runtime, "BcMath\\Number::powmod", 2, "modulus", args[1], line, 1, &modulus)) {
            ptn_bc_number_free(&left);
            ptn_bc_number_free(&exponent);
            return ptn_null();
        }
        int scale = 0;
        if (argc >= 3 &&
            !ptn_bcmath_number_expect_scale(runtime, "BcMath\\Number::powmod", 3, args[2], line, 0, &scale)) {
            ptn_bc_number_free(&left);
            ptn_bc_number_free(&exponent);
            ptn_bc_number_free(&modulus);
            return ptn_null();
        }
        PtnValue value = ptn_bc_powmod_value(runtime, &left, &exponent, &modulus, scale, line);
        ptn_bc_number_free(&left);
        ptn_bc_number_free(&exponent);
        ptn_bc_number_free(&modulus);
        if (runtime->exceptions->active_exception != NULL) {
            return ptn_null();
        }
        PtnValue result = ptn_bcmath_number_object_from_result(runtime, value, scale, line);
        ptn_value_destroy(&value);
        return result;
    }
    if (ptn_ascii_case_equal(name, "sqrt")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::sqrt", argc, 0, 1)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        int scale = (int)left.scale + 10;
        if (argc >= 1 &&
            !ptn_bcmath_number_expect_scale(runtime, "BcMath\\Number::sqrt", 1, args[0], line, scale, &scale)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        PtnValue value = ptn_bc_sqrt_value(runtime, &left, scale, line);
        ptn_bc_number_free(&left);
        if (runtime->exceptions->active_exception != NULL) {
            return ptn_null();
        }
        PtnValue result = ptn_bcmath_number_object_from_result(runtime, value, scale, line);
        ptn_value_destroy(&value);
        return result;
    }
    if (ptn_ascii_case_equal(name, "ceil") || ptn_ascii_case_equal(name, "floor")) {
        char function_name[48];
        int written = snprintf(function_name, sizeof(function_name), "BcMath\\Number::%s", name);
        if (written < 0 || (size_t)written >= sizeof(function_name)) {
            ptn_abort_out_of_memory();
        }
        if (!ptn_bcmath_number_argc_between(runtime, function_name, argc, 0, 0)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        PtnValue value = ptn_bc_ceil_floor_value(&left, ptn_ascii_case_equal(name, "ceil"));
        ptn_bc_number_free(&left);
        PtnValue result = ptn_bcmath_number_object_from_result(runtime, value, 0, line);
        ptn_value_destroy(&value);
        return result;
    }
    if (ptn_ascii_case_equal(name, "round")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::round", argc, 0, 2)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        int64_t precision_value = 0;
        if (argc >= 1) {
            precision_value = ptn_internal_expect_integer_arg(runtime, "BcMath\\Number::round", 1, "precision", args[0], line);
            if (runtime->exceptions->active_exception != NULL) {
                ptn_bc_number_free(&left);
                return ptn_null();
            }
            if (precision_value < INT_MIN || precision_value > INT_MAX) {
                char message[160];
                int written = snprintf(
                    message,
                    sizeof(message),
                    "BcMath\\Number::round(): Argument #1 ($precision) must be between %d and %d",
                    INT_MIN,
                    INT_MAX
                );
                if (written < 0 || (size_t)written >= sizeof(message)) {
                    ptn_abort_out_of_memory();
                }
                ptn_bc_number_free(&left);
                ptn_throw_exception(runtime, "ValueError", message);
                return ptn_null();
            }
        }
        const char *mode = argc >= 2 ? ptn_bc_rounding_mode_name(args[1]) : "HalfAwayFromZero";
        PtnValue value = ptn_bc_round_value(&left, (int)precision_value, mode);
        ptn_bc_number_free(&left);
        int scale = precision_value > 0 ? (int)precision_value : 0;
        PtnValue result = ptn_bcmath_number_object_from_result(runtime, value, scale, line);
        ptn_value_destroy(&value);
        return result;
    }
    if (ptn_ascii_case_equal(name, "compare")) {
        if (!ptn_bcmath_number_argc_between(runtime, "BcMath\\Number::compare", argc, 1, 2)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        PtnBcNumber right;
        if (!ptn_bcmath_number_expect_operand(runtime, "BcMath\\Number::compare", 1, "num", args[0], line, 1, &right)) {
            ptn_bc_number_free(&left);
            return ptn_null();
        }
        int scale = (int)(left.scale > right.scale ? left.scale : right.scale);
        if (argc >= 2 &&
            !ptn_bcmath_number_expect_scale(runtime, "BcMath\\Number::compare", 2, args[1], line, scale, &scale)) {
            ptn_bc_number_free(&left);
            ptn_bc_number_free(&right);
            return ptn_null();
        }
        int cmp = ptn_bc_compare_at_scale(&left, &right, (size_t)scale);
        ptn_bc_number_free(&left);
        ptn_bc_number_free(&right);
        return ptn_int(cmp < 0 ? -1 : (cmp > 0 ? 1 : 0));
    }
    ptn_bc_number_free(&left);
    ptn_throw_exception(runtime, "Error", "Call to undefined method BcMath\\Number");
    return ptn_null();
}

static PTN_UNUSED int ptn_bcmath_number_cast_array(PtnValue value, PtnValue *array_out) {
    if (!ptn_bcmath_number_is_object(value)) {
        return 0;
    }
    PtnBcNumber number;
    if (!ptn_bcmath_number_read(value, &number)) {
        return 0;
    }
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    ptn_array_set_entry(result.as.array, ptn_array_string_key("value"), ptn_bcmath_number_value_clone(value));
    ptn_array_set_entry(result.as.array, ptn_array_string_key("scale"), ptn_int((int64_t)number.scale));
    ptn_bc_number_free(&number);
    *array_out = result;
    return 1;
}

static PTN_UNUSED int ptn_bcmath_number_is_truthy(PtnValue value, int *truthy_out) {
    if (!ptn_bcmath_number_is_object(value)) {
        return 0;
    }
    PtnBcNumber number;
    if (!ptn_bcmath_number_read(value, &number)) {
        *truthy_out = 1;
        return 1;
    }
    *truthy_out = number.sign != 0;
    ptn_bc_number_free(&number);
    return 1;
}

static int ptn_bcmath_number_compare_operand(PtnValue value, PtnBcNumber *out) {
    value = ptn_value_deref(value);
    if (ptn_bcmath_number_is_object(value)) {
        return ptn_bcmath_number_read(value, out);
    }
    switch (value.type) {
        case PTN_NULL:
            return ptn_bcmath_number_parse_int64(0, out);
        case PTN_BOOL:
            return ptn_bcmath_number_parse_int64(value.as.boolean ? 1 : 0, out);
        case PTN_INT:
            return ptn_bcmath_number_parse_int64(value.as.integer, out);
        case PTN_FLOAT:
            return ptn_bcmath_number_parse_int64((int64_t)value.as.floating, out);
        case PTN_STRING: {
            PtnStringOperand operand = ptn_string_operand_borrowed_len(
                (const char *)value.as.string.data,
                value.as.string.len
            );
            return ptn_bc_parse_number_operand(operand, out);
        }
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_bcmath_number_compare(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    int *compared
) {
    (void)runtime;
    (void)line;
    if (!ptn_bcmath_number_is_object(left) && !ptn_bcmath_number_is_object(right)) {
        return 0;
    }
    PtnBcNumber left_number;
    if (!ptn_bcmath_number_compare_operand(left, &left_number)) {
        *compared = PTN_COMPARE_UNORDERED;
        return 1;
    }
    PtnBcNumber right_number;
    if (!ptn_bcmath_number_compare_operand(right, &right_number)) {
        ptn_bc_number_free(&left_number);
        *compared = PTN_COMPARE_UNORDERED;
        return 1;
    }
    size_t scale = left_number.scale > right_number.scale ? left_number.scale : right_number.scale;
    int cmp = ptn_bc_compare_at_scale(&left_number, &right_number, scale);
    ptn_bc_number_free(&left_number);
    ptn_bc_number_free(&right_number);
    *compared = cmp < 0 ? PTN_COMPARE_LESS : (cmp > 0 ? PTN_COMPARE_GREATER : PTN_COMPARE_EQUAL);
    return 1;
}

static PTN_UNUSED int ptn_bcmath_number_binary_op(
    PtnRuntime *runtime,
    const char *operator,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnValue *result_out
) {
    if (!ptn_bcmath_number_is_object(left) && !ptn_bcmath_number_is_object(right)) {
        return 0;
    }
    PtnBcNumber left_number;
    if (!ptn_bcmath_number_operator_operand(runtime, left, 1, line, &left_number)) {
        if (runtime->exceptions->active_exception == NULL) {
            ptn_throw_unsupported_operand_types(runtime, left, operator, right, line);
        }
        *result_out = ptn_null();
        return 1;
    }
    PtnBcNumber right_number;
    if (!ptn_bcmath_number_operator_operand(runtime, right, 0, line, &right_number)) {
        ptn_bc_number_free(&left_number);
        if (runtime->exceptions->active_exception == NULL) {
            ptn_throw_unsupported_operand_types(runtime, left, operator, right, line);
        }
        *result_out = ptn_null();
        return 1;
    }
    int explicit_scale = 0;
    int scale = 0;
    if (strcmp(operator, "%") == 0) {
        explicit_scale = 1;
        scale = (int)(left_number.scale > right_number.scale ? left_number.scale : right_number.scale);
    }
    *result_out = ptn_bcmath_number_binary_result(
        runtime,
        operator,
        "BcMath\\Number operator",
        &left_number,
        &right_number,
        explicit_scale,
        scale,
        line
    );
    ptn_bc_number_free(&left_number);
    ptn_bc_number_free(&right_number);
    return 1;
}

static PTN_UNUSED void ptn_bcmath_number_hydrate_unserialized(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (!ptn_bcmath_number_is_object(value)) {
        return;
    }

    PtnArrayEntry *entry = ptn_bcmath_number_property_entry(value.as.object, "value");
    if (value.as.object->properties == NULL || value.as.object->properties->len != 1 || entry == NULL) {
        ptn_bcmath_number_throw_invalid_serialization(runtime);
        return;
    }

    PtnValue property = ptn_value_deref(entry->value);
    if (property.type != PTN_STRING || property.as.string.len == 0) {
        ptn_bcmath_number_throw_invalid_serialization(runtime);
        return;
    }

    PtnStringOperand operand = ptn_string_operand_borrowed_len(
        (const char *)property.as.string.data,
        property.as.string.len
    );
    PtnBcNumber number;
    if (!ptn_bc_parse_number_operand(operand, &number)) {
        ptn_bcmath_number_throw_invalid_serialization(runtime);
        return;
    }

    PtnValue no_value = ptn_null();
    ptn_bcmath_number_declare_readonly_property(
        runtime,
        value,
        "value",
        PTN_PROPERTY_TYPE_STRING,
        no_value,
        0,
        line
    );
    if (runtime->exceptions->active_exception != NULL) {
        ptn_bc_number_free(&number);
        return;
    }

    PtnValue scale = ptn_int((int64_t)number.scale);
    ptn_bcmath_number_declare_readonly_property(
        runtime,
        value,
        "scale",
        PTN_PROPERTY_TYPE_INT,
        scale,
        1,
        line
    );
    ptn_bc_number_free(&number);
}

static PTN_UNUSED int ptn_bcmath_number_inc_dec(
    PtnRuntime *runtime,
    PtnValue value,
    int increment,
    size_t line,
    PtnValue *result_out
) {
    if (!ptn_bcmath_number_is_object(value)) {
        return 0;
    }
    PtnBcNumber number;
    if (!ptn_bcmath_number_read(value, &number)) {
        *result_out = ptn_null();
        ptn_throw_exception(runtime, "Error", "Invalid BcMath\\Number object");
        return 1;
    }
    PtnBcNumber one;
    ptn_bcmath_number_parse_int64(1, &one);
    PtnValue result_value = ptn_bc_add_or_sub_values(
        runtime,
        &number,
        &one,
        increment ? 0 : 1,
        (int)number.scale,
        line
    );
    int scale = (int)number.scale;
    ptn_bc_number_free(&one);
    ptn_bc_number_free(&number);
    *result_out = ptn_bcmath_number_object_from_result(runtime, result_value, scale, line);
    ptn_value_destroy(&result_value);
    return 1;
}

static PtnValue ptn_internal_bcadd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber left, right;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcadd", 1, "num1", args[0], line, &left)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcadd", 2, "num2", args[1], line, &right)) { ptn_bc_number_free(&left); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bcadd", 3, args[2], line, &scale)) { ptn_bc_number_free(&left); ptn_bc_number_free(&right); return ptn_null(); }
    PtnValue result = ptn_bc_add_or_sub_values(runtime, &left, &right, 0, scale, line);
    ptn_bc_number_free(&left);
    ptn_bc_number_free(&right);
    return result;
}

static PtnValue ptn_internal_bcsub(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber left, right;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcsub", 1, "num1", args[0], line, &left)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcsub", 2, "num2", args[1], line, &right)) { ptn_bc_number_free(&left); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bcsub", 3, args[2], line, &scale)) { ptn_bc_number_free(&left); ptn_bc_number_free(&right); return ptn_null(); }
    PtnValue result = ptn_bc_add_or_sub_values(runtime, &left, &right, 1, scale, line);
    ptn_bc_number_free(&left);
    ptn_bc_number_free(&right);
    return result;
}

static PtnValue ptn_internal_bcmul(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber left, right;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcmul", 1, "num1", args[0], line, &left)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcmul", 2, "num2", args[1], line, &right)) { ptn_bc_number_free(&left); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bcmul", 3, args[2], line, &scale)) { ptn_bc_number_free(&left); ptn_bc_number_free(&right); return ptn_null(); }
    PtnValue result = ptn_bc_mul_value(runtime, &left, &right, scale, line);
    ptn_bc_number_free(&left);
    ptn_bc_number_free(&right);
    return result;
}

static PtnValue ptn_internal_bcdiv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber left, right;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcdiv", 1, "num1", args[0], line, &left)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcdiv", 2, "num2", args[1], line, &right)) { ptn_bc_number_free(&left); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bcdiv", 3, args[2], line, &scale)) { ptn_bc_number_free(&left); ptn_bc_number_free(&right); return ptn_null(); }
    PtnValue result = ptn_bc_div_value(runtime, "bcdiv", &left, &right, scale, line);
    ptn_bc_number_free(&left);
    ptn_bc_number_free(&right);
    return result;
}

static PtnValue ptn_internal_bccomp(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber left, right;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bccomp", 1, "num1", args[0], line, &left)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bccomp", 2, "num2", args[1], line, &right)) { ptn_bc_number_free(&left); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bccomp", 3, args[2], line, &scale)) { ptn_bc_number_free(&left); ptn_bc_number_free(&right); return ptn_null(); }
    int cmp = ptn_bc_compare_at_scale(&left, &right, (size_t)scale);
    ptn_bc_number_free(&left);
    ptn_bc_number_free(&right);
    return ptn_int(cmp < 0 ? -1 : (cmp > 0 ? 1 : 0));
}

static PtnValue ptn_internal_bcmod(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber left, right;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcmod", 1, "num1", args[0], line, &left)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcmod", 2, "num2", args[1], line, &right)) { ptn_bc_number_free(&left); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bcmod", 3, args[2], line, &scale)) { ptn_bc_number_free(&left); ptn_bc_number_free(&right); return ptn_null(); }
    PtnValue result = ptn_bc_mod_value(runtime, &left, &right, scale, line);
    ptn_bc_number_free(&left);
    ptn_bc_number_free(&right);
    return result;
}

static PtnValue ptn_internal_bcdivmod(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber left, right;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcdivmod", 1, "num1", args[0], line, &left)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcdivmod", 2, "num2", args[1], line, &right)) { ptn_bc_number_free(&left); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bcdivmod", 3, args[2], line, &scale)) { ptn_bc_number_free(&left); ptn_bc_number_free(&right); return ptn_null(); }
    if (right.sign == 0) {
        ptn_bc_number_free(&left);
        ptn_bc_number_free(&right);
        ptn_throw_exception(runtime, "DivisionByZeroError", "Division by zero");
        return ptn_null();
    }
    char *quotient = ptn_bc_div_abs_digits(&left, &right, 0);
    int q_sign = left.sign == 0 || ptn_bc_digits_is_zero(quotient) ? 0 : left.sign * right.sign;
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    ptn_array_set_entry(result.as.array, ptn_array_int_key(0), ptn_bc_format_digits_value(quotient, q_sign, 0, 0));
    PtnValue remainder = ptn_bc_mod_value(runtime, &left, &right, scale, line);
    ptn_array_set_entry(result.as.array, ptn_array_int_key(1), remainder);
    free(quotient);
    ptn_bc_number_free(&left);
    ptn_bc_number_free(&right);
    return result;
}

static PtnValue ptn_internal_bcpow(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber base, exponent;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcpow", 1, "num", args[0], line, &base)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcpow", 2, "exponent", args[1], line, &exponent)) { ptn_bc_number_free(&base); return ptn_null(); }
    if (argc >= 3 && !ptn_bc_optional_scale(runtime, "bcpow", 3, args[2], line, &scale)) { ptn_bc_number_free(&base); ptn_bc_number_free(&exponent); return ptn_null(); }
    int64_t exp_value = 0;
    if (!ptn_bc_parse_exponent(runtime, "bcpow", 2, "exponent", &exponent, 1, &exp_value)) { ptn_bc_number_free(&base); ptn_bc_number_free(&exponent); return ptn_null(); }
    PtnValue result = ptn_bc_pow_value(runtime, &base, exp_value, scale, line);
    ptn_bc_number_free(&base);
    ptn_bc_number_free(&exponent);
    return result;
}

static PtnValue ptn_internal_bcpowmod(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber base, exponent, modulus;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcpowmod", 1, "num", args[0], line, &base)) return ptn_null();
    if (!ptn_bc_expect_number(runtime, "bcpowmod", 2, "exponent", args[1], line, &exponent)) { ptn_bc_number_free(&base); return ptn_null(); }
    if (!ptn_bc_expect_number(runtime, "bcpowmod", 3, "modulus", args[2], line, &modulus)) { ptn_bc_number_free(&base); ptn_bc_number_free(&exponent); return ptn_null(); }
    if (argc >= 4 && !ptn_bc_optional_scale(runtime, "bcpowmod", 4, args[3], line, &scale)) { ptn_bc_number_free(&base); ptn_bc_number_free(&exponent); ptn_bc_number_free(&modulus); return ptn_null(); }
    PtnValue result = ptn_bc_powmod_value(runtime, &base, &exponent, &modulus, scale, line);
    ptn_bc_number_free(&base);
    ptn_bc_number_free(&exponent);
    ptn_bc_number_free(&modulus);
    return result;
}

static PtnValue ptn_internal_bcsqrt(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber number;
    int scale = ptn_bc_current_scale(runtime);
    if (!ptn_bc_expect_number(runtime, "bcsqrt", 1, "num", args[0], line, &number)) return ptn_null();
    if (argc >= 2 && !ptn_bc_optional_scale(runtime, "bcsqrt", 2, args[1], line, &scale)) { ptn_bc_number_free(&number); return ptn_null(); }
    PtnValue result = ptn_bc_sqrt_value(runtime, &number, scale, line);
    ptn_bc_number_free(&number);
    return result;
}

static PtnValue ptn_internal_bcceil(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnBcNumber number;
    if (!ptn_bc_expect_number(runtime, "bcceil", 1, "num", args[0], line, &number)) return ptn_null();
    PtnValue result = ptn_bc_ceil_floor_value(&number, 1);
    ptn_bc_number_free(&number);
    return result;
}

static PtnValue ptn_internal_bcfloor(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnBcNumber number;
    if (!ptn_bc_expect_number(runtime, "bcfloor", 1, "num", args[0], line, &number)) return ptn_null();
    PtnValue result = ptn_bc_ceil_floor_value(&number, 0);
    ptn_bc_number_free(&number);
    return result;
}

static PtnValue ptn_internal_bcround(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnBcNumber number;
    if (!ptn_bc_expect_number(runtime, "bcround", 1, "num", args[0], line, &number)) return ptn_null();
    int64_t precision_value = 0;
    if (argc >= 2) {
        precision_value = ptn_internal_expect_integer_arg(runtime, "bcround", 2, "precision", args[1], line);
        if (runtime->exceptions->active_exception != NULL) {
            ptn_bc_number_free(&number);
            return ptn_null();
        }
        if (precision_value < INT_MIN || precision_value > INT_MAX) {
            char message[144];
            int written = snprintf(
                message,
                sizeof(message),
                "bcround(): Argument #2 ($precision) must be between %d and %d",
                INT_MIN,
                INT_MAX
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_bc_number_free(&number);
            ptn_throw_exception(runtime, "ValueError", message);
            return ptn_null();
        }
    }
    const char *mode = argc >= 3 ? ptn_bc_rounding_mode_name(args[2]) : "HalfAwayFromZero";
    PtnValue result = ptn_bc_round_value(&number, (int)precision_value, mode);
    ptn_bc_number_free(&number);
    return result;
}

static PtnValue ptn_internal_bcscale(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int previous = ptn_bc_current_scale(runtime);
    if (argc == 0 || ptn_value_deref(args[0]).type == PTN_NULL) {
        return ptn_int(previous);
    }
    int scale = previous;
    if (!ptn_bc_expect_scale(runtime, "bcscale", 1, "scale", args[0], line, previous, &scale)) {
        return ptn_null();
    }
    ptn_bc_set_current_scale(runtime, scale);
    return ptn_int(previous);
}

static PtnValue ptn_internal_rounding_mode_cases(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)args;
    (void)line;
    if (argc != 0) {
        char message[128];
        int written = snprintf(message, sizeof(message), "RoundingMode::cases() expects exactly 0 arguments, %zu given", argc);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ArgumentCountError", message);
        return ptn_null();
    }
    static const char *const names[] = {
        "HalfAwayFromZero",
        "HalfTowardsZero",
        "HalfEven",
        "HalfOdd",
        "TowardsZero",
        "AwayFromZero",
        "NegativeInfinity",
        "PositiveInfinity",
    };
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key((int64_t)i),
            ptn_enum_case(runtime, "RoundingMode", names[i])
        );
    }
    return result;
}

#define PTN_CAL_GREGORIAN 0
#define PTN_CAL_JULIAN 1
#define PTN_CAL_JEWISH 2
#define PTN_CAL_FRENCH 3
#define PTN_CAL_NUM_CALS 4
#define PTN_CAL_DOW_DAYNO 0
#define PTN_CAL_DOW_LONG 1
#define PTN_CAL_DOW_SHORT 2
#define PTN_CAL_MONTH_GREGORIAN_SHORT 0
#define PTN_CAL_MONTH_GREGORIAN_LONG 1
#define PTN_CAL_MONTH_JULIAN_SHORT 2
#define PTN_CAL_MONTH_JULIAN_LONG 3
#define PTN_CAL_MONTH_JEWISH 4
#define PTN_CAL_MONTH_FRENCH 5
#define PTN_CAL_EASTER_DEFAULT 0
#define PTN_CAL_EASTER_ROMAN 1
#define PTN_CAL_EASTER_ALWAYS_GREGORIAN 2
#define PTN_CAL_EASTER_ALWAYS_JULIAN 3
#define PTN_CAL_JEWISH_ADD_ALAFIM_GERESH 2
#define PTN_CAL_JEWISH_ADD_ALAFIM 4
#define PTN_CAL_JEWISH_ADD_GERESHAYIM 8

static const char *const PTN_CAL_GREGORIAN_MONTHS[] = {
    "", "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
};
static const char *const PTN_CAL_GREGORIAN_ABBREV_MONTHS[] = {
    "", "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
};
static const char *const PTN_CAL_JEWISH_MONTHS[] = {
    "", "Tishri", "Heshvan", "Kislev", "Tevet", "Shevat", "Adar I",
    "Adar II", "Nisan", "Iyyar", "Sivan", "Tammuz", "Av", "Elul"
};
static const char *const PTN_CAL_JEWISH_MONTHS_REGULAR[] = {
    "", "Tishri", "Heshvan", "Kislev", "Tevet", "Shevat", "",
    "Adar", "Nisan", "Iyyar", "Sivan", "Tammuz", "Av", "Elul"
};
static const char *const PTN_CAL_JEWISH_HEBREW_MONTHS_NORMAL[] = {
    "",
    "\xfa\xf9\xf8\xe9",
    "\xe7\xf9\xe5\xef",
    "\xeb\xf1\xec\xe5",
    "\xe8\xe1\xfa",
    "\xf9\xe1\xe8",
    "\xe0\xe3\xf8 \xe0'",
    "\xe0\xe3\xf8 \xe1'",
    "\xf0\xe9\xf1\xef",
    "\xe0\xe9\xe9\xf8",
    "\xf1\xe9\xe5\xef",
    "\xfa\xee\xe5\xe6",
    "\xe0\xe1",
    "\xe0\xec\xe5\xec"
};
static const char *const PTN_CAL_JEWISH_HEBREW_MONTHS_LEAP[] = {
    "",
    "\xfa\xf9\xf8\xe9",
    "\xe7\xf9\xe5\xef",
    "\xeb\xf1\xec\xe5",
    "\xe8\xe1\xfa",
    "\xf9\xe1\xe8",
    "\xe0\xe3\xf8",
    "\xe0\xe3\xf8",
    "\xf0\xe9\xf1\xef",
    "\xe0\xe9\xe9\xf8",
    "\xf1\xe9\xe5\xef",
    "\xfa\xee\xe5\xe6",
    "\xe0\xe1",
    "\xe0\xec\xe5\xec"
};
static const char *const PTN_CAL_FRENCH_MONTHS[] = {
    "", "Vendemiaire", "Brumaire", "Frimaire", "Nivose", "Pluviose", "Ventose",
    "Germinal", "Floreal", "Prairial", "Messidor", "Thermidor", "Fructidor", "Extra"
};
static const char *const PTN_CAL_DAY_NAMES[] = {
    "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"
};
static const char *const PTN_CAL_ABBREV_DAY_NAMES[] = {
    "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
};

static int ptn_cal_valid_calendar(int64_t calendar) {
    return calendar >= 0 && calendar < PTN_CAL_NUM_CALS;
}

static void ptn_cal_throw_range_between(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    int64_t min,
    int64_t max
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be between %lld and %lld",
        function_name,
        position,
        argument_name,
        (long long)min,
        (long long)max
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "ValueError", message);
}

static void ptn_cal_throw_range_less_than(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    int64_t max_exclusive
) {
    char message[176];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be less than %lld",
        function_name,
        position,
        argument_name,
        (long long)max_exclusive
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "ValueError", message);
}

static int ptn_cal_validate_month_argument(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    int64_t month
) {
    if (month < 1 || month >= INT_MAX - 1) {
        ptn_cal_throw_range_between(runtime, function_name, position, "month", 1, (int64_t)INT_MAX - 1);
        return 0;
    }
    return 1;
}

static int ptn_cal_validate_day_argument(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    int64_t day
) {
    if (day < INT_MIN || day > INT_MAX) {
        ptn_cal_throw_range_between(runtime, function_name, position, "day", INT_MIN, INT_MAX);
        return 0;
    }
    return 1;
}

static int ptn_cal_validate_year_less_than_max(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    int64_t year
) {
    if (year >= INT_MAX - 1) {
        ptn_cal_throw_range_less_than(runtime, function_name, position, "year", (int64_t)INT_MAX - 1);
        return 0;
    }
    return 1;
}

static int ptn_cal_validate_jewish_year_argument(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    int64_t year
) {
    if (year < 1 || year >= INT_MAX - 1) {
        ptn_cal_throw_range_between(runtime, function_name, position, "year", 1, (int64_t)INT_MAX - 1);
        return 0;
    }
    return 1;
}

static int ptn_cal_day_of_week(int64_t jd) {
    int64_t dow = (jd + 1) % 7;
    if (dow < 0) {
        dow += 7;
    }
    return (int)dow;
}

static int64_t ptn_cal_gregorian_to_jd(int64_t month, int64_t day, int64_t year) {
    month = (int32_t)month;
    day = (int32_t)day;
    year = (int32_t)year;
    if (month < 1 || month > 12 || day < 1 || day > 31 || year == 0) {
        return 0;
    }
    if (year < 0) {
        year++;
    }
    int64_t a = (14 - month) / 12;
    int64_t y = year + 4800 - a;
    int64_t m = month + 12 * a - 3;
    int64_t jd = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    return jd < 1 ? 0 : jd;
}

static void ptn_cal_jd_to_gregorian(int64_t jd, int64_t *month, int64_t *day, int64_t *year) {
    if (jd <= 0 || jd > (((int64_t)INT_MAX * 146097LL) / 4LL - 32045LL)) {
        *month = 0;
        *day = 0;
        *year = 0;
        return;
    }
    int64_t a = jd + 32044;
    int64_t b = (4 * a + 3) / 146097;
    int64_t c = a - (146097 * b) / 4;
    int64_t d = (4 * c + 3) / 1461;
    int64_t e = c - (1461 * d) / 4;
    int64_t m = (5 * e + 2) / 153;
    *day = e - (153 * m + 2) / 5 + 1;
    *month = m + 3 - 12 * (m / 10);
    int64_t computed_year = 100 * b + d - 4800 + m / 10;
    if (computed_year > INT_MAX || computed_year < INT_MIN) {
        *month = 0;
        *day = 0;
        *year = 0;
        return;
    }
    *year = computed_year;
    if (*year <= 0) {
        (*year)--;
    }
}

static int64_t ptn_cal_julian_to_jd(int64_t month, int64_t day, int64_t year) {
    month = (int32_t)month;
    day = (int32_t)day;
    year = (int32_t)year;
    if (month < 1 || month > 12 || day < 1 || day > 31 || year == 0) {
        return 0;
    }
    if (year < 0) {
        year++;
    }
    int64_t a = (14 - month) / 12;
    int64_t y = year + 4800 - a;
    int64_t m = month + 12 * a - 3;
    int64_t jd = day + (153 * m + 2) / 5 + 365 * y + y / 4 - 32083;
    return jd < 1 ? 0 : jd;
}

static void ptn_cal_jd_to_julian(int64_t jd, int64_t *month, int64_t *day, int64_t *year) {
    if (jd <= 0 || jd > ((((int64_t)INT_MAX * 1461LL) - (32083LL * 4LL - 1LL)) / 4LL)) {
        *month = 0;
        *day = 0;
        *year = 0;
        return;
    }
    int64_t c = jd + 32082;
    int64_t d = (4 * c + 3) / 1461;
    int64_t e = c - (1461 * d) / 4;
    int64_t m = (5 * e + 2) / 153;
    *day = e - (153 * m + 2) / 5 + 1;
    *month = m + 3 - 12 * (m / 10);
    int64_t computed_year = d - 4800 + m / 10;
    if (computed_year > INT_MAX || computed_year < INT_MIN) {
        *month = 0;
        *day = 0;
        *year = 0;
        return;
    }
    *year = computed_year;
    if (*year <= 0) {
        (*year)--;
    }
}

#define PTN_HEBREW_HALAKIM_PER_HOUR 1080LL
#define PTN_HEBREW_HALAKIM_PER_DAY 25920LL
#define PTN_HEBREW_HALAKIM_PER_LUNAR_CYCLE ((29LL * PTN_HEBREW_HALAKIM_PER_DAY) + 13753LL)
#define PTN_HEBREW_HALAKIM_PER_METONIC_CYCLE (PTN_HEBREW_HALAKIM_PER_LUNAR_CYCLE * (12LL * 19LL + 7LL))
#define PTN_JEWISH_SDN_OFFSET 347997LL
#define PTN_JEWISH_SDN_MAX 324542846LL
#define PTN_HEBREW_NEW_MOON_OF_CREATION 31524LL
#define PTN_HEBREW_NOON (18LL * PTN_HEBREW_HALAKIM_PER_HOUR)
#define PTN_HEBREW_AM3_11_20 ((9LL * PTN_HEBREW_HALAKIM_PER_HOUR) + 204LL)
#define PTN_HEBREW_AM9_32_43 ((15LL * PTN_HEBREW_HALAKIM_PER_HOUR) + 589LL)

static const int PTN_HEBREW_MONTHS_PER_YEAR[19] = {
    12, 12, 13, 12, 12, 13, 12, 13, 12, 12, 13, 12, 12, 13, 12, 12, 13, 12, 13
};

static const int PTN_HEBREW_YEAR_OFFSET[19] = {
    0, 12, 24, 37, 49, 61, 74, 86, 99, 111, 123, 136, 148, 160, 173, 185, 197, 210, 222
};

static int ptn_cal_hebrew_leap(int64_t year) {
    if (year <= 0) {
        return 0;
    }
    return PTN_HEBREW_MONTHS_PER_YEAR[(year - 1) % 19] == 13;
}

static const char *const *ptn_cal_jewish_month_names(int64_t year) {
    return ptn_cal_hebrew_leap(year) ? PTN_CAL_JEWISH_MONTHS : PTN_CAL_JEWISH_MONTHS_REGULAR;
}

static int64_t ptn_cal_positive_mod(int64_t value, int64_t modulus) {
    int64_t result = value % modulus;
    return result < 0 ? result + modulus : result;
}

static int64_t ptn_cal_hebrew_tishri1(int metonic_year, int64_t molad_day, int64_t molad_halakim) {
    int64_t tishri1 = molad_day;
    int dow = (int)ptn_cal_positive_mod(tishri1, 7);
    int leap_year = metonic_year == 2 || metonic_year == 5 || metonic_year == 7 ||
        metonic_year == 10 || metonic_year == 13 || metonic_year == 16 || metonic_year == 18;
    int last_was_leap = metonic_year == 3 || metonic_year == 6 || metonic_year == 8 ||
        metonic_year == 11 || metonic_year == 14 || metonic_year == 17 || metonic_year == 0;

    if (molad_halakim >= PTN_HEBREW_NOON ||
        (!leap_year && dow == 2 && molad_halakim >= PTN_HEBREW_AM3_11_20) ||
        (last_was_leap && dow == 1 && molad_halakim >= PTN_HEBREW_AM9_32_43)) {
        tishri1++;
        dow++;
        if (dow == 7) {
            dow = 0;
        }
    }
    if (dow == 3 || dow == 5 || dow == 0) {
        tishri1++;
    }
    return tishri1;
}

static void ptn_cal_hebrew_molad_of_metonic_cycle(int64_t metonic_cycle, int64_t *molad_day, int64_t *molad_halakim) {
    int64_t halakim = PTN_HEBREW_NEW_MOON_OF_CREATION + metonic_cycle * PTN_HEBREW_HALAKIM_PER_METONIC_CYCLE;
    *molad_day = halakim / PTN_HEBREW_HALAKIM_PER_DAY;
    *molad_halakim = halakim % PTN_HEBREW_HALAKIM_PER_DAY;
}

static void ptn_cal_hebrew_start_of_year(int64_t year, int64_t *molad_day, int64_t *molad_halakim, int64_t *tishri1) {
    int64_t metonic_cycle = (year - 1) / 19;
    int metonic_year = (int)((year - 1) % 19);
    ptn_cal_hebrew_molad_of_metonic_cycle(metonic_cycle, molad_day, molad_halakim);
    *molad_halakim += PTN_HEBREW_HALAKIM_PER_LUNAR_CYCLE * PTN_HEBREW_YEAR_OFFSET[metonic_year];
    *molad_day += *molad_halakim / PTN_HEBREW_HALAKIM_PER_DAY;
    *molad_halakim %= PTN_HEBREW_HALAKIM_PER_DAY;
    *tishri1 = ptn_cal_hebrew_tishri1(metonic_year, *molad_day, *molad_halakim);
}

static int64_t ptn_cal_jewish_to_jd(int64_t month, int64_t day, int64_t year) {
    month = (int32_t)month;
    day = (int32_t)day;
    year = (int32_t)year;
    if (year <= 0 || year >= INT_MAX - 1 || day <= 0 || day > 30) {
        return 0;
    }

    int64_t molad_day = 0;
    int64_t molad_halakim = 0;
    int64_t tishri1 = 0;
    int64_t tishri1_after = 0;
    int metonic_year = (int)((year - 1) % 19);
    int64_t sdn = 0;

    switch (month) {
        case 1:
        case 2:
            ptn_cal_hebrew_start_of_year(year, &molad_day, &molad_halakim, &tishri1);
            sdn = month == 1 ? tishri1 + day - 1 : tishri1 + day + 29;
            break;
        case 3:
            ptn_cal_hebrew_start_of_year(year, &molad_day, &molad_halakim, &tishri1);
            molad_halakim += PTN_HEBREW_HALAKIM_PER_LUNAR_CYCLE * PTN_HEBREW_MONTHS_PER_YEAR[metonic_year];
            molad_day += molad_halakim / PTN_HEBREW_HALAKIM_PER_DAY;
            molad_halakim %= PTN_HEBREW_HALAKIM_PER_DAY;
            tishri1_after = ptn_cal_hebrew_tishri1((metonic_year + 1) % 19, molad_day, molad_halakim);
            sdn = (tishri1_after - tishri1 == 355 || tishri1_after - tishri1 == 385) ?
                tishri1 + day + 59 : tishri1 + day + 58;
            break;
        case 4:
        case 5:
        case 6: {
            ptn_cal_hebrew_start_of_year(year + 1, &molad_day, &molad_halakim, &tishri1_after);
            int length_of_adar = PTN_HEBREW_MONTHS_PER_YEAR[(year - 1) % 19] == 12 ? 29 : 59;
            if (month == 4) {
                sdn = tishri1_after + day - length_of_adar - 237;
            } else if (month == 5) {
                sdn = tishri1_after + day - length_of_adar - 208;
            } else {
                sdn = tishri1_after + day - length_of_adar - 178;
            }
            break;
        }
        case 7:
        case 8:
        case 9:
        case 10:
        case 11:
        case 12:
        case 13:
            ptn_cal_hebrew_start_of_year(year + 1, &molad_day, &molad_halakim, &tishri1_after);
            if (month == 7) sdn = tishri1_after + day - 207;
            else if (month == 8) sdn = tishri1_after + day - 178;
            else if (month == 9) sdn = tishri1_after + day - 148;
            else if (month == 10) sdn = tishri1_after + day - 119;
            else if (month == 11) sdn = tishri1_after + day - 89;
            else if (month == 12) sdn = tishri1_after + day - 60;
            else sdn = tishri1_after + day - 30;
            break;
        default:
            return 0;
    }

    return sdn + PTN_JEWISH_SDN_OFFSET;
}

static void ptn_cal_jd_to_jewish(int64_t jd, int64_t *month, int64_t *day, int64_t *year) {
    if (jd <= PTN_JEWISH_SDN_OFFSET || jd > PTN_JEWISH_SDN_MAX) {
        *month = 0;
        *day = 0;
        *year = 0;
        return;
    }

    int64_t y = (jd - PTN_JEWISH_SDN_OFFSET) / 366 + 1;
    if (y < 1) {
        y = 1;
    }
    while (ptn_cal_jewish_to_jd(1, 1, y + 1) <= jd) {
        y++;
    }
    while (ptn_cal_jewish_to_jd(1, 1, y) > jd) {
        y--;
    }

    for (int m = 1; m <= 13; m++) {
        int64_t start = ptn_cal_jewish_to_jd(m, 1, y);
        int64_t next = m < 13 ? ptn_cal_jewish_to_jd(m + 1, 1, y) : ptn_cal_jewish_to_jd(1, 1, y + 1);
        if (start != 0 && next != 0 && start <= jd && jd < next) {
            *month = m;
            *day = jd - start + 1;
            *year = y;
            return;
        }
    }

    *month = 0;
    *day = 0;
    *year = 0;
}

static int ptn_cal_french_leap(int64_t year) {
    return year == 3 || year == 7 || year == 11;
}

static int64_t ptn_cal_french_to_jd(int64_t month, int64_t day, int64_t year) {
    month = (int32_t)month;
    day = (int32_t)day;
    year = (int32_t)year;
    if (year < 1 || year > 14 || month < 1 || month > 13 || day < 1) {
        return 0;
    }
    int max_day = month == 13 ? (ptn_cal_french_leap(year) ? 6 : 5) : 30;
    if (day > max_day) {
        return 0;
    }
    int64_t leap_days_before = (year + 1) / 4;
    return 2375840 + (year - 1) * 365 + leap_days_before + (month - 1) * 30 + (day - 1);
}

static void ptn_cal_jd_to_french(int64_t jd, int64_t *month, int64_t *day, int64_t *year) {
    if (jd < 2375840) {
        *month = 0;
        *day = 0;
        *year = 0;
        return;
    }
    int64_t remaining = jd - 2375840;
    int64_t y = 1;
    while (y <= 14) {
        int ydays = 365 + (ptn_cal_french_leap(y) ? 1 : 0);
        if (remaining < ydays) {
            *year = y;
            *month = remaining / 30 + 1;
            *day = remaining % 30 + 1;
            if (*month > 13) {
                *month = 13;
            }
            return;
        }
        remaining -= ydays;
        y++;
    }
    *month = 0;
    *day = 0;
    *year = 0;
}

static PtnValue ptn_cal_date_string(int64_t month, int64_t day, int64_t year) {
    char buffer[128];
    int written = snprintf(buffer, sizeof(buffer), "%lld/%lld/%lld", (long long)month, (long long)day, (long long)year);
    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_owned_string(ptn_duplicate_string(buffer));
}

static void ptn_cal_hebrew_append_byte(char *buffer, size_t *pos, unsigned char byte) {
    buffer[(*pos)++] = (char)byte;
}

static void ptn_cal_hebrew_append_bytes(char *buffer, size_t *pos, const char *bytes) {
    for (const unsigned char *p = (const unsigned char *)bytes; *p != '\0'; p++) {
        ptn_cal_hebrew_append_byte(buffer, pos, *p);
    }
}

static size_t ptn_cal_hebrew_under_1000_bytes(int value, char *out) {
    static const char *const units[] = {
        "", "\xe0", "\xe1", "\xe2", "\xe3", "\xe4", "\xe5", "\xe6", "\xe7", "\xe8"
    };
    static const char *const tens[] = {
        "", "\xe9", "\xeb", "\xec", "\xee", "\xf0", "\xf1", "\xf2", "\xf4", "\xf6"
    };
    static const char *const hundreds[] = {
        "", "\xf7", "\xf8", "\xf9"
    };
    size_t pos = 0;
    while (value >= 400) {
        ptn_cal_hebrew_append_bytes(out, &pos, "\xfa");
        value -= 400;
    }
    if (value >= 100) {
        int hundred = value / 100;
        ptn_cal_hebrew_append_bytes(out, &pos, hundreds[hundred]);
        value %= 100;
    }
    if (value == 15) {
        ptn_cal_hebrew_append_bytes(out, &pos, "\xe8\xe5");
    } else if (value == 16) {
        ptn_cal_hebrew_append_bytes(out, &pos, "\xe8\xe6");
    } else {
        if (value >= 10) {
            int ten = value / 10;
            ptn_cal_hebrew_append_bytes(out, &pos, tens[ten]);
            value %= 10;
        }
        if (value > 0) {
            ptn_cal_hebrew_append_bytes(out, &pos, units[value]);
        }
    }
    out[pos] = '\0';
    return pos;
}

static void ptn_cal_hebrew_append_numeral(char *buffer, size_t *pos, int value, int add_gereshayim) {
    char numeral[32];
    size_t len = ptn_cal_hebrew_under_1000_bytes(value, numeral);
    if (len == 0) {
        return;
    }
    if (!add_gereshayim) {
        memcpy(buffer + *pos, numeral, len);
        *pos += len;
        return;
    }
    if (len == 1) {
        memcpy(buffer + *pos, numeral, len);
        *pos += len;
        ptn_cal_hebrew_append_byte(buffer, pos, '\'');
        return;
    }
    memcpy(buffer + *pos, numeral, len - 1);
    *pos += len - 1;
    ptn_cal_hebrew_append_byte(buffer, pos, '"');
    ptn_cal_hebrew_append_byte(buffer, pos, (unsigned char)numeral[len - 1]);
}

static void ptn_cal_hebrew_append_year(char *buffer, size_t *pos, int year, int flags) {
    int thousands = year / 1000;
    int remainder = year % 1000;
    int add_alafim = (flags & PTN_CAL_JEWISH_ADD_ALAFIM) != 0;
    int add_alafim_geresh = (flags & PTN_CAL_JEWISH_ADD_ALAFIM_GERESH) != 0;
    int add_gereshayim = (flags & PTN_CAL_JEWISH_ADD_GERESHAYIM) != 0;
    if (thousands > 0) {
        ptn_cal_hebrew_append_numeral(buffer, pos, thousands, 0);
        if (add_alafim_geresh) {
            ptn_cal_hebrew_append_byte(buffer, pos, '\'');
        }
        if (add_alafim) {
            ptn_cal_hebrew_append_byte(buffer, pos, ' ');
            ptn_cal_hebrew_append_bytes(buffer, pos, "\xe0\xec\xf4\xe9\xed");
            if (remainder > 0) {
                ptn_cal_hebrew_append_byte(buffer, pos, ' ');
            }
        }
    }
    if (remainder > 0 || thousands == 0) {
        ptn_cal_hebrew_append_numeral(buffer, pos, remainder, add_gereshayim);
    }
}

static PtnValue ptn_cal_jewish_hebrew_date_string(int64_t month, int64_t day, int64_t year, int flags) {
    char buffer[256];
    size_t pos = 0;
    ptn_cal_hebrew_append_numeral(buffer, &pos, (int)day, (flags & PTN_CAL_JEWISH_ADD_GERESHAYIM) != 0);
    ptn_cal_hebrew_append_byte(buffer, &pos, ' ');
    const char *const *month_names = ptn_cal_hebrew_leap(year)
        ? PTN_CAL_JEWISH_HEBREW_MONTHS_NORMAL
        : PTN_CAL_JEWISH_HEBREW_MONTHS_LEAP;
    if (month >= 1 && month <= 13) {
        ptn_cal_hebrew_append_bytes(buffer, &pos, month_names[month]);
    }
    ptn_cal_hebrew_append_byte(buffer, &pos, ' ');
    ptn_cal_hebrew_append_year(buffer, &pos, (int)year, flags);
    buffer[pos] = '\0';
    return ptn_owned_string_len(ptn_bc_duplicate_range(buffer, pos), pos);
}

static PtnValue ptn_cal_info_for_calendar(int calendar) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    PtnValue months = ptn_array_from_literal_entries(0, NULL);
    PtnValue abbrev = ptn_array_from_literal_entries(0, NULL);
    const char *name = "Gregorian";
    const char *symbol = "CAL_GREGORIAN";
    int max = 12;
    int maxdays = 31;
    const char *const *full = PTN_CAL_GREGORIAN_MONTHS;
    const char *const *shorts = PTN_CAL_GREGORIAN_ABBREV_MONTHS;
    if (calendar == PTN_CAL_JULIAN) {
        name = "Julian";
        symbol = "CAL_JULIAN";
    } else if (calendar == PTN_CAL_JEWISH) {
        name = "Jewish";
        symbol = "CAL_JEWISH";
        max = 13;
        maxdays = 30;
        full = PTN_CAL_JEWISH_MONTHS;
        shorts = PTN_CAL_JEWISH_MONTHS;
    } else if (calendar == PTN_CAL_FRENCH) {
        name = "French";
        symbol = "CAL_FRENCH";
        max = 13;
        maxdays = 30;
        full = PTN_CAL_FRENCH_MONTHS;
        shorts = PTN_CAL_FRENCH_MONTHS;
    }
    for (int i = 1; i <= max; i++) {
        ptn_array_set_entry(months.as.array, ptn_array_int_key(i), ptn_string(full[i]));
        ptn_array_set_entry(abbrev.as.array, ptn_array_int_key(i), ptn_string(shorts[i]));
    }
    ptn_array_set_entry(result.as.array, ptn_array_string_key("months"), months);
    ptn_array_set_entry(result.as.array, ptn_array_string_key("abbrevmonths"), abbrev);
    ptn_array_set_entry(result.as.array, ptn_array_string_key("maxdaysinmonth"), ptn_int(maxdays));
    ptn_array_set_entry(result.as.array, ptn_array_string_key("calname"), ptn_string(name));
    ptn_array_set_entry(result.as.array, ptn_array_string_key("calsymbol"), ptn_string(symbol));
    return result;
}

static PtnValue ptn_internal_cal_info(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    if (argc == 0 || ptn_value_deref(args[0]).type == PTN_NULL) {
        PtnValue result = ptn_array_from_literal_entries(0, NULL);
        for (int i = 0; i < PTN_CAL_NUM_CALS; i++) {
            ptn_array_set_entry(result.as.array, ptn_array_int_key(i), ptn_cal_info_for_calendar(i));
        }
        return result;
    }
    int64_t calendar = ptn_internal_expect_integer_arg(runtime, "cal_info", 1, "calendar", args[0], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    if (!ptn_cal_valid_calendar(calendar)) {
        ptn_throw_exception(runtime, "ValueError", "cal_info(): Argument #1 ($calendar) must be a valid calendar ID");
        return ptn_null();
    }
    return ptn_cal_info_for_calendar((int)calendar);
}

static PtnValue ptn_internal_gregoriantojd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t month = ptn_internal_expect_integer_arg(runtime, "gregoriantojd", 1, "month", args[0], line);
    int64_t day = ptn_internal_expect_integer_arg(runtime, "gregoriantojd", 2, "day", args[1], line);
    int64_t year = ptn_internal_expect_integer_arg(runtime, "gregoriantojd", 3, "year", args[2], line);
    return runtime->exceptions->active_exception != NULL ? ptn_null() : ptn_int(ptn_cal_gregorian_to_jd(month, day, year));
}

static PtnValue ptn_internal_juliantojd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t month = ptn_internal_expect_integer_arg(runtime, "juliantojd", 1, "month", args[0], line);
    int64_t day = ptn_internal_expect_integer_arg(runtime, "juliantojd", 2, "day", args[1], line);
    int64_t year = ptn_internal_expect_integer_arg(runtime, "juliantojd", 3, "year", args[2], line);
    return runtime->exceptions->active_exception != NULL ? ptn_null() : ptn_int(ptn_cal_julian_to_jd(month, day, year));
}

static PtnValue ptn_internal_jewishtojd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t month = ptn_internal_expect_integer_arg(runtime, "jewishtojd", 1, "month", args[0], line);
    int64_t day = ptn_internal_expect_integer_arg(runtime, "jewishtojd", 2, "day", args[1], line);
    int64_t year = ptn_internal_expect_integer_arg(runtime, "jewishtojd", 3, "year", args[2], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    if (!ptn_cal_validate_jewish_year_argument(runtime, "jewishtojd", 3, year)) return ptn_null();
    return ptn_int(ptn_cal_jewish_to_jd(month, day, year));
}

static PtnValue ptn_internal_frenchtojd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t month = ptn_internal_expect_integer_arg(runtime, "frenchtojd", 1, "month", args[0], line);
    int64_t day = ptn_internal_expect_integer_arg(runtime, "frenchtojd", 2, "day", args[1], line);
    int64_t year = ptn_internal_expect_integer_arg(runtime, "frenchtojd", 3, "year", args[2], line);
    return runtime->exceptions->active_exception != NULL ? ptn_null() : ptn_int(ptn_cal_french_to_jd(month, day, year));
}

static PtnValue ptn_internal_jdtogregorian(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "jdtogregorian", 1, "julian_day", args[0], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    int64_t m, d, y;
    ptn_cal_jd_to_gregorian(jd, &m, &d, &y);
    return ptn_cal_date_string(m, d, y);
}

static PtnValue ptn_internal_jdtojulian(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "jdtojulian", 1, "julian_day", args[0], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    int64_t m, d, y;
    ptn_cal_jd_to_julian(jd, &m, &d, &y);
    return ptn_cal_date_string(m, d, y);
}

static PtnValue ptn_internal_jdtojewish(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "jdtojewish", 1, "julian_day", args[0], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    int hebrew = argc >= 2 && ptn_is_truthy(ptn_value_deref(args[1]));
    int64_t flags_value = argc >= 3
        ? ptn_internal_expect_integer_arg(runtime, "jdtojewish", 3, "flags", args[2], line)
        : 0;
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    int64_t m, d, y;
    ptn_cal_jd_to_jewish(jd, &m, &d, &y);
    if (hebrew) {
        if (y < 0 || y > 9999) {
            ptn_throw_exception(runtime, "ValueError", "Year out of range (0-9999)");
            return ptn_null();
        }
        return ptn_cal_jewish_hebrew_date_string(m, d, y, (int)flags_value);
    }
    return ptn_cal_date_string(m, d, y);
}

static PtnValue ptn_internal_jdtofrench(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "jdtofrench", 1, "julian_day", args[0], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    int64_t m, d, y;
    ptn_cal_jd_to_french(jd, &m, &d, &y);
    return ptn_cal_date_string(m, d, y);
}

static const char *ptn_cal_month_name_for_mode(int64_t jd, int mode) {
    int64_t m, d, y;
    (void)d;
    switch (mode) {
        case PTN_CAL_MONTH_GREGORIAN_SHORT:
        default:
            ptn_cal_jd_to_gregorian(jd, &m, &d, &y);
            return (m >= 1 && m <= 12) ? PTN_CAL_GREGORIAN_ABBREV_MONTHS[m] : "";
        case PTN_CAL_MONTH_GREGORIAN_LONG:
            ptn_cal_jd_to_gregorian(jd, &m, &d, &y);
            return (m >= 1 && m <= 12) ? PTN_CAL_GREGORIAN_MONTHS[m] : "";
        case PTN_CAL_MONTH_JULIAN_SHORT:
            ptn_cal_jd_to_julian(jd, &m, &d, &y);
            return (m >= 1 && m <= 12) ? PTN_CAL_GREGORIAN_ABBREV_MONTHS[m] : "";
        case PTN_CAL_MONTH_JULIAN_LONG:
            ptn_cal_jd_to_julian(jd, &m, &d, &y);
            return (m >= 1 && m <= 12) ? PTN_CAL_GREGORIAN_MONTHS[m] : "";
        case PTN_CAL_MONTH_JEWISH:
            ptn_cal_jd_to_jewish(jd, &m, &d, &y);
            return (y > 0 && m >= 1 && m <= 13) ? ptn_cal_jewish_month_names(y)[m] : "";
        case PTN_CAL_MONTH_FRENCH:
            ptn_cal_jd_to_french(jd, &m, &d, &y);
            return (m >= 1 && m <= 13) ? PTN_CAL_FRENCH_MONTHS[m] : "";
    }
}

static PtnValue ptn_internal_jdmonthname(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "jdmonthname", 1, "julian_day", args[0], line);
    int64_t mode = ptn_internal_expect_integer_arg(runtime, "jdmonthname", 2, "mode", args[1], line);
    return runtime->exceptions->active_exception != NULL ? ptn_null() : ptn_string(ptn_cal_month_name_for_mode(jd, (int)mode));
}

static PtnValue ptn_internal_jdtomonthname(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_jdmonthname(runtime, argc, args, line);
}

static PtnValue ptn_internal_jddayofweek(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "jddayofweek", 1, "julian_day", args[0], line);
    int64_t mode = argc >= 2 ? ptn_internal_expect_integer_arg(runtime, "jddayofweek", 2, "mode", args[1], line) : 0;
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    int dow = ptn_cal_day_of_week(jd);
    if (mode == PTN_CAL_DOW_DAYNO) return ptn_int(dow);
    if (mode == PTN_CAL_DOW_SHORT) return ptn_string(PTN_CAL_ABBREV_DAY_NAMES[dow]);
    return ptn_string(PTN_CAL_DAY_NAMES[dow]);
}

static PtnValue ptn_cal_from_jd_array(int64_t jd, int calendar) {
    int64_t m = 0, d = 0, y = 0;
    if (calendar == PTN_CAL_GREGORIAN) ptn_cal_jd_to_gregorian(jd, &m, &d, &y);
    else if (calendar == PTN_CAL_JULIAN) ptn_cal_jd_to_julian(jd, &m, &d, &y);
    else if (calendar == PTN_CAL_JEWISH) ptn_cal_jd_to_jewish(jd, &m, &d, &y);
    else ptn_cal_jd_to_french(jd, &m, &d, &y);
    int dow = ptn_cal_day_of_week(jd);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    ptn_array_set_entry(result.as.array, ptn_array_string_key("date"), ptn_cal_date_string(m, d, y));
    ptn_array_set_entry(result.as.array, ptn_array_string_key("month"), ptn_int(m));
    ptn_array_set_entry(result.as.array, ptn_array_string_key("day"), ptn_int(d));
    ptn_array_set_entry(result.as.array, ptn_array_string_key("year"), ptn_int(y));
    if (calendar == PTN_CAL_JEWISH && y <= 0) {
        ptn_array_set_entry(result.as.array, ptn_array_string_key("dow"), ptn_null());
        ptn_array_set_entry(result.as.array, ptn_array_string_key("abbrevdayname"), ptn_string(""));
        ptn_array_set_entry(result.as.array, ptn_array_string_key("dayname"), ptn_string(""));
    } else {
        ptn_array_set_entry(result.as.array, ptn_array_string_key("dow"), ptn_int(dow));
        ptn_array_set_entry(result.as.array, ptn_array_string_key("abbrevdayname"), ptn_string(PTN_CAL_ABBREV_DAY_NAMES[dow]));
        ptn_array_set_entry(result.as.array, ptn_array_string_key("dayname"), ptn_string(PTN_CAL_DAY_NAMES[dow]));
    }
    ptn_array_set_entry(result.as.array, ptn_array_string_key("abbrevmonth"), ptn_string(ptn_cal_month_name_for_mode(jd, calendar == PTN_CAL_JULIAN ? PTN_CAL_MONTH_JULIAN_SHORT : calendar == PTN_CAL_JEWISH ? PTN_CAL_MONTH_JEWISH : calendar == PTN_CAL_FRENCH ? PTN_CAL_MONTH_FRENCH : PTN_CAL_MONTH_GREGORIAN_SHORT)));
    ptn_array_set_entry(result.as.array, ptn_array_string_key("monthname"), ptn_string(ptn_cal_month_name_for_mode(jd, calendar == PTN_CAL_JULIAN ? PTN_CAL_MONTH_JULIAN_LONG : calendar == PTN_CAL_JEWISH ? PTN_CAL_MONTH_JEWISH : calendar == PTN_CAL_FRENCH ? PTN_CAL_MONTH_FRENCH : PTN_CAL_MONTH_GREGORIAN_LONG)));
    return result;
}

static PtnValue ptn_internal_cal_from_jd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "cal_from_jd", 1, "julian_day", args[0], line);
    int64_t calendar = ptn_internal_expect_integer_arg(runtime, "cal_from_jd", 2, "calendar", args[1], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    if (!ptn_cal_valid_calendar(calendar)) {
        ptn_throw_exception(runtime, "ValueError", "cal_from_jd(): Argument #2 ($calendar) must be a valid calendar ID");
        return ptn_null();
    }
    return ptn_cal_from_jd_array(jd, (int)calendar);
}

static PtnValue ptn_internal_cal_to_jd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t calendar = ptn_internal_expect_integer_arg(runtime, "cal_to_jd", 1, "calendar", args[0], line);
    int64_t month = ptn_internal_expect_integer_arg(runtime, "cal_to_jd", 2, "month", args[1], line);
    int64_t day = ptn_internal_expect_integer_arg(runtime, "cal_to_jd", 3, "day", args[2], line);
    int64_t year = ptn_internal_expect_integer_arg(runtime, "cal_to_jd", 4, "year", args[3], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    if (!ptn_cal_valid_calendar(calendar)) {
        ptn_throw_exception(runtime, "ValueError", "cal_to_jd(): Argument #1 ($calendar) must be a valid calendar ID");
        return ptn_null();
    }
    if (!ptn_cal_validate_month_argument(runtime, "cal_to_jd", 2, month)) return ptn_null();
    if (!ptn_cal_validate_day_argument(runtime, "cal_to_jd", 3, day)) return ptn_null();
    if (!ptn_cal_validate_year_less_than_max(runtime, "cal_to_jd", 4, year)) return ptn_null();
    if (calendar == PTN_CAL_GREGORIAN) return ptn_int(ptn_cal_gregorian_to_jd(month, day, year));
    if (calendar == PTN_CAL_JULIAN) return ptn_int(ptn_cal_julian_to_jd(month, day, year));
    if (calendar == PTN_CAL_JEWISH) return ptn_int(ptn_cal_jewish_to_jd(month, day, year));
    return ptn_int(ptn_cal_french_to_jd(month, day, year));
}

static int64_t ptn_cal_to_jd_by_calendar(int64_t calendar, int64_t month, int64_t day, int64_t year) {
    if (calendar == PTN_CAL_GREGORIAN) return ptn_cal_gregorian_to_jd(month, day, year);
    if (calendar == PTN_CAL_JULIAN) return ptn_cal_julian_to_jd(month, day, year);
    if (calendar == PTN_CAL_JEWISH) return ptn_cal_jewish_to_jd(month, day, year);
    return ptn_cal_french_to_jd(month, day, year);
}

static PtnValue ptn_internal_cal_days_in_month(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t calendar = ptn_internal_expect_integer_arg(runtime, "cal_days_in_month", 1, "calendar", args[0], line);
    int64_t month = ptn_internal_expect_integer_arg(runtime, "cal_days_in_month", 2, "month", args[1], line);
    int64_t year = ptn_internal_expect_integer_arg(runtime, "cal_days_in_month", 3, "year", args[2], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    if (!ptn_cal_valid_calendar(calendar)) {
        ptn_throw_exception(runtime, "ValueError", "cal_days_in_month(): Argument #1 ($calendar) must be a valid calendar ID");
        return ptn_null();
    }
    if (!ptn_cal_validate_month_argument(runtime, "cal_days_in_month", 2, month)) return ptn_null();
    if (!ptn_cal_validate_year_less_than_max(runtime, "cal_days_in_month", 3, year)) return ptn_null();
    int64_t start = ptn_cal_to_jd_by_calendar(calendar, month, 1, year);
    if (start == 0) {
        ptn_throw_exception(runtime, "ValueError", "Invalid date");
        return ptn_null();
    }
    int64_t next = ptn_cal_to_jd_by_calendar(calendar, month + 1, 1, year);
    if (next == 0) {
        next = ptn_cal_to_jd_by_calendar(calendar, 1, 1, year == -1 ? 1 : year + 1);
        if (calendar == PTN_CAL_FRENCH && next == 0) {
            next = 2380953;
        }
    }
    return ptn_int(next - start);
}

static int64_t ptn_cal_easter_days_number(int64_t year, int64_t method) {
    int64_t golden = (year % 19) + 1;
    int64_t solar = 0;
    int64_t lunar = 0;
    int64_t pfm = 0;
    int64_t dom = 0;

    if ((year <= 1582 && method != PTN_CAL_EASTER_ALWAYS_GREGORIAN) ||
        (year >= 1583 && year <= 1752 && method != PTN_CAL_EASTER_ROMAN && method != PTN_CAL_EASTER_ALWAYS_GREGORIAN) ||
        method == PTN_CAL_EASTER_ALWAYS_JULIAN) {
        dom = ptn_cal_positive_mod(year + (year / 4) + 5, 7);
        pfm = ptn_cal_positive_mod(3 - (11 * golden) - 7, 30);
    } else {
        dom = ptn_cal_positive_mod(year + (year / 4) - (year / 100) + (year / 400), 7);
        solar = (year - 1600) / 100 - (year - 1600) / 400;
        lunar = (((year - 1400) / 100) * 8) / 25;
        pfm = ptn_cal_positive_mod(3 - (11 * golden) + solar - lunar, 30);
    }

    if (pfm == 29 || (pfm == 28 && golden > 11)) {
        pfm--;
    }
    int64_t tmp = ptn_cal_positive_mod(4 - pfm - dom, 7);
    return pfm + tmp + 1;
}

static int ptn_cal_validate_easter_year(PtnRuntime *runtime, const char *function_name, int arg_number, int gm, int64_t year) {
    int64_t max_year = (INT64_MAX / 5) * 4;
    if (year <= 0 || year > max_year) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%d ($year) must be between 1 and %lld",
            function_name,
            arg_number,
            (long long)max_year
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    if (gm && year < 1970) {
        char message[128];
        int written = snprintf(message, sizeof(message), "%s(): Argument #%d ($year) must be a year after 1970 (inclusive)", function_name, arg_number);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    if (gm && year > 2000000000LL) {
        char message[128];
        int written = snprintf(message, sizeof(message), "%s(): Argument #%d ($year) must be a year before 2.000.000.000 (inclusive)", function_name, arg_number);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    return 1;
}

static PtnValue ptn_internal_easter_days(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t year = argc >= 1 && ptn_value_deref(args[0]).type != PTN_NULL
        ? ptn_internal_expect_integer_arg(runtime, "easter_days", 1, "year", args[0], line)
        : 1970;
    int64_t method = argc >= 2 ? ptn_internal_expect_integer_arg(runtime, "easter_days", 2, "mode", args[1], line) : PTN_CAL_EASTER_DEFAULT;
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    if (!ptn_cal_validate_easter_year(runtime, "easter_days", 1, 0, year)) return ptn_null();
    return ptn_int(ptn_cal_easter_days_number(year, method));
}

static PtnValue ptn_internal_easter_date(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t year = argc >= 1 && ptn_value_deref(args[0]).type != PTN_NULL
        ? ptn_internal_expect_integer_arg(runtime, "easter_date", 1, "year", args[0], line)
        : 1970;
    int64_t method = argc >= 2 ? ptn_internal_expect_integer_arg(runtime, "easter_date", 2, "mode", args[1], line) : PTN_CAL_EASTER_DEFAULT;
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    if (!ptn_cal_validate_easter_year(runtime, "easter_date", 1, 1, year)) return ptn_null();
    int64_t easter = ptn_cal_easter_days_number(year, method);
    int64_t month = easter < 11 ? 3 : 4;
    int64_t day = easter < 11 ? easter + 21 : easter - 10;
    int64_t jd = ptn_cal_gregorian_to_jd(month, day, year);
    return ptn_int((jd - 2440588) * 86400);
}

static PtnValue ptn_internal_unixtojd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t timestamp;
    if (argc == 0 || ptn_value_deref(args[0]).type == PTN_NULL) {
        timestamp = (int64_t)time(NULL);
    } else {
        timestamp = ptn_internal_expect_integer_arg(runtime, "unixtojd", 1, "timestamp", args[0], line);
        if (runtime->exceptions->active_exception != NULL) return ptn_null();
        if (timestamp < 0) {
            ptn_throw_exception(runtime, "ValueError", "unixtojd(): Argument #1 ($timestamp) must be greater than or equal to 0");
            return ptn_null();
        }
    }
    return ptn_int(timestamp / 86400 + 2440588);
}

static PtnValue ptn_internal_jdtounix(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t jd = ptn_internal_expect_integer_arg(runtime, "jdtounix", 1, "julian_day", args[0], line);
    if (runtime->exceptions->active_exception != NULL) return ptn_null();
    int64_t max_jd = INT64_MAX / 86400 + 2440588;
    if (jd < 2440588 || jd > max_jd) {
        char message[128];
        int written = snprintf(message, sizeof(message), "jday must be between 2440588 and %lld", (long long)max_jd);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return ptn_null();
    }
    return ptn_int((jd - 2440588) * 86400);
}

static void ptn_defined_constants_add_bcmath(PtnValue table) {
    ptn_get_defined_constants_add_int(table, "BC_MATH_NUMBER", 1);
}

static PtnValue ptn_defined_constants_bcmath_table(void) {
    PtnValue table = ptn_array_from_literal_entries(0, NULL);
    ptn_defined_constants_add_bcmath(table);
    return table;
}

static void ptn_defined_constants_add_calendar(PtnValue table) {
    ptn_get_defined_constants_add_int(table, "CAL_GREGORIAN", PTN_CAL_GREGORIAN);
    ptn_get_defined_constants_add_int(table, "CAL_JULIAN", PTN_CAL_JULIAN);
    ptn_get_defined_constants_add_int(table, "CAL_JEWISH", PTN_CAL_JEWISH);
    ptn_get_defined_constants_add_int(table, "CAL_FRENCH", PTN_CAL_FRENCH);
    ptn_get_defined_constants_add_int(table, "CAL_NUM_CALS", PTN_CAL_NUM_CALS);
    ptn_get_defined_constants_add_int(table, "CAL_DOW_DAYNO", PTN_CAL_DOW_DAYNO);
    ptn_get_defined_constants_add_int(table, "CAL_DOW_LONG", PTN_CAL_DOW_LONG);
    ptn_get_defined_constants_add_int(table, "CAL_DOW_SHORT", PTN_CAL_DOW_SHORT);
    ptn_get_defined_constants_add_int(table, "CAL_MONTH_GREGORIAN_SHORT", PTN_CAL_MONTH_GREGORIAN_SHORT);
    ptn_get_defined_constants_add_int(table, "CAL_MONTH_GREGORIAN_LONG", PTN_CAL_MONTH_GREGORIAN_LONG);
    ptn_get_defined_constants_add_int(table, "CAL_MONTH_JULIAN_SHORT", PTN_CAL_MONTH_JULIAN_SHORT);
    ptn_get_defined_constants_add_int(table, "CAL_MONTH_JULIAN_LONG", PTN_CAL_MONTH_JULIAN_LONG);
    ptn_get_defined_constants_add_int(table, "CAL_MONTH_JEWISH", PTN_CAL_MONTH_JEWISH);
    ptn_get_defined_constants_add_int(table, "CAL_MONTH_FRENCH", PTN_CAL_MONTH_FRENCH);
    ptn_get_defined_constants_add_int(table, "CAL_EASTER_DEFAULT", PTN_CAL_EASTER_DEFAULT);
    ptn_get_defined_constants_add_int(table, "CAL_EASTER_ROMAN", PTN_CAL_EASTER_ROMAN);
    ptn_get_defined_constants_add_int(table, "CAL_EASTER_ALWAYS_GREGORIAN", PTN_CAL_EASTER_ALWAYS_GREGORIAN);
    ptn_get_defined_constants_add_int(table, "CAL_EASTER_ALWAYS_JULIAN", PTN_CAL_EASTER_ALWAYS_JULIAN);
    ptn_get_defined_constants_add_int(table, "CAL_JEWISH_ADD_ALAFIM_GERESH", PTN_CAL_JEWISH_ADD_ALAFIM_GERESH);
    ptn_get_defined_constants_add_int(table, "CAL_JEWISH_ADD_ALAFIM", PTN_CAL_JEWISH_ADD_ALAFIM);
    ptn_get_defined_constants_add_int(table, "CAL_JEWISH_ADD_GERESHAYIM", PTN_CAL_JEWISH_ADD_GERESHAYIM);
}

static PtnValue ptn_defined_constants_calendar_table(void) {
    PtnValue table = ptn_array_from_literal_entries(0, NULL);
    ptn_defined_constants_add_calendar(table);
    return table;
}

static int ptn_reflection_constant_is_bcmath(const char *name) {
    static const char *const names[] = { "BC_MATH_NUMBER" };
    return ptn_constant_name_matches_any(name, names, sizeof(names) / sizeof(names[0]));
}

static int ptn_reflection_constant_is_calendar(const char *name) {
    static const char *const names[] = {
        "CAL_GREGORIAN",
        "CAL_JULIAN",
        "CAL_JEWISH",
        "CAL_FRENCH",
        "CAL_NUM_CALS",
        "CAL_DOW_DAYNO",
        "CAL_DOW_LONG",
        "CAL_DOW_SHORT",
        "CAL_MONTH_GREGORIAN_SHORT",
        "CAL_MONTH_GREGORIAN_LONG",
        "CAL_MONTH_JULIAN_SHORT",
        "CAL_MONTH_JULIAN_LONG",
        "CAL_MONTH_JEWISH",
        "CAL_MONTH_FRENCH",
        "CAL_EASTER_DEFAULT",
        "CAL_EASTER_ROMAN",
        "CAL_EASTER_ALWAYS_GREGORIAN",
        "CAL_EASTER_ALWAYS_JULIAN",
        "CAL_JEWISH_ADD_ALAFIM_GERESH",
        "CAL_JEWISH_ADD_ALAFIM",
        "CAL_JEWISH_ADD_GERESHAYIM",
    };
    return ptn_constant_name_matches_any(name, names, sizeof(names) / sizeof(names[0]));
}
