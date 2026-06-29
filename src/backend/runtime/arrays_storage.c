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
static PTN_UNUSED PtnReference *ptn_reference_new_owned(PtnValue value);
static PTN_UNUSED int ptn_reference_assign(PtnRuntime *runtime, PtnReference *reference, PtnValue value);
static PTN_UNUSED int ptn_reference_assign_result(PtnRuntime *runtime, PtnReference *reference, PtnValue value, PtnValue *result_out);
static PTN_UNUSED void ptn_reference_release(PtnReference *reference);
static PTN_UNUSED void ptn_gc_drain_pending_destructor_array_cycles(PtnRuntime *runtime);
static PTN_UNUSED void ptn_value_destroy(PtnValue *value);
static PTN_UNUSED void ptn_value_drop(PtnValue *value);
static PTN_UNUSED PtnArrayKey ptn_array_key_clone(PtnArrayKey key);
static PTN_UNUSED PtnArray *ptn_array_clone(PtnArray *source);
static PTN_UNUSED void ptn_array_free(PtnArray *array);
static PTN_UNUSED void ptn_array_retain(PtnArray *array);
static PTN_UNUSED void ptn_object_retain(PtnObject *object);
static PTN_UNUSED void ptn_object_release(PtnObject *object);
static PTN_UNUSED void ptn_gc_attach_value_runtime(PtnRuntime *runtime, PtnValue value, size_t depth);
static PTN_UNUSED void ptn_runtime_unregister_array(PtnRuntime *runtime, PtnArray *array);
static PTN_UNUSED void ptn_runtime_unregister_reference(PtnRuntime *runtime, PtnReference *reference);
static PTN_UNUSED void ptn_object_register_property_metadata(
    PtnObject *object,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    int is_readonly,
    int has_hooks,
    int is_virtual,
    int hook_has_get,
    int hook_get_returns_by_ref,
    int hook_has_set,
    const char *hook_get_declaring_class,
    const char *hook_set_declaring_class,
    PtnPropertyTypeKind type_kind,
    const char *type_class_name,
    const char *type_text,
    int type_allows_null
);
static PTN_UNUSED void ptn_emit_array_offset_key_conversion_diagnostic(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int emit_null_key_deprecation
);
static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_property_metadata(
    PtnObject *object,
    const char *storage_name
);
static PTN_UNUSED int ptn_lazy_object_initialize(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
);
static PTN_UNUSED void ptn_runtime_register_object(PtnRuntime *runtime, PtnObject *object);
static PTN_UNUSED void ptn_runtime_unregister_object(PtnRuntime *runtime, PtnObject *object);
static void ptn_runtime_remove_live_object_at(PtnRuntime *root, size_t index);
static PTN_UNUSED void ptn_runtime_register_closure(PtnRuntime *runtime, PtnClosure *closure);
static PTN_UNUSED void ptn_runtime_unregister_closure(PtnRuntime *runtime, PtnClosure *closure);
static PTN_UNUSED void ptn_runtime_push_temporary_root(PtnRuntime *runtime, PtnValue value);
static PTN_UNUSED void ptn_runtime_push_owned_temporary_root(PtnRuntime *runtime, PtnValue *value);
static PTN_UNUSED void ptn_runtime_pop_temporary_root(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_object_destructors_until_output_buffer(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_unreferenced_object_destructors(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_unreferenced_object_destructors_for_unwind(PtnRuntime *runtime);
static PTN_UNUSED void ptn_runtime_run_object_destructors(PtnRuntime *runtime);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED void ptn_fiber_force_close(PtnRuntime *runtime, PtnObject *object);
#endif

typedef struct PtnFiberData {
    PtnObject *object;
    PtnValue callback;
    PtnValue return_value;
    PtnValue suspension_trace;
    PtnValue suspend_value;
    PtnValue resume_value;
    PtnValue *entry_args;
    size_t entry_argc;
    size_t entry_line;
    char *executing_file;
    size_t executing_line;
    int started;
    int running;
    int completed;
    int threw;
    int resume_credit;
    int force_closing;
    int force_close_unwind;
#if !defined(_WIN32)
    ucontext_t caller_context;
    ucontext_t fiber_context;
    void *fiber_stack;
    size_t fiber_stack_size;
    int context_initialized;
    int context_finished;
    PtnRuntime *context_runtime;
    PtnTryFrame *caller_try_frame;
    PtnTryFrame *suspended_try_frame;
    PtnTraceFrame *caller_trace_frame;
    PtnTraceFrame *suspended_trace_frame;
    PtnObject *caller_fiber;
    PtnGenerator *caller_generator;
    PtnGenerator *suspended_generator;
    int caller_diagnostics_suppressed;
#endif
} PtnFiberData;

static PtnException *ptn_exception_previous_exception(PtnException *exception) {
    if (exception == NULL) {
        return NULL;
    }
    PtnValue previous = ptn_value_deref(exception->previous);
    return previous.type == PTN_EXCEPTION ? previous.as.exception : NULL;
}

static int ptn_exception_previous_chain_would_recurse(PtnException *previous, PtnException *exception) {
    PtnException *cursor = previous;
    PtnException *slow = previous;
    PtnException *fast = previous;
    while (cursor != NULL) {
        if (cursor == exception) {
            return 1;
        }
        cursor = ptn_exception_previous_exception(cursor);
        slow = ptn_exception_previous_exception(slow);
        fast = ptn_exception_previous_exception(ptn_exception_previous_exception(fast));
        if (slow != NULL && slow == fast) {
            return 1;
        }
    }
    return 0;
}

static void ptn_exception_chain_previous_if_missing(PtnException *exception, PtnException *previous) {
    if (exception == NULL || previous == NULL || exception == previous) {
        return;
    }
    PtnValue existing = ptn_value_deref(exception->previous);
    if (existing.type != PTN_NULL) {
        return;
    }
    if (ptn_exception_previous_chain_would_recurse(previous, exception)) {
        return;
    }
    ptn_value_destroy(&exception->previous);
    exception->previous = ptn_value_clone_deref(ptn_exception_borrow(previous));
}
static PTN_UNUSED void ptn_runtime_prune_weak_maps_for_released_object(PtnRuntime *runtime);

#ifndef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED void ptn_runtime_prune_weak_maps_for_released_object(PtnRuntime *runtime) {
    (void)runtime;
}
#endif

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
    if (string == NULL || len == 0 || string[0] == '+' || memchr(string, '\0', len) != NULL) {
        return 0;
    }

    size_t digit_start = 0;
    if (string[0] == '-') {
        digit_start = 1;
    }
    if (digit_start == len) {
        return 0;
    }
    if (string[digit_start] == '0' && digit_start + 1 < len) {
        return 0;
    }
    if (len == 2 && string[0] == '-' && string[1] == '0') {
        return 0;
    }
    for (size_t i = digit_start; i < len; i++) {
        if (!isdigit((unsigned char)string[i])) {
            return 0;
        }
    }

    char *copy = ptn_duplicate_string_len(string, len);
    char *end = NULL;
    errno = 0;
    long long parsed = strtoll(copy, &end, 10);
    int ok = errno != ERANGE && end != copy && *end == '\0';
    free(copy);
    if (!ok) {
        return 0;
    }
    *integer = (int64_t)parsed;
    return 1;
}

static PTN_UNUSED void ptn_abort_illegal_array_key(void) {
    fputs("Fatal error: Illegal offset type\n", stderr);
    exit(255);
}

static PTN_UNUSED int64_t ptn_float_to_php_integer(double value) {
    if (!isfinite(value)) {
        return 0;
    }
    if (value >= -9223372036854775808.0 && value < 9223372036854775808.0) {
        return (int64_t)value;
    }

    double remainder = fmod(value, 18446744073709551616.0);
    if (remainder >= 9223372036854775808.0) {
        remainder -= 18446744073709551616.0;
    } else if (remainder < -9223372036854775808.0) {
        remainder += 18446744073709551616.0;
    }
    return (int64_t)remainder;
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
            return ptn_array_int_key(ptn_float_to_php_integer(value.as.floating));
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

static PTN_UNUSED uint64_t ptn_symbol_name_hash_len(const char *name, size_t name_len) {
    uint64_t hash = 1469598103934665603ULL ^ 0x7b2d6f8fe10b25c9ULL;
    const unsigned char *bytes = (const unsigned char *)name;
    for (size_t i = 0; i < name_len; i++) {
        hash ^= (uint64_t)bytes[i];
        hash *= 1099511628211ULL;
    }
    return ptn_hash_mix_uint64(hash);
}

static PTN_UNUSED uint64_t ptn_symbol_name_hash(const char *name) {
    return ptn_symbol_name_hash_len(name, strlen(name));
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

static PTN_UNUSED int ptn_array_highest_integer_key(PtnArray *array, int64_t *highest_out) {
    int found = 0;
    int64_t highest = 0;
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayKey key = array->entries[i].key;
        if (key.type != PTN_ARRAY_KEY_INT) {
            continue;
        }
        if (!found || key.as.integer > highest) {
            found = 1;
            highest = key.as.integer;
        }
    }
    if (found) {
        *highest_out = highest;
    }
    return found;
}

static PTN_UNUSED int64_t ptn_array_next_auto_key_after_integer(int64_t key) {
    return key < INT64_MAX ? key + 1 : INT64_MAX;
}

static PTN_UNUSED void ptn_array_update_next_auto_key(PtnArray *array, PtnArrayKey key) {
    if (key.type != PTN_ARRAY_KEY_INT) {
        return;
    }

    int64_t next = ptn_array_next_auto_key_after_integer(key.as.integer);
    if (next > array->next_auto_key) {
        array->next_auto_key = next;
        return;
    }

    if (key.as.integer < 0 && array->next_auto_key == 0) {
        int64_t highest = 0;
        if (!ptn_array_highest_integer_key(array, &highest) || key.as.integer > highest) {
            array->next_auto_key = next;
        }
    }
}

static PTN_UNUSED void ptn_array_recompute_next_auto_key(PtnArray *array) {
    int64_t highest = 0;
    if (ptn_array_highest_integer_key(array, &highest)) {
        array->next_auto_key = ptn_array_next_auto_key_after_integer(highest);
    } else {
        array->next_auto_key = 0;
    }
}

static PTN_UNUSED void ptn_array_note_mutation(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    array->mutation_epoch++;
}

static PTN_UNUSED void ptn_array_note_iterator_unset(PtnArray *array, size_t index) {
    if (array == NULL || array->iterator_refcount == 0 || !array->has_iterator_current_index) {
        return;
    }
    if (index < array->iterator_current_index) {
        array->iterator_current_index--;
        if (index < array->iterator_mutation_resume_index) {
            array->iterator_mutation_resume_index--;
        }
        return;
    }
    if (index == array->iterator_current_index) {
        array->iterator_mutation_resume_index = index;
        array->iterator_mutation_epoch++;
    }
}

static PTN_UNUSED void ptn_array_note_iterator_value_replacement(PtnArray *array) {
    if (array == NULL || array->iterator_refcount == 0 || !array->has_iterator_current_index) {
        return;
    }
    array->iterator_mutation_resume_index = array->iterator_current_index;
    array->iterator_mutation_epoch++;
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
    ptn_array_note_iterator_value_replacement(old_resolved.as.array);
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

static PTN_UNUSED void ptn_array_index_remove(PtnArray *array, PtnArrayKey key) {
    if (array->index_capacity == 0) {
        return;
    }

    uint64_t hash = ptn_array_key_hash(key);
    size_t mask = array->index_capacity - 1;
    size_t slot_index = (size_t)hash & mask;
    for (;;) {
        PtnArrayIndexSlot *slot = &array->index_slots[slot_index];
        if (!slot->occupied) {
            return;
        }
        if (slot->hash == hash && ptn_array_keys_equal(array->entries[slot->entry_index].key, key)) {
            slot->occupied = 0;
            break;
        }
        slot_index = (slot_index + 1) & mask;
    }

    size_t scan = (slot_index + 1) & mask;
    while (array->index_slots[scan].occupied) {
        PtnArrayIndexSlot moving = array->index_slots[scan];
        array->index_slots[scan].occupied = 0;
        size_t destination = ptn_array_index_slot_for_key(
            array,
            array->entries[moving.entry_index].key,
            moving.hash
        );
        array->index_slots[destination] = moving;
        scan = (scan + 1) & mask;
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

static PTN_UNUSED int ptn_array_entry_default_by_ref_argument_eligible(PtnValue value) {
    return value.type == PTN_REFERENCE;
}

static PTN_UNUSED void ptn_array_set_entry_with_by_ref_argument_eligibility(
    PtnArray *array,
    PtnArrayKey key,
    PtnValue value,
    int by_ref_argument_eligible
) {
    ptn_gc_attach_value_runtime(array == NULL ? NULL : array->lifecycle_runtime, value, 0);
    size_t index = ptn_array_find_key(array, key);
    ptn_array_update_next_auto_key(array, key);
    ptn_array_note_mutation(array);
    if (index < array->len) {
        ptn_value_destroy(&array->entries[index].value);
        array->entries[index].value = value;
        array->entries[index].by_ref_argument_eligible =
            value.type == PTN_REFERENCE && by_ref_argument_eligible;
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
    array->entries[entry_index].by_ref_argument_eligible =
        value.type == PTN_REFERENCE && by_ref_argument_eligible;
    array->len++;
    ptn_array_index_insert_appended_entry(array, key, entry_index);
}

static PTN_UNUSED void ptn_array_set_entry(PtnArray *array, PtnArrayKey key, PtnValue value) {
    ptn_array_set_entry_with_by_ref_argument_eligibility(
        array,
        key,
        value,
        ptn_array_entry_default_by_ref_argument_eligible(value)
    );
}

static PTN_UNUSED void ptn_array_set_entry_publish_first(PtnArray *array, PtnArrayKey key, PtnValue value) {
    ptn_gc_attach_value_runtime(array == NULL ? NULL : array->lifecycle_runtime, value, 0);
    size_t index = ptn_array_find_key(array, key);
    ptn_array_update_next_auto_key(array, key);
    ptn_array_note_mutation(array);
    if (index < array->len) {
        PtnValue old_value = array->entries[index].value;
        array->entries[index].value = value;
        array->entries[index].by_ref_argument_eligible =
            ptn_array_entry_default_by_ref_argument_eligible(value);
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

static PTN_UNUSED PtnValue ptn_array_value_clone_for_write(PtnArray *array, PtnValue value) {
    PtnValue resolved = ptn_value_deref(value);
    if (value.type == PTN_REFERENCE && resolved.type == PTN_ARRAY && resolved.as.array == array) {
        return ptn_value_clone(value);
    }
    return ptn_value_clone(resolved);
}

static PTN_UNUSED PtnValue ptn_array_write_entry_result(PtnRuntime *runtime, PtnArray *array, PtnArrayKey key, PtnValue value) {
    PtnValue stored = ptn_array_value_clone_for_write(array, value);
    size_t index = ptn_array_find_key(array, key);
    if (index < array->len && array->entries[index].value.type == PTN_REFERENCE) {
        PtnReference *reference = array->entries[index].value.as.reference;
        if (reference->refcount == SIZE_MAX) {
            ptn_abort_out_of_memory();
        }
        reference->refcount++;
        ptn_array_update_next_auto_key(array, key);
        PtnValue result = ptn_null();
        if (ptn_reference_assign_result(runtime, reference, stored, &result)) {
            ptn_value_destroy(&stored);
            ptn_array_key_free(key);
            ptn_reference_release(reference);
            return result;
        }
        ptn_value_destroy(&stored);
        ptn_array_key_free(key);
        ptn_reference_release(reference);
        return ptn_value_clone_deref(value);
    }
    PtnValue result = ptn_value_clone_deref(stored);
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
    ptn_array_note_iterator_unset(array, index);
    PtnArrayEntry removed = array->entries[index];
    for (size_t i = index + 1; i < array->len; i++) {
        array->entries[i - 1] = array->entries[i];
    }
    array->len--;
    if (array->current_index > index) {
        array->current_index--;
    } else if (array->current_index > array->len) {
        array->current_index = array->len;
    }
    ptn_array_key_free(key);
    ptn_array_rebuild_index(array);
    ptn_array_key_free(removed.key);
    ptn_value_destroy(&removed.value);
    return 1;
}

static PTN_UNUSED void ptn_emit_null_array_offset_deprecation(PtnRuntime *runtime, size_t line);
static PTN_UNUSED void ptn_emit_resource_offset_warning(PtnRuntime *runtime, PtnResource *resource, size_t line);
static PTN_UNUSED int ptn_array_append_key_available(PtnRuntime *runtime, PtnArray *array);
static PTN_UNUSED void ptn_throw_exception_at(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
);
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
);
static PTN_UNUSED PtnValue ptn_exception_previous_or_active(
    PtnRuntime *runtime,
    PtnValue previous
);

static PTN_UNUSED const char *ptn_array_key_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_ARRAY:
            return "array";
        case PTN_OBJECT:
            return value.as.object != NULL && value.as.object->class_name != NULL
                ? value.as.object->class_name
                : "stdClass";
        case PTN_CLOSURE:
            return "Closure";
        case PTN_EXCEPTION:
            return value.as.exception != NULL && value.as.exception->class_name != NULL
                ? value.as.exception->class_name
                : "Exception";
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
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED int ptn_array_key_from_literal_value(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    PtnArrayKey *key_out
) {
    key_value = ptn_value_deref(key_value);
    if (
        key_value.type == PTN_ARRAY ||
        key_value.type == PTN_OBJECT ||
        key_value.type == PTN_CLOSURE ||
        key_value.type == PTN_EXCEPTION
    ) {
        if (runtime == NULL) {
            ptn_abort_illegal_array_key();
        }
        char message[256];
        int written = snprintf(
            message,
            sizeof(message),
            "Cannot access offset of type %s on array",
            ptn_array_key_type_name(key_value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
        return 0;
    }
    *key_out = ptn_array_key_from_value(key_value);
    return 1;
}

static void ptn_runtime_remove_live_array_at(PtnRuntime *root, size_t index) {
    if (root == NULL || index >= root->live_arrays_len) {
        return;
    }
    root->live_arrays[index] = NULL;
    while (root->live_arrays_len > 0 &&
           root->live_arrays[root->live_arrays_len - 1] == NULL) {
        root->live_arrays_len--;
    }
}

static void ptn_runtime_remove_live_reference_at(PtnRuntime *root, size_t index) {
    if (root == NULL || index >= root->live_references_len) {
        return;
    }
    root->live_references[index] = NULL;
    while (root->live_references_len > 0 &&
           root->live_references[root->live_references_len - 1] == NULL) {
        root->live_references_len--;
    }
}

static PTN_UNUSED void ptn_runtime_register_array(PtnRuntime *runtime, PtnArray *array) {
    if (runtime == NULL || array == NULL || array->lifecycle_runtime != NULL) {
        return;
    }
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return;
    }
    if (root->live_arrays_len == root->live_arrays_capacity) {
        size_t new_capacity = root->live_arrays_capacity == 0
            ? 16
            : root->live_arrays_capacity * 2;
        if (new_capacity < root->live_arrays_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnArray *)) {
            ptn_abort_out_of_memory();
        }
        PtnArray **new_arrays = realloc(root->live_arrays, new_capacity * sizeof(PtnArray *));
        if (new_arrays == NULL) {
            ptn_abort_out_of_memory();
        }
        root->live_arrays = new_arrays;
        root->live_arrays_capacity = new_capacity;
    }
    array->live_index = root->live_arrays_len;
    root->live_arrays[root->live_arrays_len++] = array;
    array->lifecycle_runtime = root;
}

static PTN_UNUSED void ptn_runtime_unregister_array(PtnRuntime *runtime, PtnArray *array) {
    if (array == NULL) {
        return;
    }
    PtnRuntime *owner = runtime != NULL ? runtime : array->lifecycle_runtime;
    PtnRuntime *root = ptn_runtime_root(owner);
    if (root != NULL) {
        size_t index = array->live_index;
        if (index < root->live_arrays_len && root->live_arrays[index] == array) {
            ptn_runtime_remove_live_array_at(root, index);
        } else {
            size_t i = root->live_arrays_len;
            while (i > 0) {
                i--;
                if (root->live_arrays[i] == array) {
                    ptn_runtime_remove_live_array_at(root, i);
                    break;
                }
            }
        }
    }
    array->lifecycle_runtime = NULL;
    array->live_index = 0;
}

static PTN_UNUSED void ptn_runtime_register_reference(PtnRuntime *runtime, PtnReference *reference) {
    if (runtime == NULL || reference == NULL || reference->lifecycle_runtime != NULL) {
        return;
    }
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return;
    }
    if (root->live_references_len == root->live_references_capacity) {
        size_t new_capacity = root->live_references_capacity == 0
            ? 16
            : root->live_references_capacity * 2;
        if (new_capacity < root->live_references_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnReference *)) {
            ptn_abort_out_of_memory();
        }
        PtnReference **new_references = realloc(
            root->live_references,
            new_capacity * sizeof(PtnReference *)
        );
        if (new_references == NULL) {
            ptn_abort_out_of_memory();
        }
        root->live_references = new_references;
        root->live_references_capacity = new_capacity;
    }
    reference->live_index = root->live_references_len;
    root->live_references[root->live_references_len++] = reference;
    reference->lifecycle_runtime = root;
}

static PTN_UNUSED void ptn_runtime_unregister_reference(PtnRuntime *runtime, PtnReference *reference) {
    if (reference == NULL) {
        return;
    }
    PtnRuntime *owner = runtime != NULL ? runtime : reference->lifecycle_runtime;
    PtnRuntime *root = ptn_runtime_root(owner);
    if (root != NULL) {
        size_t index = reference->live_index;
        if (index < root->live_references_len && root->live_references[index] == reference) {
            ptn_runtime_remove_live_reference_at(root, index);
        } else {
            size_t i = root->live_references_len;
            while (i > 0) {
                i--;
                if (root->live_references[i] == reference) {
                    ptn_runtime_remove_live_reference_at(root, i);
                    break;
                }
            }
        }
    }
    reference->lifecycle_runtime = NULL;
    reference->live_index = 0;
}

static void ptn_gc_attach_symbol_table_runtime(PtnRuntime *runtime, PtnSymbolTable *symbols, size_t depth);

static PTN_UNUSED void ptn_gc_attach_value_runtime(PtnRuntime *runtime, PtnValue value, size_t depth) {
    if (runtime == NULL || depth > 1024) {
        return;
    }
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return;
    }
    if (value.type == PTN_REFERENCE) {
        PtnReference *reference = value.as.reference;
        if (reference == NULL) {
            return;
        }
        int already_owned = reference->lifecycle_runtime == root;
        ptn_runtime_register_reference(root, reference);
        if (!already_owned) {
            ptn_gc_attach_value_runtime(root, reference->value, depth + 1);
        }
        return;
    }
    if (value.type == PTN_ARRAY) {
        PtnArray *array = value.as.array;
        if (array == NULL) {
            return;
        }
        int already_owned = array->lifecycle_runtime == root;
        ptn_runtime_register_array(root, array);
        if (already_owned) {
            return;
        }
        for (size_t i = 0; i < array->len; i++) {
            ptn_gc_attach_value_runtime(root, array->entries[i].value, depth + 1);
        }
        return;
    }
    if (value.type == PTN_OBJECT) {
        PtnObject *object = value.as.object;
        if (object == NULL) {
            return;
        }
        ptn_gc_attach_value_runtime(root, ptn_array(object->properties), depth + 1);
        ptn_gc_attach_value_runtime(root, object->lazy_initializer, depth + 1);
        ptn_gc_attach_value_runtime(root, object->lazy_proxy_instance, depth + 1);
        return;
    }
    if (value.type == PTN_CLOSURE) {
        PtnClosure *closure = value.as.closure;
        if (closure == NULL) {
            return;
        }
        ptn_gc_attach_symbol_table_runtime(root, &closure->captures, depth + 1);
        if (closure->has_wrapped_callable) {
            ptn_gc_attach_value_runtime(root, closure->wrapped_callable, depth + 1);
        }
    }
}

static void ptn_gc_attach_symbol_table_runtime(PtnRuntime *runtime, PtnSymbolTable *symbols, size_t depth) {
    if (symbols == NULL || depth > 1024) {
        return;
    }
    for (size_t i = 0; i < symbols->len; i++) {
        ptn_gc_attach_value_runtime(runtime, symbols->items[i].value, depth + 1);
    }
}

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
    array->destructing = 0;
    array->gc_mark_epoch = 0;
    array->lifecycle_runtime = NULL;
    array->live_index = 0;
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
    ptn_runtime_register_array(runtime, array);
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
            ptn_emit_array_offset_key_conversion_diagnostic(runtime, key_value, line, 1);
        }
        PtnArrayKey key;
        if (entries[i].has_key) {
            if (!ptn_array_key_from_literal_value(runtime, key_value, line, &key)) {
                continue;
            }
        } else {
            if (runtime != NULL && !ptn_array_append_key_available(runtime, array)) {
                continue;
            }
            key = ptn_array_int_key(array->next_auto_key);
        }
        PtnValue stored = ptn_value_clone(entries[i].value);
        ptn_gc_attach_value_runtime(runtime, stored, 0);
        if (
            entries[i].value.type == PTN_STRING &&
            entries[i].value.as.string.payload == NULL &&
            entries[i].value.as.string.len > 1 &&
            stored.type == PTN_STRING &&
            stored.as.string.payload != NULL
        ) {
            stored.as.string.payload->interned = 0;
        }
        ptn_array_set_entry(array, key, stored);
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
    object->live_index = root->live_objects_len;
    root->live_objects[root->live_objects_len++] = object;
}

static PTN_UNUSED void ptn_runtime_unregister_object(PtnRuntime *runtime, PtnObject *object) {
    if (runtime == NULL || object == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    size_t index = object->live_index;
    if (index < root->live_objects_len && root->live_objects[index] == object) {
        ptn_runtime_remove_live_object_at(root, index);
        object->live_index = 0;
        return;
    }
    size_t i = root->live_objects_len;
    while (i > 0) {
        i--;
        if (root->live_objects[i] == object) {
            ptn_runtime_remove_live_object_at(root, i);
            object->live_index = 0;
            return;
        }
    }
}

static PTN_UNUSED void ptn_runtime_register_closure(PtnRuntime *runtime, PtnClosure *closure) {
    if (runtime == NULL || closure == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    if (root->live_closures_len == root->live_closures_capacity) {
        size_t new_capacity = root->live_closures_capacity == 0
            ? 8
            : root->live_closures_capacity * 2;
        if (new_capacity < root->live_closures_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnClosure *)) {
            ptn_abort_out_of_memory();
        }
        PtnClosure **new_closures = realloc(
            root->live_closures,
            new_capacity * sizeof(PtnClosure *)
        );
        if (new_closures == NULL) {
            ptn_abort_out_of_memory();
        }
        root->live_closures = new_closures;
        root->live_closures_capacity = new_capacity;
    }
    root->live_closures[root->live_closures_len++] = closure;
}

static PTN_UNUSED void ptn_runtime_unregister_closure(PtnRuntime *runtime, PtnClosure *closure) {
    if (runtime == NULL || closure == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    for (size_t i = 0; i < root->live_closures_len; i++) {
        if (root->live_closures[i] != closure) {
            continue;
        }
        for (size_t j = i + 1; j < root->live_closures_len; j++) {
            root->live_closures[j - 1] = root->live_closures[j];
        }
        root->live_closures_len--;
        return;
    }
}

static PTN_UNUSED void ptn_runtime_push_temporary_root(PtnRuntime *runtime, PtnValue value) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    if (root->temporary_roots_len == root->temporary_roots_capacity) {
        size_t new_capacity = root->temporary_roots_capacity == 0
            ? 8
            : root->temporary_roots_capacity * 2;
        if (new_capacity < root->temporary_roots_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnValue)) {
            ptn_abort_out_of_memory();
        }
        PtnValue *new_roots = realloc(
            root->temporary_roots,
            new_capacity * sizeof(PtnValue)
        );
        if (new_roots == NULL) {
            ptn_abort_out_of_memory();
        }
        root->temporary_roots = new_roots;
        root->temporary_roots_capacity = new_capacity;
    }
    root->temporary_roots[root->temporary_roots_len++] = ptn_value_clone(value);
}

static PTN_UNUSED void ptn_runtime_push_owned_temporary_root(
    PtnRuntime *runtime,
    PtnValue *value
) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    if (root->temporary_roots_len == root->temporary_roots_capacity) {
        size_t new_capacity = root->temporary_roots_capacity == 0
            ? 8
            : root->temporary_roots_capacity * 2;
        if (new_capacity < root->temporary_roots_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnValue)) {
            ptn_abort_out_of_memory();
        }
        PtnValue *new_roots = realloc(
            root->temporary_roots,
            new_capacity * sizeof(PtnValue)
        );
        if (new_roots == NULL) {
            ptn_abort_out_of_memory();
        }
        root->temporary_roots = new_roots;
        root->temporary_roots_capacity = new_capacity;
    }
    if (value == NULL || value->owned <= 0) {
        root->temporary_roots[root->temporary_roots_len++] = ptn_null();
        return;
    }
    root->temporary_roots[root->temporary_roots_len++] = *value;
    value->owned = 0;
    value->by_ref_return_fallback = 0;
    value->from_string_offset = 0;
}

