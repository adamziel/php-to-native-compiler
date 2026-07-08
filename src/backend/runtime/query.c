/* Query string codec parity helpers shared by standard and request internals. */
static char *ptn_url_decode_component(const char *input, size_t len, size_t *output_len_out) {
    return ptn_url_decode_bytes(input, len, 1, output_len_out);
}

static int ptn_url_encode_is_unreserved(unsigned char byte, int raw) {
    return (byte >= 'A' && byte <= 'Z') ||
        (byte >= 'a' && byte <= 'z') ||
        (byte >= '0' && byte <= '9') ||
        byte == '-' ||
        byte == '_' ||
        byte == '.' ||
        (raw && byte == '~');
}

static PtnValue ptn_url_encode_value(PtnStringOperand string, int raw) {
    static const char hex[] = "0123456789ABCDEF";
    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    for (size_t i = 0; i < string.len; i++) {
        unsigned char byte = (unsigned char)string.data[i];
        if (ptn_url_encode_is_unreserved(byte, raw)) {
            ptn_string_buffer_append_char(&output, (char)byte);
        } else if (!raw && byte == ' ') {
            ptn_string_buffer_append_char(&output, '+');
        } else {
            char encoded[3];
            encoded[0] = '%';
            encoded[1] = hex[(byte >> 4) & 0x0f];
            encoded[2] = hex[byte & 0x0f];
            ptn_string_buffer_append_len(&output, encoded, sizeof(encoded));
        }
    }
    return ptn_owned_string_len(output.data, output.len);
}

static PtnValue ptn_internal_urlencode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "urlencode", 1, "string", args[0], line);
    PtnValue result = ptn_url_encode_value(string, 0);
    ptn_string_operand_free(string);
    return result;
}

static PtnValue ptn_internal_rawurlencode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "rawurlencode", 1, "string", args[0], line);
    PtnValue result = ptn_url_encode_value(string, 1);
    ptn_string_operand_free(string);
    return result;
}

static void ptn_http_build_query_append_encoded(
    PtnStringBuffer *output,
    const char *data,
    size_t len,
    int raw
) {
    PtnStringOperand operand = ptn_string_operand_borrowed_len(data, len);
    PtnValue encoded = ptn_url_encode_value(operand, raw);
    PtnStringOperand encoded_string = ptn_value_to_string_operand(encoded);
    ptn_string_buffer_append_len(output, encoded_string.data, encoded_string.len);
    ptn_string_operand_free(encoded_string);
    ptn_value_destroy(&encoded);
}

static void ptn_http_build_query_append_key(
    PtnStringBuffer *output,
    const char *key,
    size_t key_len,
    int raw
) {
    ptn_http_build_query_append_encoded(output, key, key_len, raw);
}

static char *ptn_http_build_query_key_from_array_key(PtnArrayKey key, const char *numeric_prefix) {
    if (key.type == PTN_ARRAY_KEY_STRING) {
        return ptn_duplicate_string_len(key.as.string, key.string_len);
    }
    int needed = snprintf(NULL, 0, "%s%lld", numeric_prefix, (long long)key.as.integer);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *result = malloc((size_t)needed + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(result, (size_t)needed + 1, "%s%lld", numeric_prefix, (long long)key.as.integer);
    return result;
}

static void ptn_http_build_query_append_value(
    PtnRuntime *runtime,
    PtnStringBuffer *output,
    const char *key,
    size_t key_len,
    PtnValue value,
    const char *arg_separator,
    int raw,
    PtnObject **seen_objects,
    size_t seen_objects_len,
    size_t line
);

static void ptn_http_build_query_append_pair(
    PtnRuntime *runtime,
    PtnStringBuffer *output,
    const char *key,
    size_t key_len,
    PtnValue value,
    const char *arg_separator,
    int raw,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_NULL || value.type == PTN_RESOURCE) {
        return;
    }
    if (output->len > 0) {
        ptn_string_buffer_append(output, arg_separator);
    }
    ptn_http_build_query_append_key(output, key, key_len, raw);
    ptn_string_buffer_append_char(output, '=');
    PtnStringOperand string = value.type == PTN_BOOL
        ? ptn_string_operand_borrowed(value.as.boolean ? "1" : "0")
        : ptn_value_to_string_operand_with_runtime(runtime, value, line);
    ptn_http_build_query_append_encoded(output, string.data, string.len, raw);
    ptn_string_operand_free(string);
}

