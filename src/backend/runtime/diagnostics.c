
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
        PtnValue old_value = symbols->items[index].value;
        ptn_array_note_value_replacement(old_value, stored_value);
        symbols->items[index].value = stored_value;
        ptn_value_destroy(&old_value);
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
    PtnValue old_value = symbol->value;
    ptn_array_note_value_replacement(old_value, reference);
    symbol->value = ptn_value_clone(reference);
    ptn_value_destroy(&old_value);
}

static PTN_UNUSED PtnClosure *ptn_closure_from_value(PtnValue closure) {
    PtnValue resolved = ptn_value_deref(closure);
    if (resolved.type != PTN_CLOSURE) {
        fputs("Fatal error: invalid closure capture target\n", stderr);
        exit(255);
    }
    return resolved.as.closure;
}

static PTN_UNUSED void ptn_closure_replace_scope(char **target, const char *class_name) {
    free(*target);
    *target = class_name == NULL ? NULL : ptn_duplicate_string(class_name);
}

static PTN_UNUSED void ptn_closure_set_scope(
    PtnValue closure,
    const char *scope_class_name,
    const char *called_class_name
) {
    PtnClosure *resolved = ptn_closure_from_value(closure);
    ptn_closure_replace_scope(&resolved->scope_class_name, scope_class_name);
    ptn_closure_replace_scope(
        &resolved->called_class_name,
        called_class_name != NULL ? called_class_name : scope_class_name
    );
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
        source->metadata,
        source->is_static,
        source->uses_this
    );
    ptn_closure_set_scope(copy, source->scope_class_name, source->called_class_name);
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
    if (source->bound_scope_name != NULL) {
        copy.as.closure->bound_scope_name = ptn_duplicate_string(source->bound_scope_name);
    }
    return copy;
}

static PTN_UNUSED void ptn_closure_set_bound_scope_name(PtnValue closure, const char *scope_name) {
    PtnClosure *resolved = ptn_closure_from_value(closure);
    free(resolved->bound_scope_name);
    resolved->bound_scope_name = scope_name == NULL ? NULL : ptn_duplicate_string(scope_name);
}

static PTN_UNUSED PtnValue ptn_closure_clone_bound(
    PtnRuntime *runtime,
    PtnValue closure,
    size_t argc,
    const PtnValue *args
) {
    PtnValue copy = ptn_closure_clone(runtime, closure);
    if (argc < 2) {
        return copy;
    }
    PtnValue scope = ptn_value_deref(args[1]);
    if (scope.type == PTN_NULL) {
        ptn_closure_set_bound_scope_name(copy, NULL);
    } else if (scope.type == PTN_STRING) {
        char *scope_name = ptn_value_to_string(scope);
        ptn_closure_set_bound_scope_name(copy, scope_name);
        free(scope_name);
    } else if (scope.type == PTN_OBJECT) {
        ptn_closure_set_bound_scope_name(copy, scope.as.object->class_name);
    } else if (scope.type == PTN_EXCEPTION) {
        ptn_closure_set_bound_scope_name(copy, scope.as.exception->class_name);
    }
    return copy;
}

static PTN_UNUSED PtnValue ptn_closure_wrap_callable(
    PtnRuntime *runtime,
    PtnValue callable,
    PtnFunctionMetadata metadata
) {
    PtnValue closure = ptn_closure(runtime, (size_t)-1, "Closure::__invoke", metadata, 0, 0);
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

static const char *ptn_closure_bind_argument_type_name(PtnValue value) {
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
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_RESOURCE:
            return "resource";
        case PTN_REFERENCE:
            return ptn_closure_bind_argument_type_name(value.as.reference->value);
    }
    return "unknown";
}

static int ptn_closure_bind_scope_class_exists(const char *class_name) {
    return ptn_declared_class_exists(class_name)
        || ptn_ascii_case_equal(class_name, "Closure")
        || ptn_ascii_case_equal(class_name, "stdClass")
        || ptn_ascii_case_equal(class_name, "Generator")
        || ptn_ascii_case_equal(class_name, "DateTime")
        || ptn_builtin_exception_class_name(class_name) != NULL;
}