static PTN_UNUSED void ptn_runtime_pop_temporary_root(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    if (root->temporary_roots_len == 0) {
        return;
    }
    root->temporary_roots_len--;
    ptn_value_destroy(&root->temporary_roots[root->temporary_roots_len]);
}

static PTN_UNUSED void ptn_runtime_clear_temporary_roots(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    while (root->temporary_roots_len > 0) {
        ptn_runtime_pop_temporary_root(root);
    }
    if (
        runtime->defer_unreferenced_destructors_for_catch ||
        root->defer_unreferenced_destructors_for_catch
    ) {
        runtime->defer_unreferenced_destructors_for_catch = 0;
        root->defer_unreferenced_destructors_for_catch = 0;
        return;
    }
    ptn_runtime_run_unreferenced_object_destructors_for_unwind(runtime);
}

static PTN_UNUSED void ptn_object_run_destructor_ex(PtnObject *object, int during_shutdown) {
    if (object == NULL || !object->destructor_enabled || object->destructor_called) {
        return;
    }
    if (object->lazy_uninitialized && !object->lazy_initializing) {
        return;
    }
    if (object->lazy_is_proxy && !object->lazy_uninitialized) {
        PtnValue real = ptn_value_deref(object->lazy_proxy_instance);
        if (real.type == PTN_OBJECT && real.as.object != NULL && real.as.object != object) {
            object->destructor_called = 1;
            ptn_object_run_destructor_ex(real.as.object, during_shutdown);
            return;
        }
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
    const char *previous_scope = root->current_class_name;
    int previous_shutdown_phase = root->destructor_shutdown_phase;
    int previous_suppress_user_call_frame_location =
        root->suppress_user_call_frame_location;
    root->current_class_name = root->destructor_access_scope;
    root->destructor_shutdown_phase = during_shutdown;
    if (during_shutdown) {
        root->suppress_user_call_frame_location = 1;
    }
    PtnException *saved_active_exception = root->exceptions == NULL
        ? NULL
        : root->exceptions->active_exception;
    PtnValue result = ptn_null();
    PtnTryFrame destructor_frame;
    int catch_gc_exception = root->gc_running;
    int preserve_active_exception =
        saved_active_exception != NULL &&
        root->defer_uncaught_exception_emit &&
        !catch_gc_exception;
    int catch_destructor_exception =
        catch_gc_exception ||
        (saved_active_exception != NULL && !preserve_active_exception);
    if (saved_active_exception != NULL && !preserve_active_exception) {
        root->exceptions->active_exception = NULL;
    }
    if (catch_destructor_exception) {
        ptn_try_frame_push(root, &destructor_frame);
        if (setjmp(destructor_frame.jump) != 0) {
            ptn_try_frame_pop(root, &destructor_frame);
            if (saved_active_exception != NULL) {
                if (root->exceptions->active_exception != NULL) {
                    ptn_exception_chain_previous_if_missing(
                        root->exceptions->active_exception,
                        saved_active_exception
                    );
                    ptn_exception_free(saved_active_exception);
                } else {
                    root->exceptions->active_exception = saved_active_exception;
                }
            }
            root->suppress_user_call_frame_location =
                previous_suppress_user_call_frame_location;
            root->destructor_shutdown_phase = previous_shutdown_phase;
            root->current_class_name = previous_scope;
            ptn_value_destroy(&result);
            return;
        }
    }
    result = root->method_dispatch(root, receiver, "__destruct", 0, NULL, destructor_line);
    if (catch_destructor_exception) {
        ptn_try_frame_pop(root, &destructor_frame);
    }
    root->suppress_user_call_frame_location =
        previous_suppress_user_call_frame_location;
    root->destructor_shutdown_phase = previous_shutdown_phase;
    root->current_class_name = previous_scope;
    if (saved_active_exception != NULL && !preserve_active_exception) {
        if (root->exceptions->active_exception == NULL) {
            root->exceptions->active_exception = saved_active_exception;
        } else {
            ptn_exception_chain_previous_if_missing(
                root->exceptions->active_exception,
                saved_active_exception
            );
            ptn_exception_free(saved_active_exception);
        }
    }
    ptn_value_destroy(&result);
}

static PTN_UNUSED void ptn_object_run_destructor(PtnObject *object) {
    ptn_object_run_destructor_ex(object, 0);
}

static PTN_UNUSED size_t ptn_runtime_run_static_property_value_destructors(
    PtnValue value,
    size_t depth
);

static PTN_UNUSED void ptn_runtime_register_static_local(
    PtnRuntime *runtime,
    size_t function_index,
    const char *name,
    PtnReference *reference
) {
    if (runtime == NULL || reference == NULL) {
        return;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    for (size_t i = 0; i < root->static_local_slots_len; i++) {
        if (root->static_local_slots[i].reference == reference) {
            root->static_local_slots[i].function_index = function_index;
            root->static_local_slots[i].name = name;
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
    root->static_local_slots[root->static_local_slots_len].function_index = function_index;
    root->static_local_slots[root->static_local_slots_len].name = name;
    root->static_local_slots[root->static_local_slots_len].reference = reference;
    root->static_local_slots_len++;
}

static PTN_UNUSED size_t ptn_runtime_static_local_count(
    PtnRuntime *runtime,
    size_t function_index
) {
    if (runtime == NULL) {
        return 0;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    size_t count = 0;
    for (size_t i = 0; i < root->static_local_slots_len; i++) {
        PtnStaticLocalSlot *slot = &root->static_local_slots[i];
        if (
            slot->reference != NULL &&
            slot->name != NULL &&
            slot->function_index == function_index
        ) {
            count++;
        }
    }
    return count;
}

static PTN_UNUSED PtnValue ptn_runtime_static_local_values(
    PtnRuntime *runtime,
    size_t function_index,
    PtnFunctionMetadata metadata
) {
    PtnValue result = metadata.static_variables_provider == NULL
        ? ptn_array_from_literal_entries(0, NULL)
        : metadata.static_variables_provider(runtime);
    if (runtime == NULL) {
        return result;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    for (size_t i = 0; i < root->static_local_slots_len; i++) {
        PtnStaticLocalSlot *slot = &root->static_local_slots[i];
        if (
            slot->reference == NULL ||
            slot->name == NULL ||
            slot->function_index != function_index
        ) {
            continue;
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key(slot->name),
            ptn_value_clone(ptn_reference_value(slot->reference))
        );
    }
    return result;
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
        root->static_local_slots[i].function_index = 0;
        root->static_local_slots[i].name = NULL;
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
    root->live_objects[index] = NULL;
    while (root->live_objects_len > 0 &&
           root->live_objects[root->live_objects_len - 1] == NULL) {
        root->live_objects_len--;
    }
}

static int ptn_value_reaches_object(PtnValue value, PtnObject *target, size_t depth);

static int ptn_exception_reaches_object(PtnException *exception, PtnObject *target, size_t depth) {
    if (exception == NULL || target == NULL || depth > 1024) {
        return 0;
    }
    return ptn_value_reaches_object(exception->trace, target, depth + 1) ||
        ptn_value_reaches_object(exception->previous, target, depth + 1) ||
        ptn_value_reaches_object(exception->dynamic_properties, target, depth + 1) ||
        ptn_value_reaches_object(exception->errors, target, depth + 1) ||
        ptn_value_reaches_object(exception->soap_fault_headerfault, target, depth + 1);
}

static int ptn_object_native_values_reach_object(PtnObject *object, PtnObject *target, size_t depth) {
    if (object == NULL || target == NULL || object->native_data == NULL || depth > 1024) {
        return 0;
    }
    if (ptn_ascii_case_equal(object->class_name, "SensitiveParameterValue")) {
        typedef struct {
            PtnValue value;
        } PtnNativeSensitiveParameterValueData;
        PtnNativeSensitiveParameterValueData *data =
            (PtnNativeSensitiveParameterValueData *)object->native_data;
        return ptn_value_reaches_object(data->value, target, depth + 1);
    }
    if (ptn_ascii_case_equal(object->class_name, "Fiber")) {
        PtnFiberData *data = (PtnFiberData *)object->native_data;
        if (ptn_value_reaches_object(data->callback, target, depth + 1) ||
            ptn_value_reaches_object(data->return_value, target, depth + 1) ||
            ptn_value_reaches_object(data->suspension_trace, target, depth + 1) ||
            ptn_value_reaches_object(data->suspend_value, target, depth + 1) ||
            ptn_value_reaches_object(data->resume_value, target, depth + 1)) {
            return 1;
        }
        for (size_t i = 0; i < data->entry_argc; i++) {
            if (ptn_value_reaches_object(data->entry_args[i], target, depth + 1)) {
                return 1;
            }
        }
        return 0;
    }
    return 0;
}

static int ptn_array_reaches_object(PtnArray *array, PtnObject *target, size_t depth) {
    if (array == NULL || target == NULL || depth > 1024) {
        return 0;
    }
    for (size_t i = 0; i < array->len; i++) {
        if (ptn_value_reaches_object(array->entries[i].value, target, depth + 1)) {
            return 1;
        }
    }
    return 0;
}

static int ptn_value_reaches_object(PtnValue value, PtnObject *target, size_t depth) {
    if (target == NULL || depth > 1024) {
        return 0;
    }
    if (value.type == PTN_REFERENCE) {
        return value.as.reference != NULL &&
            ptn_value_reaches_object(value.as.reference->value, target, depth + 1);
    }
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT) {
        if (value.as.object == target) {
            return 1;
        }
        return value.as.object != NULL &&
            (
                ptn_array_reaches_object(value.as.object->properties, target, depth + 1) ||
                ptn_object_native_values_reach_object(value.as.object, target, depth + 1)
            );
    }
    if (value.type == PTN_ARRAY) {
        return ptn_array_reaches_object(value.as.array, target, depth + 1);
    }
    if (value.type == PTN_CLOSURE && value.as.closure != NULL) {
        PtnClosure *closure = value.as.closure;
        if (closure->has_wrapped_callable &&
            ptn_value_reaches_object(closure->wrapped_callable, target, depth + 1)) {
            return 1;
        }
        for (size_t i = 0; i < closure->captures.len; i++) {
            if (ptn_value_reaches_object(closure->captures.items[i].value, target, depth + 1)) {
                return 1;
            }
        }
    }
    if (value.type == PTN_EXCEPTION) {
        return ptn_exception_reaches_object(value.as.exception, target, depth + 1);
    }
    return 0;
}

static int ptn_symbol_table_reaches_object(PtnSymbolTable *symbols, PtnObject *target) {
    if (symbols == NULL || target == NULL) {
        return 0;
    }
    for (size_t i = 0; i < symbols->len; i++) {
        if (ptn_value_reaches_object(symbols->items[i].value, target, 0)) {
            return 1;
        }
    }
    return 0;
}

static int ptn_runtime_roots_reach_object(PtnRuntime *root, PtnObject *target) {
    if (root == NULL || target == NULL) {
        return 0;
    }
    if (ptn_symbol_table_reaches_object(&root->symbols, target)) {
        return 1;
    }
    if (
        root->global_symbols != NULL &&
        root->global_symbols != &root->symbols &&
        ptn_symbol_table_reaches_object(root->global_symbols, target)
    ) {
        return 1;
    }
    if (
        root->exceptions != NULL &&
        ptn_exception_reaches_object(root->exceptions->active_exception, target, 0)
    ) {
        return 1;
    }
    PtnSymbolTable *static_properties = root->static_properties == NULL
        ? &root->owned_static_properties
        : root->static_properties;
    return ptn_symbol_table_reaches_object(static_properties, target);
}

static void ptn_runtime_run_object_destructors_matching(
    PtnRuntime *runtime,
    int only_unreferenced,
    int during_shutdown
) {
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
            (object->refcount > 1 || ptn_runtime_roots_reach_object(runtime, object))
        ) {
            continue;
        }
        ptn_runtime_remove_live_object_at(root, index);
        if (object == NULL || object->refcount == 0 || object->destructor_called) {
            continue;
        }
        ptn_object_retain(object);
        ptn_object_run_destructor_ex(object, during_shutdown);
        ptn_object_release(object);
    }
}

static PTN_UNUSED void ptn_runtime_run_unreferenced_object_destructors(PtnRuntime *runtime) {
    ptn_runtime_run_object_destructors_matching(runtime, 1, 1);
}

static PTN_UNUSED void ptn_runtime_run_unreferenced_object_destructors_for_unwind(PtnRuntime *runtime) {
    ptn_runtime_run_object_destructors_matching(runtime, 1, 0);
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
        ptn_object_run_destructor_ex(object, 1);
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
    ptn_runtime_run_object_destructors_matching(runtime, 0, 1);
}

typedef struct {
    PtnArray **arrays;
    size_t arrays_len;
    size_t arrays_capacity;
    PtnReference **references;
    size_t references_len;
    size_t references_capacity;
} PtnShutdownDestructorScan;

static void ptn_shutdown_destructor_scan_free(PtnShutdownDestructorScan *scan) {
    if (scan == NULL) {
        return;
    }
    free(scan->arrays);
    scan->arrays = NULL;
    scan->arrays_len = 0;
    scan->arrays_capacity = 0;
    free(scan->references);
    scan->references = NULL;
    scan->references_len = 0;
    scan->references_capacity = 0;
}

static int ptn_shutdown_destructor_scan_note_array(
    PtnShutdownDestructorScan *scan,
    PtnArray *array
) {
    if (scan == NULL || array == NULL) {
        return 0;
    }
    for (size_t i = 0; i < scan->arrays_len; i++) {
        if (scan->arrays[i] == array) {
            return 0;
        }
    }
    if (scan->arrays_len == scan->arrays_capacity) {
        size_t new_capacity = scan->arrays_capacity == 0 ? 8 : scan->arrays_capacity * 2;
        if (new_capacity < scan->arrays_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnArray *)) {
            ptn_abort_out_of_memory();
        }
        PtnArray **new_arrays = realloc(scan->arrays, new_capacity * sizeof(PtnArray *));
        if (new_arrays == NULL) {
            ptn_abort_out_of_memory();
        }
        scan->arrays = new_arrays;
        scan->arrays_capacity = new_capacity;
    }
    scan->arrays[scan->arrays_len++] = array;
    return 1;
}

static int ptn_shutdown_destructor_scan_note_reference(
    PtnShutdownDestructorScan *scan,
    PtnReference *reference
) {
    if (scan == NULL || reference == NULL) {
        return 0;
    }
    for (size_t i = 0; i < scan->references_len; i++) {
        if (scan->references[i] == reference) {
            return 0;
        }
    }
    if (scan->references_len == scan->references_capacity) {
        size_t new_capacity = scan->references_capacity == 0
            ? 8
            : scan->references_capacity * 2;
        if (new_capacity < scan->references_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnReference *)) {
            ptn_abort_out_of_memory();
        }
        PtnReference **new_references = realloc(
            scan->references,
            new_capacity * sizeof(PtnReference *)
        );
        if (new_references == NULL) {
            ptn_abort_out_of_memory();
        }
        scan->references = new_references;
        scan->references_capacity = new_capacity;
    }
    scan->references[scan->references_len++] = reference;
    return 1;
}

static size_t ptn_runtime_run_static_property_value_destructors_impl(
    PtnValue value,
    size_t depth,
    PtnShutdownDestructorScan *scan
);

static PTN_UNUSED size_t ptn_runtime_run_symbol_value_destructors(PtnSymbolTable *symbols) {
    if (symbols == NULL || symbols->len == 0) {
        return 0;
    }

    size_t len = symbols->len;
    PtnValue *values = malloc(len * sizeof(PtnValue));
    if (values == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        values[i] = ptn_value_clone(symbols->items[i].value);
    }

    size_t destructors_ran = 0;
    for (size_t i = len; i > 0; i--) {
        size_t index = i - 1;
        destructors_ran += ptn_runtime_run_static_property_value_destructors(values[index], 0);
        ptn_value_destroy(&values[index]);
    }
    free(values);
    return destructors_ran;
}

static size_t ptn_runtime_run_static_property_value_destructors_impl(
    PtnValue value,
    size_t depth,
    PtnShutdownDestructorScan *scan
) {
    if (depth > 1024) {
        return 0;
    }
    if (value.type == PTN_REFERENCE) {
        if (value.as.reference == NULL ||
            !ptn_shutdown_destructor_scan_note_reference(scan, value.as.reference)) {
            return 0;
        }
        return ptn_runtime_run_static_property_value_destructors_impl(
            value.as.reference->value,
            depth + 1,
            scan
        );
    }
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT && value.as.object != NULL) {
        int destructor_was_called = value.as.object->destructor_called;
        ptn_object_retain(value.as.object);
        ptn_object_run_destructor_ex(value.as.object, 1);
        int destructor_ran = !destructor_was_called && value.as.object->destructor_called;
        ptn_object_release(value.as.object);
        return destructor_ran ? 1 : 0;
    }
    if (value.type != PTN_ARRAY || value.as.array == NULL) {
        return 0;
    }
    PtnArray *array = value.as.array;
    if (!ptn_shutdown_destructor_scan_note_array(scan, array)) {
        return 0;
    }
    ptn_array_retain(array);
    size_t destructors_ran = 0;
    for (size_t i = 0; i < array->len; i++) {
        destructors_ran += ptn_runtime_run_static_property_value_destructors_impl(
            array->entries[i].value,
            depth + 1,
            scan
        );
    }
    PtnValue retained_array = ptn_array(array);
    ptn_value_destroy(&retained_array);
    return destructors_ran;
}

