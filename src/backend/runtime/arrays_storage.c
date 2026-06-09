    char *copy = malloc(len + 1);
    if (copy == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(copy, string, len + 1);
    return copy;
}

static PTN_UNUSED PtnValue ptn_value_borrow(PtnValue value);
static PTN_UNUSED PtnValue ptn_value_clone(PtnValue value);
static PTN_UNUSED void ptn_value_destroy(PtnValue *value);
static PTN_UNUSED void ptn_array_free(PtnArray *array);

static PTN_UNUSED PtnArrayKey ptn_array_int_key(int64_t integer) {
    PtnArrayKey key;
    key.type = PTN_ARRAY_KEY_INT;
    key.as.integer = integer;
    return key;
}

static PTN_UNUSED PtnArrayKey ptn_array_string_key(const char *string) {
    PtnArrayKey key;
    key.type = PTN_ARRAY_KEY_STRING;
    key.as.string = ptn_duplicate_string(string);
    return key;
}

static PTN_UNUSED int ptn_string_is_integer_array_key(const char *string, int64_t *integer) {
    if (*string == '\0' || *string == '+') {
        return 0;
    }
    if (strcmp(string, "-0") == 0) {
        return 0;
    }

    const char *digits = string;
    if (*digits == '-') {
        digits++;
    }
    if (*digits == '\0') {
        return 0;
    }
    if (*digits == '0' && digits[1] != '\0') {
        return 0;
    }
    for (const char *cursor = digits; *cursor != '\0'; cursor++) {
        if (!isdigit((unsigned char)*cursor)) {
            return 0;
        }
    }

    char *end = NULL;
    errno = 0;
    long long parsed = strtoll(string, &end, 10);
    if (errno == ERANGE || end == string || *end != '\0') {
        return 0;
    }
    *integer = (int64_t)parsed;
    return 1;
}

static PTN_UNUSED void ptn_abort_illegal_array_key(void) {
    fputs("Fatal error: Illegal offset type\n", stderr);
    exit(255);
}

static PTN_UNUSED PtnArrayKey ptn_array_key_from_value(PtnValue value) {
    switch (value.type) {
        case PTN_NULL:
            return ptn_array_string_key("");
        case PTN_BOOL:
            return ptn_array_int_key(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_array_int_key(value.as.integer);
        case PTN_FLOAT:
            return ptn_array_int_key((int64_t)value.as.floating);
        case PTN_STRING: {
            int64_t integer = 0;
            if (ptn_string_is_integer_array_key(value.as.string, &integer)) {
                return ptn_array_int_key(integer);
            }
            return ptn_array_string_key(value.as.string);
        }
        case PTN_ARRAY:
        case PTN_EXCEPTION:
            ptn_abort_illegal_array_key();
    }
    return ptn_array_string_key("");
}

static PTN_UNUSED void ptn_array_key_free(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_STRING) {
        free((char *)key.as.string);
    }
}

static PTN_UNUSED int ptn_array_keys_equal(PtnArrayKey left, PtnArrayKey right) {
    if (left.type != right.type) {
        return 0;
    }
    if (left.type == PTN_ARRAY_KEY_INT) {
        return left.as.integer == right.as.integer;
    }
    return strcmp(left.as.string, right.as.string) == 0;
}

static PTN_UNUSED uint64_t ptn_hash_mix_uint64(uint64_t value) {
    value ^= value >> 30;
    value *= 0xbf58476d1ce4e5b9ULL;
    value ^= value >> 27;
    value *= 0x94d049bb133111ebULL;
    value ^= value >> 31;
    return value;
}

static PTN_UNUSED uint64_t ptn_array_key_hash(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_hash_mix_uint64((uint64_t)key.as.integer ^ 0x9e3779b97f4a7c15ULL);
    }

    uint64_t hash = 1469598103934665603ULL ^ 0x517cc1b727220a95ULL;
    for (const unsigned char *cursor = (const unsigned char *)key.as.string; *cursor != '\0'; cursor++) {
        hash ^= (uint64_t)*cursor;
        hash *= 1099511628211ULL;
    }
    return ptn_hash_mix_uint64(hash);
}

