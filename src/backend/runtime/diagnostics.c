
static int ptn_runtime_disabled_function_list_contains(const char *list, const char *name) {
    if (list == NULL || name == NULL || name[0] == '\0') {
        return 0;
    }

    const char *cursor = list;
    while (*cursor != '\0') {
        while (*cursor == ',' || isspace((unsigned char)*cursor)) {
            cursor++;
        }
        const char *start = cursor;
        while (*cursor != '\0' && *cursor != ',' && !isspace((unsigned char)*cursor)) {
            cursor++;
        }
        const char *end = cursor;
        while (end > start && isspace((unsigned char)end[-1])) {
            end--;
        }
        size_t len = (size_t)(end - start);
        if (len == strlen(name)) {
            int matches = 1;
            for (size_t i = 0; i < len; i++) {
                if (tolower((unsigned char)start[i]) != tolower((unsigned char)name[i])) {
                    matches = 0;
                    break;
                }
            }
            if (matches) {
                return 1;
            }
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_symbols_ensure_index(PtnSymbolTable *symbols, size_t expected_entries) {
    size_t capacity = ptn_symbol_index_capacity_for_entries(expected_entries);
    if (capacity > symbols->index_capacity) {
        ptn_symbols_rebuild_index(symbols, expected_entries);
    }
}

static size_t ptn_symbols_find_len(PtnSymbolTable *symbols, const char *name, size_t name_len) {
    if (symbols->index_capacity != 0) {
        uint64_t hash = ptn_symbol_name_hash_len(name, name_len);
        size_t slot_index = ptn_symbol_index_slot_for_name_len(symbols, name, name_len, hash);
        PtnSymbolIndexSlot *slot = &symbols->index_slots[slot_index];
        return slot->occupied ? slot->symbol_index : symbols->len;
    }
    return ptn_symbols_linear_find_len(symbols, name, name_len);
}

static size_t ptn_symbols_find(PtnSymbolTable *symbols, const char *name) {
    return ptn_symbols_find_len(symbols, name, strlen(name));
}

static PTN_UNUSED void ptn_symbols_set_len(
    PtnSymbolTable *symbols,
    const char *name,
    size_t name_len,
    PtnValue value
) {
    PtnValue stored_value = ptn_value_clone(value);
    ptn_symbols_ensure_index(symbols, symbols->len + 1);
    size_t index = ptn_symbols_find_len(symbols, name, name_len);
    if (index < symbols->len) {
        PtnValue old_value = symbols->items[index].value;
        ptn_array_note_value_replacement(old_value, stored_value);
        symbols->items[index].value = stored_value;
        symbols->mutation_epoch++;
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
    symbols->items[symbol_index].name = ptn_duplicate_string_len(name, name_len);
    symbols->items[symbol_index].name_len = name_len;
    symbols->items[symbol_index].value = stored_value;
    symbols->len++;
    symbols->mutation_epoch++;
    ptn_symbol_index_insert_len(symbols, name, name_len, symbol_index);
}

static PTN_UNUSED void ptn_symbols_set(PtnSymbolTable *symbols, const char *name, PtnValue value) {
    ptn_symbols_set_len(symbols, name, strlen(name), value);
}

static PTN_UNUSED void ptn_symbols_set_with_runtime_scope_at(
    PtnSymbolTable *symbols,
    const char *name,
    PtnValue value,
    PtnRuntime *runtime,
    size_t line
) {
    PtnValue stored_value = ptn_value_clone(value);
    ptn_gc_attach_value_runtime(runtime, stored_value, 0);
    ptn_symbols_ensure_index(symbols, symbols->len + 1);
    size_t index = ptn_symbols_find(symbols, name);
    if (index < symbols->len) {
        PtnValue old_value = symbols->items[index].value;
        ptn_array_note_value_replacement(old_value, stored_value);
        symbols->items[index].value = stored_value;
        symbols->mutation_epoch++;
        ptn_value_destroy_with_runtime_scope_at(runtime, &old_value, line);
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
    symbols->items[symbol_index].name_len = strlen(name);
    symbols->items[symbol_index].value = stored_value;
    symbols->len++;
    symbols->mutation_epoch++;
    ptn_symbol_index_insert(symbols, name, symbol_index);
}

static PTN_UNUSED void ptn_symbols_set_with_runtime_scope(
    PtnSymbolTable *symbols,
    const char *name,
    PtnValue value,
    PtnRuntime *runtime
) {
    ptn_symbols_set_with_runtime_scope_at(symbols, name, value, runtime, 0);
}

static int ptn_symbols_get_len(PtnSymbolTable *symbols, const char *name, size_t name_len, PtnValue *out) {
    size_t index = ptn_symbols_find_len(symbols, name, name_len);
    if (index < symbols->len) {
        *out = ptn_value_borrow(symbols->items[index].value);
        return 1;
    }
    return 0;
}

static int ptn_symbols_get(PtnSymbolTable *symbols, const char *name, PtnValue *out) {
    return ptn_symbols_get_len(symbols, name, strlen(name), out);
}

static PTN_UNUSED PtnValue *ptn_symbols_value_slot_len(PtnSymbolTable *symbols, const char *name, size_t name_len) {
    size_t index = ptn_symbols_find_len(symbols, name, name_len);
    return index < symbols->len ? &symbols->items[index].value : NULL;
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
    symbols->items[symbol_index].name_len = strlen(name);
    symbols->items[symbol_index].value = ptn_null();
    symbols->len++;
    symbols->mutation_epoch++;
    ptn_symbol_index_insert(symbols, name, symbol_index);
    return &symbols->items[symbol_index];
}

static PTN_UNUSED PtnValue ptn_symbols_reference_for_variable(PtnSymbolTable *symbols, const char *name) {
    PtnSymbol *symbol = ptn_symbols_slot_for_write(symbols, name);
    if (symbol->value.type != PTN_REFERENCE) {
        PtnValue current = symbol->value;
        PtnReference *reference = ptn_reference_new_owned(current);
        symbol->value = ptn_reference_value(reference);
        symbols->mutation_epoch++;
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
    symbols->mutation_epoch++;
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

static PTN_UNUSED void ptn_closure_set_origin(
    PtnValue closure,
    int origin_kind,
    const char *origin_class_name,
    const char *origin_method_name
) {
    PtnClosure *resolved = ptn_closure_from_value(closure);
    resolved->origin_kind = origin_kind;
    ptn_closure_replace_scope(&resolved->origin_class_name, origin_class_name);
    ptn_closure_replace_scope(&resolved->origin_method_name, origin_method_name);
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
    if (!resolved->is_static) {
        PtnValue closure_this;
        if (ptn_symbols_get(&resolved->captures, "this", &closure_this)) {
            runtime->has_current_receiver = 1;
            runtime->current_receiver = ptn_value_deref(closure_this);
        }
    }
    for (size_t i = 0; i < resolved->captures.len; i++) {
        PtnSymbol *capture = &resolved->captures.items[i];
        if (capture->value.type == PTN_REFERENCE) {
            ptn_symbols_bind_reference(&runtime->symbols, capture->name, capture->value);
        } else {
            ptn_symbols_set(&runtime->symbols, capture->name, capture->value);
        }
    }
}

static PTN_UNUSED void ptn_generator_adopt_pending_assignment_capture(PtnRuntime *runtime, PtnValue generator) {
    if (runtime == NULL || runtime->pending_generator_assignment_name == NULL) {
        return;
    }
    const char *name = runtime->pending_generator_assignment_name;
    runtime->pending_generator_assignment_name = NULL;
    if (!runtime->owned_call_frame.has_current_closure) {
        return;
    }
    PtnValue closure_value = ptn_value_deref(runtime->owned_call_frame.current_closure);
    if (closure_value.type != PTN_CLOSURE) {
        return;
    }
    PtnValue capture;
    if (
        ptn_symbols_get(&closure_value.as.closure->captures, name, &capture) &&
        capture.type == PTN_REFERENCE
    ) {
        ptn_reference_assign(runtime, capture.as.reference, generator);
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
        copy.as.closure->suppress_wrapped_callable_deprecation =
            source->suppress_wrapped_callable_deprecation;
        copy.as.closure->wrapped_callable = ptn_value_clone(source->wrapped_callable);
    }
    if (source->bound_scope_name != NULL) {
        copy.as.closure->bound_scope_name = ptn_duplicate_string(source->bound_scope_name);
    }
    copy.as.closure->origin_kind = source->origin_kind;
    if (source->origin_class_name != NULL) {
        copy.as.closure->origin_class_name = ptn_duplicate_string(source->origin_class_name);
    }
    if (source->origin_method_name != NULL) {
        copy.as.closure->origin_method_name = ptn_duplicate_string(source->origin_method_name);
    }
    return copy;
}

static const char *ptn_closure_symbol_name_without_leading_slash(const char *name) {
    while (name != NULL && *name == '\\') {
        name++;
    }
    return name == NULL ? "" : name;
}

static int ptn_closure_scope_name_is_relative(const char *name) {
    return ptn_ascii_case_equal(name, "self")
        || ptn_ascii_case_equal(name, "static")
        || ptn_ascii_case_equal(name, "parent");
}

static char *ptn_closure_relative_called_class_name(PtnRuntime *runtime, const char *scope_name) {
    if (runtime == NULL || !ptn_closure_scope_name_is_relative(scope_name)) {
        return NULL;
    }
    const char *class_name = runtime->current_class_name;
    if (!ptn_ascii_case_equal(scope_name, "self") && runtime->current_called_class_name != NULL) {
        class_name = runtime->current_called_class_name;
    }
    return class_name == NULL ? NULL : ptn_duplicate_string(class_name);
}

static char *ptn_closure_relative_scope_class_name(PtnRuntime *runtime, const char *scope_name) {
    if (runtime == NULL || !ptn_closure_scope_name_is_relative(scope_name)) {
        return NULL;
    }
    return runtime->current_class_name == NULL
        ? NULL
        : ptn_duplicate_string(runtime->current_class_name);
}

static PtnArrayEntry *ptn_closure_array_entry_for_int_key(PtnArray *array, int64_t key) {
    if (array == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        if (entry->key.type == PTN_ARRAY_KEY_INT && entry->key.as.integer == key) {
            return entry;
        }
    }
    return NULL;
}

static PTN_UNUSED char *ptn_closure_wrapped_callable_called_class_name(
    PtnRuntime *runtime,
    PtnValue callable
) {
    PtnValue resolved = ptn_value_deref(callable);
    if (resolved.type == PTN_CLOSURE) {
        return resolved.as.closure->called_class_name == NULL
            ? NULL
            : ptn_duplicate_string(resolved.as.closure->called_class_name);
    }
    if (resolved.type == PTN_OBJECT) {
        return ptn_duplicate_string(resolved.as.object->class_name);
    }
    if (resolved.type == PTN_EXCEPTION) {
        return ptn_duplicate_string(resolved.as.exception->class_name);
    }
    if (resolved.type == PTN_STRING) {
        char *name = ptn_value_to_string(resolved);
        char *separator = strstr(name, "::");
        if (separator == NULL || separator == name || separator[2] == '\0') {
            free(name);
            return NULL;
        }
        *separator = '\0';
        char *relative_class_name = ptn_closure_relative_called_class_name(runtime, name);
        if (relative_class_name != NULL) {
            free(name);
            return relative_class_name;
        }
        const char *lookup_name = ptn_closure_symbol_name_without_leading_slash(name);
        const char *resolved_name = runtime == NULL
            ? lookup_name
            : ptn_runtime_resolve_class_alias(runtime, lookup_name);
        char *class_name = ptn_duplicate_string(ptn_declared_class_canonical_name(resolved_name));
        free(name);
        return class_name;
    }
    if (resolved.type != PTN_ARRAY || resolved.as.array == NULL) {
        return NULL;
    }
    PtnArrayEntry *scope_entry = ptn_closure_array_entry_for_int_key(resolved.as.array, 0);
    PtnArrayEntry *method_entry = ptn_closure_array_entry_for_int_key(resolved.as.array, 1);
    if (scope_entry == NULL || method_entry == NULL) {
        return NULL;
    }
    PtnValue scope = ptn_value_deref(scope_entry->value);
    PtnValue method = ptn_value_deref(method_entry->value);
    if (method.type != PTN_STRING) {
        return NULL;
    }
    if (scope.type == PTN_OBJECT) {
        return ptn_duplicate_string(scope.as.object->class_name);
    }
    if (scope.type == PTN_EXCEPTION) {
        return ptn_duplicate_string(scope.as.exception->class_name);
    }
    if (scope.type == PTN_CLOSURE) {
        return scope.as.closure->called_class_name == NULL
            ? NULL
            : ptn_duplicate_string(scope.as.closure->called_class_name);
    }
    if (scope.type != PTN_STRING) {
        return NULL;
    }
    char *scope_name = ptn_value_to_string(scope);
    char *relative_class_name = ptn_closure_relative_called_class_name(runtime, scope_name);
    if (relative_class_name != NULL) {
        free(scope_name);
        return relative_class_name;
    }
    const char *lookup_name = ptn_closure_symbol_name_without_leading_slash(scope_name);
    const char *resolved_name = runtime == NULL
        ? lookup_name
        : ptn_runtime_resolve_class_alias(runtime, lookup_name);
    char *class_name = ptn_duplicate_string(ptn_declared_class_canonical_name(resolved_name));
    free(scope_name);
    return class_name;
}

static PTN_UNUSED char *ptn_closure_declaring_method_scope_name(
    PtnRuntime *runtime,
    const char *class_name,
    const char *method_name
) {
    if (class_name == NULL || method_name == NULL || method_name[0] == '\0') {
        return NULL;
    }
    const char *declaring_class = NULL;
    int visibility = PTN_PROPERTY_PUBLIC;
    int is_abstract = 0;
    if (
        runtime != NULL &&
        runtime->declared_method_visibility_metadata != NULL &&
        runtime->declared_method_visibility_metadata(
            class_name,
            method_name,
            &declaring_class,
            &visibility,
            &is_abstract
        )
    ) {
        (void)visibility;
        (void)is_abstract;
        return declaring_class == NULL ? NULL : ptn_duplicate_string(declaring_class);
    }
    return ptn_duplicate_string(class_name);
}

static PTN_UNUSED char *ptn_closure_wrapped_callable_scope_class_name(
    PtnRuntime *runtime,
    PtnValue callable
) {
    PtnValue resolved = ptn_value_deref(callable);
    if (resolved.type == PTN_CLOSURE) {
        return resolved.as.closure->scope_class_name == NULL
            ? NULL
            : ptn_duplicate_string(resolved.as.closure->scope_class_name);
    }
    if (resolved.type == PTN_OBJECT) {
        return ptn_closure_declaring_method_scope_name(
            runtime,
            resolved.as.object->class_name,
            "__invoke"
        );
    }
    if (resolved.type == PTN_EXCEPTION) {
        return ptn_closure_declaring_method_scope_name(
            runtime,
            resolved.as.exception->class_name,
            "__invoke"
        );
    }
    if (resolved.type == PTN_STRING) {
        char *name = ptn_value_to_string(resolved);
        char *separator = strstr(name, "::");
        if (separator == NULL || separator == name || separator[2] == '\0') {
            free(name);
            return NULL;
        }
        *separator = '\0';
        char *relative_scope_name = ptn_closure_relative_scope_class_name(runtime, name);
        if (relative_scope_name != NULL) {
            free(name);
            return relative_scope_name;
        }
        const char *lookup_name = ptn_closure_symbol_name_without_leading_slash(name);
        const char *resolved_name = runtime == NULL
            ? lookup_name
            : ptn_runtime_resolve_class_alias(runtime, lookup_name);
        const char *class_name = ptn_declared_class_canonical_name(resolved_name);
        char *scope_name =
            ptn_closure_declaring_method_scope_name(runtime, class_name, separator + 2);
        free(name);
        return scope_name;
    }
    if (resolved.type != PTN_ARRAY || resolved.as.array == NULL) {
        return NULL;
    }
    PtnArrayEntry *scope_entry = ptn_closure_array_entry_for_int_key(resolved.as.array, 0);
    PtnArrayEntry *method_entry = ptn_closure_array_entry_for_int_key(resolved.as.array, 1);
    if (scope_entry == NULL || method_entry == NULL) {
        return NULL;
    }
    PtnValue scope = ptn_value_deref(scope_entry->value);
    PtnValue method = ptn_value_deref(method_entry->value);
    if (method.type != PTN_STRING) {
        return NULL;
    }
    char *method_name = ptn_value_to_string(method);
    char *scope_name = NULL;
    if (scope.type == PTN_OBJECT) {
        scope_name = ptn_closure_declaring_method_scope_name(
            runtime,
            scope.as.object->class_name,
            method_name
        );
    } else if (scope.type == PTN_EXCEPTION) {
        scope_name = ptn_closure_declaring_method_scope_name(
            runtime,
            scope.as.exception->class_name,
            method_name
        );
    } else if (scope.type == PTN_CLOSURE) {
        scope_name = scope.as.closure->scope_class_name == NULL
            ? NULL
            : ptn_duplicate_string(scope.as.closure->scope_class_name);
    } else if (scope.type == PTN_STRING) {
        char *class_name = ptn_value_to_string(scope);
        char *relative_scope_name = ptn_closure_relative_scope_class_name(runtime, class_name);
        if (relative_scope_name != NULL) {
            free(class_name);
            free(method_name);
            return relative_scope_name;
        }
        const char *lookup_name = ptn_closure_symbol_name_without_leading_slash(class_name);
        const char *resolved_name = runtime == NULL
            ? lookup_name
            : ptn_runtime_resolve_class_alias(runtime, lookup_name);
        const char *canonical_class_name = ptn_declared_class_canonical_name(resolved_name);
        scope_name = ptn_closure_declaring_method_scope_name(
            runtime,
            canonical_class_name,
            method_name
        );
        free(class_name);
    }
    free(method_name);
    return scope_name;
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
    if (runtime->has_current_receiver) {
        ptn_closure_set_capture(closure, "this", runtime->current_receiver);
    }
    char *called_class_name = ptn_closure_wrapped_callable_called_class_name(runtime, callable);
    char *scope_class_name = ptn_closure_wrapped_callable_scope_class_name(runtime, callable);
    ptn_closure_set_scope(closure, scope_class_name, called_class_name);
    free(scope_class_name);
    if (called_class_name != NULL) {
        free(called_class_name);
    }
    closure.as.closure->has_wrapped_callable = 1;
    closure.as.closure->wrapped_callable = ptn_value_clone_deref(callable);
    PtnValue resolved = ptn_value_deref(callable);
    if (resolved.type == PTN_STRING) {
        char *name = ptn_value_to_string(resolved);
        char *separator = strstr(name, "::");
        if (separator != NULL && separator != name && separator[2] != '\0') {
            *separator = '\0';
            const char *lookup_name = ptn_closure_symbol_name_without_leading_slash(name);
            const char *resolved_name = runtime == NULL
                ? lookup_name
                : ptn_runtime_resolve_class_alias(runtime, lookup_name);
            ptn_closure_set_origin(
                closure,
                PTN_CLOSURE_ORIGIN_STATIC_METHOD,
                ptn_declared_class_canonical_name(resolved_name),
                separator + 2
            );
        } else {
            ptn_closure_set_origin(closure, PTN_CLOSURE_ORIGIN_FUNCTION, NULL, name);
        }
        free(name);
    } else if (resolved.type == PTN_ARRAY && resolved.as.array != NULL) {
        PtnArrayEntry *scope_entry = ptn_closure_array_entry_for_int_key(resolved.as.array, 0);
        PtnArrayEntry *method_entry = ptn_closure_array_entry_for_int_key(resolved.as.array, 1);
        if (scope_entry != NULL && method_entry != NULL) {
            PtnValue scope = ptn_value_deref(scope_entry->value);
            PtnValue method = ptn_value_deref(method_entry->value);
            if (method.type == PTN_STRING) {
                char *method_name = ptn_value_to_string(method);
                if (scope.type == PTN_OBJECT || scope.type == PTN_EXCEPTION) {
                    const char *class_name = scope.type == PTN_OBJECT
                        ? scope.as.object->class_name
                        : scope.as.exception->class_name;
                    ptn_closure_set_origin(
                        closure,
                        PTN_CLOSURE_ORIGIN_METHOD,
                        class_name,
                        method_name
                    );
                } else if (scope.type == PTN_STRING) {
                    char *scope_name = ptn_value_to_string(scope);
                    const char *lookup_name = ptn_closure_symbol_name_without_leading_slash(scope_name);
                    const char *resolved_name = runtime == NULL
                        ? lookup_name
                        : ptn_runtime_resolve_class_alias(runtime, lookup_name);
                    ptn_closure_set_origin(
                        closure,
                        PTN_CLOSURE_ORIGIN_STATIC_METHOD,
                        ptn_declared_class_canonical_name(resolved_name),
                        method_name
                    );
                    free(scope_name);
                }
                free(method_name);
            }
        }
    } else if (resolved.type == PTN_OBJECT || resolved.type == PTN_EXCEPTION) {
        const char *class_name = resolved.type == PTN_OBJECT
            ? resolved.as.object->class_name
            : resolved.as.exception->class_name;
        ptn_closure_set_origin(
            closure,
            PTN_CLOSURE_ORIGIN_METHOD,
            class_name,
            "__invoke"
        );
    }
    return closure;
}

static PTN_UNUSED void ptn_symbols_unset(PtnSymbolTable *symbols, const char *name) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index >= symbols->len) {
        return;
    }

    char *removed_name = symbols->items[index].name;
    PtnValue removed_value = symbols->items[index].value;
    for (size_t i = index + 1; i < symbols->len; i++) {
        symbols->items[i - 1] = symbols->items[i];
    }
    symbols->len--;
    symbols->mutation_epoch++;
    ptn_symbols_rebuild_index(symbols, symbols->len);
    free(removed_name);
    ptn_value_destroy(&removed_value);
}

static PTN_UNUSED void ptn_symbols_unset_with_runtime_scope(
    PtnSymbolTable *symbols,
    const char *name,
    PtnRuntime *runtime
) {
    size_t index = ptn_symbols_find(symbols, name);
    if (index >= symbols->len) {
        return;
    }

    char *removed_name = symbols->items[index].name;
    PtnValue removed_value = symbols->items[index].value;
    for (size_t i = index + 1; i < symbols->len; i++) {
        symbols->items[i - 1] = symbols->items[i];
    }
    symbols->len--;
    symbols->mutation_epoch++;
    ptn_symbols_rebuild_index(symbols, symbols->len);
    free(removed_name);
    ptn_value_destroy_with_runtime_scope(runtime, &removed_value);
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
        || ptn_builtin_exception_class_name(class_name) != NULL
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        || ptn_internal_class_exists_name(class_name)
#endif
        ;
}

static int ptn_closure_scope_is_internal_class(const char *class_name) {
    if (class_name == NULL || ptn_ascii_case_equal(class_name, "Closure")) {
        return 0;
    }
    return ptn_ascii_case_equal(class_name, "stdClass")
        || ptn_ascii_case_equal(class_name, "Generator")
        || ptn_ascii_case_equal(class_name, "DateTime")
        || ptn_builtin_exception_class_name(class_name) != NULL
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        || ptn_internal_class_exists_name(class_name)
#endif
        ;
}

static void ptn_closure_emit_scope_rebind_warning(
    PtnRuntime *runtime,
    const char *message,
    size_t line
) {
    ptn_emit_warning(&runtime->diagnostics, message, line);
}

static int ptn_closure_bind_scope_matches_origin(PtnClosure *source, const char *scope_class_name) {
    if (source->origin_class_name == NULL) {
        return scope_class_name == NULL || ptn_ascii_case_equal(scope_class_name, "Closure");
    }
    return scope_class_name != NULL &&
        ptn_ascii_case_equal(scope_class_name, source->origin_class_name);
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

    if (
        has_new_scope &&
        ptn_closure_scope_is_internal_class(scope_class_name) &&
        (source->origin_kind == PTN_CLOSURE_ORIGIN_ANONYMOUS ||
         source->origin_kind == PTN_CLOSURE_ORIGIN_FUNCTION)
    ) {
        char warning[192];
        int written = snprintf(
            warning,
            sizeof(warning),
            "Cannot bind closure to scope of internal class %s, this will be an error in PHP 9",
            scope_class_name
        );
        if (written < 0 || (size_t)written >= sizeof(warning)) {
            free(owned_scope);
            ptn_abort_out_of_memory();
        }
        ptn_closure_emit_scope_rebind_warning(runtime, warning, line);
        free(owned_scope);
        return ptn_null();
    }

    if (
        source->origin_kind == PTN_CLOSURE_ORIGIN_FUNCTION &&
        has_new_scope &&
        scope_class_name != NULL &&
        !ptn_ascii_case_equal(scope_class_name, "Closure")
    ) {
        ptn_closure_emit_scope_rebind_warning(
            runtime,
            "Cannot rebind scope of closure created from function, this will be an error in PHP 9",
            line
        );
        free(owned_scope);
        return ptn_null();
    }

    PtnValue existing_this;
    int has_existing_this = ptn_symbols_get(&source->captures, "this", &existing_this);
    if (
        source->origin_kind == PTN_CLOSURE_ORIGIN_METHOD &&
        new_this.type == PTN_NULL &&
        has_existing_this
    ) {
        ptn_closure_emit_scope_rebind_warning(
            runtime,
            "Cannot unbind $this of method, this will be an error in PHP 9",
            line
        );
        free(owned_scope);
        return ptn_null();
    }

    if (
        (source->origin_kind == PTN_CLOSURE_ORIGIN_METHOD ||
         source->origin_kind == PTN_CLOSURE_ORIGIN_STATIC_METHOD) &&
        has_new_scope &&
        !ptn_closure_bind_scope_matches_origin(source, scope_class_name)
    ) {
        ptn_closure_emit_scope_rebind_warning(
            runtime,
            "Cannot rebind scope of closure created from method, this will be an error in PHP 9",
            line
        );
        free(owned_scope);
        return ptn_null();
    }

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
        if (
            source->origin_kind == PTN_CLOSURE_ORIGIN_METHOD &&
            source->origin_class_name != NULL &&
            !ptn_ascii_case_equal(new_this_class_name, source->origin_class_name) &&
            !ptn_declared_class_is_same_or_descendant(new_this_class_name, source->origin_class_name)
        ) {
            char warning[256];
            int written = snprintf(
                warning,
                sizeof(warning),
                "Cannot bind method %s::%s() to object of class %s, this will be an error in PHP 9",
                source->origin_class_name,
                source->origin_method_name == NULL ? "{unknown}" : source->origin_method_name,
                new_this_class_name
            );
            if (written < 0 || (size_t)written >= sizeof(warning)) {
                ptn_value_destroy(&bound);
                free(owned_scope);
                ptn_abort_out_of_memory();
            }
            ptn_closure_emit_scope_rebind_warning(runtime, warning, line);
            ptn_value_destroy(&bound);
            free(owned_scope);
            return ptn_null();
        }
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

static int64_t ptn_normalize_error_reporting(int64_t level) {
    return level & ~((int64_t)PTN_E_STRICT);
}

static void ptn_diagnostics_init(PtnDiagnosticSink *diagnostics, FILE *stream) {
    diagnostics->runtime = NULL;
    diagnostics->stream = stream;
    diagnostics->emitted_deprecation = 0;
    diagnostics->emitted_warning = 0;
    diagnostics->suppressed = 0;
    diagnostics->error_reporting = PTN_E_ALL;
    diagnostics->display_errors = 1;
    diagnostics->html_errors = 0;
    diagnostics->has_error_handler = 0;
    diagnostics->error_handler = ptn_null();
    diagnostics->error_handler_levels = PTN_E_ALL;
    diagnostics->error_handler_call_depth = 0;
    diagnostics->error_handler_stack = NULL;
    diagnostics->error_handler_stack_len = 0;
    diagnostics->error_handler_stack_capacity = 0;
    int64_t configured_error_reporting = 0;
    if (ptn_parse_int64_env("PTN_PHP_ERROR_REPORTING", &configured_error_reporting)) {
        diagnostics->error_reporting = ptn_normalize_error_reporting(configured_error_reporting);
    }
    int configured_display_errors = 1;
    if (ptn_parse_bool_env("PTN_PHP_DISPLAY_ERRORS", &configured_display_errors)) {
        diagnostics->display_errors = configured_display_errors;
    }
    int configured_html_errors = 0;
    if (ptn_parse_bool_env("PTN_PHP_HTML_ERRORS", &configured_html_errors)) {
        diagnostics->html_errors = configured_html_errors;
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

static int ptn_exception_handlers_same_callable(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right
) {
    return ptn_compare_identical(runtime, left, right, 0);
}

static int ptn_exception_handlers_try_uncaught_inner(
    PtnRuntime *runtime,
    PtnException *exception,
    size_t depth
) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (runtime == NULL || runtime->exceptions == NULL || exception == NULL) {
        return 0;
    }
    PtnExceptionState *state = runtime->exceptions;
    if (!state->has_exception_handler || depth > 16 || (depth == 0 && state->in_exception_handler)) {
        return 0;
    }
    PtnValue handler = ptn_value_clone(state->exception_handler);
    int detached_active = 0;
    if (state->active_exception == exception) {
        state->active_exception = NULL;
        detached_active = 1;
    }
    int saved_in_exception_handler = state->in_exception_handler;
    state->in_exception_handler = 1;
    PtnValue arg = ptn_exception_borrow(exception);
    PtnValue result = ptn_null();
    PtnTryFrame handler_frame;
    ptn_try_frame_push(runtime, &handler_frame);
    if (setjmp(handler_frame.jump) == 0) {
        result = ptn_call_callable(runtime, handler, 1, &arg, 0, 0);
        ptn_value_destroy(&result);
    }
    ptn_try_frame_pop(runtime, &handler_frame);
    state->in_exception_handler = saved_in_exception_handler;

    int handled = state->active_exception == NULL;
    if (!handled && state->has_exception_handler && depth < 16) {
        PtnValue current = ptn_value_clone(state->exception_handler);
        int same_handler = ptn_exception_handlers_same_callable(runtime, handler, current);
        ptn_value_destroy(&current);
        if (!same_handler) {
            handled = ptn_exception_handlers_try_uncaught_inner(
                runtime,
                state->active_exception,
                depth + 1
            );
        }
    }
    if (detached_active && state->active_exception != exception) {
        ptn_exception_free(exception);
    }
    ptn_value_destroy(&handler);
    return handled;
#else
    (void)runtime;
    (void)exception;
    (void)depth;
    return 0;
#endif
}

static PTN_UNUSED int ptn_exception_handlers_try_uncaught(
    PtnRuntime *runtime,
    PtnException *exception
) {
    return ptn_exception_handlers_try_uncaught_inner(runtime, exception, 0);
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
        PtnRuntime *root = ptn_runtime_root(diagnostics->runtime);
        PtnDiagnosticSink *root_diagnostics = root == NULL
            ? &diagnostics->runtime->diagnostics
            : &root->diagnostics;
        if (root_diagnostics->error_handler_call_depth > 0) {
            fwrite(data, 1, len, stdout);
            if (root != NULL) {
                root->output_has_started = 1;
                root->output_at_line_start = data[len - 1] == '\n';
            }
            return;
        }
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

static int ptn_diagnostic_utf8_continuation(unsigned char byte) {
    return byte >= 0x80 && byte <= 0xbf;
}

static int ptn_diagnostic_utf8_sequence_len(const unsigned char *data, size_t len, size_t *sequence_len) {
    if (len == 0) {
        return 0;
    }
    unsigned char first = data[0];
    if (first < 0x80) {
        *sequence_len = 1;
        return 1;
    }
    if (first >= 0xc2 && first <= 0xdf) {
        if (len < 2 || !ptn_diagnostic_utf8_continuation(data[1])) {
            return 0;
        }
        *sequence_len = 2;
        return 1;
    }
    if (first == 0xe0) {
        if (len < 3 || data[1] < 0xa0 || data[1] > 0xbf || !ptn_diagnostic_utf8_continuation(data[2])) {
            return 0;
        }
        *sequence_len = 3;
        return 1;
    }
    if ((first >= 0xe1 && first <= 0xec) || (first >= 0xee && first <= 0xef)) {
        if (
            len < 3 ||
            !ptn_diagnostic_utf8_continuation(data[1]) ||
            !ptn_diagnostic_utf8_continuation(data[2])
        ) {
            return 0;
        }
        *sequence_len = 3;
        return 1;
    }
    if (first == 0xed) {
        if (len < 3 || data[1] < 0x80 || data[1] > 0x9f || !ptn_diagnostic_utf8_continuation(data[2])) {
            return 0;
        }
        *sequence_len = 3;
        return 1;
    }
    if (first == 0xf0) {
        if (
            len < 4 ||
            data[1] < 0x90 ||
            data[1] > 0xbf ||
            !ptn_diagnostic_utf8_continuation(data[2]) ||
            !ptn_diagnostic_utf8_continuation(data[3])
        ) {
            return 0;
        }
        *sequence_len = 4;
        return 1;
    }
    if (first >= 0xf1 && first <= 0xf3) {
        if (
            len < 4 ||
            !ptn_diagnostic_utf8_continuation(data[1]) ||
            !ptn_diagnostic_utf8_continuation(data[2]) ||
            !ptn_diagnostic_utf8_continuation(data[3])
        ) {
            return 0;
        }
        *sequence_len = 4;
        return 1;
    }
    if (first == 0xf4) {
        if (
            len < 4 ||
            data[1] < 0x80 ||
            data[1] > 0x8f ||
            !ptn_diagnostic_utf8_continuation(data[2]) ||
            !ptn_diagnostic_utf8_continuation(data[3])
        ) {
            return 0;
        }
        *sequence_len = 4;
        return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_diagnostic_output_html_text(PtnDiagnosticSink *diagnostics, const char *data) {
    if (data == NULL) {
        return;
    }
    const unsigned char *bytes = (const unsigned char *)data;
    size_t len = strlen(data);
    size_t offset = 0;
    while (offset < len) {
        size_t sequence_len = 0;
        if (ptn_diagnostic_utf8_sequence_len(bytes + offset, len - offset, &sequence_len)) {
            ptn_diagnostic_output_write(diagnostics, data + offset, sequence_len);
            offset += sequence_len;
            continue;
        }
        ptn_diagnostic_output_write(diagnostics, "\xef\xbf\xbd", 3);
        offset++;
    }
}

static PTN_UNUSED void ptn_diagnostic_emit_html_message(
    PtnDiagnosticSink *diagnostics,
    const char *label,
    const char *message,
    const char *path,
    size_t line
) {
    ptn_diagnostic_printf(diagnostics, "<br />\n<b>%s</b>:  ", label);
    ptn_diagnostic_output_html_text(diagnostics, message);
    ptn_diagnostic_printf(
        diagnostics,
        " in <b>%s</b> on line <b>%zu</b><br />\n",
        path,
        line
    );
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
    if (line == 0) {
        return "Unknown";
    }
    const char *script_filename = getenv("PTN_SCRIPT_FILENAME");
    if (script_filename != NULL && script_filename[0] != '\0') {
        return script_filename;
    }
    return "ptn";
}

static PTN_UNUSED const char *ptn_diagnostic_html_path(PtnDiagnosticSink *diagnostics, const char *path, size_t line) {
    if (path != NULL) {
        return path;
    }
    if (line == 0) {
        return "Unknown";
    }
    return ptn_diagnostic_path(diagnostics, NULL);
}

static PTN_UNUSED const char *ptn_runtime_source_path_or(PtnRuntime *runtime, const char *fallback) {
    if (runtime != NULL && runtime->source_path != NULL && runtime->source_path[0] != '\0') {
        return runtime->source_path;
    }
    return fallback == NULL ? "" : fallback;
}

static PTN_UNUSED int ptn_diagnostics_try_error_handler_with_frame(
    PtnDiagnosticSink *diagnostics,
    int64_t severity,
    const char *message,
    const char *path,
    size_t line,
    int suppress_user_call_frame_location
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
    PtnValue call_args[4] = {
        ptn_value_borrow(args[0]),
        ptn_value_borrow(args[1]),
        ptn_value_borrow(args[2]),
        ptn_value_borrow(args[3]),
    };
    PtnValue saved_handler = ptn_value_clone(handler_diagnostics->error_handler);
    int64_t saved_handler_levels = handler_diagnostics->error_handler_levels;
    ptn_diagnostics_clear_current_error_handler(handler_diagnostics);
    PtnValue result = ptn_null();
    PtnTryFrame handler_frame;
    PtnTraceFrame *saved_trace_frame = runtime->trace_frame;
    int saved_suppress_user_call_frame_location =
        runtime->suppress_user_call_frame_location;
    int saved_warn_by_ref_argument_mismatch = runtime->warn_by_ref_argument_mismatch;
    int saved_throw_argument_count_errors = runtime->throw_argument_count_errors;
    ptn_try_frame_push(runtime, &handler_frame);
    if (setjmp(handler_frame.jump) != 0) {
        ptn_try_frame_pop(runtime, &handler_frame);
        if (handler_diagnostics->error_handler_call_depth > 0) {
            handler_diagnostics->error_handler_call_depth--;
        }
        runtime->trace_frame = saved_trace_frame;
        runtime->suppress_user_call_frame_location =
            saved_suppress_user_call_frame_location;
        runtime->warn_by_ref_argument_mismatch = saved_warn_by_ref_argument_mismatch;
        runtime->throw_argument_count_errors = saved_throw_argument_count_errors;
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
        ptn_rethrow_exception(runtime);
        return 1;
    }
    if (suppress_user_call_frame_location) {
        runtime->suppress_user_call_frame_location = 1;
    }
    handler_diagnostics->error_handler_call_depth++;
    result = ptn_call_callable(runtime, saved_handler, 4, call_args, line, 0);
    handler_diagnostics->error_handler_call_depth--;
    ptn_try_frame_pop(runtime, &handler_frame);
    runtime->trace_frame = saved_trace_frame;
    runtime->suppress_user_call_frame_location =
        saved_suppress_user_call_frame_location;
    runtime->warn_by_ref_argument_mismatch = saved_warn_by_ref_argument_mismatch;
    runtime->throw_argument_count_errors = saved_throw_argument_count_errors;
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
        ptn_value_destroy(&result);
        ptn_rethrow_exception(runtime);
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

static PTN_UNUSED int ptn_diagnostics_try_error_handler(
    PtnDiagnosticSink *diagnostics,
    int64_t severity,
    const char *message,
    const char *path,
    size_t line
) {
    return ptn_diagnostics_try_error_handler_with_frame(
        diagnostics,
        severity,
        message,
        path,
        line,
        0
    );
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

static void ptn_emit_undefined_global_variable_warning(
    PtnDiagnosticSink *diagnostics,
    const char *name,
    const char *path,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    diagnostics->emitted_warning = 1;
    int needed = snprintf(NULL, 0, "Undefined global variable $%s", name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(message, (size_t)needed + 1, "Undefined global variable $%s", name);
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, path, line)) {
        free(message);
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nWarning: Undefined global variable $%s in %s on line %zu\n",
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
        PtnRuntime *root = ptn_runtime_root(runtime);
        if (root != NULL && root->output_has_started) {
            fputc('\n', stream);
        }
        fputs("Fatal error: ", stream);
        fputs(message, stream);
        fputs(" in ", stream);
        fputs(path != NULL ? path : (runtime->source_path != NULL ? runtime->source_path : "ptn"), stream);
        fputs(" on line ", stream);
        fprintf(stream, "%zu", line);
        fputc('\n', stream);
    }
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED void ptn_emit_fatal_error_bytes_at(
    PtnRuntime *runtime,
    const char *message,
    size_t message_len,
    const char *path,
    size_t line
) {
    fflush(stdout);
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (diagnostics->display_errors) {
        FILE *stream = diagnostics->stream == NULL ? stderr : diagnostics->stream;
        PtnRuntime *root = ptn_runtime_root(runtime);
        if (root != NULL && root->output_has_started) {
            fputc('\n', stream);
        }
        fputs("Fatal error: ", stream);
        fwrite(message, 1, message_len, stream);
        fputs(" in ", stream);
        fputs(path != NULL ? path : (runtime->source_path != NULL ? runtime->source_path : "ptn"), stream);
        fputs(" on line ", stream);
        fprintf(stream, "%zu", line);
        fputc('\n', stream);
    }
    ptn_runtime_shutdown_before_exit(runtime);
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

static PTN_UNUSED void ptn_emit_deprecation_with_handler_frame(
    PtnDiagnosticSink *diagnostics,
    const char *message,
    size_t line,
    int suppress_user_call_frame_location
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    diagnostics->emitted_deprecation = 1;
    if (ptn_diagnostics_try_error_handler_with_frame(
        diagnostics,
        PTN_E_DEPRECATED,
        message,
        NULL,
        line,
        suppress_user_call_frame_location
    )) {
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

static PTN_UNUSED void ptn_emit_user_deprecation_len(PtnDiagnosticSink *diagnostics, const char *message, size_t message_len, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_USER_DEPRECATED)) {
        return;
    }
    diagnostics->emitted_deprecation = 1;
    if (memchr(message, '\0', message_len) == NULL &&
        ptn_diagnostics_try_error_handler(diagnostics, PTN_E_USER_DEPRECATED, message, NULL, line)) {
        return;
    }
    ptn_diagnostic_output_cstr(diagnostics, "\nDeprecated: ");
    ptn_diagnostic_output_write(diagnostics, message, message_len);
    ptn_diagnostic_printf(diagnostics, " in ptn on line %zu\n", line);
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

static PTN_UNUSED void ptn_emit_warning_with_handler_frame_and_newline(
    PtnDiagnosticSink *diagnostics,
    const char *message,
    size_t line,
    int suppress_user_call_frame_location,
    int leading_newline
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler_with_frame(
        diagnostics,
        PTN_E_WARNING,
        message,
        NULL,
        line,
        suppress_user_call_frame_location
    )) {
        return;
    }
    if (diagnostics->html_errors) {
        ptn_diagnostic_emit_html_message(
            diagnostics,
            "Warning",
            message,
            ptn_diagnostic_html_path(diagnostics, NULL, line),
            line
        );
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        leading_newline ? "\nWarning: %s in %s on line %zu\n" : "Warning: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_warning_with_handler_frame(
    PtnDiagnosticSink *diagnostics,
    const char *message,
    size_t line,
    int suppress_user_call_frame_location
) {
    ptn_emit_warning_with_handler_frame_and_newline(
        diagnostics,
        message,
        line,
        suppress_user_call_frame_location,
        1
    );
}

static PTN_UNUSED void ptn_emit_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    ptn_emit_warning_with_handler_frame(diagnostics, message, line, 0);
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

static PTN_UNUSED void ptn_emit_notice_with_path(
    PtnDiagnosticSink *diagnostics,
    const char *message,
    const char *path,
    size_t line,
    int leading_newline
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_NOTICE, message, path, line)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        leading_newline ? "\nNotice: %s in %s on line %zu\n" : "Notice: %s in %s on line %zu\n",
        message,
        path != NULL ? path : ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_notice_with_handler_frame(
    PtnDiagnosticSink *diagnostics,
    const char *message,
    size_t line,
    int suppress_user_call_frame_location,
    int leading_newline
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_NOTICE)) {
        return;
    }
    if (ptn_diagnostics_try_error_handler_with_frame(
        diagnostics,
        PTN_E_NOTICE,
        message,
        NULL,
        line,
        suppress_user_call_frame_location
    )) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        leading_newline ? "\nNotice: %s in %s on line %zu\n" : "Notice: %s in %s on line %zu\n",
        message,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED void ptn_emit_notice(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    ptn_emit_notice_with_path(diagnostics, message, NULL, line, 0);
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
    if (diagnostics->html_errors) {
        ptn_diagnostic_emit_html_message(
            diagnostics,
            "Warning",
            message,
            ptn_diagnostic_html_path(diagnostics, runtime->source_path, line),
            line
        );
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

static PTN_UNUSED void ptn_emit_compile_warning_direct(PtnRuntime *runtime, const char *message, const char *path, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
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

static PTN_UNUSED void ptn_emit_compile_warning(PtnRuntime *runtime, const char *message, const char *path, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, path, line)) {
        diagnostics->emitted_warning = 1;
        return;
    }
    ptn_emit_compile_warning_direct(runtime, message, path, line);
}

static PTN_UNUSED void ptn_emit_compile_deprecation_direct(PtnRuntime *runtime, const char *message, const char *path, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
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

static PTN_UNUSED void ptn_emit_compile_deprecation(PtnRuntime *runtime, const char *message, const char *path, size_t line) {
    PtnDiagnosticSink *diagnostics = &runtime->diagnostics;
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_DEPRECATED, message, path, line)) {
        diagnostics->emitted_deprecation = 1;
        return;
    }
    ptn_emit_compile_deprecation_direct(runtime, message, path, line);
}

static PTN_UNUSED void ptn_emit_spaced_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, NULL, line)) {
        return;
    }
    if (diagnostics->html_errors) {
        ptn_diagnostic_emit_html_message(
            diagnostics,
            "Warning",
            message,
            ptn_diagnostic_html_path(diagnostics, NULL, line),
            line
        );
        return;
    }
    ptn_diagnostic_printf(diagnostics, "\nWarning: %s in ptn on line %zu\n", message, line);
}

static PTN_UNUSED void ptn_emit_only_variables_assigned_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    ptn_emit_notice_with_path(
        diagnostics,
        "Only variables should be assigned by reference",
        NULL,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_only_variables_assigned_by_reference_notice_at(PtnRuntime *runtime, size_t line) {
    ptn_emit_notice_with_path(
        &runtime->diagnostics,
        "Only variables should be assigned by reference",
        runtime->source_path,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_attempting_to_set_reference_to_non_referenceable_value_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    ptn_emit_notice_with_path(
        diagnostics,
        "Attempting to set reference to non referenceable value",
        NULL,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_only_variables_passed_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    ptn_emit_notice_with_path(
        diagnostics,
        "Only variables should be passed by reference",
        NULL,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_only_variables_passed_by_reference_notice_at(PtnRuntime *runtime, size_t line) {
    ptn_emit_notice_with_path(
        &runtime->diagnostics,
        "Only variables should be passed by reference",
        runtime->source_path,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_only_variable_references_returned_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    ptn_emit_notice_with_path(
        diagnostics,
        "Only variable references should be returned by reference",
        NULL,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_only_variable_references_returned_by_reference_notice_at(PtnRuntime *runtime, size_t line) {
    ptn_emit_notice_with_path(
        &runtime->diagnostics,
        "Only variable references should be returned by reference",
        runtime->source_path,
        line,
        1
    );
}

static PTN_UNUSED void ptn_emit_only_variable_references_yielded_by_reference_notice(PtnDiagnosticSink *diagnostics, size_t line) {
    ptn_emit_notice_with_path(
        diagnostics,
        "Only variable references should be yielded by reference",
        NULL,
        line,
        1
    );
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
    size_t message_len = strlen(name) + strlen("Constant  already defined, this will be an error in PHP 9") + 1;
    char *message = malloc(message_len);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(message, message_len, "Constant %s already defined, this will be an error in PHP 9", name);
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, NULL, line)) {
        free(message);
        return;
    }
    if (diagnostics->runtime != NULL && diagnostics->runtime->output_has_started) {
        fputc('\n', stdout);
    }
    fputs("Warning: Constant ", stdout);
    fputs(name, stdout);
    fputs(" already defined, this will be an error in PHP 9 in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
    free(message);
}

static PTN_UNUSED void ptn_emit_define_case_insensitive_ignored_warning(
    PtnDiagnosticSink *diagnostics,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    const char *message =
        "define(): Argument #3 ($case_insensitive) is ignored since declaration of case-insensitive constants is no longer supported";
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, NULL, line)) {
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
    ptn_symbols_init(&runtime->owned_constant_sources);
    runtime->constant_sources = &runtime->owned_constant_sources;
    ptn_symbols_init(&runtime->owned_class_aliases);
    runtime->class_aliases = &runtime->owned_class_aliases;
    ptn_symbols_init(&runtime->owned_dynamic_classes);
    runtime->dynamic_classes = &runtime->owned_dynamic_classes;
    ptn_symbols_init(&runtime->owned_class_constants);
    runtime->class_constants = &runtime->owned_class_constants;
    ptn_symbols_init(&runtime->owned_class_constant_deprecations);
    runtime->class_constant_deprecations = &runtime->owned_class_constant_deprecations;
    ptn_symbols_init(&runtime->owned_class_constant_initializing);
    runtime->class_constant_initializing = &runtime->owned_class_constant_initializing;
    runtime->current_class_constant_initializing_class_name = NULL;
    runtime->current_class_constant_initializing_key_class_name = NULL;
    runtime->current_class_constant_initializing_constant_name = NULL;
    runtime->current_class_constant_source_path = NULL;
    runtime->class_constant_deprecation_suppress_class = NULL;
    runtime->class_constant_deprecation_suppress_constant = NULL;
    runtime->dynamic_property_deprecation_suppress_object = NULL;
    runtime->dynamic_property_deprecation_suppress_property = NULL;
    ptn_symbols_init(&runtime->owned_static_properties);
    runtime->static_properties = &runtime->owned_static_properties;
    ptn_symbols_init(&runtime->owned_static_property_initialized);
    runtime->static_property_initialized = &runtime->owned_static_property_initialized;
    ptn_symbols_init(&runtime->owned_static_property_read_visibility);
    runtime->static_property_read_visibility = &runtime->owned_static_property_read_visibility;
    ptn_symbols_init(&runtime->owned_static_property_set_visibility);
    runtime->static_property_set_visibility = &runtime->owned_static_property_set_visibility;
    ptn_symbols_init(&runtime->owned_static_property_type_kind);
    runtime->static_property_type_kind = &runtime->owned_static_property_type_kind;
    ptn_symbols_init(&runtime->owned_static_property_type_class_name);
    runtime->static_property_type_class_name = &runtime->owned_static_property_type_class_name;
    ptn_symbols_init(&runtime->owned_static_property_type_text);
    runtime->static_property_type_text = &runtime->owned_static_property_type_text;
    ptn_symbols_init(&runtime->owned_static_property_type_allows_null);
    runtime->static_property_type_allows_null = &runtime->owned_static_property_type_allows_null;
    ptn_diagnostics_init(&runtime->diagnostics, NULL);
    runtime->diagnostics.runtime = runtime;
    if (getenv("PTN_STARTUP_WARNING_EMITTED") != NULL) {
        runtime->diagnostics.emitted_warning = 1;
    }
    runtime->date_timezone_startup_warning_emitted = 0;
    runtime->owned_exceptions.active_exception = NULL;
    runtime->owned_exceptions.try_frame = NULL;
    ptn_exception_handlers_init(&runtime->owned_exceptions);
    runtime->exceptions = &runtime->owned_exceptions;
    runtime->owned_call_frame.argc = 0;
    runtime->owned_call_frame.args = NULL;
    runtime->owned_call_frame.arg_names = NULL;
    runtime->owned_call_frame.parameter_count = 0;
    runtime->owned_call_frame.parameter_names = NULL;
    runtime->owned_call_frame.has_current_closure = 0;
    runtime->owned_call_frame.current_closure = ptn_null();
    runtime->call_frame = NULL;
    runtime->next_call_arg_names = NULL;
    runtime->owned_trace_frame.runtime = NULL;
    runtime->owned_trace_frame.function_name = NULL;
    runtime->owned_trace_frame.file = NULL;
    runtime->owned_trace_frame.line = 0;
    runtime->owned_trace_frame.argc = 0;
    runtime->owned_trace_frame.args = NULL;
    runtime->owned_trace_frame.arg_names = NULL;
    runtime->owned_trace_frame.parameter_count = 0;
    runtime->owned_trace_frame.parameter_names = NULL;
    runtime->owned_trace_frame.sensitive_parameter_count = 0;
    runtime->owned_trace_frame.sensitive_parameters = NULL;
    runtime->owned_trace_frame.sensitive_variadic_position = (size_t)-1;
    runtime->owned_trace_frame.has_receiver = 0;
    runtime->owned_trace_frame.receiver = ptn_null();
    runtime->owned_trace_frame.previous = NULL;
    runtime->trace_frame = NULL;
    runtime->lifecycle_root = runtime;
    runtime->live_objects = NULL;
    runtime->live_objects_len = 0;
    runtime->live_objects_capacity = 0;
    runtime->live_closures = NULL;
    runtime->live_closures_len = 0;
    runtime->live_closures_capacity = 0;
    runtime->first_class_callable_cache_values = NULL;
    runtime->first_class_callable_cache_names = NULL;
    runtime->first_class_callable_cache_len = 0;
    runtime->first_class_callable_cache_capacity = 0;
    runtime->live_arrays = NULL;
    runtime->live_arrays_len = 0;
    runtime->live_arrays_capacity = 0;
    runtime->live_references = NULL;
    runtime->live_references_len = 0;
    runtime->live_references_capacity = 0;
    runtime->temporary_roots = NULL;
    runtime->temporary_roots_len = 0;
    runtime->temporary_roots_capacity = 0;
    runtime->static_local_slots = NULL;
    runtime->static_local_slots_len = 0;
    runtime->static_local_slots_capacity = 0;
    runtime->next_object_id = 1;
    runtime->free_object_ids = NULL;
    runtime->free_object_ids_len = 0;
    runtime->free_object_ids_capacity = 0;
    runtime->deferred_free_object_id = 0;
    runtime->has_deferred_free_object_id = 0;
    runtime->output_buffers = NULL;
    runtime->output_buffers_len = 0;
    runtime->output_buffers_capacity = 0;
    runtime->output_buffer_callback_depth = 0;
    runtime->output_buffer_callback_function_name = NULL;
    runtime->output_buffer_callback_handler_name = NULL;
    runtime->output_buffer_callback_line = 0;
    runtime->output_buffer_callback_output_warned = 0;
    runtime->output_buffer_callback_passthrough_output = 0;
    runtime->output_buffer_callback_skip_buffers = 0;
    runtime->output_at_line_start = 1;
    runtime->output_has_started = 0;
    runtime->http_response_code_initialized = 0;
    runtime->http_response_code = 0;
    runtime->header_callback_registered = 0;
    runtime->header_callback_running = 0;
    runtime->header_callback_completed = 0;
    runtime->header_callback = ptn_null();
    runtime->shutdown_functions = NULL;
    runtime->shutdown_functions_len = 0;
    runtime->shutdown_functions_capacity = 0;
    runtime->shutdown_function_index = 0;
    runtime->shutdown_functions_running = 0;
    runtime->shutdown_functions_completed = 0;
    runtime->shutdown_in_progress = 0;
    runtime->tick_enabled = 0;
    runtime->tick_functions = NULL;
    runtime->tick_functions_len = 0;
    runtime->tick_functions_capacity = 0;
    runtime->tick_functions_running = 0;
    runtime->defer_uncaught_exception_emit = 0;
    runtime->method_dispatch = NULL;
    runtime->reflected_method_dispatch = NULL;
    runtime->declared_method_exists = NULL;
    runtime->declared_method_metadata = NULL;
    runtime->declared_method_visible = NULL;
    runtime->declared_method_visibility_metadata = NULL;
    runtime->class_scope_allows = NULL;
    runtime->declared_class_is_readonly = NULL;
    runtime->declared_class_allows_dynamic_properties = NULL;
    runtime->magic_property_read = NULL;
    runtime->magic_property_isset = NULL;
    runtime->declared_user_functions = NULL;
    runtime->declared_user_classes = NULL;
    runtime->declared_user_traits = NULL;
    runtime->magic_property_get = NULL;
    runtime->magic_property_get_exists = NULL;
    runtime->magic_property_set = NULL;
    runtime->magic_property_unset = NULL;
    runtime->magic_debug_info = NULL;
    runtime->property_hook_get = NULL;
    runtime->property_hook_set = NULL;
    runtime->active_property_hook_class = NULL;
    runtime->active_property_hook_property = NULL;
    runtime->active_property_hook_object = NULL;
    runtime->class_constant_initializer = NULL;
    runtime->static_property_initializer = NULL;
    runtime->new_instance_without_constructor = NULL;
    runtime->in_magic_property_dispatch = 0;
    runtime->active_spl_object_storage_get_hash_depth = 0;
    runtime->magic_property_frames = NULL;
    runtime->magic_property_frame_len = 0;
    runtime->magic_property_frame_capacity = 0;
    runtime->source_path = NULL;
    runtime->source_snapshot_data = NULL;
    runtime->source_snapshot_len = 0;
    runtime->compiled_include_depth = 0;
    runtime->in_preload = 0;
    runtime->current_function_name = NULL;
    runtime->current_class_name = NULL;
    runtime->current_called_class_name = NULL;
    runtime->called_class_name_override = NULL;
    runtime->forward_static_called_class_name = NULL;
    runtime->destructor_access_scope = NULL;
    runtime->destructor_shutdown_phase = 0;
    runtime->current_generator = NULL;
    runtime->pending_generator_assignment_name = NULL;
    runtime->pending_yield_from_generator = NULL;
    runtime->pending_yield_from_line = 0;
    runtime->implicit_generator_foreach_rewind = 0;
    runtime->implicit_generator_foreach_source_path = NULL;
    runtime->implicit_generator_foreach_line = 0;
    runtime->generator_aborted_after_yield = 0;
    runtime->generator_aborted_rethrow_on_rewind = 0;
    runtime->generator_chained_exception_during_unwind = 0;
    runtime->suppress_generator_rewind_trace_frame = 0;
    runtime->current_fiber = NULL;
    runtime->has_current_receiver = 0;
    runtime->current_receiver = ptn_null();
    runtime->by_ref_argument_function_name_override = NULL;
    runtime->by_ref_argument_notice_pending = 0;
    runtime->by_ref_argument_notice_emitted = 0;
    runtime->by_ref_argument_notice_line = 0;
    runtime->suppress_scoped_callable_deprecation = 0;
    runtime->include_path = ptn_duplicate_string(".");
    runtime->included_files = NULL;
    runtime->included_files_len = 0;
    runtime->included_files_capacity = 0;
    runtime->autoload_callbacks = NULL;
    runtime->autoload_callback_scope_class_names = NULL;
    runtime->autoload_callback_called_class_names = NULL;
    runtime->autoload_callbacks_len = 0;
    runtime->autoload_callbacks_capacity = 0;
    runtime->spl_autoload_extensions = ptn_duplicate_string(".inc,.php");
    runtime->autoloading_class_names = NULL;
    runtime->autoloading_class_names_len = 0;
    runtime->autoloading_class_names_capacity = 0;
    runtime->last_opened_directory = NULL;
    const char *configured_open_basedir = getenv("PTN_OPEN_BASEDIR");
    runtime->open_basedir = ptn_duplicate_string(
        configured_open_basedir == NULL ? "" : configured_open_basedir
    );
    const char *configured_max_memory_limit = getenv("PTN_MAX_MEMORY_LIMIT");
    const char *configured_memory_limit = getenv("PTN_MEMORY_LIMIT");
    runtime->max_memory_limit = ptn_duplicate_string(
        configured_max_memory_limit == NULL ? "-1" : configured_max_memory_limit
    );
    runtime->memory_limit = ptn_duplicate_string(
        configured_memory_limit == NULL ? "128M" : configured_memory_limit
    );
    const char *configured_auto_detect_line_endings = getenv("PTN_AUTO_DETECT_LINE_ENDINGS");
    const char *configured_default_charset = getenv("PTN_DEFAULT_CHARSET");
    const char *configured_arg_separator_input = getenv("PTN_ARG_SEPARATOR_INPUT");
    const char *configured_arg_separator_output = getenv("PTN_ARG_SEPARATOR_OUTPUT");
    const char *configured_highlight_comment = getenv("PTN_HIGHLIGHT_COMMENT");
    const char *configured_highlight_default = getenv("PTN_HIGHLIGHT_DEFAULT");
    const char *configured_highlight_html = getenv("PTN_HIGHLIGHT_HTML");
    const char *configured_highlight_keyword = getenv("PTN_HIGHLIGHT_KEYWORD");
    const char *configured_highlight_string = getenv("PTN_HIGHLIGHT_STRING");
    const char *configured_output_handler = getenv("PTN_OUTPUT_HANDLER");
    const char *configured_filter_default = getenv("PTN_FILTER_DEFAULT");
    const char *configured_pcre_backtrack_limit = getenv("PTN_PCRE_BACKTRACK_LIMIT");
    const char *configured_pcre_recursion_limit = getenv("PTN_PCRE_RECURSION_LIMIT");
    const char *configured_pcre_jit = getenv("PTN_PCRE_JIT");
    const char *configured_opcache_blacklist_filename =
        getenv("PTN_OPCACHE_BLACKLIST_FILENAME");
    const char *configured_opcache_enable = getenv("PTN_OPCACHE_ENABLE");
    const char *configured_opcache_enable_cli = getenv("PTN_OPCACHE_ENABLE_CLI");
    const char *configured_opcache_fast_shutdown = getenv("PTN_OPCACHE_FAST_SHUTDOWN");
    const char *configured_opcache_file_cache_only = getenv("PTN_OPCACHE_FILE_CACHE_ONLY");
    const char *configured_opcache_file_update_protection =
        getenv("PTN_OPCACHE_FILE_UPDATE_PROTECTION");
    const char *configured_opcache_interned_strings_buffer =
        getenv("PTN_OPCACHE_INTERNED_STRINGS_BUFFER");
    const char *configured_opcache_log_verbosity_level =
        getenv("PTN_OPCACHE_LOG_VERBOSITY_LEVEL");
    const char *configured_opcache_optimization_level =
        getenv("PTN_OPCACHE_OPTIMIZATION_LEVEL");
    const char *configured_opcache_opt_debug_level = getenv("PTN_OPCACHE_OPT_DEBUG_LEVEL");
    const char *configured_opcache_preload = getenv("PTN_OPCACHE_PRELOAD");
    const char *configured_opcache_preload_user = getenv("PTN_OPCACHE_PRELOAD_USER");
    const char *configured_opcache_save_comments = getenv("PTN_OPCACHE_SAVE_COMMENTS");
    const char *configured_opcache_validate_timestamps =
        getenv("PTN_OPCACHE_VALIDATE_TIMESTAMPS");
    const char *configured_phar_readonly = getenv("PTN_PHAR_READONLY");
    const char *configured_phar_require_hash = getenv("PTN_PHAR_REQUIRE_HASH");
    const char *configured_phar_cache_list = getenv("PTN_PHAR_CACHE_LIST");
    const char *configured_internal_encoding = getenv("PTN_INTERNAL_ENCODING");
    const char *configured_input_encoding = getenv("PTN_INPUT_ENCODING");
    const char *configured_output_encoding = getenv("PTN_OUTPUT_ENCODING");
    const char *configured_iconv_internal_encoding = getenv("PTN_ICONV_INTERNAL_ENCODING");
    const char *configured_iconv_input_encoding = getenv("PTN_ICONV_INPUT_ENCODING");
    const char *configured_iconv_output_encoding = getenv("PTN_ICONV_OUTPUT_ENCODING");
    const char *configured_variables_order = getenv("PTN_VARIABLES_ORDER");
    const char *configured_register_argc_argv = getenv("PTN_REGISTER_ARGC_ARGV");
    const char *configured_enable_post_data_reading = getenv("PTN_ENABLE_POST_DATA_READING");
    const char *configured_file_uploads = getenv("PTN_FILE_UPLOADS");
    const char *configured_max_input_vars = getenv("PTN_MAX_INPUT_VARS");
    const char *configured_max_input_nesting_level = getenv("PTN_MAX_INPUT_NESTING_LEVEL");
    const char *configured_post_max_size = getenv("PTN_POST_MAX_SIZE");
    const char *configured_always_populate_raw_post_data =
        getenv("PTN_ALWAYS_POPULATE_RAW_POST_DATA");
    const char *configured_upload_tmp_dir = getenv("PTN_UPLOAD_TMP_DIR");
    const char *configured_expose_php = getenv("PTN_EXPOSE_PHP");
    const char *configured_user_agent = getenv("PTN_USER_AGENT");
    const char *configured_unserialize_callback_func =
        getenv("PTN_UNSERIALIZE_CALLBACK_FUNC");
    runtime->auto_detect_line_endings = ptn_duplicate_string(
        configured_auto_detect_line_endings == NULL ? "0" : configured_auto_detect_line_endings
    );
    runtime->default_charset = ptn_duplicate_string(
        configured_default_charset == NULL ? "UTF-8" : configured_default_charset
    );
    runtime->arg_separator_input = ptn_duplicate_string(
        configured_arg_separator_input == NULL ? "&" : configured_arg_separator_input
    );
    runtime->arg_separator_output = ptn_duplicate_string(
        configured_arg_separator_output == NULL ? "&" : configured_arg_separator_output
    );
    runtime->highlight_comment = ptn_duplicate_string(
        configured_highlight_comment == NULL ? "#FF8000" : configured_highlight_comment
    );
    runtime->highlight_default = ptn_duplicate_string(
        configured_highlight_default == NULL ? "#0000BB" : configured_highlight_default
    );
    runtime->highlight_html = ptn_duplicate_string(
        configured_highlight_html == NULL ? "#000000" : configured_highlight_html
    );
    runtime->highlight_keyword = ptn_duplicate_string(
        configured_highlight_keyword == NULL ? "#007700" : configured_highlight_keyword
    );
    runtime->highlight_string = ptn_duplicate_string(
        configured_highlight_string == NULL ? "#DD0000" : configured_highlight_string
    );
    runtime->output_handler = ptn_duplicate_string(
        configured_output_handler == NULL ? "" : configured_output_handler
    );
    runtime->filter_default = ptn_duplicate_string(
        configured_filter_default == NULL ? "unsafe_raw" : configured_filter_default
    );
    if (configured_filter_default != NULL &&
        ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_DEPRECATED)) {
        runtime->diagnostics.emitted_deprecation = 1;
        ptn_diagnostic_output_cstr(
            &runtime->diagnostics,
            "Deprecated: The filter.default ini setting is deprecated in Unknown on line 0\n"
        );
    }
    runtime->pcre_backtrack_limit = ptn_duplicate_string(
        configured_pcre_backtrack_limit == NULL ? "1000000" : configured_pcre_backtrack_limit
    );
    runtime->pcre_recursion_limit = ptn_duplicate_string(
        configured_pcre_recursion_limit == NULL ? "100000" : configured_pcre_recursion_limit
    );
    runtime->pcre_jit = ptn_duplicate_string(
        configured_pcre_jit == NULL ? "1" : configured_pcre_jit
    );
    runtime->opcache_blacklist_filename = ptn_duplicate_string(
        configured_opcache_blacklist_filename == NULL ? "" : configured_opcache_blacklist_filename
    );
    runtime->opcache_enable = ptn_duplicate_string(
        configured_opcache_enable == NULL ? "1" : configured_opcache_enable
    );
    runtime->opcache_enable_cli = ptn_duplicate_string(
        configured_opcache_enable_cli == NULL ? "1" : configured_opcache_enable_cli
    );
    runtime->opcache_fast_shutdown = ptn_duplicate_string(
        configured_opcache_fast_shutdown == NULL ? "0" : configured_opcache_fast_shutdown
    );
    runtime->opcache_file_cache_only = ptn_duplicate_string(
        configured_opcache_file_cache_only == NULL ? "0" : configured_opcache_file_cache_only
    );
    runtime->opcache_file_update_protection = ptn_duplicate_string(
        configured_opcache_file_update_protection == NULL ? "2" : configured_opcache_file_update_protection
    );
    runtime->opcache_interned_strings_buffer = ptn_duplicate_string(
        configured_opcache_interned_strings_buffer == NULL ? "8" : configured_opcache_interned_strings_buffer
    );
    runtime->opcache_log_verbosity_level = ptn_duplicate_string(
        configured_opcache_log_verbosity_level == NULL ? "1" : configured_opcache_log_verbosity_level
    );
    runtime->opcache_optimization_level = ptn_duplicate_string(
        configured_opcache_optimization_level == NULL ? "0x7FFEBFFF" : configured_opcache_optimization_level
    );
    runtime->opcache_opt_debug_level = ptn_duplicate_string(
        configured_opcache_opt_debug_level == NULL ? "0" : configured_opcache_opt_debug_level
    );
    runtime->opcache_preload = ptn_duplicate_string(
        configured_opcache_preload == NULL ? "" : configured_opcache_preload
    );
    runtime->opcache_preload_user = ptn_duplicate_string(
        configured_opcache_preload_user == NULL ? "" : configured_opcache_preload_user
    );
    runtime->opcache_save_comments = ptn_duplicate_string(
        configured_opcache_save_comments == NULL ? "1" : configured_opcache_save_comments
    );
    runtime->opcache_validate_timestamps = ptn_duplicate_string(
        configured_opcache_validate_timestamps == NULL ? "1" : configured_opcache_validate_timestamps
    );
    runtime->phar_readonly = ptn_duplicate_string(
        configured_phar_readonly == NULL ? "1" : configured_phar_readonly
    );
    runtime->phar_require_hash = ptn_duplicate_string(
        configured_phar_require_hash == NULL ? "1" : configured_phar_require_hash
    );
    runtime->phar_cache_list = ptn_duplicate_string(
        configured_phar_cache_list == NULL ? "" : configured_phar_cache_list
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
    runtime->iconv_internal_encoding = ptn_duplicate_string(
        configured_iconv_internal_encoding == NULL ? "" : configured_iconv_internal_encoding
    );
    runtime->iconv_input_encoding = ptn_duplicate_string(
        configured_iconv_input_encoding == NULL ? "" : configured_iconv_input_encoding
    );
    runtime->iconv_output_encoding = ptn_duplicate_string(
        configured_iconv_output_encoding == NULL ? "" : configured_iconv_output_encoding
    );
    runtime->variables_order = ptn_duplicate_string(
        configured_variables_order == NULL ? "EGPCS" : configured_variables_order
    );
    runtime->register_argc_argv = ptn_duplicate_string(
        configured_register_argc_argv == NULL ? "0" : configured_register_argc_argv
    );
    runtime->enable_post_data_reading = ptn_duplicate_string(
        configured_enable_post_data_reading == NULL ? "1" : configured_enable_post_data_reading
    );
    runtime->native_argc = 0;
    runtime->native_argv = NULL;
    runtime->file_uploads = ptn_duplicate_string(
        configured_file_uploads == NULL ? "1" : configured_file_uploads
    );
    runtime->max_input_vars = ptn_duplicate_string(
        configured_max_input_vars == NULL ? "1000" : configured_max_input_vars
    );
    runtime->max_input_nesting_level = ptn_duplicate_string(
        configured_max_input_nesting_level == NULL ? "64" : configured_max_input_nesting_level
    );
    runtime->post_max_size = ptn_duplicate_string(
        configured_post_max_size == NULL ? "8M" : configured_post_max_size
    );
    runtime->always_populate_raw_post_data = ptn_duplicate_string(
        configured_always_populate_raw_post_data == NULL ? "-1" : configured_always_populate_raw_post_data
    );
    runtime->upload_tmp_dir = ptn_duplicate_string(
        configured_upload_tmp_dir == NULL ? "" : configured_upload_tmp_dir
    );
    runtime->expose_php = ptn_duplicate_string(
        configured_expose_php == NULL ? "1" : configured_expose_php
    );
    runtime->docref_root = ptn_duplicate_string("");
    runtime->user_agent = ptn_duplicate_string(
        configured_user_agent == NULL ? "" : configured_user_agent
    );
    runtime->unserialize_callback_func = ptn_duplicate_string(
        configured_unserialize_callback_func == NULL ? "" : configured_unserialize_callback_func
    );
    runtime->unserialize_max_depth = PTN_DEFAULT_UNSERIALIZE_MAX_DEPTH;
    runtime->request_body = NULL;
    runtime->request_body_len = 0;
    ptn_symbols_init(&runtime->session_ini);
    runtime->session_id = ptn_duplicate_string("");
    runtime->session_active = 0;
    runtime->session_was_started = 0;
    runtime->session_auto_started = 0;
    runtime->session_start_path = NULL;
    runtime->session_start_line = 0;
    runtime->session_save_handler_kind = 0;
    runtime->session_save_handler_object = ptn_null();
    for (size_t i = 0; i < sizeof(runtime->session_save_handler_callbacks) / sizeof(runtime->session_save_handler_callbacks[0]); i++) {
        runtime->session_save_handler_callbacks[i] = ptn_null();
    }
    runtime->session_save_handler_register_shutdown = 1;
    runtime->session_save_handler_in_callback = 0;
    runtime->session_save_handler_shutdown_warning_pending = 0;
    runtime->session_parent_handler_open = 0;
    runtime->session_parent_save_handler = NULL;
    runtime->session_lazy_write = 1;
    runtime->session_last_data = NULL;
    runtime->session_last_data_len = 0;
    runtime->session_last_data_valid = 0;
    runtime->precision = ptn_ini_precision_value(
        getenv("PTN_PHP_PRECISION"),
        PTN_DEFAULT_PRECISION,
        PTN_MAX_FLOAT_FORMAT_PRECISION
    );
    runtime->serialize_precision = ptn_ini_precision_value(
        getenv("PTN_PHP_SERIALIZE_PRECISION"),
        PTN_DEFAULT_SERIALIZE_PRECISION,
        PTN_MAX_FLOAT_FORMAT_PRECISION
    );
    runtime->initial_precision = PTN_DEFAULT_PRECISION;
    runtime->initial_serialize_precision = PTN_DEFAULT_SERIALIZE_PRECISION;
    runtime->bcmath_scale = 0;
    int64_t configured_bcmath_scale = 0;
    if (ptn_parse_int64_env("PTN_BCMATH_SCALE", &configured_bcmath_scale) &&
        configured_bcmath_scale >= 0 && configured_bcmath_scale <= INT_MAX) {
        runtime->bcmath_scale = (int)configured_bcmath_scale;
    }
    runtime->initial_bcmath_scale = runtime->bcmath_scale;
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
    runtime->tick_enabled = 0;
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
    runtime->assert_active = 1;
    int configured_assert_active = 1;
    if (ptn_parse_bool_env("PTN_ASSERT_ACTIVE", &configured_assert_active)) {
        runtime->assert_active = configured_assert_active;
    }
    runtime->assert_warning = 1;
    int configured_assert_warning = 1;
    if (ptn_parse_bool_env("PTN_ASSERT_WARNING", &configured_assert_warning)) {
        runtime->assert_warning = configured_assert_warning;
    }
    runtime->assert_bail = 0;
    int configured_assert_bail = 0;
    if (ptn_parse_bool_env("PTN_ASSERT_BAIL", &configured_assert_bail)) {
        runtime->assert_bail = configured_assert_bail;
    }
    const char *configured_assert_callback = getenv("PTN_ASSERT_CALLBACK");
    runtime->assert_callback_ini = ptn_duplicate_string(
        configured_assert_callback == NULL ? "" : configured_assert_callback
    );
    runtime->assert_callback = configured_assert_callback == NULL ||
            configured_assert_callback[0] == '\0'
        ? ptn_null()
        : ptn_owned_string(ptn_duplicate_string(configured_assert_callback));
    runtime->assert_exception = 1;
    int configured_assert_exception = 1;
    if (ptn_parse_bool_env("PTN_ASSERT_EXCEPTION", &configured_assert_exception)) {
        runtime->assert_exception = configured_assert_exception;
    }
    const char *configured_disabled_functions = getenv("PTN_DISABLE_FUNCTIONS");
    runtime->disabled_functions = ptn_duplicate_string(
        configured_disabled_functions == NULL ? "" : configured_disabled_functions
    );
    if (ptn_runtime_disabled_function_list_contains(runtime->disabled_functions, "exit")) {
        ptn_emit_warning(&runtime->diagnostics, "Cannot disable function exit()", 0);
    }
    if (ptn_runtime_disabled_function_list_contains(runtime->disabled_functions, "die")) {
        ptn_emit_warning(&runtime->diagnostics, "Cannot disable function die()", 0);
    }
    runtime->call_site_line = 0;
    runtime->suppress_user_call_frame_location = 0;
    runtime->suppress_user_argument_count_location = 0;
    runtime->warn_by_ref_argument_mismatch = 0;
    runtime->throw_argument_count_errors = 0;
    runtime->gc_enabled = 1;
    int configured_zend_enable_gc = 1;
    if (ptn_parse_bool_env("PTN_ZEND_ENABLE_GC", &configured_zend_enable_gc)) {
        runtime->gc_enabled = configured_zend_enable_gc ? 1 : 0;
    }
    runtime->gc_running = 0;
    runtime->gc_mark_epoch = 0;
    runtime->gc_runs = 0;
    runtime->gc_collected = 0;
    runtime->gc_roots = 0;
    runtime->active_serialize_state = NULL;
    runtime->active_unserialize_state = NULL;
    runtime->strtok_string = NULL;
    runtime->strtok_len = 0;
    runtime->strtok_offset = 0;
    runtime->strtok_has_state = 0;
    runtime->json_last_error = 0;
    runtime->json_last_error_line = 0;
    runtime->json_last_error_column = 0;
    runtime->pcre_last_error = PTN_PREG_NO_ERROR;
    runtime->pcre_utf8_cache_data = NULL;
    runtime->pcre_utf8_cache_len = 0;
    runtime->pcre_utf8_cache_known = 0;
    runtime->pcre_utf8_cache_valid = 0;
    int64_t configured_intl_error_level = 0;
    runtime->intl_error_level = ptn_parse_int64_env("PTN_INTL_ERROR_LEVEL", &configured_intl_error_level)
        ? (int)configured_intl_error_level
        : 0;
    int configured_intl_use_exceptions = 0;
    runtime->intl_use_exceptions =
        ptn_parse_bool_env("PTN_INTL_USE_EXCEPTIONS", &configured_intl_use_exceptions)
            ? configured_intl_use_exceptions
            : 0;
    runtime->intl_last_error_message = ptn_duplicate_string("U_ZERO_ERROR");
}
