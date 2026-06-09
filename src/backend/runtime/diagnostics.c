
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

static void ptn_diagnostics_init(PtnDiagnosticSink *diagnostics, FILE *stream) {
    diagnostics->stream = stream;
    diagnostics->emitted_deprecation = 0;
}

static void ptn_emit_undefined_variable_warning(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    const char *path,
    size_t line
) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Warning: Undefined variable $", stream);
    fputs(name, stream);
    fputs(" in ", stream);
    fputs(path, stream);
    fputs(" on line ", stream);
    fprintf(stream, "%zu", line);
    fputc('\n', stream);
}

static PTN_UNUSED void ptn_emit_undefined_function_error(PtnDiagnosticSink *diagnostics, const char *name) {
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: Call to undefined function ", stream);
    fputs(name, stream);
    fputs("()\n", stream);
}

static void ptn_emit_undefined_constant_error(PtnDiagnosticSink *diagnostics, const char *name) {
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
    FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
    fputs("Fatal error: ", stream);
    fputs(message, stream);
    fputc('\n', stream);
}

static void ptn_emit_deprecation(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (diagnostics->emitted_deprecation) {
        fputc('\n', stdout);
    }
    diagnostics->emitted_deprecation = 1;
    fputs("Deprecated: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    (void)diagnostics;
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ptn on line ", stdout);
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
    (void)diagnostics;
    fputs("Warning: Constant ", stdout);
    fputs(name, stdout);
    fputs(" already defined, this will be an error in PHP 9 in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static void ptn_runtime_init(PtnRuntime *runtime) {
    ptn_symbols_init(&runtime->symbols);
    ptn_symbols_init(&runtime->owned_constants);
    runtime->constants = &runtime->owned_constants;
    ptn_diagnostics_init(&runtime->diagnostics, stderr);
}
