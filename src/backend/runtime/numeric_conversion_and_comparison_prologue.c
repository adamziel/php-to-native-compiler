static PTN_UNUSED void ptn_runtime_init_function_frame(PtnRuntime *runtime, PtnRuntime *caller_runtime) {
    ptn_symbols_init(&runtime->symbols);
    runtime->global_symbols = caller_runtime->global_symbols;
    ptn_symbols_init(&runtime->owned_constants);
    runtime->constants = caller_runtime->constants;
    ptn_symbols_init(&runtime->owned_class_constants);
    runtime->class_constants = caller_runtime->class_constants;
    ptn_symbols_init(&runtime->owned_static_properties);
    runtime->static_properties = caller_runtime->static_properties;
    ptn_symbols_init(&runtime->owned_static_property_read_visibility);
    runtime->static_property_read_visibility = caller_runtime->static_property_read_visibility;
    ptn_symbols_init(&runtime->owned_static_property_set_visibility);
    runtime->static_property_set_visibility = caller_runtime->static_property_set_visibility;
    ptn_diagnostics_init(&runtime->diagnostics, NULL);
    runtime->diagnostics.error_reporting = caller_runtime->diagnostics.error_reporting;
    runtime->diagnostics.suppressed = caller_runtime->diagnostics.suppressed;
    runtime->owned_exceptions.active_exception = NULL;
    runtime->owned_exceptions.try_frame = NULL;
    runtime->exceptions = caller_runtime->exceptions;
    runtime->owned_call_frame.argc = 0;
    runtime->owned_call_frame.args = NULL;
    runtime->owned_call_frame.parameter_count = 0;
    runtime->owned_call_frame.parameter_names = NULL;
    runtime->call_frame = NULL;
    runtime->owned_trace_frame.function_name = NULL;
    runtime->owned_trace_frame.file = NULL;
    runtime->owned_trace_frame.line = 0;
    runtime->owned_trace_frame.argc = 0;
    runtime->owned_trace_frame.args = NULL;
    runtime->owned_trace_frame.previous = NULL;
    runtime->trace_frame = caller_runtime->trace_frame;
    runtime->lifecycle_root = caller_runtime->lifecycle_root == NULL
        ? caller_runtime
        : caller_runtime->lifecycle_root;
    runtime->live_objects = NULL;
    runtime->live_objects_len = 0;
    runtime->live_objects_capacity = 0;
    runtime->next_object_id = 0;
    runtime->free_object_ids = NULL;
    runtime->free_object_ids_len = 0;
    runtime->free_object_ids_capacity = 0;
    runtime->method_dispatch = caller_runtime->method_dispatch;
    runtime->declared_method_exists = caller_runtime->declared_method_exists;
    runtime->class_scope_allows = caller_runtime->class_scope_allows;
    runtime->declared_class_is_readonly = caller_runtime->declared_class_is_readonly;
    runtime->magic_property_read = caller_runtime->magic_property_read;
    runtime->magic_property_isset = caller_runtime->magic_property_isset;
    runtime->declared_user_functions = caller_runtime->declared_user_functions;
    runtime->magic_property_get = caller_runtime->magic_property_get;
    runtime->magic_property_get_exists = caller_runtime->magic_property_get_exists;
    runtime->magic_property_set = caller_runtime->magic_property_set;
    runtime->magic_property_unset = caller_runtime->magic_property_unset;
    runtime->magic_debug_info = caller_runtime->magic_debug_info;
    runtime->in_magic_property_dispatch = caller_runtime->in_magic_property_dispatch;
    runtime->source_path = caller_runtime->source_path;
    runtime->current_function_name = NULL;
    runtime->current_class_name = NULL;
    runtime->current_called_class_name = NULL;
    runtime->called_class_name_override = NULL;
    runtime->has_current_receiver = 0;
    runtime->current_receiver = ptn_null();
    runtime->by_ref_argument_function_name_override =
        caller_runtime->by_ref_argument_function_name_override;
    runtime->include_path = NULL;
    runtime->memory_limit = NULL;
    runtime->max_memory_limit = NULL;
    runtime->exception_string_param_max_len = caller_runtime->exception_string_param_max_len;
    runtime->strict_types = caller_runtime->strict_types;
    runtime->initial_zend_assertions = caller_runtime->initial_zend_assertions;
    runtime->zend_assertions = caller_runtime->zend_assertions;
    runtime->assert_exception = caller_runtime->assert_exception;
    runtime->call_site_line = 0;
    runtime->warn_by_ref_argument_mismatch = caller_runtime->warn_by_ref_argument_mismatch;
    runtime->throw_argument_count_errors = caller_runtime->throw_argument_count_errors;
}

static PTN_UNUSED void ptn_runtime_set_call_frame(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t parameter_count,
    const char *const *parameter_names
) {
    runtime->owned_call_frame.argc = argc;
    runtime->owned_call_frame.args = args;
    runtime->owned_call_frame.parameter_count = parameter_count;
    runtime->owned_call_frame.parameter_names = parameter_names;
    runtime->call_frame = &runtime->owned_call_frame;
    runtime->owned_trace_frame.function_name = runtime->current_function_name;
    runtime->owned_trace_frame.file = runtime->source_path;
    runtime->owned_trace_frame.line = runtime->call_site_line;
    runtime->owned_trace_frame.argc = argc;
    runtime->owned_trace_frame.args = args;
    runtime->owned_trace_frame.previous = runtime->trace_frame;
    runtime->trace_frame = &runtime->owned_trace_frame;
}

static PTN_UNUSED void ptn_runtime_push_trace_frame(
    PtnRuntime *runtime,
    PtnTraceFrame *frame,
    const char *function_name,
    const char *file,
    size_t line,
    size_t argc,
    const PtnValue *args
) {
    frame->function_name = function_name;
    frame->file = file;
    frame->line = line;
    frame->argc = argc;
    frame->args = args;
    frame->previous = runtime->trace_frame;
    runtime->trace_frame = frame;
}

static PTN_UNUSED void ptn_runtime_pop_trace_frame(PtnRuntime *runtime, PtnTraceFrame *frame) {
    if (runtime->trace_frame == frame) {
        runtime->trace_frame = frame->previous;
    }
}

static void ptn_runtime_free(PtnRuntime *runtime) {
    if (runtime->lifecycle_root == runtime) {
        ptn_runtime_run_object_destructors(runtime);
    }
    ptn_symbols_free(&runtime->owned_static_property_set_visibility);
    ptn_symbols_free(&runtime->owned_static_property_read_visibility);
    ptn_symbols_free(&runtime->owned_static_properties);
    ptn_symbols_free(&runtime->owned_class_constants);
    ptn_symbols_free(&runtime->owned_constants);
    ptn_symbols_free(&runtime->symbols);
    if (runtime->lifecycle_root == runtime) {
        free(runtime->include_path);
        runtime->include_path = NULL;
        free(runtime->memory_limit);
        runtime->memory_limit = NULL;
        free(runtime->max_memory_limit);
        runtime->max_memory_limit = NULL;
        free(runtime->live_objects);
        runtime->live_objects = NULL;
        runtime->live_objects_len = 0;
        runtime->live_objects_capacity = 0;
        free(runtime->free_object_ids);
        runtime->free_object_ids = NULL;
        runtime->free_object_ids_len = 0;
        runtime->free_object_ids_capacity = 0;
    }
}

static PTN_UNUSED PtnLookupResult ptn_object_property_lookup_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED void ptn_string_buffer_init(PtnStringBuffer *buffer);
static PTN_UNUSED void ptn_string_buffer_append(PtnStringBuffer *buffer, const char *value);
static PTN_UNUSED void ptn_string_buffer_append_len(
    PtnStringBuffer *buffer,
    const char *value,
    size_t len
);
static PTN_UNUSED void ptn_string_buffer_append_char(PtnStringBuffer *buffer, char value);
static PTN_UNUSED void ptn_string_buffer_append_format(
    PtnStringBuffer *buffer,
    const char *format,
    ...
);

static PTN_UNUSED PtnSymbolTable *ptn_runtime_global_symbol_table(PtnRuntime *runtime) {
    return runtime->global_symbols == NULL ? &runtime->symbols : runtime->global_symbols;
}

static PTN_UNUSED PtnValue ptn_runtime_globals_snapshot(PtnRuntime *runtime) {
    PtnSymbolTable *globals = ptn_runtime_global_symbol_table(runtime);
    PtnArrayLiteralEntry *entries = NULL;
    if (globals->len != 0) {
        entries = malloc(globals->len * sizeof(PtnArrayLiteralEntry));
        if (entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }

    size_t entry_count = 0;
    for (size_t i = 0; i < globals->len; i++) {
        if (strcmp(globals->items[i].name, "GLOBALS") == 0) {
            continue;
        }
        entries[entry_count].has_key = 1;
        entries[entry_count].key = ptn_string(globals->items[i].name);
        entries[entry_count].value = ptn_value_deref(globals->items[i].value);
        entry_count++;
    }

    PtnValue snapshot = ptn_array_from_literal_entries(entry_count, entries);
    free(entries);
    return snapshot;
}

static PTN_UNUSED void ptn_runtime_write_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnValue current;
    if (ptn_symbols_get(&runtime->symbols, name, &current) && current.type == PTN_REFERENCE) {
        ptn_reference_assign(current.as.reference, value);
        return;
    }
    ptn_symbols_set(&runtime->symbols, name, ptn_value_deref(value));
}