static PTN_UNUSED PtnValue ptn_closure_bind_to(
    PtnRuntime *runtime,
    PtnValue closure_value,
    PtnValue new_this_value,
    int has_new_scope,
    PtnValue new_scope_value,
    const char *function_name,
    size_t new_this_position,
    size_t new_scope_position,
    size_t line
) {
    PtnValue resolved_closure = ptn_value_deref(closure_value);
    if (resolved_closure.type != PTN_CLOSURE) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #1 ($closure) must be of type Closure, %s given",
            function_name,
            ptn_closure_bind_argument_type_name(resolved_closure)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }

    PtnValue new_this = ptn_value_deref(new_this_value);
    if (new_this.type != PTN_NULL && new_this.type != PTN_OBJECT && new_this.type != PTN_EXCEPTION) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($newThis) must be of type object or null, %s given",
            function_name,
            new_this_position,
            ptn_closure_bind_argument_type_name(new_this)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }

    PtnClosure *source = resolved_closure.as.closure;
    if (source->is_static && new_this.type != PTN_NULL) {
        ptn_emit_warning(
            &runtime->diagnostics,
            "Cannot bind an instance to a static closure, this will be an error in PHP 9",
            line
        );
        return ptn_null();
    }

    const char *scope_class_name = source->scope_class_name;
    char *owned_scope = NULL;
    if (has_new_scope) {
        PtnValue scope = ptn_value_deref(new_scope_value);
        if (scope.type == PTN_NULL) {
            scope_class_name = "Closure";
        } else if (scope.type == PTN_STRING) {
            owned_scope = ptn_value_to_string(scope);
            if (ptn_ascii_case_equal(owned_scope, "static")) {
                scope_class_name = source->scope_class_name;
            } else if (!ptn_closure_bind_scope_class_exists(owned_scope)) {
                char message[192];
                int written = snprintf(
                    message,
                    sizeof(message),
                    "Class \"%s\" not found",
                    owned_scope
                );
                if (written < 0 || (size_t)written >= sizeof(message)) {
                    free(owned_scope);
                    ptn_abort_out_of_memory();
                }
                ptn_emit_warning(&runtime->diagnostics, message, line);
                free(owned_scope);
                return ptn_null();
            } else {
                scope_class_name = owned_scope;
            }
        } else if (scope.type == PTN_OBJECT) {
            scope_class_name = scope.as.object->class_name;
        } else if (scope.type == PTN_EXCEPTION) {
            scope_class_name = scope.as.exception->class_name;
        } else {
            char message[224];
            int written = snprintf(
                message,
                sizeof(message),
                "%s(): Argument #%zu ($newScope) must be of type object|string|null, %s given",
                function_name,
                new_scope_position,
                ptn_closure_bind_argument_type_name(scope)
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "TypeError", message);
            return ptn_null();
        }
    }

    PtnValue existing_this;
    int has_existing_this = ptn_symbols_get(&source->captures, "this", &existing_this);
    if (new_this.type == PTN_NULL && source->uses_this && has_existing_this) {
        ptn_emit_warning(
            &runtime->diagnostics,
            "Cannot unbind $this of closure using $this, this will be an error in PHP 9",
            line
        );
        free(owned_scope);
        return ptn_null();
    }

    PtnValue bound = ptn_closure_clone(runtime, resolved_closure);
    const char *called_class_name = scope_class_name;
    if (new_this.type == PTN_NULL) {
        ptn_symbols_unset(&bound.as.closure->captures, "this");
    } else {
        const char *new_this_class_name = new_this.type == PTN_EXCEPTION
            ? new_this.as.exception->class_name
            : new_this.as.object->class_name;
        if (!has_new_scope) {
            called_class_name = new_this_class_name;
        }
        if (scope_class_name == NULL) {
            scope_class_name = "Closure";
        }
        ptn_closure_set_capture(bound, "this", new_this);
    }
    if (called_class_name == NULL) {
        called_class_name = scope_class_name;
    }
    ptn_closure_set_scope(bound, scope_class_name, called_class_name);
    free(owned_scope);
    return bound;
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
    diagnostics->runtime = NULL;
    diagnostics->stream = stream;
    diagnostics->emitted_deprecation = 0;
    diagnostics->emitted_warning = 0;
    diagnostics->suppressed = 0;
    diagnostics->error_reporting = PTN_E_ALL;
    diagnostics->display_errors = 1;
    diagnostics->has_error_handler = 0;
    diagnostics->error_handler = ptn_null();
    diagnostics->error_handler_levels = PTN_E_ALL;
    diagnostics->error_handler_stack = NULL;
    diagnostics->error_handler_stack_len = 0;
    diagnostics->error_handler_stack_capacity = 0;
    int64_t configured_error_reporting = 0;
    if (ptn_parse_int64_env("PTN_PHP_ERROR_REPORTING", &configured_error_reporting)) {
        diagnostics->error_reporting = configured_error_reporting;
    }
    int configured_display_errors = 1;
    if (ptn_parse_bool_env("PTN_PHP_DISPLAY_ERRORS", &configured_display_errors)) {
        diagnostics->display_errors = configured_display_errors;
    }
}