static void ptn_http_build_query_append_array(
    PtnRuntime *runtime,
    PtnStringBuffer *output,
    const char *key,
    size_t key_len,
    PtnArray *array,
    const char *arg_separator,
    int raw,
    PtnObject **seen_objects,
    size_t seen_objects_len,
    size_t line
) {
    for (size_t i = 0; i < array->len; i++) {
        char *child_key = ptn_http_build_query_key_from_array_key(array->entries[i].key, "");
        size_t child_key_len = strlen(child_key);
        int needed = snprintf(NULL, 0, "%.*s[%.*s]", (int)key_len, key, (int)child_key_len, child_key);
        if (needed < 0) {
            free(child_key);
            ptn_abort_out_of_memory();
        }
        char *compound = malloc((size_t)needed + 1);
        if (compound == NULL) {
            free(child_key);
            ptn_abort_out_of_memory();
        }
        snprintf(compound, (size_t)needed + 1, "%.*s[%.*s]", (int)key_len, key, (int)child_key_len, child_key);
        ptn_http_build_query_append_value(
            runtime,
            output,
            compound,
            (size_t)needed,
            array->entries[i].value,
            arg_separator,
            raw,
            seen_objects,
            seen_objects_len,
            line
        );
        free(compound);
        free(child_key);
    }
}

static void ptn_http_build_query_append_object(
    PtnRuntime *runtime,
    PtnStringBuffer *output,
    const char *key,
    size_t key_len,
    PtnObject *object,
    const char *arg_separator,
    int raw,
    PtnObject **seen_objects,
    size_t seen_objects_len,
    size_t line
) {
    const char *access_scope = runtime == NULL ? NULL : runtime->current_class_name;
    for (size_t i = 0; i < object->properties->len; i++) {
        PtnArrayEntry *entry = &object->properties->entries[i];
        if (!ptn_object_property_visible_for_foreach(runtime, object, entry->key, access_scope)) {
            continue;
        }
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(object, entry->key.as.string);
        const char *property = metadata == NULL ? entry->key.as.string : metadata->display_name;
        size_t property_len = strlen(property);
        if (key == NULL) {
            ptn_http_build_query_append_value(
                runtime,
                output,
                property,
                property_len,
                entry->value,
                arg_separator,
                raw,
                seen_objects,
                seen_objects_len,
                line
            );
            continue;
        }
        int needed = snprintf(NULL, 0, "%.*s[%.*s]", (int)key_len, key, (int)property_len, property);
        if (needed < 0) {
            ptn_abort_out_of_memory();
        }
        char *compound = malloc((size_t)needed + 1);
        if (compound == NULL) {
            ptn_abort_out_of_memory();
        }
        snprintf(compound, (size_t)needed + 1, "%.*s[%.*s]", (int)key_len, key, (int)property_len, property);
        ptn_http_build_query_append_value(
            runtime,
            output,
            compound,
            (size_t)needed,
            entry->value,
            arg_separator,
            raw,
            seen_objects,
            seen_objects_len,
            line
        );
        free(compound);
    }
}

