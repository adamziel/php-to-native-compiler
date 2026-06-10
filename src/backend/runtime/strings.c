}

static PTN_UNUSED void ptn_string_buffer_init(PtnStringBuffer *buffer) {
    buffer->capacity = 128;
    buffer->len = 0;
    buffer->data = malloc(buffer->capacity);
    if (buffer->data == NULL) {
        ptn_abort_out_of_memory();
    }
    buffer->data[0] = '\0';
}

static PTN_UNUSED void ptn_string_buffer_reserve(PtnStringBuffer *buffer, size_t additional_len) {
    if (additional_len > SIZE_MAX - buffer->len - 1) {
        ptn_abort_out_of_memory();
    }
    size_t required = buffer->len + additional_len + 1;
    if (required <= buffer->capacity) {
        return;
    }
    size_t new_capacity = buffer->capacity;
    while (new_capacity < required) {
        if (new_capacity > SIZE_MAX / 2) {
            ptn_abort_out_of_memory();
        }
        new_capacity *= 2;
    }
    char *new_data = realloc(buffer->data, new_capacity);
    if (new_data == NULL) {
        ptn_abort_out_of_memory();
    }
    buffer->data = new_data;
    buffer->capacity = new_capacity;
}

static PTN_UNUSED void ptn_string_buffer_append_len(PtnStringBuffer *buffer, const char *value, size_t len) {
    ptn_string_buffer_reserve(buffer, len);
    memcpy(buffer->data + buffer->len, value, len);
    buffer->len += len;
    buffer->data[buffer->len] = '\0';
}

static PTN_UNUSED void ptn_string_buffer_append(PtnStringBuffer *buffer, const char *value) {
    ptn_string_buffer_append_len(buffer, value, strlen(value));
}

static PTN_UNUSED void ptn_string_buffer_append_char(PtnStringBuffer *buffer, char value) {
    ptn_string_buffer_reserve(buffer, 1);
    buffer->data[buffer->len] = value;
    buffer->len++;
    buffer->data[buffer->len] = '\0';
}

static PTN_UNUSED void ptn_string_buffer_append_format(PtnStringBuffer *buffer, const char *format, ...) {
    va_list args;
    va_start(args, format);
    va_list copy;
    va_copy(copy, args);
    int needed = vsnprintf(NULL, 0, format, copy);
    va_end(copy);
    if (needed < 0) {
        va_end(args);
        ptn_abort_out_of_memory();
    }
    ptn_string_buffer_reserve(buffer, (size_t)needed);
    int written = vsnprintf(buffer->data + buffer->len, buffer->capacity - buffer->len, format, args);
    va_end(args);
    if (written < 0 || written != needed) {
        ptn_abort_out_of_memory();
    }
    buffer->len += (size_t)written;
}

static PTN_UNUSED void ptn_string_buffer_append_indent(PtnStringBuffer *buffer, size_t indent) {
    for (size_t i = 0; i < indent; i++) {
        ptn_string_buffer_append_char(buffer, ' ');
    }
}

static PTN_UNUSED PtnValue ptn_bitwise_not(PtnValue value, const char *path, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_STRING) {
        PtnStringOperand string = {
            (const char *)value.as.string.data,
            NULL,
            value.as.string.len
        };
        return ptn_bitwise_string_not(string);
    }
    if (value.type == PTN_ARRAY) {
        ptn_abort_type_error_at("Cannot perform bitwise not on array", path, line);
    }
    return ptn_int(~ptn_bitwise_integer_operand(value));
}

static PTN_UNUSED int64_t ptn_shift_distance(PtnValue value) {
    value = ptn_value_deref(value);
    int64_t distance = ptn_bitwise_integer_operand(value);
    if (distance < 0) {
        ptn_abort_arithmetic_error("Bit shift by negative number");
    }
    return distance;
}

static PTN_UNUSED PtnValue ptn_shift_left(PtnValue left, PtnValue right) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    uint64_t left_bits = (uint64_t)ptn_bitwise_integer_operand(left);
    int64_t distance = ptn_shift_distance(right);
    if (distance >= 64) {
        return ptn_int(0);
    }
    return ptn_int((int64_t)(left_bits << (unsigned int)distance));
}

static PTN_UNUSED PtnValue ptn_shift_right(PtnValue left, PtnValue right) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    int64_t left_integer = ptn_bitwise_integer_operand(left);
    int64_t distance = ptn_shift_distance(right);
    if (distance >= 64) {
        return ptn_int(left_integer < 0 ? -1 : 0);
    }
    return ptn_int(left_integer >> (unsigned int)distance);
}