static void ptn_diagnostics_clear_current_error_handler(PtnDiagnosticSink *diagnostics) {
    if (diagnostics == NULL || !diagnostics->has_error_handler) {
        return;
    }
    ptn_value_destroy(&diagnostics->error_handler);
    diagnostics->error_handler = ptn_null();
    diagnostics->has_error_handler = 0;
    diagnostics->error_handler_levels = PTN_E_ALL;
}

static PTN_UNUSED PtnValue ptn_diagnostics_current_error_handler(PtnDiagnosticSink *diagnostics) {
    if (diagnostics == NULL || !diagnostics->has_error_handler) {
        return ptn_null();
    }
    return ptn_value_clone(diagnostics->error_handler);
}

static PTN_UNUSED void ptn_diagnostics_push_error_handler(
    PtnDiagnosticSink *diagnostics,
    int has_handler,
    PtnValue handler,
    int64_t levels
) {
    if (diagnostics == NULL) {
        return;
    }
    if (diagnostics->error_handler_stack_len == diagnostics->error_handler_stack_capacity) {
        size_t capacity = diagnostics->error_handler_stack_capacity == 0
            ? 4
            : diagnostics->error_handler_stack_capacity * 2;
        PtnErrorHandlerFrame *stack = realloc(
            diagnostics->error_handler_stack,
            capacity * sizeof(PtnErrorHandlerFrame)
        );
        if (stack == NULL) {
            ptn_abort_out_of_memory();
        }
        diagnostics->error_handler_stack = stack;
        diagnostics->error_handler_stack_capacity = capacity;
    }
    PtnErrorHandlerFrame *frame =
        &diagnostics->error_handler_stack[diagnostics->error_handler_stack_len++];
    frame->has_handler = diagnostics->has_error_handler;
    frame->handler = diagnostics->has_error_handler
        ? ptn_value_clone(diagnostics->error_handler)
        : ptn_null();
    frame->levels = diagnostics->error_handler_levels;

    ptn_diagnostics_clear_current_error_handler(diagnostics);
    diagnostics->has_error_handler = has_handler;
    diagnostics->error_handler = has_handler ? ptn_value_clone(handler) : ptn_null();
    diagnostics->error_handler_levels = levels;
}

static PTN_UNUSED void ptn_diagnostics_restore_error_handler(PtnDiagnosticSink *diagnostics) {
    if (diagnostics == NULL) {
        return;
    }
    ptn_diagnostics_clear_current_error_handler(diagnostics);
    if (diagnostics->error_handler_stack_len == 0) {
        return;
    }
    PtnErrorHandlerFrame frame =
        diagnostics->error_handler_stack[--diagnostics->error_handler_stack_len];
    diagnostics->has_error_handler = frame.has_handler;
    diagnostics->error_handler = frame.has_handler ? frame.handler : ptn_null();
    diagnostics->error_handler_levels = frame.levels;
}

