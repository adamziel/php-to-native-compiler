    ptn_runtime_define_constant(runtime, name, value);
    return 1;
}

static PTN_UNUSED PtnValue ptn_read_constant(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_runtime_constant_value(runtime, name, &value)) {
        return value;
    }
    ptn_emit_undefined_constant_error(&runtime->diagnostics, name);
    exit(255);
    return ptn_null();
}

static PTN_UNUSED void ptn_echo(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            break;
        case PTN_BOOL:
            if (value.as.boolean) {
                fputs("1", stdout);
            }
            break;
        case PTN_INT:
            printf("%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            printf("%.14g", value.as.floating);
            break;
        case PTN_STRING:
            fwrite(value.as.string.data, 1, value.as.string.len, stdout);
            break;
        case PTN_ARRAY:
            fputs("Array", stdout);
            break;
        case PTN_EXCEPTION:
            fputs("Object", stdout);
            break;
        case PTN_REFERENCE:
            break;
    }
}

/* PTN_DIRECT_INTERNAL_HELPERS_START */
static PTN_UNUSED const char *ptn_count_operand_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_BOOL) {
        return value.as.boolean ? "true" : "false";
    }
    return ptn_offset_container_type_name(value);
}

static PTN_UNUSED PtnValue ptn_count_value(PtnRuntime *runtime, PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        return ptn_int((int64_t)value.as.array->len);
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "count(): Argument #1 ($value) must be of type Countable|array, %s given",
        ptn_count_operand_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_array_key_exists_value(PtnRuntime *runtime, PtnValue key_value, PtnValue array_value, size_t line) {
    key_value = ptn_value_deref(key_value);
    array_value = ptn_value_deref(array_value);
    if (array_value.type != PTN_ARRAY) {
        fputs("Fatal error: array_key_exists(): Argument #2 ($array) must be of type array\n", stderr);
        exit(255);
    }
    if (key_value.type == PTN_NULL) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Using null as the key parameter for array_key_exists() is deprecated, use an empty string instead",
            line
        );
    }
    PtnArrayKey key = ptn_array_key_from_value(key_value);
    int exists = ptn_array_entry_for_key(array_value.as.array, key) != NULL;
    ptn_array_key_free(key);
    return ptn_bool(exists);
}
/* PTN_DIRECT_INTERNAL_HELPERS_END */

/* PTN_INTERNAL_FUNCTIONS_START */
static void ptn_var_dump_indent(size_t indent) {
    for (size_t i = 0; i < indent; i++) {
        fputs("  ", stdout);
    }
}

static void ptn_var_dump_value_indented(PtnValue value, size_t indent) {
    int print_reference = value.type == PTN_REFERENCE && value.as.reference->refcount > 1;
    if (value.type == PTN_REFERENCE) {
        value = ptn_value_deref(value);
    }
    ptn_var_dump_indent(indent);
    if (print_reference) {
        fputs("&", stdout);
    }
    switch (value.type) {
        case PTN_NULL:
            fputs("NULL\n", stdout);
            break;
        case PTN_BOOL:
            fputs(value.as.boolean ? "bool(true)\n" : "bool(false)\n", stdout);
            break;
        case PTN_INT:
            printf("int(%lld)\n", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[64];
            ptn_format_var_dump_float(value.as.floating, formatted, sizeof(formatted));
            printf("float(%s)\n", formatted);
            break;
        }
        case PTN_STRING:
            printf("string(%zu) \"", value.as.string.len);
            fwrite(value.as.string.data, 1, value.as.string.len, stdout);
            fputs("\"\n", stdout);
            break;
        case PTN_ARRAY: {
            PtnArray *array = value.as.array;
            printf("array(%zu) {\n", array->len);
            for (size_t i = 0; i < array->len; i++) {
                ptn_var_dump_indent(indent + 1);
                PtnArrayKey key = array->entries[i].key;
                if (key.type == PTN_ARRAY_KEY_INT) {
                    printf("[%lld]=>\n", (long long)key.as.integer);
                } else {
                    printf("[\"%s\"]=>\n", key.as.string);
                }
                ptn_var_dump_value_indented(array->entries[i].value, indent + 1);
            }
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        }
        case PTN_EXCEPTION:
            printf("object(%s)#1 (1) {\n", value.as.exception->class_name);
            ptn_var_dump_indent(indent + 1);
            fputs("[\"message\"]=>\n", stdout);
            ptn_var_dump_indent(indent + 1);
            printf("string(%zu) \"", strlen(value.as.exception->message));
            fputs(value.as.exception->message, stdout);
            fputs("\"\n", stdout);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        case PTN_REFERENCE:
            fputs("NULL\n", stdout);
            break;
    }
}

static void ptn_var_dump_value(PtnValue value) {
    ptn_var_dump_value_indented(value, 0);
}

static PtnValue ptn_internal_var_dump(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    for (size_t i = 0; i < argc; i++) {
        ptn_var_dump_value(args[i]);
    }
    return ptn_null();
}

static PtnValue ptn_internal__ptn_cow_debug_reset(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    ptn_cow_debug_reset();
    return ptn_null();
}

static PtnValue ptn_internal__ptn_cow_debug_counter(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    size_t counter = 0;
    int found = ptn_cow_debug_counter(name, &counter);
    free(name);
    if (!found) {
        ptn_cow_debug_abort("unknown counter");
    }
    if (counter > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_int((int64_t)counter);
}

static PtnValue ptn_internal__ptn_cow_debug_assert_counter(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    PtnValue expected = ptn_cast_int(args[1]);
    ptn_cow_debug_assert_named_counter(name, expected.as.integer);
    free(name);
    return ptn_bool(1);
}

static PtnValue ptn_internal__ptn_cow_debug_assert_balanced(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    ptn_cow_debug_assert_balanced();
    return ptn_bool(1);
}

static void ptn_print_r_value_indented(PtnStringBuffer *buffer, PtnValue value, size_t indent);

static void ptn_print_r_key(PtnStringBuffer *buffer, PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        ptn_string_buffer_append_format(buffer, "%lld", (long long)key.as.integer);
    } else {
        ptn_string_buffer_append(buffer, key.as.string);
    }
}

static void ptn_print_r_array(PtnStringBuffer *buffer, PtnArray *array, size_t indent) {
    ptn_string_buffer_append(buffer, "Array\n");
    ptn_string_buffer_append_indent(buffer, indent);
    ptn_string_buffer_append(buffer, "(\n");
    for (size_t i = 0; i < array->len; i++) {
        ptn_string_buffer_append_indent(buffer, indent + 4);
        ptn_string_buffer_append_char(buffer, '[');
        ptn_print_r_key(buffer, array->entries[i].key);
        ptn_string_buffer_append(buffer, "] => ");
        PtnValue entry_value = ptn_value_deref(array->entries[i].value);
        if (entry_value.type == PTN_ARRAY) {
            ptn_print_r_value_indented(buffer, entry_value, indent + 8);
            ptn_string_buffer_append_char(buffer, '\n');
        } else {
            ptn_print_r_value_indented(buffer, entry_value, indent);
            ptn_string_buffer_append_char(buffer, '\n');
        }
    }
    ptn_string_buffer_append_indent(buffer, indent);
    ptn_string_buffer_append(buffer, ")\n");
}

