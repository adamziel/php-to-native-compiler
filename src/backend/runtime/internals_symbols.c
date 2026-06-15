static PTN_UNUSED void ptn_exception_free(PtnException *exception) {
    if (exception == NULL) {
        return;
    }
    if (exception->refcount > 1) {
        exception->refcount--;
        return;
    }
    ptn_runtime_release_object_id(exception->lifecycle_runtime, exception->object_id);
    free(exception->message);
    exception->message_len = 0;
    ptn_value_destroy(&exception->trace);
    ptn_value_destroy(&exception->previous);
    free(exception);
}

static PTN_UNUSED PtnReference *ptn_reference_new_owned(PtnValue value) {
    PtnReference *reference = malloc(sizeof(PtnReference));
    if (reference == NULL) {
        ptn_abort_out_of_memory();
    }
    reference->refcount = 1;
    reference->value = value;
    return reference;
}

static PTN_UNUSED void ptn_reference_assign(PtnReference *reference, PtnValue value) {
    PtnValue stored_value = ptn_value_clone_deref(value);
    ptn_value_destroy(&reference->value);
    reference->value = stored_value;
}

static PTN_UNUSED size_t ptn_array_count_reference(PtnArray *array, PtnReference *reference, size_t depth) {
    if (array == NULL || reference == NULL || depth > 1024) {
        return 0;
    }

    size_t count = 0;
    for (size_t i = 0; i < array->len; i++) {
        PtnValue *entry = &array->entries[i].value;
        if (entry->type == PTN_REFERENCE) {
            if (entry->as.reference == reference) {
                count++;
            }
            continue;
        }
        if (entry->type == PTN_ARRAY) {
            count += ptn_array_count_reference(entry->as.array, reference, depth + 1);
        }
    }
    return count;
}

static PTN_UNUSED void ptn_array_break_reference_cycle(PtnArray *array, PtnReference *reference, size_t depth) {
    if (array == NULL || reference == NULL || depth > 1024) {
        return;
    }

    for (size_t i = 0; i < array->len; i++) {
        PtnValue *entry = &array->entries[i].value;
        if (entry->type == PTN_REFERENCE) {
            if (entry->as.reference == reference) {
                if (reference->refcount > 0) {
                    reference->refcount--;
                }
                *entry = ptn_null();
            }
            continue;
        }
        if (entry->type == PTN_ARRAY) {
            ptn_array_break_reference_cycle(entry->as.array, reference, depth + 1);
        }
    }
}

static PTN_UNUSED void ptn_reference_release(PtnReference *reference) {
    if (reference == NULL) {
        return;
    }
    if (reference->refcount == 0) {
        return;
    }
    if (reference->value.type == PTN_ARRAY &&
        reference->value.as.array != NULL &&
        reference->value.as.array->refcount == 1) {
        size_t internal_refs = ptn_array_count_reference(reference->value.as.array, reference, 0);
        if (internal_refs > 0 && reference->refcount == internal_refs + 1) {
            ptn_array_break_reference_cycle(reference->value.as.array, reference, 0);
        }
    }
    reference->refcount--;
    if (reference->refcount != 0) {
        return;
    }
    ptn_value_destroy(&reference->value);
    free(reference);
}

static PTN_UNUSED void ptn_closure_release(PtnClosure *closure) {
    if (closure == NULL) {
        return;
    }
    if (closure->refcount == 0) {
        return;
    }
    closure->refcount--;
    if (closure->refcount != 0) {
        return;
    }
    ptn_runtime_release_object_id(closure->lifecycle_runtime, closure->object_id);
    ptn_symbols_free(&closure->captures);
    if (closure->has_wrapped_callable) {
        ptn_value_destroy(&closure->wrapped_callable);
    }
    free(closure);
}

static PTN_UNUSED void ptn_array_destroy_storage(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    ptn_cow_debug_note_array_free();
    for (size_t i = 0; i < array->len; i++) {
        ptn_array_key_free(array->entries[i].key);
        ptn_value_destroy(&array->entries[i].value);
    }
    free(array->index_slots);
    free(array->entries);
    free(array);
}

static PTN_UNUSED void ptn_array_free(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    ptn_cow_debug_assert_array_refcount(array, "release");
    ptn_cow_debug_note_array_release();
    if (array->refcount > 1) {
        array->refcount--;
        return;
    }
    array->refcount = 0;
    if (array->iterator_refcount != 0) {
        return;
    }
    ptn_array_destroy_storage(array);
}

static PTN_UNUSED void ptn_value_drop(PtnValue *value) {
    if (value == NULL || !value->owned) {
        return;
    }
    switch (value->type) {
        case PTN_STRING:
            ptn_string_payload_release(value->as.string.payload);
            break;
        case PTN_ARRAY:
            ptn_array_free(value->as.array);
            break;
        case PTN_OBJECT:
            ptn_object_release(value->as.object);
            break;
        case PTN_CLOSURE:
            ptn_closure_release(value->as.closure);
            break;
        case PTN_EXCEPTION:
            ptn_exception_free(value->as.exception);
            break;
        case PTN_RESOURCE:
            ptn_resource_release(value->as.resource);
            break;
        case PTN_REFERENCE:
            ptn_reference_release(value->as.reference);
            break;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
            break;
    }
    *value = ptn_null();
}

