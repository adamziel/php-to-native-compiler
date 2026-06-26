static int ptn_csv_char_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line,
    char default_value,
    int allow_empty,
    int *enabled
) {
    if (ptn_value_deref(value).type == PTN_NULL) {
        if (enabled != NULL) {
            *enabled = 1;
        }
        return (unsigned char)default_value;
    }
    PtnStringOperand operand = ptn_internal_expect_string_arg(runtime, function_name, position, argument_name, value, line);
    if (operand.len == 0 && allow_empty) {
        ptn_string_operand_free(operand);
        if (enabled != NULL) {
            *enabled = 0;
        }
        return 0;
    }
    if (operand.len != 1) {
        char message[160];
        const char *requirement = allow_empty ? "empty or a single character" : "a single character";
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) must be %s",
            function_name,
            position,
            argument_name,
            requirement
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        ptn_string_operand_free(operand);
        return 0;
    }
    unsigned char byte = (unsigned char)operand.data[0];
    ptn_string_operand_free(operand);
    if (enabled != NULL) {
        *enabled = 1;
    }
    return byte;
}

static int ptn_csv_char_is_escaped(
    const char *data,
    size_t len_before_char,
    int escape_enabled,
    char escape
) {
    if (!escape_enabled || data == NULL) {
        return 0;
    }
    size_t count = 0;
    while (len_before_char > 0 && (unsigned char)data[len_before_char - 1] == (unsigned char)escape) {
        count++;
        len_before_char--;
    }
    return (count % 2) == 1;
}

static size_t ptn_csv_record_payload_len(const char *data, size_t len) {
    if (len > 0 && data[len - 1] == '\n') {
        len--;
        if (len > 0 && data[len - 1] == '\r') {
            len--;
        }
    } else if (len > 0 && data[len - 1] == '\r') {
        len--;
    }
    return len;
}

static int ptn_stream_read_csv_record(
    PtnRuntime *runtime,
    PtnResource *resource,
    int64_t length,
    char delimiter,
    char enclosure,
    int escape_enabled,
    char escape,
    PtnStringBuffer *record,
    size_t line
) {
    ptn_string_buffer_init(record);
    int in_enclosure = 0;
    int at_field_start = 1;
    int length_crossed_in_enclosure = 0;
    while (1) {
        if (length > 0 && record->len >= (size_t)length && !in_enclosure && !length_crossed_in_enclosure) {
            break;
        }
        errno = 0;
        int byte = ptn_stream_get_byte(resource);
        if (byte == EOF) {
            if (ptn_stream_error(resource)) {
                ptn_emit_stream_read_notice(runtime, "fgetcsv", length > 0 ? (size_t)length : 8192, line);
                ptn_stream_clear_error(resource);
                free(record->data);
                ptn_string_buffer_init(record);
                return -1;
            }
            break;
        }
        ptn_string_buffer_append_char(record, (char)(unsigned char)byte);
        if (length > 0 && record->len >= (size_t)length && in_enclosure) {
            length_crossed_in_enclosure = 1;
        }
        if (at_field_start && byte == (unsigned char)enclosure) {
            in_enclosure = 1;
            at_field_start = 0;
            continue;
        }
        if (byte == (unsigned char)enclosure && in_enclosure) {
            if (ptn_csv_char_is_escaped(record->data, record->len - 1, escape_enabled, escape)) {
                at_field_start = 0;
                continue;
            }
            int next = ptn_stream_get_byte(resource);
            if (next == (unsigned char)enclosure) {
                ptn_string_buffer_append_char(record, (char)(unsigned char)next);
                continue;
            }
            if (next != EOF) {
                if (length_crossed_in_enclosure && (next == '\n' || next == '\r')) {
                    ptn_string_buffer_append_char(record, (char)(unsigned char)next);
                    if (next == '\r') {
                        int maybe_lf = ptn_stream_get_byte(resource);
                        if (maybe_lf == '\n') {
                            ptn_string_buffer_append_char(record, (char)(unsigned char)maybe_lf);
                        } else if (maybe_lf != EOF) {
                            ptn_stream_unget_byte(resource, maybe_lf);
                        }
                    }
                    return 1;
                }
                ptn_stream_unget_byte(resource, next);
            }
            in_enclosure = 0;
            if (length_crossed_in_enclosure && length > 0 && record->len == (size_t)length) {
                break;
            }
            continue;
        }
        if (!in_enclosure && byte == '\n') {
            break;
        }
        if (!in_enclosure && byte == (unsigned char)delimiter) {
            at_field_start = 1;
            continue;
        }
        if (!in_enclosure && at_field_start && (byte == ' ' || byte == '\t')) {
            continue;
        }
        at_field_start = 0;
    }
    return record->len == 0 ? 0 : 1;
}