static PTN_UNUSED size_t ptn_runtime_run_static_property_value_destructors(PtnValue value, size_t depth) {
    PtnShutdownDestructorScan scan = {0};
    size_t destructors_ran = ptn_runtime_run_static_property_value_destructors_impl(
        value,
        depth,
        &scan
    );
    ptn_shutdown_destructor_scan_free(&scan);
    return destructors_ran;
}

static PTN_UNUSED size_t ptn_runtime_run_static_property_destructors_once(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return 0;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    PtnSymbolTable *static_properties = root->static_properties == NULL
        ? &root->owned_static_properties
        : root->static_properties;
    size_t len = static_properties->len;
    size_t destructors_ran = 0;
    for (size_t i = 0; i < len; i++) {
        destructors_ran += ptn_runtime_run_static_property_value_destructors(
            static_properties->items[i].value,
            0
        );
    }
    return destructors_ran;
}

static PTN_UNUSED void ptn_runtime_run_static_property_destructors(PtnRuntime *runtime) {
    for (size_t guard = 0; guard < 1024; guard++) {
        if (ptn_runtime_run_static_property_destructors_once(runtime) == 0) {
            return;
        }
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
    object->debug_hidden_refcount = 0;
    object->object_id = ptn_runtime_alloc_object_id(root);
    object->gc_mark_epoch = 0;
    object->class_name = ptn_duplicate_string(class_name);
    object->enum_case_name = NULL;
    object->properties = properties.as.array;
    object->property_metadata = NULL;
    object->property_metadata_len = 0;
    object->property_metadata_capacity = 0;
    object->native_data = NULL;
    object->native_data_free = NULL;
    object->lifecycle_runtime = root;
    object->live_index = 0;
    object->destructor_enabled = 1;
    object->destructor_called = 0;
    object->lazy_uninitialized = 0;
    object->lazy_is_proxy = 0;
    object->lazy_options = 0;
    object->lazy_initializing = 0;
    object->lazy_initializer_refcount_guards = 0;
    object->readonly_clone_initializing = 0;
    object->defer_object_id_release_once = 0;
    object->var_dump_property_count_initialized = 0;
    object->last_var_dump_property_count = 0;
    object->active_property_value_unsets = 0;
    object->lazy_initializer = ptn_null();
    object->lazy_proxy_instance = ptn_null();
    ptn_runtime_register_object(root, object);
    ptn_runtime_register_array(root, object->properties);
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
    object->destructor_called = 0;
    object->lazy_initializer_refcount_guards = 0;
    object->lazy_initializer = ptn_value_clone_deref(initializer);
    object->lazy_proxy_instance = ptn_null();
    if (object->property_metadata_len == 0) {
        object->lazy_uninitialized = 0;
        object->lazy_is_proxy = 0;
        object->lazy_options = 0;
        ptn_value_destroy(&object->lazy_initializer);
        object->lazy_initializer = ptn_null();
    }
}

static int ptn_reference_property_type_matches_metadata(
    const char *declaring_class,
    const char *property_name,
    const PtnObjectPropertyMetadata *metadata
) {
    return declaring_class != NULL &&
        property_name != NULL &&
        metadata != NULL &&
        strcmp(declaring_class, metadata->declaring_class) == 0 &&
        strcmp(property_name, metadata->display_name) == 0;
}

static void ptn_reference_property_type_source_free(PtnReferencePropertyTypeSource *source) {
    if (source == NULL) {
        return;
    }
    free(source->class_name);
    free(source->text);
    free(source->declaring_class);
    free(source->property_name);
}

static void ptn_reference_property_type_source_free_list(
    PtnReferencePropertyTypeSource *sources,
    size_t len
) {
    if (sources == NULL) {
        return;
    }
    for (size_t i = 0; i < len; i++) {
        ptn_reference_property_type_source_free(&sources[i]);
    }
    free(sources);
}

static void ptn_reference_forget_property_type(
    PtnReference *reference,
    const PtnObjectPropertyMetadata *metadata
) {
    if (reference == NULL || metadata == NULL) {
        return;
    }
    if (ptn_reference_property_type_matches_metadata(
            reference->property_declaring_class,
            reference->property_name,
            metadata
        )) {
        free(reference->property_type_class_name);
        free(reference->property_type_text);
        free(reference->property_declaring_class);
        free(reference->property_name);
        if (reference->property_type_source_len == 0) {
            reference->property_type_kind = PTN_PROPERTY_TYPE_NONE;
            reference->property_type_class_name = NULL;
            reference->property_type_text = NULL;
            reference->property_type_allows_null = 0;
            reference->property_declaring_class = NULL;
            reference->property_name = NULL;
            return;
        }
        PtnReferencePropertyTypeSource promoted = reference->property_type_sources[0];
        reference->property_type_kind = promoted.kind;
        reference->property_type_class_name = promoted.class_name;
        reference->property_type_text = promoted.text;
        reference->property_type_allows_null = promoted.allows_null;
        reference->property_declaring_class = promoted.declaring_class;
        reference->property_name = promoted.property_name;
        if (reference->property_type_source_len > 1) {
            memmove(
                reference->property_type_sources,
                reference->property_type_sources + 1,
                (reference->property_type_source_len - 1) * sizeof(PtnReferencePropertyTypeSource)
            );
        }
        reference->property_type_source_len--;
        return;
    }
    for (size_t i = 0; i < reference->property_type_source_len; i++) {
        PtnReferencePropertyTypeSource *source = &reference->property_type_sources[i];
        if (!ptn_reference_property_type_matches_metadata(
                source->declaring_class,
                source->property_name,
                metadata
            )) {
            continue;
        }
        ptn_reference_property_type_source_free(source);
        if (i + 1 < reference->property_type_source_len) {
            memmove(
                source,
                source + 1,
                (reference->property_type_source_len - i - 1) * sizeof(PtnReferencePropertyTypeSource)
            );
        }
        reference->property_type_source_len--;
        return;
    }
}

static void ptn_object_property_metadata_free_fields(PtnObjectPropertyMetadata *metadata) {
    if (metadata == NULL) {
        return;
    }
    free(metadata->storage_name);
    free(metadata->display_name);
    free(metadata->declaring_class);
    free(metadata->hook_get_declaring_class);
    free(metadata->hook_set_declaring_class);
    free(metadata->last_type_name);
    free(metadata->type_class_name);
    free(metadata->type_text);
}

static void ptn_object_property_metadata_free_list(
    PtnObjectPropertyMetadata *metadata,
    size_t len
) {
    if (metadata == NULL) {
        return;
    }
    for (size_t i = 0; i < len; i++) {
        ptn_object_property_metadata_free_fields(&metadata[i]);
    }
    free(metadata);
}

static PtnObjectPropertyMetadata ptn_object_property_metadata_clone_entry(
    const PtnObjectPropertyMetadata *source
) {
    PtnObjectPropertyMetadata clone = *source;
    clone.storage_name = source->storage_name == NULL
        ? NULL
        : ptn_duplicate_string(source->storage_name);
    clone.display_name = source->display_name == NULL
        ? NULL
        : ptn_duplicate_string(source->display_name);
    clone.declaring_class = source->declaring_class == NULL
        ? NULL
        : ptn_duplicate_string(source->declaring_class);
    clone.hook_get_declaring_class = source->hook_get_declaring_class == NULL
        ? NULL
        : ptn_duplicate_string(source->hook_get_declaring_class);
    clone.hook_set_declaring_class = source->hook_set_declaring_class == NULL
        ? NULL
        : ptn_duplicate_string(source->hook_set_declaring_class);
    clone.last_type_name = source->last_type_name == NULL
        ? NULL
        : ptn_duplicate_string(source->last_type_name);
    clone.type_class_name = source->type_class_name == NULL
        ? NULL
        : ptn_duplicate_string(source->type_class_name);
    clone.type_text = source->type_text == NULL
        ? NULL
        : ptn_duplicate_string(source->type_text);
    return clone;
}

static PtnObjectPropertyMetadata *ptn_object_property_metadata_clone_list(
    const PtnObjectPropertyMetadata *source,
    size_t len
) {
    if (source == NULL || len == 0) {
        return NULL;
    }
    PtnObjectPropertyMetadata *clone = calloc(len, sizeof(PtnObjectPropertyMetadata));
    if (clone == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        clone[i] = ptn_object_property_metadata_clone_entry(&source[i]);
    }
    return clone;
}

static PtnReferencePropertyTypeSource ptn_reference_property_type_source_clone(
    const PtnReferencePropertyTypeSource *source
) {
    PtnReferencePropertyTypeSource clone = *source;
    clone.class_name = source->class_name == NULL
        ? NULL
        : ptn_duplicate_string(source->class_name);
    clone.text = source->text == NULL
        ? NULL
        : ptn_duplicate_string(source->text);
    clone.declaring_class = source->declaring_class == NULL
        ? NULL
        : ptn_duplicate_string(source->declaring_class);
    clone.property_name = source->property_name == NULL
        ? NULL
        : ptn_duplicate_string(source->property_name);
    return clone;
}

static PtnReferencePropertyTypeSource *ptn_reference_property_type_source_clone_list(
    const PtnReferencePropertyTypeSource *sources,
    size_t len
) {
    if (sources == NULL || len == 0) {
        return NULL;
    }
    PtnReferencePropertyTypeSource *clone = calloc(
        len,
        sizeof(PtnReferencePropertyTypeSource)
    );
    if (clone == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        clone[i] = ptn_reference_property_type_source_clone(&sources[i]);
    }
    return clone;
}

static PtnValue ptn_lazy_object_snapshot_value_clone(PtnValue value);

static PtnReference *ptn_lazy_object_snapshot_reference_clone(PtnReference *source) {
    if (source == NULL) {
        return ptn_reference_new_owned(ptn_null());
    }
    PtnReference *clone =
        ptn_reference_new_owned(ptn_lazy_object_snapshot_value_clone(source->value));
    clone->property_type_kind = source->property_type_kind;
    clone->property_type_class_name = source->property_type_class_name == NULL
        ? NULL
        : ptn_duplicate_string(source->property_type_class_name);
    clone->property_type_text = source->property_type_text == NULL
        ? NULL
        : ptn_duplicate_string(source->property_type_text);
    clone->property_type_allows_null = source->property_type_allows_null;
    clone->property_declaring_class = source->property_declaring_class == NULL
        ? NULL
        : ptn_duplicate_string(source->property_declaring_class);
    clone->property_name = source->property_name == NULL
        ? NULL
        : ptn_duplicate_string(source->property_name);
    clone->property_type_sources = ptn_reference_property_type_source_clone_list(
        source->property_type_sources,
        source->property_type_source_len
    );
    clone->property_type_source_len = source->property_type_source_len;
    clone->property_type_source_cap = source->property_type_source_len;
    return clone;
}

static PtnArray *ptn_lazy_object_snapshot_array_clone(PtnArray *source) {
    PtnArray *clone = ptn_array_clone(source);
    if (clone == NULL || source == NULL) {
        return clone;
    }
    for (size_t i = 0; i < clone->len && i < source->len; i++) {
        PtnValue replacement = ptn_lazy_object_snapshot_value_clone(source->entries[i].value);
        ptn_value_destroy(&clone->entries[i].value);
        clone->entries[i].value = replacement;
    }
    return clone;
}

static PtnValue ptn_lazy_object_snapshot_value_clone(PtnValue value) {
    value.by_ref_return_fallback = 0;
    switch (value.type) {
        case PTN_REFERENCE:
            return ptn_reference_value(ptn_lazy_object_snapshot_reference_clone(value.as.reference));
        case PTN_ARRAY:
            return ptn_array(ptn_lazy_object_snapshot_array_clone(value.as.array));
        case PTN_STRING:
            return ptn_value_clone(value);
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

typedef struct {
    PtnReference *reference;
    PtnValue value;
    PtnPropertyTypeKind property_type_kind;
    char *property_type_class_name;
    char *property_type_text;
    int property_type_allows_null;
    char *property_declaring_class;
    char *property_name;
    PtnReferencePropertyTypeSource *property_type_sources;
    size_t property_type_source_len;
} PtnLazyObjectReferenceSnapshot;

static PtnLazyObjectReferenceSnapshot ptn_lazy_object_reference_snapshot_capture(
    PtnReference *reference
) {
    PtnLazyObjectReferenceSnapshot snapshot;
    snapshot.reference = reference;
    snapshot.value = reference == NULL
        ? ptn_null()
        : ptn_lazy_object_snapshot_value_clone(reference->value);
    snapshot.property_type_kind = reference == NULL
        ? PTN_PROPERTY_TYPE_NONE
        : reference->property_type_kind;
    snapshot.property_type_class_name =
        reference == NULL || reference->property_type_class_name == NULL
            ? NULL
            : ptn_duplicate_string(reference->property_type_class_name);
    snapshot.property_type_text =
        reference == NULL || reference->property_type_text == NULL
            ? NULL
            : ptn_duplicate_string(reference->property_type_text);
    snapshot.property_type_allows_null = reference == NULL
        ? 0
        : reference->property_type_allows_null;
    snapshot.property_declaring_class =
        reference == NULL || reference->property_declaring_class == NULL
            ? NULL
            : ptn_duplicate_string(reference->property_declaring_class);
    snapshot.property_name = reference == NULL || reference->property_name == NULL
        ? NULL
        : ptn_duplicate_string(reference->property_name);
    snapshot.property_type_sources = reference == NULL
        ? NULL
        : ptn_reference_property_type_source_clone_list(
            reference->property_type_sources,
            reference->property_type_source_len
        );
    snapshot.property_type_source_len = reference == NULL
        ? 0
        : reference->property_type_source_len;
    if (reference != NULL) {
        if (reference->refcount == SIZE_MAX) {
            ptn_abort_out_of_memory();
        }
        reference->refcount++;
    }
    return snapshot;
}

static void ptn_lazy_object_reference_snapshot_drop(
    PtnLazyObjectReferenceSnapshot *snapshot
) {
    if (snapshot == NULL) {
        return;
    }
    ptn_value_destroy(&snapshot->value);
    free(snapshot->property_type_class_name);
    free(snapshot->property_type_text);
    free(snapshot->property_declaring_class);
    free(snapshot->property_name);
    ptn_reference_property_type_source_free_list(
        snapshot->property_type_sources,
        snapshot->property_type_source_len
    );
    if (snapshot->reference != NULL) {
        ptn_reference_release(snapshot->reference);
    }
    snapshot->reference = NULL;
    snapshot->value = ptn_null();
    snapshot->property_type_kind = PTN_PROPERTY_TYPE_NONE;
    snapshot->property_type_class_name = NULL;
    snapshot->property_type_text = NULL;
    snapshot->property_type_allows_null = 0;
    snapshot->property_declaring_class = NULL;
    snapshot->property_name = NULL;
    snapshot->property_type_sources = NULL;
    snapshot->property_type_source_len = 0;
}

static void ptn_lazy_object_reference_snapshot_restore(
    PtnLazyObjectReferenceSnapshot *snapshot
) {
    if (snapshot == NULL || snapshot->reference == NULL) {
        return;
    }
    PtnReference *reference = snapshot->reference;
    ptn_value_destroy(&reference->value);
    free(reference->property_type_class_name);
    free(reference->property_type_text);
    free(reference->property_declaring_class);
    free(reference->property_name);
    ptn_reference_property_type_source_free_list(
        reference->property_type_sources,
        reference->property_type_source_len
    );
    reference->value = snapshot->value;
    reference->property_type_kind = snapshot->property_type_kind;
    reference->property_type_class_name = snapshot->property_type_class_name;
    reference->property_type_text = snapshot->property_type_text;
    reference->property_type_allows_null = snapshot->property_type_allows_null;
    reference->property_declaring_class = snapshot->property_declaring_class;
    reference->property_name = snapshot->property_name;
    reference->property_type_sources = snapshot->property_type_sources;
    reference->property_type_source_len = snapshot->property_type_source_len;
    reference->property_type_source_cap = snapshot->property_type_source_len;
    snapshot->value = ptn_null();
    snapshot->property_type_kind = PTN_PROPERTY_TYPE_NONE;
    snapshot->property_type_class_name = NULL;
    snapshot->property_type_text = NULL;
    snapshot->property_type_allows_null = 0;
    snapshot->property_declaring_class = NULL;
    snapshot->property_name = NULL;
    snapshot->property_type_sources = NULL;
    snapshot->property_type_source_len = 0;
    snapshot->reference = NULL;
    ptn_reference_release(reference);
}

static int ptn_lazy_object_reset_should_remove_property(
    const PtnObjectPropertyMetadata *metadata,
    const char *class_name
) {
    if (metadata == NULL) {
        return 1;
    }
    if (class_name == NULL) {
        return 0;
    }
    if (ptn_ascii_case_equal(metadata->declaring_class, class_name)) {
        return 1;
    }
    return metadata->read_visibility != PTN_PROPERTY_PRIVATE &&
        ptn_declared_class_property_exists(class_name, metadata->display_name);
}

static int ptn_lazy_object_reset_should_preserve_readonly_property(
    PtnObject *object,
    const PtnObjectPropertyMetadata *metadata,
    const char *class_name
) {
    if (object == NULL ||
        object->properties == NULL ||
        metadata == NULL ||
        !metadata->is_readonly ||
        class_name == NULL ||
        ptn_ascii_case_equal(metadata->declaring_class, class_name)) {
        return 0;
    }
    PtnArrayKey key = ptn_array_string_key(metadata->storage_name);
    size_t index = ptn_array_find_key(object->properties, key);
    ptn_array_key_free(key);
    return index < object->properties->len;
}

static void ptn_lazy_object_mark_property_uninitialized(
    const PtnObjectPropertyMetadata *metadata
) {
    if (metadata == NULL) {
        return;
    }
    PtnObjectPropertyMetadata *mutable_metadata = (PtnObjectPropertyMetadata *)metadata;
    mutable_metadata->is_unset = metadata->type_kind == PTN_PROPERTY_TYPE_NONE ? 0 : 1;
    mutable_metadata->lazy_skip = 0;
    mutable_metadata->readonly_clone_reinitialized = 0;
    free(mutable_metadata->last_type_name);
    mutable_metadata->last_type_name = NULL;
}

static void ptn_object_forget_property_reference_sources(PtnObject *object) {
    if (object == NULL || object->properties == NULL) {
        return;
    }
    for (size_t i = 0; i < object->properties->len; i++) {
        PtnArrayEntry *entry = &object->properties->entries[i];
        if (entry->key.type != PTN_ARRAY_KEY_STRING ||
            entry->value.type != PTN_REFERENCE) {
            continue;
        }
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(object, entry->key.as.string);
        ptn_reference_forget_property_type(entry->value.as.reference, metadata);
    }
}

static void ptn_lazy_object_reset_property_entry_to_default(
    PtnObject *object,
    PtnArrayEntry *entry,
    const PtnObjectPropertyMetadata *metadata
) {
    if (object == NULL || entry == NULL || metadata == NULL) {
        return;
    }
    if (entry->value.type == PTN_REFERENCE) {
        ptn_reference_forget_property_type(entry->value.as.reference, metadata);
    }
    PtnArrayKey key = ptn_array_string_key_len(entry->key.as.string, entry->key.string_len);
    ptn_array_set_entry(object->properties, key, ptn_null());
}

static PTN_UNUSED void ptn_lazy_object_reset_property_storage(
    PtnObject *object,
    const char *class_name
) {
    if (object == NULL || object->properties == NULL) {
        return;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &object->property_metadata[i];
        if (ptn_lazy_object_reset_should_preserve_readonly_property(object, metadata, class_name)) {
            metadata->lazy_skip = 1;
            metadata->readonly_clone_reinitialized = 0;
        } else if (ptn_lazy_object_reset_should_remove_property(metadata, class_name)) {
            ptn_lazy_object_mark_property_uninitialized(metadata);
        } else {
            metadata->lazy_skip = 1;
        }
    }
    for (size_t i = 0; i < object->properties->len;) {
        PtnArrayEntry *entry = &object->properties->entries[i];
        if (entry->key.type != PTN_ARRAY_KEY_STRING) {
            i++;
            continue;
        }
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(object, entry->key.as.string);
        if (!ptn_lazy_object_reset_should_remove_property(metadata, class_name)) {
            i++;
            continue;
        }
        if (ptn_lazy_object_reset_should_preserve_readonly_property(object, metadata, class_name)) {
            i++;
            continue;
        }
        if (metadata != NULL &&
            metadata->type_kind == PTN_PROPERTY_TYPE_NONE &&
            !metadata->is_readonly) {
            ptn_lazy_object_reset_property_entry_to_default(object, entry, metadata);
            i++;
            continue;
        }
        if (entry->value.type == PTN_REFERENCE) {
            ptn_reference_forget_property_type(entry->value.as.reference, metadata);
        }
        PtnArrayKey key = ptn_array_string_key_len(entry->key.as.string, entry->key.string_len);
        ptn_array_unset_entry(object->properties, key);
    }
}

static PTN_UNUSED PtnValue ptn_lazy_object_detach_initialized_proxy_for_reset(
    PtnObject *object
) {
    if (object == NULL || !object->lazy_is_proxy || object->lazy_uninitialized) {
        return ptn_null();
    }
    PtnValue real = ptn_value_clone_deref(object->lazy_proxy_instance);
    ptn_value_destroy(&object->lazy_proxy_instance);
    object->lazy_proxy_instance = ptn_null();
    object->lazy_is_proxy = 0;
    object->lazy_options = 0;
    object->lazy_initializing = 0;
    return real;
}

static PTN_UNUSED void ptn_lazy_object_copy_properties_from_instance(
    PtnObject *target,
    PtnObject *source
) {
    if (target == NULL || source == NULL || source->properties == NULL) {
        return;
    }
    PtnArray *copied = ptn_array_clone(source->properties);
    ptn_object_forget_property_reference_sources(target);
    ptn_array_free(target->properties);
    target->properties = copied;
}

static PtnValue ptn_lazy_object_proxy_sync_property_value_clone(PtnValue value) {
    if (value.type == PTN_REFERENCE &&
        value.as.reference != NULL &&
        value.as.reference->refcount <= 2 &&
        value.as.reference->property_type_kind == PTN_PROPERTY_TYPE_NONE &&
        value.as.reference->property_type_source_len == 0 &&
        value.as.reference->property_declaring_class == NULL &&
        value.as.reference->property_name == NULL) {
        return ptn_value_clone_deref(value);
    }
    return ptn_value_clone(value);
}

static PtnArray *ptn_lazy_object_proxy_sync_properties_clone(PtnArray *source) {
    PtnArray *clone = ptn_array_clone(source);
    if (clone == NULL || source == NULL) {
        return clone;
    }
    for (size_t i = 0; i < clone->len && i < source->len; i++) {
        PtnValue replacement =
            ptn_lazy_object_proxy_sync_property_value_clone(source->entries[i].value);
        ptn_value_destroy(&clone->entries[i].value);
        clone->entries[i].value = replacement;
    }
    return clone;
}

static void ptn_lazy_object_sync_properties_to_proxy_instance(
    PtnObject *target,
    PtnObject *source
) {
    if (target == NULL || source == NULL || source->properties == NULL) {
        return;
    }
    ptn_object_forget_property_reference_sources(target);
    PtnArray *old_properties = target->properties;
    target->properties = NULL;
    ptn_array_free(old_properties);
    target->properties = ptn_lazy_object_proxy_sync_properties_clone(source->properties);
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
    ptn_lazy_object_sync_properties_to_proxy_instance(real.as.object, proxy);
    ptn_lazy_object_sync_proxy_instance_properties_depth(real.as.object, depth + 1);
}

static PTN_UNUSED void ptn_lazy_object_sync_proxy_instance_properties(PtnObject *proxy) {
    ptn_lazy_object_sync_proxy_instance_properties_depth(proxy, 0);
}

static PTN_UNUSED void ptn_lazy_object_sync_forwarded_proxy_property_reference(
    PtnValue original_receiver,
    PtnValue effective_receiver,
    const char *storage_key,
    PtnValue reference
) {
    original_receiver = ptn_value_deref(original_receiver);
    effective_receiver = ptn_value_deref(effective_receiver);
    if (storage_key == NULL ||
        reference.type != PTN_REFERENCE ||
        original_receiver.type != PTN_OBJECT ||
        effective_receiver.type != PTN_OBJECT ||
        original_receiver.as.object == NULL ||
        effective_receiver.as.object == NULL ||
        original_receiver.as.object == effective_receiver.as.object ||
        !original_receiver.as.object->lazy_is_proxy ||
        original_receiver.as.object->lazy_uninitialized ||
        original_receiver.as.object->properties == NULL) {
        return;
    }
    PtnValue real = ptn_value_deref(original_receiver.as.object->lazy_proxy_instance);
    if (real.type != PTN_OBJECT || real.as.object != effective_receiver.as.object) {
        return;
    }
    ptn_array_set_entry(
        original_receiver.as.object->properties,
        ptn_array_string_key(storage_key),
        ptn_value_clone(reference)
    );
}

static PTN_UNUSED PtnValue ptn_lazy_object_effective_initialized_proxy_receiver(PtnValue receiver) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT || receiver.as.object == NULL) {
        return receiver;
    }
    PtnObject *object = receiver.as.object;
    for (size_t depth = 0; depth < 64; depth++) {
        if (!object->lazy_is_proxy || object->lazy_uninitialized) {
            return ptn_value_borrow(ptn_object(object));
        }
        PtnValue real = ptn_value_deref(object->lazy_proxy_instance);
        if (real.type != PTN_OBJECT ||
            real.as.object == NULL ||
            real.as.object == object) {
            return ptn_value_borrow(ptn_object(object));
        }
        object = real.as.object;
    }
    return ptn_value_borrow(ptn_object(object));
}

static PTN_UNUSED PtnValue ptn_lazy_object_effective_initialized_proxy_receiver_for_access(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT || receiver.as.object == NULL) {
        return receiver;
    }
    PtnObject *object = receiver.as.object;
    for (size_t depth = 0; depth < 64; depth++) {
        if (!object->lazy_is_proxy || object->lazy_uninitialized) {
            return ptn_value_borrow(ptn_object(object));
        }
        PtnValue real = ptn_value_deref(object->lazy_proxy_instance);
        if (real.type != PTN_OBJECT ||
            real.as.object == NULL ||
            real.as.object == object) {
            return ptn_value_borrow(ptn_object(object));
        }
        if (real.as.object->lazy_uninitialized && !real.as.object->lazy_initializing) {
            if (!ptn_lazy_object_initialize(runtime, real, line)) {
                return ptn_value_borrow(ptn_object(real.as.object));
            }
            real = ptn_value_deref(real);
            if (real.type != PTN_OBJECT || real.as.object == NULL) {
                return ptn_value_borrow(ptn_object(object));
            }
        }
        object = real.as.object;
    }
    return ptn_value_borrow(ptn_object(object));
}

