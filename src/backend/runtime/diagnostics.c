
static PTN_UNUSED void ptn_symbols_ensure_index(PtnSymbolTable *symbols, size_t expected_entries) {
    size_t capacity = ptn_symbol_index_capacity_for_entries(expected_entries);
    if (capacity > symbols->index_capacity) {
        ptn_symbols_rebuild_index(symbols, expected_entries);
    }
}

static size_t ptn_symbols_find(PtnSymbolTable *symbols, const char *name) {
    if (symbols->index_capacity != 0) {
        uint64_t hash = ptn_symbol_name_hash(name);
        size_t slot_index = ptn_symbol_index_slot_for_name(symbols, name, hash);
        PtnSymbolIndexSlot *slot = &symbols->index_slots[slot_index];
        return slot->occupied ? slot->symbol_index : symbols->len;
    }
    return ptn_symbols_linear_find(symbols, name);
}

static PTN_UNUSED void ptn_symbols_set(PtnSymbolTable *symbols, const char *name, PtnValue value) {
    PtnValue stored_value = ptn_value_clone(value);
    ptn_symbols_ensure_index(symbols, symbols->len + 1);
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        ptn_value_destroy(&symbols->items[index].value);
        symbols->items[index].value = stored_value;
        return;
    }
    if (symbols->len == symbols->capacity) {
        size_t new_capacity = symbols->capacity == 0 ? 8 : symbols->capacity * 2;
        PtnSymbol *new_items = realloc(symbols->items, new_capacity * sizeof(PtnSymbol));
        if (new_items == NULL) {
            ptn_abort_out_of_memory();
        }
        symbols->items = new_items;
        symbols->capacity = new_capacity;
    }
    size_t symbol_index = symbols->len;
    symbols->items[symbol_index].name = ptn_duplicate_string(name);
    symbols->items[symbol_index].value = stored_value;
    symbols->len++;
    ptn_symbol_index_insert(symbols, name, symbol_index);
}

static int ptn_symbols_get(PtnSymbolTable *symbols, const char *name, PtnValue *out) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        *out = ptn_value_borrow(symbols->items[index].value);
        return 1;
    }
    return 0;
}

static PTN_UNUSED PtnValue *ptn_symbols_value_slot(PtnSymbolTable *symbols, const char *name) {
    size_t index = ptn_symbols_find(symbols, name);
    return index < symbols->len ? &symbols->items[index].value : NULL;
}

static PTN_UNUSED PtnValue *ptn_symbols_get_slot(PtnSymbolTable *symbols, const char *name) {
    return ptn_symbols_value_slot(symbols, name);
}

static PTN_UNUSED PtnSymbol *ptn_symbols_slot_for_write(PtnSymbolTable *symbols, const char *name) {
    ptn_symbols_ensure_index(symbols, symbols->len + 1);
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        return &symbols->items[index];
    }
    if (symbols->len == symbols->capacity) {
        size_t new_capacity = symbols->capacity == 0 ? 8 : symbols->capacity * 2;
        PtnSymbol *new_items = realloc(symbols->items, new_capacity * sizeof(PtnSymbol));
        if (new_items == NULL) {
            ptn_abort_out_of_memory();
        }
        symbols->items = new_items;
        symbols->capacity = new_capacity;
    }
    size_t symbol_index = symbols->len;
    symbols->items[symbol_index].name = ptn_duplicate_string(name);
    symbols->items[symbol_index].value = ptn_null();
    symbols->len++;
    ptn_symbol_index_insert(symbols, name, symbol_index);
    return &symbols->items[symbol_index];
}

static PTN_UNUSED PtnValue ptn_symbols_reference_for_variable(PtnSymbolTable *symbols, const char *name) {
    PtnSymbol *symbol = ptn_symbols_slot_for_write(symbols, name);
    if (symbol->value.type != PTN_REFERENCE) {
        PtnValue current = symbol->value;
        PtnReference *reference = ptn_reference_new_owned(current);
        symbol->value = ptn_reference_value(reference);
    }
    return ptn_value_clone(symbol->value);
}

