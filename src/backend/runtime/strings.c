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

static PTN_UNUSED PtnValue ptn_bitwise_not(
    PtnRuntime *runtime,
    PtnValue value,
    const char *path,
    size_t line
) {
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
    return ptn_int(~ptn_bitwise_integer_operand_checked(runtime, value, line));
}

static PTN_UNUSED int64_t ptn_shift_distance(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    int64_t distance = ptn_bitwise_integer_operand_checked(runtime, value, line);
    if (distance < 0) {
        ptn_throw_exception_at(runtime, "ArithmeticError", "Bit shift by negative number", runtime->source_path, line);
        return 0;
    }
    return distance;
}

static PTN_UNUSED PtnValue ptn_shift_left(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (ptn_numeric_operator_rejects_operand(left) ||
        ptn_numeric_operator_rejects_operand(right)) {
        ptn_throw_unsupported_operand_types(runtime, left, "<<", right, line);
        return ptn_null();
    }
    uint64_t left_bits = (uint64_t)ptn_bitwise_integer_operand_checked(runtime, left, line);
    int64_t distance = ptn_shift_distance(runtime, right, line);
    if (distance >= 64) {
        return ptn_int(0);
    }
    return ptn_int((int64_t)(left_bits << (unsigned int)distance));
}

static PTN_UNUSED PtnValue ptn_shift_right(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (ptn_numeric_operator_rejects_operand(left) ||
        ptn_numeric_operator_rejects_operand(right)) {
        ptn_throw_unsupported_operand_types(runtime, left, ">>", right, line);
        return ptn_null();
    }
    int64_t left_integer = ptn_bitwise_integer_operand_checked(runtime, left, line);
    int64_t distance = ptn_shift_distance(runtime, right, line);
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
            ptn_format_scalar_float(value.as.floating, buffer, sizeof(buffer));
            written = (int)strlen(buffer);
            break;
        case PTN_STRING:
            return ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len);
        case PTN_ARRAY:
            return ptn_duplicate_string("Array");
        case PTN_OBJECT:
        case PTN_CLOSURE:
            return ptn_duplicate_string("Object");
        case PTN_EXCEPTION: {
            PtnException *exception = value.as.exception;
            const char *path = exception->path != NULL ? exception->path : "ptn";
            int needed = snprintf(
                NULL,
                0,
                "%s: %s in %s:%zu\nStack trace:\n#0 {main}",
                exception->class_name,
                exception->message,
                path,
                exception->line
            );
            if (needed < 0) {
                ptn_abort_out_of_memory();
            }
            char *result = malloc((size_t)needed + 1);
            if (result == NULL) {
                ptn_abort_out_of_memory();
            }
            snprintf(
                result,
                (size_t)needed + 1,
                "%s: %s in %s:%zu\nStack trace:\n#0 {main}",
                exception->class_name,
                exception->message,
                path,
                exception->line
            );
            return result;
        }
        case PTN_RESOURCE:
            written = snprintf(buffer, sizeof(buffer), "Resource id #%lld", (long long)value.as.resource->id);
            break;
        case PTN_REFERENCE:
            return ptn_duplicate_string("");
    }

    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_duplicate_string(buffer);
}

static PTN_UNUSED void ptn_match_append_quoted_string(PtnStringBuffer *buffer, PtnString string) {
    size_t limit = string.len < 15 ? string.len : 15;
    ptn_string_buffer_append_char(buffer, '\'');
    for (size_t i = 0; i < limit; i++) {
        unsigned char byte = string.data[i];
        switch (byte) {
            case '\n':
                ptn_string_buffer_append(buffer, "\\n");
                break;
            case '\r':
                ptn_string_buffer_append(buffer, "\\r");
                break;
            case '\t':
                ptn_string_buffer_append(buffer, "\\t");
                break;
            case '\'':
                ptn_string_buffer_append(buffer, "\\'");
                break;
            case '\\':
                ptn_string_buffer_append(buffer, "\\\\");
                break;
            default:
                ptn_string_buffer_append_char(buffer, (char)byte);
                break;
        }
    }
    if (string.len > limit) {
        ptn_string_buffer_append(buffer, "...");
    }
    ptn_string_buffer_append_char(buffer, '\'');
}