static PTN_UNUSED char *ptn_value_to_string(PtnValue value) {
    value = ptn_value_deref(value);
    char buffer[128];
    int written = 0;

    switch (value.type) {
        case PTN_NULL:
            return ptn_duplicate_string("");
        case PTN_BOOL:
            return ptn_duplicate_string(value.as.boolean ? "1" : "");
        case PTN_INT:
            written = snprintf(buffer, sizeof(buffer), "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            written = snprintf(buffer, sizeof(buffer), "%.14g", value.as.floating);
            break;
        case PTN_STRING:
            return ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len);
        case PTN_ARRAY:
            return ptn_duplicate_string("Array");
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            return ptn_duplicate_string("Object");
        case PTN_REFERENCE:
            return ptn_duplicate_string("");
    }

    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_duplicate_string(buffer);
}

static PTN_UNUSED PtnStringOperand ptn_string_operand_borrowed_len(const char *data, size_t len) {
    PtnStringOperand operand;
    operand.data = data;
    operand.owned = NULL;
    operand.len = len;
    return operand;
}

static PTN_UNUSED PtnStringOperand ptn_string_operand_borrowed(const char *data) {
    return ptn_string_operand_borrowed_len(data, strlen(data));
}

static PTN_UNUSED PtnStringOperand ptn_string_operand_owned_len(char *data, size_t len) {
    PtnStringOperand operand;
    operand.data = data;
    operand.owned = data;
    operand.len = len;
    return operand;
}

static PTN_UNUSED PtnStringOperand ptn_string_operand_owned(char *data) {
    return ptn_string_operand_owned_len(data, strlen(data));
}

static PTN_UNUSED void ptn_string_operand_free(PtnStringOperand operand) {
    free(operand.owned);
}

static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand(PtnValue value) {
    value = ptn_value_deref(value);
    char buffer[128];
    int written = 0;

    switch (value.type) {
        case PTN_NULL:
            return ptn_string_operand_borrowed("");
        case PTN_BOOL:
            return ptn_string_operand_borrowed(value.as.boolean ? "1" : "");
        case PTN_INT:
            written = snprintf(buffer, sizeof(buffer), "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            written = snprintf(buffer, sizeof(buffer), "%.14g", value.as.floating);
            break;
        case PTN_STRING:
            return ptn_string_operand_borrowed_len((const char *)value.as.string.data, value.as.string.len);
        case PTN_ARRAY:
            return ptn_string_operand_borrowed("Array");
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            return ptn_string_operand_borrowed("Object");
        case PTN_REFERENCE:
            return ptn_string_operand_borrowed("");
    }

    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_string_operand_owned_len(ptn_duplicate_string_len(buffer, (size_t)written), (size_t)written);
}

static PTN_UNUSED PtnStringOperand ptn_concat_string_operand(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
    }
    return ptn_value_to_string_operand(value);
}