static void ptn_print_r_value_indented(PtnStringBuffer *buffer, PtnValue value, size_t indent) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            break;
        case PTN_BOOL:
            if (value.as.boolean) {
                ptn_string_buffer_append_char(buffer, '1');
            }
            break;
        case PTN_INT:
            ptn_string_buffer_append_format(buffer, "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT:
            ptn_string_buffer_append_format(buffer, "%.14g", value.as.floating);
            break;
        case PTN_STRING:
            ptn_string_buffer_append_len(
                buffer,
                (const char *)value.as.string.data,
                value.as.string.len
            );
            break;
        case PTN_ARRAY:
            ptn_print_r_array(buffer, value.as.array, indent);
            break;
        case PTN_EXCEPTION:
            ptn_string_buffer_append(buffer, "Object");
            break;
        case PTN_REFERENCE:
            break;
    }
}

static PtnValue ptn_internal_print_r(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    int return_output = argc >= 2 && ptn_is_truthy(args[1]);
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    ptn_print_r_value_indented(&buffer, args[0], 0);
    if (return_output) {
        return ptn_owned_string_len(buffer.data, buffer.len);
    }
    fwrite(buffer.data, 1, buffer.len, stdout);
    free(buffer.data);
    return ptn_bool(1);
}

static PtnArray *ptn_internal_expect_array_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        return value.as.array;
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be of type array, %s given",
        function_name,
        position,
        argument_name,
        ptn_offset_container_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_type_error(&runtime->diagnostics, message);
    exit(255);
    return NULL;
}

static PtnArray *ptn_internal_expect_mutable_array_variable_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    const char *variable_name,
    PtnValue value
) {
    PtnArray *array = ptn_internal_expect_array_arg(
        runtime,
        function_name,
        position,
        argument_name,
        value
    );
    size_t index = ptn_symbols_find(&runtime->symbols, variable_name);
    if (index >= runtime->symbols.len || runtime->symbols.items[index].value.type != PTN_ARRAY) {
        return array;
    }
    return ptn_array_detach_value(&runtime->symbols.items[index].value);
}

static PTN_UNUSED PtnValue ptn_runtime_array_pop_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_pop",
        1,
        "array",
        name,
        value
    );
    return ptn_array_pop_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_push_variable(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t value_count,
    const PtnValue *values
) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_push",
        1,
        "array",
        name,
        value
    );
    return ptn_int(ptn_array_push_values(array, value_count, values));
}

static PTN_UNUSED PtnValue ptn_runtime_array_shift_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_shift",
        1,
        "array",
        name,
        value
    );
    return ptn_array_shift_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_unshift_variable(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t value_count,
    const PtnValue *values
) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_unshift",
        1,
        "array",
        name,
        value
    );
    return ptn_int(ptn_array_unshift_values(array, value_count, values));
}

static PTN_UNUSED PtnValue ptn_runtime_array_next_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "next",
        1,
        "array",
        name,
        value
    );
    return ptn_array_next_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_end_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "end",
        1,
        "array",
        name,
        value
    );
    return ptn_array_end_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_prev_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "prev",
        1,
        "array",
        name,
        value
    );
    return ptn_array_prev_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_reset_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "reset",
        1,
        "array",
        name,
        value
    );
    return ptn_array_reset_value(array);
}

static PtnValue ptn_internal_array_pop(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_pop", 1, "array", args[0]);
    return ptn_array_pop_value(array);
}

static PtnValue ptn_internal_array_push(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_push", 1, "array", args[0]);
    return ptn_int(ptn_array_push_values(array, argc - 1, args + 1));
}

static PtnValue ptn_internal_array_shift(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_shift", 1, "array", args[0]);
    return ptn_array_shift_value(array);
}

static PtnValue ptn_internal_array_unshift(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_unshift", 1, "array", args[0]);
    return ptn_int(ptn_array_unshift_values(array, argc - 1, args + 1));
}

static PtnValue ptn_internal_array_values(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_values", 1, "array", args[0]);
    if (array->len == 0) {
        return ptn_array_from_literal_entries(0, NULL);
    }

    PtnArrayLiteralEntry *entries = malloc(array->len * sizeof(PtnArrayLiteralEntry));
    if (entries == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < array->len; i++) {
        entries[i].has_key = 0;
        entries[i].value = ptn_array_reindexing_internal_value(array->entries[i].value);
    }

    PtnValue result = ptn_array_from_literal_entries(array->len, entries);
    free(entries);
    return result;
}

static PtnValue ptn_internal_array_reverse(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_reverse", 1, "array", args[0]);
    int preserve_keys = argc >= 2 && ptn_is_truthy(args[1]);
    if (array->len == 0) {
        return ptn_array_from_literal_entries(0, NULL);
    }

    PtnArrayLiteralEntry *entries = malloc(array->len * sizeof(PtnArrayLiteralEntry));
    if (entries == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *source = &array->entries[array->len - i - 1];
        entries[i].has_key = preserve_keys || source->key.type == PTN_ARRAY_KEY_STRING;
        entries[i].key = entries[i].has_key ? ptn_array_key_value(source->key) : ptn_null();
        entries[i].value = ptn_array_reindexing_internal_value(source->value);
    }

    PtnValue result = ptn_array_from_literal_entries(array->len, entries);
    for (size_t i = 0; i < array->len; i++) {
        if (entries[i].has_key) {
            ptn_value_destroy(&entries[i].key);
        }
    }
    free(entries);
    return result;
}

static PtnValue ptn_internal_current(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "current", 1, "array", args[0]);
    return ptn_array_current_value(array);
}

static PtnValue ptn_internal_key(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "key", 1, "array", args[0]);
    return ptn_array_current_key_value(array);
}

static PtnValue ptn_internal_next(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "next", 1, "array", args[0]);
    return ptn_array_next_value(array);
}

static PtnValue ptn_internal_end(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "end", 1, "array", args[0]);
    return ptn_array_end_value(array);
}

static PtnValue ptn_internal_prev(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "prev", 1, "array", args[0]);
    return ptn_array_prev_value(array);
}

static PtnValue ptn_internal_reset(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "reset", 1, "array", args[0]);
    return ptn_array_reset_value(array);
}

static PtnValue ptn_internal_strlen(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_int((int64_t)len);
}

static char *ptn_rot13_string(const char *string, size_t len) {
    char *rotated = malloc(len + 1);
    if (rotated == NULL) {
        ptn_abort_out_of_memory();
    }

    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)string[i];
        if (byte >= 'a' && byte <= 'z') {
            rotated[i] = (char)('a' + ((byte - 'a' + 13) % 26));
        } else if (byte >= 'A' && byte <= 'Z') {
            rotated[i] = (char)('A' + ((byte - 'A' + 13) % 26));
        } else {
            rotated[i] = (char)byte;
        }
    }
    rotated[len] = '\0';
    return rotated;
}

static PtnValue ptn_internal_str_rot13(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    char *rotated = ptn_rot13_string(string.data, string.len);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(rotated, len);
}

static PtnValue ptn_internal_strcmp(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand left = ptn_value_to_string_operand(args[0]);
    PtnStringOperand right = ptn_value_to_string_operand(args[1]);
    int compared = ptn_compare_string_bytes(
        (const unsigned char *)left.data,
        left.len,
        (const unsigned char *)right.data,
        right.len
    );
    ptn_string_operand_free(left);
    ptn_string_operand_free(right);
    if (compared < 0) {
        return ptn_int(-1);
    }
    if (compared > 0) {
        return ptn_int(1);
    }
    return ptn_int(0);
}