static PTN_UNUSED void ptn_diagnostics_clear_error_handler(PtnDiagnosticSink *diagnostics) {
    if (diagnostics == NULL) {
        return;
    }
    ptn_diagnostics_clear_current_error_handler(diagnostics);
    for (size_t i = 0; i < diagnostics->error_handler_stack_len; i++) {
        if (diagnostics->error_handler_stack[i].has_handler) {
            ptn_value_destroy(&diagnostics->error_handler_stack[i].handler);
        }
    }
    free(diagnostics->error_handler_stack);
    diagnostics->error_handler_stack = NULL;
    diagnostics->error_handler_stack_len = 0;
    diagnostics->error_handler_stack_capacity = 0;
}

static PTN_UNUSED void ptn_exception_handlers_init(PtnExceptionState *state) {
    if (state == NULL) {
        return;
    }
    state->has_exception_handler = 0;
    state->exception_handler = ptn_null();
    state->exception_handler_stack = NULL;
    state->exception_handler_stack_len = 0;
    state->exception_handler_stack_capacity = 0;
    state->in_exception_handler = 0;
}

static void ptn_exception_handlers_clear_current(PtnExceptionState *state) {
    if (state == NULL || !state->has_exception_handler) {
        return;
    }
    ptn_value_destroy(&state->exception_handler);
    state->exception_handler = ptn_null();
    state->has_exception_handler = 0;
}

static PTN_UNUSED PtnValue ptn_exception_handlers_current(PtnExceptionState *state) {
    if (state == NULL || !state->has_exception_handler) {
        return ptn_null();
    }
    return ptn_value_clone(state->exception_handler);
}

static PTN_UNUSED void ptn_exception_handlers_push(
    PtnExceptionState *state,
    int has_handler,
    PtnValue handler
) {
    if (state == NULL) {
        return;
    }
    if (state->exception_handler_stack_len == state->exception_handler_stack_capacity) {
        size_t capacity = state->exception_handler_stack_capacity == 0
            ? 4
            : state->exception_handler_stack_capacity * 2;
        PtnExceptionHandlerFrame *stack = realloc(
            state->exception_handler_stack,
            capacity * sizeof(PtnExceptionHandlerFrame)
        );
        if (stack == NULL) {
            ptn_abort_out_of_memory();
        }
        state->exception_handler_stack = stack;
        state->exception_handler_stack_capacity = capacity;
    }
    PtnExceptionHandlerFrame *frame =
        &state->exception_handler_stack[state->exception_handler_stack_len++];
    frame->has_handler = state->has_exception_handler;
    frame->handler = state->has_exception_handler
        ? ptn_value_clone(state->exception_handler)
        : ptn_null();

    ptn_exception_handlers_clear_current(state);
    state->has_exception_handler = has_handler;
    state->exception_handler = has_handler ? ptn_value_clone(handler) : ptn_null();
}

static PTN_UNUSED void ptn_exception_handlers_restore(PtnExceptionState *state) {
    if (state == NULL) {
        return;
    }
    ptn_exception_handlers_clear_current(state);
    if (state->exception_handler_stack_len == 0) {
        return;
    }
    PtnExceptionHandlerFrame frame =
        state->exception_handler_stack[--state->exception_handler_stack_len];
    state->has_exception_handler = frame.has_handler;
    state->exception_handler = frame.has_handler ? frame.handler : ptn_null();
}

static PTN_UNUSED void ptn_exception_handlers_clear(PtnExceptionState *state) {
    if (state == NULL) {
        return;
    }
    ptn_exception_handlers_clear_current(state);
    for (size_t i = 0; i < state->exception_handler_stack_len; i++) {
        if (state->exception_handler_stack[i].has_handler) {
            ptn_value_destroy(&state->exception_handler_stack[i].handler);
        }
    }
    free(state->exception_handler_stack);
    state->exception_handler_stack = NULL;
    state->exception_handler_stack_len = 0;
    state->exception_handler_stack_capacity = 0;
    state->in_exception_handler = 0;
}

