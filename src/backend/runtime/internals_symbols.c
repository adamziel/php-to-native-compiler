            return ptn_owned_string(ptn_duplicate_string(value.as.string));
        case PTN_ARRAY:
            return ptn_array(ptn_array_clone(value.as.array));
        case PTN_OBJECT:
            return ptn_object(ptn_object_clone(value.as.object));
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
            return value;
    }
    return value;
}

static PTN_UNUSED void ptn_array_free(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    for (size_t i = 0; i < array->len; i++) {
        ptn_array_key_free(array->entries[i].key);
        ptn_value_destroy(&array->entries[i].value);
    }
    free(array->index_slots);
    free(array->entries);
    free(array);
}

static PTN_UNUSED void ptn_value_destroy(PtnValue *value) {
    if (value == NULL || !value->owned) {
        return;
    }
    switch (value->type) {
        case PTN_STRING:
            free((char *)value->as.string);
            break;
        case PTN_ARRAY:
            ptn_array_free(value->as.array);
            break;
        case PTN_OBJECT:
            ptn_object_free(value->as.object);
            break;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
            break;
    }
    *value = ptn_null();
}

static PTN_UNUSED PtnValue ptn_value_borrow(PtnValue value) {
    value.owned = 0;
    return value;
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