static void ptn_http_build_query_append_value(
    PtnRuntime *runtime,
    PtnStringBuffer *output,
    const char *key,
    size_t key_len,
    PtnValue value,
    const char *arg_separator,
    int raw,
    PtnObject **seen_objects,
    size_t seen_objects_len,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_ARRAY) {
        ptn_http_build_query_append_array(
            runtime,
            output,
            key,
            key_len,
            resolved.as.array,
            arg_separator,
            raw,
            seen_objects,
            seen_objects_len,
            line
        );
        return;
    }
    if (resolved.type == PTN_OBJECT) {
        for (size_t i = 0; i < seen_objects_len; i++) {
            if (seen_objects[i] == resolved.as.object) {
                return;
            }
        }
        PtnObject **nested_seen = malloc(sizeof(PtnObject *) * (seen_objects_len + 1));
        if (nested_seen == NULL) {
            ptn_abort_out_of_memory();
        }
        if (seen_objects_len > 0) {
            memcpy(nested_seen, seen_objects, sizeof(PtnObject *) * seen_objects_len);
        }
        nested_seen[seen_objects_len] = resolved.as.object;
        ptn_http_build_query_append_object(
            runtime,
            output,
            key,
            key_len,
            resolved.as.object,
            arg_separator,
            raw,
            nested_seen,
            seen_objects_len + 1,
            line
        );
        free(nested_seen);
        return;
    }
    ptn_http_build_query_append_pair(runtime, output, key, key_len, resolved, arg_separator, raw, line);
}

static PtnValue ptn_internal_http_build_query(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue data = ptn_value_deref(args[0]);
    if (data.type != PTN_ARRAY && data.type != PTN_OBJECT) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "http_build_query(): Argument #1 ($data) must be of type array, %s given",
            ptn_offset_container_type_name(data)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }

    char *numeric_prefix = argc >= 2 ? ptn_value_to_string(args[1]) : ptn_duplicate_string("");
    char *arg_separator = NULL;
    if (argc >= 3 && ptn_value_deref(args[2]).type != PTN_NULL) {
        arg_separator = ptn_value_to_string(args[2]);
    } else {
        arg_separator = ptn_duplicate_string("&");
    }
    int64_t encoding_type = argc >= 4
        ? ptn_internal_expect_integer_arg(runtime, "http_build_query", 4, "encoding_type", args[3], line)
        : 1;
    int raw = encoding_type == 2;

    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    if (data.type == PTN_ARRAY) {
        for (size_t i = 0; i < data.as.array->len; i++) {
            char *key = ptn_http_build_query_key_from_array_key(data.as.array->entries[i].key, numeric_prefix);
            ptn_http_build_query_append_value(
                runtime,
                &output,
                key,
                strlen(key),
                data.as.array->entries[i].value,
                arg_separator,
                raw,
                NULL,
                0,
                line
            );
            free(key);
        }
    } else {
        PtnObject *seen_object = data.as.object;
        ptn_http_build_query_append_object(
            runtime,
            &output,
            NULL,
            0,
            data.as.object,
            arg_separator,
            raw,
            &seen_object,
            1,
            line
        );
    }

    free(numeric_prefix);
    free(arg_separator);
    return ptn_owned_string_len(output.data, output.len);
}


static PtnValue ptn_url_decode_value(PtnStringOperand string, int plus_as_space) {
    size_t output_len = 0;
    char *output = ptn_url_decode_bytes(string.data, string.len, plus_as_space, &output_len);
    return ptn_owned_string_len(output, output_len);
}

static PtnValue ptn_internal_urldecode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "urldecode", 1, "string", args[0], line);
    PtnValue result = ptn_url_decode_value(string, 1);
    ptn_string_operand_free(string);
    return result;
}

static PtnValue ptn_internal_rawurldecode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "rawurldecode", 1, "string", args[0], line);
    PtnValue result = ptn_url_decode_value(string, 0);
    ptn_string_operand_free(string);
    return result;
}


static char *ptn_parse_str_mangle_name(const char *input, size_t len, size_t *output_len_out) {
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }
    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        char byte = input[i];
        output[out++] = (byte == ' ' || byte == '.' || byte == '[') ? '_' : byte;
    }
    output[out] = '\0';
    *output_len_out = out;
    return output;
}

typedef struct {
    PtnArrayKey key;
    int append;
} PtnParseStrPathSegment;

