    char *copy = malloc(len + 1);
    if (copy == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(copy, string, len + 1);
    return copy;
}

static PTN_UNUSED char *ptn_duplicate_string_len(const char *string, size_t len) {
    char *copy = malloc(len + 1);
    if (copy == NULL) {
        ptn_abort_out_of_memory();
    }
    if (len != 0) {
        memcpy(copy, string, len);
    }
    copy[len] = '\0';
    return copy;
}

static PTN_UNUSED PtnValue ptn_value_borrow(PtnValue value);
static PTN_UNUSED PtnValue ptn_value_share(PtnValue value);
static PTN_UNUSED PtnValue ptn_value_deref(PtnValue value);
static PTN_UNUSED PtnValue ptn_value_clone(PtnValue value);
static PTN_UNUSED PtnValue ptn_value_clone_deref(PtnValue value);
static PTN_UNUSED int ptn_reference_assign(PtnRuntime *runtime, PtnReference *reference, PtnValue value);
static PTN_UNUSED void ptn_reference_release(PtnReference *reference);
static PTN_UNUSED void ptn_value_destroy(PtnValue *value);
static PTN_UNUSED void ptn_value_drop(PtnValue *value);
static PTN_UNUSED PtnArray *ptn_array_clone(PtnArray *source);
static PTN_UNUSED void ptn_array_free(PtnArray *array);
static PTN_UNUSED void ptn_object_retain(PtnObject *object);
static PTN_UNUSED void ptn_object_release(PtnObject *object);
static PTN_UNUSED void ptn_object_register_property_metadata(
    PtnObject *object,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    int is_readonly,
    PtnPropertyTypeKind type_kind,
    const char *type_class_name,
    const char *type_text,
    int type_allows_null
);
static PTN_UNUSED void ptn_runtime_register_object(PtnRuntime *runtime, PtnObject *object);
static PTN_UNUSED void ptn_runtime_unregister_object(PtnRuntime *runtime, PtnObject *object);
static PTN_UNUSED void ptn_runtime_run_object_destructors_until_output_buffer(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_unreferenced_object_destructors(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_object_destructors(PtnRuntime *runtime);

static PTN_UNUSED PtnArrayKey ptn_array_int_key(int64_t integer) {
    PtnArrayKey key;
    key.type = PTN_ARRAY_KEY_INT;
    key.string_len = 0;
    key.as.integer = integer;
    return key;
}

static PTN_UNUSED PtnArrayKey ptn_array_string_key_len(const char *string, size_t len) {
    PtnArrayKey key;
    key.type = PTN_ARRAY_KEY_STRING;
    key.string_len = len;
    key.as.string = ptn_duplicate_string_len(string, len);
    return key;
}

static PTN_UNUSED PtnArrayKey ptn_array_string_key(const char *string) {
    return ptn_array_string_key_len(string, strlen(string));
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

static PTN_UNUSED int ptn_string_is_integer_array_key_len(const char *string, size_t len, int64_t *integer) {
    if (memchr(string, '\0', len) != NULL) {
        return 0;
    }
    return ptn_string_is_integer_array_key(string, integer);
}

static PTN_UNUSED void ptn_abort_illegal_array_key(void) {
    fputs("Fatal error: Illegal offset type\n", stderr);
    exit(255);
}

static PTN_UNUSED PtnArrayKey ptn_array_key_from_value(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return ptn_array_string_key("");
        case PTN_BOOL:
            return ptn_array_int_key(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_array_int_key(value.as.integer);
        case PTN_FLOAT:
            return ptn_array_int_key((int64_t)value.as.floating);
        case PTN_RESOURCE:
            return ptn_array_int_key(value.as.resource->id);
        case PTN_STRING: {
            int64_t integer = 0;
            const char *string = (const char *)value.as.string.data;
            if (ptn_string_is_integer_array_key_len(string, value.as.string.len, &integer)) {
                return ptn_array_int_key(integer);
            }
            return ptn_array_string_key_len(string, value.as.string.len);
        }
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
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
    return left.string_len == right.string_len &&
        memcmp(left.as.string, right.as.string, left.string_len) == 0;
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
    const unsigned char *string = (const unsigned char *)key.as.string;
    for (size_t i = 0; i < key.string_len; i++) {
        hash ^= (uint64_t)string[i];
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
        key.as.integer >= array->next_auto_key) {
        array->next_auto_key = key.as.integer < INT64_MAX ? key.as.integer + 1 : INT64_MAX;
    }
}

static PTN_UNUSED void ptn_array_recompute_next_auto_key(PtnArray *array) {
    array->next_auto_key = 0;
    for (size_t i = 0; i < array->len; i++) {
        ptn_array_update_next_auto_key(array, array->entries[i].key);
    }
}

static PTN_UNUSED void ptn_array_note_mutation(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    array->mutation_epoch++;
}

static PTN_UNUSED void ptn_array_note_value_replacement(PtnValue old_value, PtnValue new_value) {
    PtnValue old_resolved = ptn_value_deref(old_value);
    if (old_resolved.type != PTN_ARRAY || old_resolved.as.array == NULL) {
        return;
    }
    PtnValue new_resolved = ptn_value_deref(new_value);
    if (new_resolved.type == PTN_ARRAY && new_resolved.as.array == old_resolved.as.array) {
        return;
    }
    ptn_array_note_mutation(old_resolved.as.array);
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

static PTN_UNUSED void ptn_array_index_insert_appended_entry(
    PtnArray *array,
    PtnArrayKey key,
    size_t entry_index
) {
    if (array->index_capacity == 0) {
        if (array->len >= PTN_ARRAY_INDEX_MIN_ENTRIES) {
            ptn_array_rebuild_index(array);
        }
        return;
    }
    if (array->len > array->index_capacity / 2) {
        ptn_array_rebuild_index(array);
        return;
    }
    ptn_array_index_insert(array, key, entry_index);
}

static PTN_UNUSED void ptn_array_set_entry(PtnArray *array, PtnArrayKey key, PtnValue value) {
    size_t index = ptn_array_find_key(array, key);
    ptn_array_update_next_auto_key(array, key);
    ptn_array_note_mutation(array);
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
    ptn_array_index_insert_appended_entry(array, key, entry_index);
}

static PTN_UNUSED void ptn_array_set_entry_publish_first(PtnArray *array, PtnArrayKey key, PtnValue value) {
    size_t index = ptn_array_find_key(array, key);
    ptn_array_update_next_auto_key(array, key);
    ptn_array_note_mutation(array);
    if (index < array->len) {
        PtnValue old_value = array->entries[index].value;
        array->entries[index].value = value;
        ptn_array_key_free(key);
        ptn_value_destroy(&old_value);
        return;
    }
    ptn_array_set_entry(array, key, value);
}

static PTN_UNUSED void ptn_array_write_entry(PtnRuntime *runtime, PtnArray *array, PtnArrayKey key, PtnValue value) {
    size_t index = ptn_array_find_key(array, key);
    if (index < array->len && array->entries[index].value.type == PTN_REFERENCE) {
        ptn_array_update_next_auto_key(array, key);
        ptn_reference_assign(runtime, array->entries[index].value.as.reference, value);
        ptn_value_destroy(&value);
        ptn_array_key_free(key);
        return;
    }
    ptn_array_set_entry(array, key, value);
}

static PTN_UNUSED PtnValue ptn_array_write_entry_result(PtnRuntime *runtime, PtnArray *array, PtnArrayKey key, PtnValue value) {
    PtnValue stored = ptn_value_clone(ptn_value_deref(value));
    size_t index = ptn_array_find_key(array, key);
    if (index < array->len && array->entries[index].value.type == PTN_REFERENCE) {
        ptn_array_update_next_auto_key(array, key);
        if (ptn_reference_assign(runtime, array->entries[index].value.as.reference, stored)) {
            PtnValue result = ptn_value_clone(array->entries[index].value.as.reference->value);
            ptn_value_destroy(&stored);
            ptn_array_key_free(key);
            return result;
        }
        ptn_value_destroy(&stored);
        ptn_array_key_free(key);
        return ptn_value_clone_deref(value);
    }
    PtnValue result = ptn_value_clone(stored);
    ptn_array_set_entry(array, key, stored);
    return result;
}

static PTN_UNUSED int ptn_array_unset_entry(PtnArray *array, PtnArrayKey key) {
    size_t index = ptn_array_find_key(array, key);
    if (index >= array->len) {
        ptn_array_key_free(key);
        return 0;
    }

    ptn_array_note_mutation(array);
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

static PTN_UNUSED void ptn_emit_null_array_offset_deprecation(PtnRuntime *runtime, size_t line);
static PTN_UNUSED void ptn_emit_resource_offset_warning(PtnRuntime *runtime, PtnResource *resource, size_t line);
static PTN_UNUSED int ptn_array_append_key_available(PtnRuntime *runtime, PtnArray *array);

static PTN_UNUSED PtnValue ptn_array_from_literal_entries_impl(
    PtnRuntime *runtime,
    size_t line,
    size_t entry_count,
    const PtnArrayLiteralEntry *entries
) {
    PtnArray *array = malloc(sizeof(PtnArray));
    if (array == NULL) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_note_array_alloc();
    array->refcount = 1;
    array->debug_hidden_refcount = 0;
    array->debug_reference_wrapped = 0;
    array->iterator_refcount = 0;
    array->len = 0;
    array->capacity = entry_count;
    array->entries = NULL;
    array->index_slots = NULL;
    array->index_capacity = 0;
    array->next_auto_key = 0;
    array->current_index = 0;
    array->has_iterator_current_index = 0;
    array->iterator_current_index = 0;
    array->iterator_mutation_resume_index = 0;
    array->iterator_mutation_epoch = 0;
    array->mutation_epoch = 0;
    if (entry_count != 0) {
        array->entries = malloc(entry_count * sizeof(PtnArrayEntry));
        if (array->entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }
    ptn_array_index_init(array, entry_count);

    for (size_t i = 0; i < entry_count; i++) {
        PtnValue key_value = entries[i].has_key ? ptn_value_deref(entries[i].key) : ptn_null();
        if (runtime != NULL && entries[i].has_key) {
            if (key_value.type == PTN_NULL) {
                ptn_emit_null_array_offset_deprecation(runtime, line);
            } else if (key_value.type == PTN_RESOURCE) {
                ptn_emit_resource_offset_warning(runtime, key_value.as.resource, line);
            }
        }
        PtnArrayKey key;
        if (entries[i].has_key) {
            key = ptn_array_key_from_value(key_value);
        } else {
            if (runtime != NULL && !ptn_array_append_key_available(runtime, array)) {
                continue;
            }
            key = ptn_array_int_key(array->next_auto_key);
        }
        ptn_array_set_entry(array, key, ptn_value_clone(entries[i].value));
    }
    return ptn_array(array);
}

static PTN_UNUSED PtnValue ptn_array_from_literal_entries(size_t entry_count, const PtnArrayLiteralEntry *entries) {
    return ptn_array_from_literal_entries_impl(NULL, 0, entry_count, entries);
}

static PTN_UNUSED PtnValue ptn_array_from_literal_entries_at(
    PtnRuntime *runtime,
    size_t line,
    size_t entry_count,
    const PtnArrayLiteralEntry *entries
) {
    return ptn_array_from_literal_entries_impl(runtime, line, entry_count, entries);
}

static PTN_UNUSED void ptn_runtime_register_object(PtnRuntime *runtime, PtnObject *object) {
    if (runtime == NULL || object == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    if (root->live_objects_len == root->live_objects_capacity) {
        size_t new_capacity = root->live_objects_capacity == 0
            ? 8
            : root->live_objects_capacity * 2;
        if (new_capacity < root->live_objects_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnObject *)) {
            ptn_abort_out_of_memory();
        }
        PtnObject **new_objects = realloc(
            root->live_objects,
            new_capacity * sizeof(PtnObject *)
        );
        if (new_objects == NULL) {
            ptn_abort_out_of_memory();
        }
        root->live_objects = new_objects;
        root->live_objects_capacity = new_capacity;
    }
    root->live_objects[root->live_objects_len++] = object;
}

static PTN_UNUSED void ptn_runtime_unregister_object(PtnRuntime *runtime, PtnObject *object) {
    if (runtime == NULL || object == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    for (size_t i = 0; i < root->live_objects_len; i++) {
        if (root->live_objects[i] != object) {
            continue;
        }
        for (size_t j = i + 1; j < root->live_objects_len; j++) {
            root->live_objects[j - 1] = root->live_objects[j];
        }
        root->live_objects_len--;
        return;
    }
}

static PTN_UNUSED void ptn_object_run_destructor(PtnObject *object) {
    if (object == NULL || !object->destructor_enabled || object->destructor_called) {
        return;
    }
    PtnRuntime *runtime = object->lifecycle_runtime;
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    PtnValue receiver = ptn_value_borrow(ptn_object(object));
    size_t destructor_line = runtime->call_site_line != 0
        ? runtime->call_site_line
        : root->call_site_line;
    if (destructor_line == 0) {
        destructor_line = 1;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_internal_class_name_is_zip_archive(object->class_name)) {
        object->destructor_called = 1;
        ptn_zip_archive_run_destructor(runtime, receiver, destructor_line);
        return;
    }
#endif
    if (root->method_dispatch == NULL ||
        root->declared_method_exists == NULL ||
        !root->declared_method_exists(object->class_name, "__destruct")) {
        return;
    }

    object->destructor_called = 1;
    PtnValue result = root->method_dispatch(root, receiver, "__destruct", 0, NULL, destructor_line);
    ptn_value_destroy(&result);
}

static PTN_UNUSED void ptn_runtime_run_static_property_value_destructors(
    PtnValue value,
    size_t depth
);

static PTN_UNUSED void ptn_runtime_register_static_local(
    PtnRuntime *runtime,
    PtnReference *reference
) {
    if (runtime == NULL || reference == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    for (size_t i = 0; i < root->static_local_slots_len; i++) {
        if (root->static_local_slots[i].reference == reference) {
            return;
        }
    }
    if (root->static_local_slots_len == root->static_local_slots_capacity) {
        size_t new_capacity = root->static_local_slots_capacity == 0
            ? 8
            : root->static_local_slots_capacity * 2;
        if (new_capacity < root->static_local_slots_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnStaticLocalSlot)) {
            ptn_abort_out_of_memory();
        }
        PtnStaticLocalSlot *new_slots = realloc(
            root->static_local_slots,
            new_capacity * sizeof(PtnStaticLocalSlot)
        );
        if (new_slots == NULL) {
            ptn_abort_out_of_memory();
        }
        root->static_local_slots = new_slots;
        root->static_local_slots_capacity = new_capacity;
    }
    root->static_local_slots[root->static_local_slots_len++].reference = reference;
}

static PTN_UNUSED void ptn_runtime_run_static_local_destructors(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    for (size_t i = 0; i < root->static_local_slots_len; i++) {
        PtnReference *reference = root->static_local_slots[i].reference;
        if (reference == NULL) {
            continue;
        }
        ptn_runtime_run_static_property_value_destructors(ptn_reference_value(reference), 0);
    }
}

static PTN_UNUSED void ptn_runtime_release_static_locals(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    for (size_t i = 0; i < root->static_local_slots_len; i++) {
        ptn_reference_release(root->static_local_slots[i].reference);
        root->static_local_slots[i].reference = NULL;
    }
    free(root->static_local_slots);
    root->static_local_slots = NULL;
    root->static_local_slots_len = 0;
    root->static_local_slots_capacity = 0;
}

static void ptn_runtime_remove_live_object_at(PtnRuntime *root, size_t index) {
    if (root == NULL || index >= root->live_objects_len) {
        return;
    }
    for (size_t i = index + 1; i < root->live_objects_len; i++) {
        root->live_objects[i - 1] = root->live_objects[i];
    }
    root->live_objects_len--;
}

static void ptn_runtime_run_object_destructors_matching(PtnRuntime *runtime, int only_unreferenced) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    size_t index = root->live_objects_len;
    while (index > 0) {
        index--;
        PtnObject *object = root->live_objects[index];
        if (
            object != NULL &&
            object->refcount != 0 &&
            !object->destructor_called &&
            only_unreferenced &&
            object->refcount > 1
        ) {
            continue;
        }
        ptn_runtime_remove_live_object_at(root, index);
        if (object == NULL || object->refcount == 0 || object->destructor_called) {
            continue;
        }
        ptn_object_retain(object);
        ptn_object_run_destructor(object);
        ptn_object_release(object);
    }
}

static PTN_UNUSED void ptn_runtime_run_unreferenced_object_destructors(PtnRuntime *runtime) {
    ptn_runtime_run_object_destructors_matching(runtime, 1);
}

static PTN_UNUSED void ptn_runtime_run_object_destructors_until_output_buffer(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    size_t initial_output_buffers_len = root->output_buffers_len;
    size_t index = root->live_objects_len;
    while (index > 0) {
        index--;
        PtnObject *object = root->live_objects[index];
        ptn_runtime_remove_live_object_at(root, index);
        if (object == NULL || object->refcount == 0 || object->destructor_called) {
            continue;
        }
        ptn_object_retain(object);
        ptn_object_run_destructor(object);
        ptn_object_release(object);
        if (root->output_buffers_len > initial_output_buffers_len) {
            return;
        }
        if (index > root->live_objects_len) {
            index = root->live_objects_len;
        }
    }
}

static PTN_UNUSED void ptn_runtime_run_object_destructors(PtnRuntime *runtime) {
    ptn_runtime_run_object_destructors_matching(runtime, 0);
}

static PTN_UNUSED void ptn_runtime_run_static_property_value_destructors(PtnValue value, size_t depth) {
    if (depth > 1024) {
        return;
    }
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT && value.as.object != NULL) {
        ptn_object_retain(value.as.object);
        ptn_object_run_destructor(value.as.object);
        ptn_object_release(value.as.object);
        return;
    }
    if (value.type != PTN_ARRAY || value.as.array == NULL) {
        return;
    }
    PtnArray *array = value.as.array;
    for (size_t i = 0; i < array->len; i++) {
        ptn_runtime_run_static_property_value_destructors(array->entries[i].value, depth + 1);
    }
}

static PTN_UNUSED void ptn_runtime_run_static_property_destructors(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    PtnSymbolTable *static_properties = root->static_properties == NULL
        ? &root->owned_static_properties
        : root->static_properties;
    size_t len = static_properties->len;
    for (size_t i = 0; i < len; i++) {
        ptn_runtime_run_static_property_value_destructors(static_properties->items[i].value, 0);
    }
}

static PTN_UNUSED PtnValue ptn_object_new_shell(PtnRuntime *runtime, const char *class_name) {
    PtnObject *object = malloc(sizeof(PtnObject));
    if (object == NULL) {
        ptn_abort_out_of_memory();
    }
    PtnRuntime *root = runtime == NULL || runtime->lifecycle_root == NULL
        ? runtime
        : runtime->lifecycle_root;
    PtnValue properties = ptn_array_from_literal_entries(0, NULL);
    object->refcount = 1;
    object->object_id = ptn_runtime_alloc_object_id(root);
    object->class_name = ptn_duplicate_string(class_name);
    object->enum_case_name = NULL;
    object->properties = properties.as.array;
    object->property_metadata = NULL;
    object->property_metadata_len = 0;
    object->property_metadata_capacity = 0;
    object->native_data = NULL;
    object->native_data_free = NULL;
    object->lifecycle_runtime = root;
    object->destructor_enabled = 1;
    object->destructor_called = 0;
    object->lazy_uninitialized = 0;
    object->lazy_is_proxy = 0;
    object->lazy_options = 0;
    object->lazy_initializing = 0;
    object->lazy_initializer = ptn_null();
    object->lazy_proxy_instance = ptn_null();
    ptn_runtime_register_object(root, object);
    return ptn_object(object);
}

static PTN_UNUSED int ptn_lazy_object_is_uninitialized(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_OBJECT &&
        value.as.object != NULL &&
        value.as.object->lazy_uninitialized;
}

static PTN_UNUSED void ptn_lazy_object_mark(
    PtnValue value,
    PtnValue initializer,
    int is_proxy,
    int options
) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT || value.as.object == NULL) {
        return;
    }
    PtnObject *object = value.as.object;
    ptn_value_destroy(&object->lazy_initializer);
    ptn_value_destroy(&object->lazy_proxy_instance);
    object->lazy_uninitialized = 1;
    object->lazy_is_proxy = is_proxy ? 1 : 0;
    object->lazy_options = options;
    object->lazy_initializing = 0;
    object->lazy_initializer = ptn_value_clone_deref(initializer);
    object->lazy_proxy_instance = ptn_null();
}

static PTN_UNUSED void ptn_lazy_object_copy_properties_from_instance(
    PtnObject *target,
    PtnObject *source
) {
    if (target == NULL || source == NULL || source->properties == NULL) {
        return;
    }
    PtnArray *copied = ptn_array_clone(source->properties);
    ptn_array_free(target->properties);
    target->properties = copied;
}

static void ptn_lazy_object_sync_proxy_instance_properties_depth(PtnObject *proxy, size_t depth) {
    if (proxy == NULL || proxy->lazy_uninitialized || !proxy->lazy_is_proxy) {
        return;
    }
    if (depth > 64) {
        return;
    }
    PtnValue real = ptn_value_deref(proxy->lazy_proxy_instance);
    if (real.type != PTN_OBJECT || real.as.object == NULL) {
        return;
    }
    if (real.as.object == proxy) {
        return;
    }
    ptn_lazy_object_copy_properties_from_instance(real.as.object, proxy);
    ptn_lazy_object_sync_proxy_instance_properties_depth(real.as.object, depth + 1);
}

static PTN_UNUSED void ptn_lazy_object_sync_proxy_instance_properties(PtnObject *proxy) {
    ptn_lazy_object_sync_proxy_instance_properties_depth(proxy, 0);
}

static PTN_UNUSED int ptn_lazy_object_real_instance_compatible(
    PtnRuntime *runtime,
    PtnObject *proxy,
    PtnValue real
) {
    (void)runtime;
    real = ptn_value_deref(real);
    if (proxy == NULL || real.type != PTN_OBJECT || real.as.object == NULL) {
        return 0;
    }
    if (!ptn_declared_class_is_same_or_descendant(proxy->class_name, real.as.object->class_name)) {
        return 0;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (!ptn_ascii_case_equal(proxy->class_name, real.as.object->class_name) &&
        (ptn_declared_class_direct_non_private_method_exists(proxy->class_name, "__destruct") ||
         ptn_declared_class_direct_non_private_method_exists(proxy->class_name, "__clone"))) {
        return 0;
    }
#endif
    return 1;
}

static PTN_UNUSED const char *ptn_lazy_object_initializer_type_name(PtnValue value) {
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
            return value.as.object->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED int ptn_lazy_object_initialize(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    (void)line;
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT || value.as.object == NULL) {
        return 1;
    }
    PtnObject *object = value.as.object;
    if (!object->lazy_uninitialized) {
        return 1;
    }
    if (object->lazy_initializing) {
        return 1;
    }
    object->lazy_initializing = 1;
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue initializer = ptn_value_clone_deref(object->lazy_initializer);
    PtnValue arg = ptn_value_borrow(ptn_object(object));
    PtnValue result = ptn_null();
    PtnTryFrame initializer_frame;
    int initializer_frame_active = 0;
    if (runtime != NULL && runtime->exceptions != NULL) {
        ptn_try_frame_push(runtime, &initializer_frame);
        initializer_frame_active = 1;
        if (setjmp(initializer_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &initializer_frame);
            object->lazy_initializing = 0;
            ptn_value_destroy(&initializer);
            ptn_value_destroy(&result);
            ptn_rethrow_exception(runtime);
            return 0;
        }
    }
    result = ptn_call_callable(runtime, initializer, 1, &arg, line);
    if (initializer_frame_active) {
        ptn_try_frame_pop(runtime, &initializer_frame);
    }
    ptn_value_destroy(&initializer);
    if (runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL) {
        object->lazy_initializing = 0;
        ptn_value_destroy(&result);
        return 0;
    }
    if (object->lazy_is_proxy) {
        PtnValue real = ptn_value_deref(result);
        if (real.type != PTN_OBJECT) {
            char message[256];
            const char *type_name = ptn_lazy_object_initializer_type_name(real);
            int written = snprintf(
                message,
                sizeof(message),
                "Lazy proxy factory must return an instance of a class compatible with %s, %s returned",
                object->class_name,
                type_name
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            ptn_throw_exception(runtime, "TypeError", message);
            return 0;
        }
        if (real.as.object == object) {
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            ptn_throw_exception(runtime, "Error", "Lazy proxy factory must return a non-lazy object");
            return 0;
        }
        if (!ptn_lazy_object_real_instance_compatible(runtime, object, real)) {
            char message[512];
            int written = snprintf(
                message,
                sizeof(message),
                "The real instance class %s is not compatible with the proxy class %s. The proxy must be a instance of the same class as the real instance, or a sub-class with no additional properties, and no overrides of the __destructor or __clone methods.",
                real.as.object->class_name,
                object->class_name
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            ptn_throw_exception(runtime, "TypeError", message);
            return 0;
        }
        ptn_value_destroy(&object->lazy_proxy_instance);
        object->lazy_proxy_instance = ptn_value_clone_deref(real);
        ptn_lazy_object_copy_properties_from_instance(object, real.as.object);
    } else {
        PtnValue returned = ptn_value_deref(result);
        if (returned.type != PTN_NULL) {
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            ptn_throw_exception(
                runtime,
                "TypeError",
                "Lazy object initializer must return NULL or no value"
            );
            return 0;
        }
    }
    ptn_value_destroy(&result);
    ptn_value_destroy(&object->lazy_initializer);
    object->lazy_initializer = ptn_null();
    object->lazy_uninitialized = 0;
    object->lazy_initializing = 0;
    return 1;
#else
    object->lazy_initializing = 0;
    ptn_throw_exception(runtime, "Error", "Lazy object initializer dispatch is unavailable");
    return 0;
#endif
}

static PTN_UNUSED PtnValue ptn_enum_case_with_backing(
    PtnRuntime *runtime,
    const char *class_name,
    const char *case_name,
    int has_backing,
    PtnPropertyTypeKind backing_type_kind,
    const char *backing_type_text,
    PtnValue backing_value
) {
    PtnValue value = ptn_object_new_shell(runtime, class_name);
    value.as.object->enum_case_name = ptn_duplicate_string(case_name);
    ptn_object_register_property_metadata(
        value.as.object,
        "name",
        class_name,
        PTN_PROPERTY_PUBLIC,
        PTN_PROPERTY_PUBLIC,
        1,
        PTN_PROPERTY_TYPE_STRING,
        NULL,
        "string",
        0
    );
    ptn_array_set_entry(
        value.as.object->properties,
        ptn_array_string_key("name"),
        ptn_string(case_name)
    );
    if (has_backing) {
        ptn_object_register_property_metadata(
            value.as.object,
            "value",
            class_name,
            PTN_PROPERTY_PUBLIC,
            PTN_PROPERTY_PUBLIC,
            1,
            backing_type_kind,
            NULL,
            backing_type_text,
            0
        );
        ptn_array_set_entry(
            value.as.object->properties,
            ptn_array_string_key("value"),
            ptn_value_clone_deref(backing_value)
        );
    }
    return value;
}

static PTN_UNUSED PtnValue ptn_enum_case(
    PtnRuntime *runtime,
    const char *class_name,
    const char *case_name
) {
    return ptn_enum_case_with_backing(
        runtime,
        class_name,
        case_name,
        0,
        PTN_PROPERTY_TYPE_NONE,
        NULL,
        ptn_null()
    );
}

static PTN_UNUSED char *ptn_object_private_storage_key(
    const char *declaring_class,
    const char *property
) {
    size_t declaring_len = strlen(declaring_class);
    size_t property_len = strlen(property);
    const char *prefix = "__ptn_private:";
    size_t prefix_len = strlen(prefix);
    if (declaring_len > SIZE_MAX - prefix_len - property_len - 2) {
        ptn_abort_out_of_memory();
    }
    size_t len = prefix_len + declaring_len + 1 + property_len;
    char *storage_name = malloc(len + 1);
    if (storage_name == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(storage_name, prefix, prefix_len);
    memcpy(storage_name + prefix_len, declaring_class, declaring_len);
    storage_name[prefix_len + declaring_len] = ':';
    memcpy(storage_name + prefix_len + declaring_len + 1, property, property_len);
    storage_name[len] = '\0';
    return storage_name;
}

static PTN_UNUSED char *ptn_object_storage_key_for_declaration(
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility
) {
    if (read_visibility == PTN_PROPERTY_PRIVATE) {
        return ptn_object_private_storage_key(declaring_class, property);
    }
    return ptn_duplicate_string(property);
}

static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_property_metadata(
    PtnObject *object,
    const char *storage_name
) {
    if (object == NULL || storage_name == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        if (strcmp(object->property_metadata[i].storage_name, storage_name) == 0) {
            return &object->property_metadata[i];
        }
    }
    return NULL;
}

static PTN_UNUSED void ptn_object_register_property_metadata(
    PtnObject *object,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    int is_readonly,
    PtnPropertyTypeKind type_kind,
    const char *type_class_name,
    const char *type_text,
    int type_allows_null
) {
    if (object == NULL || property == NULL || declaring_class == NULL) {
        return;
    }
    char *storage_name = ptn_object_storage_key_for_declaration(
        property,
        declaring_class,
        read_visibility
    );
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        if (strcmp(object->property_metadata[i].storage_name, storage_name) == 0) {
            free(storage_name);
            free(object->property_metadata[i].display_name);
            free(object->property_metadata[i].declaring_class);
            object->property_metadata[i].display_name = ptn_duplicate_string(property);
            object->property_metadata[i].declaring_class = ptn_duplicate_string(declaring_class);
            object->property_metadata[i].read_visibility = read_visibility;
            object->property_metadata[i].set_visibility = set_visibility;
            object->property_metadata[i].is_readonly = is_readonly;
            object->property_metadata[i].is_unset = 0;
            object->property_metadata[i].lazy_skip = 0;
            object->property_metadata[i].has_hooks = 0;
            object->property_metadata[i].is_virtual = 0;
            object->property_metadata[i].hook_has_get = 0;
            object->property_metadata[i].hook_has_set = 0;
            object->property_metadata[i].hook_set_uses_return = 0;
            free(object->property_metadata[i].last_type_name);
            object->property_metadata[i].last_type_name = NULL;
            free(object->property_metadata[i].type_class_name);
            free(object->property_metadata[i].type_text);
            object->property_metadata[i].type_kind = type_kind;
            object->property_metadata[i].type_class_name =
                type_class_name == NULL ? NULL : ptn_duplicate_string(type_class_name);
            object->property_metadata[i].type_text =
                type_text == NULL ? NULL : ptn_duplicate_string(type_text);
            object->property_metadata[i].type_allows_null = type_allows_null;
            return;
        }
    }
    if (object->property_metadata_len == object->property_metadata_capacity) {
        size_t new_capacity = object->property_metadata_capacity == 0
            ? 4
            : object->property_metadata_capacity * 2;
        if (new_capacity < object->property_metadata_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnObjectPropertyMetadata)) {
            ptn_abort_out_of_memory();
        }
        PtnObjectPropertyMetadata *new_metadata = realloc(
            object->property_metadata,
            new_capacity * sizeof(PtnObjectPropertyMetadata)
        );
        if (new_metadata == NULL) {
            ptn_abort_out_of_memory();
        }
        object->property_metadata = new_metadata;
        object->property_metadata_capacity = new_capacity;
    }
    PtnObjectPropertyMetadata *metadata =
        &object->property_metadata[object->property_metadata_len++];
    metadata->storage_name = storage_name;
    metadata->display_name = ptn_duplicate_string(property);
    metadata->declaring_class = ptn_duplicate_string(declaring_class);
    metadata->read_visibility = read_visibility;
    metadata->set_visibility = set_visibility;
    metadata->is_readonly = is_readonly;
    metadata->is_unset = 0;
    metadata->lazy_skip = 0;
    metadata->has_hooks = 0;
    metadata->is_virtual = 0;
    metadata->hook_has_get = 0;
    metadata->hook_has_set = 0;
    metadata->hook_set_uses_return = 0;
    metadata->last_type_name = NULL;
    metadata->type_kind = type_kind;
    metadata->type_class_name = type_class_name == NULL ? NULL : ptn_duplicate_string(type_class_name);
    metadata->type_text = type_text == NULL ? NULL : ptn_duplicate_string(type_text);
    metadata->type_allows_null = type_allows_null;
}

static PTN_UNUSED void ptn_object_register_property_hooks(
    PtnObject *object,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    int is_virtual,
    int hook_has_get,
    int hook_has_set,
    int hook_set_uses_return
) {
    if (object == NULL || property == NULL || declaring_class == NULL) {
        return;
    }
    char *storage_name = ptn_object_storage_key_for_declaration(
        property,
        declaring_class,
        read_visibility
    );
    PtnObjectPropertyMetadata *metadata = NULL;
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        if (strcmp(object->property_metadata[i].storage_name, storage_name) == 0) {
            metadata = &object->property_metadata[i];
            break;
        }
    }
    free(storage_name);
    if (metadata == NULL) {
        return;
    }
    metadata->has_hooks = 1;
    metadata->is_virtual = is_virtual ? 1 : 0;
    metadata->hook_has_get = hook_has_get ? 1 : 0;
    metadata->hook_has_set = hook_has_set ? 1 : 0;
    metadata->hook_set_uses_return = hook_set_uses_return ? 1 : 0;
}

static PTN_UNUSED PtnArrayKey ptn_array_key_clone(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_array_int_key(key.as.integer);
    }
    return ptn_array_string_key_len(key.as.string, key.string_len);
}

static PTN_UNUSED PtnArray *ptn_array_clone(PtnArray *source) {
    PtnArray *array = malloc(sizeof(PtnArray));
    if (array == NULL) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_note_array_alloc();
    ptn_cow_debug_note_array_clone();
    array->refcount = 1;
    array->debug_hidden_refcount = 0;
    array->debug_reference_wrapped = 0;
    array->iterator_refcount = 0;
    array->len = 0;
    array->capacity = source->len;
    array->entries = NULL;
    array->index_slots = NULL;
    array->index_capacity = 0;
    array->next_auto_key = 0;
    array->current_index = source->current_index <= source->len ? source->current_index : source->len;
    array->has_iterator_current_index = 0;
    array->iterator_current_index = 0;
    array->iterator_mutation_resume_index = 0;
    array->iterator_mutation_epoch = 0;
    array->mutation_epoch = 0;
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

static PTN_UNUSED void ptn_array_retain(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    ptn_cow_debug_assert_array_refcount(array, "retain");
    if (array->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_note_array_retain();
    array->refcount++;
}

static PTN_UNUSED void ptn_object_retain(PtnObject *object) {
    if (object == NULL) {
        return;
    }
    if (object->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    object->refcount++;
}

static PTN_UNUSED void ptn_object_release(PtnObject *object) {
    if (object == NULL) {
        return;
    }
    if (object->refcount == 0) {
        return;
    }
    object->refcount--;
    if (object->refcount != 0) {
        return;
    }
    object->refcount = 1;
    ptn_object_run_destructor(object);
    if (object->refcount > 1) {
        object->refcount--;
        return;
    }
    object->refcount = 0;
    ptn_runtime_unregister_object(object->lifecycle_runtime, object);
    ptn_runtime_release_object_id(object->lifecycle_runtime, object->object_id);
    if (object->native_data_free != NULL) {
        object->native_data_free(object->native_data);
    }
    ptn_value_destroy(&object->lazy_initializer);
    ptn_value_destroy(&object->lazy_proxy_instance);
    free(object->class_name);
    free(object->enum_case_name);
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        free(object->property_metadata[i].storage_name);
        free(object->property_metadata[i].display_name);
        free(object->property_metadata[i].declaring_class);
        free(object->property_metadata[i].last_type_name);
        free(object->property_metadata[i].type_class_name);
        free(object->property_metadata[i].type_text);
    }
    free(object->property_metadata);
    ptn_array_free(object->properties);
    free(object);
}

static PTN_UNUSED void ptn_array_debug_hide_ref(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    if (array->debug_hidden_refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    array->debug_hidden_refcount++;
}

static PTN_UNUSED void ptn_array_debug_unhide_ref(PtnArray *array) {
    if (array == NULL || array->debug_hidden_refcount == 0) {
        return;
    }
    array->debug_hidden_refcount--;
}

static PTN_UNUSED void ptn_value_debug_hide_ref(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_array_debug_hide_ref(value.as.array);
    }
}

static PTN_UNUSED void ptn_value_debug_unhide_ref(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_array_debug_unhide_ref(value.as.array);
    }
}

static PTN_UNUSED size_t ptn_array_debug_visible_refcount(PtnArray *array) {
    if (array == NULL) {
        return 0;
    }
    if (array->debug_hidden_refcount >= array->refcount) {
        return 1;
    }
    return array->refcount - array->debug_hidden_refcount;
}

static PTN_UNUSED void ptn_array_debug_note_reference_wrapped(PtnArray *array) {
    if (array != NULL) {
        array->debug_reference_wrapped = 1;
    }
}

static PTN_UNUSED void ptn_value_debug_note_reference_wrapped(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_array_debug_note_reference_wrapped(value.as.array);
    }
}

static PTN_UNUSED void ptn_array_iterator_retain(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    if (array->iterator_refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    array->iterator_refcount++;
}

static PTN_UNUSED PtnArray *ptn_array_detach_value(PtnValue *value) {
    if (value == NULL || !value->owned || value->type != PTN_ARRAY || value->as.array == NULL) {
        return NULL;
    }
    PtnArray *array = value->as.array;
    ptn_cow_debug_assert_array_refcount(array, "detach");
    if (array->refcount <= 1) {
        ptn_cow_debug_note_array_detach_skip();
        return array;
    }

    ptn_cow_debug_note_array_detach();
    PtnArray *detached = ptn_array_clone(array);
    ptn_array_note_mutation(array);
    ptn_value_destroy(value);
    *value = ptn_array(detached);
    return detached;
}

static PTN_UNUSED PtnValue ptn_value_deep_clone(PtnValue value) {
    switch (value.type) {
        case PTN_REFERENCE:
            value.as.reference->refcount++;
            value.owned = 1;
            return value;
        case PTN_STRING:
            return ptn_owned_string_len(
                ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len),
                value.as.string.len
            );
        case PTN_ARRAY:
            return ptn_array(ptn_array_clone(value.as.array));
        case PTN_OBJECT:
            ptn_object_retain(value.as.object);
            return ptn_object(value.as.object);
        case PTN_CLOSURE:
            value.as.closure->refcount++;
            value.owned = 1;
            return value;
        case PTN_EXCEPTION:
            ptn_exception_retain(value.as.exception);
            return ptn_exception_value(value.as.exception);
        case PTN_RESOURCE:
            ptn_resource_retain(value.as.resource);
            return ptn_resource(value.as.resource);
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
            return value;
    }
    return value;
}

static PTN_UNUSED PtnValue ptn_value_share(PtnValue value) {
    switch (value.type) {
        case PTN_REFERENCE:
            value.as.reference->refcount++;
            value.owned = 1;
            return value;
        case PTN_STRING:
            if (value.as.string.payload != NULL) {
                ptn_string_payload_retain(value.as.string.payload);
                value.owned = 1;
                return value;
            }
            value.owned = 0;
            return value;
        case PTN_ARRAY:
            ptn_array_retain(value.as.array);
            return ptn_array(value.as.array);
        case PTN_OBJECT:
            ptn_object_retain(value.as.object);
            return ptn_object(value.as.object);
        case PTN_CLOSURE:
            value.as.closure->refcount++;
            value.owned = 1;
            return value;
        case PTN_EXCEPTION:
            ptn_exception_retain(value.as.exception);
            return ptn_exception_value(value.as.exception);
        case PTN_RESOURCE:
            ptn_resource_retain(value.as.resource);
            return ptn_resource(value.as.resource);
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
            return value;
    }
    return value;
}

static PTN_UNUSED PtnValue ptn_value_clone(PtnValue value) {
    return ptn_value_share(value);
}

static PTN_UNUSED PtnValue ptn_value_snapshot_for_array_path_write(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY || value.type == PTN_OBJECT) {
        return ptn_value_clone(value);
    }
    return value;
}

static PTN_UNUSED PtnArray *ptn_value_detach_array(PtnValue *value) {
    return ptn_array_detach_value(value);
}