static PTN_UNUSED char *ptn_unhandled_match_message(PtnValue value) {
    value = ptn_value_deref(value);
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    ptn_string_buffer_append(&buffer, "Unhandled match case ");

    char scalar[128];
    switch (value.type) {
        case PTN_NULL:
            ptn_string_buffer_append(&buffer, "NULL");
            break;
        case PTN_BOOL:
            ptn_string_buffer_append(&buffer, value.as.boolean ? "true" : "false");
            break;
        case PTN_INT:
            ptn_string_buffer_append_format(&buffer, "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            ptn_format_scalar_float(value.as.floating, scalar, sizeof(scalar));
            if (
                isfinite(value.as.floating) &&
                strchr(scalar, '.') == NULL &&
                strchr(scalar, 'E') == NULL &&
                strchr(scalar, 'e') == NULL
            ) {
                ptn_string_buffer_append_format(&buffer, "%s.0", scalar);
            } else {
                ptn_string_buffer_append(&buffer, scalar);
            }
            break;
        case PTN_STRING:
            ptn_match_append_quoted_string(&buffer, value.as.string);
            break;
        case PTN_ARRAY:
            ptn_string_buffer_append(&buffer, "of type array");
            break;
        case PTN_OBJECT:
            ptn_string_buffer_append_format(&buffer, "of type %s", value.as.object->class_name);
            break;
        case PTN_CLOSURE:
            ptn_string_buffer_append(&buffer, "of type Closure");
            break;
        case PTN_EXCEPTION:
            ptn_string_buffer_append_format(&buffer, "of type %s", value.as.exception->class_name);
            break;
        case PTN_RESOURCE:
            ptn_string_buffer_append(&buffer, "of type resource");
            break;
        case PTN_REFERENCE:
            ptn_string_buffer_append(&buffer, "of type reference");
            break;
    }
    return buffer.data;
}

static PTN_UNUSED char *ptn_dynamic_variable_name(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_STRING && ptn_string_has_embedded_nul(value.as.string)) {
        ptn_emit_type_error(
            &runtime->diagnostics,
            "Unsupported dynamic variable name containing embedded NUL"
        );
        exit(255);
    }

    switch (value.type) {
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_RESOURCE:
            return ptn_value_to_string(value);
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            break;
    }

    (void)line;
    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "Unsupported dynamic variable name of type %s",
        ptn_offset_container_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_type_error(&runtime->diagnostics, message);
    exit(255);
}

static PTN_UNUSED char *ptn_dynamic_property_name(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_STRING && value.as.string.len > 0 && value.as.string.data[0] == '\0') {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Cannot access property starting with \"\\0\"",
            runtime->source_path,
            line
        );
    }
    if (value.type == PTN_STRING && ptn_string_has_embedded_nul(value.as.string)) {
        ptn_emit_type_error(
            &runtime->diagnostics,
            "Unsupported dynamic property name containing embedded NUL"
        );
        exit(255);
    }
    return ptn_dynamic_variable_name(runtime, value, line);
}

static PTN_UNUSED char *ptn_callable_function_name(PtnValue callable) {
    callable = ptn_value_deref(callable);
    if (callable.type == PTN_ARRAY && callable.as.array->len == 2) {
        PtnArrayKey scope_key = ptn_array_int_key(0);
        PtnArrayKey method_key = ptn_array_int_key(1);
        PtnArrayEntry *scope_entry = ptn_array_entry_for_key(callable.as.array, scope_key);
        PtnArrayEntry *method_entry = ptn_array_entry_for_key(callable.as.array, method_key);
        if (scope_entry == NULL || method_entry == NULL) {
            return ptn_value_to_string(callable);
        }
        PtnValue scope = ptn_value_deref(scope_entry->value);
        PtnValue method = ptn_value_deref(method_entry->value);
        if (scope.type == PTN_STRING && method.type == PTN_STRING) {
            size_t scope_len = scope.as.string.len;
            size_t method_len = method.as.string.len;
            if (scope_len > SIZE_MAX - method_len - 3) {
                ptn_abort_out_of_memory();
            }
            char *name = malloc(scope_len + method_len + 3);
            if (name == NULL) {
                ptn_abort_out_of_memory();
            }
            memcpy(name, scope.as.string.data, scope_len);
            memcpy(name + scope_len, "::", 2);
            memcpy(name + scope_len + 2, method.as.string.data, method_len);
            name[scope_len + method_len + 2] = '\0';
            return name;
        }
    }
    return ptn_value_to_string(callable);
}