typedef struct {
    PtnArray *properties;
    PtnObjectPropertyMetadata *metadata;
    size_t metadata_len;
    size_t metadata_capacity;
    PtnLazyObjectReferenceSnapshot *references;
    size_t reference_len;
    size_t reference_capacity;
    PtnRuntime *root_runtime;
    int properties_rooted;
} PtnLazyObjectInitializerSnapshot;

static int ptn_lazy_object_initializer_snapshot_has_reference(
    const PtnLazyObjectInitializerSnapshot *snapshot,
    PtnReference *reference
) {
    if (snapshot == NULL || reference == NULL) {
        return 1;
    }
    for (size_t i = 0; i < snapshot->reference_len; i++) {
        if (snapshot->references[i].reference == reference) {
            return 1;
        }
    }
    return 0;
}

static void ptn_lazy_object_initializer_snapshot_add_reference(
    PtnLazyObjectInitializerSnapshot *snapshot,
    PtnReference *reference
) {
    if (snapshot == NULL ||
        reference == NULL ||
        ptn_lazy_object_initializer_snapshot_has_reference(snapshot, reference)) {
        return;
    }
    if (snapshot->reference_len == snapshot->reference_capacity) {
        size_t new_capacity = snapshot->reference_capacity == 0
            ? 4
            : snapshot->reference_capacity * 2;
        PtnLazyObjectReferenceSnapshot *new_references = realloc(
            snapshot->references,
            new_capacity * sizeof(PtnLazyObjectReferenceSnapshot)
        );
        if (new_references == NULL) {
            ptn_abort_out_of_memory();
        }
        snapshot->references = new_references;
        snapshot->reference_capacity = new_capacity;
    }
    snapshot->references[snapshot->reference_len++] =
        ptn_lazy_object_reference_snapshot_capture(reference);
}