static PTN_UNUSED void ptn_runtime_write_global_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnSymbolTable *globals = ptn_runtime_global_symbol_table(runtime);
    PtnValue current;
    if (ptn_symbols_get(globals, name, &current) && current.type == PTN_REFERENCE) {
        ptn_reference_assign(current.as.reference, value);
        return;
    }
    ptn_symbols_set(globals, name, ptn_value_deref(value));
}

static PTN_UNUSED void ptn_runtime_bind_variable_reference(PtnRuntime *runtime, const char *name, PtnValue reference) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    ptn_symbols_bind_reference(&runtime->symbols, name, reference);
}

static PTN_UNUSED void ptn_runtime_bind_global_variable(PtnRuntime *runtime, const char *name) {
    PtnValue reference = ptn_symbols_reference_for_variable(ptn_runtime_global_symbol_table(runtime), name);
    ptn_symbols_bind_reference(&runtime->symbols, name, reference);
    ptn_value_destroy(&reference);
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_variable(PtnRuntime *runtime, const char *name) {
    return ptn_symbols_reference_for_variable(&runtime->symbols, name);
}

static PTN_UNUSED PtnValue *ptn_runtime_global_variable_slot(PtnRuntime *runtime, const char *name) {
    return ptn_symbols_value_slot(ptn_runtime_global_symbol_table(runtime), name);
}

static PTN_UNUSED PtnValue *ptn_runtime_global_variable_slot_for_write(PtnRuntime *runtime, const char *name) {
    return &ptn_symbols_slot_for_write(ptn_runtime_global_symbol_table(runtime), name)->value;
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_global_variable_quiet(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_symbols_get(ptn_runtime_global_symbol_table(runtime), name, &value)) {
        return ptn_lookup_found(ptn_value_deref(value));
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED void ptn_runtime_unset_global_variable(PtnRuntime *runtime, const char *name) {
    ptn_symbols_unset(ptn_runtime_global_symbol_table(runtime), name);
}

static PTN_UNUSED void ptn_abort_by_reference_argument_error(
    const char *function_name,
    size_t position,
    const char *parameter_name
) {
    const int has_parameter_name = parameter_name != NULL && parameter_name[0] != '\0';
    fprintf(
        stderr,
        has_parameter_name
            ? "Fatal error: %s(): Argument #%zu ($%s) cannot be passed by reference\n"
            : "Fatal error: %s(): Argument #%zu cannot be passed by reference\n",
        function_name,
        position,
        has_parameter_name ? parameter_name : ""
    );
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception_at(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
);

static PTN_UNUSED void ptn_throw_by_reference_argument_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    size_t line
) {
    char message[256];
    const int has_parameter_name = parameter_name != NULL && parameter_name[0] != '\0';
    int written;
    if (has_parameter_name) {
        written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) could not be passed by reference",
            function_name,
            position,
            parameter_name
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu could not be passed by reference",
            function_name,
            position
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED PtnValue ptn_runtime_by_reference_argument_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    size_t line
) {
    ptn_throw_by_reference_argument_error(runtime, function_name, position, parameter_name, line);
    return ptn_null();
}

static PTN_UNUSED const char *ptn_by_reference_argument_function_name(
    PtnRuntime *runtime,
    const char *fallback
) {
    if (runtime != NULL && runtime->by_ref_argument_function_name_override != NULL) {
        return runtime->by_ref_argument_function_name_override;
    }
    return fallback;
}

static PTN_UNUSED void ptn_emit_by_reference_argument_warning(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    size_t line
) {
    int needed = snprintf(
        NULL,
        0,
        "%s(): Argument #%zu ($%s) must be passed by reference, value given",
        function_name,
        position,
        parameter_name
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
        "%s(): Argument #%zu ($%s) must be passed by reference, value given",
        function_name,
        position,
        parameter_name
    );
    if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        fputc('\n', stdout);
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
    free(message);
}

static PTN_UNUSED void ptn_abort_by_reference_return_error(void) {
    fputs("Fatal error: by-reference return did not produce a reference\n", stderr);
    exit(255);
}

static PTN_UNUSED PtnValue ptn_reference_source_or_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    if (value.type == PTN_REFERENCE) {
        return ptn_value_clone(value);
    }
    ptn_emit_only_variable_references_returned_by_reference_notice(&runtime->diagnostics, line);
    return ptn_reference_value(ptn_reference_new_owned(ptn_value_clone(value)));
}

static PTN_UNUSED PtnValue ptn_by_ref_argument_source_or_temporary(PtnRuntime *runtime, PtnValue value, size_t line) {
    if (value.type == PTN_REFERENCE) {
        return ptn_value_clone(value);
    }
    ptn_emit_only_variables_passed_by_reference_notice(&runtime->diagnostics, line);
    return ptn_reference_value(ptn_reference_new_owned(ptn_value_clone(ptn_value_deref(value))));
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    if (strcmp(name, "GLOBALS") == 0) {
        return ptn_runtime_globals_snapshot(runtime);
    }
    PtnValue value;
    if (ptn_symbols_get(&runtime->symbols, name, &value)) {
        return ptn_value_deref(value);
    }
    if (strcmp(name, "this") == 0) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Using $this when not in object context",
            path,
            line
        );
        return ptn_null();
    }
    ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable_for_array_mutation(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnValue *slot = ptn_symbols_get_slot(&runtime->symbols, name);
    if (slot == NULL) {
        if (strcmp(name, "this") == 0) {
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Using $this when not in object context",
                path,
                line
            );
            return ptn_null();
        }
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
        return ptn_null();
    }
    if (slot->type == PTN_ARRAY) {
        (void)ptn_value_detach_array(slot);
    }
    return ptn_value_borrow(*slot);
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_variable_quiet(PtnRuntime *runtime, const char *name) {
    if (strcmp(name, "GLOBALS") == 0) {
        return ptn_lookup_found(ptn_runtime_globals_snapshot(runtime));
    }
    PtnValue value;
    if (ptn_symbols_get(&runtime->symbols, name, &value)) {
        return ptn_lookup_found(ptn_value_deref(value));
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED int ptn_runtime_variable_is_set(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    return ptn_symbols_get(&runtime->symbols, name, &value) && ptn_value_deref(value).type != PTN_NULL;
}

static PTN_UNUSED int ptn_runtime_variable_is_empty(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    return !ptn_symbols_get(&runtime->symbols, name, &value) || !ptn_is_truthy(ptn_value_deref(value));
}

static PTN_UNUSED void ptn_runtime_unset_variable(PtnRuntime *runtime, const char *name) {
    ptn_symbols_unset(&runtime->symbols, name);
}

static PTN_UNUSED PtnValue ptn_trace_value_snapshot_depth(PtnValue value, size_t depth) {
    value = ptn_value_deref(value);
    if (value.type != PTN_ARRAY || depth > 64) {
        return ptn_value_clone(value);
    }

    PtnValue snapshot = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < value.as.array->len; i++) {
        ptn_array_set_entry(
            snapshot.as.array,
            ptn_array_key_clone(value.as.array->entries[i].key),
            ptn_trace_value_snapshot_depth(value.as.array->entries[i].value, depth + 1)
        );
    }
    snapshot.as.array->next_auto_key = value.as.array->next_auto_key;
    snapshot.as.array->current_index = value.as.array->current_index <= snapshot.as.array->len
        ? value.as.array->current_index
        : snapshot.as.array->len;
    return snapshot;
}

static PTN_UNUSED PtnValue ptn_trace_value_snapshot(PtnValue value) {
    return ptn_trace_value_snapshot_depth(value, 0);
}

static PTN_UNUSED PtnValue ptn_trace_frame_args_array(PtnTraceFrame *frame) {
    PtnValue args = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < frame->argc; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            args.as.array,
            ptn_array_int_key((int64_t)i),
            ptn_trace_value_snapshot(frame->args[i])
        );
    }
    return args;
}

static PTN_UNUSED PtnValue ptn_trace_frame_array(PtnTraceFrame *frame) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (frame->file != NULL && frame->line != 0) {
        if (frame->line > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("file"),
            ptn_owned_string(ptn_duplicate_string(frame->file))
        );
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("line"),
            ptn_int((int64_t)frame->line)
        );
    }
    if (frame->function_name != NULL) {
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("function"),
            ptn_owned_string(ptn_duplicate_string(frame->function_name))
        );
    }
    ptn_array_set_entry(
        result.as.array,
        ptn_array_string_key("args"),
        ptn_trace_frame_args_array(frame)
    );
    return result;
}

static PTN_UNUSED size_t ptn_runtime_exception_string_param_max_len(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return 15;
    }
    return root->exception_string_param_max_len;
}

static PTN_UNUSED void ptn_runtime_set_exception_string_param_max_len(
    PtnRuntime *runtime,
    size_t value
) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return;
    }
    root->exception_string_param_max_len = value;
}

static PtnValue *ptn_trace_array_string_slot(PtnValue frame, const char *key) {
    frame = ptn_value_deref(frame);
    if (frame.type != PTN_ARRAY || frame.as.array == NULL) {
        return NULL;
    }
    size_t key_len = strlen(key);
    for (size_t i = 0; i < frame.as.array->len; i++) {
        PtnArrayEntry *entry = &frame.as.array->entries[i];
        if (
            entry->key.type == PTN_ARRAY_KEY_STRING &&
            entry->key.string_len == key_len &&
            memcmp(entry->key.as.string, key, key_len) == 0
        ) {
            return &entry->value;
        }
    }
    return NULL;
}

static void ptn_trace_append_quoted_string(
    PtnStringBuffer *buffer,
    const unsigned char *data,
    size_t len,
    size_t max_len
) {
    ptn_string_buffer_append_char(buffer, '\'');
    size_t display_len = len;
    int append_ellipsis = 0;
    if (len > max_len) {
        display_len = max_len;
        append_ellipsis = 1;
    }
    for (size_t i = 0; i < display_len; i++) {
        unsigned char byte = data[i];
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
            case '\\':
                ptn_string_buffer_append(buffer, "\\\\");
                break;
            case '\'':
                ptn_string_buffer_append(buffer, "\\'");
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
    if (append_ellipsis) {
        ptn_string_buffer_append(buffer, "...");
    }
    ptn_string_buffer_append_char(buffer, '\'');
}

static void ptn_trace_append_arg(PtnStringBuffer *buffer, PtnValue value, size_t max_string_len) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            ptn_string_buffer_append(buffer, "NULL");
            break;
        case PTN_BOOL:
            ptn_string_buffer_append(buffer, value.as.boolean ? "true" : "false");
            break;
        case PTN_INT:
            ptn_string_buffer_append_format(buffer, "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[128];
            ptn_format_scalar_float(value.as.floating, formatted, sizeof(formatted));
            ptn_string_buffer_append(buffer, formatted);
            break;
        }
        case PTN_STRING:
            ptn_trace_append_quoted_string(
                buffer,
                value.as.string.data,
                value.as.string.len,
                max_string_len
            );
            break;
        case PTN_ARRAY:
            ptn_string_buffer_append(buffer, "Array");
            break;
        case PTN_OBJECT:
            ptn_string_buffer_append_format(buffer, "Object(%s)", value.as.object->class_name);
            break;
        case PTN_CLOSURE:
            ptn_string_buffer_append(buffer, "Object(Closure)");
            break;
        case PTN_EXCEPTION:
            ptn_string_buffer_append_format(buffer, "Object(%s)", value.as.exception->class_name);
            break;
        case PTN_RESOURCE:
            ptn_string_buffer_append_format(
                buffer,
                "Resource id #%lld",
                (long long)value.as.resource->id
            );
            break;
        case PTN_REFERENCE:
            ptn_string_buffer_append(buffer, "NULL");
            break;
    }
}

static void ptn_exception_append_display_function(
    PtnStringBuffer *buffer,
    const char *function_name
) {
    const char *constructor_separator = strstr(function_name, "::__construct");
    if (constructor_separator != NULL && constructor_separator[13] == '\0') {
        ptn_string_buffer_append_len(
            buffer,
            function_name,
            (size_t)(constructor_separator - function_name)
        );
        ptn_string_buffer_append(buffer, "->__construct");
        return;
    }
    ptn_string_buffer_append(buffer, function_name);
}

static void ptn_exception_append_trace_frame(
    PtnStringBuffer *buffer,
    size_t index,
    PtnValue frame,
    size_t max_string_len
) {
    ptn_string_buffer_append_format(buffer, "#%zu ", index);
    frame = ptn_value_deref(frame);
    if (frame.type != PTN_ARRAY || frame.as.array == NULL) {
        ptn_string_buffer_append(buffer, "{main}");
        return;
    }

    PtnValue *file_slot = ptn_trace_array_string_slot(frame, "file");
    PtnValue *line_slot = ptn_trace_array_string_slot(frame, "line");
    PtnValue file_value = file_slot == NULL ? ptn_null() : ptn_value_deref(*file_slot);
    PtnValue line_value = line_slot == NULL ? ptn_null() : ptn_value_deref(*line_slot);
    if (file_value.type == PTN_STRING && line_value.type == PTN_INT) {
        ptn_string_buffer_append_len(
            buffer,
            (const char *)file_value.as.string.data,
            file_value.as.string.len
        );
        ptn_string_buffer_append_format(buffer, "(%lld): ", (long long)line_value.as.integer);
    }

    PtnValue *function_slot = ptn_trace_array_string_slot(frame, "function");
    PtnValue function_value = function_slot == NULL ? ptn_null() : ptn_value_deref(*function_slot);
    if (function_value.type == PTN_STRING) {
        char *function_name = ptn_duplicate_string_len(
            (const char *)function_value.as.string.data,
            function_value.as.string.len
        );
        ptn_exception_append_display_function(buffer, function_name);
        free(function_name);
    }
    ptn_string_buffer_append_char(buffer, '(');
    PtnValue *args_slot = ptn_trace_array_string_slot(frame, "args");
    PtnValue args_value = args_slot == NULL ? ptn_null() : ptn_value_deref(*args_slot);
    if (args_value.type == PTN_ARRAY && args_value.as.array != NULL) {
        for (size_t i = 0; i < args_value.as.array->len; i++) {
            if (i != 0) {
                ptn_string_buffer_append(buffer, ", ");
            }
            ptn_trace_append_arg(buffer, args_value.as.array->entries[i].value, max_string_len);
        }
    }
    ptn_string_buffer_append_char(buffer, ')');
}

static PTN_UNUSED PtnStringOperand ptn_exception_trace_as_string_operand(
    PtnRuntime *runtime,
    PtnException *exception
) {
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    size_t max_string_len = ptn_runtime_exception_string_param_max_len(runtime);
    PtnValue trace = exception == NULL ? ptn_null() : ptn_value_deref(exception->trace);
    size_t index = 0;
    if (trace.type == PTN_ARRAY && trace.as.array != NULL) {
        for (size_t i = 0; i < trace.as.array->len; i++) {
            if (index != 0) {
                ptn_string_buffer_append_char(&buffer, '\n');
            }
            ptn_exception_append_trace_frame(
                &buffer,
                index,
                trace.as.array->entries[i].value,
                max_string_len
            );
            index++;
        }
    }
    if (index != 0) {
        ptn_string_buffer_append_char(&buffer, '\n');
    }
    ptn_string_buffer_append_format(&buffer, "#%zu {main}", index);
    return (PtnStringOperand) { buffer.data, buffer.data, buffer.len };
}

static PTN_UNUSED PtnStringOperand ptn_exception_to_string_operand(
    PtnRuntime *runtime,
    PtnException *exception
) {
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    if (exception == NULL) {
        return (PtnStringOperand) { buffer.data, buffer.data, buffer.len };
    }
    ptn_string_buffer_append(&buffer, exception->class_name);
    if (exception->message_len != 0) {
        ptn_string_buffer_append(&buffer, ": ");
        ptn_string_buffer_append_len(&buffer, exception->message, exception->message_len);
    }
    ptn_string_buffer_append(&buffer, " in ");
    ptn_string_buffer_append(&buffer, exception->path == NULL ? "ptn" : exception->path);
    ptn_string_buffer_append_format(&buffer, ":%zu\nStack trace:\n", exception->line);
    PtnStringOperand trace = ptn_exception_trace_as_string_operand(runtime, exception);
    ptn_string_buffer_append_len(&buffer, trace.data, trace.len);
    free(trace.owned);
    return (PtnStringOperand) { buffer.data, buffer.data, buffer.len };
}

static PTN_UNUSED PtnValue ptn_exception_capture_trace(PtnRuntime *runtime) {
    PtnValue trace = ptn_array_from_literal_entries(0, NULL);
    size_t index = 0;
    for (PtnTraceFrame *frame = runtime != NULL ? runtime->trace_frame : NULL;
         frame != NULL;
         frame = frame->previous) {
        if (index > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            trace.as.array,
            ptn_array_int_key((int64_t)index),
            ptn_trace_frame_array(frame)
        );
        index++;
    }
    return trace;
}

static PTN_UNUSED PtnException *ptn_exception_new_owned(
    PtnRuntime *runtime,
    const char *class_name,
    char *message,
    size_t message_len,
    int64_t code,
    PtnValue previous,
    int64_t severity,
    const char *path,
    size_t line
) {
    PtnException *exception = malloc(sizeof(PtnException));
    if (exception == NULL) {
        ptn_abort_out_of_memory();
    }
    exception->refcount = 1;
    exception->object_id = ptn_runtime_alloc_object_id(runtime);
    exception->lifecycle_runtime = ptn_runtime_root(runtime);
    exception->class_name = class_name;
    exception->message = message;
    exception->message_len = message_len;
    exception->code = code;
    exception->path = path;
    exception->line = line;
    exception->trace = ptn_exception_capture_trace(runtime);
    exception->previous = ptn_value_clone_deref(previous);
    exception->severity = severity;
    return exception;
}

static PTN_UNUSED PtnException *ptn_exception_new_owned_cstr(
    PtnRuntime *runtime,
    const char *class_name,
    char *message,
    const char *path,
    size_t line
) {
    return ptn_exception_new_owned(
        runtime,
        class_name,
        message,
        strlen(message),
        0,
        ptn_null(),
        PTN_E_ERROR,
        path,
        line
    );
}