static PTN_UNUSED char *ptn_callable_output_name(PtnValue callable) {
    callable = ptn_value_deref(callable);
    if (callable.type == PTN_CLOSURE) {
        return ptn_duplicate_string("Closure::__invoke");
    }
    if (callable.type == PTN_ARRAY && callable.as.array->len == 2) {
        PtnArrayKey scope_key = ptn_array_int_key(0);
        PtnArrayKey method_key = ptn_array_int_key(1);
        PtnArrayEntry *scope_entry = ptn_array_entry_for_key(callable.as.array, scope_key);
        PtnArrayEntry *method_entry = ptn_array_entry_for_key(callable.as.array, method_key);
        ptn_array_key_free(scope_key);
        ptn_array_key_free(method_key);
        if (scope_entry == NULL || method_entry == NULL) {
            return ptn_value_to_string(callable);
        }

        PtnValue scope = ptn_value_deref(scope_entry->value);
        PtnValue method = ptn_value_deref(method_entry->value);
        if (method.type != PTN_STRING) {
            return ptn_value_to_string(callable);
        }

        char *scope_name = NULL;
        if (scope.type == PTN_OBJECT) {
            scope_name = ptn_duplicate_string(scope.as.object->class_name);
        } else if (scope.type == PTN_EXCEPTION) {
            scope_name = ptn_duplicate_string(scope.as.exception->class_name);
        } else if (scope.type == PTN_CLOSURE) {
            scope_name = ptn_duplicate_string("Closure");
        } else if (scope.type == PTN_STRING) {
            scope_name = ptn_duplicate_string_len((const char *)scope.as.string.data, scope.as.string.len);
        }
        if (scope_name == NULL) {
            return ptn_value_to_string(callable);
        }

        size_t scope_len = strlen(scope_name);
        size_t method_len = method.as.string.len;
        if (scope_len > SIZE_MAX - method_len - 3) {
            ptn_abort_out_of_memory();
        }
        char *name = malloc(scope_len + method_len + 3);
        if (name == NULL) {
            ptn_abort_out_of_memory();
        }
        memcpy(name, scope_name, scope_len);
        memcpy(name + scope_len, "::", 2);
        memcpy(name + scope_len + 2, method.as.string.data, method_len);
        name[scope_len + method_len + 2] = '\0';
        free(scope_name);
        return name;
    }
    return ptn_value_to_string(callable);
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
            ptn_format_scalar_float(value.as.floating, buffer, sizeof(buffer));
            written = (int)strlen(buffer);
            break;
        case PTN_STRING:
            return ptn_string_operand_borrowed_len((const char *)value.as.string.data, value.as.string.len);
        case PTN_ARRAY:
            return ptn_string_operand_borrowed("Array");
        case PTN_OBJECT:
        case PTN_CLOSURE:
            return ptn_string_operand_borrowed("Object");
        case PTN_EXCEPTION:
            return ptn_string_operand_owned(ptn_value_to_string(value));
        case PTN_RESOURCE:
            written = snprintf(buffer, sizeof(buffer), "Resource id #%lld", (long long)value.as.resource->id);
            break;
        case PTN_REFERENCE:
            return ptn_string_operand_borrowed("");
    }

    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_string_operand_owned_len(ptn_duplicate_string_len(buffer, (size_t)written), (size_t)written);
}

static PTN_UNUSED int ptn_tostring_return_value_is_allowed(PtnRuntime *runtime, PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_STRING) {
        return 1;
    }
    if (runtime != NULL && runtime->strict_types) {
        return 0;
    }
    return value.type == PTN_BOOL || value.type == PTN_INT || value.type == PTN_FLOAT;
}

static PTN_UNUSED const char *ptn_tostring_return_type_name(PtnValue value) {
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

static PTN_UNUSED int ptn_try_object_to_string_operand(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnStringOperand *out
) {
    value = ptn_value_deref(value);
    if (
        runtime == NULL ||
        runtime->method_dispatch == NULL ||
        runtime->declared_method_exists == NULL ||
        value.type != PTN_OBJECT ||
        !runtime->declared_method_exists(value.as.object->class_name, "__toString")
    ) {
        return 0;
    }

    PtnValue result = runtime->method_dispatch(runtime, value, "__toString", 0, NULL, line);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&result);
        *out = ptn_string_operand_borrowed("");
        return 1;
    }
    if (!ptn_tostring_return_value_is_allowed(runtime, result)) {
        const char *return_type = ptn_tostring_return_type_name(result);
        int needed = snprintf(
            NULL,
            0,
            "%s::__toString(): Return value must be of type string, %s returned",
            value.as.object->class_name,
            return_type
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
            "%s::__toString(): Return value must be of type string, %s returned",
            value.as.object->class_name,
            return_type
        );
        ptn_value_destroy(&result);
        ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
        free(message);
        *out = ptn_string_operand_borrowed("");
        return 1;
    }
    PtnStringOperand result_string = ptn_value_to_string_operand(result);
    char *copy = ptn_duplicate_string_len(result_string.data, result_string.len);
    *out = ptn_string_operand_owned_len(copy, result_string.len);
    ptn_string_operand_free(result_string);
    ptn_value_destroy(&result);
    return 1;
}