static void ptn_value_unwrap_reference_slots(PtnValue *slot, PtnReference *reference, size_t depth) {
    if (slot == NULL || reference == NULL || depth > 1024) {
        return;
    }
    if (slot->type == PTN_REFERENCE && slot->as.reference == reference) {
        PtnValue old = *slot;
        *slot = ptn_value_clone_deref(old);
        ptn_value_destroy(&old);
        return;
    }

    PtnValue value = ptn_value_deref(*slot);
    if (value.type == PTN_ARRAY && value.as.array != NULL) {
        for (size_t i = 0; i < value.as.array->len; i++) {
            ptn_value_unwrap_reference_slots(&value.as.array->entries[i].value, reference, depth + 1);
        }
    } else if (value.type == PTN_OBJECT && value.as.object != NULL && value.as.object->properties != NULL) {
        for (size_t i = 0; i < value.as.object->properties->len; i++) {
            ptn_value_unwrap_reference_slots(&value.as.object->properties->entries[i].value, reference, depth + 1);
        }
    }
}

static void ptn_symbols_unwrap_reference_slots(PtnSymbolTable *symbols, PtnReference *reference) {
    if (symbols == NULL || reference == NULL) {
        return;
    }
    for (size_t i = 0; i < symbols->len; i++) {
        ptn_value_unwrap_reference_slots(&symbols->items[i].value, reference, 0);
    }
}

static PTN_UNUSED void ptn_runtime_unwrap_reference_slots_if_unaliased(
    PtnRuntime *runtime,
    PtnValue reference_value,
    size_t expected_refcount
) {
    if (runtime == NULL || reference_value.type != PTN_REFERENCE) {
        return;
    }
    PtnReference *reference = reference_value.as.reference;
    if (reference == NULL || reference->refcount != expected_refcount) {
        return;
    }
    ptn_symbols_unwrap_reference_slots(&runtime->symbols, reference);
    if (runtime->global_symbols != NULL && runtime->global_symbols != &runtime->symbols) {
        ptn_symbols_unwrap_reference_slots(runtime->global_symbols, reference);
    }
}

static PTN_UNUSED void ptn_symbols_bind_reference(PtnSymbolTable *symbols, const char *name, PtnValue reference) {
    PtnSymbol *symbol = ptn_symbols_slot_for_write(symbols, name);
    ptn_value_destroy(&symbol->value);
    symbol->value = ptn_value_clone(reference);
}

static PTN_UNUSED PtnClosure *ptn_closure_from_value(PtnValue closure) {
    PtnValue resolved = ptn_value_deref(closure);
    if (resolved.type != PTN_CLOSURE) {
        fputs("Fatal error: invalid closure capture target\n", stderr);
        exit(255);
    }
    return resolved.as.closure;
}

static PTN_UNUSED void ptn_closure_set_capture(PtnValue closure, const char *name, PtnValue value) {
    PtnClosure *resolved = ptn_closure_from_value(closure);
    ptn_symbols_set(&resolved->captures, name, ptn_value_deref(value));
}

static PTN_UNUSED void ptn_closure_bind_capture_reference(PtnValue closure, const char *name, PtnValue reference) {
    PtnClosure *resolved = ptn_closure_from_value(closure);
    ptn_symbols_bind_reference(&resolved->captures, name, reference);
}

static PTN_UNUSED void ptn_runtime_import_closure_captures(PtnRuntime *runtime, PtnValue closure) {
    PtnClosure *resolved = ptn_closure_from_value(closure);
    for (size_t i = 0; i < resolved->captures.len; i++) {
        PtnSymbol *capture = &resolved->captures.items[i];
        if (capture->value.type == PTN_REFERENCE) {
            ptn_symbols_bind_reference(&runtime->symbols, capture->name, capture->value);
        } else {
            ptn_symbols_set(&runtime->symbols, capture->name, capture->value);
        }
    }
}

static PTN_UNUSED PtnValue ptn_closure_clone(PtnRuntime *runtime, PtnValue closure) {
    PtnClosure *source = ptn_closure_from_value(closure);
    PtnValue copy = ptn_closure(
        runtime,
        source->function_index,
        source->display_name,
        source->metadata
    );
    for (size_t i = 0; i < source->captures.len; i++) {
        PtnSymbol *capture = &source->captures.items[i];
        if (capture->value.type == PTN_REFERENCE) {
            ptn_closure_bind_capture_reference(copy, capture->name, capture->value);
        } else {
            ptn_closure_set_capture(copy, capture->name, capture->value);
        }
    }
    if (source->has_wrapped_callable) {
        copy.as.closure->has_wrapped_callable = 1;
        copy.as.closure->wrapped_callable = ptn_value_clone(source->wrapped_callable);
    }
    return copy;
}