static void ptn_lazy_object_initializer_snapshot_collect_value_references(
    PtnLazyObjectInitializerSnapshot *snapshot,
    PtnValue value,
    size_t depth
) {
    if (depth > 1024) {
        return;
    }
    if (value.type == PTN_REFERENCE) {
        ptn_lazy_object_initializer_snapshot_add_reference(snapshot, value.as.reference);
        return;
    }
    value = ptn_value_deref(value);
    if (value.type != PTN_ARRAY || value.as.array == NULL) {
        return;
    }
    for (size_t i = 0; i < value.as.array->len; i++) {
        ptn_lazy_object_initializer_snapshot_collect_value_references(
            snapshot,
            value.as.array->entries[i].value,
            depth + 1
        );
    }
}

static void ptn_lazy_object_initializer_snapshot_collect_references(
    PtnLazyObjectInitializerSnapshot *snapshot,
    PtnObject *object
) {
    if (snapshot == NULL || object == NULL || object->properties == NULL) {
        return;
    }
    for (size_t i = 0; i < object->properties->len; i++) {
        ptn_lazy_object_initializer_snapshot_collect_value_references(
            snapshot,
            object->properties->entries[i].value,
            0
        );
    }
}

static PtnLazyObjectInitializerSnapshot ptn_lazy_object_initializer_snapshot(PtnObject *object) {
    PtnLazyObjectInitializerSnapshot snapshot;
    snapshot.references = NULL;
    snapshot.reference_len = 0;
    snapshot.reference_capacity = 0;
    snapshot.root_runtime = object == NULL ? NULL : object->lifecycle_runtime;
    snapshot.properties_rooted = 0;
    ptn_lazy_object_initializer_snapshot_collect_references(&snapshot, object);
    snapshot.properties = object == NULL || object->properties == NULL
        ? NULL
        : ptn_array_clone(object->properties);
    if (snapshot.root_runtime != NULL && snapshot.properties != NULL) {
        ptn_runtime_push_temporary_root(
            snapshot.root_runtime,
            ptn_value_borrow(ptn_array(snapshot.properties))
        );
        snapshot.properties_rooted = 1;
    }
    snapshot.metadata_len = object == NULL ? 0 : object->property_metadata_len;
    snapshot.metadata_capacity = object == NULL ? 0 : object->property_metadata_len;
    snapshot.metadata = object == NULL
        ? NULL
        : ptn_object_property_metadata_clone_list(
            object->property_metadata,
            object->property_metadata_len
        );
    return snapshot;
}

