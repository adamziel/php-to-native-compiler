        snprintf(buffer, buffer_len, "%lld", (long long)value.as.integer);
    } else {
        snprintf(buffer, buffer_len, "%.14g", value.as.floating);
    }
}

static PTN_UNUSED int ptn_compare_number_and_string(PtnValue number, PtnString string, int number_is_left) {
    char number_string[128];
    ptn_number_value_to_string(number, number_string, sizeof(number_string));
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

static PTN_UNUSED int ptn_compare_strings_loose(PtnString left, PtnString right) {
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

static PTN_UNUSED int ptn_compare_equal(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_identical(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_not_identical(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_order(PtnValue left, PtnValue right);
static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand(PtnValue value);
static PTN_UNUSED void ptn_string_operand_free(PtnStringOperand operand);

static PTN_UNUSED PtnArrayEntry *ptn_array_entry_for_key(PtnArray *array, PtnArrayKey key) {
    size_t index = ptn_array_find_key(array, key);
    return index < array->len ? &array->entries[index] : NULL;
}

static PTN_UNUSED const char *ptn_offset_container_type_name(PtnValue value) {
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
        case PTN_ARRAY:
            return "array";
        case PTN_EXCEPTION:
            return "object";
    }
    return "unknown";
}

static PTN_UNUSED void ptn_emit_array_runtime_diagnostic(const char *kind, const char *message, size_t line) {
    fputc('\n', stdout);
    fputs(kind, stdout);
    fputs(": ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_null_array_offset_deprecation(size_t line) {
    ptn_emit_array_runtime_diagnostic(
        "Deprecated",
        "Using null as an array offset is deprecated, use an empty string instead",
        line
    );
}

static PTN_UNUSED void ptn_emit_foreach_non_array_warning(PtnValue value, size_t line) {
    char message[128];
    snprintf(
        message,
        sizeof(message),
        "foreach() argument must be of type array|object, %s given",
        ptn_offset_container_type_name(value)
    );
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    (void)runtime;
    PtnArrayIterator iterator;
    iterator.array = NULL;
    iterator.index = 0;
    iterator.valid = 0;
    if (value.type != PTN_ARRAY) {
        ptn_emit_foreach_non_array_warning(value, line);
        return iterator;
    }
    iterator.array = value.as.array;
    iterator.valid = iterator.array->len != 0;
    return iterator;
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_key(PtnArrayIterator *iterator) {
    PtnArrayKey key = iterator->array->entries[iterator->index].key;
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_string(key.as.string);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_value(PtnArrayIterator *iterator) {
    return ptn_value_borrow(iterator->array->entries[iterator->index].value);
}

static PTN_UNUSED void ptn_array_iterator_advance(PtnArrayIterator *iterator) {
    iterator->index++;
    iterator->valid = iterator->array != NULL && iterator->index < iterator->array->len;
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

    size_t key_len = strlen(key.as.string);
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

static PTN_UNUSED void ptn_emit_undefined_array_key_warning(PtnArrayKey key, size_t line) {
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
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
    free(message);
    free(display);
}

static PTN_UNUSED void ptn_emit_string_offset_cast_warning(size_t line) {
    ptn_emit_array_runtime_diagnostic("Warning", "String offset cast occurred", line);
}

static PTN_UNUSED void ptn_emit_illegal_string_offset_warning(const char *key, size_t line) {
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
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
    free(message);
}

static PTN_UNUSED void ptn_emit_uninitialized_string_offset_warning(int64_t offset, size_t line) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Uninitialized string offset %lld", (long long)offset);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED void ptn_emit_illegal_string_offset_integer_warning(int64_t offset, size_t line) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Illegal string offset %lld", (long long)offset);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED void ptn_emit_string_offset_assignment_byte_warning(size_t line) {
    ptn_emit_array_runtime_diagnostic(
        "Warning",
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

static PTN_UNUSED int ptn_string_offset_from_value(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int quiet,
    int64_t *offset
) {
    switch (key_value.type) {
        case PTN_INT:
            *offset = key_value.as.integer;
            return 1;
        case PTN_BOOL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = key_value.as.boolean ? 1 : 0;
            return 1;
        case PTN_NULL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = 0;
            return 1;
        case PTN_FLOAT:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = (int64_t)key_value.as.floating;
            return 1;
        case PTN_STRING: {
            int warn_illegal = 0;
            const char *key_string = (const char *)key_value.as.string.data;
            if (ptn_string_to_offset(key_string, offset, &warn_illegal)) {
                if (warn_illegal) {
                    if (quiet) {
                        return 0;
                    }
                    ptn_emit_illegal_string_offset_warning(key_string, line);
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
        case PTN_EXCEPTION:
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type object on string");
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
    if (!ptn_string_offset_from_value(runtime, key_value, line, quiet, &offset)) {
        return ptn_lookup_missing();
    }
    size_t index = 0;
    if (!ptn_string_offset_index(container.as.string.len, offset, &index)) {
        if (!quiet) {
            ptn_emit_uninitialized_string_offset_warning(offset, line);
            return ptn_lookup_found(ptn_string(""));
        }
        return ptn_lookup_missing();
    }

    char *result = malloc(2);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = (char)container.as.string.data[index];
    result[1] = '\0';
    return ptn_lookup_found(ptn_owned_string_len(result, 1));
}

static PTN_UNUSED int ptn_string_offset_assignment_index(
    size_t string_len,
    int64_t offset,
    size_t line,
    size_t *index,
    size_t *new_len
) {
    if (offset >= 0) {
        uint64_t positive = (uint64_t)offset;
        if (positive >= (uint64_t)SIZE_MAX - 1) {
            ptn_abort_out_of_memory();
        }
        *index = (size_t)positive;
        *new_len = *index >= string_len ? *index + 1 : string_len;
        return 1;
    }

    if (ptn_string_offset_index(string_len, offset, index)) {
        *new_len = string_len;
        return 1;
    }

    ptn_emit_illegal_string_offset_integer_warning(offset, line);
    return 0;
}

static PTN_UNUSED unsigned char ptn_string_offset_assignment_byte(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    if (value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
    }

    PtnStringOperand string = ptn_value_to_string_operand(value);
    if (string.len == 0) {
        ptn_string_operand_free(string);
        ptn_throw_exception(runtime, "Error", "Cannot assign an empty string to a string offset");
        return 0;
    }
    if (string.len > 1) {
        ptn_emit_string_offset_assignment_byte_warning(line);
    }

    unsigned char byte = (unsigned char)string.data[0];
    ptn_string_operand_free(string);
    return byte;
}

static PTN_UNUSED void ptn_runtime_string_offset_set(
    PtnRuntime *runtime,
    const char *name,
    PtnValue container,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    int64_t offset = 0;
    if (!ptn_string_offset_from_value(runtime, key_value, line, 0, &offset)) {
        return;
    }

    size_t index = 0;
    size_t new_len = 0;
    if (!ptn_string_offset_assignment_index(container.as.string.len, offset, line, &index, &new_len)) {
        return;
    }

    unsigned char byte = ptn_string_offset_assignment_byte(runtime, value, line);
    char *buffer = malloc(new_len + 1);
    if (buffer == NULL) {
        ptn_abort_out_of_memory();
    }
    if (container.as.string.len != 0) {
        memcpy(buffer, container.as.string.data, container.as.string.len);
    }
    if (new_len > container.as.string.len) {
        memset(buffer + container.as.string.len, ' ', new_len - container.as.string.len);
    }
    buffer[index] = (char)byte;
    buffer[new_len] = '\0';

    PtnValue updated = ptn_owned_string_len(buffer, new_len);
    ptn_runtime_write_variable(runtime, name, updated);
    ptn_value_destroy(&updated);
}

static PTN_UNUSED PtnLookupResult ptn_offset_lookup(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line, int quiet) {
    if (container.type == PTN_STRING) {
        return ptn_string_offset_lookup(runtime, container, key_value, line, quiet);
    }

    if (container.type != PTN_ARRAY) {
        if (!quiet) {
            const char *prefix = "Trying to access array offset on value of type ";
            const char *type_name = ptn_offset_container_type_name(container);
            char message[128];
            int written = snprintf(message, sizeof(message), "%s%s", prefix, type_name);
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_array_runtime_diagnostic("Warning", message, line);
        }
        return ptn_lookup_missing();
    }

    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(line);
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    if (entry == NULL) {
        if (!quiet) {
            ptn_emit_undefined_array_key_warning(key, line);
        }
        ptn_array_key_free(key);
        return ptn_lookup_missing();
    }
    PtnValue value = ptn_value_clone(entry->value);
    ptn_array_key_free(key);
    return ptn_lookup_found(value);
}

static PTN_UNUSED PtnValue ptn_array_read(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    PtnLookupResult result = ptn_offset_lookup(runtime, container, key_value, line, 0);
    if (!result.exists) {
        return ptn_null();
    }
    return result.value;
}

static PTN_UNUSED int ptn_offset_is_set(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    if (container.type == PTN_STRING) {
        int64_t offset = 0;
        size_t index = 0;
        return ptn_string_offset_from_value(runtime, key_value, line, 1, &offset) &&
            ptn_string_offset_index(container.as.string.len, offset, &index);
    }

    if (container.type != PTN_ARRAY) {
        return 0;
    }
    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(line);
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    int result = entry != NULL && entry->value.type != PTN_NULL;
    ptn_array_key_free(key);
    return result;
}

static PTN_UNUSED int ptn_offset_is_empty(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    if (container.type == PTN_STRING) {
        int64_t offset = 0;
        size_t index = 0;
        if (!ptn_string_offset_from_value(runtime, key_value, line, 1, &offset) ||
            !ptn_string_offset_index(container.as.string.len, offset, &index)) {
            return 1;
        }
        return container.as.string.data[index] == '0';
    }

    if (container.type != PTN_ARRAY) {
        return 1;
    }
    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(line);
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    int result = entry == NULL || !ptn_is_truthy(entry->value);
    ptn_array_key_free(key);
    return result;
}

static PTN_UNUSED PtnValue ptn_array_key_value(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_owned_string(ptn_duplicate_string(key.as.string));
}

static PTN_UNUSED void ptn_emit_assign_op_missing_array_key(PtnValue key_value, size_t line) {
    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(line);
    }
    PtnArrayKey key = ptn_array_key_from_value(key_value);
    ptn_emit_undefined_array_key_warning(key, line);
    ptn_array_key_free(key);
}

static PTN_UNUSED void ptn_runtime_array_warn_missing_base_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnValue container;
    if (!ptn_symbols_get(&runtime->symbols, name, &container)) {
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
    }
}

static PTN_UNUSED PtnArray *ptn_runtime_array_detach_variable(PtnRuntime *runtime, const char *name) {
    size_t index = ptn_symbols_find(&runtime->symbols, name);
    if (index >= runtime->symbols.len || runtime->symbols.items[index].value.type != PTN_ARRAY) {
        return NULL;
    }
    return ptn_array_detach_value(&runtime->symbols.items[index].value);
}

static PTN_UNUSED void ptn_runtime_separate_array_variable(PtnRuntime *runtime, const char *name) {
    (void)ptn_runtime_array_detach_variable(runtime, name);
}

static PTN_UNUSED PtnArray *ptn_value_replace_with_empty_array(PtnValue *value) {
    ptn_value_destroy(value);
    *value = ptn_array_from_literal_entries(0, NULL);
    return value->as.array;
}

static PTN_UNUSED PtnArray *ptn_runtime_array_root_for_write(
    PtnRuntime *runtime,
    const char *name,
    size_t line
) {
    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot != NULL) {
        if (slot->type == PTN_ARRAY) {
            return ptn_array_detach_value(slot);
        }
        if (slot->type == PTN_NULL) {
            return ptn_value_replace_with_empty_array(slot);
        }
        ptn_emit_array_runtime_diagnostic("Warning", "Cannot use a scalar value as an array", line);
        return NULL;
    }

    PtnValue array = ptn_array_from_literal_entries(0, NULL);
    ptn_runtime_write_variable(runtime, name, array);
    ptn_value_destroy(&array);
    slot = ptn_symbols_value_slot(&runtime->symbols, name);
    return slot != NULL && slot->type == PTN_ARRAY ? slot->as.array : NULL;
}

static PTN_UNUSED PtnArrayKey ptn_array_path_segment_key(
    PtnArray *array,
    const PtnArrayPathSegment *segment
) {
    if (segment->append) {
        return ptn_array_int_key(array->next_auto_key);
    }
    return ptn_array_key_from_value(segment->value);
}

static PTN_UNUSED void ptn_array_path_emit_null_key_deprecation(
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_null_key_deprecation
) {
    if (emit_null_key_deprecation && !segment->append && segment->value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(line);
    }
}

static PTN_UNUSED PtnArray *ptn_array_descend_for_write(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_array_path_emit_null_key_deprecation(segment, line, emit_null_key_deprecation);
    PtnArrayKey key = ptn_array_path_segment_key(array, segment);
    PtnArrayEntry *entry = segment->append ? NULL : ptn_array_entry_for_key(array, key);

    if (entry == NULL) {
        PtnValue child = ptn_array_from_literal_entries(0, NULL);
        ptn_array_set_entry(array, key, child);
        return array->entries[array->len - 1].value.as.array;
    }

    ptn_array_key_free(key);
    if (entry->value.type == PTN_ARRAY) {
        return ptn_array_detach_value(&entry->value);
    }
    if (entry->value.type == PTN_NULL) {
        return ptn_value_replace_with_empty_array(&entry->value);
    }

    (void)runtime;
    ptn_emit_array_runtime_diagnostic("Warning", "Cannot use a scalar value as an array", line);
    return NULL;
}

static PTN_UNUSED void ptn_array_set_path_leaf(
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_array_path_emit_null_key_deprecation(segment, line, emit_null_key_deprecation);
    PtnArrayKey key = ptn_array_path_segment_key(array, segment);
    ptn_array_set_entry(array, key, ptn_value_clone(value));
}

static PTN_UNUSED void ptn_runtime_array_path_set_impl(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (segment_count == 0) {
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot != NULL && slot->type == PTN_STRING && segment_count == 1) {
        if (segments[0].append) {
            ptn_throw_exception(runtime, "Error", "[] operator not supported for strings");
            return;
        }
        ptn_runtime_string_offset_set(runtime, name, ptn_value_borrow(*slot), segments[0].value, value, line);
        return;
    }

    PtnArray *array = ptn_runtime_array_root_for_write(runtime, name, line);
    if (array == NULL) {
        return;
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_write(
            runtime,
            array,
            &segments[i],
            line,
            emit_null_key_deprecation
        );
        if (array == NULL) {
            return;
        }
    }

    ptn_array_set_path_leaf(
        array,
        &segments[segment_count - 1],
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

static PTN_UNUSED void ptn_runtime_array_path_set_from_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_path_set_impl(runtime, name, segments, segment_count, value, line, 0);
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

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot == NULL) {
        if (!segments[0].append) {
            ptn_emit_assign_op_missing_array_key(segments[0].value, line);
        }
        return ptn_null();
    }
    if (slot->type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception(runtime, "Error", "[] operator not supported for strings");
            return ptn_null();
        }
        ptn_throw_exception(runtime, "Error", "Cannot use assign-op operators with string offsets");
        return ptn_null();
    }

    PtnValue container = ptn_value_borrow(*slot);
    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return ptn_null();
        }
        if (segment->value.type == PTN_NULL) {
            ptn_emit_null_array_offset_deprecation(line);
        }
        PtnArrayKey key = ptn_array_key_from_value(segment->value);
        if (container.type == PTN_ARRAY) {
            PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
            if (entry == NULL) {
                ptn_emit_undefined_array_key_warning(key, line);
                ptn_array_key_free(key);
                return ptn_null();
            }
            ptn_array_key_free(key);
            if (i + 1 == segment_count) {
                return ptn_value_clone(entry->value);
            }
            container = ptn_value_borrow(entry->value);
            continue;
        }
        if (container.type == PTN_NULL) {
            ptn_emit_undefined_array_key_warning(key, line);
            ptn_array_key_free(key);
            return ptn_null();
        }
        ptn_array_key_free(key);
        return ptn_null();
    }
    return ptn_null();
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
    (void)line;
    if (segment_count == 0) {
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot == NULL) {
        return;
    }
    if (slot->type == PTN_STRING) {
        ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        return;
    }
    if (slot->type != PTN_ARRAY) {
        return;
    }

    PtnArray *array = ptn_array_detach_value(slot);
    for (size_t i = 0; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return;
        }
        PtnArrayKey key = ptn_array_key_from_value(segment->value);
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
        ptn_array_key_free(key);
        if (entry == NULL || entry->value.type != PTN_ARRAY) {
            return;
        }
        array = ptn_array_detach_value(&entry->value);
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append) {
        return;
    }
    PtnArrayKey key = ptn_array_key_from_value(leaf->value);
    (void)ptn_array_unset_entry(array, key);
}

static PTN_UNUSED PtnValue ptn_array_current_value(PtnArray *array) {
    if (array->current_index >= array->len) {
        return ptn_bool(0);
    }
    return ptn_value_clone(array->entries[array->current_index].value);
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
    return ptn_value_clone(array->entries[array->current_index].value);
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
    PtnValue removed = array->entries[removed_index].value;
    ptn_array_key_free(array->entries[removed_index].key);
    array->len--;
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    return removed;
}

static PTN_UNUSED PtnValue ptn_array_shift_value(PtnArray *array) {
    if (array->len == 0) {
        array->current_index = 0;
        return ptn_null();
    }

    PtnValue removed = array->entries[0].value;
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
    return removed;
}

static PTN_UNUSED int64_t ptn_array_push_values(PtnArray *array, size_t argc, const PtnValue *values) {