static void ptn_csv_append_null_field(PtnValue result, size_t *field_index) {
    ptn_array_set_entry(
        result.as.array,
        ptn_array_int_key((int64_t)(*field_index)),
        ptn_null()
    );
    (*field_index)++;
}

static void ptn_csv_append_field(PtnValue result, PtnStringBuffer *field, size_t *field_index) {
    char *copy = ptn_duplicate_string_len(field->data == NULL ? "" : field->data, field->len);
    ptn_array_set_entry(
        result.as.array,
        ptn_array_int_key((int64_t)(*field_index)),
        ptn_owned_string_len(copy, field->len)
    );
    (*field_index)++;
    field->len = 0;
    if (field->data != NULL) {
        field->data[0] = '\0';
    }
}

static int ptn_csv_field_padding_before_enclosure(
    const char *data,
    size_t len,
    size_t offset,
    char delimiter,
    char enclosure
) {
    size_t cursor = offset;
    while (
        cursor < len
        && data[cursor] != delimiter
        && (data[cursor] == ' ' || data[cursor] == '\t')
    ) {
        cursor++;
    }
    return cursor < len && data[cursor] == enclosure;
}

static PtnValue ptn_parse_csv_record(
    const char *data,
    size_t len,
    char delimiter,
    char enclosure,
    int escape_enabled,
    char escape
) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (ptn_csv_record_payload_len(data, len) == 0) {
        size_t field_index = 0;
        ptn_csv_append_null_field(result, &field_index);
        return result;
    }

    PtnStringBuffer field;
    ptn_string_buffer_init(&field);
    size_t field_index = 0;
    int in_enclosure = 0;
    int at_field_start = 1;

    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)data[i];
        if (!in_enclosure && (byte == '\n' || byte == '\r')) {
            if (byte == '\r' && i + 1 < len && data[i + 1] == '\n') {
                i++;
            }
            break;
        }
        if (at_field_start &&
            byte != (unsigned char)delimiter &&
            (byte == ' ' || byte == '\t') &&
            ptn_csv_field_padding_before_enclosure(data, len, i, delimiter, enclosure)) {
            continue;
        }
        if (in_enclosure) {
            if (byte == (unsigned char)enclosure) {
                if (ptn_csv_char_is_escaped(field.data, field.len, escape_enabled, escape)) {
                    ptn_string_buffer_append_char(&field, (char)byte);
                    at_field_start = 0;
                    continue;
                }
                if (i + 1 < len && data[i + 1] == enclosure) {
                    ptn_string_buffer_append_char(&field, enclosure);
                    i++;
                    continue;
                }
                in_enclosure = 0;
                at_field_start = 0;
                continue;
            }
            if (escape_enabled && byte == (unsigned char)escape && i + 1 < len) {
                ptn_string_buffer_append_char(&field, (char)byte);
                ptn_string_buffer_append_char(&field, data[++i]);
                at_field_start = 0;
                continue;
            }
            ptn_string_buffer_append_char(&field, (char)byte);
            at_field_start = 0;
            continue;
        }
        if (at_field_start && byte == (unsigned char)enclosure) {
            field.len = 0;
            if (field.data != NULL) {
                field.data[0] = '\0';
            }
            in_enclosure = 1;
            at_field_start = 0;
            continue;
        }
        if (byte == (unsigned char)delimiter) {
            ptn_csv_append_field(result, &field, &field_index);
            at_field_start = 1;
            continue;
        }
        if (at_field_start && (byte == ' ' || byte == '\t')) {
            ptn_string_buffer_append_char(&field, (char)byte);
            continue;
        }
        ptn_string_buffer_append_char(&field, (char)byte);
        at_field_start = 0;
    }
    if (in_enclosure && escape_enabled && escape == '\0') {
        ptn_string_buffer_append_char(&field, '\0');
    }
    ptn_csv_append_field(result, &field, &field_index);
    free(field.data);
    return result;
}