typedef struct {
    PtnParseStrPathSegment *segments;
    size_t len;
    size_t capacity;
} PtnParseStrPath;

static void ptn_parse_str_path_push(PtnParseStrPath *path, PtnArrayKey key, int append) {
    if (path->len == path->capacity) {
        size_t new_capacity = path->capacity == 0 ? 4 : path->capacity * 2;
        if (new_capacity < path->capacity) {
            ptn_abort_out_of_memory();
        }
        PtnParseStrPathSegment *segments = realloc(path->segments, new_capacity * sizeof(PtnParseStrPathSegment));
        if (segments == NULL) {
            ptn_abort_out_of_memory();
        }
        path->segments = segments;
        path->capacity = new_capacity;
    }
    path->segments[path->len].key = key;
    path->segments[path->len].append = append;
    path->len++;
}

static void ptn_parse_str_path_free(PtnParseStrPath *path) {
    for (size_t i = 0; i < path->len; i++) {
        ptn_array_key_free(path->segments[i].key);
    }
    free(path->segments);
}

static PtnArrayKey ptn_parse_str_key_from_decoded(const char *data, size_t len) {
    int64_t integer = 0;
    char *key = ptn_duplicate_string_len(data, len);
    if (ptn_string_is_integer_array_key(key, &integer)) {
        free(key);
        return ptn_array_int_key(integer);
    }
    free(key);
    return ptn_array_string_key_len(data, len);
}

static PtnParseStrPath ptn_parse_str_parse_key(const char *data, size_t len) {
    PtnParseStrPath path = {0};
    size_t name_start = 0;
    while (name_start < len && data[name_start] == ' ') {
        name_start++;
    }
    size_t base_end = name_start;
    while (base_end < len && data[base_end] != '[') {
        base_end++;
    }

    size_t mangled_len = 0;
    char *mangled = ptn_parse_str_mangle_name(data + name_start, base_end - name_start, &mangled_len);
    ptn_parse_str_path_push(&path, ptn_parse_str_key_from_decoded(mangled, mangled_len), 0);
    free(mangled);

    size_t cursor = base_end;
    int valid_segment_count = 0;
    while (cursor < len && data[cursor] == '[') {
        size_t close = cursor + 1;
        while (close < len && data[close] != ']') {
            close++;
        }
        if (close >= len) {
            if (valid_segment_count == 0) {
                ptn_array_key_free(path.segments[0].key);
                path.len = 0;
                mangled = ptn_parse_str_mangle_name(data + name_start, len - name_start, &mangled_len);
                ptn_parse_str_path_push(&path, ptn_parse_str_key_from_decoded(mangled, mangled_len), 0);
                free(mangled);
            }
            break;
        }
        size_t segment_len = close - cursor - 1;
        if (segment_len == 0) {
            ptn_parse_str_path_push(&path, ptn_array_int_key(0), 1);
        } else {
            ptn_parse_str_path_push(
                &path,
                ptn_parse_str_key_from_decoded(data + cursor + 1, segment_len),
                0
            );
        }
        valid_segment_count++;
        cursor = close + 1;
    }
    return path;
}