static void ptn_lazy_object_initializer_snapshot_unroot(
    PtnLazyObjectInitializerSnapshot *snapshot
) {
    if (snapshot == NULL || !snapshot->properties_rooted) {
        return;
    }
    ptn_runtime_pop_temporary_root(snapshot->root_runtime);
    snapshot->root_runtime = NULL;
    snapshot->properties_rooted = 0;
}

static void ptn_lazy_object_initializer_snapshot_discard(
    PtnLazyObjectInitializerSnapshot *snapshot
) {
    if (snapshot == NULL) {
        return;
    }
    ptn_lazy_object_initializer_snapshot_unroot(snapshot);
    ptn_array_free(snapshot->properties);
    ptn_object_property_metadata_free_list(snapshot->metadata, snapshot->metadata_len);
    for (size_t i = 0; i < snapshot->reference_len; i++) {
        ptn_lazy_object_reference_snapshot_drop(&snapshot->references[i]);
    }
    free(snapshot->references);
    snapshot->properties = NULL;
    snapshot->metadata = NULL;
    snapshot->metadata_len = 0;
    snapshot->metadata_capacity = 0;
    snapshot->references = NULL;
    snapshot->reference_len = 0;
    snapshot->reference_capacity = 0;
    snapshot->root_runtime = NULL;
    snapshot->properties_rooted = 0;
}