static PtnValue ptn_internal_fgetcsv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnResource *resource = ptn_internal_expect_open_stream_arg(runtime, "fgetcsv", args[0], line);
    if (resource == NULL) {
        return ptn_null();
    }
    int64_t length = 0;
    if (argc >= 2 && ptn_value_deref(args[1]).type != PTN_NULL) {
        length = ptn_internal_expect_integer_arg(runtime, "fgetcsv", 2, "length", args[1], line);
        if (length < 0 || length == LLONG_MAX) {
            char message[128];
            int written = snprintf(
                message,
                sizeof(message),
                "fgetcsv(): Argument #2 ($length) must be between 0 and %lld",
                (long long)LLONG_MAX - 1
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "ValueError", message);
            return ptn_null();
        }
        if (length == LLONG_MAX - 1) {
            fflush(stdout);
            ptn_abort_out_of_memory();
        }
    }
    char delimiter = argc >= 3
        ? (char)ptn_csv_char_arg(runtime, "fgetcsv", 3, "separator", args[2], line, ',', 0, NULL)
        : ',';
    char enclosure = argc >= 4
        ? (char)ptn_csv_char_arg(runtime, "fgetcsv", 4, "enclosure", args[3], line, '"', 0, NULL)
        : '"';
    int escape_enabled = 1;
    char escape = argc >= 5
        ? (char)ptn_csv_char_arg(runtime, "fgetcsv", 5, "escape", args[4], line, '\\', 1, &escape_enabled)
        : '\\';
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    if (argc < 5) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "fgetcsv(): the $escape parameter must be provided as its default value will change",
            line
        );
    }

    PtnStringBuffer record;
    int status = ptn_stream_read_csv_record(
        runtime,
        resource,
        length,
        delimiter,
        enclosure,
        escape_enabled,
        escape,
        &record,
        line
    );
    if (status <= 0) {
        return ptn_bool(0);
    }
    PtnValue parsed = ptn_parse_csv_record(record.data, record.len, delimiter, enclosure, escape_enabled, escape);
    free(record.data);
    return parsed;
}

static PtnValue ptn_internal_str_getcsv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand string =
        ptn_internal_expect_string_arg(runtime, "str_getcsv", 1, "string", args[0], line);
    char delimiter = argc >= 2
        ? (char)ptn_csv_char_arg(runtime, "str_getcsv", 2, "separator", args[1], line, ',', 0, NULL)
        : ',';
    char enclosure = argc >= 3
        ? (char)ptn_csv_char_arg(runtime, "str_getcsv", 3, "enclosure", args[2], line, '"', 0, NULL)
        : '"';
    int escape_enabled = 1;
    char escape = argc >= 4
        ? (char)ptn_csv_char_arg(runtime, "str_getcsv", 4, "escape", args[3], line, '\\', 1, &escape_enabled)
        : '\\';
    if (runtime->exceptions->active_exception != NULL) {
        ptn_string_operand_free(string);
        return ptn_null();
    }

    if (string.len == 0) {
        PtnValue result = ptn_array_from_literal_entries(0, NULL);
        ptn_array_set_entry(result.as.array, ptn_array_int_key(0), ptn_null());
        ptn_string_operand_free(string);
        return result;
    }

    PtnValue parsed = ptn_parse_csv_record(
        string.data,
        string.len,
        delimiter,
        enclosure,
        escape_enabled,
        escape
    );
    ptn_string_operand_free(string);
    return parsed;
}

static int ptn_csv_field_needs_enclosure(
    PtnStringOperand field,
    char delimiter,
    char enclosure,
    int escape_enabled,
    char escape
) {
    for (size_t i = 0; i < field.len; i++) {
        unsigned char byte = (unsigned char)field.data[i];
        if (byte == (unsigned char)delimiter ||
            byte == (unsigned char)enclosure ||
            (escape_enabled && byte == (unsigned char)escape) ||
            byte == '\n' ||
            byte == '\r' ||
            byte == '\t' ||
            byte == ' ') {
            return 1;
        }
    }
    return 0;
}