static PTN_UNUSED int ptn_exception_handlers_try_uncaught(
    PtnRuntime *runtime,
    PtnException *exception
) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (runtime == NULL || runtime->exceptions == NULL || exception == NULL) {
        return 0;
    }
    PtnExceptionState *state = runtime->exceptions;
    if (!state->has_exception_handler || state->in_exception_handler) {
        return 0;
    }
    PtnValue handler = ptn_value_clone(state->exception_handler);
    PtnException *saved_active = state->active_exception;
    if (saved_active == exception) {
        state->active_exception = NULL;
    }
    state->in_exception_handler = 1;
    PtnValue arg = ptn_exception_borrow(exception);
    PtnValue result = ptn_call_callable(runtime, handler, 1, &arg, 0);
    ptn_value_destroy(&result);
    state->in_exception_handler = 0;
    ptn_value_destroy(&handler);
    return state->active_exception == NULL;
#else
    (void)runtime;
    (void)exception;
    return 0;
#endif
}

static PTN_UNUSED void ptn_diagnostic_output_write(
    PtnDiagnosticSink *diagnostics,
    const char *data,
    size_t len
) {
    if (data == NULL || len == 0) {
        return;
    }
    if (diagnostics != NULL && diagnostics->stream != NULL) {
        fwrite(data, 1, len, diagnostics->stream);
        return;
    }
    if (diagnostics != NULL && diagnostics->runtime != NULL) {
        ptn_output_write(diagnostics->runtime, data, len);
        return;
    }
    fwrite(data, 1, len, stdout);
}

static PTN_UNUSED void ptn_diagnostic_output_cstr(PtnDiagnosticSink *diagnostics, const char *data) {
    ptn_diagnostic_output_write(diagnostics, data, data == NULL ? 0 : strlen(data));
}

static PTN_UNUSED void ptn_diagnostic_printf(PtnDiagnosticSink *diagnostics, const char *format, ...) {
    va_list args;
    va_start(args, format);
    va_list copy;
    va_copy(copy, args);
    int needed = vsnprintf(NULL, 0, format, args);
    va_end(args);
    if (needed < 0) {
        va_end(copy);
        ptn_abort_out_of_memory();
    }
    char *buffer = malloc((size_t)needed + 1);
    if (buffer == NULL) {
        va_end(copy);
        ptn_abort_out_of_memory();
    }
    int written = vsnprintf(buffer, (size_t)needed + 1, format, copy);
    va_end(copy);
    if (written < 0 || written != needed) {
        free(buffer);
        ptn_abort_out_of_memory();
    }
    ptn_diagnostic_output_write(diagnostics, buffer, (size_t)written);
    free(buffer);
}

static PTN_UNUSED int ptn_diagnostics_should_emit(PtnDiagnosticSink *diagnostics, int64_t severity) {
    return diagnostics->display_errors &&
        diagnostics->suppressed <= 0 &&
        (diagnostics->error_reporting & severity) != 0;
}

static PTN_UNUSED const char *ptn_diagnostic_path(PtnDiagnosticSink *diagnostics, const char *path) {
    if (path != NULL) {
        return path;
    }
    if (diagnostics != NULL && diagnostics->runtime != NULL && diagnostics->runtime->source_path != NULL) {
        return diagnostics->runtime->source_path;
    }
    return "ptn";
}

static PTN_UNUSED const char *ptn_diagnostic_builtin_path(size_t line) {
    return line == 0 ? "Unknown" : "ptn";
}