static PTN_UNUSED uint64_t ptn_symbol_name_hash(const char *name) {
    uint64_t hash = 1469598103934665603ULL ^ 0x7b2d6f8fe10b25c9ULL;
    for (const unsigned char *cursor = (const unsigned char *)name; *cursor != '\0'; cursor++) {
        hash ^= (uint64_t)*cursor;
        hash *= 1099511628211ULL;
    }
    return ptn_hash_mix_uint64(hash);
}

static PTN_UNUSED void ptn_array_index_init(PtnArray *array, size_t expected_entries) {
    array->index_slots = NULL;
    array->index_capacity = 0;

    if (expected_entries < PTN_ARRAY_INDEX_MIN_ENTRIES) {
        return;
    }
    if (expected_entries > SIZE_MAX / 2) {
        ptn_abort_out_of_memory();
    }

    size_t wanted = expected_entries * 2;
    size_t capacity = PTN_ARRAY_INDEX_MIN_ENTRIES;
    while (capacity < wanted) {
        if (capacity > SIZE_MAX / 2) {
            ptn_abort_out_of_memory();
        }
        capacity *= 2;
    }

    array->index_slots = calloc(capacity, sizeof(PtnArrayIndexSlot));
    if (array->index_slots == NULL) {
        ptn_abort_out_of_memory();
    }
    array->index_capacity = capacity;
}

static PTN_UNUSED size_t ptn_array_index_slot_for_key(PtnArray *array, PtnArrayKey key, uint64_t hash);

static PTN_UNUSED void ptn_array_rebuild_index(PtnArray *array) {
    free(array->index_slots);
    ptn_array_index_init(array, array->len);
    if (array->index_capacity == 0) {
        return;
    }
    for (size_t i = 0; i < array->len; i++) {
        uint64_t hash = ptn_array_key_hash(array->entries[i].key);
        size_t slot_index = ptn_array_index_slot_for_key(array, array->entries[i].key, hash);
        PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
        slot->occupied = 1;
        slot->hash = hash;
        slot->entry_index = i;
    }
}

static PTN_UNUSED size_t ptn_array_linear_find_key(PtnArray *array, PtnArrayKey key) {
    for (size_t i = 0; i < array->len; i++) {
        if (ptn_array_keys_equal(array->entries[i].key, key)) {
            return i;
        }
    }
    return array->len;
}

static PTN_UNUSED size_t ptn_array_index_slot_for_key(PtnArray *array, PtnArrayKey key, uint64_t hash) {
    size_t mask = array->index_capacity - 1;
    size_t slot_index = (size_t)hash & mask;
    for (;;) {
        PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
        if (!slot->occupied ||
            (slot->hash == hash && ptn_array_keys_equal(array->entries[slot->entry_index].key, key))) {
            return slot_index;
        }
        slot_index = (slot_index + 1) & mask;
    }
}

static PTN_UNUSED void ptn_array_update_next_auto_key(PtnArray *array, PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT &&
        key.as.integer >= array->next_auto_key &&
        key.as.integer < INT64_MAX) {
        array->next_auto_key = key.as.integer + 1;
    }
}

static PTN_UNUSED void ptn_array_recompute_next_auto_key(PtnArray *array) {
    array->next_auto_key = 0;
    for (size_t i = 0; i < array->len; i++) {
        ptn_array_update_next_auto_key(array, array->entries[i].key);
    }
}

static PTN_UNUSED size_t ptn_array_find_key(PtnArray *array, PtnArrayKey key) {
    if (array->index_capacity != 0) {
        uint64_t hash = ptn_array_key_hash(key);
        size_t slot_index = ptn_array_index_slot_for_key(array, key, hash);
        PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
        return slot->occupied ? slot->entry_index : array->len;
    }
    return ptn_array_linear_find_key(array, key);
}

static PTN_UNUSED void ptn_array_index_insert(PtnArray *array, PtnArrayKey key, size_t entry_index) {
    if (array->index_capacity == 0) {
        return;
    }
    uint64_t hash = ptn_array_key_hash(key);
    size_t slot_index = ptn_array_index_slot_for_key(array, key, hash);
    PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
    if (!slot->occupied) {
        slot->occupied = 1;
        slot->hash = hash;
        slot->entry_index = entry_index;
    }
}