static PTN_UNUSED PtnException *ptn_exception_new(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
) {
    return ptn_exception_new_owned(
        runtime,
        class_name,
        ptn_duplicate_string(message),
        strlen(message),
        0,
        ptn_null(),
        PTN_E_ERROR,
        path,
        line
    );
}

static PTN_UNUSED int ptn_exception_name_equal(const char *left, const char *right);

static PTN_UNUSED const char *ptn_builtin_exception_class_name(const char *class_name) {
    if (class_name[0] == '\\') {
        class_name++;
    }
    if (ptn_exception_name_equal(class_name, "Exception")) {
        return "Exception";
    }
    if (ptn_exception_name_equal(class_name, "ErrorException")) {
        return "ErrorException";
    }
    if (ptn_exception_name_equal(class_name, "ReflectionException")) {
        return "ReflectionException";
    }
    if (ptn_exception_name_equal(class_name, "RuntimeException")) {
        return "RuntimeException";
    }
    if (ptn_exception_name_equal(class_name, "Error")) {
        return "Error";
    }
    if (ptn_exception_name_equal(class_name, "UnhandledMatchError")) {
        return "UnhandledMatchError";
    }
    if (ptn_exception_name_equal(class_name, "TypeError")) {
        return "TypeError";
    }
    if (ptn_exception_name_equal(class_name, "ArgumentCountError")) {
        return "ArgumentCountError";
    }
    if (ptn_exception_name_equal(class_name, "ValueError")) {
        return "ValueError";
    }
    if (ptn_exception_name_equal(class_name, "ArithmeticError")) {
        return "ArithmeticError";
    }
    if (ptn_exception_name_equal(class_name, "DivisionByZeroError")) {
        return "DivisionByZeroError";
    }
    if (ptn_exception_name_equal(class_name, "AssertionError")) {
        return "AssertionError";
    }
    if (ptn_exception_name_equal(class_name, "ParseError")) {
        return "ParseError";
    }
    if (ptn_exception_name_equal(class_name, "UnhandledMatchError")) {
        return "UnhandledMatchError";
    }
    return NULL;
}

static PTN_UNUSED int ptn_exception_name_equal(const char *left, const char *right) {
    while (*left != '\0' && *right != '\0') {
        int left_byte = tolower((unsigned char)*left);
        int right_byte = tolower((unsigned char)*right);
        if (left_byte != right_byte) {
            return 0;
        }
        left++;
        right++;
    }
    return *left == '\0' && *right == '\0';
}