static PTN_UNUSED int ptn_diagnostics_try_error_handler(
    PtnDiagnosticSink *diagnostics,
    int64_t severity,
    const char *message,
    const char *path,
    size_t line
) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (diagnostics == NULL || diagnostics->runtime == NULL) {
        return 0;
    }
    PtnRuntime *runtime = diagnostics->runtime;
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    PtnDiagnosticSink *handler_diagnostics = &root->diagnostics;
    if (
        !handler_diagnostics->has_error_handler ||
        (handler_diagnostics->error_handler_levels & severity) == 0
    ) {
        return 0;
    }

    const char *effective_message = message == NULL ? "" : message;
    const char *effective_path = (path == NULL && line == 0)
        ? "Unknown"
        : ptn_diagnostic_path(diagnostics, path);
    PtnValue args[4] = {
        ptn_int(severity),
        ptn_owned_string(ptn_duplicate_string(effective_message)),
        ptn_owned_string(ptn_duplicate_string(effective_path)),
        ptn_int((int64_t)line),
    };
    PtnValue saved_handler = ptn_value_clone(handler_diagnostics->error_handler);
    int64_t saved_handler_levels = handler_diagnostics->error_handler_levels;
    ptn_diagnostics_clear_current_error_handler(handler_diagnostics);
    PtnValue result = ptn_call_callable(runtime, saved_handler, 4, args, line);
    if (!handler_diagnostics->has_error_handler) {
        handler_diagnostics->error_handler = saved_handler;
        handler_diagnostics->has_error_handler = 1;
        handler_diagnostics->error_handler_levels = saved_handler_levels;
    } else {
        ptn_value_destroy(&saved_handler);
    }
    for (size_t i = 0; i < 4; i++) {
        ptn_value_destroy(&args[i]);
    }
    if (runtime->exceptions != NULL && runtime->exceptions->active_exception != NULL) {
        return 1;
    }
    PtnValue resolved = ptn_value_deref(result);
    int use_builtin_handler = resolved.type == PTN_BOOL && !resolved.as.boolean;
    ptn_value_destroy(&result);
    return !use_builtin_handler;
#else
    (void)diagnostics;
    (void)severity;
    (void)message;
    (void)path;
    (void)line;
    return 0;
#endif
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
    diagnostics->emitted_warning = 1;
    int needed = snprintf(NULL, 0, "Undefined variable $%s", name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(message, (size_t)needed + 1, "Undefined variable $%s", name);
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, path, line)) {
        free(message);
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nWarning: Undefined variable $%s in %s on line %zu\n",
        name,
        path,
        line
    );
    free(message);
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

static PTN_UNUSED void ptn_emit_fatal_error_at(
    PtnRuntime *runtime,
    const char *message,
    const char *path,
    size_t line
) {
    fflush(stdout);
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (diagnostics->display_errors) {
        FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
        fputs("Fatal error: ", stream);
        fputs(message, stream);
        fputs(" in ", stream);
        fputs(path != NULL ? path : (runtime->source_path != NULL ? runtime->source_path : "ptn"), stream);
        fputs(" on line ", stream);
        fprintf(stream, "%zu", line);
        fputc('\n', stream);
    }
    exit(255);
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

static PTN_UNUSED void ptn_emit_deprecation(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    diagnostics->emitted_deprecation = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_DEPRECATED, message, NULL, line)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nDeprecated: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_user_deprecation(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_USER_DEPRECATED)) {
        return;
    }
    diagnostics->emitted_deprecation = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_USER_DEPRECATED, message, NULL, line)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nDeprecated: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_runtime_deprecation(PtnRuntime *runtime, const char *message, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    diagnostics->emitted_deprecation = 1;
    if (ptn_diagnostics_try_error_handler(
        diagnostics,
        PTN_E_DEPRECATED,
        message,
        runtime->source_path != NULL ? runtime->source_path : "ptn",
        line
    )) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nDeprecated: %s in %s on line %zu\n",
        message,
        runtime->source_path != NULL ? runtime->source_path : "ptn",
        line
    );
}

static PTN_UNUSED void ptn_emit_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, NULL, line)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nWarning: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_user_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_USER_WARNING)) {
        return;
    }
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_USER_WARNING, message, NULL, line)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nWarning: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_user_notice(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_USER_NOTICE)) {
        return;
    }
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_USER_NOTICE, message, NULL, line)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nNotice: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_notice(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "Notice: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_runtime_warning(PtnRuntime *runtime, const char *message, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(
        diagnostics,
        PTN_E_WARNING,
        message,
        runtime->source_path != NULL ? runtime->source_path : "ptn",
        line
    )) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nWarning: %s in %s on line %zu\n",
        message,
        runtime->source_path != NULL ? runtime->source_path : "ptn",
        line
    );
}