static PTN_UNUSED PtnValue ptn_closure_wrap_callable(
    PtnRuntime *runtime,
    PtnValue callable,
    PtnFunctionMetadata metadata
) {
    PtnValue closure = ptn_closure(runtime, (size_t)-1, "Closure::__invoke", metadata);
    closure.as.closure->has_wrapped_callable = 1;
    closure.as.closure->wrapped_callable = ptn_value_clone_deref(callable);
    return closure;
}

static PTN_UNUSED void ptn_symbols_unset(PtnSymbolTable *symbols, const char *name) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index >= symbols->len) {
        return;
    }

    free(symbols->items[index].name);
    ptn_value_destroy(&symbols->items[index].value);
    for (size_t i = index + 1; i < symbols->len; i++) {
        symbols->items[i - 1] = symbols->items[i];
    }
    symbols->len--;
    ptn_symbols_rebuild_index(symbols, symbols->len);
}

static int ptn_parse_int64_env(const char *name, int64_t *out) {
    const char *configured = getenv(name);
    if (configured == NULL || configured[0] == '\0') {
        return 0;
    }
    char *end = NULL;
    errno = 0;
    long long parsed = strtoll(configured, &end, 10);
    if (errno != 0 || end == configured || *end != '\0') {
        return 0;
    }
    *out = (int64_t)parsed;
    return 1;
}

static int ptn_ascii_case_equal_literal(const char *value, const char *literal) {
    while (*value != '\0' && *literal != '\0') {
        if (tolower((unsigned char)*value) != tolower((unsigned char)*literal)) {
            return 0;
        }
        value++;
        literal++;
    }
    return *value == '\0' && *literal == '\0';
}

static int ptn_parse_bool_env(const char *name, int *out) {
    const char *configured = getenv(name);
    if (configured == NULL) {
        return 0;
    }
    if (configured[0] == '\0') {
        *out = 0;
        return 1;
    }
    if (
        strcmp(configured, "0") == 0 ||
        ptn_ascii_case_equal_literal(configured, "false") ||
        ptn_ascii_case_equal_literal(configured, "off") ||
        ptn_ascii_case_equal_literal(configured, "no")
    ) {
        *out = 0;
        return 1;
    }
    *out = 1;
    return 1;
}

static void ptn_diagnostics_init(PtnDiagnosticSink *diagnostics, FILE *stream) {
    diagnostics->stream = stream;
    diagnostics->emitted_deprecation = 0;
    diagnostics->emitted_warning = 0;
    diagnostics->suppressed = 0;
    diagnostics->error_reporting = PTN_E_ALL;
    diagnostics->display_errors = 1;
    int64_t configured_error_reporting = 0;
    if (ptn_parse_int64_env("PTN_PHP_ERROR_REPORTING", &configured_error_reporting)) {
        diagnostics->error_reporting = configured_error_reporting;
    }
    int configured_display_errors = 1;
    if (ptn_parse_bool_env("PTN_PHP_DISPLAY_ERRORS", &configured_display_errors)) {
        diagnostics->display_errors = configured_display_errors;
    }
}

static PTN_UNUSED int ptn_diagnostics_should_emit(PtnDiagnosticSink *diagnostics, int64_t severity) {
    return diagnostics->display_errors &&
        diagnostics->suppressed <= 0 &&
        (diagnostics->error_reporting & severity) != 0;
}

static void ptn_emit_undefined_variable_warning(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    const char *path,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    FILE *stream = diagnostics->stream == NULL ? stdout : diagnostics->stream;
    fputc('\n', stream);
    diagnostics->emitted_warning = 1;
    fputs("Warning: Undefined variable $", stream);
    fputs(name, stream);
    fputs(" in ", stream);
    fputs(path, stream);
    fputs(" on line ", stream);
    fprintf(stream, "%zu", line);
    fputc('\n', stream);
}

static PTN_UNUSED void ptn_emit_undefined_function_error(PtnDiagnosticSink *diagnostics, const char *name) {
    if (!diagnostics->display_errors) {
        return;
    }
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: Call to undefined function ", stream);
    fputs(name, stream);
    fputs("()\n", stream);
}

static PTN_UNUSED void ptn_emit_undefined_constant_error(PtnDiagnosticSink *diagnostics, const char *name) {
    if (!diagnostics->display_errors) {
        return;
    }
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: Undefined constant \"", stream);
    fputs(name, stream);
    fputs("\"\n", stream);
}