static PTN_UNUSED PtnValue ptn_concat_many(
    PtnRuntime *runtime,
    const PtnConcatOperand *operands,
    size_t count,
    PtnStringOperand *strings
) {
    if (count == 0) {
        return ptn_string("");
    }

    size_t joined_len = 0;
    for (size_t i = 0; i < count; i++) {
        strings[i] = ptn_concat_string_operand(runtime, operands[i].value, operands[i].line);
        if (strings[i].len > SIZE_MAX - joined_len) {
            ptn_abort_out_of_memory();
        }
        joined_len += strings[i].len;
    }
    if (joined_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    char *joined = malloc(joined_len + 1);
    if (joined == NULL) {
        ptn_abort_out_of_memory();
    }
    size_t offset = 0;
    for (size_t i = 0; i < count; i++) {
        memcpy(joined + offset, strings[i].data, strings[i].len);
        offset += strings[i].len;
    }
    joined[joined_len] = '\0';
    for (size_t i = 0; i < count; i++) {
        ptn_string_operand_free(strings[i]);
    }
    return ptn_owned_string_len(joined, joined_len);
}

static PTN_UNUSED PtnValue ptn_concat(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    PtnConcatOperand operands[] = { { left, line }, { right, line } };
    PtnStringOperand strings[2];
    return ptn_concat_many(runtime, operands, 2, strings);
}

static PTN_UNUSED PtnValue ptn_cast_string(PtnValue value) {
    PtnStringOperand string = ptn_value_to_string_operand(value);
    char *copy = ptn_duplicate_string_len(string.data, string.len);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(copy, len);
}

static PTN_UNUSED PtnValue ptn_cast_bool(PtnValue value) {
    return ptn_bool(ptn_is_truthy(value));
}

typedef enum {
    PTN_CAST_TARGET_INT,
    PTN_CAST_TARGET_FLOAT,
    PTN_CAST_TARGET_STRING,
    PTN_CAST_TARGET_BOOL
} PtnCastTarget;

static PTN_UNUSED PtnValue ptn_cast_target(PtnValue value, PtnCastTarget target) {
    switch (target) {
        case PTN_CAST_TARGET_INT:
            return ptn_cast_int(value);
        case PTN_CAST_TARGET_FLOAT:
            return ptn_cast_float(value);
        case PTN_CAST_TARGET_STRING:
            return ptn_cast_string(value);
        case PTN_CAST_TARGET_BOOL:
            return ptn_cast_bool(value);
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_cast_noncanonical(
    PtnRuntime *runtime,
    PtnValue value,
    const char *spelling,
    const char *canonical,
    PtnCastTarget target,
    size_t line
) {
    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "Non-canonical cast (%s) is deprecated, use the (%s) cast instead",
        spelling,
        canonical
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_deprecation(&runtime->diagnostics, message, line);
    return ptn_cast_target(value, target);
}

static PTN_UNUSED PtnValue ptn_gettype_value(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return ptn_string("NULL");
        case PTN_BOOL:
            return ptn_string("boolean");
        case PTN_INT:
            return ptn_string("integer");
        case PTN_FLOAT:
            return ptn_string("double");
        case PTN_STRING:
            return ptn_string("string");
        case PTN_ARRAY:
            return ptn_string("array");
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            return ptn_string("object");
        case PTN_REFERENCE:
            return ptn_string("unknown type");
    }
    return ptn_string("unknown type");
}

static PTN_UNUSED PtnValue ptn_is_type(PtnValue value, PtnType type) {
    return ptn_bool(ptn_value_deref(value).type == type);
}

static PTN_UNUSED PtnValue ptn_is_scalar(PtnValue value) {
    return ptn_bool(
        value.type == PTN_BOOL ||
        value.type == PTN_INT ||
        value.type == PTN_FLOAT ||
        value.type == PTN_STRING
    );
}

static PTN_UNUSED int ptn_ascii_case_compare(const char *left, const char *right) {
    while (*left != '\0' && *right != '\0') {
        int left_byte = tolower((unsigned char)*left);
        int right_byte = tolower((unsigned char)*right);
        if (left_byte != right_byte) {
            return left_byte < right_byte ? -1 : 1;
        }
        left++;
        right++;
    }
    if (*left == '\0' && *right == '\0') {
        return 0;
    }
    return *left == '\0' ? -1 : 1;
}

static PTN_UNUSED int ptn_ascii_case_equal(const char *left, const char *right) {
    return ptn_ascii_case_compare(left, right) == 0;
}

static PTN_UNUSED int ptn_builtin_constant_value(const char *name, PtnValue *out) {
    if (strcmp(name, "E_ERROR") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "M_E") == 0) {
        *out = ptn_float(2.718281828459045);
        return 1;
    }
    if (strcmp(name, "M_LOG2E") == 0) {
        *out = ptn_float(1.4426950408889634);
        return 1;
    }
    if (strcmp(name, "M_LOG10E") == 0) {
        *out = ptn_float(0.4342944819032518);
        return 1;
    }
    if (strcmp(name, "M_LN2") == 0) {
        *out = ptn_float(0.6931471805599453);
        return 1;
    }
    if (strcmp(name, "M_LN10") == 0) {
        *out = ptn_float(2.302585092994046);
        return 1;
    }
    if (strcmp(name, "PHP_INT_MIN") == 0) {
        *out = ptn_int(INT64_MIN);
        return 1;
    }
    if (strcmp(name, "PHP_INT_MAX") == 0) {
        *out = ptn_int(INT64_MAX);
        return 1;
    }
    if (strcmp(name, "PHP_INT_SIZE") == 0) {
        *out = ptn_int((int64_t)sizeof(int64_t));
        return 1;
    }
    if (strcmp(name, "PHP_EOL") == 0) {
        *out = ptn_string("\n");
        return 1;
    }
    if (strcmp(name, "DIRECTORY_SEPARATOR") == 0) {
#if defined(_WIN32)
        *out = ptn_string("\\");
#else
        *out = ptn_string("/");
#endif
        return 1;
    }
    if (strcmp(name, "PATH_SEPARATOR") == 0) {
#if defined(_WIN32)
        *out = ptn_string(";");
#else
        *out = ptn_string(":");
#endif
        return 1;
    }
    if (strcmp(name, "INF") == 0) {
        *out = ptn_float(INFINITY);
        return 1;
    }
    if (strcmp(name, "NAN") == 0) {
        *out = ptn_float(NAN);
        return 1;
    }
    if (strcmp(name, "M_PI") == 0) {
        *out = ptn_float(3.14159265358979323846264338327950288);
        return 1;
    }
    if (strcmp(name, "M_PI_2") == 0) {
        *out = ptn_float(1.5707963267948966);
        return 1;
    }
    if (strcmp(name, "M_PI_4") == 0) {
        *out = ptn_float(0.7853981633974483);
        return 1;
    }
    if (strcmp(name, "M_1_PI") == 0) {
        *out = ptn_float(0.3183098861837907);
        return 1;
    }
    if (strcmp(name, "M_2_PI") == 0) {
        *out = ptn_float(0.6366197723675814);
        return 1;
    }
    if (strcmp(name, "M_SQRTPI") == 0) {
        *out = ptn_float(1.772453850905516);
        return 1;
    }
    if (strcmp(name, "M_2_SQRTPI") == 0) {
        *out = ptn_float(1.1283791670955126);
        return 1;
    }
    if (strcmp(name, "M_LNPI") == 0) {
        *out = ptn_float(1.1447298858494002);
        return 1;
    }
    if (strcmp(name, "M_EULER") == 0) {
        *out = ptn_float(0.5772156649015329);
        return 1;
    }
    if (strcmp(name, "M_SQRT2") == 0) {
        *out = ptn_float(1.4142135623730951);
        return 1;
    }
    if (strcmp(name, "M_SQRT1_2") == 0) {
        *out = ptn_float(0.7071067811865476);
        return 1;
    }
    if (strcmp(name, "M_SQRT3") == 0) {
        *out = ptn_float(1.7320508075688772);
        return 1;
    }
    return 0;
}

static PTN_UNUSED int ptn_same_double(double left, double right) {
    return memcmp(&left, &right, sizeof(double)) == 0;
}

static PTN_UNUSED void ptn_normalize_var_dump_exponent(char *buffer) {
    for (char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == 'e' || *cursor == 'E') {
            *cursor = 'E';
            cursor++;
            if (*cursor == '+' || *cursor == '-') {
                cursor++;
            }
            while (*cursor == '0' && isdigit((unsigned char)cursor[1])) {
                memmove(cursor, cursor + 1, strlen(cursor));
            }
            return;
        }
    }
}