static PTN_UNUSED void ptn_emit_compile_warning(PtnRuntime *runtime, const char *message, const char *path, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, path, line)) {
        diagnostics->emitted_warning = 1;
        return;
    }
    if (diagnostics->emitted_warning) {
        ptn_diagnostic_output_cstr(diagnostics, "\n");
    }
    diagnostics->emitted_warning = 1;
    ptn_diagnostic_printf(
        diagnostics,
        "Warning: %s in %s on line %zu\n",
        message,
        path != NULL ? path : "ptn",
        line
    );
}

static PTN_UNUSED void ptn_emit_compile_deprecation(PtnRuntime *runtime, const char *message, const char *path, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_DEPRECATED, message, path, line)) {
        diagnostics->emitted_deprecation = 1;
        return;
    }
    if (diagnostics->emitted_deprecation) {
        ptn_diagnostic_output_cstr(diagnostics, "\n");
    }
    diagnostics->emitted_deprecation = 1;
    ptn_diagnostic_printf(
        diagnostics,
        "Deprecated: %s in %s on line %zu\n",
        message,
        path != NULL ? path : "ptn",
        line
    );
}

static PTN_UNUSED void ptn_emit_spaced_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    ptn_diagnostic_printf(diagnostics, "\nWarning: %s in ptn on line %zu\n", message, line);
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

