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
    free(exception->uncaught_text);
    exception->uncaught_text_len = 0;
    ptn_value_destroy(&exception->trace);
    ptn_value_destroy(&exception->previous);
    ptn_value_destroy(&exception->dynamic_properties);
    ptn_value_destroy(&exception->errors);
    free(exception->soap_fault_code);
    ptn_value_destroy(&exception->soap_fault_headerfault);
    free(exception);
}

static PTN_UNUSED PtnReference *ptn_reference_new_owned(PtnValue value) {
    PtnReference *reference = malloc(sizeof(PtnReference));
    if (reference == NULL) {
        ptn_abort_out_of_memory();
    }
    reference->refcount = 1;
    reference->value = value;
    reference->lifecycle_runtime = NULL;
    reference->live_index = 0;
    reference->gc_mark_epoch = 0;
    reference->gc_collecting = 0;
    reference->property_type_kind = PTN_PROPERTY_TYPE_NONE;
    reference->property_type_class_name = NULL;
    reference->property_type_text = NULL;
    reference->property_type_allows_null = 0;
    reference->property_declaring_class = NULL;
    reference->property_name = NULL;
    reference->property_type_sources = NULL;
    reference->property_type_source_len = 0;
    reference->property_type_source_cap = 0;
    return reference;
}