static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand_with_runtime(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    PtnStringOperand object_string;
    if (ptn_try_object_to_string_operand(runtime, value, line, &object_string)) {
        return object_string;
    }
    PtnValue resolved = ptn_value_deref(value);
    const char *class_name = NULL;
    if (resolved.type == PTN_OBJECT) {
        class_name = resolved.as.object->class_name;
    } else if (resolved.type == PTN_CLOSURE) {
        class_name = "Closure";
    } else if (resolved.type == PTN_EXCEPTION) {
        return ptn_value_to_string_operand(value);
    }
    if (class_name != NULL && runtime != NULL) {
        int needed = snprintf(NULL, 0, "Object of class %s could not be converted to string", class_name);
        if (needed < 0) {
            ptn_abort_out_of_memory();
        }
        char *message = malloc((size_t)needed + 1);
        if (message == NULL) {
            ptn_abort_out_of_memory();
        }
        snprintf(message, (size_t)needed + 1, "Object of class %s could not be converted to string", class_name);
        ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
        free(message);
        return ptn_string_operand_borrowed("");
    }
    return ptn_value_to_string_operand(value);
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
    return ptn_value_to_string_operand_with_runtime(runtime, value, line);
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

static PTN_UNUSED PtnValue ptn_cast_string_with_runtime(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT || value.type == PTN_CLOSURE || value.type == PTN_EXCEPTION) {
        if (value.type == PTN_EXCEPTION) {
            char *exception_string = ptn_value_to_string(value);
            return ptn_owned_string(exception_string);
        }
        PtnStringOperand object_string;
        if (ptn_try_object_to_string_operand(runtime, value, line, &object_string)) {
            char *copy = ptn_duplicate_string_len(object_string.data, object_string.len);
            size_t len = object_string.len;
            ptn_string_operand_free(object_string);
            return ptn_owned_string_len(copy, len);
        }

        const char *class_name = "Object";
        if (value.type == PTN_OBJECT) {
            class_name = value.as.object->class_name;
        } else if (value.type == PTN_CLOSURE) {
            class_name = "Closure";
        } else if (value.type == PTN_EXCEPTION) {
            class_name = value.as.exception->class_name;
        }

        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Object of class %s could not be converted to string",
            class_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        if (runtime != NULL) {
            ptn_throw_exception(runtime, "Error", message);
            return ptn_string("");
        }
        fprintf(stderr, "Fatal error: %s\n", message);
        exit(255);
    }

    PtnStringOperand string = ptn_value_to_string_operand_with_runtime(runtime, value, line);
    char *copy = ptn_duplicate_string_len(string.data, string.len);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(copy, len);
}

static PTN_UNUSED PtnValue ptn_cast_string(PtnValue value) {
    return ptn_cast_string_with_runtime(NULL, value, 0);
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
        case PTN_OBJECT:
        case PTN_CLOSURE:
            return ptn_string("object");
        case PTN_EXCEPTION:
            return ptn_string("object");
        case PTN_RESOURCE:
            return ptn_string(
                (value.as.resource->stream == NULL && value.as.resource->directory == NULL)
                    ? "resource (closed)"
                    : "resource"
            );
        case PTN_REFERENCE:
            return ptn_string("unknown type");
    }
    return ptn_string("unknown type");
}

static PTN_UNUSED PtnValue ptn_is_type(PtnValue value, PtnType type) {
    return ptn_bool(ptn_value_deref(value).type == type);
}