static void ptn_lazy_object_initializer_snapshot_restore(
    PtnObject *object,
    PtnLazyObjectInitializerSnapshot *snapshot
) {
    if (object == NULL || snapshot == NULL) {
        return;
    }
    ptn_lazy_object_initializer_snapshot_unroot(snapshot);
    for (size_t i = 0; i < snapshot->reference_len; i++) {
        ptn_lazy_object_reference_snapshot_restore(&snapshot->references[i]);
    }
    free(snapshot->references);
    ptn_array_free(object->properties);
    ptn_object_property_metadata_free_list(object->property_metadata, object->property_metadata_len);
    object->properties = snapshot->properties;
    object->property_metadata = snapshot->metadata;
    object->property_metadata_len = snapshot->metadata_len;
    object->property_metadata_capacity = snapshot->metadata_capacity;
    snapshot->properties = NULL;
    snapshot->metadata = NULL;
    snapshot->metadata_len = 0;
    snapshot->metadata_capacity = 0;
    snapshot->references = NULL;
    snapshot->reference_len = 0;
    snapshot->reference_capacity = 0;
    snapshot->root_runtime = NULL;
    snapshot->properties_rooted = 0;
}

static PtnValue ptn_lazy_object_dynamic_properties_snapshot(PtnObject *object) {
    PtnValue dynamic_properties = ptn_array_from_literal_entries(0, NULL);
    if (object == NULL || object->properties == NULL) {
        return dynamic_properties;
    }
    for (size_t i = 0; i < object->properties->len; i++) {
        PtnArrayEntry *entry = &object->properties->entries[i];
        if (entry->key.type == PTN_ARRAY_KEY_STRING &&
            ptn_object_property_metadata(object, entry->key.as.string) != NULL) {
            continue;
        }
        ptn_array_set_entry(
            dynamic_properties.as.array,
            ptn_array_key_clone(entry->key),
            ptn_value_clone(entry->value)
        );
    }
    return dynamic_properties;
}

static void ptn_lazy_object_restore_dynamic_properties(
    PtnObject *object,
    PtnValue dynamic_properties
) {
    dynamic_properties = ptn_value_deref(dynamic_properties);
    if (object == NULL ||
        object->properties == NULL ||
        dynamic_properties.type != PTN_ARRAY ||
        dynamic_properties.as.array == NULL) {
        return;
    }
    for (size_t i = 0; i < dynamic_properties.as.array->len; i++) {
        PtnArrayEntry *entry = &dynamic_properties.as.array->entries[i];
        ptn_array_set_entry(
            object->properties,
            ptn_array_key_clone(entry->key),
            ptn_value_clone(entry->value)
        );
    }
}

static PTN_UNUSED int ptn_lazy_object_initialize_for_dynamic_property_compound(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT || value.as.object == NULL) {
        return 1;
    }
    PtnObject *object = value.as.object;
    if (!object->lazy_uninitialized || object->lazy_initializing || object->lazy_is_proxy) {
        return ptn_lazy_object_initialize(runtime, value, line);
    }
    PtnLazyObjectInitializerSnapshot snapshot =
        ptn_lazy_object_initializer_snapshot(object);
    if (!ptn_lazy_object_initialize(runtime, value, line)) {
        ptn_lazy_object_initializer_snapshot_discard(&snapshot);
        return 0;
    }
    PtnValue dynamic_properties =
        ptn_lazy_object_dynamic_properties_snapshot(object);
    ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
    ptn_lazy_object_restore_dynamic_properties(object, dynamic_properties);
    ptn_value_destroy(&dynamic_properties);
    return 1;
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

static void ptn_lazy_object_prepare_type_error(
    PtnRuntime *runtime,
    const char *message,
    size_t line
) {
    if (runtime == NULL || runtime->exceptions == NULL) {
        return;
    }
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        "TypeError",
        ptn_duplicate_string(message),
        strlen(message),
        0,
        previous,
        PTN_E_ERROR,
        runtime->source_path,
        line
    );
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
}

static void ptn_lazy_object_throw_released_during_initialization(
    PtnRuntime *runtime,
    size_t line
) {
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Lazy object was released during initialization",
        runtime == NULL ? NULL : runtime->source_path,
        line
    );
}

static void ptn_lazy_object_release_initializer_refcount_guards(PtnObject *object) {
    if (object == NULL || object->lazy_initializer_refcount_guards == 0) {
        return;
    }
    size_t guards = object->lazy_initializer_refcount_guards;
    object->lazy_initializer_refcount_guards = 0;
    for (size_t i = 0; i < guards; i++) {
        ptn_object_release(object);
    }
}

static void ptn_magic_property_update_lazy_proxy_frame_ids(
    PtnRuntime *runtime,
    size_t proxy_object_id,
    size_t real_object_id
) {
    if (runtime == NULL || proxy_object_id == 0 || real_object_id == 0) {
        return;
    }
    for (size_t i = 0; i < runtime->magic_property_frame_len; i++) {
        PtnMagicPropertyFrame *frame = &runtime->magic_property_frames[i];
        if (frame->object_id == proxy_object_id ||
            frame->effective_object_id == proxy_object_id) {
            frame->effective_object_id = real_object_id;
        }
    }
}