static PTN_UNUSED void ptn_emit_only_variables_passed_by_reference_notice_at(PtnRuntime *runtime, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Only variables should be passed by reference in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
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

static PTN_UNUSED void ptn_emit_only_variable_references_returned_by_reference_notice_at(PtnRuntime *runtime, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Only variable references should be returned by reference in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
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
    ptn_symbols_init(&runtime->owned_class_constant_deprecations);
    runtime->class_constant_deprecations = &runtime->owned_class_constant_deprecations;
    runtime->class_constant_deprecation_suppress_class = NULL;
    runtime->class_constant_deprecation_suppress_constant = NULL;
    ptn_symbols_init(&runtime->owned_static_properties);
    runtime->static_properties = &runtime->owned_static_properties;
    ptn_symbols_init(&runtime->owned_static_property_read_visibility);
    runtime->static_property_read_visibility = &runtime->owned_static_property_read_visibility;
    ptn_symbols_init(&runtime->owned_static_property_set_visibility);
    runtime->static_property_set_visibility = &runtime->owned_static_property_set_visibility;
    ptn_diagnostics_init(&runtime->diagnostics, NULL);
    runtime->diagnostics.runtime = runtime;
    runtime->owned_exceptions.active_exception = NULL;
    runtime->owned_exceptions.try_frame = NULL;
    ptn_exception_handlers_init(&runtime->owned_exceptions);
    runtime->exceptions = &runtime->owned_exceptions;
    runtime->owned_call_frame.argc = 0;
    runtime->owned_call_frame.args = NULL;
    runtime->owned_call_frame.parameter_count = 0;
    runtime->owned_call_frame.parameter_names = NULL;
    runtime->call_frame = NULL;
    runtime->owned_trace_frame.runtime = NULL;
    runtime->owned_trace_frame.function_name = NULL;
    runtime->owned_trace_frame.file = NULL;
    runtime->owned_trace_frame.line = 0;
    runtime->owned_trace_frame.argc = 0;
    runtime->owned_trace_frame.args = NULL;
    runtime->owned_trace_frame.parameter_count = 0;
    runtime->owned_trace_frame.parameter_names = NULL;
    runtime->owned_trace_frame.has_receiver = 0;
    runtime->owned_trace_frame.receiver = ptn_null();
    runtime->owned_trace_frame.previous = NULL;
    runtime->trace_frame = NULL;
    runtime->lifecycle_root = runtime;
    runtime->live_objects = NULL;
    runtime->live_objects_len = 0;
    runtime->live_objects_capacity = 0;
    runtime->static_local_slots = NULL;
    runtime->static_local_slots_len = 0;
    runtime->static_local_slots_capacity = 0;
    runtime->next_object_id = 1;
    runtime->free_object_ids = NULL;
    runtime->free_object_ids_len = 0;
    runtime->free_object_ids_capacity = 0;
    runtime->output_buffers = NULL;
    runtime->output_buffers_len = 0;
    runtime->output_buffers_capacity = 0;
    runtime->output_buffer_callback_depth = 0;
    runtime->method_dispatch = NULL;
    runtime->reflected_method_dispatch = NULL;
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
    runtime->class_constant_initializer = NULL;
    runtime->new_instance_without_constructor = NULL;
    runtime->in_magic_property_dispatch = 0;
    runtime->magic_property_frames = NULL;
    runtime->magic_property_frame_len = 0;
    runtime->magic_property_frame_capacity = 0;
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
    runtime->included_files = NULL;
    runtime->included_files_len = 0;
    runtime->included_files_capacity = 0;
    runtime->open_basedir = ptn_duplicate_string("");
    const char *configured_max_memory_limit = getenv("PTN_MAX_MEMORY_LIMIT");
    const char *configured_memory_limit = getenv("PTN_MEMORY_LIMIT");
    runtime->max_memory_limit = ptn_duplicate_string(
        configured_max_memory_limit == NULL ? "-1" : configured_max_memory_limit
    );
    runtime->memory_limit = ptn_duplicate_string(
        configured_memory_limit == NULL ? "128M" : configured_memory_limit
    );
    const char *configured_default_charset = getenv("PTN_DEFAULT_CHARSET");
    const char *configured_arg_separator_input = getenv("PTN_ARG_SEPARATOR_INPUT");
    const char *configured_output_handler = getenv("PTN_OUTPUT_HANDLER");
    const char *configured_filter_default = getenv("PTN_FILTER_DEFAULT");
    const char *configured_internal_encoding = getenv("PTN_INTERNAL_ENCODING");
    const char *configured_input_encoding = getenv("PTN_INPUT_ENCODING");
    const char *configured_output_encoding = getenv("PTN_OUTPUT_ENCODING");
    runtime->default_charset = ptn_duplicate_string(
        configured_default_charset == NULL ? "UTF-8" : configured_default_charset
    );
    runtime->arg_separator_input = ptn_duplicate_string(
        configured_arg_separator_input == NULL ? "&" : configured_arg_separator_input
    );
    runtime->output_handler = ptn_duplicate_string(
        configured_output_handler == NULL ? "" : configured_output_handler
    );
    runtime->filter_default = ptn_duplicate_string(
        configured_filter_default == NULL ? "unsafe_raw" : configured_filter_default
    );
    runtime->internal_encoding = ptn_duplicate_string(
        configured_internal_encoding == NULL ? "" : configured_internal_encoding
    );
    runtime->input_encoding = ptn_duplicate_string(
        configured_input_encoding == NULL ? "" : configured_input_encoding
    );
    runtime->output_encoding = ptn_duplicate_string(
        configured_output_encoding == NULL ? "" : configured_output_encoding
    );
    runtime->exception_ignore_args = 0;
    int configured_exception_ignore_args = 0;
    if (ptn_parse_bool_env("PTN_EXCEPTION_IGNORE_ARGS", &configured_exception_ignore_args)) {
        runtime->exception_ignore_args = configured_exception_ignore_args;
    }
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
    runtime->suppress_user_call_frame_location = 0;
    runtime->warn_by_ref_argument_mismatch = 0;
    runtime->throw_argument_count_errors = 0;
    runtime->active_serialize_state = NULL;
    runtime->active_unserialize_state = NULL;
    runtime->strtok_string = NULL;
    runtime->strtok_len = 0;
    runtime->strtok_offset = 0;
    runtime->strtok_has_state = 0;
    runtime->json_last_error = 0;
}