static PTN_UNUSED void ptn_emit_argument_count_error(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t min_args,
    size_t argc
) {
    if (!diagnostics->display_errors) {
        return;
    }
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(name, stream);
    fputs("() expects at least ", stream);
    fprintf(stream, "%zu", min_args);
    fputs(" argument", stream);
    if (min_args != 1) {
        fputc('s', stream);
    }
    fputs(", ", stream);
    fprintf(stream, "%zu", argc);
    fputs(" given\n", stream);
}

static PTN_UNUSED void ptn_emit_too_many_arguments_error(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t max_args,
    size_t argc
) {
    if (!diagnostics->display_errors) {
        return;
    }
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(name, stream);
    fputs("() expects at most ", stream);
    fprintf(stream, "%zu", max_args);
    fputs(" argument", stream);
    if (max_args != 1) {
        fputc('s', stream);
    }
    fputs(", ", stream);
    fprintf(stream, "%zu", argc);
    fputs(" given\n", stream);
}

static PTN_UNUSED void ptn_emit_type_error(PtnDiagnosticSink *diagnostics, const char *message) {
    if (!diagnostics->display_errors) {
        return;
    }
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(message, stream);
    fputc('\n', stream);
}

static PTN_UNUSED void ptn_emit_memory_allocation_overflow_error(
    PtnRuntime *runtime,
    size_t count,
    size_t element_size,
    size_t overhead,
    size_t line
) {
    fflush(stdout);
    if (runtime->diagnostics.display_errors) {
        FILE *stream = runtime->diagnostics.stream == NULL ? stderr : runtime->diagnostics.stream;
        fprintf(
            stream,
            "\nFatal error: Possible integer overflow in memory allocation (%zu * %zu + %zu) in %s on line %zu\n",
            count,
            element_size,
            overhead,
            runtime->source_path != NULL ? runtime->source_path : "ptn",
            line
        );
    }
    exit(255);
}