static void ptn_csv_write_field(
    PtnStringBuffer *buffer,
    PtnStringOperand field,
    char delimiter,
    char enclosure,
    int escape_enabled,
    char escape
) {
    int quote = ptn_csv_field_needs_enclosure(field, delimiter, enclosure, escape_enabled, escape);
    if (quote) {
        ptn_string_buffer_append_char(buffer, enclosure);
    }
    for (size_t i = 0; i < field.len; i++) {
        char byte = field.data[i];
        if (quote && byte == enclosure) {
            if (ptn_csv_char_is_escaped(field.data, i, escape_enabled, escape)) {
                ptn_string_buffer_append_char(buffer, byte);
            } else {
                ptn_string_buffer_append_char(buffer, enclosure);
                ptn_string_buffer_append_char(buffer, enclosure);
            }
        } else {
            ptn_string_buffer_append_char(buffer, byte);
        }
    }
    if (quote) {
        ptn_string_buffer_append_char(buffer, enclosure);
    }
}

static PtnValue ptn_internal_fputcsv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnResource *resource = ptn_internal_expect_open_stream_arg(runtime, "fputcsv", args[0], line);
    if (resource == NULL) {
        return ptn_null();
    }
    PtnValue fields_value = ptn_value_deref(args[1]);
    if (fields_value.type != PTN_ARRAY) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "fputcsv(): Argument #2 ($fields) must be of type array, %s given",
            ptn_offset_container_type_name(fields_value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    char delimiter = argc >= 3
        ? (char)ptn_csv_char_arg(runtime, "fputcsv", 3, "separator", args[2], line, ',', 0, NULL)
        : ',';
    char enclosure = argc >= 4
        ? (char)ptn_csv_char_arg(runtime, "fputcsv", 4, "enclosure", args[3], line, '"', 0, NULL)
        : '"';
    int escape_enabled = 1;
    char escape = argc >= 5
        ? (char)ptn_csv_char_arg(runtime, "fputcsv", 5, "escape", args[4], line, '\\', 1, &escape_enabled)
        : '\\';
    PtnStringOperand eol = argc >= 6
        ? ptn_internal_expect_string_arg(runtime, "fputcsv", 6, "eol", args[5], line)
        : ptn_string_operand_borrowed("\n");
    if (runtime->exceptions->active_exception != NULL) {
        if (argc >= 6) {
            ptn_string_operand_free(eol);
        }
        return ptn_null();
    }
    if (argc < 5) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "fputcsv(): the $escape parameter must be provided as its default value will change",
            line
        );
    }

    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    for (size_t i = 0; i < fields_value.as.array->len; i++) {
        if (i > 0) {
            ptn_string_buffer_append_char(&output, delimiter);
        }
        PtnValue field_value = fields_value.as.array->entries[i].value;
        if (ptn_value_deref(field_value).type == PTN_ARRAY) {
            ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
        }
        PtnStringOperand field = ptn_value_to_string_operand_with_runtime(runtime, field_value, line);
        if (runtime->exceptions->active_exception != NULL) {
            ptn_string_operand_free(field);
            if (argc >= 6) {
                ptn_string_operand_free(eol);
            }
            free(output.data);
            return ptn_null();
        }
        ptn_csv_write_field(&output, field, delimiter, enclosure, escape_enabled, escape);
        ptn_string_operand_free(field);
    }
    ptn_string_buffer_append_len(&output, eol.data, eol.len);
    if (argc >= 6) {
        ptn_string_operand_free(eol);
    }

    size_t written = ptn_stream_write_filtered(runtime, "fputcsv", resource, output.data, output.len, line);
    if (written != output.len && ptn_stream_error(resource)) {
        ptn_stream_clear_error(resource);
        free(output.data);
        return ptn_bool(0);
    }
    if (written > (size_t)INT64_MAX) {
        free(output.data);
        ptn_abort_out_of_memory();
    }
    free(output.data);
    return ptn_int((int64_t)written);
}