static void ptn_parse_str_assign(PtnArray *array, PtnParseStrPath *path, size_t index, PtnValue value) {
    PtnParseStrPathSegment *segment = &path->segments[index];
    PtnArrayKey key = segment->append
        ? ptn_array_int_key(array->next_auto_key)
        : ptn_array_key_clone(segment->key);
    if (index + 1 == path->len) {
        ptn_array_set_entry(array, key, value);
        return;
    }
    PtnArrayKey lookup_key = ptn_array_key_clone(key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(array, lookup_key);
    if (entry == NULL || ptn_value_deref(entry->value).type != PTN_ARRAY) {
        ptn_array_set_entry(array, key, ptn_array_from_literal_entries(0, NULL));
        entry = ptn_array_entry_for_key(array, lookup_key);
    } else {
        ptn_array_key_free(key);
    }
    ptn_array_key_free(lookup_key);
    PtnValue child = ptn_value_deref(entry->value);
    ptn_parse_str_assign(child.as.array, path, index + 1, value);
}

static int ptn_parse_str_is_separator(char byte, const char *separators) {
    if (separators == NULL || separators[0] == '\0') {
        return byte == '&';
    }
    for (const char *cursor = separators; *cursor != '\0'; cursor++) {
        if (byte == *cursor) {
            return 1;
        }
    }
    return 0;
}

typedef char *(*PtnParseStrDecodedConverter)(
    PtnRuntime *runtime,
    const char *data,
    size_t len,
    void *context,
    size_t *output_len_out
);

static char *ptn_parse_str_convert_decoded_component(
    PtnRuntime *runtime,
    char *decoded,
    size_t decoded_len,
    PtnParseStrDecodedConverter converter,
    void *converter_context,
    size_t *converted_len_out
) {
    if (converter == NULL) {
        *converted_len_out = decoded_len;
        return decoded;
    }
    char *converted = converter(runtime, decoded, decoded_len, converter_context, converted_len_out);
    free(decoded);
    return converted;
}

static PtnValue ptn_parse_str_to_array_with_converter(
    PtnRuntime *runtime,
    const char *data,
    size_t len,
    const char *separators,
    PtnParseStrDecodedConverter converter,
    void *converter_context
) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    size_t cursor = 0;
    while (cursor <= len) {
        size_t pair_start = cursor;
        while (cursor < len && !ptn_parse_str_is_separator(data[cursor], separators)) {
            cursor++;
        }
        size_t pair_len = cursor - pair_start;
        if (pair_len != 0) {
            const char *pair = data + pair_start;
            size_t equals = 0;
            while (equals < pair_len && pair[equals] != '=') {
                equals++;
            }
            size_t key_decoded_len = 0;
            char *key_decoded = ptn_url_decode_component(pair, equals, &key_decoded_len);
            size_t value_decoded_len = 0;
            char *value_decoded = equals < pair_len
                ? ptn_url_decode_component(pair + equals + 1, pair_len - equals - 1, &value_decoded_len)
                : ptn_duplicate_string_len("", 0);
            key_decoded = ptn_parse_str_convert_decoded_component(
                runtime,
                key_decoded,
                key_decoded_len,
                converter,
                converter_context,
                &key_decoded_len
            );
            value_decoded = ptn_parse_str_convert_decoded_component(
                runtime,
                value_decoded,
                value_decoded_len,
                converter,
                converter_context,
                &value_decoded_len
            );
            PtnParseStrPath path = ptn_parse_str_parse_key(key_decoded, key_decoded_len);
            ptn_parse_str_assign(
                result.as.array,
                &path,
                0,
                ptn_owned_string_len(value_decoded, value_decoded_len)
            );
            ptn_parse_str_path_free(&path);
            free(key_decoded);
        }
        if (cursor == len) {
            break;
        }
        cursor++;
    }
    return result;
}

static PtnValue ptn_parse_str_to_array_with_separators(
    PtnRuntime *runtime,
    const char *data,
    size_t len,
    const char *separators
) {
    return ptn_parse_str_to_array_with_converter(runtime, data, len, separators, NULL, NULL);
}

static PtnValue ptn_internal_parse_str(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "parse_str", 1, "string", args[0], line);
    if (memchr(string.data, '\0', string.len) != NULL) {
        ptn_string_operand_free(string);
        ptn_throw_exception(runtime, "ValueError", "parse_str(): Argument #1 ($string) must not contain any null bytes");
        return ptn_null();
    }

    PtnValue result = ptn_parse_str_to_array_with_separators(
        runtime,
        string.data,
        string.len,
        ptn_runtime_arg_separator_input(runtime)
    );

    if (args[1].type == PTN_REFERENCE) {
        ptn_reference_assign(runtime, args[1].as.reference, result);
    } else {
        ptn_value_drop(&result);
    }
    ptn_string_operand_free(string);
    return ptn_null();
}