static PTN_UNUSED int ptn_exception_type_matches_name(const char *class_name, const char *type_name) {
    if (type_name[0] == '\\') {
        type_name++;
    }
    if (ptn_exception_name_equal(class_name, type_name)) {
        return 1;
    }
    if (ptn_exception_name_equal(type_name, "Error")) {
        return ptn_exception_name_equal(class_name, "Error") ||
            ptn_exception_name_equal(class_name, "TypeError") ||
            ptn_exception_name_equal(class_name, "ArgumentCountError") ||
            ptn_exception_name_equal(class_name, "ValueError") ||
            ptn_exception_name_equal(class_name, "ArithmeticError") ||
            ptn_exception_name_equal(class_name, "DivisionByZeroError") ||
            ptn_exception_name_equal(class_name, "UnhandledMatchError") ||
            ptn_exception_name_equal(class_name, "AssertionError") ||
            ptn_exception_name_equal(class_name, "ParseError") ||
            ptn_exception_name_equal(class_name, "UnhandledMatchError");
    }
    if (ptn_exception_name_equal(type_name, "TypeError")) {
        return ptn_exception_name_equal(class_name, "ArgumentCountError");
    }
    if (ptn_exception_name_equal(type_name, "ArithmeticError")) {
        return ptn_exception_name_equal(class_name, "DivisionByZeroError");
    }
    if (ptn_exception_name_equal(type_name, "Exception")) {
        return ptn_exception_name_equal(class_name, "ErrorException") ||
            ptn_exception_name_equal(class_name, "ReflectionException") ||
            ptn_exception_name_equal(class_name, "RuntimeException");
    }
    if (ptn_exception_name_equal(type_name, "Throwable")) {
        return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_try_frame_push(PtnRuntime *runtime, PtnTryFrame *frame) {
    frame->previous = runtime->exceptions->try_frame;
    runtime->exceptions->try_frame = frame;
}

static PTN_UNUSED void ptn_try_frame_pop(PtnRuntime *runtime, PtnTryFrame *frame) {
    if (runtime->exceptions->try_frame == frame) {
        runtime->exceptions->try_frame = frame->previous;
    }
}

static void ptn_emit_uncaught_trace_arg(FILE *stream, PtnRuntime *runtime, PtnValue value) {
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    ptn_trace_append_arg(
        &buffer,
        value,
        ptn_runtime_exception_string_param_max_len(runtime)
    );
    fwrite(buffer.data, 1, buffer.len, stream);
    free(buffer.data);
}

static int ptn_emit_uncaught_internal_trace(PtnRuntime *runtime) {
    if (runtime == NULL || runtime->trace_frame == NULL) {
        return 0;
    }

    size_t index = 0;
    for (PtnTraceFrame *frame = runtime->trace_frame; frame != NULL; frame = frame->previous) {
        if (frame->function_name == NULL) {
            continue;
        }
        const char *file = frame->file;
        size_t line = frame->line;
        if ((file == NULL || line == 0) && runtime->source_path != NULL && runtime->call_site_line != 0) {
            file = runtime->source_path;
            line = runtime->call_site_line;
        }
        fprintf(stderr, "#%zu ", index);
        if (file != NULL && line != 0) {
            fprintf(stderr, "%s(%zu): ", file, line);
        }
        const char *constructor_separator = strstr(frame->function_name, "::__construct");
        if (constructor_separator != NULL && constructor_separator[13] == '\0') {
            fwrite(
                frame->function_name,
                1,
                (size_t)(constructor_separator - frame->function_name),
                stderr
            );
            fputs("->__construct", stderr);
        } else {
            fputs(frame->function_name, stderr);
        }
        fputc('(', stderr);
        for (size_t i = 0; i < frame->argc; i++) {
            if (i != 0) {
                fputs(", ", stderr);
            }
            ptn_emit_uncaught_trace_arg(stderr, runtime, frame->args[i]);
        }
        fputs(")\n", stderr);
        index++;
    }
    if (index == 0) {
        return 0;
    }
    fprintf(stderr, "#%zu {main}\n", index);
    return 1;
}

static PTN_UNUSED void ptn_emit_uncaught_exception_chain_entry(
    PtnException *exception,
    int *first
) {
    if (exception->previous.type == PTN_EXCEPTION) {
        ptn_emit_uncaught_exception_chain_entry(exception->previous.as.exception, first);
    }
    const char *display_path = exception->path != NULL ? exception->path : "[no active file]";
    size_t display_line = exception->line;
    if (*first) {
        fputc('\n', stderr);
        fprintf(
            stderr,
            "Fatal error: Uncaught %s: %s in %s:%zu\n",
            exception->class_name,
            exception->message,
            display_path,
            display_line
        );
        *first = 0;
    } else {
        fprintf(
            stderr,
            "\nNext %s: %s in %s:%zu\n",
            exception->class_name,
            exception->message,
            display_path,
            display_line
        );
    }
    fputs("Stack trace:\n#0 {main}\n", stderr);
}

static PTN_UNUSED void ptn_emit_uncaught_exception(PtnRuntime *runtime, PtnException *exception) {
    fflush(stdout);
    if (!runtime->diagnostics.display_errors) {
        return;
    }
    if (exception->previous.type == PTN_EXCEPTION) {
        int first = 1;
        ptn_emit_uncaught_exception_chain_entry(exception, &first);
        const char *display_path = exception->path != NULL ? exception->path : "[no active file]";
        fprintf(stderr, "  thrown in %s on line %zu\n", display_path, exception->line);
        return;
    }
    const char *display_path = exception->path;
    size_t display_line = exception->line;
    PtnTraceFrame *frame = runtime != NULL ? runtime->trace_frame : NULL;
    if (
        (display_path == NULL || display_line == 0) &&
        runtime != NULL &&
        runtime->current_function_name == NULL &&
        frame != NULL &&
        frame->file != NULL &&
        frame->line != 0
    ) {
        display_path = frame->file;
        display_line = frame->line;
    }
    if (display_path == NULL || display_line == 0) {
        fputs("Fatal error: ", stderr);
        fputs(exception->message, stderr);
        fputc('\n', stderr);
        return;
    }

    fputc('\n', stderr);
    fprintf(stderr, "Fatal error: Uncaught %s", exception->class_name);
    if (exception->message_len != 0) {
        fputs(": ", stderr);
        fwrite(exception->message, 1, exception->message_len, stderr);
    }
    fprintf(stderr, " in %s:%zu\n", display_path, display_line);
    fputs("Stack trace:\n", stderr);
    if (ptn_emit_uncaught_internal_trace(runtime)) {
        /* Trace emitted from the active internal call frame. */
    } else if (runtime->current_function_name != NULL && runtime->call_site_line != 0) {
        fprintf(
            stderr,
            "#0 %s(%zu): %s()\n#1 {main}\n",
            runtime->source_path != NULL ? runtime->source_path : exception->path,
            runtime->call_site_line,
            runtime->current_function_name
        );
    } else {
        fputs("#0 {main}\n", stderr);
    }
    fprintf(stderr, "  thrown in %s on line %zu\n", display_path, display_line);
}

static PTN_UNUSED void ptn_throw_exception_at(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
) {
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = ptn_exception_new(runtime, class_name, message, path, line);
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception(PtnRuntime *runtime, const char *class_name, const char *message) {
    ptn_throw_exception_at(runtime, class_name, message, NULL, 0);
}

static PTN_UNUSED void ptn_throw_exception_owned_message(
    PtnRuntime *runtime,
    const char *class_name,
    char *message
) {
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception =
        ptn_exception_new_owned_cstr(runtime, class_name, message, NULL, 0);
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception_owned_message_at(
    PtnRuntime *runtime,
    const char *class_name,
    char *message,
    const char *path,
    size_t line
) {
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception =
        ptn_exception_new_owned_cstr(runtime, class_name, message, path, line);
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    exit(255);
}

static PTN_UNUSED const char *ptn_exception_constructor_declaring_class(
    PtnRuntime *runtime,
    const char *class_name
) {
    if (
        ptn_exception_name_equal(class_name, "ErrorException") ||
        (
            runtime != NULL &&
            runtime->class_scope_allows != NULL &&
            runtime->class_scope_allows(class_name, "ErrorException")
        )
    ) {
        return "ErrorException";
    }
    if (
        ptn_exception_name_equal(class_name, "Error") ||
        (
            runtime != NULL &&
            runtime->class_scope_allows != NULL &&
            runtime->class_scope_allows(class_name, "Error")
        )
    ) {
        return "Error";
    }
    return "Exception";
}

static PTN_UNUSED size_t ptn_exception_constructor_max_args(const char *declaring_class) {
    return ptn_exception_name_equal(declaring_class, "ErrorException") ? 6 : 3;
}

static const char *ptn_exception_constructor_given_type(PtnValue value) {
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
        case PTN_ARRAY:
            return "array";
        case PTN_OBJECT:
            return value.as.object->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_RESOURCE:
            return "resource";
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED PtnStringOperand ptn_exception_constructor_message(
    PtnRuntime *runtime,
    const char *declaring_class,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    if (argc == 0) {
        char *message = ptn_duplicate_string("");
        return (PtnStringOperand) { message, message, 0 };
    }

    PtnTraceFrame trace_frame;
    char trace_name[64];
    int written = snprintf(trace_name, sizeof(trace_name), "%s::__construct", declaring_class);
    if (written < 0 || (size_t)written >= sizeof(trace_name)) {
        ptn_abort_out_of_memory();
    }
    ptn_runtime_push_trace_frame(
        runtime,
        &trace_frame,
        trace_name,
        runtime != NULL ? runtime->source_path : NULL,
        line,
        argc,
        args
    );

    PtnValue value = ptn_value_deref(args[0]);
    if (
        value.type == PTN_ARRAY ||
        value.type == PTN_OBJECT ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION ||
        value.type == PTN_RESOURCE
    ) {
        const char *given = ptn_exception_constructor_given_type(value);
        int needed = snprintf(
            NULL,
            0,
            "%s::__construct(): Argument #1 ($message) must be of type string, %s given",
            declaring_class,
            given
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
            "%s::__construct(): Argument #1 ($message) must be of type string, %s given",
            declaring_class,
            given
        );
        ptn_throw_exception_owned_message_at(
            runtime,
            "TypeError",
            message,
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
    }

    char *message;
    size_t message_len;
    if (value.type == PTN_STRING) {
        message_len = value.as.string.len;
        message = ptn_duplicate_string_len((const char *)value.as.string.data, message_len);
    } else {
        message = ptn_value_to_string(value);
        message_len = strlen(message);
    }
    ptn_runtime_pop_trace_frame(runtime, &trace_frame);
    return (PtnStringOperand) { message, message, message_len };
}

static PTN_UNUSED int ptn_object_is_declared_throwable(PtnRuntime *runtime, PtnObject *object) {
    return runtime->class_scope_allows != NULL &&
        (runtime->class_scope_allows(object->class_name, "Exception") ||
            runtime->class_scope_allows(object->class_name, "Error"));
}

static PTN_UNUSED PtnStringOperand ptn_object_exception_message(
    PtnRuntime *runtime,
    PtnValue object,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(object);
    PtnLookupResult lookup = ptn_object_property_lookup_quiet(
        runtime,
        resolved,
        "message",
        resolved.as.object->class_name,
        line
    );
    if (!lookup.exists) {
        char *message = ptn_duplicate_string("");
        return (PtnStringOperand) { message, message, 0 };
    }
    PtnValue message_value = ptn_value_deref(lookup.value);
    char *message;
    size_t message_len;
    if (message_value.type == PTN_STRING) {
        message_len = message_value.as.string.len;
        message = ptn_duplicate_string_len((const char *)message_value.as.string.data, message_len);
    } else {
        message = ptn_value_to_string(message_value);
        message_len = strlen(message);
    }
    ptn_value_destroy(&lookup.value);
    return (PtnStringOperand) { message, message, message_len };
}

static PTN_UNUSED PtnLookupResult ptn_throwable_object_property(
    PtnRuntime *runtime,
    PtnValue object,
    const char *property,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(object);
    if (resolved.type != PTN_OBJECT) {
        return ptn_lookup_missing();
    }
    return ptn_object_property_lookup_quiet(
        runtime,
        resolved,
        property,
        resolved.as.object->class_name,
        line
    );
}

static PTN_UNUSED char *ptn_throwable_message_string(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_duplicate_string_len(
            receiver.as.exception->message,
            receiver.as.exception->message_len
        );
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, "message", line);
    if (!lookup.exists) {
        return ptn_duplicate_string("");
    }
    char *message = ptn_value_to_string(lookup.value);
    ptn_value_destroy(&lookup.value);
    return message;
}

static PTN_UNUSED int64_t ptn_throwable_int_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    int64_t fallback,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        if (ptn_exception_name_equal(property, "code")) {
            return receiver.as.exception->code;
        }
        if (ptn_exception_name_equal(property, "line")) {
            return receiver.as.exception->line > (size_t)INT64_MAX
                ? INT64_MAX
                : (int64_t)receiver.as.exception->line;
        }
        if (ptn_exception_name_equal(property, "severity")) {
            return receiver.as.exception->severity;
        }
        return fallback;
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, property, line);
    if (!lookup.exists) {
        return fallback;
    }
    PtnValue value = ptn_value_deref(lookup.value);
    int64_t result = fallback;
    if (value.type == PTN_INT) {
        result = value.as.integer;
    } else if (value.type == PTN_BOOL) {
        result = value.as.boolean ? 1 : 0;
    } else if (value.type == PTN_FLOAT) {
        result = (int64_t)value.as.floating;
    }
    ptn_value_destroy(&lookup.value);
    return result;
}

static PTN_UNUSED PtnValue ptn_throwable_file_value(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_owned_string(ptn_duplicate_string(receiver.as.exception->path != NULL ? receiver.as.exception->path : ""));
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, "file", line);
    if (!lookup.exists) {
        return ptn_owned_string(ptn_duplicate_string(runtime->source_path != NULL ? runtime->source_path : ""));
    }
    char *file = ptn_value_to_string(lookup.value);
    ptn_value_destroy(&lookup.value);
    return ptn_owned_string(file);
}

static PTN_UNUSED PtnValue ptn_throwable_previous_value(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_value_clone_deref(receiver.as.exception->previous);
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, "previous", line);
    if (!lookup.exists) {
        return ptn_null();
    }
    PtnValue previous = ptn_value_clone_deref(lookup.value);
    ptn_value_destroy(&lookup.value);
    return previous;
}

static PTN_UNUSED PtnValue ptn_throwable_trace_value(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_value_clone(receiver.as.exception->trace);
    }
    (void)runtime;
    (void)line;
    return ptn_array_from_literal_entries(0, NULL);
}

static PTN_UNUSED PtnValue ptn_throwable_trace_string(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        PtnStringOperand trace = ptn_exception_trace_as_string_operand(
            runtime,
            receiver.as.exception
        );
        return ptn_owned_string_len(trace.owned, trace.len);
    }
    return ptn_owned_string(ptn_duplicate_string("#0 {main}"));
}