static PTN_UNUSED PtnValue ptn_is_object(PtnValue value) {
    value = ptn_value_deref(value);
    return ptn_bool(
        value.type == PTN_OBJECT ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION
    );
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
        *out = ptn_int(PTN_E_ERROR);
        return 1;
    }
    if (strcmp(name, "E_WARNING") == 0) {
        *out = ptn_int(PTN_E_WARNING);
        return 1;
    }
    if (strcmp(name, "E_PARSE") == 0) {
        *out = ptn_int(PTN_E_PARSE);
        return 1;
    }
    if (strcmp(name, "E_NOTICE") == 0) {
        *out = ptn_int(PTN_E_NOTICE);
        return 1;
    }
    if (strcmp(name, "E_CORE_ERROR") == 0) {
        *out = ptn_int(PTN_E_CORE_ERROR);
        return 1;
    }
    if (strcmp(name, "E_CORE_WARNING") == 0) {
        *out = ptn_int(PTN_E_CORE_WARNING);
        return 1;
    }
    if (strcmp(name, "E_COMPILE_ERROR") == 0) {
        *out = ptn_int(PTN_E_COMPILE_ERROR);
        return 1;
    }
    if (strcmp(name, "E_COMPILE_WARNING") == 0) {
        *out = ptn_int(PTN_E_COMPILE_WARNING);
        return 1;
    }
    if (strcmp(name, "E_USER_ERROR") == 0) {
        *out = ptn_int(PTN_E_USER_ERROR);
        return 1;
    }
    if (strcmp(name, "E_USER_WARNING") == 0) {
        *out = ptn_int(PTN_E_USER_WARNING);
        return 1;
    }
    if (strcmp(name, "E_USER_NOTICE") == 0) {
        *out = ptn_int(PTN_E_USER_NOTICE);
        return 1;
    }
    if (strcmp(name, "E_STRICT") == 0) {
        *out = ptn_int(PTN_E_STRICT);
        return 1;
    }
    if (strcmp(name, "E_RECOVERABLE_ERROR") == 0) {
        *out = ptn_int(PTN_E_RECOVERABLE_ERROR);
        return 1;
    }
    if (strcmp(name, "E_DEPRECATED") == 0) {
        *out = ptn_int(PTN_E_DEPRECATED);
        return 1;
    }
    if (strcmp(name, "E_USER_DEPRECATED") == 0) {
        *out = ptn_int(PTN_E_USER_DEPRECATED);
        return 1;
    }
    if (strcmp(name, "E_ALL") == 0) {
        *out = ptn_int(PTN_E_ALL);
        return 1;
    }
    if (strcmp(name, "INI_USER") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "INI_PERDIR") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "INI_SYSTEM") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "INI_ALL") == 0) {
        *out = ptn_int(7);
        return 1;
    }
    if (strcmp(name, "CASE_LOWER") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "CASE_UPPER") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "SORT_REGULAR") == 0) {
        *out = ptn_int(PTN_SORT_REGULAR);
        return 1;
    }
    if (strcmp(name, "SORT_NUMERIC") == 0) {
        *out = ptn_int(PTN_SORT_NUMERIC);
        return 1;
    }
    if (strcmp(name, "SORT_STRING") == 0) {
        *out = ptn_int(PTN_SORT_STRING);
        return 1;
    }
    if (strcmp(name, "SORT_DESC") == 0) {
        *out = ptn_int(PTN_SORT_DESC);
        return 1;
    }
    if (strcmp(name, "SORT_ASC") == 0) {
        *out = ptn_int(PTN_SORT_ASC);
        return 1;
    }
    if (strcmp(name, "SORT_LOCALE_STRING") == 0) {
        *out = ptn_int(PTN_SORT_LOCALE_STRING);
        return 1;
    }
    if (strcmp(name, "SORT_NATURAL") == 0) {
        *out = ptn_int(PTN_SORT_NATURAL);
        return 1;
    }
    if (strcmp(name, "SORT_FLAG_CASE") == 0) {
        *out = ptn_int(PTN_SORT_FLAG_CASE);
        return 1;
    }
    if (strcmp(name, "ARRAY_FILTER_USE_BOTH") == 0) {
        *out = ptn_int(PTN_ARRAY_FILTER_USE_BOTH);
        return 1;
    }
    if (strcmp(name, "ARRAY_FILTER_USE_KEY") == 0) {
        *out = ptn_int(PTN_ARRAY_FILTER_USE_KEY);
        return 1;
    }
    if (strcmp(name, "STR_PAD_LEFT") == 0) {
        *out = ptn_int(PTN_STR_PAD_LEFT);
        return 1;
    }
    if (strcmp(name, "STR_PAD_RIGHT") == 0) {
        *out = ptn_int(PTN_STR_PAD_RIGHT);
        return 1;
    }
    if (strcmp(name, "STR_PAD_BOTH") == 0) {
        *out = ptn_int(PTN_STR_PAD_BOTH);
        return 1;
    }
    if (strcmp(name, "HTML_SPECIALCHARS") == 0) {
        *out = ptn_int(PTN_HTML_SPECIALCHARS);
        return 1;
    }
    if (strcmp(name, "HTML_ENTITIES") == 0) {
        *out = ptn_int(PTN_HTML_ENTITIES);
        return 1;
    }
    if (strcmp(name, "ENT_NOQUOTES") == 0) {
        *out = ptn_int(PTN_ENT_NOQUOTES);
        return 1;
    }
    if (strcmp(name, "ENT_COMPAT") == 0) {
        *out = ptn_int(PTN_ENT_COMPAT);
        return 1;
    }
    if (strcmp(name, "ENT_QUOTES") == 0) {
        *out = ptn_int(PTN_ENT_QUOTES);
        return 1;
    }
    if (strcmp(name, "ENT_IGNORE") == 0) {
        *out = ptn_int(PTN_ENT_IGNORE);
        return 1;
    }
    if (strcmp(name, "ENT_SUBSTITUTE") == 0) {
        *out = ptn_int(PTN_ENT_SUBSTITUTE);
        return 1;
    }
    if (strcmp(name, "ENT_DISALLOWED") == 0) {
        *out = ptn_int(PTN_ENT_DISALLOWED);
        return 1;
    }
    if (strcmp(name, "ENT_HTML401") == 0) {
        *out = ptn_int(PTN_ENT_HTML401);
        return 1;
    }
    if (strcmp(name, "ENT_XML1") == 0) {
        *out = ptn_int(PTN_ENT_XML1);
        return 1;
    }
    if (strcmp(name, "ENT_XHTML") == 0) {
        *out = ptn_int(PTN_ENT_XHTML);
        return 1;
    }
    if (strcmp(name, "ENT_HTML5") == 0) {
        *out = ptn_int(PTN_ENT_HTML5);
        return 1;
    }
    if (strcmp(name, "COUNT_NORMAL") == 0) {
        *out = ptn_int(PTN_COUNT_NORMAL);
        return 1;
    }
    if (strcmp(name, "COUNT_RECURSIVE") == 0) {
        *out = ptn_int(PTN_COUNT_RECURSIVE);
        return 1;
    }
    if (strcmp(name, "PATHINFO_DIRNAME") == 0) {
        *out = ptn_int(PTN_PATHINFO_DIRNAME);
        return 1;
    }
    if (strcmp(name, "PATHINFO_BASENAME") == 0) {
        *out = ptn_int(PTN_PATHINFO_BASENAME);
        return 1;
    }
    if (strcmp(name, "PATHINFO_EXTENSION") == 0) {
        *out = ptn_int(PTN_PATHINFO_EXTENSION);
        return 1;
    }
    if (strcmp(name, "PATHINFO_FILENAME") == 0) {
        *out = ptn_int(PTN_PATHINFO_FILENAME);
        return 1;
    }
    if (strcmp(name, "PATHINFO_ALL") == 0) {
        *out = ptn_int(PTN_PATHINFO_ALL);
        return 1;
    }
    if (strcmp(name, "FILE_USE_INCLUDE_PATH") == 0) {
        *out = ptn_int(PTN_FILE_USE_INCLUDE_PATH);
        return 1;
    }
    if (strcmp(name, "FILE_IGNORE_NEW_LINES") == 0) {
        *out = ptn_int(PTN_FILE_IGNORE_NEW_LINES);
        return 1;
    }
    if (strcmp(name, "FILE_SKIP_EMPTY_LINES") == 0) {
        *out = ptn_int(PTN_FILE_SKIP_EMPTY_LINES);
        return 1;
    }
    if (strcmp(name, "SEEK_SET") == 0) {
        *out = ptn_int(PTN_SEEK_SET);
        return 1;
    }
    if (strcmp(name, "SEEK_CUR") == 0) {
        *out = ptn_int(PTN_SEEK_CUR);
        return 1;
    }
    if (strcmp(name, "SEEK_END") == 0) {
        *out = ptn_int(PTN_SEEK_END);
        return 1;
    }
    if (strcmp(name, "STDIN") == 0) {
        *out = ptn_standard_stream_resource_value(1);
        return 1;
    }
    if (strcmp(name, "STDOUT") == 0) {
        *out = ptn_standard_stream_resource_value(2);
        return 1;
    }
    if (strcmp(name, "STDERR") == 0) {
        *out = ptn_standard_stream_resource_value(3);
        return 1;
    }
    if (strcmp(name, "LC_ALL") == 0) {
        *out = ptn_int(PTN_LC_ALL);
        return 1;
    }
    if (strcmp(name, "LC_COLLATE") == 0) {
        *out = ptn_int(PTN_LC_COLLATE);
        return 1;
    }
    if (strcmp(name, "LC_CTYPE") == 0) {
        *out = ptn_int(PTN_LC_CTYPE);
        return 1;
    }
    if (strcmp(name, "LC_MESSAGES") == 0) {
        *out = ptn_int(PTN_LC_MESSAGES);
        return 1;
    }
    if (strcmp(name, "LC_MONETARY") == 0) {
        *out = ptn_int(PTN_LC_MONETARY);
        return 1;
    }
    if (strcmp(name, "LC_NUMERIC") == 0) {
        *out = ptn_int(PTN_LC_NUMERIC);
        return 1;
    }
    if (strcmp(name, "LC_TIME") == 0) {
        *out = ptn_int(PTN_LC_TIME);
        return 1;
    }
    if (strcmp(name, "DATE_ATOM") == 0) {
        *out = ptn_string("Y-m-d\\TH:i:sP");
        return 1;
    }
    if (strcmp(name, "DATE_COOKIE") == 0) {
        *out = ptn_string("l, d-M-Y H:i:s T");
        return 1;
    }
    if (strcmp(name, "DATE_ISO8601") == 0) {
        *out = ptn_string("Y-m-d\\TH:i:sO");
        return 1;
    }
    if (strcmp(name, "DATE_ISO8601_EXPANDED") == 0) {
        *out = ptn_string("X-m-d\\TH:i:sP");
        return 1;
    }
    if (strcmp(name, "DATE_RFC822") == 0) {
        *out = ptn_string("D, d M y H:i:s O");
        return 1;
    }
    if (strcmp(name, "DATE_RFC850") == 0 || strcmp(name, "DATE_RFC1036") == 0) {
        *out = ptn_string("l, d-M-y H:i:s T");
        return 1;
    }
    if (
        strcmp(name, "DATE_RFC1123") == 0 ||
        strcmp(name, "DATE_RFC2822") == 0 ||
        strcmp(name, "DATE_RSS") == 0
    ) {
        *out = ptn_string("D, d M Y H:i:s O");
        return 1;
    }
    if (strcmp(name, "DATE_RFC7231") == 0) {
        *out = ptn_string("D, d M Y H:i:s \\G\\M\\T");
        return 1;
    }
    if (strcmp(name, "DATE_RFC3339") == 0 || strcmp(name, "DATE_W3C") == 0) {
        *out = ptn_string("Y-m-d\\TH:i:sP");
        return 1;
    }
    if (strcmp(name, "DATE_RFC3339_EXTENDED") == 0) {
        *out = ptn_string("Y-m-d\\TH:i:s.vP");
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
    if (strcmp(name, "PHP_MAXPATHLEN") == 0) {
        *out = ptn_int(PTN_PHP_MAXPATHLEN);
        return 1;
    }
    if (strcmp(name, "PHP_VERSION") == 0) {
        *out = ptn_string(PTN_PHP_VERSION);
        return 1;
    }
    if (strcmp(name, "PHP_MAJOR_VERSION") == 0) {
        *out = ptn_int(PTN_PHP_MAJOR_VERSION);
        return 1;
    }
    if (strcmp(name, "PHP_MINOR_VERSION") == 0) {
        *out = ptn_int(PTN_PHP_MINOR_VERSION);
        return 1;
    }
    if (strcmp(name, "PHP_RELEASE_VERSION") == 0) {
        *out = ptn_int(PTN_PHP_RELEASE_VERSION);
        return 1;
    }
    if (strcmp(name, "PHP_EXTRA_VERSION") == 0) {
        *out = ptn_string(PTN_PHP_EXTRA_VERSION);
        return 1;
    }
    if (strcmp(name, "PHP_VERSION_ID") == 0) {
        *out = ptn_int(PTN_PHP_VERSION_ID);
        return 1;
    }
    if (strcmp(name, "PHP_ZTS") == 0) {
        *out = ptn_int(PTN_PHP_ZTS);
        return 1;
    }
    if (strcmp(name, "PHP_DEBUG") == 0) {
        *out = ptn_int(PTN_PHP_DEBUG);
        return 1;
    }
    if (strcmp(name, "PHP_SAPI") == 0) {
        *out = ptn_string(PTN_PHP_SAPI_NAME);
        return 1;
    }
    if (strcmp(name, "PHP_OS") == 0) {
        *out = ptn_string(PTN_PHP_OS);
        return 1;
    }
    if (strcmp(name, "PHP_OS_FAMILY") == 0) {
        *out = ptn_string(PTN_PHP_OS_FAMILY);
        return 1;
    }
    if (strcmp(name, "PHP_SHLIB_SUFFIX") == 0) {
        *out = ptn_string(PTN_PHP_SHLIB_SUFFIX);
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

static PTN_UNUSED int ptn_var_dump_has_nonnegative_exponent(const char *buffer) {
    for (const char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == 'e' || *cursor == 'E') {
            cursor++;
            return *cursor != '-';
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_var_dump_ensure_exponent_decimal(char *buffer) {
    for (char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == '.') {
            return;
        }
        if (*cursor == 'E') {
            size_t tail_len = strlen(cursor);
            memmove(cursor + 2, cursor, tail_len + 1);
            cursor[0] = '.';
            cursor[1] = '0';
            return;
        }
    }
}

static PTN_UNUSED int ptn_format_var_dump_integral_fixed(double value, char *buffer, size_t buffer_size) {
    double integral = 0.0;
    char candidate[64];
    char *end = NULL;
    double reparsed;

    if (fabs(value) >= 1e17 || modf(value, &integral) != 0.0) {
        return 0;
    }

    snprintf(candidate, sizeof(candidate), "%.0f", value);
    errno = 0;
    reparsed = strtod(candidate, &end);
    if (errno != 0 || end == NULL || *end != '\0' || !ptn_same_double(reparsed, value)) {
        return 0;
    }

    snprintf(buffer, buffer_size, "%s", candidate);
    return 1;
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
            if (ptn_var_dump_has_nonnegative_exponent(candidate) &&
                ptn_format_var_dump_integral_fixed(value, buffer, buffer_size)) {
                return;
            }
            ptn_var_dump_ensure_exponent_decimal(candidate);
            snprintf(buffer, buffer_size, "%s", candidate);
            return;
        }
    }

    snprintf(buffer, buffer_size, "%.17g", value);
    ptn_normalize_var_dump_exponent(buffer);
    ptn_var_dump_ensure_exponent_decimal(buffer);
}

static PTN_UNUSED int ptn_ascii_case_equal_n(const char *left, const char *right, size_t len) {
    for (size_t i = 0; i < len; i++) {
        unsigned char l = (unsigned char)left[i];
        unsigned char r = (unsigned char)right[i];
        if (tolower(l) != tolower(r)) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED int ptn_runtime_class_constant_value(
    PtnRuntime *runtime,
    const char *name,
    PtnValue *out
) {
    const char *separator = strstr(name, "::");
    if (separator == NULL || separator == name || separator[2] == '\0') {
        return 0;
    }

    size_t class_len = (size_t)(separator - name);
    const char *constant_name = separator + 2;
    PtnSymbolTable *constants = ptn_runtime_class_constant_table(runtime);
    for (size_t i = 0; i < constants->len; i++) {
        const char *stored_name = constants->items[i].name;
        const char *stored_separator = strstr(stored_name, "::");
        if (stored_separator == NULL || (size_t)(stored_separator - stored_name) != class_len) {
            continue;
        }
        if (!ptn_ascii_case_equal_n(stored_name, name, class_len)) {
            continue;
        }
        if (strcmp(stored_separator + 2, constant_name) != 0) {
            continue;
        }
        *out = ptn_value_borrow(constants->items[i].value);
        return 1;
    }
    return ptn_builtin_class_constant_value_span(name, class_len, constant_name, out);
}

static PTN_UNUSED int ptn_runtime_constant_value(PtnRuntime *runtime, const char *name, PtnValue *out) {
    if (ptn_symbols_get(runtime->constants, name, out)) {
        return 1;
    }
    if (ptn_builtin_constant_value(name, out)) {
        return 1;
    }
    return ptn_runtime_class_constant_value(runtime, name, out);
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