static PTN_UNUSED void ptn_format_var_dump_float(double value, char *buffer, size_t buffer_size) {
    if (isnan(value)) {
        snprintf(buffer, buffer_size, "NAN");
        return;
    }
    if (isinf(value)) {
        snprintf(buffer, buffer_size, signbit(value) ? "-INF" : "INF");
        return;
    }

    for (int precision = 1; precision <= 17; precision++) {
        char candidate[64];
        char *end = NULL;
        double reparsed;
        snprintf(candidate, sizeof(candidate), "%.*g", precision, value);
        ptn_normalize_var_dump_exponent(candidate);
        errno = 0;
        reparsed = strtod(candidate, &end);
        if (errno == 0 && end != NULL && *end == '\0' && ptn_same_double(reparsed, value)) {
            snprintf(buffer, buffer_size, "%s", candidate);
            return;
        }
    }

    snprintf(buffer, buffer_size, "%.17g", value);
    ptn_normalize_var_dump_exponent(buffer);
}

static PTN_UNUSED int ptn_runtime_constant_value(PtnRuntime *runtime, const char *name, PtnValue *out) {
    if (ptn_symbols_get(runtime->constants, name, out)) {
        return 1;
    }
    return ptn_builtin_constant_value(name, out);
}

static PTN_UNUSED int ptn_runtime_constant_is_defined(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    return ptn_runtime_constant_value(runtime, name, &value);
}

static PTN_UNUSED int ptn_runtime_define_constant_if_absent(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t line
) {
    if (ptn_runtime_constant_is_defined(runtime, name)) {
        ptn_emit_constant_already_defined_warning(&runtime->diagnostics, name, line);
        return 0;
    }