static PtnValue ptn_internal_str_contains(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand haystack = ptn_value_to_string_operand(args[0]);
    PtnStringOperand needle = ptn_value_to_string_operand(args[1]);
    int contains = needle.len == 0;
    if (!contains && needle.len <= haystack.len) {
        for (size_t offset = 0; offset <= haystack.len - needle.len; offset++) {
            if (memcmp(haystack.data + offset, needle.data, needle.len) == 0) {
                contains = 1;
                break;
            }
        }
    }
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return ptn_bool(contains);
}

static PtnValue ptn_internal_str_starts_with(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand haystack = ptn_value_to_string_operand(args[0]);
    PtnStringOperand needle = ptn_value_to_string_operand(args[1]);
    size_t haystack_len = haystack.len;
    size_t needle_len = needle.len;
    int starts = needle_len <= haystack_len && memcmp(haystack.data, needle.data, needle_len) == 0;
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return ptn_bool(starts);
}

static PtnValue ptn_internal_str_ends_with(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand haystack = ptn_value_to_string_operand(args[0]);
    PtnStringOperand needle = ptn_value_to_string_operand(args[1]);
    size_t haystack_len = haystack.len;
    size_t needle_len = needle.len;
    int ends =
        needle_len <= haystack_len &&
        memcmp(haystack.data + haystack_len - needle_len, needle.data, needle_len) == 0;
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return ptn_bool(ends);
}

static int ptn_quotemeta_needs_escape(unsigned char byte) {
    switch (byte) {
        case '.':
        case '\\':
        case '+':
        case '*':
        case '?':
        case '[':
        case '^':
        case ']':
        case '(':
        case '$':
        case ')':
            return 1;
        default:
            return 0;
    }
}

static char *ptn_quotemeta_string(const char *input, size_t len, size_t *output_len_out) {
    size_t escape_count = 0;
    for (size_t i = 0; i < len; i++) {
        if (ptn_quotemeta_needs_escape((unsigned char)input[i])) {
            escape_count++;
        }
    }
    if (escape_count > SIZE_MAX - len - 1) {
        ptn_abort_out_of_memory();
    }

    char *output = malloc(len + escape_count + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)input[i];
        if (ptn_quotemeta_needs_escape(byte)) {
            output[out++] = '\\';
        }
        output[out++] = (char)byte;
    }
    output[out] = '\0';
    *output_len_out = out;
    return output;
}