static PTN_UNUSED void ptn_value_destroy(PtnValue *value) {
    ptn_value_drop(value);
}

static PTN_UNUSED void ptn_value_detach_for_write(PtnValue *value) {
    if (value == NULL || value->type != PTN_STRING) {
        return;
    }
    PtnStringPayload *payload = value->as.string.payload;
    if (value->owned && payload != NULL && payload->refcount == 1) {
        return;
    }

    int release_old_payload = value->owned && payload != NULL;
    PtnValue detached = ptn_value_deep_clone(*value);
    if (release_old_payload) {
        ptn_string_payload_release(payload);
    }
    *value = detached;
}

static PTN_UNUSED PtnValue ptn_value_borrow(PtnValue value) {
    value.owned = 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_value_deref(PtnValue value) {
    while (value.type == PTN_REFERENCE) {
        value = value.as.reference->value;
    }
    return ptn_value_borrow(value);
}

static PTN_UNUSED PtnValue ptn_value_clone_deref(PtnValue value) {
    return ptn_value_clone(ptn_value_deref(value));
}

static void ptn_symbols_init(PtnSymbolTable *symbols) {
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
    symbols->index_slots = NULL;
    symbols->index_capacity = 0;
}

static void ptn_symbols_free(PtnSymbolTable *symbols) {
    for (size_t i = 0; i < symbols->len; i++) {
        free(symbols->items[i].name);
        ptn_value_destroy(&symbols->items[i].value);
    }
    free(symbols->index_slots);
    free(symbols->items);
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
    symbols->index_slots = NULL;
    symbols->index_capacity = 0;
}

static PTN_UNUSED size_t ptn_symbol_index_capacity_for_entries(size_t expected_entries) {
    if (expected_entries < PTN_SYMBOL_INDEX_MIN_ENTRIES) {
        return 0;
    }
    if (expected_entries > SIZE_MAX / 2) {
        ptn_abort_out_of_memory();
    }

    size_t wanted = expected_entries * 2;
    size_t capacity = PTN_SYMBOL_INDEX_MIN_ENTRIES;
    while (capacity < wanted) {
        if (capacity > SIZE_MAX / 2) {
            ptn_abort_out_of_memory();
        }
        capacity *= 2;
    }
    return capacity;
}

static PTN_UNUSED size_t ptn_symbols_linear_find(PtnSymbolTable *symbols, const char *name) {
    for (size_t i = 0; i < symbols->len; i++) {
        if (strcmp(symbols->items[i].name, name) == 0) {
            return i;
        }
    }
    return symbols->len;
}

static PTN_UNUSED size_t ptn_symbol_index_slot_for_name(PtnSymbolTable *symbols, const char *name, uint64_t hash) {
    size_t mask = symbols->index_capacity - 1;
    size_t slot_index = (size_t)hash & mask;
    for (;;) {
        PtnSymbolIndexSlot *slot = &symbols->index_slots[slot_index];
        if (!slot->occupied ||
            (slot->hash == hash && strcmp(symbols->items[slot->symbol_index].name, name) == 0)) {
            return slot_index;
        }
        slot_index = (slot_index + 1) & mask;
    }
}

static PTN_UNUSED void ptn_symbol_index_insert(PtnSymbolTable *symbols, const char *name, size_t symbol_index) {
    if (symbols->index_capacity == 0) {
        return;
    }
    uint64_t hash = ptn_symbol_name_hash(name);
    size_t slot_index = ptn_symbol_index_slot_for_name(symbols, name, hash);
    PtnSymbolIndexSlot *slot = &symbols->index_slots[slot_index];
    if (!slot->occupied) {
        slot->occupied = 1;
        slot->hash = hash;
        slot->symbol_index = symbol_index;
    }
}

static PTN_UNUSED void ptn_symbols_rebuild_index(PtnSymbolTable *symbols, size_t expected_entries) {
    size_t capacity = ptn_symbol_index_capacity_for_entries(expected_entries);
    free(symbols->index_slots);
    symbols->index_slots = NULL;
    symbols->index_capacity = 0;
    if (capacity == 0) {
        return;
    }

    symbols->index_slots = calloc(capacity, sizeof(PtnSymbolIndexSlot));
    if (symbols->index_slots == NULL) {
        ptn_abort_out_of_memory();
    }
    symbols->index_capacity = capacity;
    for (size_t i = 0; i < symbols->len; i++) {
        ptn_symbol_index_insert(symbols, symbols->items[i].name, i);
    }
}