static PTN_UNUSED PtnValue ptn_throwable_to_string(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        PtnStringOperand text = ptn_exception_to_string_operand(
            runtime,
            receiver.as.exception
        );
        return ptn_owned_string_len(text.owned, text.len);
    }
    const char *class_name = receiver.type == PTN_OBJECT ? receiver.as.object->class_name : "Exception";
    char *message = ptn_throwable_message_string(runtime, receiver, line);
    PtnValue file_value = ptn_throwable_file_value(runtime, receiver, line);
    char *file = ptn_value_to_string(file_value);
    ptn_value_destroy(&file_value);
    int64_t throwable_line = ptn_throwable_int_property(runtime, receiver, "line", 0, line);
    int needed = snprintf(
        NULL,
        0,
        "%s: %s in %s:%lld\nStack trace:\n#0 {main}",
        class_name,
        message,
        file,
        (long long)throwable_line
    );
    if (needed < 0) {
        free(file);
        free(message);
        ptn_abort_out_of_memory();
    }
    char *result = malloc((size_t)needed + 1);
    if (result == NULL) {
        free(file);
        free(message);
        ptn_abort_out_of_memory();
    }
    snprintf(
        result,
        (size_t)needed + 1,
        "%s: %s in %s:%lld\nStack trace:\n#0 {main}",
        class_name,
        message,
        file,
        (long long)throwable_line
    );
    free(file);
    free(message);
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_throw_value(
    PtnRuntime *runtime,
    PtnValue value,
    const char *path,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_OBJECT && ptn_object_is_declared_throwable(runtime, resolved.as.object)) {
        PtnStringOperand message = ptn_object_exception_message(runtime, resolved, line);
        int64_t code = ptn_throwable_int_property(runtime, resolved, "code", 0, line);
        int64_t severity = ptn_throwable_int_property(runtime, resolved, "severity", PTN_E_ERROR, line);
        PtnValue previous = ptn_throwable_previous_value(runtime, resolved, line);
        PtnValue file_value = ptn_throwable_file_value(runtime, resolved, line);
        char *exception_path = ptn_value_to_string(file_value);
        ptn_value_destroy(&file_value);
        int64_t stored_line = ptn_throwable_int_property(runtime, resolved, "line", (int64_t)line, line);
        ptn_exception_free(runtime->exceptions->active_exception);
        runtime->exceptions->active_exception = ptn_exception_new_owned(
            runtime,
            resolved.as.object->class_name,
            message.owned,
            message.len,
            code,
            previous,
            severity,
            exception_path,
            stored_line < 0 ? line : (size_t)stored_line
        );
        ptn_value_destroy(&previous);
        if (runtime->exceptions->try_frame != NULL) {
            longjmp(runtime->exceptions->try_frame->jump, 1);
        }
        ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
        exit(255);
        return ptn_null();
    }
    if (resolved.type != PTN_EXCEPTION) {
        ptn_throw_exception_at(runtime, "Error", "Can only throw objects", path, line);
        return ptn_null();
    }
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = resolved.as.exception;
    ptn_exception_retain(runtime->exceptions->active_exception);
    if (runtime->exceptions->active_exception->path == NULL) {
        runtime->exceptions->active_exception->path = path;
        runtime->exceptions->active_exception->line = line;
    }
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    exit(255);
    return ptn_null();
}