static PTN_UNUSED void ptn_reference_retain(PtnReference *reference) {
    if (reference == NULL) {
        return;
    }
    if (reference->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    reference->refcount++;
}

static PTN_UNUSED void ptn_gc_begin_replaced_reference_cycle_suppression(
    PtnReference *reference
);
static PTN_UNUSED void ptn_gc_end_replaced_reference_cycle_suppression(
    PtnReference *reference
);

static PTN_UNUSED int ptn_reference_assign_result_with_context_at(
    PtnRuntime *runtime,
    PtnReference *reference,
    PtnValue value,
    int reference_context,
    size_t line,
    PtnValue *result_out
) {
    PtnValue stored_value = ptn_null();
    if (!ptn_property_reference_coerce_assignment(
        runtime,
        reference,
        value,
        reference_context,
        line,
        &stored_value
    )) {
        return 0;
    }
    ptn_gc_attach_value_runtime(
        runtime != NULL ? runtime : reference->lifecycle_runtime,
        stored_value,
        0
    );
    PtnValue result = ptn_value_clone(stored_value);
    ptn_array_note_value_replacement(reference->value, stored_value);
    ptn_gc_begin_replaced_reference_cycle_suppression(reference);
    ptn_value_destroy_with_runtime_scope(runtime, &reference->value);
    ptn_gc_end_replaced_reference_cycle_suppression(reference);
    reference->value = stored_value;
    *result_out = result;
    return 1;
}

static PTN_UNUSED int ptn_reference_assign_result_with_context(
    PtnRuntime *runtime,
    PtnReference *reference,
    PtnValue value,
    int reference_context,
    PtnValue *result_out
) {
    return ptn_reference_assign_result_with_context_at(
        runtime,
        reference,
        value,
        reference_context,
        0,
        result_out
    );
}

static PTN_UNUSED int ptn_reference_assign_result(PtnRuntime *runtime, PtnReference *reference, PtnValue value, PtnValue *result_out) {
    return ptn_reference_assign_result_with_context(runtime, reference, value, 1, result_out);
}

static PTN_UNUSED int ptn_reference_assign(PtnRuntime *runtime, PtnReference *reference, PtnValue value) {
    PtnValue result = ptn_null();
    if (!ptn_reference_assign_result(runtime, reference, value, &result)) {
        return 0;
    }
    ptn_value_destroy(&result);
    return 1;
}

static PTN_UNUSED int ptn_reference_assign_publish_first_result_with_context(
    PtnRuntime *runtime,
    PtnReference *reference,
    PtnValue value,
    int reference_context,
    PtnValue *result_out
) {
    PtnValue stored_value = ptn_null();
    if (!ptn_property_reference_coerce_assignment(
        runtime,
        reference,
        value,
        reference_context,
        0,
        &stored_value
    )) {
        return 0;
    }
    ptn_gc_attach_value_runtime(
        runtime != NULL ? runtime : reference->lifecycle_runtime,
        stored_value,
        0
    );
    PtnValue result = ptn_value_clone(stored_value);
    PtnValue old_value = reference->value;
    ptn_array_note_value_replacement(old_value, stored_value);
    reference->value = stored_value;
    ptn_value_destroy_with_runtime_scope(runtime, &old_value);
    if (result_out != NULL) {
        *result_out = result;
    } else {
        ptn_value_destroy(&result);
    }
    return 1;
}

static PTN_UNUSED int ptn_reference_assign_publish_first_result(
    PtnRuntime *runtime,
    PtnReference *reference,
    PtnValue value,
    PtnValue *result_out
) {
    return ptn_reference_assign_publish_first_result_with_context(
        runtime,
        reference,
        value,
        1,
        result_out
    );
}

static PTN_UNUSED int ptn_reference_assign_publish_first(PtnRuntime *runtime, PtnReference *reference, PtnValue value) {
    return ptn_reference_assign_publish_first_result(runtime, reference, value, NULL);
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

static PTN_UNUSED int ptn_reference_is_protected_by_live_array_iterator(
    PtnRuntime *runtime,
    PtnReference *reference
) {
    if (reference == NULL) {
        return 0;
    }
    if (
        reference->value.type == PTN_ARRAY &&
        reference->value.as.array != NULL &&
        reference->value.as.array->iterator_refcount != 0
    ) {
        return 1;
    }

    PtnRuntime *owner = runtime != NULL ? runtime : reference->lifecycle_runtime;
    if (
        owner == NULL &&
        reference->value.type == PTN_ARRAY &&
        reference->value.as.array != NULL
    ) {
        owner = reference->value.as.array->lifecycle_runtime;
    }
    PtnRuntime *root = ptn_runtime_root(owner);
    if (root == NULL) {
        return 0;
    }
    for (size_t i = 0; i < root->live_arrays_len; i++) {
        PtnArray *array = root->live_arrays[i];
        if (
            array != NULL &&
            array->iterator_refcount != 0 &&
            ptn_array_count_reference(array, reference, 0) != 0
        ) {
            return 1;
        }
    }
    return 0;
}

static int ptn_object_has_pending_declared_destructor(PtnObject *object) {
    if (
        object == NULL ||
        !object->destructor_enabled ||
        object->destructor_called ||
        object->class_name == NULL
    ) {
        return 0;
    }
    PtnRuntime *runtime = object->lifecycle_runtime;
    if (runtime == NULL) {
        return 0;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    return root->declared_method_exists != NULL &&
        root->declared_method_exists(object->class_name, "__destruct");
}

typedef struct {
    PtnArray **arrays;
    size_t arrays_len;
    size_t arrays_capacity;
    PtnReference **references;
    size_t references_len;
    size_t references_capacity;
} PtnPendingDestructorScan;

static void ptn_pending_destructor_scan_free(PtnPendingDestructorScan *scan) {
    if (scan == NULL) {
        return;
    }
    free(scan->arrays);
    free(scan->references);
    scan->arrays = NULL;
    scan->arrays_len = 0;
    scan->arrays_capacity = 0;
    scan->references = NULL;
    scan->references_len = 0;
    scan->references_capacity = 0;
}

static int ptn_pending_destructor_scan_note_array(
    PtnPendingDestructorScan *scan,
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

static int ptn_pending_destructor_scan_note_reference(
    PtnPendingDestructorScan *scan,
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

static int ptn_value_contains_pending_destructor_impl(
    PtnValue value,
    size_t depth,
    PtnPendingDestructorScan *scan
);

static int ptn_array_contains_pending_destructor_impl(
    PtnArray *array,
    size_t depth,
    PtnPendingDestructorScan *scan
) {
    if (array == NULL || depth > 1024) {
        return 0;
    }
    if (!ptn_pending_destructor_scan_note_array(scan, array)) {
        return 0;
    }
    for (size_t i = 0; i < array->len; i++) {
        if (ptn_value_contains_pending_destructor_impl(
                array->entries[i].value,
                depth + 1,
                scan
            )) {
            return 1;
        }
    }
    return 0;
}

static int ptn_array_contains_pending_destructor(PtnArray *array, size_t depth) {
    PtnPendingDestructorScan scan = {0};
    int result = ptn_array_contains_pending_destructor_impl(array, depth, &scan);
    ptn_pending_destructor_scan_free(&scan);
    return result;
}

static int ptn_value_contains_pending_destructor_impl(
    PtnValue value,
    size_t depth,
    PtnPendingDestructorScan *scan
) {
    if (depth > 1024) {
        return 0;
    }
    if (value.type == PTN_REFERENCE) {
        if (value.as.reference == NULL ||
            !ptn_pending_destructor_scan_note_reference(scan, value.as.reference)) {
            return 0;
        }
        return ptn_value_contains_pending_destructor_impl(
            value.as.reference->value,
            depth + 1,
            scan
        );
    }
    if (value.type == PTN_ARRAY) {
        return ptn_array_contains_pending_destructor_impl(value.as.array, depth + 1, scan);
    }
    if (value.type == PTN_CLOSURE && value.as.closure != NULL) {
        if (value.as.closure->has_wrapped_callable &&
            ptn_value_contains_pending_destructor_impl(
                value.as.closure->wrapped_callable,
                depth + 1,
                scan
            )) {
            return 1;
        }
        for (size_t i = 0; i < value.as.closure->captures.len; i++) {
            if (ptn_value_contains_pending_destructor_impl(
                value.as.closure->captures.items[i].value,
                depth + 1,
                scan
            )) {
                return 1;
            }
        }
        return 0;
    }
    if (value.type != PTN_OBJECT || value.as.object == NULL) {
        return 0;
    }
    if (value.as.object->destructor_called) {
        PtnRuntime *root = ptn_runtime_root(value.as.object->lifecycle_runtime);
        if (root != NULL && root->gc_running) {
            return 1;
        }
    }
    if (ptn_object_has_pending_declared_destructor(value.as.object)) {
        return 1;
    }
    return value.as.object->properties != NULL &&
        ptn_array_contains_pending_destructor_impl(
            value.as.object->properties,
            depth + 1,
            scan
        );
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

static size_t ptn_pending_array_cycle_collections = 0;
static int ptn_pending_array_cycle_auto_flushed = 0;
static PtnReference **ptn_pending_destructor_array_cycle_references = NULL;
static size_t ptn_pending_destructor_array_cycle_references_len = 0;
static size_t ptn_pending_destructor_array_cycle_references_capacity = 0;
static int ptn_pending_destructor_array_cycle_references_draining = 0;
static PtnReference *ptn_replaced_reference_cycle_suppressed_reference = NULL;
static size_t ptn_replaced_reference_cycle_suppression_depth = 0;

/* Approximate Zend's root-buffer auto collection for many short-lived array cycles. */
#define PTN_GC_PENDING_ARRAY_AUTO_COLLECT_THRESHOLD 10000

static PTN_UNUSED void ptn_gc_auto_flush_pending_array_reference_cycles(size_t total) {
    ptn_pending_array_cycle_collections =
        total - PTN_GC_PENDING_ARRAY_AUTO_COLLECT_THRESHOLD;
    ptn_pending_array_cycle_auto_flushed = 1;
}

static PTN_UNUSED void ptn_gc_note_array_reference_cycles(size_t count) {
    if (count == 0) {
        return;
    }
    if (ptn_pending_array_cycle_collections > SIZE_MAX - count) {
        ptn_abort_out_of_memory();
    }
    size_t total = ptn_pending_array_cycle_collections + count;
    if (
        !ptn_pending_array_cycle_auto_flushed &&
        total > PTN_GC_PENDING_ARRAY_AUTO_COLLECT_THRESHOLD
    ) {
        ptn_gc_auto_flush_pending_array_reference_cycles(total);
        return;
    }
    ptn_pending_array_cycle_collections = total;
}

static PTN_UNUSED void ptn_gc_note_destructor_array_reference_cycle(size_t count) {
    if (count == 0) {
        return;
    }
    if (ptn_pending_array_cycle_collections != 0) {
        ptn_pending_array_cycle_collections = 0;
        ptn_pending_array_cycle_auto_flushed = 0;
    }
    if (ptn_pending_array_cycle_collections > SIZE_MAX - count) {
        ptn_abort_out_of_memory();
    }
    size_t total = ptn_pending_array_cycle_collections + count;
    if (
        !ptn_pending_array_cycle_auto_flushed &&
        total >= PTN_GC_PENDING_ARRAY_AUTO_COLLECT_THRESHOLD
    ) {
        ptn_gc_auto_flush_pending_array_reference_cycles(total);
    }
}

static PTN_UNUSED int ptn_gc_array_reference_auto_flushed(void) {
    return ptn_pending_array_cycle_auto_flushed;
}

static PTN_UNUSED void ptn_gc_enqueue_pending_destructor_array_cycle(PtnReference *reference) {
    if (reference == NULL || reference->refcount == 0) {
        return;
    }
    for (size_t i = 0; i < ptn_pending_destructor_array_cycle_references_len; i++) {
        if (ptn_pending_destructor_array_cycle_references[i] == reference) {
            return;
        }
    }
    if (
        ptn_pending_destructor_array_cycle_references_len ==
        ptn_pending_destructor_array_cycle_references_capacity
    ) {
        size_t new_capacity = ptn_pending_destructor_array_cycle_references_capacity == 0
            ? 8
            : ptn_pending_destructor_array_cycle_references_capacity * 2;
        if (
            new_capacity < ptn_pending_destructor_array_cycle_references_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnReference *)
        ) {
            ptn_abort_out_of_memory();
        }
        PtnReference **new_references = realloc(
            ptn_pending_destructor_array_cycle_references,
            new_capacity * sizeof(PtnReference *)
        );
        if (new_references == NULL) {
            ptn_abort_out_of_memory();
        }
        ptn_pending_destructor_array_cycle_references = new_references;
        ptn_pending_destructor_array_cycle_references_capacity = new_capacity;
    }
    ptn_reference_retain(reference);
    ptn_pending_destructor_array_cycle_references[
        ptn_pending_destructor_array_cycle_references_len++
    ] = reference;
}

static void ptn_gc_detach_unreachable_destructed_objects_in_value(
    PtnRuntime *root,
    PtnValue *value,
    size_t depth
);

static void ptn_gc_detach_unreachable_destructed_objects_in_array(
    PtnRuntime *root,
    PtnArray *array,
    size_t depth
) {
    if (array == NULL || depth > 1024) {
        return;
    }
    for (size_t i = 0; i < array->len; i++) {
        ptn_gc_detach_unreachable_destructed_objects_in_value(
            root,
            &array->entries[i].value,
            depth + 1
        );
    }
}

static void ptn_gc_detach_unreachable_destructed_objects_in_value(
    PtnRuntime *root,
    PtnValue *value,
    size_t depth
) {
    if (value == NULL || depth > 1024) {
        return;
    }
    if (value->type == PTN_REFERENCE && value->as.reference != NULL) {
        ptn_gc_detach_unreachable_destructed_objects_in_value(
            root,
            &value->as.reference->value,
            depth + 1
        );
        return;
    }
    PtnValue deref = ptn_value_deref(*value);
    if (deref.type == PTN_ARRAY) {
        ptn_gc_detach_unreachable_destructed_objects_in_array(
            root,
            deref.as.array,
            depth + 1
        );
        return;
    }
    if (deref.type != PTN_OBJECT || deref.as.object == NULL) {
        return;
    }
    PtnObject *object = deref.as.object;
    if (
        !object->destructor_called ||
        object->refcount == 0 ||
        ptn_runtime_roots_reach_object(root, object)
    ) {
        return;
    }
    PtnValue old = *value;
    *value = ptn_null();
    ptn_value_destroy(&old);
}

static PTN_UNUSED void ptn_gc_drain_pending_destructor_array_cycles(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (
        root == NULL ||
        ptn_pending_destructor_array_cycle_references_draining ||
        ptn_pending_destructor_array_cycle_references_len == 0
    ) {
        return;
    }

    ptn_pending_destructor_array_cycle_references_draining = 1;
    size_t len = ptn_pending_destructor_array_cycle_references_len;
    PtnReference **references = ptn_pending_destructor_array_cycle_references;
    ptn_pending_destructor_array_cycle_references = NULL;
    ptn_pending_destructor_array_cycle_references_len = 0;
    ptn_pending_destructor_array_cycle_references_capacity = 0;

    for (size_t i = 0; i < len; i++) {
        PtnReference *reference = references[i];
        if (
            reference != NULL &&
            reference->refcount != 0 &&
            reference->value.type == PTN_ARRAY &&
            reference->value.as.array != NULL
        ) {
            ptn_runtime_run_static_property_value_destructors(reference->value, 0);
            ptn_gc_detach_unreachable_destructed_objects_in_value(
                root,
                &reference->value,
                0
            );
        }
        ptn_reference_release(reference);
    }
    free(references);
    ptn_pending_destructor_array_cycle_references_draining = 0;
}

static PTN_UNUSED void ptn_gc_begin_replaced_reference_cycle_suppression(
    PtnReference *reference
) {
    if (reference == NULL) {
        return;
    }
    if (ptn_replaced_reference_cycle_suppression_depth == 0) {
        ptn_replaced_reference_cycle_suppressed_reference = reference;
    }
    ptn_replaced_reference_cycle_suppression_depth++;
}

static PTN_UNUSED void ptn_gc_end_replaced_reference_cycle_suppression(
    PtnReference *reference
) {
    if (
        reference == NULL ||
        ptn_replaced_reference_cycle_suppression_depth == 0 ||
        ptn_replaced_reference_cycle_suppressed_reference != reference
    ) {
        return;
    }
    ptn_replaced_reference_cycle_suppression_depth--;
    if (ptn_replaced_reference_cycle_suppression_depth == 0) {
        ptn_replaced_reference_cycle_suppressed_reference = NULL;
    }
}

static PTN_UNUSED int ptn_gc_suppresses_replaced_reference_cycle(
    PtnReference *reference
) {
    return reference != NULL &&
        ptn_replaced_reference_cycle_suppression_depth != 0 &&
        ptn_replaced_reference_cycle_suppressed_reference == reference;
}

static PTN_UNUSED void ptn_reference_destroy_storage(PtnReference *reference) {
    if (reference == NULL) {
        return;
    }
    ptn_value_destroy(&reference->value);
    free(reference->property_type_class_name);
    free(reference->property_type_text);
    free(reference->property_declaring_class);
    free(reference->property_name);
    for (size_t i = 0; i < reference->property_type_source_len; i++) {
        free(reference->property_type_sources[i].class_name);
        free(reference->property_type_sources[i].text);
        free(reference->property_type_sources[i].declaring_class);
        free(reference->property_type_sources[i].property_name);
    }
    free(reference->property_type_sources);
    free(reference);
}

static PTN_UNUSED void ptn_reference_release(PtnReference *reference) {
    if (reference == NULL) {
        return;
    }
    if (reference->gc_collecting) {
        return;
    }
    if (reference->refcount == 0) {
        return;
    }
    int enqueue_pending_destructor_cycle = 0;
    if (reference->value.type == PTN_ARRAY &&
        reference->value.as.array != NULL &&
        reference->value.as.array->refcount == 1 &&
        !ptn_reference_is_protected_by_live_array_iterator(NULL, reference)) {
        size_t internal_refs = ptn_array_count_reference(reference->value.as.array, reference, 0);
        if (
            internal_refs > 0 &&
            reference->refcount == internal_refs + 1 &&
            !ptn_gc_suppresses_replaced_reference_cycle(reference)
        ) {
            int contains_pending_destructor =
                ptn_array_contains_pending_destructor(reference->value.as.array, 0);
            if (!contains_pending_destructor) {
                if (!ptn_pending_destructor_array_cycle_references_draining) {
                    ptn_gc_note_array_reference_cycles(internal_refs);
                }
                ptn_array_break_reference_cycle(reference->value.as.array, reference, 0);
            } else {
                ptn_gc_note_destructor_array_reference_cycle(internal_refs);
                if (ptn_gc_array_reference_auto_flushed()) {
                    enqueue_pending_destructor_cycle = 1;
                }
            }
        }
    }
    reference->refcount--;
    if (enqueue_pending_destructor_cycle && reference->refcount != 0) {
        ptn_gc_enqueue_pending_destructor_array_cycle(reference);
    }
    if (reference->refcount != 0) {
        return;
    }
    ptn_runtime_unregister_reference(reference->lifecycle_runtime, reference);
    ptn_reference_destroy_storage(reference);
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
    ptn_runtime_unregister_closure(closure->lifecycle_runtime, closure);
    ptn_runtime_release_object_id(closure->lifecycle_runtime, closure->object_id);
    free(closure->scope_class_name);
    free(closure->called_class_name);
    ptn_symbols_free(&closure->captures);
    if (closure->has_wrapped_callable) {
        ptn_value_destroy(&closure->wrapped_callable);
    }
    free(closure->dynamic_body);
    if (closure->dynamic_parameter_names != NULL) {
        for (size_t i = 0; i < closure->dynamic_parameter_count; i++) {
            free(closure->dynamic_parameter_names[i]);
        }
        free(closure->dynamic_parameter_names);
    }
    free(closure->bound_scope_name);
    free(closure->origin_class_name);
    free(closure->origin_method_name);
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
    if (array->destructing) {
        ptn_cow_debug_note_array_release();
        if (array->refcount > 0) {
            array->refcount--;
        }
        return;
    }
    ptn_cow_debug_assert_array_refcount(array, "release");
    ptn_cow_debug_note_array_release();
    if (array->refcount > 1) {
        array->refcount--;
        return;
    }
    array->destructing = 1;
    if (array->iterator_refcount != 0) {
        array->refcount = 0;
        return;
    }
    ptn_runtime_unregister_array(array->lifecycle_runtime, array);
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

static PTN_UNUSED void ptn_value_destroy_with_runtime_scope_at(PtnRuntime *runtime, PtnValue *value, size_t line) {
    PtnRuntime *root = runtime == NULL ? NULL : ptn_runtime_root(runtime);
    if (root == NULL) {
        ptn_value_destroy(value);
        return;
    }
    const char *previous_scope = root->destructor_access_scope;
    size_t previous_call_site_line = root->call_site_line;
    PtnTraceFrame *previous_trace_frame = root->trace_frame;
    root->destructor_access_scope = runtime->current_class_name;
    if (runtime != root && runtime->trace_frame != NULL && line != 0) {
        root->trace_frame = runtime->trace_frame;
    }
    if (line != 0) {
        root->call_site_line = line;
    } else if (runtime != root && runtime->call_site_line != 0) {
        root->call_site_line = runtime->call_site_line;
    } else if (
        runtime != root &&
        runtime->trace_frame != NULL &&
        runtime->trace_frame->line != 0
    ) {
        root->call_site_line = runtime->trace_frame->line;
    }
    PtnTryFrame frame;
    int caught_exception = 0;
    int frame_active = root->exceptions != NULL;
    if (frame_active) {
        ptn_try_frame_push(root, &frame);
        if (setjmp(frame.jump) != 0) {
            caught_exception = 1;
        }
    }
    if (!caught_exception) {
        ptn_value_destroy(value);
    }
    if (frame_active) {
        ptn_try_frame_pop(root, &frame);
    }
    root->trace_frame = previous_trace_frame;
    root->call_site_line = previous_call_site_line;
    root->destructor_access_scope = previous_scope;
    if (caught_exception) {
        ptn_rethrow_exception(root);
    }
}

static PTN_UNUSED void ptn_value_destroy_with_runtime_scope(PtnRuntime *runtime, PtnValue *value) {
    ptn_value_destroy_with_runtime_scope_at(runtime, value, 0);
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

static PTN_UNUSED PtnValue ptn_value_clone_deref_preserve_return_reference_fallback(PtnValue value) {
    int is_fallback = ptn_value_is_return_reference_fallback(ptn_value_deref(value));
    PtnValue clone = ptn_value_clone_deref(value);
    if (is_fallback) {
        clone = ptn_value_mark_return_reference_fallback(clone);
    }
    return clone;
}

static void ptn_symbols_init(PtnSymbolTable *symbols) {
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
    symbols->index_slots = NULL;
    symbols->index_capacity = 0;
    symbols->mutation_epoch = 0;
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
    symbols->mutation_epoch = 0;
}

static PTN_UNUSED void ptn_symbols_free_with_runtime_scope(PtnSymbolTable *symbols, PtnRuntime *runtime) {
    for (size_t i = 0; i < symbols->len; i++) {
        free(symbols->items[i].name);
        ptn_value_destroy_with_runtime_scope(runtime, &symbols->items[i].value);
    }
    free(symbols->index_slots);
    free(symbols->items);
    symbols->items = NULL;
    symbols->len = 0;
    symbols->capacity = 0;
    symbols->index_slots = NULL;
    symbols->index_capacity = 0;
    symbols->mutation_epoch = 0;
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

static PTN_UNUSED size_t ptn_symbols_linear_find_len(PtnSymbolTable *symbols, const char *name, size_t name_len) {
    for (size_t i = 0; i < symbols->len; i++) {
        if (
            symbols->items[i].name_len == name_len &&
            memcmp(symbols->items[i].name, name, name_len) == 0
        ) {
            return i;
        }
    }
    return symbols->len;
}

static PTN_UNUSED size_t ptn_symbols_linear_find(PtnSymbolTable *symbols, const char *name) {
    return ptn_symbols_linear_find_len(symbols, name, strlen(name));
}

static PTN_UNUSED size_t ptn_symbol_index_slot_for_name_len(
    PtnSymbolTable *symbols,
    const char *name,
    size_t name_len,
    uint64_t hash
) {
    size_t mask = symbols->index_capacity - 1;
    size_t slot_index = (size_t)hash & mask;
    for (;;) {
        PtnSymbolIndexSlot *slot = &symbols->index_slots[slot_index];
        if (!slot->occupied ||
            (
                slot->hash == hash &&
                symbols->items[slot->symbol_index].name_len == name_len &&
                memcmp(symbols->items[slot->symbol_index].name, name, name_len) == 0
            )) {
            return slot_index;
        }
        slot_index = (slot_index + 1) & mask;
    }
}

static PTN_UNUSED size_t ptn_symbol_index_slot_for_name(PtnSymbolTable *symbols, const char *name, uint64_t hash) {
    return ptn_symbol_index_slot_for_name_len(symbols, name, strlen(name), hash);
}

static PTN_UNUSED void ptn_symbol_index_insert_len(
    PtnSymbolTable *symbols,
    const char *name,
    size_t name_len,
    size_t symbol_index
) {
    if (symbols->index_capacity == 0) {
        return;
    }
    uint64_t hash = ptn_symbol_name_hash_len(name, name_len);
    size_t slot_index = ptn_symbol_index_slot_for_name_len(symbols, name, name_len, hash);
    PtnSymbolIndexSlot *slot = &symbols->index_slots[slot_index];
    if (!slot->occupied) {
        slot->occupied = 1;
        slot->hash = hash;
        slot->symbol_index = symbol_index;
    }
}

static PTN_UNUSED void ptn_symbol_index_insert(PtnSymbolTable *symbols, const char *name, size_t symbol_index) {
    ptn_symbol_index_insert_len(symbols, name, strlen(name), symbol_index);
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
        ptn_symbol_index_insert_len(symbols, symbols->items[i].name, symbols->items[i].name_len, i);
    }
}