static void ptn_magic_property_note_lazy_proxy_initialized(
    PtnRuntime *runtime,
    size_t proxy_object_id,
    size_t real_object_id
) {
    ptn_magic_property_update_lazy_proxy_frame_ids(runtime, proxy_object_id, real_object_id);
    if (runtime != NULL &&
        runtime->lifecycle_root != NULL &&
        runtime->lifecycle_root != runtime) {
        ptn_magic_property_update_lazy_proxy_frame_ids(
            runtime->lifecycle_root,
            proxy_object_id,
            real_object_id
        );
    }
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
    PtnLazyObjectInitializerSnapshot snapshot =
        ptn_lazy_object_initializer_snapshot(object);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    size_t refcount_before_initializer = object->refcount;
    ptn_object_retain(object);
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
            ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
            object->lazy_initializing = 0;
            ptn_value_destroy(&initializer);
            ptn_value_destroy(&result);
            ptn_object_release(object);
            ptn_rethrow_exception(runtime);
            return 0;
        }
    }
    result = ptn_call_callable(runtime, initializer, 1, &arg, line, 0);
    if (initializer_frame_active) {
        ptn_try_frame_pop(runtime, &initializer_frame);
    }
    ptn_value_destroy(&initializer);
    size_t initializer_refcount_guards = object->lazy_initializer_refcount_guards;
    int object_released = object->refcount < refcount_before_initializer ||
        (object->refcount == refcount_before_initializer && initializer_refcount_guards == 0);
    ptn_lazy_object_release_initializer_refcount_guards(object);
    int object_destroyed_during_initializer = object->destructor_called;
    if (runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL) {
        ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
        object->lazy_initializing = 0;
        ptn_value_destroy(&result);
        ptn_object_release(object);
        return 0;
    }
    if (object_destroyed_during_initializer && !object_released) {
        object->lazy_initializing = 0;
        ptn_value_destroy(&result);
        ptn_lazy_object_initializer_snapshot_discard(&snapshot);
        ptn_object_release(object);
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
            ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            if (object_released) {
                ptn_lazy_object_prepare_type_error(runtime, message, line);
                ptn_object_release(object);
                ptn_lazy_object_throw_released_during_initialization(runtime, line);
                return 0;
            }
            ptn_object_release(object);
            ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
            return 0;
        }
        if (real.as.object == object) {
            ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            if (object_released) {
                ptn_lazy_object_prepare_type_error(
                    runtime,
                    "Lazy proxy factory must return a non-lazy object",
                    line
                );
                ptn_object_release(object);
                ptn_lazy_object_throw_released_during_initialization(runtime, line);
                return 0;
            }
            ptn_object_release(object);
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Lazy proxy factory must return a non-lazy object",
                runtime->source_path,
                line
            );
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
            ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            if (object_released) {
                ptn_lazy_object_prepare_type_error(runtime, message, line);
                ptn_object_release(object);
                ptn_lazy_object_throw_released_during_initialization(runtime, line);
                return 0;
            }
            ptn_object_release(object);
            ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
            return 0;
        }
        if (object_released) {
            ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
            ptn_value_destroy(&result);
            object->lazy_initializing = 0;
            ptn_object_release(object);
            ptn_lazy_object_throw_released_during_initialization(runtime, line);
            return 0;
        }
        ptn_value_destroy(&object->lazy_proxy_instance);
        object->lazy_proxy_instance = ptn_value_clone_deref(real);
        ptn_magic_property_note_lazy_proxy_initialized(
            runtime,
            object->object_id,
            real.as.object->object_id
        );
        ptn_lazy_object_copy_properties_from_instance(object, real.as.object);
    } else {
        PtnValue returned = ptn_value_deref(result);
        if (returned.type != PTN_NULL) {
            ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
            object->lazy_initializing = 0;
            ptn_value_destroy(&result);
            if (object_released) {
                ptn_lazy_object_prepare_type_error(
                    runtime,
                    "Lazy object initializer must return NULL or no value",
                    line
                );
                ptn_object_release(object);
                ptn_lazy_object_throw_released_during_initialization(runtime, line);
                return 0;
            }
            ptn_object_release(object);
            ptn_throw_exception_at(
                runtime,
                "TypeError",
                "Lazy object initializer must return NULL or no value",
                runtime->source_path,
                line
            );
            return 0;
        }
        if (object_released) {
            ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
            ptn_value_destroy(&result);
            ptn_object_run_destructor(object);
            object->lazy_initializing = 0;
            ptn_object_release(object);
            ptn_lazy_object_throw_released_during_initialization(runtime, line);
            return 0;
        }
    }
    ptn_value_destroy(&result);
    ptn_value_destroy(&object->lazy_initializer);
    object->lazy_initializer = ptn_null();
    object->lazy_uninitialized = 0;
    object->lazy_initializing = 0;
    ptn_lazy_object_initializer_snapshot_discard(&snapshot);
    ptn_object_release(object);
    return 1;
#else
    ptn_lazy_object_initializer_snapshot_restore(object, &snapshot);
    object->lazy_initializing = 0;
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Lazy object initializer dispatch is unavailable",
        runtime->source_path,
        line
    );
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
        0,
        0,
        0,
        0,
        0,
        NULL,
        NULL,
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
            0,
            0,
            0,
            0,
            0,
            NULL,
            NULL,
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
    int has_hooks,
    int is_virtual,
    int hook_has_get,
    int hook_get_returns_by_ref,
    int hook_has_set,
    const char *hook_get_declaring_class,
    const char *hook_set_declaring_class,
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
            object->property_metadata[i].has_hooks = has_hooks;
            object->property_metadata[i].is_virtual = is_virtual;
            object->property_metadata[i].hook_has_get = hook_has_get;
            object->property_metadata[i].hook_get_returns_by_ref = hook_get_returns_by_ref;
            object->property_metadata[i].hook_has_set = hook_has_set;
            free(object->property_metadata[i].hook_get_declaring_class);
            free(object->property_metadata[i].hook_set_declaring_class);
            object->property_metadata[i].hook_get_declaring_class =
                hook_get_declaring_class == NULL ? NULL : ptn_duplicate_string(hook_get_declaring_class);
            object->property_metadata[i].hook_set_declaring_class =
                hook_set_declaring_class == NULL ? NULL : ptn_duplicate_string(hook_set_declaring_class);
            object->property_metadata[i].is_unset = 0;
            object->property_metadata[i].lazy_skip = 0;
            object->property_metadata[i].readonly_clone_reinitialized = 0;
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
    metadata->has_hooks = has_hooks;
    metadata->is_virtual = is_virtual;
    metadata->hook_has_get = hook_has_get;
    metadata->hook_get_returns_by_ref = hook_get_returns_by_ref;
    metadata->hook_has_set = hook_has_set;
    metadata->hook_get_declaring_class =
        hook_get_declaring_class == NULL ? NULL : ptn_duplicate_string(hook_get_declaring_class);
    metadata->hook_set_declaring_class =
        hook_set_declaring_class == NULL ? NULL : ptn_duplicate_string(hook_set_declaring_class);
    metadata->is_unset = 0;
    metadata->lazy_skip = 0;
    metadata->readonly_clone_reinitialized = 0;
    metadata->last_type_name = NULL;
    metadata->type_kind = type_kind;
    metadata->type_class_name = type_class_name == NULL ? NULL : ptn_duplicate_string(type_class_name);
    metadata->type_text = type_text == NULL ? NULL : ptn_duplicate_string(type_text);
    metadata->type_allows_null = type_allows_null;
}

static PTN_UNUSED PtnArrayKey ptn_array_key_clone(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_array_int_key(key.as.integer);
    }
    return ptn_array_string_key_len(key.as.string, key.string_len);
}

static PtnArray *ptn_array_clone_with_mode(PtnArray *source, int unwrap_entry_references) {
    PtnArray *array = malloc(sizeof(PtnArray));
    if (array == NULL) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_note_array_alloc();
    ptn_cow_debug_note_array_clone();
    array->refcount = 1;
    array->destructing = 0;
    array->gc_mark_epoch = 0;
    array->lifecycle_runtime = NULL;
    array->live_index = 0;
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
        PtnValue value = unwrap_entry_references
            ? ptn_value_clone_deref(source->entries[i].value)
            : ptn_value_clone(source->entries[i].value);
        ptn_array_set_entry_with_by_ref_argument_eligibility(
            array,
            key,
            value,
            source->entries[i].by_ref_argument_eligible
        );
    }
    array->next_auto_key = source->next_auto_key;
    ptn_runtime_register_array(source == NULL ? NULL : source->lifecycle_runtime, array);
    for (size_t i = 0; i < array->len; i++) {
        ptn_gc_attach_value_runtime(array->lifecycle_runtime, array->entries[i].value, 0);
    }
    return array;
}

static PTN_UNUSED PtnArray *ptn_array_clone(PtnArray *source) {
    return ptn_array_clone_with_mode(source, 0);
}

static PTN_UNUSED PtnArray *ptn_array_clone_unwrap_references(PtnArray *source) {
    return ptn_array_clone_with_mode(source, 1);
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
        if (object->refcount > 1) {
            ptn_gc_drain_pending_destructor_array_cycles(object->lifecycle_runtime);
        }
        return;
    }
    object->refcount = 1;
    ptn_object_run_destructor(object);
    if (object->refcount > 1) {
        object->refcount--;
        return;
    }
    if (ptn_object_is_generator(object)) {
        ptn_generator_force_close(object->lifecycle_runtime, (PtnGenerator *)object->native_data);
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_internal_class_name_is_fiber(object->class_name)) {
        ptn_fiber_force_close(object->lifecycle_runtime, object);
    }
#endif
    object->refcount = 0;
    ptn_runtime_unregister_object(object->lifecycle_runtime, object);
    ptn_runtime_prune_weak_maps_for_released_object(object->lifecycle_runtime);
    if (object->native_data_free != NULL) {
        object->native_data_free(object->native_data);
    }
    ptn_object_forget_property_reference_sources(object);
    ptn_value_destroy(&object->lazy_initializer);
    ptn_value_destroy(&object->lazy_proxy_instance);
    free(object->class_name);
    free(object->enum_case_name);
    ptn_object_property_metadata_free_list(
        object->property_metadata,
        object->property_metadata_len
    );
    ptn_array_free(object->properties);
    if (object->defer_object_id_release_once) {
        ptn_runtime_release_object_id_after_next_allocation(
            object->lifecycle_runtime,
            object->object_id
        );
    } else {
        ptn_runtime_release_object_id(object->lifecycle_runtime, object->object_id);
    }
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

static PTN_UNUSED void ptn_object_debug_hide_ref(PtnObject *object) {
    if (object == NULL) {
        return;
    }
    if (object->debug_hidden_refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    object->debug_hidden_refcount++;
}

static PTN_UNUSED void ptn_object_debug_unhide_ref(PtnObject *object) {
    if (object == NULL || object->debug_hidden_refcount == 0) {
        return;
    }
    object->debug_hidden_refcount--;
}

static PTN_UNUSED void ptn_value_debug_hide_ref(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_array_debug_hide_ref(value.as.array);
    } else if (value.type == PTN_OBJECT) {
        ptn_object_debug_hide_ref(value.as.object);
    }
}

static PTN_UNUSED void ptn_value_debug_unhide_ref(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_array_debug_unhide_ref(value.as.array);
    } else if (value.type == PTN_OBJECT) {
        ptn_object_debug_unhide_ref(value.as.object);
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

static PTN_UNUSED size_t ptn_object_debug_visible_refcount(PtnObject *object) {
    if (object == NULL) {
        return 0;
    }
    if (object->debug_hidden_refcount >= object->refcount) {
        return 1;
    }
    return object->refcount - object->debug_hidden_refcount;
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
    if (ptn_array_debug_visible_refcount(array) <= 1) {
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
    value.by_ref_return_fallback = 0;
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
    value.by_ref_return_fallback = 0;
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

static PTN_UNUSED PtnValue ptn_debug_zval_argument(PtnValue *value) {
    if (value == NULL) {
        return ptn_null();
    }
    if (value->owned) {
        PtnValue moved = *value;
        *value = ptn_null();
        return moved;
    }
    return ptn_value_share(*value);
}

static PTN_UNUSED PtnValue ptn_value_clone(PtnValue value) {
    if (value.type == PTN_STRING && value.as.string.payload == NULL) {
        PtnValue clone = ptn_owned_string_len(
            ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len),
            value.as.string.len
        );
        clone.as.string.payload->interned = 1;
        return clone;
    }
    return ptn_value_share(value);
}

static PTN_UNUSED PtnValue ptn_value_snapshot_for_array_path_write(PtnValue value) {
    if (value.type == PTN_REFERENCE) {
        PtnValue resolved = ptn_value_deref(value);
        if (resolved.type == PTN_ARRAY) {
            return ptn_value_clone(value);
        }
    }
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY || value.type == PTN_OBJECT) {
        return ptn_value_clone(value);
    }
    return value;
}

static PTN_UNUSED PtnValue ptn_value_snapshot_for_self_array_path_write(PtnValue value) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_ARRAY) {
        return ptn_value_clone(resolved);
    }
    return ptn_value_snapshot_for_array_path_write(value);
}

static PTN_UNUSED void ptn_value_separate_temporary_array_root_for_write(PtnValue *value) {
    if (value == NULL || value->type == PTN_REFERENCE) {
        return;
    }
    PtnValue resolved = ptn_value_deref(*value);
    if (resolved.type != PTN_ARRAY) {
        return;
    }
    PtnValue separated = ptn_array(ptn_array_clone_unwrap_references(resolved.as.array));
    ptn_value_destroy(value);
    *value = separated;
}

static PTN_UNUSED PtnArray *ptn_value_detach_array(PtnValue *value) {
    return ptn_array_detach_value(value);
}