static PtnValue ptn_internal_quotemeta(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand input = ptn_value_to_string_operand(args[0]);
    size_t output_len = 0;
    char *output = ptn_quotemeta_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static char *ptn_chunk_split_string(
    const char *input,
    size_t input_len,
    size_t chunk_len,
    const char *ending,
    size_t ending_len,
    size_t *output_len_out
) {
    size_t chunk_count = input_len == 0 ? 0 : ((input_len - 1) / chunk_len) + 1;
    if (chunk_count != 0 && ending_len > (SIZE_MAX - input_len) / chunk_count) {
        ptn_abort_out_of_memory();
    }
    size_t output_len = input_len + (chunk_count * ending_len);
    if (output_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }

    char *output = malloc(output_len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t input_offset = 0;
    size_t output_offset = 0;
    while (input_offset < input_len) {
        size_t remaining = input_len - input_offset;
        size_t copy_len = remaining < chunk_len ? remaining : chunk_len;
        memcpy(output + output_offset, input + input_offset, copy_len);
        input_offset += copy_len;
        output_offset += copy_len;
        memcpy(output + output_offset, ending, ending_len);
        output_offset += ending_len;
    }
    output[output_offset] = '\0';
    *output_len_out = output_offset;
    return output;
}

static PtnValue ptn_internal_chunk_split(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    PtnStringOperand input = ptn_value_to_string_operand(args[0]);
    int64_t chunk_len_value = argc >= 2 ? ptn_value_to_integer(args[1]) : 76;
    if (chunk_len_value <= 0) {
        ptn_string_operand_free(input);
        ptn_abort_arithmetic_error("chunk_split(): Argument #2 ($length) must be greater than 0");
    }
    PtnStringOperand ending;
    if (argc >= 3) {
        ending = ptn_value_to_string_operand(args[2]);
    } else {
        ending.data = "\r\n";
        ending.owned = NULL;
        ending.len = 2;
    }
    size_t output_len = 0;
    char *output = ptn_chunk_split_string(
        input.data,
        input.len,
        (size_t)chunk_len_value,
        ending.data,
        ending.len,
        &output_len
    );
    ptn_string_operand_free(input);
    ptn_string_operand_free(ending);
    return ptn_owned_string_len(output, output_len);
}

static char *ptn_strip_tags_string(const char *input, size_t len, size_t *output_len_out) {
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t input_offset = 0;
    size_t output_offset = 0;
    while (input_offset < len) {
        if (input[input_offset] == '<') {
            if (input_offset + 1 < len && input[input_offset + 1] == '?') {
                size_t tag_end = input_offset + 2;
                while (tag_end + 1 < len && !(input[tag_end] == '?' && input[tag_end + 1] == '>')) {
                    tag_end++;
                }
                if (tag_end + 1 < len) {
                    input_offset = tag_end + 2;
                    continue;
                }
            } else if (input_offset + 1 < len && input[input_offset + 1] == '%') {
                size_t tag_end = input_offset + 2;
                while (tag_end + 1 < len && !(input[tag_end] == '%' && input[tag_end + 1] == '>')) {
                    tag_end++;
                }
                if (tag_end + 1 < len) {
                    input_offset = tag_end + 2;
                    continue;
                }
            } else if (input_offset + 3 < len && memcmp(input + input_offset, "<!--", 4) == 0) {
                size_t tag_end = input_offset + 4;
                while (tag_end + 2 < len && memcmp(input + tag_end, "-->", 3) != 0) {
                    tag_end++;
                }
                if (tag_end + 2 < len) {
                    input_offset = tag_end + 3;
                    continue;
                }
            } else {
                size_t tag_end = input_offset + 1;
                while (tag_end < len && input[tag_end] != '>') {
                    tag_end++;
                }
                if (tag_end < len) {
                    input_offset = tag_end + 1;
                    continue;
                }
            }
        }
        if (input[input_offset] == '\0') {
            input_offset++;
            continue;
        }
        output[output_offset++] = input[input_offset++];
    }
    output[output_offset] = '\0';
    *output_len_out = output_offset;
    return output;
}

static PtnValue ptn_internal_strip_tags(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand input = ptn_value_to_string_operand(args[0]);
    size_t output_len = 0;
    char *output = ptn_strip_tags_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static uint32_t ptn_rotate_left32(uint32_t value, uint32_t amount) {
    return (value << amount) | (value >> (32 - amount));
}

static char *ptn_digest_hex_string(const unsigned char *digest, size_t digest_len) {
    static const char hex_digits[] = "0123456789abcdef";
    if (digest_len > (SIZE_MAX - 1) / 2) {
        ptn_abort_out_of_memory();
    }
    char *hex = malloc((digest_len * 2) + 1);
    if (hex == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < digest_len; i++) {
        hex[i * 2] = hex_digits[digest[i] >> 4];
        hex[i * 2 + 1] = hex_digits[digest[i] & 0x0f];
    }
    hex[digest_len * 2] = '\0';
    return hex;
}

static char *ptn_digest_raw_string(const unsigned char *digest, size_t digest_len) {
    char *raw = malloc(digest_len + 1);
    if (raw == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(raw, digest, digest_len);
    raw[digest_len] = '\0';
    return raw;
}

static PtnValue ptn_digest_value(const unsigned char *digest, size_t digest_len, int raw_output) {
    if (raw_output) {
        return ptn_owned_string_len(ptn_digest_raw_string(digest, digest_len), digest_len);
    }
    return ptn_owned_string_len(ptn_digest_hex_string(digest, digest_len), digest_len * 2);
}

static void ptn_md5_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[16]) {
    static const uint32_t shifts[64] = {
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21
    };
    static const uint32_t constants[64] = {
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391
    };

    size_t padded_len = input_len + 1;
    while ((padded_len % 64) != 56) {
        padded_len++;
    }
    if (padded_len < input_len || padded_len > SIZE_MAX - 8) {
        ptn_abort_out_of_memory();
    }

    unsigned char *message = calloc(padded_len + 8, 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (input_len != 0) {
        memcpy(message, input, input_len);
    }
    message[input_len] = 0x80;

    uint64_t bit_len = (uint64_t)input_len * 8;
    for (size_t i = 0; i < 8; i++) {
        message[padded_len + i] = (unsigned char)(bit_len >> (8 * i));
    }

    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xefcdab89;
    uint32_t h2 = 0x98badcfe;
    uint32_t h3 = 0x10325476;

    for (size_t offset = 0; offset < padded_len; offset += 64) {
        uint32_t words[16];
        for (size_t i = 0; i < 16; i++) {
            size_t base = offset + i * 4;
            words[i] = (uint32_t)message[base]
                | ((uint32_t)message[base + 1] << 8)
                | ((uint32_t)message[base + 2] << 16)
                | ((uint32_t)message[base + 3] << 24);
        }

        uint32_t a = h0;
        uint32_t b = h1;
        uint32_t c = h2;
        uint32_t d = h3;

        for (uint32_t i = 0; i < 64; i++) {
            uint32_t f;
            uint32_t g;
            if (i < 16) {
                f = (b & c) | ((~b) & d);
                g = i;
            } else if (i < 32) {
                f = (d & b) | ((~d) & c);
                g = (5 * i + 1) % 16;
            } else if (i < 48) {
                f = b ^ c ^ d;
                g = (3 * i + 5) % 16;
            } else {
                f = c ^ (b | (~d));
                g = (7 * i) % 16;
            }

            uint32_t next = d;
            d = c;
            c = b;
            b = b + ptn_rotate_left32(a + f + constants[i] + words[g], shifts[i]);
            a = next;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
    }

    free(message);

    uint32_t words[4] = { h0, h1, h2, h3 };
    for (size_t i = 0; i < 4; i++) {
        digest[i * 4] = (unsigned char)words[i];
        digest[i * 4 + 1] = (unsigned char)(words[i] >> 8);
        digest[i * 4 + 2] = (unsigned char)(words[i] >> 16);
        digest[i * 4 + 3] = (unsigned char)(words[i] >> 24);
    }
}

static PtnValue ptn_internal_md5(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    PtnStringOperand input = ptn_value_to_string_operand(args[0]);
    unsigned char digest[16];
    ptn_md5_digest_bytes((const unsigned char *)input.data, input.len, digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    ptn_string_operand_free(input);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static void ptn_sha1_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[20]) {
    size_t padded_len = input_len + 1;
    while ((padded_len % 64) != 56) {
        padded_len++;
    }
    if (padded_len < input_len || padded_len > SIZE_MAX - 8) {
        ptn_abort_out_of_memory();
    }

    unsigned char *message = calloc(padded_len + 8, 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (input_len != 0) {
        memcpy(message, input, input_len);
    }
    message[input_len] = 0x80;

    uint64_t bit_len = (uint64_t)input_len * 8;
    for (size_t i = 0; i < 8; i++) {
        message[padded_len + 7 - i] = (unsigned char)(bit_len >> (8 * i));
    }

    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xefcdab89;
    uint32_t h2 = 0x98badcfe;
    uint32_t h3 = 0x10325476;
    uint32_t h4 = 0xc3d2e1f0;

    for (size_t offset = 0; offset < padded_len; offset += 64) {
        uint32_t words[80];
        for (size_t i = 0; i < 16; i++) {
            size_t base = offset + i * 4;
            words[i] = ((uint32_t)message[base] << 24)
                | ((uint32_t)message[base + 1] << 16)
                | ((uint32_t)message[base + 2] << 8)
                | (uint32_t)message[base + 3];
        }
        for (size_t i = 16; i < 80; i++) {
            words[i] = ptn_rotate_left32(words[i - 3] ^ words[i - 8] ^ words[i - 14] ^ words[i - 16], 1);
        }

        uint32_t a = h0;
        uint32_t b = h1;
        uint32_t c = h2;
        uint32_t d = h3;
        uint32_t e = h4;

        for (size_t i = 0; i < 80; i++) {
            uint32_t f;
            uint32_t k;
            if (i < 20) {
                f = (b & c) | ((~b) & d);
                k = 0x5a827999;
            } else if (i < 40) {
                f = b ^ c ^ d;
                k = 0x6ed9eba1;
            } else if (i < 60) {
                f = (b & c) | (b & d) | (c & d);
                k = 0x8f1bbcdc;
            } else {
                f = b ^ c ^ d;
                k = 0xca62c1d6;
            }

            uint32_t temp = ptn_rotate_left32(a, 5) + f + e + k + words[i];
            e = d;
            d = c;
            c = ptn_rotate_left32(b, 30);
            b = a;
            a = temp;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
        h4 += e;
    }

    free(message);

    uint32_t words[5] = { h0, h1, h2, h3, h4 };
    for (size_t i = 0; i < 5; i++) {
        digest[i * 4] = (unsigned char)(words[i] >> 24);
        digest[i * 4 + 1] = (unsigned char)(words[i] >> 16);
        digest[i * 4 + 2] = (unsigned char)(words[i] >> 8);
        digest[i * 4 + 3] = (unsigned char)words[i];
    }
}

static PtnValue ptn_internal_sha1(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    PtnStringOperand input = ptn_value_to_string_operand(args[0]);
    unsigned char digest[20];
    ptn_sha1_digest_bytes((const unsigned char *)input.data, input.len, digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    ptn_string_operand_free(input);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static char *ptn_path_operand_to_c_string(PtnStringOperand path) {
    if (memchr(path.data, '\0', path.len) != NULL) {
        return NULL;
    }
    return ptn_duplicate_string_len(path.data, path.len);
}

static void ptn_emit_file_warning(
    PtnRuntime *runtime,
    const char *function_name,
    const char *path,
    const char *detail,
    size_t line
) {
    int needed = snprintf(NULL, 0, "%s(%s): %s", function_name, path, detail);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(message, (size_t)needed + 1, "%s(%s): %s", function_name, path, detail);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    if (runtime->diagnostics.suppressed <= 0) {
        fputc('\n', stdout);
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
    free(message);
}

static PtnValue ptn_internal_file_put_contents(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "file_put_contents(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    PtnStringOperand data = ptn_value_to_string_operand(args[1]);
    FILE *stream = fopen(path, "wb");
    if (stream == NULL) {
        char detail[192];
        int needed = snprintf(detail, sizeof(detail), "Failed to open stream: %s", strerror(errno));
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "file_put_contents", path, detail, line);
        free(path);
        ptn_string_operand_free(data);
        return ptn_bool(0);
    }

    size_t written = fwrite(data.data, 1, data.len, stream);
    int failed = written != data.len || fclose(stream) != 0;
    if (failed) {
        char detail[192];
        int needed = snprintf(detail, sizeof(detail), "Failed to write %zu bytes: %s", data.len, strerror(errno));
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "file_put_contents", path, detail, line);
        free(path);
        ptn_string_operand_free(data);
        return ptn_bool(0);
    }

    if (data.len > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    PtnValue result = ptn_int((int64_t)data.len);
    free(path);
    ptn_string_operand_free(data);
    return result;
}

static int ptn_read_file_bytes(const char *path, unsigned char **data_out, size_t *len_out) {
    FILE *stream = fopen(path, "rb");
    if (stream == NULL) {
        return 0;
    }

    unsigned char *data = NULL;
    size_t len = 0;
    size_t capacity = 0;
    unsigned char chunk[4096];
    for (;;) {
        size_t read_len = fread(chunk, 1, sizeof(chunk), stream);
        if (read_len != 0) {
            if (read_len > SIZE_MAX - len) {
                fclose(stream);
                free(data);
                ptn_abort_out_of_memory();
            }
            size_t required = len + read_len;
            if (required > capacity) {
                size_t new_capacity = capacity == 0 ? 4096 : capacity;
                while (new_capacity < required) {
                    if (new_capacity > SIZE_MAX / 2) {
                        fclose(stream);
                        free(data);
                        ptn_abort_out_of_memory();
                    }
                    new_capacity *= 2;
                }
                unsigned char *new_data = realloc(data, new_capacity);
                if (new_data == NULL) {
                    fclose(stream);
                    free(data);
                    ptn_abort_out_of_memory();
                }
                data = new_data;
                capacity = new_capacity;
            }
            memcpy(data + len, chunk, read_len);
            len = required;
        }
        if (read_len < sizeof(chunk)) {
            if (ferror(stream)) {
                fclose(stream);
                free(data);
                return -1;
            }
            break;
        }
    }

    if (fclose(stream) != 0) {
        free(data);
        return -1;
    }
    *data_out = data;
    *len_out = len;
    return 1;
}

static PtnValue ptn_internal_sha1_file(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "sha1_file(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    unsigned char *data = NULL;
    size_t data_len = 0;
    int read_result = ptn_read_file_bytes(path, &data, &data_len);
    if (read_result <= 0) {
        char detail[192];
        int needed = snprintf(
            detail,
            sizeof(detail),
            "%s: %s",
            read_result == 0 ? "Failed to open stream" : "Failed to read stream",
            strerror(errno)
        );
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "sha1_file", path, detail, line);
        free(path);
        free(data);
        return ptn_bool(0);
    }

    unsigned char digest[20];
    ptn_sha1_digest_bytes(data, data_len, digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    free(path);
    free(data);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static PtnValue ptn_internal_unlink(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "unlink(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    if (remove(path) == 0) {
        free(path);
        return ptn_bool(1);
    }

    ptn_emit_file_warning(runtime, "unlink", path, strerror(errno), line);
    free(path);
    return ptn_bool(0);
}

static size_t ptn_substr_clamped_positive(int64_t value, size_t limit) {
    if (value <= 0) {
        return 0;
    }
    uint64_t unsigned_value = (uint64_t)value;
    if (unsigned_value > (uint64_t)limit) {
        return limit;
    }
    return (size_t)unsigned_value;
}

static size_t ptn_substr_clamped_negative_distance(int64_t value, size_t limit) {
    if (value >= 0) {
        return 0;
    }
    if (value == INT64_MIN) {
        return limit;
    }
    uint64_t distance = (uint64_t)(-value);
    if (distance > (uint64_t)limit) {
        return limit;
    }
    return (size_t)distance;
}

static size_t ptn_substr_start_offset(size_t string_len, int64_t start) {
    if (start >= 0) {
        return ptn_substr_clamped_positive(start, string_len);
    }
    size_t distance = ptn_substr_clamped_negative_distance(start, string_len);
    return string_len - distance;
}

static char *ptn_substr_copy(const char *string, size_t start, size_t len) {
    if (len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    char *result = malloc(len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(result, string + start, len);
    result[len] = '\0';
    return result;
}

static PtnValue ptn_internal_substr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    size_t string_len = string.len;
    size_t start = ptn_substr_start_offset(string_len, ptn_value_to_integer(args[1]));
    size_t end = string_len;

    if (argc >= 3 && args[2].type != PTN_NULL) {
        int64_t length = ptn_value_to_integer(args[2]);
        if (length >= 0) {
            size_t requested_len = ptn_substr_clamped_positive(length, string_len);
            size_t available_len = string_len - start;
            if (requested_len > available_len) {
                requested_len = available_len;
            }
            end = start + requested_len;
        } else {
            size_t truncate_len = ptn_substr_clamped_negative_distance(length, string_len);
            end = string_len - truncate_len;
            if (end < start) {
                end = start;
            }
        }
    }

    char *substring = ptn_substr_copy(string.data, start, end - start);
    size_t substring_len = end - start;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(substring, substring_len);
}

static int ptn_is_path_separator(char byte) {
    return byte == '/' || byte == '\\';
}

static char *ptn_dirname_string(const char *path, size_t len) {
    if (len == 0) {
        return ptn_duplicate_string(".");
    }
    while (len > 1 && ptn_is_path_separator(path[len - 1])) {
        len--;
    }

    size_t end = len;
    while (end > 0 && !ptn_is_path_separator(path[end - 1])) {
        end--;
    }
    if (end == 0) {
        return ptn_duplicate_string(".");
    }
    while (end > 1 && ptn_is_path_separator(path[end - 1])) {
        end--;
    }

    char *dirname = malloc(end + 1);
    if (dirname == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(dirname, path, end);
    dirname[end] = '\0';
    return dirname;
}

static PtnValue ptn_internal_dirname(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand path = ptn_value_to_string_operand(args[0]);
    char *dirname = ptn_dirname_string(path.data, path.len);
    ptn_string_operand_free(path);
    return ptn_owned_string(dirname);
}

static PtnValue ptn_internal_gettype(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_gettype_value(args[0]);
}

static PtnValue ptn_internal_is_null(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_NULL);
}

static PtnValue ptn_internal_is_array(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_ARRAY);
}

static PtnValue ptn_internal_is_bool(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_BOOL);
}

static PtnValue ptn_internal_is_int(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_INT);
}

static PtnValue ptn_internal_is_float(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_FLOAT);
}

static PtnValue ptn_internal_is_string(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_STRING);
}

static PtnValue ptn_internal_is_scalar(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_scalar(args[0]);
}

static PtnValue ptn_internal_is_finite(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    if (args[0].type != PTN_FLOAT) {
        return ptn_bool(1);
    }
    return ptn_bool(isfinite(args[0].as.floating));
}

static PtnValue ptn_internal_is_infinite(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_bool(args[0].type == PTN_FLOAT && isinf(args[0].as.floating));
}

static PtnValue ptn_internal_is_nan(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_bool(args[0].type == PTN_FLOAT && isnan(args[0].as.floating));
}

static PtnValue ptn_internal_bin2hex(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    static const char hex_digits[] = "0123456789abcdef";
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    size_t len = string.len;
    if (len > (SIZE_MAX - 1) / 2) {
        ptn_abort_out_of_memory();
    }
    char *hex = malloc((len * 2) + 1);
    if (hex == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)string.data[i];
        hex[i * 2] = hex_digits[byte >> 4];
        hex[(i * 2) + 1] = hex_digits[byte & 0x0f];
    }
    hex[len * 2] = '\0';
    ptn_string_operand_free(string);
    return ptn_owned_string_len(hex, len * 2);
}

static int ptn_hex_nibble(unsigned char byte) {
    if (byte >= '0' && byte <= '9') {
        return (int)(byte - '0');
    }
    if (byte >= 'a' && byte <= 'f') {
        return 10 + (int)(byte - 'a');
    }
    if (byte >= 'A' && byte <= 'F') {
        return 10 + (int)(byte - 'A');
    }
    return -1;
}

static PtnValue ptn_internal_hex2bin(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand hex = ptn_value_to_string_operand(args[0]);
    size_t len = hex.len;
    if ((len % 2) != 0) {
        ptn_emit_warning(
            &runtime->diagnostics,
            "hex2bin(): Hexadecimal input string must have an even length",
            line
        );
        ptn_string_operand_free(hex);
        return ptn_bool(0);
    }

    char *binary = malloc((len / 2) + 1);
    if (binary == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t output_len = 0;
    for (size_t i = 0; i < len; i += 2) {
        int high = ptn_hex_nibble((unsigned char)hex.data[i]);
        int low = ptn_hex_nibble((unsigned char)hex.data[i + 1]);
        if (high < 0 || low < 0) {
            ptn_emit_warning(
                &runtime->diagnostics,
                "hex2bin(): Input string must be hexadecimal string",
                line
            );
            free(binary);
            ptn_string_operand_free(hex);
            return ptn_bool(0);
        }
        binary[output_len++] = (char)((high << 4) | low);
    }
    binary[output_len] = '\0';
    ptn_string_operand_free(hex);
    return ptn_owned_string_len(binary, output_len);
}

static char *ptn_quoted_printable_decode_string(const char *input, size_t len, size_t *output_len_out) {
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        if (input[i] == '=') {
            if (i + 1 < len && input[i + 1] == '\n') {
                i += 1;
                continue;
            }
            if (i + 2 < len && input[i + 1] == '\r' && input[i + 2] == '\n') {
                i += 2;
                continue;
            }
            if (i + 2 < len) {
                int high = ptn_hex_nibble((unsigned char)input[i + 1]);
                int low = ptn_hex_nibble((unsigned char)input[i + 2]);
                if (high >= 0 && low >= 0) {
                    output[out++] = (char)((high << 4) | low);
                    i += 2;
                    continue;
                }
            }
        }
        output[out++] = input[i];
    }
    output[out] = '\0';
    *output_len_out = out;
    return output;
}

static PtnValue ptn_internal_quoted_printable_decode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand input = ptn_value_to_string_operand(args[0]);
    size_t output_len = 0;
    char *output = ptn_quoted_printable_decode_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static int ptn_ascii_is_letter(unsigned char byte) {
    return (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z');
}

static unsigned char ptn_ascii_upper(unsigned char byte) {
    if (byte >= 'a' && byte <= 'z') {
        return (unsigned char)(byte - ('a' - 'A'));
    }
    return byte;
}

static char ptn_soundex_code(unsigned char byte) {
    switch (ptn_ascii_upper(byte)) {
        case 'B':
        case 'F':
        case 'P':
        case 'V':
            return '1';
        case 'C':
        case 'G':
        case 'J':
        case 'K':
        case 'Q':
        case 'S':
        case 'X':
        case 'Z':
            return '2';
        case 'D':
        case 'T':
            return '3';
        case 'L':
            return '4';
        case 'M':
        case 'N':
            return '5';
        case 'R':
            return '6';
        default:
            return '\0';
    }
}

static int ptn_soundex_resets_previous(unsigned char byte) {
    switch (ptn_ascii_upper(byte)) {
        case 'A':
        case 'E':
        case 'I':
        case 'O':
        case 'U':
        case 'Y':
        case 'H':
        case 'W':
            return 1;
        default:
            return 0;
    }
}

static PtnValue ptn_internal_soundex(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    char *result = malloc(5);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = '0';
    result[1] = '0';
    result[2] = '0';
    result[3] = '0';
    result[4] = '\0';

    size_t first = 0;
    while (first < string.len && !ptn_ascii_is_letter((unsigned char)string.data[first])) {
        first++;
    }
    if (first == string.len) {
        ptn_string_operand_free(string);
        return ptn_owned_string_len(result, 4);
    }

    result[0] = (char)ptn_ascii_upper((unsigned char)string.data[first]);
    char previous = ptn_soundex_code((unsigned char)string.data[first]);
    size_t output_len = 1;
    for (size_t i = first + 1; i < string.len && output_len < 4; i++) {
        unsigned char byte = (unsigned char)string.data[i];
        char code = ptn_soundex_code(byte);
        if (code == '\0') {
            if (ptn_soundex_resets_previous(byte)) {
                previous = '\0';
            }
            continue;
        }
        if (code != previous) {
            result[output_len++] = code;
        }
        previous = code;
    }

    ptn_string_operand_free(string);
    return ptn_owned_string_len(result, 4);
}

static double ptn_value_to_double(PtnValue value) {
    double fast_number = 0.0;
    if (ptn_fast_scalar_double(value, &fast_number)) {
        return fast_number;
    }

    PtnNumber number = ptn_to_number(value);
    return number.floating;
}

static PtnValue ptn_internal_ceil(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_float(ceil(ptn_value_to_double(args[0])));
}

static PtnValue ptn_internal_floor(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_float(floor(ptn_value_to_double(args[0])));
}

static PtnValue ptn_internal_abs(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    if (args[0].type == PTN_NULL) {
        ptn_emit_array_runtime_diagnostic(
            "Deprecated",
            "abs(): Passing null to parameter #1 ($num) of type int|float is deprecated",
            line
        );
    }

    PtnNumber number = ptn_to_number(args[0]);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(fabs(number.floating));
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(fabs((double)number.integer));
    }
    if (number.integer < 0) {
        return ptn_int(-number.integer);
    }
    return ptn_int(number.integer);
}

static PtnValue ptn_internal_sqrt(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_float(sqrt(ptn_value_to_double(args[0])));
}

static PtnValue ptn_internal_fdiv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    double dividend = ptn_value_to_double(args[0]);
    double divisor = ptn_value_to_double(args[1]);
    return ptn_float(dividend / divisor);
}

static PtnValue ptn_internal_intdiv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    int64_t dividend = ptn_value_to_integer_with_precision_deprecation(args[0]);
    int64_t divisor = ptn_value_to_integer_with_precision_deprecation(args[1]);
    if (divisor == 0) {
        ptn_abort_arithmetic_error("Division by zero");
    }
    if (dividend == INT64_MIN && divisor == -1) {
        ptn_abort_arithmetic_error("Division of PHP_INT_MIN by -1 is not an integer");
    }
    return ptn_int(dividend / divisor);
}

static PtnValue ptn_internal_pi(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_float(3.14159265358979323846264338327950288);
}

static PtnValue ptn_internal_getrandmax(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_int(2147483647);
}

static PtnValue ptn_internal_getmypid(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
#if defined(_WIN32)
    return ptn_int((int64_t)_getpid());
#else
    return ptn_int((int64_t)getpid());
#endif
}

static PtnValue ptn_internal_php_sapi_name(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_string(PTN_PHP_SAPI_NAME);
}

static PtnValue ptn_internal_phpversion(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    if (argc == 0) {
        return ptn_string(PTN_PHP_VERSION);
    }

    PtnStringOperand extension = ptn_value_to_string_operand(args[0]);
    int modeled_extension =
        extension.data[0] == '\0' ||
        ptn_ascii_case_equal(extension.data, "core") ||
        ptn_ascii_case_equal(extension.data, "standard");
    ptn_string_operand_free(extension);
    if (modeled_extension) {
        return ptn_string(PTN_PHP_VERSION);
    }
    return ptn_bool(0);
}

static int ptn_digit_value_for_base(unsigned char byte, int base) {
    int value = -1;
    if (byte >= '0' && byte <= '9') {
        value = (int)(byte - '0');
    } else if (byte >= 'a' && byte <= 'f') {
        value = 10 + (int)(byte - 'a');
    } else if (byte >= 'A' && byte <= 'F') {
        value = 10 + (int)(byte - 'A');
    }
    return value >= 0 && value < base ? value : -1;
}

static PtnValue ptn_base_string_to_number(
    PtnRuntime *runtime,
    const char *string,
    size_t string_len,
    int base,
    char prefix,
    size_t line
) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }

    const char *end = string + string_len;
    while (end > start && isspace((unsigned char)*(end - 1))) {
        end--;
    }

    if ((end - start) >= 2 && start[0] == '0' && tolower((unsigned char)start[1]) == prefix) {
        start += 2;
    }

    int saw_digit = 0;
    int saw_invalid = 0;
    int fits_integer = 1;
    int64_t integer = 0;
    double floating = 0.0;

    for (const char *cursor = start; cursor < end; cursor++) {
        int digit = ptn_digit_value_for_base((unsigned char)*cursor, base);
        if (digit < 0) {
            saw_invalid = 1;
            continue;
        }
        saw_digit = 1;
        floating = (floating * (double)base) + (double)digit;
        if (fits_integer) {
            if (integer > (INT64_MAX - digit) / base) {
                fits_integer = 0;
            } else {
                integer = (integer * base) + digit;
            }
        }
    }

    if (saw_invalid) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Invalid characters passed for attempted conversion, these have been ignored",
            line
        );
    }
    if (!saw_digit) {
        return ptn_int(0);
    }
    return fits_integer ? ptn_int(integer) : ptn_float(floating);
}