static PTN_UNUSED void ptn_throw_user_argument_count_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t expected,
    size_t passed,
    int exactly
) {
    const char *mode = exactly ? "exactly" : "at least";
    int needed = snprintf(
        NULL,
        0,
        "Too few arguments to function %s(), %zu passed and %s %zu expected",
        function_name,
        passed,
        mode,
        expected
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
        "Too few arguments to function %s(), %zu passed and %s %zu expected",
        function_name,
        passed,
        mode,
        expected
    );
    ptn_throw_exception_owned_message(runtime, "ArgumentCountError", message);
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_table(PtnRuntime *runtime) {
    return runtime->static_properties == NULL ? &runtime->owned_static_properties : runtime->static_properties;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_read_visibility_table(PtnRuntime *runtime) {
    return runtime->static_property_read_visibility == NULL
        ? &runtime->owned_static_property_read_visibility
        : runtime->static_property_read_visibility;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_set_visibility_table(PtnRuntime *runtime) {
    return runtime->static_property_set_visibility == NULL
        ? &runtime->owned_static_property_set_visibility
        : runtime->static_property_set_visibility;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_class_constant_table(PtnRuntime *runtime) {
    return runtime->class_constants == NULL ? &runtime->owned_class_constants : runtime->class_constants;
}

static PTN_UNUSED int ptn_property_class_names_equal(const char *left, const char *right) {
    if (left == NULL || right == NULL) {
        return 0;
    }
    while (*left != '\0' && *right != '\0') {
        if (tolower((unsigned char)*left) != tolower((unsigned char)*right)) {
            return 0;
        }
        left++;
        right++;
    }
    return *left == '\0' && *right == '\0';
}

static PTN_UNUSED int ptn_property_visibility_allows(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *access_scope
) {
    if (visibility == PTN_PROPERTY_PUBLIC) {
        return 1;
    }
    if (access_scope == NULL || declaring_class == NULL) {
        return 0;
    }
    if (visibility == PTN_PROPERTY_PRIVATE) {
        return ptn_property_class_names_equal(access_scope, declaring_class);
    }
    if (runtime->class_scope_allows != NULL) {
        return runtime->class_scope_allows(access_scope, declaring_class);
    }
    return ptn_property_class_names_equal(access_scope, declaring_class);
}

static PTN_UNUSED const char *ptn_property_visibility_name(PtnPropertyVisibility visibility) {
    if (visibility == PTN_PROPERTY_PRIVATE) {
        return "private";
    }
    if (visibility == PTN_PROPERTY_PROTECTED) {
        return "protected";
    }
    return "public";
}

static PTN_UNUSED void ptn_throw_property_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot access %s property %s::$%s",
        ptn_property_visibility_name(visibility),
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_property_set_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope
) {
    char message[320];
    const char *scope = access_scope == NULL ? "global scope" : access_scope;
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot modify %s(set) property %s::$%s from %s",
            ptn_property_visibility_name(visibility),
            declaring_class,
            property,
            scope
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot modify %s(set) property %s::$%s from scope %s",
            ptn_property_visibility_name(visibility),
            declaring_class,
            property,
            scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_property_indirect_set_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope
) {
    char message[320];
    const char *scope = access_scope == NULL ? "global scope" : access_scope;
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot indirectly modify %s(set) property %s::$%s from %s",
            ptn_property_visibility_name(visibility),
            declaring_class,
            property,
            scope
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot indirectly modify %s(set) property %s::$%s from scope %s",
            ptn_property_visibility_name(visibility),
            declaring_class,
            property,
            scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_property_unset_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope,
    int asymmetric_set_visibility
) {
    char message[320];
    const char *scope = access_scope == NULL ? "global scope" : access_scope;
    const char *set_suffix = asymmetric_set_visibility ? "(set)" : "";
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot unset %s%s property %s::$%s from %s",
            ptn_property_visibility_name(visibility),
            set_suffix,
            declaring_class,
            property,
            scope
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot unset %s%s property %s::$%s from scope %s",
            ptn_property_visibility_name(visibility),
            set_suffix,
            declaring_class,
            property,
            scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED char *ptn_static_property_key(const char *class_name, const char *property) {
    size_t class_len = strlen(class_name);
    size_t property_len = strlen(property);
    size_t len = class_len + property_len + 4;
    char *key = malloc(len);
    if (key == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(key, class_name, class_len);
    memcpy(key + class_len, "::$", 3);
    memcpy(key + class_len + 3, property, property_len);
    key[len - 1] = '\0';
    return key;
}

static PTN_UNUSED char *ptn_runtime_resolve_static_property_key(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char **declaring_class_out
) {
    const char *lookup_class_name = class_name;
    while (lookup_class_name != NULL) {
        char *key = ptn_static_property_key(lookup_class_name, property);
        if (ptn_symbols_value_slot(ptn_runtime_static_property_table(runtime), key) != NULL) {
            if (declaring_class_out != NULL) {
                *declaring_class_out = lookup_class_name;
            }
            return key;
        }
        free(key);
        lookup_class_name = ptn_declared_class_parent_name(lookup_class_name);
    }
    if (declaring_class_out != NULL) {
        *declaring_class_out = NULL;
    }
    return NULL;
}

static PTN_UNUSED char *ptn_class_constant_key(const char *class_name, const char *constant) {
    size_t class_len = strlen(class_name);
    size_t constant_len = strlen(constant);
    size_t len = class_len + constant_len + 3;
    char *key = malloc(len);
    if (key == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(key, class_name, class_len);
    memcpy(key + class_len, "::", 2);
    memcpy(key + class_len + 2, constant, constant_len);
    key[len - 1] = '\0';
    return key;
}

static PTN_UNUSED char ptn_ascii_lower_char(char ch) {
    return (ch >= 'A' && ch <= 'Z') ? (char)(ch - 'A' + 'a') : ch;
}

static PTN_UNUSED const char *ptn_symbol_name_without_leading_slash(const char *name) {
    return name[0] == '\\' ? name + 1 : name;
}

static PTN_UNUSED int ptn_ascii_case_equal_span_to_string(
    const char *left,
    size_t left_len,
    const char *right
) {
    size_t right_len = strlen(right);
    if (left_len != right_len) {
        return 0;
    }
    for (size_t i = 0; i < left_len; i++) {
        if (ptn_ascii_lower_char(left[i]) != ptn_ascii_lower_char(right[i])) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED int ptn_builtin_class_constant_value_span(
    const char *class_name,
    size_t class_len,
    const char *constant,
    PtnValue *out
) {
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ArrayObject")) {
        if (strcmp(constant, "STD_PROP_LIST") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "ARRAY_AS_PROPS") == 0) {
            *out = ptn_int(2);
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_builtin_class_constant_value(
    const char *class_name,
    const char *constant,
    PtnValue *out
) {
    return ptn_builtin_class_constant_value_span(class_name, strlen(class_name), constant, out);
}

static PTN_UNUSED PtnValue ptn_runtime_undeclared_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Access to undeclared static property %s::$%s",
        class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
    return ptn_null();
}

static PTN_UNUSED void ptn_runtime_define_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    PtnValue value
) {
    char *key = ptn_static_property_key(class_name, property);
    ptn_symbols_set(ptn_runtime_static_property_table(runtime), key, ptn_value_deref(value));
    ptn_symbols_set(
        ptn_runtime_static_property_read_visibility_table(runtime),
        key,
        ptn_int((int64_t)read_visibility)
    );
    ptn_symbols_set(
        ptn_runtime_static_property_set_visibility_table(runtime),
        key,
        ptn_int((int64_t)set_visibility)
    );
    free(key);
}

static PTN_UNUSED void ptn_runtime_define_class_constant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    PtnValue value
) {
    char *key = ptn_class_constant_key(class_name, constant);
    ptn_symbols_set(ptn_runtime_class_constant_table(runtime), key, ptn_value_deref(value));
    free(key);
}

static PTN_UNUSED PtnValue ptn_runtime_undefined_class_constant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Undefined constant %s::%s",
        class_name,
        constant
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    size_t line
) {
    (void)line;
    const char *lookup_class_name = class_name;
    while (lookup_class_name != NULL) {
        char *key = ptn_class_constant_key(lookup_class_name, constant);
        PtnValue value;
        if (ptn_symbols_get(ptn_runtime_class_constant_table(runtime), key, &value)) {
            free(key);
            return ptn_value_clone_deref(value);
        }
        free(key);
        lookup_class_name = ptn_declared_class_parent_name(lookup_class_name);
    }
    PtnValue builtin_value;
    if (ptn_builtin_class_constant_value(class_name, constant, &builtin_value)) {
        return builtin_value;
    }
    return ptn_runtime_undefined_class_constant(runtime, class_name, constant);
}

static PTN_UNUSED const char *ptn_dynamic_class_name_fetch_type_name(PtnValue value) {
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
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            return "object";
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED PtnValue ptn_runtime_fetch_dynamic_class_name(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    const char *class_name = NULL;
    if (receiver.type == PTN_OBJECT) {
        class_name = receiver.as.object->class_name;
    } else if (receiver.type == PTN_EXCEPTION) {
        class_name = receiver.as.exception->class_name;
    } else if (receiver.type == PTN_CLOSURE) {
        class_name = "Closure";
    } else if (receiver.type == PTN_STRING) {
        return ptn_value_clone_deref(receiver);
    }
    if (class_name != NULL) {
        return ptn_owned_string(ptn_duplicate_string(class_name));
    }

    if (receiver.type == PTN_NULL) {
        ptn_throw_exception(runtime, "TypeError", "Cannot use \"::class\" on null");
        return ptn_null();
    }

    const char *type_name = ptn_dynamic_class_name_fetch_type_name(receiver);
    int needed = snprintf(NULL, 0, "Cannot use \"::class\" on value of type %s", type_name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(message, (size_t)needed + 1, "Cannot use \"::class\" on value of type %s", type_name);
    ptn_throw_exception_owned_message(runtime, "TypeError", message);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_read_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line
) {
    (void)line;
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    PtnValue value;
    if (key != NULL && ptn_symbols_get(ptn_runtime_static_property_table(runtime), key, &value)) {
        PtnValue visibility_value;
        PtnPropertyVisibility visibility = PTN_PROPERTY_PUBLIC;
        if (
            ptn_symbols_get(
                ptn_runtime_static_property_read_visibility_table(runtime),
                key,
                &visibility_value
            ) &&
            ptn_value_deref(visibility_value).type == PTN_INT
        ) {
            visibility = (PtnPropertyVisibility)ptn_value_deref(visibility_value).as.integer;
        }
        if (!ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
            free(key);
            ptn_throw_property_visibility_error(runtime, visibility, declaring_class, property);
            return ptn_null();
        }
        free(key);
        return ptn_value_clone_deref(value);
    }
    free(key);
    return ptn_runtime_undeclared_static_property(runtime, class_name, property);
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_static_property_quiet(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line
) {
    (void)line;
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    PtnValue value;
    if (key != NULL && ptn_symbols_get(ptn_runtime_static_property_table(runtime), key, &value)) {
        PtnValue visibility_value;
        PtnPropertyVisibility visibility = PTN_PROPERTY_PUBLIC;
        if (
            ptn_symbols_get(
                ptn_runtime_static_property_read_visibility_table(runtime),
                key,
                &visibility_value
            ) &&
            ptn_value_deref(visibility_value).type == PTN_INT
        ) {
            visibility = (PtnPropertyVisibility)ptn_value_deref(visibility_value).as.integer;
        }
        if (!ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
            free(key);
            return ptn_lookup_missing();
        }
        free(key);
        return ptn_lookup_found(ptn_value_clone_deref(value));
    }
    free(key);
    return ptn_lookup_missing();
}

static PTN_UNUSED PtnValue ptn_runtime_write_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    (void)line;
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        return ptn_runtime_undeclared_static_property(runtime, class_name, property);
    }
    PtnValue read_visibility_value;
    PtnValue set_visibility_value;
    PtnPropertyVisibility read_visibility = PTN_PROPERTY_PUBLIC;
    PtnPropertyVisibility set_visibility = PTN_PROPERTY_PUBLIC;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_read_visibility_table(runtime),
            key,
            &read_visibility_value
        ) &&
        ptn_value_deref(read_visibility_value).type == PTN_INT
    ) {
        read_visibility = (PtnPropertyVisibility)ptn_value_deref(read_visibility_value).as.integer;
    }
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_set_visibility_table(runtime),
            key,
            &set_visibility_value
        ) &&
        ptn_value_deref(set_visibility_value).type == PTN_INT
    ) {
        set_visibility = (PtnPropertyVisibility)ptn_value_deref(set_visibility_value).as.integer;
    }
    if (!ptn_property_visibility_allows(runtime, set_visibility, declaring_class, access_scope)) {
        free(key);
        if (set_visibility != read_visibility) {
            ptn_throw_property_set_visibility_error(
                runtime,
                set_visibility,
                declaring_class,
                property,
                access_scope
            );
        } else {
            ptn_throw_property_visibility_error(runtime, set_visibility, declaring_class, property);
        }
        return ptn_null();
    }
    PtnValue result = ptn_value_clone_deref(value);
    ptn_symbols_set(ptn_runtime_static_property_table(runtime), key, result);
    free(key);
    return result;
}

static PTN_UNUSED int ptn_exception_matches(PtnRuntime *runtime, const char *type_name) {
    if (runtime->exceptions->active_exception == NULL) {
        return 0;
    }
    const char *class_name = runtime->exceptions->active_exception->class_name;
    if (ptn_exception_type_matches_name(class_name, type_name)) {
        return 1;
    }
    if (type_name[0] == '\\') {
        type_name++;
    }
    return runtime->class_scope_allows != NULL &&
        runtime->class_scope_allows(class_name, type_name);
}

static PTN_UNUSED PtnValue ptn_current_exception_value(PtnRuntime *runtime) {
    if (runtime->exceptions->active_exception == NULL) {
        return ptn_null();
    }
    return ptn_exception_borrow(runtime->exceptions->active_exception);
}

static PTN_UNUSED void ptn_clear_exception(PtnRuntime *runtime) {
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = NULL;
}

static PTN_UNUSED void ptn_rethrow_exception(PtnRuntime *runtime) {
    PtnException *exception = runtime->exceptions->active_exception;
    if (exception == NULL) {
        return;
    }
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, exception);
    exit(255);
}

static PTN_UNUSED PtnValue ptn_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    (void)args;
    (void)line;
    receiver = ptn_value_deref(receiver);
    int is_throwable_receiver = receiver.type == PTN_EXCEPTION ||
        (receiver.type == PTN_OBJECT && ptn_object_is_declared_throwable(runtime, receiver.as.object));
    if (is_throwable_receiver && (
        ptn_exception_name_equal(name, "getMessage") ||
        ptn_exception_name_equal(name, "getCode") ||
        ptn_exception_name_equal(name, "getFile") ||
        ptn_exception_name_equal(name, "getLine") ||
        ptn_exception_name_equal(name, "getPrevious") ||
        ptn_exception_name_equal(name, "getTrace") ||
        ptn_exception_name_equal(name, "getTraceAsString") ||
        ptn_exception_name_equal(name, "getSeverity") ||
        ptn_exception_name_equal(name, "__toString")
    )) {
        if (argc != 0) {
            ptn_throw_exception(
                runtime,
                "ArgumentCountError",
                "Too many arguments to exception method"
            );
        }
        if (ptn_exception_name_equal(name, "getMessage")) {
            if (receiver.type == PTN_EXCEPTION) {
                return ptn_owned_string_len(
                    ptn_duplicate_string_len(
                        receiver.as.exception->message,
                        receiver.as.exception->message_len
                    ),
                    receiver.as.exception->message_len
                );
            }
            PtnStringOperand message = ptn_object_exception_message(runtime, receiver, line);
            return ptn_owned_string_len(message.owned, message.len);
        }
        if (ptn_exception_name_equal(name, "getCode")) {
            return ptn_int(ptn_throwable_int_property(runtime, receiver, "code", 0, line));
        }
        if (ptn_exception_name_equal(name, "getFile")) {
            return ptn_throwable_file_value(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getLine")) {
            return ptn_int(ptn_throwable_int_property(runtime, receiver, "line", 0, line));
        }
        if (ptn_exception_name_equal(name, "getPrevious")) {
            return ptn_throwable_previous_value(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getTrace")) {
            return ptn_throwable_trace_value(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getTraceAsString")) {
            return ptn_throwable_trace_string(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getSeverity")) {
            return ptn_int(ptn_throwable_int_property(runtime, receiver, "severity", PTN_E_ERROR, line));
        }
        if (ptn_exception_name_equal(name, "__toString")) {
            return ptn_throwable_to_string(runtime, receiver, line);
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_class(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_class_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_function(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_function_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_method(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_method_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_array_iterator(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_array_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_array_object(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_array_object_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_iterator_iterator(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_iterator_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
#endif
    ptn_throw_exception(runtime, "Error", "Call to undefined method");
    return ptn_null();
}

static PTN_UNUSED void ptn_runtime_define_constant(PtnRuntime *runtime, const char *name, PtnValue value) {
    ptn_symbols_set(runtime->constants, name, value);
}

static PTN_UNUSED PtnNumber ptn_number_int(int64_t integer) {
    PtnNumber number;
    number.type = PTN_NUMBER_INT;
    number.integer = integer;
    number.floating = (double)integer;
    return number;
}

static PTN_UNUSED PtnNumber ptn_number_float(double floating) {
    PtnNumber number;
    number.type = PTN_NUMBER_FLOAT;
    number.integer = 0;
    number.floating = floating;
    return number;
}

static PTN_UNUSED int ptn_contains_float_marker(const char *start, const char *end) {
    for (const char *cursor = start; cursor < end; cursor++) {
        if (*cursor == '.' || *cursor == 'e' || *cursor == 'E') {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_string_has_embedded_nul(PtnString string) {
    return memchr(string.data, '\0', string.len) != NULL;
}

static PTN_UNUSED PtnNumber ptn_string_to_number(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return ptn_number_int(0);
    }

    char *int_end = NULL;
    errno = 0;
    long long integer = strtoll(start, &int_end, 10);
    int int_errno = errno;

    char *float_end = NULL;
    errno = 0;
    double floating = strtod(start, &float_end);
    if (float_end == start) {
        return ptn_number_int(0);
    }

    if (int_end == float_end && int_errno != ERANGE && !ptn_contains_float_marker(start, int_end)) {
        return ptn_number_int((int64_t)integer);
    }
    return ptn_number_float(floating);
}

static PTN_UNUSED PtnNumber ptn_to_number(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return ptn_number_int(0);
        case PTN_BOOL:
            return ptn_number_int(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_number_int(value.as.integer);
        case PTN_FLOAT:
            return ptn_number_float(value.as.floating);
        case PTN_STRING:
            if (ptn_string_has_embedded_nul(value.as.string)) {
                return ptn_number_int(0);
            }
            return ptn_string_to_number((const char *)value.as.string.data);
        case PTN_ARRAY:
            return ptn_number_int(value.as.array->len == 0 ? 0 : 1);
        case PTN_OBJECT:
        case PTN_CLOSURE:
            return ptn_number_int(1);
        case PTN_EXCEPTION:
            return ptn_number_int(1);
        case PTN_RESOURCE:
            return ptn_number_int(value.as.resource->id);
        case PTN_REFERENCE:
            return ptn_number_int(0);
    }
    return ptn_number_int(0);
}

static PTN_UNUSED int ptn_fast_integer_value(PtnValue value, int64_t *integer) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            *integer = 0;
            return 1;
        case PTN_BOOL:
            *integer = value.as.boolean ? 1 : 0;
            return 1;
        case PTN_INT:
            *integer = value.as.integer;
            return 1;
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_fast_scalar_double(PtnValue value, double *number) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        *number = (double)integer;
        return 1;
    }
    if (value.type == PTN_FLOAT) {
        *number = value.as.floating;
        return 1;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_negate(PtnValue value) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        if (integer == INT64_MIN) {
            return ptn_float(-(double)integer);
        }
        return ptn_int(-integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(-value.as.floating);
    }

    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(-number.floating);
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(-(double)number.integer);
    }
    return ptn_int(-number.integer);
}

static PTN_UNUSED PtnValue ptn_positive(PtnValue value) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_int(integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(value.as.floating);
    }

    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED int ptn_is_truthy(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return 0;
        case PTN_BOOL:
            return value.as.boolean != 0;
        case PTN_INT:
            return value.as.integer != 0;
        case PTN_FLOAT:
            return value.as.floating != 0.0;
        case PTN_STRING:
            return value.as.string.len != 0 &&
                !(value.as.string.len == 1 && value.as.string.data[0] == '0');
        case PTN_ARRAY:
            return value.as.array->len != 0;
        case PTN_OBJECT:
        case PTN_CLOSURE:
            return 1;
        case PTN_EXCEPTION:
            return 1;
        case PTN_RESOURCE:
            return 1;
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_not(PtnValue value) {
    return ptn_bool(!ptn_is_truthy(value));
}

static PTN_UNUSED PtnValue ptn_cast_int(PtnValue value) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_int(integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_int((int64_t)value.as.floating);
    }

    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_int((int64_t)number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED PtnValue ptn_cast_float(PtnValue value) {
    value = ptn_value_deref(value);
    double fast_number = 0.0;
    if (ptn_fast_scalar_double(value, &fast_number)) {
        return ptn_float(fast_number);
    }

    PtnNumber number = ptn_to_number(value);
    return ptn_float(number.floating);
}

static PTN_UNUSED void ptn_abort_arithmetic_error(const char *message) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(1);
}

static PTN_UNUSED void ptn_abort_type_error_at(const char *message, const char *path, size_t line) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputs(" in ", stderr);
    fputs(path, stderr);
    fputs(" on line ", stderr);
    fprintf(stderr, "%zu", line);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED void ptn_abort_control_error(const char *message, const char *path, size_t line) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputs(" in ", stderr);
    fputs(path, stderr);
    fputs(" on line ", stderr);
    fprintf(stderr, "%zu", line);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED int ptn_is_number_type(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_INT || value.type == PTN_FLOAT;
}

static PTN_UNUSED int ptn_string_may_be_numeric(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }
    if (isdigit((unsigned char)*start) || *start == '+' || *start == '-' || *start == '.') {
        return 1;
    }
    return *start == 'i' || *start == 'I' || *start == 'n' || *start == 'N';
}

static PTN_UNUSED int ptn_is_numeric_string(const char *string, double *number) {
    if (!ptn_string_may_be_numeric(string)) {
        return 0;
    }

    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    char *end = NULL;
    double parsed = strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    if (*end != '\0') {
        return 0;
    }
    *number = parsed;
    return 1;
}

static PTN_UNUSED int ptn_comparison_numeric_value(PtnValue value, double *number) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_INT:
            *number = (double)value.as.integer;
            return 1;
        case PTN_FLOAT:
            *number = value.as.floating;
            return 1;
        case PTN_STRING:
            if (ptn_string_has_embedded_nul(value.as.string)) {
                return 0;
            }
            return ptn_is_numeric_string((const char *)value.as.string.data, number);
        case PTN_RESOURCE:
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

enum {
    PTN_COMPARE_LESS = -1,
    PTN_COMPARE_EQUAL = 0,
    PTN_COMPARE_GREATER = 1,
    PTN_COMPARE_UNORDERED = 2
};

static PTN_UNUSED int ptn_compare_numbers(double left, double right) {
    if (isnan(left) || isnan(right)) {
        return PTN_COMPARE_UNORDERED;
    }
    if (left < right) {
        return PTN_COMPARE_LESS;
    }
    if (left > right) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_integers(int64_t left, int64_t right) {
    if (left < right) {
        return PTN_COMPARE_LESS;
    }
    if (left > right) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_strings(const char *left, const char *right) {
    int compared = strcmp(left, right);
    return compared < 0 ? -1 : (compared > 0 ? 1 : 0);
}

static PTN_UNUSED int ptn_compare_string_bytes(
    const unsigned char *left,
    size_t left_len,
    const unsigned char *right,
    size_t right_len
) {
    size_t shared_len = left_len < right_len ? left_len : right_len;
    int compared = shared_len == 0 ? 0 : memcmp(left, right, shared_len);
    if (compared < 0) {
        return PTN_COMPARE_LESS;
    }
    if (compared > 0) {
        return PTN_COMPARE_GREATER;
    }
    if (left_len < right_len) {
        return PTN_COMPARE_LESS;
    }
    if (left_len > right_len) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_value_strings(PtnString left, PtnString right) {
    return ptn_compare_string_bytes(left.data, left.len, right.data, right.len);
}

static PTN_UNUSED void ptn_number_value_to_string(PtnValue value, char *buffer, size_t buffer_len) {
    if (value.type == PTN_INT) {
