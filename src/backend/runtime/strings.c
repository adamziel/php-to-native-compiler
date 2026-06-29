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
    if (value.type == PTN_NULL) {
        ptn_throw_exception_at(runtime, "TypeError", "Cannot perform bitwise not on null", path, line);
        return ptn_null();
    }
    if (value.type == PTN_BOOL) {
        ptn_throw_exception_at(runtime, "TypeError", "Cannot perform bitwise not on bool", path, line);
        return ptn_null();
    }
    if (value.type == PTN_ARRAY) {
        ptn_throw_exception_at(runtime, "TypeError", "Cannot perform bitwise not on array", path, line);
        return ptn_null();
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
    if (ptn_integer_operator_rejects_operand(left) ||
        ptn_integer_operator_rejects_operand(right)) {
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
    if (ptn_integer_operator_rejects_operand(left) ||
        ptn_integer_operator_rejects_operand(right)) {
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
    char buffer[PTN_FLOAT_FORMAT_BUFFER_SIZE];
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
            return ptn_duplicate_string("Object");
        case PTN_CLOSURE:
            return ptn_duplicate_string("Closure");
        case PTN_EXCEPTION: {
            PtnException *exception = value.as.exception;
            PtnStringOperand exception_string = ptn_exception_to_string_operand(NULL, exception);
            char *result = ptn_duplicate_string_len(exception_string.data, exception_string.len);
            free(exception_string.owned);
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

static PTN_UNUSED void ptn_match_append_quoted_string(
    PtnStringBuffer *buffer,
    PtnString string,
    size_t max_string_len
) {
    size_t limit = string.len < max_string_len ? string.len : max_string_len;
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
                if (byte < 0x20 || byte >= 0x7f) {
                    ptn_string_buffer_append_format(buffer, "\\x%02X", (unsigned int)byte);
                } else {
                    ptn_string_buffer_append_char(buffer, (char)byte);
                }
                break;
        }
    }
    if (string.len > limit) {
        ptn_string_buffer_append(buffer, "...");
    }
    ptn_string_buffer_append_char(buffer, '\'');
}

static PTN_UNUSED char *ptn_unhandled_match_message(PtnRuntime *runtime, PtnValue value) {
    value = ptn_value_deref(value);
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    ptn_string_buffer_append(&buffer, "Unhandled match case ");
    int ignore_args = ptn_runtime_exception_ignore_args(runtime);
    size_t max_string_len = ptn_runtime_exception_string_param_max_len(runtime);

    char scalar[PTN_FLOAT_FORMAT_BUFFER_SIZE];
    switch (value.type) {
        case PTN_NULL:
            ptn_string_buffer_append(&buffer, ignore_args ? "of type null" : "NULL");
            break;
        case PTN_BOOL:
            if (ignore_args) {
                ptn_string_buffer_append(&buffer, "of type bool");
            } else {
                ptn_string_buffer_append(&buffer, value.as.boolean ? "true" : "false");
            }
            break;
        case PTN_INT:
            if (ignore_args) {
                ptn_string_buffer_append(&buffer, "of type int");
            } else {
                ptn_string_buffer_append_format(&buffer, "%lld", (long long)value.as.integer);
            }
            break;
        case PTN_FLOAT:
            if (ignore_args) {
                ptn_string_buffer_append(&buffer, "of type float");
                break;
            }
            ptn_format_runtime_scalar_float(runtime, value.as.floating, scalar, sizeof(scalar));
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
            if (ignore_args || max_string_len == 0) {
                ptn_string_buffer_append(&buffer, "of type string");
            } else {
                ptn_match_append_quoted_string(&buffer, value.as.string, max_string_len);
            }
            break;
        case PTN_ARRAY:
            ptn_string_buffer_append(&buffer, "of type array");
            break;
        case PTN_OBJECT:
            if (value.as.object->enum_case_name != NULL) {
                ptn_string_buffer_append_format(
                    &buffer,
                    "%s::%s",
                    value.as.object->class_name,
                    value.as.object->enum_case_name
                );
            } else {
                ptn_string_buffer_append_format(&buffer, "of type %s", value.as.object->class_name);
            }
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

static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand_with_runtime(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
);

static PTN_UNUSED char *ptn_dynamic_variable_name(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);

    switch (value.type) {
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_RESOURCE:
            return ptn_value_to_string(value);
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION: {
            PtnStringOperand string = ptn_value_to_string_operand_with_runtime(runtime, value, line);
            char *result = ptn_duplicate_string_len(string.data, string.len);
            ptn_string_operand_free(string);
            return result;
        }
        case PTN_ARRAY:
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

static PTN_UNUSED char *ptn_dynamic_property_name_with_array_warning(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    int emit_array_warning
) {
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
    if (emit_array_warning && value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
    }
    PtnStringOperand string = ptn_value_to_string_operand_with_runtime(runtime, value, line);
    char *result = ptn_duplicate_string_len(string.data, string.len);
    ptn_string_operand_free(string);
    return result;
}

static PTN_UNUSED char *ptn_dynamic_property_name(PtnRuntime *runtime, PtnValue value, size_t line) {
    return ptn_dynamic_property_name_with_array_warning(runtime, value, line, 1);
}

static PTN_UNUSED char *ptn_dynamic_property_name_without_array_warning(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    return ptn_dynamic_property_name_with_array_warning(runtime, value, line, 0);
}

static PTN_UNUSED char *ptn_dynamic_property_name_for_read(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    size_t *len_out,
    int emit_array_warning
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_STRING) {
        if (
            value.as.string.len > 0 &&
            value.as.string.data[0] != '\0' &&
            ptn_string_has_embedded_nul(value.as.string)
        ) {
            ptn_emit_type_error(
                &runtime->diagnostics,
                "Unsupported dynamic property name containing embedded NUL"
            );
            exit(255);
        }
        *len_out = value.as.string.len;
        return ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len);
    }

    char *name = ptn_dynamic_property_name_with_array_warning(runtime, value, line, emit_array_warning);
    *len_out = strlen(name);
    return name;
}

static PTN_UNUSED char *ptn_dynamic_property_name_for_write(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    size_t *len_out,
    int emit_array_warning
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_STRING) {
        if (
            value.as.string.len > 0 &&
            value.as.string.data[0] != '\0' &&
            ptn_string_has_embedded_nul(value.as.string)
        ) {
            ptn_emit_type_error(
                &runtime->diagnostics,
                "Unsupported dynamic property name containing embedded NUL"
            );
            exit(255);
        }
        *len_out = value.as.string.len;
        return ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len);
    }

    char *name = ptn_dynamic_property_name_with_array_warning(runtime, value, line, emit_array_warning);
    *len_out = strlen(name);
    return name;
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
        if (callable.as.closure->has_wrapped_callable) {
            return ptn_callable_output_name(callable.as.closure->wrapped_callable);
        }
        return ptn_duplicate_string(callable.as.closure->display_name);
    }
    if (callable.type == PTN_OBJECT || callable.type == PTN_EXCEPTION) {
        const char *class_name = callable.type == PTN_OBJECT
            ? callable.as.object->class_name
            : callable.as.exception->class_name;
        size_t class_len = strlen(class_name);
        const char *invoke_suffix = "::__invoke";
        size_t suffix_len = strlen(invoke_suffix);
        if (class_len > SIZE_MAX - suffix_len - 1) {
            ptn_abort_out_of_memory();
        }
        char *name = malloc(class_len + suffix_len + 1);
        if (name == NULL) {
            ptn_abort_out_of_memory();
        }
        memcpy(name, class_name, class_len);
        memcpy(name + class_len, invoke_suffix, suffix_len);
        name[class_len + suffix_len] = '\0';
        return name;
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

static PTN_UNUSED PtnValue ptn_callable_output_name_value(PtnValue callable) {
    callable = ptn_value_deref(callable);
    if (callable.type == PTN_CLOSURE && callable.as.closure->has_wrapped_callable) {
        return ptn_callable_output_name_value(callable.as.closure->wrapped_callable);
    }
    if (callable.type == PTN_STRING) {
        return ptn_owned_string_len(
            ptn_duplicate_string_len((const char *)callable.as.string.data, callable.as.string.len),
            callable.as.string.len
        );
    }
    if (callable.type == PTN_ARRAY && callable.as.array->len == 2) {
        PtnArrayKey scope_key = ptn_array_int_key(0);
        PtnArrayKey method_key = ptn_array_int_key(1);
        PtnArrayEntry *scope_entry = ptn_array_entry_for_key(callable.as.array, scope_key);
        PtnArrayEntry *method_entry = ptn_array_entry_for_key(callable.as.array, method_key);
        ptn_array_key_free(scope_key);
        ptn_array_key_free(method_key);
        if (scope_entry != NULL && method_entry != NULL) {
            PtnValue scope = ptn_value_deref(scope_entry->value);
            PtnValue method = ptn_value_deref(method_entry->value);
            const char *scope_data = NULL;
            size_t scope_len = 0;
            if (scope.type == PTN_OBJECT) {
                scope_data = scope.as.object->class_name;
                scope_len = strlen(scope_data);
            } else if (scope.type == PTN_EXCEPTION) {
                scope_data = scope.as.exception->class_name;
                scope_len = strlen(scope_data);
            } else if (scope.type == PTN_CLOSURE) {
                scope_data = "Closure";
                scope_len = strlen(scope_data);
            } else if (scope.type == PTN_STRING) {
                scope_data = (const char *)scope.as.string.data;
                scope_len = scope.as.string.len;
            }
            if (scope_data != NULL && method.type == PTN_STRING) {
                size_t method_len = method.as.string.len;
                if (scope_len > SIZE_MAX - method_len - 3) {
                    ptn_abort_out_of_memory();
                }
                char *name = malloc(scope_len + method_len + 3);
                if (name == NULL) {
                    ptn_abort_out_of_memory();
                }
                memcpy(name, scope_data, scope_len);
                memcpy(name + scope_len, "::", 2);
                memcpy(name + scope_len + 2, method.as.string.data, method_len);
                name[scope_len + method_len + 2] = '\0';
                return ptn_owned_string_len(name, scope_len + method_len + 2);
            }
        }
    }
    return ptn_owned_string(ptn_callable_output_name(callable));
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

static PTN_UNUSED PtnStringOperand ptn_string_operand_ensure_owned(PtnStringOperand operand) {
    if (operand.owned != NULL) {
        return operand;
    }
    return ptn_string_operand_owned_len(
        ptn_duplicate_string_len(operand.data, operand.len),
        operand.len
    );
}

static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand(PtnValue value) {
    value = ptn_value_deref(value);
    char buffer[PTN_FLOAT_FORMAT_BUFFER_SIZE];
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
            return ptn_exception_to_string_operand(NULL, value.as.exception);
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
    int has_to_string = 0;
    if (runtime != NULL && runtime->declared_method_exists != NULL && value.type == PTN_OBJECT) {
        has_to_string = runtime->declared_method_exists(value.as.object->class_name, "__toString");
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        has_to_string = has_to_string || ptn_internal_class_method_exists(value.as.object->class_name, "__toString");
#endif
    }
    if (
        runtime == NULL ||
        runtime->method_dispatch == NULL ||
        value.type != PTN_OBJECT ||
        !has_to_string
    ) {
        return 0;
    }

    PtnException *active_exception_before = runtime->exceptions->active_exception;
    PtnValue result = runtime->method_dispatch(runtime, value, "__toString", 0, NULL, line);
    if (runtime->exceptions->active_exception != active_exception_before) {
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
    PtnStringOperand result_string = ptn_value_to_string_operand(ptn_value_deref(result));
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
        return ptn_exception_to_string_operand(runtime, resolved.as.exception);
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
    if (resolved.type == PTN_FLOAT) {
        char buffer[PTN_FLOAT_FORMAT_BUFFER_SIZE];
        ptn_format_runtime_scalar_float(runtime, resolved.as.floating, buffer, sizeof(buffer));
        size_t len = strlen(buffer);
        return ptn_string_operand_owned_len(ptn_duplicate_string_len(buffer, len), len);
    }
    return ptn_value_to_string_operand(value);
}

static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand_with_runtime_skipping_current_trace_frame(
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
        return ptn_exception_to_string_operand(runtime, resolved.as.exception);
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
        ptn_throw_exception_at_without_current_trace_frame(runtime, "Error", message, runtime->source_path, line);
        free(message);
        return ptn_string_operand_borrowed("");
    }
    if (resolved.type == PTN_FLOAT) {
        char buffer[PTN_FLOAT_FORMAT_BUFFER_SIZE];
        ptn_format_runtime_scalar_float(runtime, resolved.as.floating, buffer, sizeof(buffer));
        size_t len = strlen(buffer);
        return ptn_string_operand_owned_len(ptn_duplicate_string_len(buffer, len), len);
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

static PTN_UNUSED void ptn_concat_enforce_memory_limit(
    PtnRuntime *runtime,
    size_t joined_len,
    size_t line
) {
    size_t limit = 0;
    if (!ptn_runtime_memory_limit_bytes(runtime, &limit) || limit == 0) {
        return;
    }

    size_t allocation_len = joined_len + 1;
    size_t peak_len = joined_len > SIZE_MAX - allocation_len
        ? SIZE_MAX
        : joined_len + allocation_len;
    if (peak_len <= limit) {
        return;
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Allowed memory size of %zu bytes exhausted (tried to allocate %zu bytes)",
        limit,
        allocation_len
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_fatal_error_at(runtime, message, runtime->source_path, line);
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
        if (i + 1 < count) {
            strings[i] = ptn_string_operand_ensure_owned(strings[i]);
        }
    }
    if (joined_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_concat_enforce_memory_limit(runtime, joined_len, operands[count - 1].line);
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
    if (value.type == PTN_FLOAT && isnan(value.as.floating)) {
        ptn_emit_nan_coercion_warning(runtime, "string", line);
    }
    if (value.type == PTN_OBJECT || value.type == PTN_CLOSURE || value.type == PTN_EXCEPTION) {
        if (value.type == PTN_EXCEPTION) {
            PtnStringOperand exception_string =
                ptn_exception_to_string_operand(runtime, value.as.exception);
            return ptn_owned_string_len(exception_string.owned, exception_string.len);
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
            ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
            return ptn_string("");
        }
        fprintf(stderr, "Fatal error: %s\n", message);
        exit(255);
    }

    if (runtime != NULL && value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
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

static PTN_UNUSED PtnValue ptn_cast_bool_with_runtime(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_FLOAT && isnan(value.as.floating)) {
        ptn_emit_nan_coercion_warning(runtime, "bool", line);
    }
    return ptn_bool(ptn_is_truthy(value));
}

typedef enum {
    PTN_CAST_TARGET_INT,
    PTN_CAST_TARGET_FLOAT,
    PTN_CAST_TARGET_STRING,
    PTN_CAST_TARGET_BOOL
} PtnCastTarget;

static PTN_UNUSED PtnValue ptn_cast_target(
    PtnRuntime *runtime,
    PtnValue value,
    PtnCastTarget target,
    size_t line
) {
    switch (target) {
        case PTN_CAST_TARGET_INT:
            return ptn_cast_int_with_runtime(runtime, value, line);
        case PTN_CAST_TARGET_FLOAT:
            return ptn_cast_float_with_runtime(runtime, value, line);
        case PTN_CAST_TARGET_STRING:
            return ptn_cast_string_with_runtime(runtime, value, line);
        case PTN_CAST_TARGET_BOOL:
            return ptn_cast_bool_with_runtime(runtime, value, line);
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
    return ptn_cast_target(runtime, value, target, line);
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
            return ptn_string(ptn_resource_is_open(value.as.resource) ? "resource" : "resource (closed)");
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

typedef struct {
    int attempted;
    int version;
    char dotted[32];
    char loaded[32];
} PtnLibxmlVersionInfo;

static PtnLibxmlVersionInfo ptn_libxml_version_info = {0};

static void ptn_libxml_format_version_info(PtnLibxmlVersionInfo *info, int version) {
    info->version = version;
    snprintf(info->loaded, sizeof(info->loaded), "%d", version);
    snprintf(
        info->dotted,
        sizeof(info->dotted),
        "%d.%d.%d",
        version / 10000,
        (version / 100) % 100,
        version % 100
    );
}

static int ptn_libxml_parse_version_string(const char *version) {
    if (version == NULL || version[0] == '\0') {
        return 0;
    }
    if (strchr(version, '.') == NULL) {
        int value = 0;
        for (size_t i = 0; version[i] != '\0'; i++) {
            if (!isdigit((unsigned char)version[i])) {
                return 0;
            }
            if (value > (INT_MAX - (version[i] - '0')) / 10) {
                return 0;
            }
            value = value * 10 + (version[i] - '0');
        }
        return value;
    }
    int parts[3] = {0, 0, 0};
    size_t part = 0;
    for (size_t i = 0; version[i] != '\0'; i++) {
        if (version[i] == '.') {
            if (++part >= 3) {
                return 0;
            }
            continue;
        }
        if (!isdigit((unsigned char)version[i])) {
            return 0;
        }
        if (parts[part] > (INT_MAX - (version[i] - '0')) / 10) {
            return 0;
        }
        parts[part] = parts[part] * 10 + (version[i] - '0');
    }
    return parts[0] * 10000 + parts[1] * 100 + parts[2];
}

static void *ptn_libxml_version_open_known_name(void) {
#if defined(_WIN32)
    return NULL;
#else
    const char *env_library = getenv("PTN_LIBXML2_LIBRARY");
    const char *names[] = {
        env_library == NULL ? "" : env_library,
#ifdef PTN_LIBXML2_DEFAULT_LIBRARY
        PTN_LIBXML2_DEFAULT_LIBRARY,
#endif
        "libxml2.so.2",
        "libxml2.so",
        NULL
    };
    for (size_t i = 0; names[i] != NULL; i++) {
        if (names[i][0] == '\0') {
            continue;
        }
        void *handle = dlopen(names[i], RTLD_LAZY | RTLD_LOCAL);
        if (handle != NULL) {
            return handle;
        }
    }
    return NULL;
#endif
}

static void *ptn_libxml_version_open_nix_store(void) {
#if defined(_WIN32)
    return NULL;
#else
    const char *patterns[] = {
        "/nix/store/*-libxml2-*/lib/libxml2.so.2*",
        "/nix/store/*-libxml2-*/lib/libxml2.so",
        NULL
    };
    for (size_t pattern = 0; patterns[pattern] != NULL; pattern++) {
        glob_t matches;
        memset(&matches, 0, sizeof(matches));
        void *handle = NULL;
        if (glob(patterns[pattern], 0, NULL, &matches) == 0) {
            for (size_t i = 0; i < matches.gl_pathc; i++) {
                handle = dlopen(matches.gl_pathv[i], RTLD_LAZY | RTLD_LOCAL);
                if (handle != NULL) {
                    break;
                }
            }
        }
        globfree(&matches);
        if (handle != NULL) {
            return handle;
        }
    }
    return NULL;
#endif
}

static const PtnLibxmlVersionInfo *ptn_libxml_version_info_load(void) {
    if (ptn_libxml_version_info.attempted) {
        return &ptn_libxml_version_info;
    }
    ptn_libxml_version_info.attempted = 1;
    ptn_libxml_format_version_info(&ptn_libxml_version_info, 20914);
#if !defined(_WIN32)
    void *handle = ptn_libxml_version_open_known_name();
    if (handle == NULL) {
        handle = ptn_libxml_version_open_nix_store();
    }
    if (handle != NULL) {
        const char **parser_version = (const char **)dlsym(handle, "xmlParserVersion");
        int parsed = parser_version == NULL ? 0 : ptn_libxml_parse_version_string(*parser_version);
        if (parsed > 0) {
            ptn_libxml_format_version_info(&ptn_libxml_version_info, parsed);
        }
    }
#endif
    return &ptn_libxml_version_info;
}

#define PTN_PCRE2_CONFIG_VERSION 11

typedef struct {
    int attempted;
    char version[64];
    int major;
    int minor;
} PtnPcre2VersionInfo;

static PtnPcre2VersionInfo ptn_pcre2_version_info;

static void ptn_pcre2_version_info_set_fallback(PtnPcre2VersionInfo *info) {
    snprintf(info->version, sizeof(info->version), "10.44");
    info->major = 10;
    info->minor = 44;
}

static int ptn_pcre2_parse_version(PtnPcre2VersionInfo *info) {
    char *major_end = NULL;
    errno = 0;
    long major = strtol(info->version, &major_end, 10);
    if (errno == ERANGE || major_end == info->version || major_end == NULL || *major_end != '.') {
        return 0;
    }
    char *minor_end = NULL;
    errno = 0;
    long minor = strtol(major_end + 1, &minor_end, 10);
    if (errno == ERANGE || minor_end == major_end + 1 || major < 0 || minor < 0) {
        return 0;
    }
    info->major = (int)major;
    info->minor = (int)minor;
    return 1;
}

#if !defined(_WIN32)
static void *ptn_pcre2_version_open_known_name(void) {
    const char *env_library = getenv("PTN_PCRE2_LIBRARY");
    const char *candidates[] = {
        env_library == NULL ? "" : env_library,
#ifdef PTN_PCRE2_DEFAULT_LIBRARY
        PTN_PCRE2_DEFAULT_LIBRARY,
#endif
        "libpcre2-8.so.0",
        "libpcre2-8.so",
        NULL
    };
    for (size_t i = 0; candidates[i] != NULL; i++) {
        if (candidates[i][0] == '\0') {
            continue;
        }
        void *handle = dlopen(candidates[i], RTLD_LAZY | RTLD_LOCAL);
        if (handle != NULL) {
            return handle;
        }
    }
    return NULL;
}
#endif

static const PtnPcre2VersionInfo *ptn_pcre2_version_info_load(void) {
    if (ptn_pcre2_version_info.attempted) {
        return &ptn_pcre2_version_info;
    }
    ptn_pcre2_version_info.attempted = 1;
    ptn_pcre2_version_info_set_fallback(&ptn_pcre2_version_info);
#if !defined(_WIN32)
    void *handle = ptn_pcre2_version_open_known_name();
    if (handle != NULL) {
        int (*config)(uint32_t, void *) = NULL;
        void *symbol = dlsym(handle, "pcre2_config_8");
        if (symbol != NULL) {
            memcpy(&config, &symbol, sizeof(config));
            char version[sizeof(ptn_pcre2_version_info.version)] = { 0 };
            if (config(PTN_PCRE2_CONFIG_VERSION, version) >= 0 && version[0] != '\0') {
                memcpy(ptn_pcre2_version_info.version, version, sizeof(ptn_pcre2_version_info.version));
                ptn_pcre2_version_info.version[sizeof(ptn_pcre2_version_info.version) - 1] = '\0';
                if (!ptn_pcre2_parse_version(&ptn_pcre2_version_info)) {
                    ptn_pcre2_version_info_set_fallback(&ptn_pcre2_version_info);
                }
            }
        }
    }
#endif
    return &ptn_pcre2_version_info;
}

static PTN_UNUSED int ptn_builtin_constant_value(const char *name, PtnValue *out) {
#define PTN_BUILTIN_INT_CONSTANT(constant_name, value) \
    if (strcmp(name, constant_name) == 0) { \
        *out = ptn_int(value); \
        return 1; \
    }
    PTN_BUILTIN_INT_CONSTANT("FILE_BINARY", 0)
    PTN_BUILTIN_INT_CONSTANT("FILE_TEXT", 0)
    PTN_BUILTIN_INT_CONSTANT("MT_RAND_MT19937", 0)
    PTN_BUILTIN_INT_CONSTANT("MT_RAND_PHP", 1)
    if (strcmp(name, "PHP_FLOAT_EPSILON") == 0) {
        *out = ptn_float(DBL_EPSILON);
        return 1;
    }
    PTN_BUILTIN_INT_CONSTANT("HASH_HMAC", PTN_HASH_HMAC)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_ASSOC", 1)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_NUM", 2)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_BOTH", 3)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_INTEGER", 1)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_FLOAT", 2)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_TEXT", 3)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_BLOB", 4)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_NULL", 5)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_OPEN_READONLY", 1)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_OPEN_READWRITE", 2)
    PTN_BUILTIN_INT_CONSTANT("SQLITE3_OPEN_CREATE", 4)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_URL", 10002)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_FILE", 10001)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_HEADER", 42)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_UPLOAD", 46)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_INFILE", 10009)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_POSTFIELDS", 10015)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_WRITEFUNCTION", 20011)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_READFUNCTION", 20012)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_RETURNTRANSFER", 19913)
    PTN_BUILTIN_INT_CONSTANT("CURLOPT_HEADERFUNCTION", 20079)
    PTN_BUILTIN_INT_CONSTANT("CURLINFO_EFFECTIVE_URL", 1048577)
    PTN_BUILTIN_INT_CONSTANT("CURLM_OK", 0)
    PTN_BUILTIN_INT_CONSTANT("CURLMSG_DONE", 1)
    PTN_BUILTIN_INT_CONSTANT("CURLE_OK", 0)
    PTN_BUILTIN_INT_CONSTANT("U_ZERO_ERROR", 0)
    PTN_BUILTIN_INT_CONSTANT("U_INVALID_CHAR_FOUND", 10)
    PTN_BUILTIN_INT_CONSTANT("GRAPHEME_EXTR_COUNT", PTN_GRAPHEME_EXTR_COUNT)
    PTN_BUILTIN_INT_CONSTANT("GRAPHEME_EXTR_MAXBYTES", PTN_GRAPHEME_EXTR_MAXBYTES)
    PTN_BUILTIN_INT_CONSTANT("GRAPHEME_EXTR_MAXCHARS", PTN_GRAPHEME_EXTR_MAXCHARS)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_UNKNOWN", 0)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_GIF", 1)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_JPEG", 2)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_PNG", 3)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_SWF", 4)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_PSD", 5)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_BMP", 6)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_TIFF_II", 7)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_TIFF_MM", 8)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_JPC", 9)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_JP2", 10)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_JPX", 11)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_JB2", 12)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_SWC", 13)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_IFF", 14)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_WBMP", 15)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_JPEG2000", 9)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_XBM", 16)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_ICO", 17)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_WEBP", 18)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_AVIF", 19)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_HEIF", 20)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_SVG", 21)
    PTN_BUILTIN_INT_CONSTANT("IMAGETYPE_COUNT", 22)
    PTN_BUILTIN_INT_CONSTANT("ICONV_MIME_DECODE_STRICT", PTN_ICONV_MIME_DECODE_STRICT)
    PTN_BUILTIN_INT_CONSTANT("ICONV_MIME_DECODE_CONTINUE_ON_ERROR", PTN_ICONV_MIME_DECODE_CONTINUE_ON_ERROR)
    if (strcmp(name, "INTL_ICU_VERSION") == 0) {
        *out = ptn_string(PTN_INTL_ICU_VERSION);
        return 1;
    }
    PTN_BUILTIN_INT_CONSTANT("T_INCLUDE", PTN_T_INCLUDE)
    PTN_BUILTIN_INT_CONSTANT("T_INCLUDE_ONCE", PTN_T_INCLUDE_ONCE)
    PTN_BUILTIN_INT_CONSTANT("T_EVAL", PTN_T_EVAL)
    PTN_BUILTIN_INT_CONSTANT("T_REQUIRE", PTN_T_REQUIRE)
    PTN_BUILTIN_INT_CONSTANT("T_REQUIRE_ONCE", PTN_T_REQUIRE_ONCE)
    PTN_BUILTIN_INT_CONSTANT("T_LOGICAL_OR", PTN_T_LOGICAL_OR)
    PTN_BUILTIN_INT_CONSTANT("T_LOGICAL_XOR", PTN_T_LOGICAL_XOR)
    PTN_BUILTIN_INT_CONSTANT("T_LOGICAL_AND", PTN_T_LOGICAL_AND)
    PTN_BUILTIN_INT_CONSTANT("T_PRINT", PTN_T_PRINT)
    PTN_BUILTIN_INT_CONSTANT("T_PLUS_EQUAL", PTN_T_PLUS_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_MINUS_EQUAL", PTN_T_MINUS_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_MUL_EQUAL", PTN_T_MUL_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_DIV_EQUAL", PTN_T_DIV_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_CONCAT_EQUAL", PTN_T_CONCAT_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_MOD_EQUAL", PTN_T_MOD_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_AND_EQUAL", PTN_T_AND_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_OR_EQUAL", PTN_T_OR_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_XOR_EQUAL", PTN_T_XOR_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_SL_EQUAL", PTN_T_SL_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_SR_EQUAL", PTN_T_SR_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_BOOLEAN_OR", PTN_T_BOOLEAN_OR)
    PTN_BUILTIN_INT_CONSTANT("T_BOOLEAN_AND", PTN_T_BOOLEAN_AND)
    PTN_BUILTIN_INT_CONSTANT("T_IS_EQUAL", PTN_T_IS_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_IS_NOT_EQUAL", PTN_T_IS_NOT_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_IS_IDENTICAL", PTN_T_IS_IDENTICAL)
    PTN_BUILTIN_INT_CONSTANT("T_IS_NOT_IDENTICAL", PTN_T_IS_NOT_IDENTICAL)
    PTN_BUILTIN_INT_CONSTANT("T_IS_SMALLER_OR_EQUAL", PTN_T_IS_SMALLER_OR_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_IS_GREATER_OR_EQUAL", PTN_T_IS_GREATER_OR_EQUAL)
    PTN_BUILTIN_INT_CONSTANT("T_SL", PTN_T_SL)
    PTN_BUILTIN_INT_CONSTANT("T_SR", PTN_T_SR)
    PTN_BUILTIN_INT_CONSTANT("T_INC", PTN_T_INC)
    PTN_BUILTIN_INT_CONSTANT("T_DEC", PTN_T_DEC)
    PTN_BUILTIN_INT_CONSTANT("T_INT_CAST", PTN_T_INT_CAST)
    PTN_BUILTIN_INT_CONSTANT("T_DOUBLE_CAST", PTN_T_DOUBLE_CAST)
    PTN_BUILTIN_INT_CONSTANT("T_STRING_CAST", PTN_T_STRING_CAST)
    PTN_BUILTIN_INT_CONSTANT("T_ARRAY_CAST", PTN_T_ARRAY_CAST)
    PTN_BUILTIN_INT_CONSTANT("T_OBJECT_CAST", PTN_T_OBJECT_CAST)
    PTN_BUILTIN_INT_CONSTANT("T_BOOL_CAST", PTN_T_BOOL_CAST)
    PTN_BUILTIN_INT_CONSTANT("T_UNSET_CAST", PTN_T_UNSET_CAST)
    PTN_BUILTIN_INT_CONSTANT("T_NEW", PTN_T_NEW)
    PTN_BUILTIN_INT_CONSTANT("T_EXIT", PTN_T_EXIT)
    PTN_BUILTIN_INT_CONSTANT("T_IF", PTN_T_IF)
    PTN_BUILTIN_INT_CONSTANT("T_ELSEIF", PTN_T_ELSEIF)
    PTN_BUILTIN_INT_CONSTANT("T_ELSE", PTN_T_ELSE)
    PTN_BUILTIN_INT_CONSTANT("T_ENDIF", PTN_T_ENDIF)
    PTN_BUILTIN_INT_CONSTANT("T_LNUMBER", PTN_T_LNUMBER)
    PTN_BUILTIN_INT_CONSTANT("T_DNUMBER", PTN_T_DNUMBER)
    PTN_BUILTIN_INT_CONSTANT("T_STRING", PTN_T_STRING)
    PTN_BUILTIN_INT_CONSTANT("T_STRING_VARNAME", PTN_T_STRING_VARNAME)
    PTN_BUILTIN_INT_CONSTANT("T_VARIABLE", PTN_T_VARIABLE)
    PTN_BUILTIN_INT_CONSTANT("T_NUM_STRING", PTN_T_NUM_STRING)
    PTN_BUILTIN_INT_CONSTANT("T_INLINE_HTML", PTN_T_INLINE_HTML)
    PTN_BUILTIN_INT_CONSTANT("T_ENCAPSED_AND_WHITESPACE", PTN_T_ENCAPSED_AND_WHITESPACE)
    PTN_BUILTIN_INT_CONSTANT("T_CONSTANT_ENCAPSED_STRING", PTN_T_CONSTANT_ENCAPSED_STRING)
    PTN_BUILTIN_INT_CONSTANT("T_ECHO", PTN_T_ECHO)
    PTN_BUILTIN_INT_CONSTANT("T_DO", PTN_T_DO)
    PTN_BUILTIN_INT_CONSTANT("T_WHILE", PTN_T_WHILE)
    PTN_BUILTIN_INT_CONSTANT("T_ENDWHILE", PTN_T_ENDWHILE)
    PTN_BUILTIN_INT_CONSTANT("T_FOR", PTN_T_FOR)
    PTN_BUILTIN_INT_CONSTANT("T_ENDFOR", PTN_T_ENDFOR)
    PTN_BUILTIN_INT_CONSTANT("T_FOREACH", PTN_T_FOREACH)
    PTN_BUILTIN_INT_CONSTANT("T_ENDFOREACH", PTN_T_ENDFOREACH)
    PTN_BUILTIN_INT_CONSTANT("T_DECLARE", PTN_T_DECLARE)
    PTN_BUILTIN_INT_CONSTANT("T_ENDDECLARE", PTN_T_ENDDECLARE)
    PTN_BUILTIN_INT_CONSTANT("T_AS", PTN_T_AS)
    PTN_BUILTIN_INT_CONSTANT("T_SWITCH", PTN_T_SWITCH)
    PTN_BUILTIN_INT_CONSTANT("T_ENDSWITCH", PTN_T_ENDSWITCH)
    PTN_BUILTIN_INT_CONSTANT("T_CASE", PTN_T_CASE)
    PTN_BUILTIN_INT_CONSTANT("T_DEFAULT", PTN_T_DEFAULT)
    PTN_BUILTIN_INT_CONSTANT("T_BREAK", PTN_T_BREAK)
    PTN_BUILTIN_INT_CONSTANT("T_CONTINUE", PTN_T_CONTINUE)
    PTN_BUILTIN_INT_CONSTANT("T_FUNCTION", PTN_T_FUNCTION)
    PTN_BUILTIN_INT_CONSTANT("T_CONST", PTN_T_CONST)
    PTN_BUILTIN_INT_CONSTANT("T_RETURN", PTN_T_RETURN)
    PTN_BUILTIN_INT_CONSTANT("T_USE", PTN_T_USE)
    PTN_BUILTIN_INT_CONSTANT("T_GLOBAL", PTN_T_GLOBAL)
    PTN_BUILTIN_INT_CONSTANT("T_STATIC", PTN_T_STATIC)
    PTN_BUILTIN_INT_CONSTANT("T_VAR", PTN_T_VAR)
    PTN_BUILTIN_INT_CONSTANT("T_UNSET", PTN_T_UNSET)
    PTN_BUILTIN_INT_CONSTANT("T_ISSET", PTN_T_ISSET)
    PTN_BUILTIN_INT_CONSTANT("T_EMPTY", PTN_T_EMPTY)
    PTN_BUILTIN_INT_CONSTANT("T_CLASS", PTN_T_CLASS)
    PTN_BUILTIN_INT_CONSTANT("T_EXTENDS", PTN_T_EXTENDS)
    PTN_BUILTIN_INT_CONSTANT("T_INTERFACE", PTN_T_INTERFACE)
    PTN_BUILTIN_INT_CONSTANT("T_IMPLEMENTS", PTN_T_IMPLEMENTS)
    PTN_BUILTIN_INT_CONSTANT("T_OBJECT_OPERATOR", PTN_T_OBJECT_OPERATOR)
    PTN_BUILTIN_INT_CONSTANT("T_DOUBLE_ARROW", PTN_T_DOUBLE_ARROW)
    PTN_BUILTIN_INT_CONSTANT("T_LIST", PTN_T_LIST)
    PTN_BUILTIN_INT_CONSTANT("T_ARRAY", PTN_T_ARRAY)
    PTN_BUILTIN_INT_CONSTANT("T_CLASS_C", PTN_T_CLASS_C)
    PTN_BUILTIN_INT_CONSTANT("T_FUNC_C", PTN_T_FUNC_C)
    PTN_BUILTIN_INT_CONSTANT("T_PROPERTY_C", PTN_T_PROPERTY_C)
    PTN_BUILTIN_INT_CONSTANT("T_METHOD_C", PTN_T_METHOD_C)
    PTN_BUILTIN_INT_CONSTANT("T_LINE", PTN_T_LINE)
    PTN_BUILTIN_INT_CONSTANT("T_FILE", PTN_T_FILE)
    PTN_BUILTIN_INT_CONSTANT("T_COMMENT", PTN_T_COMMENT)
    PTN_BUILTIN_INT_CONSTANT("T_DOC_COMMENT", PTN_T_DOC_COMMENT)
    PTN_BUILTIN_INT_CONSTANT("T_OPEN_TAG", PTN_T_OPEN_TAG)
    PTN_BUILTIN_INT_CONSTANT("T_OPEN_TAG_WITH_ECHO", PTN_T_OPEN_TAG_WITH_ECHO)
    PTN_BUILTIN_INT_CONSTANT("T_CLOSE_TAG", PTN_T_CLOSE_TAG)
    PTN_BUILTIN_INT_CONSTANT("T_WHITESPACE", PTN_T_WHITESPACE)
    PTN_BUILTIN_INT_CONSTANT("T_START_HEREDOC", PTN_T_START_HEREDOC)
    PTN_BUILTIN_INT_CONSTANT("T_END_HEREDOC", PTN_T_END_HEREDOC)
    PTN_BUILTIN_INT_CONSTANT("T_DOLLAR_OPEN_CURLY_BRACES", PTN_T_DOLLAR_OPEN_CURLY_BRACES)
    PTN_BUILTIN_INT_CONSTANT("T_CURLY_OPEN", PTN_T_CURLY_OPEN)
    PTN_BUILTIN_INT_CONSTANT("T_PAAMAYIM_NEKUDOTAYIM", PTN_T_PAAMAYIM_NEKUDOTAYIM)
    PTN_BUILTIN_INT_CONSTANT("T_DOUBLE_COLON", PTN_T_DOUBLE_COLON)
    PTN_BUILTIN_INT_CONSTANT("T_ABSTRACT", PTN_T_ABSTRACT)
    PTN_BUILTIN_INT_CONSTANT("T_CATCH", PTN_T_CATCH)
    PTN_BUILTIN_INT_CONSTANT("T_FINAL", PTN_T_FINAL)
    PTN_BUILTIN_INT_CONSTANT("T_INSTANCEOF", PTN_T_INSTANCEOF)
    PTN_BUILTIN_INT_CONSTANT("T_PRIVATE", PTN_T_PRIVATE)
    PTN_BUILTIN_INT_CONSTANT("T_PROTECTED", PTN_T_PROTECTED)
    PTN_BUILTIN_INT_CONSTANT("T_PUBLIC", PTN_T_PUBLIC)
    PTN_BUILTIN_INT_CONSTANT("T_THROW", PTN_T_THROW)
    PTN_BUILTIN_INT_CONSTANT("T_TRY", PTN_T_TRY)
    PTN_BUILTIN_INT_CONSTANT("T_CLONE", PTN_T_CLONE)
    PTN_BUILTIN_INT_CONSTANT("T_HALT_COMPILER", PTN_T_HALT_COMPILER)
    PTN_BUILTIN_INT_CONSTANT("T_NAME_FULLY_QUALIFIED", PTN_T_NAME_FULLY_QUALIFIED)
    PTN_BUILTIN_INT_CONSTANT("T_NAME_RELATIVE", PTN_T_NAME_RELATIVE)
    PTN_BUILTIN_INT_CONSTANT("T_NAME_QUALIFIED", PTN_T_NAME_QUALIFIED)
    PTN_BUILTIN_INT_CONSTANT("T_NS_SEPARATOR", PTN_T_NS_SEPARATOR)
#undef PTN_BUILTIN_INT_CONSTANT
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
    if (strcmp(name, "OPENSSL_VERSION_NUMBER") == 0) {
        *out = ptn_int(PTN_OPENSSL_VERSION_NUMBER);
        return 1;
    }
    if (strcmp(name, "OPENSSL_VERSION_TEXT") == 0) {
        *out = ptn_string(PTN_OPENSSL_VERSION_TEXT);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ALGO_SHA1") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ALGO_MD5") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ALGO_SHA224") == 0) {
        *out = ptn_int(6);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ALGO_SHA256") == 0) {
        *out = ptn_int(7);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ALGO_SHA384") == 0) {
        *out = ptn_int(8);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ALGO_SHA512") == 0) {
        *out = ptn_int(9);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ENCODING_DER") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ENCODING_SMIME") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ENCODING_PEM") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "OPENSSL_RAW_DATA") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "OPENSSL_ZERO_PADDING") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "LIBXML_DOTTED_VERSION") == 0) {
        *out = ptn_string(ptn_libxml_version_info_load()->dotted);
        return 1;
    }
    if (strcmp(name, "LIBXML_VERSION") == 0) {
        *out = ptn_int(ptn_libxml_version_info_load()->version);
        return 1;
    }
    if (strcmp(name, "LIBXML_LOADED_VERSION") == 0) {
        *out = ptn_string(ptn_libxml_version_info_load()->loaded);
        return 1;
    }
    if (strcmp(name, "LIBXML_RECOVER") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "LIBXML_NOENT") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "LIBXML_DTDLOAD") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "LIBXML_DTDATTR") == 0) {
        *out = ptn_int(8);
        return 1;
    }
    if (strcmp(name, "LIBXML_DTDVALID") == 0) {
        *out = ptn_int(16);
        return 1;
    }
    if (strcmp(name, "LIBXML_NOERROR") == 0) {
        *out = ptn_int(32);
        return 1;
    }
    if (strcmp(name, "LIBXML_NOWARNING") == 0) {
        *out = ptn_int(64);
        return 1;
    }
    if (strcmp(name, "LIBXML_PEDANTIC") == 0) {
        *out = ptn_int(128);
        return 1;
    }
    if (strcmp(name, "LIBXML_NOBLANKS") == 0) {
        *out = ptn_int(256);
        return 1;
    }
    if (strcmp(name, "LIBXML_XINCLUDE") == 0) {
        *out = ptn_int(1024);
        return 1;
    }
    if (strcmp(name, "LIBXML_NONET") == 0) {
        *out = ptn_int(2048);
        return 1;
    }
    if (strcmp(name, "LIBXML_NSCLEAN") == 0) {
        *out = ptn_int(8192);
        return 1;
    }
    if (strcmp(name, "LIBXML_NOCDATA") == 0) {
        *out = ptn_int(16384);
        return 1;
    }
    if (strcmp(name, "LIBXML_COMPACT") == 0) {
        *out = ptn_int(65536);
        return 1;
    }
    if (strcmp(name, "LIBXML_PARSEHUGE") == 0) {
        *out = ptn_int(524288);
        return 1;
    }
    if (strcmp(name, "LIBXML_BIGLINES") == 0) {
        *out = ptn_int(4194304);
        return 1;
    }
    if (strcmp(name, "LIBXML_NOXMLDECL") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "LIBXML_NOEMPTYTAG") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "LIBXML_SCHEMA_CREATE") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "LIBXML_HTML_NODEFDTD") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "LIBXML_HTML_NOIMPLIED") == 0) {
        *out = ptn_int(8192);
        return 1;
    }
    if (strcmp(name, "Dom\\HTML_NO_DEFAULT_NS") == 0) {
        *out = ptn_int(2147483648LL);
        return 1;
    }
    if (strcmp(name, "LIBXML_ERR_NONE") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "LIBXML_ERR_WARNING") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "LIBXML_ERR_ERROR") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "LIBXML_ERR_FATAL") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "XML_SAX_IMPL") == 0) {
        *out = ptn_string("libxml");
        return 1;
    }
    if (strcmp(name, "XML_OPTION_CASE_FOLDING") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "XML_OPTION_TARGET_ENCODING") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "XML_OPTION_SKIP_TAGSTART") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "XML_OPTION_SKIP_WHITE") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "XML_OPTION_PARSE_HUGE") == 0) {
        *out = ptn_int(5);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_NONE") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_NO_MEMORY") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_SYNTAX") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_NO_ELEMENTS") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_INVALID_TOKEN") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_UNCLOSED_TOKEN") == 0) {
        *out = ptn_int(5);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_PARTIAL_CHAR") == 0) {
        *out = ptn_int(6);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_TAG_MISMATCH") == 0) {
        *out = ptn_int(7);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_DUPLICATE_ATTRIBUTE") == 0) {
        *out = ptn_int(8);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_JUNK_AFTER_DOC_ELEMENT") == 0) {
        *out = ptn_int(9);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_PARAM_ENTITY_REF") == 0) {
        *out = ptn_int(10);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_UNDEFINED_ENTITY") == 0) {
        *out = ptn_int(11);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_RECURSIVE_ENTITY_REF") == 0) {
        *out = ptn_int(12);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_ASYNC_ENTITY") == 0) {
        *out = ptn_int(13);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_BAD_CHAR_REF") == 0) {
        *out = ptn_int(14);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_BINARY_ENTITY_REF") == 0) {
        *out = ptn_int(15);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_ATTRIBUTE_EXTERNAL_ENTITY_REF") == 0) {
        *out = ptn_int(16);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_MISPLACED_XML_PI") == 0) {
        *out = ptn_int(17);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_UNKNOWN_ENCODING") == 0) {
        *out = ptn_int(18);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_INCORRECT_ENCODING") == 0) {
        *out = ptn_int(19);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_UNCLOSED_CDATA_SECTION") == 0) {
        *out = ptn_int(20);
        return 1;
    }
    if (strcmp(name, "XML_ERROR_EXTERNAL_ENTITY_HANDLING") == 0) {
        *out = ptn_int(21);
        return 1;
    }
    if (strcmp(name, "XML_ELEMENT_NODE") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "XML_ATTRIBUTE_NODE") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "XML_TEXT_NODE") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "XML_CDATA_SECTION_NODE") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "XML_ENTITY_REF_NODE") == 0) {
        *out = ptn_int(5);
        return 1;
    }
    if (strcmp(name, "XML_PI_NODE") == 0) {
        *out = ptn_int(7);
        return 1;
    }
    if (strcmp(name, "XML_COMMENT_NODE") == 0) {
        *out = ptn_int(8);
        return 1;
    }
    if (strcmp(name, "XML_DOCUMENT_NODE") == 0) {
        *out = ptn_int(9);
        return 1;
    }
    if (strcmp(name, "XML_DOCUMENT_FRAG_NODE") == 0) {
        *out = ptn_int(11);
        return 1;
    }
    if (strcmp(name, "DOM_INVALID_CHARACTER_ERR") == 0) {
        *out = ptn_int(5);
        return 1;
    }
    if (strcmp(name, "ASSERT_ACTIVE") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "ASSERT_CALLBACK") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "ASSERT_BAIL") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "ASSERT_WARNING") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "ASSERT_EXCEPTION") == 0) {
        *out = ptn_int(5);
        return 1;
    }
    if (strcmp(name, "PHP_SESSION_DISABLED") == 0) {
        *out = ptn_int(PTN_PHP_SESSION_DISABLED);
        return 1;
    }
    if (strcmp(name, "PHP_SESSION_NONE") == 0) {
        *out = ptn_int(PTN_PHP_SESSION_NONE);
        return 1;
    }
    if (strcmp(name, "PHP_SESSION_ACTIVE") == 0) {
        *out = ptn_int(PTN_PHP_SESSION_ACTIVE);
        return 1;
    }
    if (strcmp(name, "SID") == 0) {
        *out = ptn_string("");
        return 1;
    }
    if (strcmp(name, "DEBUG_BACKTRACE_PROVIDE_OBJECT") == 0) {
        *out = ptn_int(PTN_DEBUG_BACKTRACE_PROVIDE_OBJECT);
        return 1;
    }
    if (strcmp(name, "DEBUG_BACKTRACE_IGNORE_ARGS") == 0) {
        *out = ptn_int(PTN_DEBUG_BACKTRACE_IGNORE_ARGS);
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
    if (strcmp(name, "INI_SCANNER_NORMAL") == 0) {
        *out = ptn_int(PTN_INI_SCANNER_NORMAL);
        return 1;
    }
    if (strcmp(name, "INI_SCANNER_RAW") == 0) {
        *out = ptn_int(PTN_INI_SCANNER_RAW);
        return 1;
    }
    if (strcmp(name, "INI_SCANNER_TYPED") == 0) {
        *out = ptn_int(PTN_INI_SCANNER_TYPED);
        return 1;
    }
    if (strcmp(name, "BC_MATH_NUMBER") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "CAL_GREGORIAN") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "CAL_JULIAN") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "CAL_JEWISH") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "CAL_FRENCH") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "CAL_NUM_CALS") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "CAL_DOW_DAYNO") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "CAL_DOW_LONG") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "CAL_DOW_SHORT") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "CAL_MONTH_GREGORIAN_SHORT") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "CAL_MONTH_GREGORIAN_LONG") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "CAL_MONTH_JULIAN_SHORT") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "CAL_MONTH_JULIAN_LONG") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "CAL_MONTH_JEWISH") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "CAL_MONTH_FRENCH") == 0) {
        *out = ptn_int(5);
        return 1;
    }
    if (strcmp(name, "CAL_EASTER_DEFAULT") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "CAL_EASTER_ROMAN") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "CAL_EASTER_ALWAYS_GREGORIAN") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "CAL_EASTER_ALWAYS_JULIAN") == 0) {
        *out = ptn_int(3);
        return 1;
    }
    if (strcmp(name, "CAL_JEWISH_ADD_ALAFIM_GERESH") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "CAL_JEWISH_ADD_ALAFIM") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "CAL_JEWISH_ADD_GERESHAYIM") == 0) {
        *out = ptn_int(8);
        return 1;
    }
    if (strcmp(name, "TCP_NODELAY") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "AF_UNIX") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "AF_INET") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "AF_INET6") == 0) {
        *out = ptn_int(10);
        return 1;
    }
    if (strcmp(name, "SOCK_STREAM") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "SOCK_DGRAM") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "SOL_TCP") == 0) {
        *out = ptn_int(6);
        return 1;
    }
    if (strcmp(name, "SOL_UDP") == 0) {
        *out = ptn_int(17);
        return 1;
    }
    if (strcmp(name, "SOL_SOCKET") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "IPPROTO_IP") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "IPPROTO_IPV6") == 0) {
        *out = ptn_int(41);
        return 1;
    }
    if (strcmp(name, "MCAST_JOIN_GROUP") == 0) {
        *out = ptn_int(42);
        return 1;
    }
    if (strcmp(name, "SO_REUSEPORT") == 0) {
        *out = ptn_int(15);
        return 1;
    }
    if (strcmp(name, "SO_REUSEADDR") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "SO_ERROR") == 0) {
        *out = ptn_int(4);
        return 1;
    }
    if (strcmp(name, "SO_LINGER") == 0) {
        *out = ptn_int(13);
        return 1;
    }
    if (strcmp(name, "PHP_NORMAL_READ") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "PHP_BINARY_READ") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "XSD_NAMESPACE") == 0) {
        *out = ptn_string("http://www.w3.org/2001/XMLSchema");
        return 1;
    }
    if (strcmp(name, "XSD_1999_NAMESPACE") == 0) {
        *out = ptn_string("http://www.w3.org/1999/XMLSchema");
        return 1;
    }
    static const struct {
        const char *name;
        int value;
    } soap_constants[] = {
        { "SOAP_1_1", 1 },
        { "SOAP_1_2", 2 },
        { "SOAP_PERSISTENCE_SESSION", 1 },
        { "SOAP_PERSISTENCE_REQUEST", 2 },
        { "SOAP_FUNCTIONS_ALL", 999 },
        { "SOAP_ENCODED", 1 },
        { "SOAP_LITERAL", 2 },
        { "SOAP_RPC", 1 },
        { "SOAP_DOCUMENT", 2 },
        { "SOAP_ACTOR_NEXT", 1 },
        { "SOAP_ACTOR_NONE", 2 },
        { "SOAP_ACTOR_UNLIMATERECEIVER", 3 },
        { "SOAP_COMPRESSION_ACCEPT", 32 },
        { "SOAP_COMPRESSION_GZIP", 0 },
        { "SOAP_COMPRESSION_DEFLATE", 16 },
        { "SOAP_AUTHENTICATION_BASIC", 0 },
        { "SOAP_AUTHENTICATION_DIGEST", 1 },
        { "SOAP_SINGLE_ELEMENT_ARRAYS", 1 },
        { "SOAP_WAIT_ONE_WAY_CALLS", 2 },
        { "SOAP_USE_XSI_ARRAY_TYPE", 4 },
        { "UNKNOWN_TYPE", 999998 },
        { "SOAP_ENC_ARRAY", 300 },
        { "SOAP_ENC_OBJECT", 301 },
        { "XSD_STRING", 101 },
        { "XSD_BOOLEAN", 102 },
        { "XSD_DECIMAL", 103 },
        { "XSD_FLOAT", 104 },
        { "XSD_DOUBLE", 105 },
        { "XSD_DURATION", 106 },
        { "XSD_DATETIME", 107 },
        { "XSD_TIME", 108 },
        { "XSD_DATE", 109 },
        { "XSD_GYEARMONTH", 110 },
        { "XSD_GYEAR", 111 },
        { "XSD_GMONTHDAY", 112 },
        { "XSD_GDAY", 113 },
        { "XSD_GMONTH", 114 },
        { "XSD_HEXBINARY", 115 },
        { "XSD_BASE64BINARY", 116 },
        { "XSD_ANYURI", 117 },
        { "XSD_QNAME", 118 },
        { "XSD_NOTATION", 119 },
        { "XSD_NORMALIZEDSTRING", 120 },
        { "XSD_TOKEN", 121 },
        { "XSD_LANGUAGE", 122 },
        { "XSD_NMTOKEN", 123 },
        { "XSD_NAME", 124 },
        { "XSD_NCNAME", 125 },
        { "XSD_ID", 126 },
        { "XSD_IDREF", 127 },
        { "XSD_IDREFS", 128 },
        { "XSD_ENTITY", 129 },
        { "XSD_ENTITIES", 130 },
        { "XSD_INTEGER", 131 },
        { "XSD_NONPOSITIVEINTEGER", 132 },
        { "XSD_NEGATIVEINTEGER", 133 },
        { "XSD_LONG", 134 },
        { "XSD_INT", 135 },
        { "XSD_SHORT", 136 },
        { "XSD_BYTE", 137 },
        { "XSD_NONNEGATIVEINTEGER", 138 },
        { "XSD_UNSIGNEDLONG", 139 },
        { "XSD_UNSIGNEDINT", 140 },
        { "XSD_UNSIGNEDSHORT", 141 },
        { "XSD_UNSIGNEDBYTE", 142 },
        { "XSD_POSITIVEINTEGER", 143 },
        { "XSD_NMTOKENS", 144 },
        { "XSD_ANYTYPE", 145 },
        { "XSD_ANYXML", 147 },
        { "APACHE_MAP", 200 },
        { "XSD_1999_TIMEINSTANT", 401 },
        { "WSDL_CACHE_NONE", 0 },
        { "WSDL_CACHE_DISK", 1 },
        { "WSDL_CACHE_MEMORY", 2 },
        { "WSDL_CACHE_BOTH", 3 },
        { "SOAP_SSL_METHOD_TLS", 0 },
        { "SOAP_SSL_METHOD_SSLv2", 1 },
        { "SOAP_SSL_METHOD_SSLv3", 2 },
        { "SOAP_SSL_METHOD_SSLv23", 3 },
    };
    for (size_t i = 0; i < sizeof(soap_constants) / sizeof(soap_constants[0]); i++) {
        if (strcmp(name, soap_constants[i].name) == 0) {
            *out = ptn_int(soap_constants[i].value);
            return 1;
        }
    }
    if (strcmp(name, "SOAP_1_1") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "SOAP_1_2") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "EXTR_OVERWRITE") == 0) {
        *out = ptn_int(PTN_EXTR_OVERWRITE);
        return 1;
    }
    if (strcmp(name, "EXTR_SKIP") == 0) {
        *out = ptn_int(PTN_EXTR_SKIP);
        return 1;
    }
    if (strcmp(name, "EXTR_PREFIX_SAME") == 0) {
        *out = ptn_int(PTN_EXTR_PREFIX_SAME);
        return 1;
    }
    if (strcmp(name, "EXTR_PREFIX_ALL") == 0) {
        *out = ptn_int(PTN_EXTR_PREFIX_ALL);
        return 1;
    }
    if (strcmp(name, "EXTR_PREFIX_INVALID") == 0) {
        *out = ptn_int(PTN_EXTR_PREFIX_INVALID);
        return 1;
    }
    if (strcmp(name, "EXTR_PREFIX_IF_EXISTS") == 0) {
        *out = ptn_int(PTN_EXTR_PREFIX_IF_EXISTS);
        return 1;
    }
    if (strcmp(name, "EXTR_IF_EXISTS") == 0) {
        *out = ptn_int(PTN_EXTR_IF_EXISTS);
        return 1;
    }
    if (strcmp(name, "EXTR_REFS") == 0) {
        *out = ptn_int(PTN_EXTR_REFS);
        return 1;
    }
    if (strcmp(name, "CONNECTION_NORMAL") == 0) {
        *out = ptn_int(0);
        return 1;
    }
    if (strcmp(name, "CONNECTION_ABORTED") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "CONNECTION_TIMEOUT") == 0) {
        *out = ptn_int(2);
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
    if (strcmp(name, "SCANDIR_SORT_ASCENDING") == 0) {
        *out = ptn_int(PTN_SCANDIR_SORT_ASCENDING);
        return 1;
    }
    if (strcmp(name, "SCANDIR_SORT_DESCENDING") == 0) {
        *out = ptn_int(PTN_SCANDIR_SORT_DESCENDING);
        return 1;
    }
    if (strcmp(name, "SCANDIR_SORT_NONE") == 0) {
        *out = ptn_int(PTN_SCANDIR_SORT_NONE);
        return 1;
    }
    if (strcmp(name, "PHP_ROUND_HALF_UP") == 0) {
        *out = ptn_int(PTN_PHP_ROUND_HALF_UP);
        return 1;
    }
    if (strcmp(name, "PHP_ROUND_HALF_DOWN") == 0) {
        *out = ptn_int(PTN_PHP_ROUND_HALF_DOWN);
        return 1;
    }
    if (strcmp(name, "PHP_ROUND_HALF_EVEN") == 0) {
        *out = ptn_int(PTN_PHP_ROUND_HALF_EVEN);
        return 1;
    }
    if (strcmp(name, "PHP_ROUND_HALF_ODD") == 0) {
        *out = ptn_int(PTN_PHP_ROUND_HALF_ODD);
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
    if (strcmp(name, "FILTER_VALIDATE_REGEXP") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_REGEXP);
        return 1;
    }
    if (strcmp(name, "FNM_NOESCAPE") == 0) {
        *out = ptn_int(PTN_FNM_NOESCAPE);
        return 1;
    }
    if (strcmp(name, "FNM_PATHNAME") == 0) {
        *out = ptn_int(PTN_FNM_PATHNAME);
        return 1;
    }
    if (strcmp(name, "FNM_PERIOD") == 0) {
        *out = ptn_int(PTN_FNM_PERIOD);
        return 1;
    }
    if (strcmp(name, "FNM_CASEFOLD") == 0) {
        *out = ptn_int(PTN_FNM_CASEFOLD);
        return 1;
    }
    if (strcmp(name, "GLOB_MARK") == 0) {
        *out = ptn_int(PTN_GLOB_MARK);
        return 1;
    }
    if (strcmp(name, "GLOB_NOSORT") == 0) {
        *out = ptn_int(PTN_GLOB_NOSORT);
        return 1;
    }
    if (strcmp(name, "GLOB_NOCHECK") == 0) {
        *out = ptn_int(PTN_GLOB_NOCHECK);
        return 1;
    }
    if (strcmp(name, "GLOB_NOESCAPE") == 0) {
        *out = ptn_int(PTN_GLOB_NOESCAPE);
        return 1;
    }
    if (strcmp(name, "GLOB_BRACE") == 0) {
        *out = ptn_int(PTN_GLOB_BRACE);
        return 1;
    }
    if (strcmp(name, "GLOB_ONLYDIR") == 0) {
        *out = ptn_int(PTN_GLOB_ONLYDIR);
        return 1;
    }
    if (strcmp(name, "GLOB_ERR") == 0) {
        *out = ptn_int(PTN_GLOB_ERR);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_CLEANABLE") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_CLEANABLE);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_FLUSHABLE") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_FLUSHABLE);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_REMOVABLE") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_REMOVABLE);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_STDFLAGS") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_STDFLAGS);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_WRITE") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_WRITE);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_START") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_START);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_CLEAN") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_CLEAN);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_FLUSH") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_FLUSH);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_FINAL") == 0 ||
        strcmp(name, "PHP_OUTPUT_HANDLER_END") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_FINAL);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_STARTED") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_STARTED);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_DISABLED") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_DISABLED);
        return 1;
    }
    if (strcmp(name, "PHP_OUTPUT_HANDLER_PROCESSED") == 0) {
        *out = ptn_int(PTN_PHP_OUTPUT_HANDLER_PROCESSED);
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
    if (strcmp(name, "JSON_ERROR_NONE") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_NONE);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_DEPTH") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_DEPTH);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_STATE_MISMATCH") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_STATE_MISMATCH);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_CTRL_CHAR") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_CTRL_CHAR);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_SYNTAX") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_SYNTAX);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_UTF8") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_UTF8);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_RECURSION") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_RECURSION);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_INF_OR_NAN") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_INF_OR_NAN);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_UNSUPPORTED_TYPE") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_UNSUPPORTED_TYPE);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_INVALID_PROPERTY_NAME") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_INVALID_PROPERTY_NAME);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_UTF16") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_UTF16);
        return 1;
    }
    if (strcmp(name, "JSON_ERROR_NON_BACKED_ENUM") == 0) {
        *out = ptn_int(PTN_JSON_ERROR_NON_BACKED_ENUM);
        return 1;
    }
    if (strcmp(name, "JSON_HEX_TAG") == 0) {
        *out = ptn_int(PTN_JSON_HEX_TAG);
        return 1;
    }
    if (strcmp(name, "JSON_HEX_AMP") == 0) {
        *out = ptn_int(PTN_JSON_HEX_AMP);
        return 1;
    }
    if (strcmp(name, "JSON_HEX_APOS") == 0) {
        *out = ptn_int(PTN_JSON_HEX_APOS);
        return 1;
    }
    if (strcmp(name, "JSON_HEX_QUOT") == 0) {
        *out = ptn_int(PTN_JSON_HEX_QUOT);
        return 1;
    }
    if (strcmp(name, "JSON_FORCE_OBJECT") == 0) {
        *out = ptn_int(PTN_JSON_FORCE_OBJECT);
        return 1;
    }
    if (strcmp(name, "JSON_NUMERIC_CHECK") == 0) {
        *out = ptn_int(PTN_JSON_NUMERIC_CHECK);
        return 1;
    }
    if (strcmp(name, "JSON_UNESCAPED_SLASHES") == 0) {
        *out = ptn_int(PTN_JSON_UNESCAPED_SLASHES);
        return 1;
    }
    if (strcmp(name, "JSON_PRETTY_PRINT") == 0) {
        *out = ptn_int(PTN_JSON_PRETTY_PRINT);
        return 1;
    }
    if (strcmp(name, "JSON_UNESCAPED_UNICODE") == 0) {
        *out = ptn_int(PTN_JSON_UNESCAPED_UNICODE);
        return 1;
    }
    if (strcmp(name, "JSON_PARTIAL_OUTPUT_ON_ERROR") == 0) {
        *out = ptn_int(PTN_JSON_PARTIAL_OUTPUT_ON_ERROR);
        return 1;
    }
    if (strcmp(name, "JSON_PRESERVE_ZERO_FRACTION") == 0) {
        *out = ptn_int(PTN_JSON_PRESERVE_ZERO_FRACTION);
        return 1;
    }
    if (strcmp(name, "JSON_UNESCAPED_LINE_TERMINATORS") == 0) {
        *out = ptn_int(PTN_JSON_UNESCAPED_LINE_TERMINATORS);
        return 1;
    }
    if (strcmp(name, "JSON_OBJECT_AS_ARRAY") == 0) {
        *out = ptn_int(PTN_JSON_OBJECT_AS_ARRAY);
        return 1;
    }
    if (strcmp(name, "JSON_BIGINT_AS_STRING") == 0) {
        *out = ptn_int(PTN_JSON_BIGINT_AS_STRING);
        return 1;
    }
    if (strcmp(name, "JSON_INVALID_UTF8_IGNORE") == 0) {
        *out = ptn_int(PTN_JSON_INVALID_UTF8_IGNORE);
        return 1;
    }
    if (strcmp(name, "JSON_INVALID_UTF8_SUBSTITUTE") == 0) {
        *out = ptn_int(PTN_JSON_INVALID_UTF8_SUBSTITUTE);
        return 1;
    }
    if (strcmp(name, "JSON_THROW_ON_ERROR") == 0) {
        *out = ptn_int(PTN_JSON_THROW_ON_ERROR);
        return 1;
    }
    if (strcmp(name, "PREG_PATTERN_ORDER") == 0) {
        *out = ptn_int(PTN_PREG_PATTERN_ORDER);
        return 1;
    }
    if (strcmp(name, "PREG_SET_ORDER") == 0) {
        *out = ptn_int(PTN_PREG_SET_ORDER);
        return 1;
    }
    if (strcmp(name, "PREG_OFFSET_CAPTURE") == 0) {
        *out = ptn_int(PTN_PREG_OFFSET_CAPTURE);
        return 1;
    }
    if (strcmp(name, "PREG_UNMATCHED_AS_NULL") == 0) {
        *out = ptn_int(PTN_PREG_UNMATCHED_AS_NULL);
        return 1;
    }
    if (strcmp(name, "PREG_SPLIT_NO_EMPTY") == 0) {
        *out = ptn_int(PTN_PREG_SPLIT_NO_EMPTY);
        return 1;
    }
    if (strcmp(name, "PREG_SPLIT_DELIM_CAPTURE") == 0) {
        *out = ptn_int(PTN_PREG_SPLIT_DELIM_CAPTURE);
        return 1;
    }
    if (strcmp(name, "PREG_SPLIT_OFFSET_CAPTURE") == 0) {
        *out = ptn_int(PTN_PREG_SPLIT_OFFSET_CAPTURE);
        return 1;
    }
    if (strcmp(name, "PREG_GREP_INVERT") == 0) {
        *out = ptn_int(PTN_PREG_GREP_INVERT);
        return 1;
    }
    if (strcmp(name, "PREG_NO_ERROR") == 0) {
        *out = ptn_int(PTN_PREG_NO_ERROR);
        return 1;
    }
    if (strcmp(name, "PREG_INTERNAL_ERROR") == 0) {
        *out = ptn_int(PTN_PREG_INTERNAL_ERROR);
        return 1;
    }
    if (strcmp(name, "PREG_BACKTRACK_LIMIT_ERROR") == 0) {
        *out = ptn_int(PTN_PREG_BACKTRACK_LIMIT_ERROR);
        return 1;
    }
    if (strcmp(name, "PREG_RECURSION_LIMIT_ERROR") == 0) {
        *out = ptn_int(PTN_PREG_RECURSION_LIMIT_ERROR);
        return 1;
    }
    if (strcmp(name, "PREG_BAD_UTF8_ERROR") == 0) {
        *out = ptn_int(PTN_PREG_BAD_UTF8_ERROR);
        return 1;
    }
    if (strcmp(name, "PREG_BAD_UTF8_OFFSET_ERROR") == 0) {
        *out = ptn_int(PTN_PREG_BAD_UTF8_OFFSET_ERROR);
        return 1;
    }
    if (strcmp(name, "PREG_JIT_STACKLIMIT_ERROR") == 0) {
        *out = ptn_int(PTN_PREG_JIT_STACKLIMIT_ERROR);
        return 1;
    }
    if (strcmp(name, "PCRE_VERSION") == 0) {
        *out = ptn_string(ptn_pcre2_version_info_load()->version);
        return 1;
    }
    if (strcmp(name, "PCRE_VERSION_MAJOR") == 0) {
        *out = ptn_int(ptn_pcre2_version_info_load()->major);
        return 1;
    }
    if (strcmp(name, "PCRE_VERSION_MINOR") == 0) {
        *out = ptn_int(ptn_pcre2_version_info_load()->minor);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_INT") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_INT);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_BOOLEAN") == 0 || strcmp(name, "FILTER_VALIDATE_BOOL") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_BOOLEAN);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_FLOAT") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_FLOAT);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_REGEXP") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_REGEXP);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_DOMAIN") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_DOMAIN);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_URL") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_URL);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_EMAIL") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_EMAIL);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_IP") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_IP);
        return 1;
    }
    if (strcmp(name, "FILTER_VALIDATE_MAC") == 0) {
        *out = ptn_int(PTN_FILTER_VALIDATE_MAC);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_STRING") == 0 || strcmp(name, "FILTER_SANITIZE_STRIPPED") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_STRING);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_ENCODED") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_ENCODED);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_SPECIAL_CHARS") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_SPECIAL_CHARS);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_FULL_SPECIAL_CHARS") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_FULL_SPECIAL_CHARS);
        return 1;
    }
    if (strcmp(name, "FILTER_UNSAFE_RAW") == 0 || strcmp(name, "FILTER_DEFAULT") == 0) {
        *out = ptn_int(PTN_FILTER_UNSAFE_RAW);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_EMAIL") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_EMAIL);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_URL") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_URL);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_NUMBER_INT") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_NUMBER_INT);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_NUMBER_FLOAT") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_NUMBER_FLOAT);
        return 1;
    }
    if (strcmp(name, "FILTER_SANITIZE_ADD_SLASHES") == 0) {
        *out = ptn_int(PTN_FILTER_SANITIZE_ADD_SLASHES);
        return 1;
    }
    if (strcmp(name, "FILTER_CALLBACK") == 0) {
        *out = ptn_int(PTN_FILTER_CALLBACK);
        return 1;
    }
    if (strcmp(name, "FILTER_REQUIRE_ARRAY") == 0) {
        *out = ptn_int(PTN_FILTER_REQUIRE_ARRAY);
        return 1;
    }
    if (strcmp(name, "FILTER_REQUIRE_SCALAR") == 0) {
        *out = ptn_int(PTN_FILTER_REQUIRE_SCALAR);
        return 1;
    }
    if (strcmp(name, "FILTER_FORCE_ARRAY") == 0) {
        *out = ptn_int(PTN_FILTER_FORCE_ARRAY);
        return 1;
    }
    if (strcmp(name, "FILTER_NULL_ON_FAILURE") == 0) {
        *out = ptn_int(PTN_FILTER_NULL_ON_FAILURE);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ALLOW_OCTAL") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ALLOW_OCTAL);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ALLOW_HEX") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ALLOW_HEX);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_STRIP_LOW") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_STRIP_LOW);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_STRIP_HIGH") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_STRIP_HIGH);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ENCODE_LOW") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ENCODE_LOW);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ENCODE_HIGH") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ENCODE_HIGH);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ENCODE_AMP") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ENCODE_AMP);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_NO_ENCODE_QUOTES") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_NO_ENCODE_QUOTES);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_EMPTY_STRING_NULL") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_EMPTY_STRING_NULL);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_STRIP_BACKTICK") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_STRIP_BACKTICK);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ALLOW_FRACTION") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ALLOW_FRACTION);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ALLOW_THOUSAND") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ALLOW_THOUSAND);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_ALLOW_SCIENTIFIC") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_ALLOW_SCIENTIFIC);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_PATH_REQUIRED") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_PATH_REQUIRED);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_QUERY_REQUIRED") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_QUERY_REQUIRED);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_IPV4") == 0 || strcmp(name, "FILTER_FLAG_HOSTNAME") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_HOSTNAME);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_EMAIL_UNICODE") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_EMAIL_UNICODE);
        return 1;
    }
    if (strcmp(name, "FILTER_THROW_ON_FAILURE") == 0) {
        *out = ptn_int(PTN_FILTER_THROW_ON_FAILURE);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_IPV6") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_IPV6);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_NO_RES_RANGE") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_NO_RES_RANGE);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_NO_PRIV_RANGE") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_NO_PRIV_RANGE);
        return 1;
    }
    if (strcmp(name, "FILTER_FLAG_GLOBAL_RANGE") == 0) {
        *out = ptn_int(PTN_FILTER_FLAG_GLOBAL_RANGE);
        return 1;
    }
    if (strcmp(name, "INPUT_GET") == 0) {
        *out = ptn_int(PTN_INPUT_GET);
        return 1;
    }
    if (strcmp(name, "INPUT_POST") == 0) {
        *out = ptn_int(PTN_INPUT_POST);
        return 1;
    }
    if (strcmp(name, "INPUT_COOKIE") == 0) {
        *out = ptn_int(PTN_INPUT_COOKIE);
        return 1;
    }
    if (strcmp(name, "INPUT_ENV") == 0) {
        *out = ptn_int(PTN_INPUT_ENV);
        return 1;
    }
    if (strcmp(name, "INPUT_SERVER") == 0) {
        *out = ptn_int(PTN_INPUT_SERVER);
        return 1;
    }
    if (strcmp(name, "MB_CASE_UPPER") == 0) {
        *out = ptn_int(PTN_MB_CASE_UPPER);
        return 1;
    }
    if (strcmp(name, "MB_CASE_LOWER") == 0) {
        *out = ptn_int(PTN_MB_CASE_LOWER);
        return 1;
    }
    if (strcmp(name, "MB_CASE_TITLE") == 0) {
        *out = ptn_int(PTN_MB_CASE_TITLE);
        return 1;
    }
    if (strcmp(name, "MB_CASE_FOLD") == 0) {
        *out = ptn_int(PTN_MB_CASE_FOLD);
        return 1;
    }
    if (strcmp(name, "MB_CASE_UPPER_SIMPLE") == 0) {
        *out = ptn_int(PTN_MB_CASE_UPPER_SIMPLE);
        return 1;
    }
    if (strcmp(name, "MB_CASE_LOWER_SIMPLE") == 0) {
        *out = ptn_int(PTN_MB_CASE_LOWER_SIMPLE);
        return 1;
    }
    if (strcmp(name, "MB_CASE_TITLE_SIMPLE") == 0) {
        *out = ptn_int(PTN_MB_CASE_TITLE_SIMPLE);
        return 1;
    }
    if (strcmp(name, "MB_CASE_FOLD_SIMPLE") == 0) {
        *out = ptn_int(PTN_MB_CASE_FOLD_SIMPLE);
        return 1;
    }
    if (strcmp(name, "MB_ONIGURUMA_VERSION") == 0) {
        *out = ptn_string(PTN_MB_ONIGURUMA_VERSION);
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
    if (strcmp(name, "PHP_URL_SCHEME") == 0) {
        *out = ptn_int(PTN_PHP_URL_SCHEME);
        return 1;
    }
    if (strcmp(name, "PHP_URL_HOST") == 0) {
        *out = ptn_int(PTN_PHP_URL_HOST);
        return 1;
    }
    if (strcmp(name, "PHP_URL_PORT") == 0) {
        *out = ptn_int(PTN_PHP_URL_PORT);
        return 1;
    }
    if (strcmp(name, "PHP_URL_USER") == 0) {
        *out = ptn_int(PTN_PHP_URL_USER);
        return 1;
    }
    if (strcmp(name, "PHP_URL_PASS") == 0) {
        *out = ptn_int(PTN_PHP_URL_PASS);
        return 1;
    }
    if (strcmp(name, "PHP_URL_PATH") == 0) {
        *out = ptn_int(PTN_PHP_URL_PATH);
        return 1;
    }
    if (strcmp(name, "PHP_URL_QUERY") == 0) {
        *out = ptn_int(PTN_PHP_URL_QUERY);
        return 1;
    }
    if (strcmp(name, "PHP_URL_FRAGMENT") == 0) {
        *out = ptn_int(PTN_PHP_URL_FRAGMENT);
        return 1;
    }
    if (strcmp(name, "PHP_QUERY_RFC1738") == 0) {
        *out = ptn_int(PTN_PHP_QUERY_RFC1738);
        return 1;
    }
    if (strcmp(name, "PHP_QUERY_RFC3986") == 0) {
        *out = ptn_int(PTN_PHP_QUERY_RFC3986);
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
    if (strcmp(name, "CRYPT_SALT_LENGTH") == 0) {
        *out = ptn_int(PTN_CRYPT_SALT_LENGTH);
        return 1;
    }
    if (strcmp(name, "CRYPT_STD_DES") == 0) {
        *out = ptn_int(PTN_CRYPT_STD_DES);
        return 1;
    }
    if (strcmp(name, "CRYPT_EXT_DES") == 0) {
        *out = ptn_int(PTN_CRYPT_EXT_DES);
        return 1;
    }
    if (strcmp(name, "CRYPT_MD5") == 0) {
        *out = ptn_int(PTN_CRYPT_MD5);
        return 1;
    }
    if (strcmp(name, "CRYPT_BLOWFISH") == 0) {
        *out = ptn_int(PTN_CRYPT_BLOWFISH);
        return 1;
    }
    if (strcmp(name, "CRYPT_SHA256") == 0) {
        *out = ptn_int(PTN_CRYPT_SHA256);
        return 1;
    }
    if (strcmp(name, "CRYPT_SHA512") == 0) {
        *out = ptn_int(PTN_CRYPT_SHA512);
        return 1;
    }
    if (strcmp(name, "PASSWORD_BCRYPT") == 0) {
        *out = ptn_string(PTN_PASSWORD_BCRYPT);
        return 1;
    }
    if (strcmp(name, "PASSWORD_DEFAULT") == 0) {
        *out = ptn_string(PTN_PASSWORD_BCRYPT);
        return 1;
    }
    if (strcmp(name, "PASSWORD_BCRYPT_DEFAULT_COST") == 0) {
        *out = ptn_int(PTN_PASSWORD_BCRYPT_DEFAULT_COST);
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
    if (strcmp(name, "INFO_GENERAL") == 0) {
        *out = ptn_int(PTN_INFO_GENERAL);
        return 1;
    }
    if (strcmp(name, "INFO_CREDITS") == 0) {
        *out = ptn_int(PTN_INFO_CREDITS);
        return 1;
    }
    if (strcmp(name, "INFO_CONFIGURATION") == 0) {
        *out = ptn_int(PTN_INFO_CONFIGURATION);
        return 1;
    }
    if (strcmp(name, "INFO_MODULES") == 0) {
        *out = ptn_int(PTN_INFO_MODULES);
        return 1;
    }
    if (strcmp(name, "INFO_ENVIRONMENT") == 0) {
        *out = ptn_int(PTN_INFO_ENVIRONMENT);
        return 1;
    }
    if (strcmp(name, "INFO_VARIABLES") == 0) {
        *out = ptn_int(PTN_INFO_VARIABLES);
        return 1;
    }
    if (strcmp(name, "INFO_LICENSE") == 0) {
        *out = ptn_int(PTN_INFO_LICENSE);
        return 1;
    }
    if (strcmp(name, "INFO_ALL") == 0) {
        *out = ptn_int(PTN_INFO_ALL);
        return 1;
    }
    if (strcmp(name, "CREDITS_GROUP") == 0) {
        *out = ptn_int(PTN_CREDITS_GROUP);
        return 1;
    }
    if (strcmp(name, "CREDITS_GENERAL") == 0) {
        *out = ptn_int(PTN_CREDITS_GENERAL);
        return 1;
    }
    if (strcmp(name, "CREDITS_SAPI") == 0) {
        *out = ptn_int(PTN_CREDITS_SAPI);
        return 1;
    }
    if (strcmp(name, "CREDITS_MODULES") == 0) {
        *out = ptn_int(PTN_CREDITS_MODULES);
        return 1;
    }
    if (strcmp(name, "CREDITS_DOCS") == 0) {
        *out = ptn_int(PTN_CREDITS_DOCS);
        return 1;
    }
    if (strcmp(name, "CREDITS_FULLPAGE") == 0) {
        *out = ptn_int(PTN_CREDITS_FULLPAGE);
        return 1;
    }
    if (strcmp(name, "CREDITS_QA") == 0) {
        *out = ptn_int(PTN_CREDITS_QA);
        return 1;
    }
    if (strcmp(name, "CREDITS_WEB") == 0) {
        *out = ptn_int(PTN_CREDITS_WEB);
        return 1;
    }
    if (strcmp(name, "CREDITS_ALL") == 0) {
        *out = ptn_int(PTN_CREDITS_ALL);
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
    if (strcmp(name, "FILE_APPEND") == 0) {
        *out = ptn_int(PTN_FILE_APPEND);
        return 1;
    }
    if (strcmp(name, "FILE_NO_DEFAULT_CONTEXT") == 0) {
        *out = ptn_int(PTN_FILE_NO_DEFAULT_CONTEXT);
        return 1;
    }
    if (strcmp(name, "SCANDIR_SORT_ASCENDING") == 0) {
        *out = ptn_int(PTN_SCANDIR_SORT_ASCENDING);
        return 1;
    }
    if (strcmp(name, "SCANDIR_SORT_DESCENDING") == 0) {
        *out = ptn_int(PTN_SCANDIR_SORT_DESCENDING);
        return 1;
    }
    if (strcmp(name, "SCANDIR_SORT_NONE") == 0) {
        *out = ptn_int(PTN_SCANDIR_SORT_NONE);
        return 1;
    }
    if (strcmp(name, "LOCK_SH") == 0) {
        *out = ptn_int(PTN_LOCK_SH);
        return 1;
    }
    if (strcmp(name, "LOCK_EX") == 0) {
        *out = ptn_int(PTN_LOCK_EX);
        return 1;
    }
    if (strcmp(name, "LOCK_UN") == 0) {
        *out = ptn_int(PTN_LOCK_UN);
        return 1;
    }
    if (strcmp(name, "LOCK_NB") == 0) {
        *out = ptn_int(PTN_LOCK_NB);
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
    if (strcmp(name, "STREAM_FILTER_READ") == 0) {
        *out = ptn_int(PTN_STREAM_FILTER_READ);
        return 1;
    }
    if (strcmp(name, "STREAM_FILTER_WRITE") == 0) {
        *out = ptn_int(PTN_STREAM_FILTER_WRITE);
        return 1;
    }
    if (strcmp(name, "STREAM_FILTER_ALL") == 0) {
        *out = ptn_int(PTN_STREAM_FILTER_ALL);
        return 1;
    }
    if (strcmp(name, "DNS_A") == 0) {
        *out = ptn_int(PTN_DNS_A);
        return 1;
    }
    if (strcmp(name, "DNS_NS") == 0) {
        *out = ptn_int(PTN_DNS_NS);
        return 1;
    }
    if (strcmp(name, "DNS_CNAME") == 0) {
        *out = ptn_int(PTN_DNS_CNAME);
        return 1;
    }
    if (strcmp(name, "DNS_SOA") == 0) {
        *out = ptn_int(PTN_DNS_SOA);
        return 1;
    }
    if (strcmp(name, "DNS_PTR") == 0) {
        *out = ptn_int(PTN_DNS_PTR);
        return 1;
    }
    if (strcmp(name, "DNS_HINFO") == 0) {
        *out = ptn_int(PTN_DNS_HINFO);
        return 1;
    }
    if (strcmp(name, "DNS_CAA") == 0) {
        *out = ptn_int(PTN_DNS_CAA);
        return 1;
    }
    if (strcmp(name, "DNS_MX") == 0) {
        *out = ptn_int(PTN_DNS_MX);
        return 1;
    }
    if (strcmp(name, "DNS_TXT") == 0) {
        *out = ptn_int(PTN_DNS_TXT);
        return 1;
    }
    if (strcmp(name, "DNS_A6") == 0) {
        *out = ptn_int(PTN_DNS_A6);
        return 1;
    }
    if (strcmp(name, "DNS_SRV") == 0) {
        *out = ptn_int(PTN_DNS_SRV);
        return 1;
    }
    if (strcmp(name, "DNS_NAPTR") == 0) {
        *out = ptn_int(PTN_DNS_NAPTR);
        return 1;
    }
    if (strcmp(name, "DNS_AAAA") == 0) {
        *out = ptn_int(PTN_DNS_AAAA);
        return 1;
    }
    if (strcmp(name, "DNS_ALL") == 0) {
        *out = ptn_int(PTN_DNS_ALL);
        return 1;
    }
    if (strcmp(name, "DNS_ANY") == 0) {
        *out = ptn_int(PTN_DNS_ANY);
        return 1;
    }
    if (strcmp(name, "STREAM_IS_URL") == 0) {
        *out = ptn_int(PTN_STREAM_IS_URL);
        return 1;
    }
    if (strcmp(name, "STREAM_REPORT_ERRORS") == 0) {
        *out = ptn_int(PTN_STREAM_REPORT_ERRORS);
        return 1;
    }
    if (strcmp(name, "PSFS_ERR_FATAL") == 0) {
        *out = ptn_int(PTN_PSFS_ERR_FATAL);
        return 1;
    }
    if (strcmp(name, "PSFS_FEED_ME") == 0) {
        *out = ptn_int(PTN_PSFS_FEED_ME);
        return 1;
    }
    if (strcmp(name, "PSFS_PASS_ON") == 0) {
        *out = ptn_int(PTN_PSFS_PASS_ON);
        return 1;
    }
    if (strcmp(name, "STREAM_OOB") == 0) {
        *out = ptn_int(PTN_STREAM_OOB);
        return 1;
    }
    if (strcmp(name, "STREAM_PEEK") == 0) {
        *out = ptn_int(PTN_STREAM_PEEK);
        return 1;
    }
    if (strcmp(name, "ZLIB_ENCODING_RAW") == 0) {
        *out = ptn_int(PTN_ZLIB_ENCODING_RAW);
        return 1;
    }
    if (strcmp(name, "ZLIB_ENCODING_GZIP") == 0) {
        *out = ptn_int(PTN_ZLIB_ENCODING_GZIP);
        return 1;
    }
    if (strcmp(name, "ZLIB_ENCODING_DEFLATE") == 0) {
        *out = ptn_int(PTN_ZLIB_ENCODING_DEFLATE);
        return 1;
    }
    if (strcmp(name, "ZLIB_VERSION") == 0) {
        *out = ptn_string(PTN_ZLIB_VERSION);
        return 1;
    }
    if (strcmp(name, "ZLIB_VERNUM") == 0) {
        *out = ptn_int(PTN_ZLIB_VERNUM);
        return 1;
    }
    if (strcmp(name, "FORCE_GZIP") == 0) {
        *out = ptn_int(PTN_FORCE_GZIP);
        return 1;
    }
    if (strcmp(name, "FORCE_DEFLATE") == 0) {
        *out = ptn_int(PTN_FORCE_DEFLATE);
        return 1;
    }
    if (strcmp(name, "ZLIB_OK") == 0) {
        *out = ptn_int(PTN_ZLIB_OK);
        return 1;
    }
    if (strcmp(name, "ZLIB_STREAM_END") == 0) {
        *out = ptn_int(PTN_ZLIB_STREAM_END);
        return 1;
    }
    if (strcmp(name, "ZLIB_NO_FLUSH") == 0) {
        *out = ptn_int(PTN_ZLIB_NO_FLUSH);
        return 1;
    }
    if (strcmp(name, "ZLIB_PARTIAL_FLUSH") == 0) {
        *out = ptn_int(PTN_ZLIB_PARTIAL_FLUSH);
        return 1;
    }
    if (strcmp(name, "ZLIB_SYNC_FLUSH") == 0) {
        *out = ptn_int(PTN_ZLIB_SYNC_FLUSH);
        return 1;
    }
    if (strcmp(name, "ZLIB_FULL_FLUSH") == 0) {
        *out = ptn_int(PTN_ZLIB_FULL_FLUSH);
        return 1;
    }
    if (strcmp(name, "ZLIB_BLOCK") == 0) {
        *out = ptn_int(PTN_ZLIB_BLOCK);
        return 1;
    }
    if (strcmp(name, "ZLIB_FINISH") == 0) {
        *out = ptn_int(PTN_ZLIB_FINISH);
        return 1;
    }
    if (strcmp(name, "STREAM_PF_UNIX") == 0) {
        *out = ptn_int(PTN_STREAM_PF_UNIX);
        return 1;
    }
    if (strcmp(name, "STREAM_SOCK_STREAM") == 0) {
        *out = ptn_int(PTN_STREAM_SOCK_STREAM);
        return 1;
    }
    if (strcmp(name, "STREAM_IPPROTO_IP") == 0) {
        *out = ptn_int(PTN_STREAM_IPPROTO_IP);
        return 1;
    }
    if (strcmp(name, "STREAM_SHUT_RD") == 0) {
        *out = ptn_int(PTN_STREAM_SHUT_RD);
        return 1;
    }
    if (strcmp(name, "STREAM_SHUT_WR") == 0) {
        *out = ptn_int(PTN_STREAM_SHUT_WR);
        return 1;
    }
    if (strcmp(name, "STREAM_SHUT_RDWR") == 0) {
        *out = ptn_int(PTN_STREAM_SHUT_RDWR);
        return 1;
    }
    if (strcmp(name, "STREAM_CLIENT_PERSISTENT") == 0) {
        *out = ptn_int(PTN_STREAM_CLIENT_PERSISTENT);
        return 1;
    }
    if (strcmp(name, "STREAM_CLIENT_ASYNC_CONNECT") == 0) {
        *out = ptn_int(PTN_STREAM_CLIENT_ASYNC_CONNECT);
        return 1;
    }
    if (strcmp(name, "STREAM_CLIENT_CONNECT") == 0) {
        *out = ptn_int(PTN_STREAM_CLIENT_CONNECT);
        return 1;
    }
    if (strcmp(name, "STREAM_SERVER_BIND") == 0) {
        *out = ptn_int(PTN_STREAM_SERVER_BIND);
        return 1;
    }
    if (strcmp(name, "STREAM_SERVER_LISTEN") == 0) {
        *out = ptn_int(PTN_STREAM_SERVER_LISTEN);
        return 1;
    }
    if (strcmp(name, "DNS_A") == 0) {
        *out = ptn_int(1);
        return 1;
    }
    if (strcmp(name, "DNS_NS") == 0) {
        *out = ptn_int(2);
        return 1;
    }
    if (strcmp(name, "DNS_CNAME") == 0) {
        *out = ptn_int(16);
        return 1;
    }
    if (strcmp(name, "DNS_SOA") == 0) {
        *out = ptn_int(32);
        return 1;
    }
    if (strcmp(name, "DNS_PTR") == 0) {
        *out = ptn_int(2048);
        return 1;
    }
    if (strcmp(name, "DNS_HINFO") == 0) {
        *out = ptn_int(4096);
        return 1;
    }
    if (strcmp(name, "DNS_CAA") == 0) {
        *out = ptn_int(8192);
        return 1;
    }
    if (strcmp(name, "DNS_MX") == 0) {
        *out = ptn_int(16384);
        return 1;
    }
    if (strcmp(name, "DNS_TXT") == 0) {
        *out = ptn_int(32768);
        return 1;
    }
    if (strcmp(name, "DNS_A6") == 0) {
        *out = ptn_int(16777216);
        return 1;
    }
    if (strcmp(name, "DNS_SRV") == 0) {
        *out = ptn_int(33554432);
        return 1;
    }
    if (strcmp(name, "DNS_NAPTR") == 0) {
        *out = ptn_int(67108864);
        return 1;
    }
    if (strcmp(name, "DNS_AAAA") == 0) {
        *out = ptn_int(134217728);
        return 1;
    }
    if (strcmp(name, "DNS_ANY") == 0) {
        *out = ptn_int(268435456);
        return 1;
    }
    if (strcmp(name, "DNS_ALL") == 0) {
        *out = ptn_int(251721779);
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
    if (strcmp(name, "ABDAY_1") == 0) { *out = ptn_int(PTN_ABDAY_1); return 1; }
    if (strcmp(name, "ABDAY_2") == 0) { *out = ptn_int(PTN_ABDAY_2); return 1; }
    if (strcmp(name, "ABDAY_3") == 0) { *out = ptn_int(PTN_ABDAY_3); return 1; }
    if (strcmp(name, "ABDAY_4") == 0) { *out = ptn_int(PTN_ABDAY_4); return 1; }
    if (strcmp(name, "ABDAY_5") == 0) { *out = ptn_int(PTN_ABDAY_5); return 1; }
    if (strcmp(name, "ABDAY_6") == 0) { *out = ptn_int(PTN_ABDAY_6); return 1; }
    if (strcmp(name, "ABDAY_7") == 0) { *out = ptn_int(PTN_ABDAY_7); return 1; }
    if (strcmp(name, "DAY_1") == 0) { *out = ptn_int(PTN_DAY_1); return 1; }
    if (strcmp(name, "DAY_2") == 0) { *out = ptn_int(PTN_DAY_2); return 1; }
    if (strcmp(name, "DAY_3") == 0) { *out = ptn_int(PTN_DAY_3); return 1; }
    if (strcmp(name, "DAY_4") == 0) { *out = ptn_int(PTN_DAY_4); return 1; }
    if (strcmp(name, "DAY_5") == 0) { *out = ptn_int(PTN_DAY_5); return 1; }
    if (strcmp(name, "DAY_6") == 0) { *out = ptn_int(PTN_DAY_6); return 1; }
    if (strcmp(name, "DAY_7") == 0) { *out = ptn_int(PTN_DAY_7); return 1; }
    if (strcmp(name, "ABMON_1") == 0) { *out = ptn_int(PTN_ABMON_1); return 1; }
    if (strcmp(name, "ABMON_2") == 0) { *out = ptn_int(PTN_ABMON_2); return 1; }
    if (strcmp(name, "ABMON_3") == 0) { *out = ptn_int(PTN_ABMON_3); return 1; }
    if (strcmp(name, "ABMON_4") == 0) { *out = ptn_int(PTN_ABMON_4); return 1; }
    if (strcmp(name, "ABMON_5") == 0) { *out = ptn_int(PTN_ABMON_5); return 1; }
    if (strcmp(name, "ABMON_6") == 0) { *out = ptn_int(PTN_ABMON_6); return 1; }
    if (strcmp(name, "ABMON_7") == 0) { *out = ptn_int(PTN_ABMON_7); return 1; }
    if (strcmp(name, "ABMON_8") == 0) { *out = ptn_int(PTN_ABMON_8); return 1; }
    if (strcmp(name, "ABMON_9") == 0) { *out = ptn_int(PTN_ABMON_9); return 1; }
    if (strcmp(name, "ABMON_10") == 0) { *out = ptn_int(PTN_ABMON_10); return 1; }
    if (strcmp(name, "ABMON_11") == 0) { *out = ptn_int(PTN_ABMON_11); return 1; }
    if (strcmp(name, "ABMON_12") == 0) { *out = ptn_int(PTN_ABMON_12); return 1; }
    if (strcmp(name, "MON_1") == 0) { *out = ptn_int(PTN_MON_1); return 1; }
    if (strcmp(name, "MON_2") == 0) { *out = ptn_int(PTN_MON_2); return 1; }
    if (strcmp(name, "MON_3") == 0) { *out = ptn_int(PTN_MON_3); return 1; }
    if (strcmp(name, "MON_4") == 0) { *out = ptn_int(PTN_MON_4); return 1; }
    if (strcmp(name, "MON_5") == 0) { *out = ptn_int(PTN_MON_5); return 1; }
    if (strcmp(name, "MON_6") == 0) { *out = ptn_int(PTN_MON_6); return 1; }
    if (strcmp(name, "MON_7") == 0) { *out = ptn_int(PTN_MON_7); return 1; }
    if (strcmp(name, "MON_8") == 0) { *out = ptn_int(PTN_MON_8); return 1; }
    if (strcmp(name, "MON_9") == 0) { *out = ptn_int(PTN_MON_9); return 1; }
    if (strcmp(name, "MON_10") == 0) { *out = ptn_int(PTN_MON_10); return 1; }
    if (strcmp(name, "MON_11") == 0) { *out = ptn_int(PTN_MON_11); return 1; }
    if (strcmp(name, "MON_12") == 0) { *out = ptn_int(PTN_MON_12); return 1; }
    if (strcmp(name, "RADIXCHAR") == 0) { *out = ptn_int(PTN_RADIXCHAR); return 1; }
    if (strcmp(name, "THOUSEP") == 0) { *out = ptn_int(PTN_THOUSEP); return 1; }
    if (strcmp(name, "YESEXPR") == 0) { *out = ptn_int(PTN_YESEXPR); return 1; }
    if (strcmp(name, "NOEXPR") == 0) { *out = ptn_int(PTN_NOEXPR); return 1; }
    if (strcmp(name, "CODESET") == 0) { *out = ptn_int(PTN_CODESET); return 1; }
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
    if (strcmp(name, "DATE_RFC850") == 0) {
        *out = ptn_string("l, d-M-y H:i:s T");
        return 1;
    }
    if (strcmp(name, "DATE_RFC1036") == 0) {
        *out = ptn_string("D, d M y H:i:s O");
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
    if (strcmp(name, "SUNFUNCS_RET_TIMESTAMP") == 0) {
        *out = ptn_int(PTN_SUNFUNCS_RET_TIMESTAMP);
        return 1;
    }
    if (strcmp(name, "SUNFUNCS_RET_STRING") == 0) {
        *out = ptn_int(PTN_SUNFUNCS_RET_STRING);
        return 1;
    }
    if (strcmp(name, "SUNFUNCS_RET_DOUBLE") == 0) {
        *out = ptn_int(PTN_SUNFUNCS_RET_DOUBLE);
        return 1;
    }
    if (strcmp(name, "PHP_MAXPATHLEN") == 0) {
        *out = ptn_int(PTN_PHP_MAXPATHLEN);
        return 1;
    }
    if (strcmp(name, "PHP_BINARY") == 0) {
        *out = ptn_string(PTN_PHP_BINARY);
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

static PTN_UNUSED void ptn_format_var_dump_float(
    double value,
    int serialize_precision,
    char *buffer,
    size_t buffer_size
) {
    if (isnan(value)) {
        snprintf(buffer, buffer_size, "NAN");
        return;
    }
    if (isinf(value)) {
        snprintf(buffer, buffer_size, signbit(value) ? "-INF" : "INF");
        return;
    }

    if (serialize_precision >= 0) {
        int written = snprintf(buffer, buffer_size, "%.*g", serialize_precision, value);
        if (written < 0 || (size_t)written >= buffer_size) {
            ptn_abort_out_of_memory();
        }
        ptn_normalize_var_dump_exponent(buffer);
        ptn_var_dump_ensure_exponent_decimal(buffer);
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

static PTN_UNUSED PtnStringOperand ptn_runtime_global_constant_key_len(const char *name, size_t name_len) {
    char *key = ptn_duplicate_string_len(name, name_len);
    size_t namespace_len = (size_t)-1;
    for (size_t i = 0; i < name_len; i++) {
        if (key[i] == '\\') {
            namespace_len = i;
        }
    }
    if (namespace_len != (size_t)-1) {
        for (size_t i = 0; i < namespace_len; i++) {
            key[i] = ptn_ascii_lower_char(key[i]);
        }
    }
    return ptn_string_operand_owned_len(key, name_len);
}

static PTN_UNUSED char *ptn_runtime_global_constant_key(const char *name) {
    PtnStringOperand key = ptn_runtime_global_constant_key_len(name, strlen(name));
    return key.owned;
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
    char *class_name = malloc(class_len + 1);
    if (class_name == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(class_name, name, class_len);
    class_name[class_len] = '\0';

    const char *resolved_class_name =
        ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, 0);
    if (runtime->exceptions != NULL && runtime->exceptions->active_exception != NULL) {
        free(class_name);
        return 0;
    }
    const char *target_class_name = ptn_declared_class_canonical_name(resolved_class_name);
    const char *lookup_class_name = target_class_name;
    const char *access_scope = runtime == NULL ? NULL : runtime->current_class_name;
    while (lookup_class_name != NULL) {
        const char *metadata_declaring_class = lookup_class_name;
        int metadata_visibility_int = (int)PTN_PROPERTY_PUBLIC;
        int lookup_has_metadata = ptn_declared_class_constant_metadata(
            lookup_class_name,
            constant_name,
            &metadata_declaring_class,
            &metadata_visibility_int
        );
        char *key = ptn_class_constant_key(metadata_declaring_class, constant_name);
        PtnValue value;
        if (ptn_symbols_get(ptn_runtime_class_constant_table(runtime), key, &value)) {
            const char *declaring_class = lookup_has_metadata ? metadata_declaring_class : lookup_class_name;
            int visibility_int = lookup_has_metadata ? metadata_visibility_int : (int)PTN_PROPERTY_PUBLIC;
            if (!lookup_has_metadata && !ptn_property_class_names_equal(target_class_name, lookup_class_name)) {
                free(key);
                lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                continue;
            }
            PtnPropertyVisibility visibility = (PtnPropertyVisibility)visibility_int;
            if (visibility == PTN_PROPERTY_PRIVATE &&
                !ptn_property_class_names_equal(target_class_name, declaring_class)) {
                free(key);
                lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                continue;
            }
            if (!ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
                free(key);
                free(class_name);
                return 0;
            }
            *out = ptn_value_borrow(value);
            free(key);
            free(class_name);
            return 1;
        }
        if (runtime->class_constant_initializer != NULL &&
            runtime->class_constant_initializer(runtime, metadata_declaring_class, constant_name)) {
            if (runtime->exceptions != NULL && runtime->exceptions->active_exception != NULL) {
                free(key);
                free(class_name);
                return 0;
            }
            if (ptn_symbols_get(ptn_runtime_class_constant_table(runtime), key, &value)) {
                const char *declaring_class = lookup_has_metadata ? metadata_declaring_class : lookup_class_name;
                int visibility_int = lookup_has_metadata ? metadata_visibility_int : (int)PTN_PROPERTY_PUBLIC;
                if (!lookup_has_metadata && !ptn_property_class_names_equal(target_class_name, lookup_class_name)) {
                    free(key);
                    lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                    continue;
                }
                PtnPropertyVisibility visibility = (PtnPropertyVisibility)visibility_int;
                if (visibility == PTN_PROPERTY_PRIVATE &&
                    !ptn_property_class_names_equal(target_class_name, declaring_class)) {
                    free(key);
                    lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                    continue;
                }
                if (!ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
                    free(key);
                    free(class_name);
                    return 0;
                }
                *out = ptn_value_borrow(value);
                free(key);
                free(class_name);
                return 1;
            }
        }
        free(key);
        lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
    }

    const char *property_hook_type_case = ptn_ascii_case_equal(resolved_class_name, "PropertyHookType")
        ? ptn_property_hook_type_case_name(constant_name)
        : NULL;
    if (property_hook_type_case != NULL) {
        *out = ptn_builtin_enum_case_singleton(runtime, "PropertyHookType", property_hook_type_case);
        free(class_name);
        return 1;
    }
    const char *dom_adjacent_position_case = ptn_ascii_case_equal(resolved_class_name, "Dom\\AdjacentPosition")
        ? ptn_dom_adjacent_position_case_name(constant_name)
        : NULL;
    if (dom_adjacent_position_case != NULL) {
        *out = ptn_builtin_enum_case_singleton(runtime, "Dom\\AdjacentPosition", dom_adjacent_position_case);
        free(class_name);
        return 1;
    }
    if (ptn_builtin_class_constant_value_span(name, class_len, constant_name, out)) {
        free(class_name);
        return 1;
    }
    if (ptn_builtin_class_constant_value(resolved_class_name, constant_name, out)) {
        free(class_name);
        return 1;
    }
    free(class_name);
    return 0;
}

static PTN_UNUSED int ptn_runtime_constant_value_len(
    PtnRuntime *runtime,
    const char *name,
    size_t name_len,
    PtnValue *out
) {
    PtnStringOperand key = ptn_runtime_global_constant_key_len(name, name_len);
    int found = ptn_symbols_get_len(runtime->constants, key.data, key.len, out);
    ptn_string_operand_free(key);
    if (found) {
        return 1;
    }
    if (memchr(name, '\0', name_len) != NULL) {
        return 0;
    }
    if (ptn_ascii_case_equal(name, "true")) {
        *out = ptn_bool(1);
        return 1;
    }
    if (ptn_ascii_case_equal(name, "false")) {
        *out = ptn_bool(0);
        return 1;
    }
    if (ptn_ascii_case_equal(name, "null")) {
        *out = ptn_null();
        return 1;
    }
    if (ptn_builtin_constant_value(name, out)) {
        return 1;
    }
    return ptn_runtime_class_constant_value(runtime, name, out);
}

static PTN_UNUSED int ptn_runtime_constant_value(PtnRuntime *runtime, const char *name, PtnValue *out) {
    return ptn_runtime_constant_value_len(runtime, name, strlen(name), out);
}

static PTN_UNUSED int ptn_runtime_constant_is_defined(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    return ptn_runtime_constant_value(runtime, name, &value);
}

static PTN_UNUSED int ptn_runtime_constant_is_defined_len(PtnRuntime *runtime, const char *name, size_t name_len) {
    PtnValue value;
    return ptn_runtime_constant_value_len(runtime, name, name_len, &value);
}

static PTN_UNUSED int ptn_reserved_constant_name_len(const char *name, size_t name_len) {
    const char *reserved = "__COMPILER_HALT_OFFSET__";
    size_t reserved_len = strlen(reserved);
    return name_len == reserved_len && memcmp(name, reserved, reserved_len) == 0;
}

static PTN_UNUSED int ptn_reserved_constant_name(const char *name) {
    return ptn_reserved_constant_name_len(name, strlen(name));
}

typedef struct {
    PtnArray **items;
    size_t len;
    size_t capacity;
} PtnConstantArrayStack;

static PTN_UNUSED int ptn_constant_array_stack_contains(PtnConstantArrayStack *stack, PtnArray *array) {
    for (size_t i = 0; i < stack->len; i++) {
        if (stack->items[i] == array) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_constant_array_stack_push(PtnConstantArrayStack *stack, PtnArray *array) {
    if (stack->len == stack->capacity) {
        size_t next_capacity = stack->capacity == 0 ? 8 : stack->capacity * 2;
        if (next_capacity < stack->capacity) {
            ptn_abort_out_of_memory();
        }
        PtnArray **items = realloc(stack->items, next_capacity * sizeof(PtnArray *));
        if (items == NULL) {
            ptn_abort_out_of_memory();
        }
        stack->items = items;
        stack->capacity = next_capacity;
    }
    stack->items[stack->len++] = array;
}

static PTN_UNUSED void ptn_constant_array_stack_pop(PtnConstantArrayStack *stack) {
    if (stack->len != 0) {
        stack->len--;
    }
}

static PTN_UNUSED int ptn_constant_clone_value(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnConstantArrayStack *stack,
    PtnValue *out
);

static PTN_UNUSED int ptn_constant_clone_array(
    PtnRuntime *runtime,
    PtnArray *source,
    size_t line,
    PtnConstantArrayStack *stack,
    PtnValue *out
) {
    if (ptn_constant_array_stack_contains(stack, source)) {
        ptn_throw_exception_at(
            runtime,
            "ValueError",
            "define(): Argument #2 ($value) cannot be a recursive array",
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
        return 0;
    }

    ptn_constant_array_stack_push(stack, source);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < source->len; i++) {
        PtnValue cloned = ptn_null();
        if (!ptn_constant_clone_value(runtime, source->entries[i].value, line, stack, &cloned)) {
            ptn_value_destroy(&result);
            ptn_constant_array_stack_pop(stack);
            return 0;
        }
        ptn_array_set_entry_with_by_ref_argument_eligibility(
            result.as.array,
            ptn_array_key_clone(source->entries[i].key),
            cloned,
            source->entries[i].by_ref_argument_eligible
        );
    }
    result.as.array->next_auto_key = source->next_auto_key;
    result.as.array->current_index = source->current_index <= source->len
        ? source->current_index
        : source->len;
    ptn_constant_array_stack_pop(stack);
    *out = result;
    return 1;
}

static PTN_UNUSED int ptn_constant_clone_value(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnConstantArrayStack *stack,
    PtnValue *out
) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_ARRAY) {
        return ptn_constant_clone_array(runtime, resolved.as.array, line, stack, out);
    }
    *out = ptn_value_deep_clone(resolved);
    return 1;
}

static PTN_UNUSED int ptn_runtime_define_constant_if_absent_len(
    PtnRuntime *runtime,
    const char *name,
    size_t name_len,
    PtnValue value,
    size_t line
) {
    if (ptn_reserved_constant_name_len(name, name_len)) {
        ptn_emit_constant_already_defined_warning(&runtime->diagnostics, name, line);
        return 0;
    }
    if (ptn_runtime_constant_is_defined_len(runtime, name, name_len)) {
        ptn_emit_constant_already_defined_warning(&runtime->diagnostics, name, line);
        return 0;
    }