static void ptn_emit_deprecation(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    fputc('\n', stdout);
    diagnostics->emitted_deprecation = 1;
    fputs("Deprecated: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    if (diagnostics->emitted_deprecation || diagnostics->emitted_warning) {
        fputc('\n', stdout);
    }
    diagnostics->emitted_warning = 1;
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_notice(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputs("Notice: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_runtime_warning(PtnRuntime *runtime, const char *message, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    diagnostics->emitted_warning = 1;
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_spaced_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_only_variables_assigned_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Only variables should be assigned by reference in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_only_variables_assigned_by_reference_notice_at(PtnRuntime *runtime, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Only variables should be assigned by reference in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_attempting_to_set_reference_to_non_referenceable_value_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Attempting to set reference to non referenceable value in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_only_variables_passed_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Only variables should be passed by reference in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_only_variable_references_returned_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Only variable references should be returned by reference in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_only_variable_references_yielded_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Only variable references should be yielded by reference in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_control_warning(const char *message, const char *path, size_t line) {
    fputc('\n', stdout);
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ", stdout);
    fputs(path, stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static void ptn_emit_constant_already_defined_warning(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    fputs("Warning: Constant ", stdout);
    fputs(name, stdout);
    fputs(" already defined, this will be an error in PHP 9 in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_define_case_insensitive_ignored_warning(
    PtnDiagnosticSink *diagnostics,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    fputs(
        "Warning: define(): Argument #3 ($case_insensitive) is ignored since declaration of case-insensitive constants is no longer supported in ptn on line ",
        stdout
    );
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static void ptn_runtime_init(PtnRuntime *runtime) {
    ptn_symbols_init(&runtime->symbols);
    runtime->global_symbols = &runtime->symbols;
    ptn_symbols_init(&runtime->owned_constants);
    runtime->constants = &runtime->owned_constants;
    ptn_symbols_init(&runtime->owned_class_aliases);
    runtime->class_aliases = &runtime->owned_class_aliases;
    ptn_symbols_init(&runtime->owned_class_constants);
    runtime->class_constants = &runtime->owned_class_constants;
    ptn_symbols_init(&runtime->owned_static_properties);
    runtime->static_properties = &runtime->owned_static_properties;
    ptn_symbols_init(&runtime->owned_static_property_read_visibility);
    runtime->static_property_read_visibility = &runtime->owned_static_property_read_visibility;
    ptn_symbols_init(&runtime->owned_static_property_set_visibility);
    runtime->static_property_set_visibility = &runtime->owned_static_property_set_visibility;
    ptn_diagnostics_init(&runtime->diagnostics, NULL);
    runtime->owned_exceptions.active_exception = NULL;
    runtime->owned_exceptions.try_frame = NULL;
    runtime->exceptions = &runtime->owned_exceptions;
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
    runtime->trace_frame = NULL;
    runtime->lifecycle_root = runtime;
    runtime->live_objects = NULL;
    runtime->live_objects_len = 0;
    runtime->live_objects_capacity = 0;
    runtime->next_object_id = 1;
    runtime->free_object_ids = NULL;
    runtime->free_object_ids_len = 0;
    runtime->free_object_ids_capacity = 0;
    runtime->output_buffers = NULL;
    runtime->output_buffers_len = 0;
    runtime->output_buffers_capacity = 0;
    runtime->output_buffer_callback_depth = 0;
    runtime->method_dispatch = NULL;
    runtime->declared_method_exists = NULL;
    runtime->class_scope_allows = NULL;
    runtime->declared_class_is_readonly = NULL;
    runtime->magic_property_read = NULL;
    runtime->magic_property_isset = NULL;
    runtime->declared_user_functions = NULL;
    runtime->magic_property_get = NULL;
    runtime->magic_property_get_exists = NULL;
    runtime->magic_property_set = NULL;
    runtime->magic_property_unset = NULL;
    runtime->magic_debug_info = NULL;
    runtime->in_magic_property_dispatch = 0;
    runtime->source_path = NULL;
    runtime->current_function_name = NULL;
    runtime->current_class_name = NULL;
    runtime->current_called_class_name = NULL;
    runtime->called_class_name_override = NULL;
    runtime->current_generator = NULL;
    runtime->has_current_receiver = 0;
    runtime->current_receiver = ptn_null();
    runtime->by_ref_argument_function_name_override = NULL;
    runtime->by_ref_argument_notice_pending = 0;
    runtime->by_ref_argument_notice_emitted = 0;
    runtime->by_ref_argument_notice_line = 0;
    runtime->include_path = ptn_duplicate_string(".");
    const char *configured_max_memory_limit = getenv("PTN_MAX_MEMORY_LIMIT");
    const char *configured_memory_limit = getenv("PTN_MEMORY_LIMIT");
    runtime->max_memory_limit = ptn_duplicate_string(
        configured_max_memory_limit == NULL ? "-1" : configured_max_memory_limit
    );
    runtime->memory_limit = ptn_duplicate_string(
        configured_memory_limit == NULL ? "128M" : configured_memory_limit
    );
    runtime->exception_string_param_max_len = 15;
    int64_t configured_exception_string_param_max_len = 0;
    if (ptn_parse_int64_env(
            "PTN_EXCEPTION_STRING_PARAM_MAX_LEN",
            &configured_exception_string_param_max_len
        ) && configured_exception_string_param_max_len >= 0 &&
        configured_exception_string_param_max_len <= 1000000) {
        runtime->exception_string_param_max_len =
            (size_t)configured_exception_string_param_max_len;
    }
    runtime->strict_types = 0;
    runtime->zend_assertions = 1;
    int64_t configured_zend_assertions = 0;
    if (ptn_parse_int64_env("PTN_ZEND_ASSERTIONS", &configured_zend_assertions)) {
        if (configured_zend_assertions < 0) {
            runtime->zend_assertions = -1;
        } else if (configured_zend_assertions == 0) {
            runtime->zend_assertions = 0;
        } else {
            runtime->zend_assertions = 1;
        }
    }
    runtime->initial_zend_assertions = runtime->zend_assertions;
    runtime->assert_exception = 1;
    int configured_assert_exception = 1;
    if (ptn_parse_bool_env("PTN_ASSERT_EXCEPTION", &configured_assert_exception)) {
        runtime->assert_exception = configured_assert_exception;
    }
    runtime->call_site_line = 0;
    runtime->warn_by_ref_argument_mismatch = 0;
    runtime->throw_argument_count_errors = 0;
    runtime->active_unserialize_state = NULL;
    runtime->strtok_string = NULL;
    runtime->strtok_len = 0;
    runtime->strtok_offset = 0;
    runtime->strtok_has_state = 0;
}