static PtnValue ptn_internal_bindec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string.data, string.len, 2, 'b', line);
    ptn_string_operand_free(string);
    return value;
}

static PtnValue ptn_internal_hexdec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string.data, string.len, 16, 'x', line);
    ptn_string_operand_free(string);
    return value;
}

static PtnValue ptn_internal_octdec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string.data, string.len, 8, 'o', line);
    ptn_string_operand_free(string);
    return value;
}

static PtnValue ptn_internal_intval(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    if (argc >= 2 && args[0].type == PTN_STRING) {
        int64_t base = ptn_number_to_integer(ptn_to_number(args[1]));
        if (base == 0 || (base >= 2 && base <= 36)) {
            const char *start = (const char *)args[0].as.string.data;
            while (isspace((unsigned char)*start)) {
                start++;
            }
            errno = 0;
            long long integer = strtoll(start, NULL, (int)base);
            if (errno != ERANGE) {
                return ptn_int((int64_t)integer);
            }
        }
    }
    return ptn_cast_int(args[0]);
}

static PtnValue ptn_internal_chr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    int64_t integer = ptn_value_to_integer(args[0]);
    int64_t normalized = integer % 256;
    if (normalized < 0) {
        normalized += 256;
    }
    char *string = malloc(2);
    if (string == NULL) {
        ptn_abort_out_of_memory();
    }
    string[0] = (char)(unsigned char)normalized;
    string[1] = '\0';
    return ptn_owned_string_len(string, 1);
}

static PtnValue ptn_internal_ord(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    size_t len = string.len;
    int64_t byte = 0;
    if (len == 0) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "ord(): Providing an empty string is deprecated",
            line
        );
    } else {
        byte = (int64_t)(unsigned char)string.data[0];
        if (len != 1) {
            ptn_emit_deprecation(
                &runtime->diagnostics,
                "ord(): Providing a string that is not one byte long is deprecated. Use ord($str[0]) instead",
                line
            );
        }
    }
    ptn_string_operand_free(string);
    return ptn_int(byte);
}

static PtnValue ptn_internal_count(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    return ptn_count_value(runtime, args[0]);
}

static PtnValue ptn_internal_error_reporting(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_int(0);
}

static PtnCallFrame *ptn_current_call_frame(PtnRuntime *runtime, const char *function_name) {
    if (runtime->call_frame != NULL) {
        return runtime->call_frame;
    }
    char message[128];
    const char *format = strcmp(function_name, "func_num_args") == 0
        ? "%s() must be called from a function context"
        : "%s() cannot be called from the global scope";
    int written = snprintf(message, sizeof(message), format, function_name);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
    return NULL;
}

static PtnValue ptn_call_frame_arg_value(PtnRuntime *runtime, PtnCallFrame *frame, size_t position) {
    if (position < frame->parameter_count) {
        PtnValue value;
        if (ptn_symbols_get(&runtime->symbols, frame->parameter_names[position], &value)) {
            return ptn_value_clone_deref(value);
        }
        return ptn_null();
    }
    return ptn_value_clone_deref(frame->args[position]);
}

static PtnValue ptn_internal_func_num_args(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)args;
    (void)line;
    PtnCallFrame *frame = ptn_current_call_frame(runtime, "func_num_args");
    if (frame->argc > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_int((int64_t)frame->argc);
}

static PtnValue ptn_internal_func_get_arg(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnCallFrame *frame = ptn_current_call_frame(runtime, "func_get_arg");
    int64_t position = ptn_value_to_integer(args[0]);
    if (position < 0) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "func_get_arg(): Argument #1 ($position) must be greater than or equal to 0"
        );
    }
    if ((uint64_t)position >= (uint64_t)frame->argc) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function"
        );
    }
    return ptn_call_frame_arg_value(runtime, frame, (size_t)position);
}

static PtnValue ptn_internal_func_get_args(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)args;
    (void)line;
    PtnCallFrame *frame = ptn_current_call_frame(runtime, "func_get_args");
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < frame->argc; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key((int64_t)i),
            ptn_call_frame_arg_value(runtime, frame, i)
        );
    }
    return result;
}

static PtnValue ptn_internal_define(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_constant(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static int ptn_user_function_exists(const char *name);
static PtnValue ptn_internal_defined(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_function_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_array_key_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);

static const PtnInternalFunction *ptn_internal_functions(size_t *count) {
    /* Keep sorted by ASCII case-insensitive name for ptn_find_internal_function. */
    static const PtnInternalFunction functions[] = {
        { "_ptn_cow_debug_assert_balanced", 0, 0, ptn_internal__ptn_cow_debug_assert_balanced },
        { "_ptn_cow_debug_assert_counter", 2, 2, ptn_internal__ptn_cow_debug_assert_counter },
        { "_ptn_cow_debug_counter", 1, 1, ptn_internal__ptn_cow_debug_counter },
        { "_ptn_cow_debug_reset", 0, 0, ptn_internal__ptn_cow_debug_reset },
        { "abs", 1, 1, ptn_internal_abs },
        { "array_key_exists", 2, 2, ptn_internal_array_key_exists },
        { "array_pop", 1, 1, ptn_internal_array_pop },
        { "array_push", 1, PTN_VARIADIC_ARGS, ptn_internal_array_push },
        { "array_reverse", 1, 2, ptn_internal_array_reverse },
        { "array_shift", 1, 1, ptn_internal_array_shift },
        { "array_unshift", 1, PTN_VARIADIC_ARGS, ptn_internal_array_unshift },
        { "array_values", 1, 1, ptn_internal_array_values },
        { "bin2hex", 1, 1, ptn_internal_bin2hex },
        { "bindec", 1, 1, ptn_internal_bindec },
        { "ceil", 1, 1, ptn_internal_ceil },
        { "chr", 1, 1, ptn_internal_chr },
        { "chunk_split", 1, 3, ptn_internal_chunk_split },
        { "constant", 1, 1, ptn_internal_constant },
        { "count", 1, 1, ptn_internal_count },
        { "current", 1, 1, ptn_internal_current },
        { "define", 2, 2, ptn_internal_define },
        { "defined", 1, 1, ptn_internal_defined },
        { "dirname", 1, 1, ptn_internal_dirname },
        { "end", 1, 1, ptn_internal_end },
        { "error_reporting", 0, 1, ptn_internal_error_reporting },
        { "fdiv", 2, 2, ptn_internal_fdiv },
        { "file_put_contents", 2, 2, ptn_internal_file_put_contents },
        { "floor", 1, 1, ptn_internal_floor },
        { "func_get_arg", 1, 1, ptn_internal_func_get_arg },
        { "func_get_args", 0, 0, ptn_internal_func_get_args },
        { "func_num_args", 0, 0, ptn_internal_func_num_args },
        { "function_exists", 1, 1, ptn_internal_function_exists },
        { "getmypid", 0, 0, ptn_internal_getmypid },
        { "getrandmax", 0, 0, ptn_internal_getrandmax },
        { "gettype", 1, 1, ptn_internal_gettype },
        { "hex2bin", 1, 1, ptn_internal_hex2bin },
        { "hexdec", 1, 1, ptn_internal_hexdec },
        { "intdiv", 2, 2, ptn_internal_intdiv },
        { "intval", 1, 2, ptn_internal_intval },
        { "is_array", 1, 1, ptn_internal_is_array },
        { "is_bool", 1, 1, ptn_internal_is_bool },
        { "is_double", 1, 1, ptn_internal_is_float },
        { "is_finite", 1, 1, ptn_internal_is_finite },
        { "is_float", 1, 1, ptn_internal_is_float },
        { "is_infinite", 1, 1, ptn_internal_is_infinite },
        { "is_int", 1, 1, ptn_internal_is_int },
        { "is_integer", 1, 1, ptn_internal_is_int },
        { "is_long", 1, 1, ptn_internal_is_int },
        { "is_nan", 1, 1, ptn_internal_is_nan },
        { "is_null", 1, 1, ptn_internal_is_null },
        { "is_scalar", 1, 1, ptn_internal_is_scalar },
        { "is_string", 1, 1, ptn_internal_is_string },
        { "key", 1, 1, ptn_internal_key },
        { "md5", 1, 2, ptn_internal_md5 },
        { "next", 1, 1, ptn_internal_next },
        { "octdec", 1, 1, ptn_internal_octdec },
        { "ord", 1, 1, ptn_internal_ord },
        { "php_sapi_name", 0, 0, ptn_internal_php_sapi_name },
        { "phpversion", 0, 1, ptn_internal_phpversion },
        { "pi", 0, 0, ptn_internal_pi },
        { "prev", 1, 1, ptn_internal_prev },
        { "print_r", 1, 2, ptn_internal_print_r },
        { "quoted_printable_decode", 1, 1, ptn_internal_quoted_printable_decode },
        { "quotemeta", 1, 1, ptn_internal_quotemeta },
        { "reset", 1, 1, ptn_internal_reset },
        { "sha1", 1, 2, ptn_internal_sha1 },
        { "sha1_file", 1, 2, ptn_internal_sha1_file },
        { "soundex", 1, 1, ptn_internal_soundex },
        { "sqrt", 1, 1, ptn_internal_sqrt },
        { "str_contains", 2, 2, ptn_internal_str_contains },
        { "str_ends_with", 2, 2, ptn_internal_str_ends_with },
        { "str_rot13", 1, 1, ptn_internal_str_rot13 },
        { "str_starts_with", 2, 2, ptn_internal_str_starts_with },
        { "strcmp", 2, 2, ptn_internal_strcmp },
        { "strip_tags", 1, 1, ptn_internal_strip_tags },
        { "strlen", 1, 1, ptn_internal_strlen },
        { "substr", 2, 3, ptn_internal_substr },
        { "unlink", 1, 1, ptn_internal_unlink },
        { "var_dump", 1, PTN_VARIADIC_ARGS, ptn_internal_var_dump },
    };
    *count = sizeof(functions) / sizeof(functions[0]);
    return functions;
}

static const PtnInternalFunction *ptn_find_internal_function(const char *name) {
    size_t count = 0;
    const PtnInternalFunction *functions = ptn_internal_functions(&count);
    size_t low = 0;
    size_t high = count;
    while (low < high) {
        size_t mid = low + ((high - low) / 2);
        int ordering = ptn_ascii_case_compare(name, functions[mid].name);
        if (ordering == 0) {
            return &functions[mid];
        }
        if (ordering < 0) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    return NULL;
}

static PtnValue ptn_internal_define(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    char *name = ptn_value_to_string(args[0]);
    int did_define = ptn_runtime_define_constant_if_absent(runtime, name, args[1], line);
    free(name);
    return ptn_bool(did_define);
}

static PtnValue ptn_internal_constant(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    PtnValue value = ptn_read_constant(runtime, name);
    free(name);
    return value;
}

static PtnValue ptn_internal_defined(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    int exists = ptn_runtime_constant_is_defined(runtime, name);
    free(name);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_function_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    int exists = ptn_user_function_exists(name) || ptn_find_internal_function(name) != NULL;
    free(name);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_array_key_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_array_key_exists_value(runtime, args[0], args[1], line);
}

static PTN_UNUSED PtnValue ptn_call_internal(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line) {
    const PtnInternalFunction *function = ptn_find_internal_function(name);
    if (function != NULL) {
        if (argc < function->min_args) {
            ptn_emit_argument_count_error(&runtime->diagnostics, name, function->min_args, argc);
            exit(255);
        }
        if (function->max_args != PTN_VARIADIC_ARGS && argc > function->max_args) {
            ptn_emit_too_many_arguments_error(&runtime->diagnostics, name, function->max_args, argc);
            exit(255);
        }
        return function->handler(runtime, argc, args, line);
    }

    ptn_emit_undefined_function_error(&runtime->diagnostics, name);
    exit(255);
    return ptn_null();
}
/* PTN_INTERNAL_FUNCTIONS_END */