static PTN_UNUSED void ptn_array_set_entry(PtnArray *array, PtnArrayKey key, PtnValue value) {
    size_t index = ptn_array_find_key(array, key);
    ptn_array_update_next_auto_key(array, key);
    if (index < array->len) {
        ptn_value_destroy(&array->entries[index].value);
        array->entries[index].value = value;
        ptn_array_key_free(key);
        return;
    }
    if (array->len == array->capacity) {
        size_t new_capacity = array->capacity == 0 ? 8 : array->capacity * 2;
        if (new_capacity < array->capacity) {
            ptn_abort_out_of_memory();
        }
        PtnArrayEntry *new_entries = realloc(array->entries, new_capacity * sizeof(PtnArrayEntry));
        if (new_entries == NULL) {
            ptn_abort_out_of_memory();
        }
        array->entries = new_entries;
        array->capacity = new_capacity;
    }
    size_t entry_index = array->len;
    array->entries[entry_index].key = key;
    array->entries[entry_index].value = value;
    array->len++;
    ptn_array_index_insert(array, key, entry_index);
}

static PTN_UNUSED int ptn_array_unset_entry(PtnArray *array, PtnArrayKey key) {
    size_t index = ptn_array_find_key(array, key);
    if (index >= array->len) {
        ptn_array_key_free(key);
        return 0;
    }

    ptn_array_key_free(array->entries[index].key);
    ptn_value_destroy(&array->entries[index].value);
    for (size_t i = index + 1; i < array->len; i++) {
        array->entries[i - 1] = array->entries[i];
    }
    array->len--;
    if (array->current_index > array->len) {
        array->current_index = array->len;
    }
    ptn_array_key_free(key);
    ptn_array_rebuild_index(array);
    return 1;
}

static PTN_UNUSED PtnValue ptn_array_from_literal_entries(size_t entry_count, const PtnArrayLiteralEntry *entries) {
    PtnArray *array = malloc(sizeof(PtnArray));
    if (array == NULL) {
        ptn_abort_out_of_memory();
    }
    array->len = 0;
    array->capacity = entry_count;
    array->entries = NULL;
    array->index_slots = NULL;
    array->index_capacity = 0;
    array->next_auto_key = 0;
    array->current_index = 0;
    if (entry_count != 0) {
        array->entries = malloc(entry_count * sizeof(PtnArrayEntry));
        if (array->entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }
    ptn_array_index_init(array, entry_count);

    for (size_t i = 0; i < entry_count; i++) {
        PtnArrayKey key = entries[i].has_key
            ? ptn_array_key_from_value(entries[i].key)
            : ptn_array_int_key(array->next_auto_key);
        ptn_array_set_entry(array, key, ptn_value_clone(entries[i].value));
    }
    return ptn_array(array);
}

static PTN_UNUSED PtnArrayKey ptn_array_key_clone(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_array_int_key(key.as.integer);
    }
    return ptn_array_string_key(key.as.string);
}

static PTN_UNUSED PtnArray *ptn_array_clone(PtnArray *source) {
    PtnArray *array = malloc(sizeof(PtnArray));
    if (array == NULL) {
        ptn_abort_out_of_memory();
    }
    array->len = 0;
    array->capacity = source->len;
    array->entries = NULL;
    array->index_slots = NULL;
    array->index_capacity = 0;
    array->next_auto_key = 0;
    array->current_index = source->current_index <= source->len ? source->current_index : source->len;
    if (source->len != 0) {
        array->entries = malloc(source->len * sizeof(PtnArrayEntry));
        if (array->entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }
    ptn_array_index_init(array, source->len);
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayKey key = ptn_array_key_clone(source->entries[i].key);
        PtnValue value = ptn_value_clone(source->entries[i].value);
        ptn_array_set_entry(array, key, value);
    }
    array->next_auto_key = source->next_auto_key;
    return array;
}

static PTN_UNUSED PtnValue ptn_value_clone(PtnValue value) {
    switch (value.type) {
        case PTN_STRING:
