        snprintf(buffer, buffer_len, "%lld", (long long)value.as.integer);
    } else {
        ptn_format_scalar_float(value.as.floating, buffer, buffer_len);
    }
}

static PTN_UNUSED int ptn_compare_number_and_string(PtnValue number, PtnString string, int number_is_left) {
    char number_string[128];
    ptn_number_value_to_string(number, number_string, sizeof(number_string));
    size_t number_len = strlen(number_string);
    int compared = ptn_compare_string_bytes(
        (const unsigned char *)number_string,
        number_len,
        string.data,
        string.len
    );
    return number_is_left ? compared : -compared;
}

static PTN_UNUSED int ptn_compare_number_types(PtnValue left, PtnValue right, int *compared) {
    if (left.type == PTN_INT) {
        if (right.type == PTN_INT) {
            *compared = ptn_compare_integers(left.as.integer, right.as.integer);
            return 1;
        }
        if (right.type == PTN_FLOAT) {
            *compared = ptn_compare_numbers((double)left.as.integer, right.as.floating);
            return 1;
        }
        return 0;
    }
    if (left.type == PTN_FLOAT) {
        if (right.type == PTN_INT) {
            *compared = ptn_compare_numbers(left.as.floating, (double)right.as.integer);
            return 1;
        }
        if (right.type == PTN_FLOAT) {
            *compared = ptn_compare_numbers(left.as.floating, right.as.floating);
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_strings_loose(PtnString left, PtnString right) {
    double left_number = 0.0;
    double right_number = 0.0;
    if (!ptn_string_has_embedded_nul(left) &&
        !ptn_string_has_embedded_nul(right) &&
        ptn_is_numeric_string((const char *)left.data, &left_number) &&
        ptn_is_numeric_string((const char *)right.data, &right_number)) {
        return ptn_compare_numbers(left_number, right_number);
    }
    return ptn_compare_value_strings(left, right);
}

static PTN_UNUSED int ptn_compare_equal(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_identical(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_not_identical(PtnValue left, PtnValue right);
static PTN_UNUSED int ptn_compare_order(PtnValue left, PtnValue right);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand(PtnValue value);
static PTN_UNUSED PtnStringOperand ptn_value_to_string_operand_with_runtime(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
);
static PTN_UNUSED void ptn_string_operand_free(PtnStringOperand operand);
static PTN_UNUSED int ptn_float_to_int_loses_precision(double value);
static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    double value,
    const char *path,
    size_t line
);
static PTN_UNUSED PtnArray *ptn_runtime_array_detach_variable(PtnRuntime *runtime, const char *name);
static PTN_UNUSED PtnArray *ptn_value_replace_with_empty_array(PtnValue *value);

static PTN_UNUSED PtnArrayEntry *ptn_array_entry_for_key(PtnArray *array, PtnArrayKey key) {
    size_t index = ptn_array_find_key(array, key);
    return index < array->len ? &array->entries[index] : NULL;
}

static PTN_UNUSED const char *ptn_offset_container_type_name(PtnValue value) {
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
        case PTN_CLOSURE:
            return "object";
        case PTN_EXCEPTION:
            return "object";
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED void ptn_emit_array_runtime_diagnostic_at_path(
    const char *kind,
    const char *message,
    const char *path,
    size_t line
) {
    fputc('\n', stdout);
    fputs(kind, stdout);
    fputs(": ", stdout);
    fputs(message, stdout);
    fputs(" in ", stdout);
    fputs(path, stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_array_runtime_diagnostic(const char *kind, const char *message, size_t line) {
    ptn_emit_array_runtime_diagnostic_at_path(kind, message, "ptn", line);
}

static PTN_UNUSED void ptn_emit_array_runtime_warning(PtnRuntime *runtime, const char *message, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED void ptn_emit_resource_offset_warning(PtnRuntime *runtime, PtnResource *resource, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "Resource ID#%lld used as offset, casting to integer (%lld)",
        (long long)resource->id,
        (long long)resource->id
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    fputc('\n', stdout);
    ptn_emit_warning(&runtime->diagnostics, message, line);
}

static PTN_UNUSED int ptn_class_name_is_stdclass(const char *class_name) {
    const char *stdclass = "stdClass";
    while (*class_name != '\0' && *stdclass != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*stdclass)) {
            return 0;
        }
        class_name++;
        stdclass++;
    }
    return *class_name == '\0' && *stdclass == '\0';
}

static PTN_UNUSED PtnValue ptn_new_object(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    (void)args;
    (void)line;
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_internal_class_name_is_reflection_function(class_name)) {
        return ptn_reflection_function_new(runtime, argc, args, line);
    }
#endif
    if (!ptn_class_name_is_stdclass(class_name)) {
        char message[192];
        int written = snprintf(message, sizeof(message), "Class \"%s\" not found", class_name);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return ptn_null();
    }
    if (argc != 0) {
        ptn_throw_exception(runtime, "ArgumentCountError", "stdClass constructor expects 0 arguments");
        return ptn_null();
    }
    return ptn_object_new_shell(runtime, "stdClass");
}

static PTN_UNUSED PtnValue ptn_clone_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type != PTN_OBJECT) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "clone(): Argument #1 ($object) must be of type object, %s given",
            ptn_offset_container_type_name(resolved)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    PtnObject *source = resolved.as.object;
    if (source->native_data != NULL) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Trying to clone an uncloneable object of class %s",
            source->class_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return ptn_null();
    }

    PtnValue clone = ptn_object_new_shell(runtime, source->class_name);
    PtnObject *cloned = clone.as.object;
    ptn_array_free(cloned->properties);
    cloned->properties = ptn_array_clone(source->properties);
    for (size_t i = 0; i < source->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &source->property_metadata[i];
        ptn_object_register_property_metadata(
            cloned,
            metadata->display_name,
            metadata->declaring_class,
            metadata->read_visibility,
            metadata->set_visibility
        );
    }

    PtnRuntime *root = runtime == NULL || runtime->lifecycle_root == NULL
        ? runtime
        : runtime->lifecycle_root;
    if (root != NULL &&
        root->method_dispatch != NULL &&
        root->declared_method_exists != NULL &&
        root->declared_method_exists(cloned->class_name, "__clone")) {
        PtnValue result = root->method_dispatch(root, clone, "__clone", 0, NULL, line);
        ptn_value_destroy(&result);
    }
    return clone;
}

static PTN_UNUSED PtnValue ptn_cast_object(PtnRuntime *runtime, PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT) {
        return ptn_value_clone(value);
    }

    PtnValue object_value = ptn_object_new_shell(runtime, "stdClass");
    PtnObject *object = object_value.as.object;
    if (value.type == PTN_NULL) {
        return object_value;
    }

    if (value.type == PTN_ARRAY) {
        for (size_t i = 0; i < value.as.array->len; i++) {
            PtnArrayEntry *entry = &value.as.array->entries[i];
            PtnArrayKey property_key;
            if (entry->key.type == PTN_ARRAY_KEY_INT) {
                char key_buffer[64];
                int written = snprintf(
                    key_buffer,
                    sizeof(key_buffer),
                    "%lld",
                    (long long)entry->key.as.integer
                );
                if (written < 0 || (size_t)written >= sizeof(key_buffer)) {
                    ptn_abort_out_of_memory();
                }
                property_key = ptn_array_string_key(key_buffer);
            } else {
                property_key = ptn_array_string_key_len(
                    entry->key.as.string,
                    entry->key.string_len
                );
            }
            ptn_array_set_entry(
                object->properties,
                property_key,
                ptn_value_clone_deref(entry->value)
            );
        }
        return object_value;
    }

    ptn_array_set_entry(
        object->properties,
        ptn_array_string_key("scalar"),
        ptn_value_clone(value)
    );
    return object_value;
}

static PTN_UNUSED void ptn_emit_non_object_property_read_warning(
    PtnRuntime *runtime,
    const char *property,
    PtnValue receiver,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Attempt to read property \"%s\" on %s",
        property,
        ptn_offset_container_type_name(receiver)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
}

static PTN_UNUSED void ptn_emit_undefined_property_warning(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Undefined property: %s::$%s",
        object->class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
}

static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_private_property_for_scope(
    PtnObject *object,
    const char *property,
    const char *access_scope
) {
    if (object == NULL || property == NULL || access_scope == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &object->property_metadata[i];
        if (
            metadata->read_visibility == PTN_PROPERTY_PRIVATE
            && strcmp(metadata->display_name, property) == 0
            && ptn_property_class_names_equal(metadata->declaring_class, access_scope)
        ) {
            return metadata;
        }
    }
    return NULL;
}

static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_own_private_property(
    PtnObject *object,
    const char *property
) {
    if (object == NULL || property == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        PtnObjectPropertyMetadata *metadata = &object->property_metadata[i];
        if (
            metadata->read_visibility == PTN_PROPERTY_PRIVATE
            && strcmp(metadata->display_name, property) == 0
            && ptn_property_class_names_equal(metadata->declaring_class, object->class_name)
        ) {
            return metadata;
        }
    }
    return NULL;
}

static PTN_UNUSED const PtnObjectPropertyMetadata *ptn_object_named_shared_property(
    PtnObject *object,
    const char *property
) {
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(object, property);
    if (metadata != NULL && metadata->read_visibility != PTN_PROPERTY_PRIVATE) {
        return metadata;
    }
    return NULL;
}

static PTN_UNUSED int ptn_object_public_property_slot_exists(
    PtnObject *object,
    const char *property
) {
    if (object == NULL || property == NULL) {
        return 0;
    }
    PtnArrayKey key = ptn_array_string_key(property);
    PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
    ptn_array_key_free(key);
    return entry != NULL;
}

static PTN_UNUSED char *ptn_object_resolve_property_storage_key(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int for_write,
    int quiet
) {
    const PtnObjectPropertyMetadata *scoped_private =
        ptn_object_private_property_for_scope(object, property, access_scope);
    if (scoped_private != NULL) {
        PtnPropertyVisibility visibility = for_write
            ? scoped_private->set_visibility
            : scoped_private->read_visibility;
        if (!ptn_property_visibility_allows(
            runtime,
            visibility,
            scoped_private->declaring_class,
            access_scope
        )) {
            if (quiet) {
                return NULL;
            }
            if (for_write && scoped_private->set_visibility != scoped_private->read_visibility) {
                ptn_throw_property_set_visibility_error(
                    runtime,
                    scoped_private->set_visibility,
                    scoped_private->declaring_class,
                    property,
                    access_scope
                );
            } else {
                ptn_throw_property_visibility_error(
                    runtime,
                    visibility,
                    scoped_private->declaring_class,
                    property
                );
            }
            return NULL;
        }
        return ptn_duplicate_string(scoped_private->storage_name);
    }
    const PtnObjectPropertyMetadata *shared_property =
        ptn_object_named_shared_property(object, property);
    if (shared_property != NULL) {
        PtnPropertyVisibility visibility = for_write
            ? shared_property->set_visibility
            : shared_property->read_visibility;
        if (!ptn_property_visibility_allows(
            runtime,
            visibility,
            shared_property->declaring_class,
            access_scope
        )) {
            if (quiet) {
                return NULL;
            }
            if (for_write && shared_property->set_visibility != shared_property->read_visibility) {
                ptn_throw_property_set_visibility_error(
                    runtime,
                    shared_property->set_visibility,
                    shared_property->declaring_class,
                    property,
                    access_scope
                );
            } else {
                ptn_throw_property_visibility_error(
                    runtime,
                    visibility,
                    shared_property->declaring_class,
                    property
                );
            }
            return NULL;
        }
        return ptn_duplicate_string(shared_property->storage_name);
    }
    const PtnObjectPropertyMetadata *own_private =
        ptn_object_own_private_property(object, property);
    if (own_private == NULL) {
        return ptn_duplicate_string(property);
    }
    if (quiet) {
        return NULL;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot access private property %s::$%s",
        own_private->declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
    return NULL;
}

static PTN_UNUSED PtnValue ptn_object_read_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        ptn_emit_non_object_property_read_warning(runtime, property, receiver, line);
        return ptn_null();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        0,
        0
    );
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        ptn_emit_undefined_property_warning(runtime, receiver.as.object, property, line);
        return ptn_null();
    }
    return ptn_value_clone_deref(entry->value);
}

static PTN_UNUSED PtnLookupResult ptn_object_property_lookup_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return ptn_lookup_missing();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        0,
        1
    );
    if (storage_key == NULL) {
        return ptn_lookup_missing();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        return ptn_lookup_missing();
    }
    return ptn_lookup_found(ptn_value_clone_deref(entry->value));
}

static PTN_UNUSED PtnLookupResult ptn_object_property_probe_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return ptn_lookup_missing();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        0,
        1
    );
    if (storage_key == NULL) {
        return ptn_lookup_missing();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        return ptn_lookup_missing();
    }
    return ptn_lookup_found(ptn_value_clone_deref(entry->value));
}

static PTN_UNUSED PtnValue ptn_object_write_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Attempt to assign property \"%s\" on %s",
            property,
            ptn_offset_container_type_name(receiver)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return ptn_null();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        1,
        0
    );
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnValue stored = ptn_value_clone_deref(value);
    PtnValue result = ptn_value_clone(stored);
    PtnArrayKey key = ptn_array_string_key(storage_key);
    ptn_array_write_entry(receiver.as.object->properties, key, stored);
    free(storage_key);
    return result;
}

static PTN_UNUSED void ptn_object_bind_property_reference(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue reference,
    size_t line
) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Attempt to assign property \"%s\" on %s",
            property,
            ptn_offset_container_type_name(receiver)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "Error", message);
        return;
    }
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        1,
        0
    );
    if (storage_key == NULL) {
        return;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    ptn_array_set_entry(receiver.as.object->properties, key, ptn_value_clone(reference));
    free(storage_key);
}

static PTN_UNUSED void ptn_object_unset_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return;
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        1,
        1
    );
    if (storage_key == NULL) {
        return;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    ptn_array_unset_entry(receiver.as.object->properties, key);
    free(storage_key);
}

static PTN_UNUSED PtnValue ptn_object_declare_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    PtnValue value,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_OBJECT) {
        ptn_object_register_property_metadata(
            receiver.as.object,
            property,
            declaring_class,
            read_visibility,
            set_visibility
        );
    }
    return ptn_object_write_property(runtime, receiver, property, declaring_class, value, line);
}

static PTN_UNUSED void ptn_emit_null_array_offset_deprecation(PtnRuntime *runtime, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    fputc('\n', stdout);
    runtime->diagnostics.emitted_deprecation = 1;
    fputs("Deprecated: Using null as an array offset is deprecated, use an empty string instead in ptn on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_emit_false_array_conversion_deprecation(size_t line) {
    ptn_emit_array_runtime_diagnostic(
        "Deprecated",
        "Automatic conversion of false to array is deprecated",
        line
    );
}

static PTN_UNUSED PtnArray *ptn_array_convertible_scalar_for_write(PtnValue *value, size_t line) {
    if (value->type == PTN_NULL) {
        return ptn_value_replace_with_empty_array(value);
    }
    if (value->type == PTN_BOOL && !value->as.boolean) {
        ptn_emit_false_array_conversion_deprecation(line);
        return ptn_value_replace_with_empty_array(value);
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_foreach_operand_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_BOOL) {
        return value.as.boolean ? "true" : "false";
    }
    return ptn_offset_container_type_name(value);
}

static PTN_UNUSED void ptn_emit_foreach_non_array_warning(PtnValue value, const char *path, size_t line) {
    char message[128];
    snprintf(
        message,
        sizeof(message),
        "foreach() argument must be of type array|object, %s given",
        ptn_foreach_operand_type_name(value)
    );
    ptn_emit_array_runtime_diagnostic_at_path("Warning", message, path, line);
}

static PTN_UNUSED PtnArray *ptn_runtime_array_for_reference_write(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot == NULL) {
        PtnValue array = ptn_array_from_literal_entries(0, NULL);
        ptn_runtime_write_variable(runtime, name, array);
        ptn_value_destroy(&array);
        slot = ptn_symbols_value_slot(&runtime->symbols, name);
        if (slot == NULL) {
            return NULL;
        }
    }

    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_ARRAY) {
        PtnArray *array = ptn_runtime_array_detach_variable(runtime, name);
        return array != NULL ? array : value->as.array;
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(value, line);
    if (converted != NULL) {
        return converted;
    }
    if (value->type == PTN_STRING) {
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return NULL;
    }

    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED PtnArrayEntry *ptn_array_reference_entry(PtnArray *array, const PtnValue *key_value) {
    if (key_value == NULL) {
        PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
        size_t index = array->len;
        ptn_array_set_entry(array, key, ptn_reference_value(ptn_reference_new_owned(ptn_null())));
        return &array->entries[index];
    }

    PtnArrayKey key = ptn_array_key_from_value(*key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
    if (entry != NULL) {
        ptn_array_key_free(key);
        return entry;
    }

    size_t index = array->len;
    ptn_array_set_entry(array, key, ptn_reference_value(ptn_reference_new_owned(ptn_null())));
    return &array->entries[index];
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_array_dim(
    PtnRuntime *runtime,
    const char *name,
    const PtnValue *key_value,
    const char *path,
    size_t line
) {
    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    PtnArrayEntry *entry = ptn_array_reference_entry(array, key_value);
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_array_value_dim(
    PtnRuntime *runtime,
    PtnValue *container,
    const PtnValue *key_value,
    const char *path,
    size_t line
) {
    if (container == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    PtnValue *value = container->type == PTN_REFERENCE
        ? &container->as.reference->value
        : container;
    PtnArray *array = NULL;
    if (value->type == PTN_ARRAY) {
        array = ptn_array_detach_value(value);
    } else if (value->type == PTN_STRING) {
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    } else if ((array = ptn_array_convertible_scalar_for_write(value, line)) != NULL) {
        /* false/null conversion handled by shared lvalue write semantics. */
    } else {
        ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    PtnArrayEntry *entry = ptn_array_reference_entry(array, key_value);
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_runtime_bind_array_dim_reference(
    PtnRuntime *runtime,
    const char *name,
    const PtnValue *key_value,
    PtnValue reference,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return;
    }
    PtnArrayKey key = key_value == NULL
        ? ptn_array_int_key(array->next_auto_key)
        : ptn_array_key_from_value(*key_value);
    ptn_array_set_entry(array, key, ptn_value_clone(reference));
}

static PTN_UNUSED int ptn_object_property_visible_for_foreach(
    PtnRuntime *runtime,
    PtnObject *object,
    PtnArrayKey key,
    const char *access_scope
) {
    if (object == NULL || key.type != PTN_ARRAY_KEY_STRING) {
        return 1;
    }
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(object, key.as.string);
    if (metadata == NULL || metadata->read_visibility == PTN_PROPERTY_PUBLIC) {
        return 1;
    }
    return ptn_property_visibility_allows(
        runtime,
        metadata->read_visibility,
        metadata->declaring_class,
        access_scope
    );
}

static PTN_UNUSED PtnValue ptn_object_foreach_key_value(
    PtnObject *object,
    PtnArrayKey key
) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    if (object != NULL) {
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(object, key.as.string);
        if (metadata != NULL) {
            return ptn_string(metadata->display_name);
        }
    }
    if (key.string_len >= 3 && key.as.string[0] == '\0') {
        const char *second_nul = memchr(key.as.string + 1, '\0', key.string_len - 1);
        if (second_nul != NULL) {
            size_t prefix_len = (size_t)(second_nul - key.as.string) + 1;
            if (prefix_len <= key.string_len) {
                size_t display_len = key.string_len - prefix_len;
                return ptn_owned_string_len(
                    ptn_duplicate_string_len(key.as.string + prefix_len, display_len),
                    display_len
                );
            }
        }
    }
    return ptn_owned_string_len(ptn_duplicate_string_len(key.as.string, key.string_len), key.string_len);
}

static PTN_UNUSED void ptn_array_iterator_skip_invisible_object_properties(PtnArrayIterator *iterator) {
    if (iterator->object == NULL || iterator->array == NULL) {
        return;
    }
    size_t limit = iterator->live ? iterator->array->len : iterator->length;
    while (iterator->index < limit &&
        !ptn_object_property_visible_for_foreach(
            iterator->runtime,
            iterator->object,
            iterator->array->entries[iterator->index].key,
            iterator->access_scope
        )) {
        iterator->index++;
        limit = iterator->live ? iterator->array->len : iterator->length;
    }
    iterator->valid = iterator->index < limit;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_empty(void) {
    PtnArrayIterator iterator;
    iterator.array = NULL;
    iterator.object = NULL;
    iterator.runtime = NULL;
    iterator.access_scope = NULL;
    iterator.index = 0;
    iterator.length = 0;
    iterator.valid = 0;
    iterator.live = 0;
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_object_properties(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *access_scope
) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (object == NULL || object->properties == NULL) {
        return iterator;
    }
    iterator.array = object->properties;
    iterator.object = object;
    iterator.runtime = runtime;
    iterator.access_scope = access_scope;
    iterator.valid = iterator.array->len != 0;
    iterator.live = 1;
    ptn_array_iterator_retain(iterator.array);
    ptn_array_iterator_skip_invisible_object_properties(&iterator);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_value(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line
) {
    (void)runtime;
    value = ptn_value_deref(value);
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (value.type != PTN_ARRAY) {
        if (value.type == PTN_OBJECT) {
            return ptn_array_iterator_from_object_properties(runtime, value.as.object, access_scope);
        }
        ptn_emit_foreach_non_array_warning(value, path, line);
        return iterator;
    }
    iterator.array = value.as.array;
    iterator.length = iterator.array->len;
    ptn_array_retain(iterator.array);
    iterator.valid = iterator.length != 0;
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_slot(
    PtnRuntime *runtime,
    PtnValue *slot,
    const char *access_scope,
    const char *path,
    size_t line
) {
    (void)runtime;
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (slot == NULL) {
        ptn_emit_foreach_non_array_warning(ptn_null(), path, line);
        return iterator;
    }

    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_OBJECT) {
        return ptn_array_iterator_from_object_properties(runtime, value->as.object, access_scope);
    }
    if (value->type != PTN_ARRAY) {
        ptn_emit_foreach_non_array_warning(ptn_value_deref(*value), path, line);
        return iterator;
    }

    PtnArray *array = ptn_array_detach_value(value);
    if (array == NULL) {
        ptn_emit_foreach_non_array_warning(ptn_value_deref(*value), path, line);
        return iterator;
    }

    iterator.array = array;
    iterator.index = 0;
    iterator.length = 0;
    iterator.valid = array->len != 0;
    iterator.live = 1;
    ptn_array_iterator_retain(array);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_variable(
    PtnRuntime *runtime,
    const char *name,
    const char *access_scope,
    const char *path,
    size_t line
) {
    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot == NULL) {
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
        ptn_emit_foreach_non_array_warning(ptn_null(), path, line);
        return ptn_array_iterator_empty();
    }
    return ptn_array_iterator_by_ref_from_slot(runtime, slot, access_scope, path, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_reference(
    PtnRuntime *runtime,
    PtnValue reference,
    const char *access_scope,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    return ptn_array_iterator_by_ref_from_slot(runtime, &reference, access_scope, path, line);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_by_ref_from_value(
    PtnRuntime *runtime,
    PtnValue *value,
    const char *access_scope,
    const char *path,
    size_t line
) {
    return ptn_array_iterator_by_ref_from_slot(runtime, value, access_scope, path, line);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_key(PtnArrayIterator *iterator) {
    PtnArrayKey key = iterator->array->entries[iterator->index].key;
    if (iterator->object != NULL) {
        return ptn_object_foreach_key_value(iterator->object, key);
    }
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_owned_string_len(ptn_duplicate_string_len(key.as.string, key.string_len), key.string_len);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_value(PtnArrayIterator *iterator) {
    return ptn_value_borrow(iterator->array->entries[iterator->index].value);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_reference(PtnArrayIterator *iterator) {
    PtnArrayEntry *entry = &iterator->array->entries[iterator->index];
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_array_iterator_advance(PtnArrayIterator *iterator) {
    iterator->index++;
    size_t limit = iterator->live && iterator->array != NULL ? iterator->array->len : iterator->length;
    iterator->valid = iterator->array != NULL && iterator->index < limit;
    ptn_array_iterator_skip_invisible_object_properties(iterator);
}

static PTN_UNUSED void ptn_array_iterator_release(PtnArray *array) {
    if (array == NULL) {
        return;
    }
    if (array->iterator_refcount == 0) {
        return;
    }
    array->iterator_refcount--;
    if (array->iterator_refcount == 0 && array->refcount == 0) {
        ptn_array_destroy_storage(array);
    }
}

static PTN_UNUSED void ptn_array_iterator_destroy(PtnArrayIterator *iterator) {
    if (iterator->array != NULL) {
        if (iterator->live) {
            ptn_array_iterator_release(iterator->array);
        } else {
            ptn_array_free(iterator->array);
        }
        iterator->array = NULL;
    }
    iterator->object = NULL;
    iterator->runtime = NULL;
    iterator->access_scope = NULL;
    iterator->index = 0;
    iterator->length = 0;
    iterator->valid = 0;
    iterator->live = 0;
}

static PTN_UNUSED char *ptn_array_key_diagnostic_name(PtnArrayKey key) {
    char buffer[64];
    if (key.type == PTN_ARRAY_KEY_INT) {
        int written = snprintf(buffer, sizeof(buffer), "%lld", (long long)key.as.integer);
        if (written < 0 || (size_t)written >= sizeof(buffer)) {
            ptn_abort_out_of_memory();
        }
        return ptn_duplicate_string(buffer);
    }

    size_t key_len = key.string_len;
    if (key_len > SIZE_MAX - 3) {
        ptn_abort_out_of_memory();
    }
    char *display = malloc(key_len + 3);
    if (display == NULL) {
        ptn_abort_out_of_memory();
    }
    display[0] = '"';
    memcpy(display + 1, key.as.string, key_len);
    display[key_len + 1] = '"';
    display[key_len + 2] = '\0';
    return display;
}

static PTN_UNUSED void ptn_emit_undefined_array_key_warning(PtnRuntime *runtime, PtnArrayKey key, size_t line) {
    const char *prefix = "Undefined array key ";
    char *display = ptn_array_key_diagnostic_name(key);
    size_t prefix_len = strlen(prefix);
    size_t display_len = strlen(display);
    if (prefix_len > SIZE_MAX - display_len - 1) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc(prefix_len + display_len + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(message, prefix, prefix_len);
    memcpy(message + prefix_len, display, display_len + 1);
    ptn_emit_array_runtime_warning(runtime, message, line);
    free(message);
    free(display);
}

static PTN_UNUSED void ptn_emit_string_offset_cast_warning(size_t line) {
    ptn_emit_array_runtime_diagnostic("Warning", "String offset cast occurred", line);
}

static PTN_UNUSED void ptn_emit_illegal_string_offset_warning(const char *key, size_t line) {
    const char *prefix = "Illegal string offset \"";
    size_t prefix_len = strlen(prefix);
    size_t key_len = strlen(key);
    if (key_len > SIZE_MAX - prefix_len - 2) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc(prefix_len + key_len + 2);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(message, prefix, prefix_len);
    memcpy(message + prefix_len, key, key_len);
    message[prefix_len + key_len] = '"';
    message[prefix_len + key_len + 1] = '\0';
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
    free(message);
}

static PTN_UNUSED void ptn_emit_uninitialized_string_offset_warning(int64_t offset, size_t line) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Uninitialized string offset %lld", (long long)offset);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED void ptn_emit_illegal_string_offset_integer_warning(int64_t offset, size_t line) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Illegal string offset %lld", (long long)offset);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_array_runtime_diagnostic("Warning", message, line);
}

static PTN_UNUSED void ptn_emit_string_offset_assignment_byte_warning(size_t line) {
    ptn_emit_array_runtime_diagnostic(
        "Warning",
        "Only the first byte will be assigned to the string offset",
        line
    );
}

static PTN_UNUSED int ptn_string_to_offset(const char *string, int64_t *offset, int *warn_illegal) {
    const char *cursor = string;
    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }

    const char *number_start = cursor;
    if (*cursor == '-' || *cursor == '+') {
        cursor++;
    }
    if (!isdigit((unsigned char)*cursor)) {
        return 0;
    }
    while (isdigit((unsigned char)*cursor)) {
        cursor++;
    }

    char *end = NULL;
    errno = 0;
    long long parsed = strtoll(number_start, &end, 10);
    if (errno == ERANGE || end == number_start) {
        return 0;
    }

    cursor = end;
    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }
    if (*cursor == '\0') {
        *offset = (int64_t)parsed;
        return 1;
    }
    if (*cursor == '.') {
        return 0;
    }

    *offset = (int64_t)parsed;
    *warn_illegal = 1;
    return 1;
}

static PTN_UNUSED int ptn_string_offset_from_value(
    PtnRuntime *runtime,
    PtnValue key_value,
    size_t line,
    int quiet,
    int64_t *offset
) {
    key_value = ptn_value_deref(key_value);
    switch (key_value.type) {
        case PTN_INT:
            *offset = key_value.as.integer;
            return 1;
        case PTN_BOOL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = key_value.as.boolean ? 1 : 0;
            return 1;
        case PTN_NULL:
            if (!quiet) {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = 0;
            return 1;
        case PTN_FLOAT:
            if (quiet) {
                if (ptn_float_to_int_loses_precision(key_value.as.floating)) {
                    ptn_emit_float_to_int_precision_deprecation_at(
                        &runtime->diagnostics,
                        key_value.as.floating,
                        runtime->source_path == NULL ? "ptn" : runtime->source_path,
                        line
                    );
                }
            } else {
                ptn_emit_string_offset_cast_warning(line);
            }
            *offset = (int64_t)key_value.as.floating;
            return 1;
        case PTN_RESOURCE:
            if (quiet) {
                return 0;
            }
            if (!quiet) {
                ptn_emit_resource_offset_warning(runtime, key_value.as.resource, line);
            }
            *offset = key_value.as.resource->id;
            return 1;
        case PTN_STRING: {
            int warn_illegal = 0;
            const char *key_string = (const char *)key_value.as.string.data;
            if (ptn_string_to_offset(key_string, offset, &warn_illegal)) {
                if (warn_illegal) {
                    if (quiet) {
                        return 0;
                    }
                    ptn_emit_illegal_string_offset_warning(key_string, line);
                }
                return 1;
            }
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type string on string");
            return 0;
        }
        case PTN_ARRAY:
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type array on string");
            return 0;
        case PTN_OBJECT:
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type object on string");
            return 0;
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            if (quiet) {
                return 0;
            }
            ptn_throw_exception(runtime, "TypeError", "Cannot access offset of type object on string");
            return 0;
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_string_offset_index(size_t string_len, int64_t offset, size_t *index) {
    if (offset >= 0) {
        uint64_t positive = (uint64_t)offset;
        if (positive >= string_len) {
            return 0;
        }
        *index = (size_t)positive;
        return 1;
    }

    uint64_t distance = (uint64_t)(-(offset + 1)) + 1;
    if (distance > string_len) {
        return 0;
    }
    *index = string_len - (size_t)distance;
    return 1;
}

static PTN_UNUSED PtnLookupResult ptn_string_offset_lookup(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line,
    int quiet
) {
    int64_t offset = 0;
    if (!ptn_string_offset_from_value(runtime, key_value, line, quiet, &offset)) {
        return ptn_lookup_missing();
    }
    size_t index = 0;
    if (!ptn_string_offset_index(container.as.string.len, offset, &index)) {
        if (!quiet) {
            ptn_emit_uninitialized_string_offset_warning(offset, line);
            return ptn_lookup_found(ptn_string(""));
        }
        return ptn_lookup_missing();
    }

    char *result = malloc(2);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = (char)container.as.string.data[index];
    result[1] = '\0';
    return ptn_lookup_found(ptn_owned_string_len(result, 1));
}

static PTN_UNUSED int ptn_string_offset_assignment_index(
    size_t string_len,
    int64_t offset,
    size_t line,
    size_t *index,
    size_t *new_len
) {
    if (offset >= 0) {
        uint64_t positive = (uint64_t)offset;
        if (positive >= (uint64_t)SIZE_MAX - 1) {
            ptn_abort_out_of_memory();
        }
        *index = (size_t)positive;
        *new_len = *index >= string_len ? *index + 1 : string_len;
        return 1;
    }

    if (ptn_string_offset_index(string_len, offset, index)) {
        *new_len = string_len;
        return 1;
    }

    ptn_emit_illegal_string_offset_integer_warning(offset, line);
    return 0;
}

static PTN_UNUSED unsigned char ptn_string_offset_assignment_byte(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
    }

    PtnStringOperand string = ptn_value_to_string_operand_with_runtime(runtime, value, line);
    if (string.len == 0) {
        ptn_string_operand_free(string);
        ptn_throw_exception(runtime, "Error", "Cannot assign an empty string to a string offset");
        return 0;
    }
    if (string.len > 1) {
        ptn_emit_string_offset_assignment_byte_warning(line);
    }

    unsigned char byte = (unsigned char)string.data[0];
    ptn_string_operand_free(string);
    return byte;
}

static PTN_UNUSED void ptn_runtime_string_offset_set(
    PtnRuntime *runtime,
    PtnValue *target,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    if (target == NULL || target->type != PTN_STRING) {
        return;
    }

    int64_t offset = 0;
    if (!ptn_string_offset_from_value(runtime, key_value, line, 0, &offset)) {
        return;
    }

    size_t index = 0;
    size_t new_len = 0;
    if (!ptn_string_offset_assignment_index(target->as.string.len, offset, line, &index, &new_len)) {
        return;
    }

    unsigned char byte = ptn_string_offset_assignment_byte(runtime, value, line);
    ptn_cow_debug_note_string_detach();
    ptn_value_detach_for_write(target);
    if (target->as.string.len != new_len) {
        ptn_string_value_resize(target, new_len);
    }
    target->as.string.payload->data[index] = byte;
    ptn_string_value_refresh(target);
}

static PTN_UNUSED PtnLookupResult ptn_offset_lookup(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line, int quiet) {
    container = ptn_value_deref(container);
    key_value = ptn_value_deref(key_value);
    if (container.type == PTN_STRING) {
        return ptn_string_offset_lookup(runtime, container, key_value, line, quiet);
    }

    if (container.type != PTN_ARRAY) {
        if (!quiet) {
            const char *prefix = "Trying to access array offset on ";
            const char *type_name = ptn_offset_container_type_name(container);
            char message[128];
            int written = snprintf(message, sizeof(message), "%s%s", prefix, type_name);
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_array_runtime_diagnostic("Warning", message, line);
        }
        return ptn_lookup_missing();
    }

    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(runtime, line);
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    if (entry == NULL) {
        if (!quiet) {
            ptn_emit_undefined_array_key_warning(runtime, key, line);
        }
        ptn_array_key_free(key);
        return ptn_lookup_missing();
    }
    PtnValue value = ptn_value_clone_deref(entry->value);
    ptn_array_key_free(key);
    return ptn_lookup_found(value);
}

static PTN_UNUSED PtnValue ptn_array_read(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    PtnLookupResult result = ptn_offset_lookup(runtime, container, key_value, line, 0);
    if (!result.exists) {
        return ptn_null();
    }
    return result.value;
}

static PTN_UNUSED int ptn_offset_is_set(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    container = ptn_value_deref(container);
    key_value = ptn_value_deref(key_value);
    if (container.type == PTN_STRING) {
        int64_t offset = 0;
        size_t index = 0;
        return ptn_string_offset_from_value(runtime, key_value, line, 1, &offset) &&
            ptn_string_offset_index(container.as.string.len, offset, &index);
    }

    if (container.type != PTN_ARRAY) {
        return 0;
    }
    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(runtime, line);
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    int result = entry != NULL && ptn_value_deref(entry->value).type != PTN_NULL;
    ptn_array_key_free(key);
    return result;
}

static PTN_UNUSED int ptn_offset_is_empty(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line) {
    container = ptn_value_deref(container);
    key_value = ptn_value_deref(key_value);
    if (container.type == PTN_STRING) {
        int64_t offset = 0;
        size_t index = 0;
        if (!ptn_string_offset_from_value(runtime, key_value, line, 1, &offset) ||
            !ptn_string_offset_index(container.as.string.len, offset, &index)) {
            return 1;
        }
        return container.as.string.data[index] == '0';
    }

    if (container.type != PTN_ARRAY) {
        return 1;
    }
    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(runtime, line);
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    int result = entry == NULL || !ptn_is_truthy(ptn_value_deref(entry->value));
    ptn_array_key_free(key);
    return result;
}

static PTN_UNUSED PtnValue ptn_array_key_value(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_owned_string_len(ptn_duplicate_string_len(key.as.string, key.string_len), key.string_len);
}

static PTN_UNUSED void ptn_emit_assign_op_missing_array_key(PtnRuntime *runtime, PtnValue key_value, size_t line) {
    key_value = ptn_value_deref(key_value);
    if (key_value.type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(runtime, line);
    }
    PtnArrayKey key = ptn_array_key_from_value(key_value);
    ptn_emit_undefined_array_key_warning(runtime, key, line);
    ptn_array_key_free(key);
}

static PTN_UNUSED void ptn_runtime_array_warn_missing_base_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnValue container;
    if (!ptn_symbols_get(&runtime->symbols, name, &container)) {
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
    }
}

static PTN_UNUSED PtnArray *ptn_runtime_array_detach_variable(PtnRuntime *runtime, const char *name) {
    size_t index = ptn_symbols_find(&runtime->symbols, name);
    if (index >= runtime->symbols.len) {
        return NULL;
    }
    PtnValue *value = &runtime->symbols.items[index].value;
    if (value->type == PTN_REFERENCE) {
        return ptn_array_detach_value(&value->as.reference->value);
    }
    if (value->type != PTN_ARRAY) {
        return NULL;
    }
    return ptn_array_detach_value(value);
}

static PTN_UNUSED void ptn_runtime_separate_array_variable(PtnRuntime *runtime, const char *name) {
    (void)ptn_runtime_array_detach_variable(runtime, name);
}

static PTN_UNUSED PtnArray *ptn_value_replace_with_empty_array(PtnValue *value) {
    ptn_value_destroy(value);
    *value = ptn_array_from_literal_entries(0, NULL);
    return value->as.array;
}

static PTN_UNUSED PtnArray *ptn_array_root_slot_for_write(
    PtnRuntime *runtime,
    PtnValue *slot,
    size_t line
) {
    if (slot == NULL) {
        return NULL;
    }
    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_ARRAY) {
        return ptn_array_detach_value(value);
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(value, line);
    if (converted != NULL) {
        return converted;
    }
    (void)runtime;
    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED PtnArray *ptn_runtime_array_root_for_write(
    PtnRuntime *runtime,
    const char *name,
    size_t line
) {
    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot != NULL) {
        return ptn_array_root_slot_for_write(runtime, slot, line);
    }

    PtnValue array = ptn_array_from_literal_entries(0, NULL);
    ptn_runtime_write_variable(runtime, name, array);
    ptn_value_destroy(&array);
    slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot == NULL) {
        return NULL;
    }
    return ptn_array_root_slot_for_write(runtime, slot, line);
}

static PTN_UNUSED PtnArrayKey ptn_array_path_segment_key(
    PtnArray *array,
    const PtnArrayPathSegment *segment
) {
    if (segment->append) {
        return ptn_array_int_key(array->next_auto_key);
    }
    return ptn_array_key_from_value(segment->value);
}

static PTN_UNUSED int ptn_runtime_is_globals_name(const char *name) {
    return strcmp(name, "GLOBALS") == 0;
}

static PTN_UNUSED char *ptn_runtime_global_name_from_segment(
    const PtnArrayPathSegment *segment
) {
    if (segment->append) {
        return NULL;
    }
    return ptn_value_to_string(segment->value);
}

static PTN_UNUSED PtnLookupResult ptn_runtime_globals_array_path_lookup_quiet(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_lookup_found(ptn_runtime_globals_snapshot(runtime));
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return ptn_lookup_missing();
    }

    PtnLookupResult root = ptn_runtime_read_global_variable_quiet(runtime, global_name);
    free(global_name);
    if (!root.exists || segment_count == 1) {
        return root;
    }

    PtnValue container = ptn_value_deref(root.value);
    PtnValue owned_container = ptn_null();
    int has_owned_container = 0;
    for (size_t i = 1; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return ptn_lookup_missing();
        }

        PtnLookupResult result = ptn_offset_lookup(runtime, container, segment->value, line, 1);
        if (!result.exists || i + 1 == segment_count) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return result;
        }

        if (has_owned_container) {
            ptn_value_destroy(&owned_container);
        }
        owned_container = result.value;
        has_owned_container = 1;
        container = ptn_value_deref(owned_container);
    }

    if (has_owned_container) {
        ptn_value_destroy(&owned_container);
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED PtnLookupResult ptn_runtime_array_path_lookup_quiet(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (ptn_runtime_is_globals_name(name)) {
        return ptn_runtime_globals_array_path_lookup_quiet(runtime, segments, segment_count, line);
    }
    if (segment_count == 0) {
        return ptn_lookup_missing();
    }

    PtnLookupResult root = ptn_runtime_read_variable_quiet(runtime, name);
    if (!root.exists) {
        return ptn_lookup_missing();
    }

    PtnValue container = ptn_value_deref(root.value);
    PtnValue owned_container = ptn_null();
    int has_owned_container = 0;
    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return ptn_lookup_missing();
        }

        PtnLookupResult result = ptn_offset_lookup(runtime, container, segment->value, line, 1);
        if (!result.exists || i + 1 == segment_count) {
            if (has_owned_container) {
                ptn_value_destroy(&owned_container);
            }
            return result;
        }

        if (has_owned_container) {
            ptn_value_destroy(&owned_container);
        }
        owned_container = result.value;
        has_owned_container = 1;
        container = ptn_value_deref(owned_container);
    }

    if (has_owned_container) {
        ptn_value_destroy(&owned_container);
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED void ptn_array_path_emit_null_key_deprecation(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_null_key_deprecation
) {
    if (emit_null_key_deprecation && !segment->append && ptn_value_deref(segment->value).type == PTN_NULL) {
        ptn_emit_null_array_offset_deprecation(runtime, line);
    }
}

static PTN_UNUSED PtnArray *ptn_array_descend_for_reference_write(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    const char *path,
    size_t line
) {
    PtnArrayKey key = ptn_array_path_segment_key(array, segment);
    PtnArrayEntry *entry = segment->append ? NULL : ptn_array_entry_for_key(array, key);

    if (entry == NULL) {
        PtnValue child = ptn_array_from_literal_entries(0, NULL);
        ptn_array_set_entry(array, key, child);
        return array->entries[array->len - 1].value.as.array;
    }

    ptn_array_key_free(key);
    PtnValue *entry_value = entry->value.type == PTN_REFERENCE
        ? &entry->value.as.reference->value
        : &entry->value;
    if (entry_value->type == PTN_ARRAY) {
        return ptn_array_detach_value(entry_value);
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(entry_value, line);
    if (converted != NULL) {
        return converted;
    }
    if (entry_value->type == PTN_STRING) {
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return NULL;
    }

    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_array_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_runtime_reference_for_variable(runtime, name);
    }

    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_reference_write(runtime, array, &segments[i], path, line);
        if (array == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    PtnArrayEntry *entry = ptn_array_reference_entry(array, leaf->append ? NULL : &leaf->value);
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_runtime_bind_array_path_reference(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue reference,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    if (segment_count == 0) {
        ptn_runtime_bind_variable_reference(runtime, name, reference);
        return;
    }

    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return;
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_reference_write(runtime, array, &segments[i], path, line);
        if (array == NULL) {
            return;
        }
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    PtnArrayKey key = leaf->append
        ? ptn_array_int_key(array->next_auto_key)
        : ptn_array_key_from_value(leaf->value);
    ptn_array_set_entry(array, key, ptn_value_clone(reference));
}

static PTN_UNUSED PtnArray *ptn_array_descend_for_write(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_array_path_emit_null_key_deprecation(runtime, segment, line, emit_null_key_deprecation);
    PtnArrayKey key = ptn_array_path_segment_key(array, segment);
    PtnArrayEntry *entry = segment->append ? NULL : ptn_array_entry_for_key(array, key);

    if (entry == NULL) {
        PtnValue child = ptn_array_from_literal_entries(0, NULL);
        ptn_array_set_entry(array, key, child);
        return array->entries[array->len - 1].value.as.array;
    }

    ptn_array_key_free(key);
    PtnValue *entry_value = entry->value.type == PTN_REFERENCE
        ? &entry->value.as.reference->value
        : &entry->value;
    if (entry_value->type == PTN_ARRAY) {
        return ptn_array_detach_value(entry_value);
    }
    PtnArray *converted = ptn_array_convertible_scalar_for_write(entry_value, line);
    if (converted != NULL) {
        return converted;
    }

    (void)runtime;
    ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
    return NULL;
}

static PTN_UNUSED void ptn_array_set_path_leaf(
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    ptn_array_path_emit_null_key_deprecation(runtime, segment, line, emit_null_key_deprecation);
    PtnArrayKey key = ptn_array_path_segment_key(array, segment);
    ptn_array_write_entry(array, key, ptn_value_clone(ptn_value_deref(value)));
}

static PTN_UNUSED void ptn_runtime_globals_array_path_set_impl(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (segment_count == 0) {
        return;
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return;
    }

    if (segment_count == 1) {
        ptn_runtime_write_global_variable(runtime, global_name, value);
        free(global_name);
        return;
    }

    PtnValue *slot = ptn_runtime_global_variable_slot_for_write(runtime, global_name);
    free(global_name);
    if (slot == NULL) {
        return;
    }

    if (segment_count == 2) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type == PTN_STRING) {
            if (segments[1].append) {
                ptn_throw_exception(runtime, "Error", "[] operator not supported for strings");
                return;
            }
            ptn_runtime_string_offset_set(runtime, slot_value, segments[1].value, value, line);
            return;
        }
    }

    PtnArray *array = ptn_array_root_slot_for_write(runtime, slot, line);
    if (array == NULL) {
        return;
    }

    for (size_t i = 1; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_write(
            runtime,
            array,
            &segments[i],
            line,
            emit_null_key_deprecation
        );
        if (array == NULL) {
            return;
        }
    }

    ptn_array_set_path_leaf(
        runtime,
        array,
        &segments[segment_count - 1],
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED void ptn_runtime_array_path_set_impl(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (ptn_runtime_is_globals_name(name)) {
        ptn_runtime_globals_array_path_set_impl(
            runtime,
            segments,
            segment_count,
            value,
            line,
            emit_null_key_deprecation
        );
        return;
    }
    if (segment_count == 0) {
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot != NULL && segment_count == 1) {
        PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
        if (slot_value->type != PTN_STRING) {
            slot_value = NULL;
        }
        if (slot_value != NULL) {
            if (segments[0].append) {
                ptn_throw_exception(runtime, "Error", "[] operator not supported for strings");
                return;
            }
            ptn_runtime_string_offset_set(runtime, slot_value, segments[0].value, value, line);
            return;
        }
    }

    PtnArray *array = ptn_runtime_array_root_for_write(runtime, name, line);
    if (array == NULL) {
        return;
    }

    for (size_t i = 0; i + 1 < segment_count; i++) {
        array = ptn_array_descend_for_write(
            runtime,
            array,
            &segments[i],
            line,
            emit_null_key_deprecation
        );
        if (array == NULL) {
            return;
        }
    }

    ptn_array_set_path_leaf(
        runtime,
        array,
        &segments[segment_count - 1],
        value,
        line,
        emit_null_key_deprecation
    );
}

static PTN_UNUSED void ptn_runtime_array_path_set(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_path_set_impl(runtime, name, segments, segment_count, value, line, 1);
}

static PTN_UNUSED void ptn_runtime_array_path_set_from_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_path_set_impl(runtime, name, segments, segment_count, value, line, 0);
}

static PTN_UNUSED PtnValue ptn_runtime_array_path_read_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_null();
    }

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot == NULL) {
        if (!segments[0].append) {
            ptn_emit_assign_op_missing_array_key(runtime, segments[0].value, line);
        }
        return ptn_null();
    }
    PtnValue slot_value = ptn_value_deref(*slot);
    if (slot_value.type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception(runtime, "Error", "[] operator not supported for strings");
            return ptn_null();
        }
        int64_t offset = 0;
        if (!ptn_string_offset_from_value(runtime, segments[0].value, line, 0, &offset)) {
            return ptn_null();
        }
        (void)offset;
        if (segment_count > 1) {
            fflush(stdout);
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Cannot use string offset as an array",
                runtime->source_path,
                line
            );
            return ptn_null();
        }
        ptn_throw_exception(runtime, "Error", "Cannot use assign-op operators with string offsets");
        return ptn_null();
    }

    PtnValue container = ptn_value_borrow(slot_value);
    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return ptn_null();
        }
        if (ptn_value_deref(segment->value).type == PTN_NULL) {
            ptn_emit_null_array_offset_deprecation(runtime, line);
        }
        PtnArrayKey key = ptn_array_key_from_value(segment->value);
        if (container.type == PTN_ARRAY) {
            PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
            if (entry == NULL) {
                ptn_emit_undefined_array_key_warning(runtime, key, line);
                ptn_array_key_free(key);
                return ptn_null();
            }
            ptn_array_key_free(key);
            if (i + 1 == segment_count) {
                return ptn_value_clone(entry->value);
            }
            container = ptn_value_deref(entry->value);
            continue;
        }
        if (container.type == PTN_NULL) {
            ptn_emit_undefined_array_key_warning(runtime, key, line);
            ptn_array_key_free(key);
            return ptn_null();
        }
        ptn_array_key_free(key);
        return ptn_null();
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_array_read_for_assign_op(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    const PtnValue *key_value,
    size_t line
) {
    (void)path;
    if (key_value == NULL) {
        return ptn_null();
    }
    PtnArrayPathSegment segment = { 0, *key_value };
    return ptn_runtime_array_path_read_for_assign_op(runtime, name, &segment, 1, line);
}

static PTN_UNUSED void ptn_runtime_array_set_impl(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    PtnArrayPathSegment segment = { 0, key_value };
    ptn_runtime_array_path_set_impl(runtime, name, &segment, 1, value, line, emit_null_key_deprecation);
}

static PTN_UNUSED void ptn_runtime_array_set(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_set_impl(runtime, name, key_value, value, line, 1);
}

static PTN_UNUSED void ptn_runtime_array_set_from_assign_op(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    ptn_runtime_array_set_impl(runtime, name, key_value, value, line, 0);
}

static PTN_UNUSED void ptn_runtime_array_append(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t line
) {
    PtnArrayPathSegment segment = { 1, ptn_null() };
    ptn_runtime_array_path_set(runtime, name, &segment, 1, value, line);
}

static PTN_UNUSED void ptn_runtime_array_path_unset(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
);

static PTN_UNUSED void ptn_runtime_globals_array_path_unset(
    PtnRuntime *runtime,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    (void)line;
    if (segment_count == 0) {
        return;
    }

    char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
    if (global_name == NULL) {
        return;
    }

    if (segment_count == 1) {
        ptn_runtime_unset_global_variable(runtime, global_name);
        free(global_name);
        return;
    }

    PtnValue *slot = ptn_runtime_global_variable_slot(runtime, global_name);
    free(global_name);
    if (slot == NULL) {
        return;
    }

    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_STRING) {
        ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        return;
    }
    if (value->type != PTN_ARRAY) {
        return;
    }

    PtnArray *array = ptn_array_detach_value(value);
    for (size_t i = 1; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return;
        }
        PtnArrayKey key = ptn_array_key_from_value(segment->value);
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
        ptn_array_key_free(key);
        if (entry == NULL) {
            return;
        }
        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        if (entry_value->type == PTN_STRING) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
            return;
        }
        if (entry_value->type != PTN_ARRAY) {
            return;
        }
        array = ptn_array_detach_value(entry_value);
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append) {
        return;
    }
    PtnArrayKey key = ptn_array_key_from_value(leaf->value);
    (void)ptn_array_unset_entry(array, key);
}

static PTN_UNUSED void ptn_runtime_array_unset(
    PtnRuntime *runtime,
    const char *name,
    PtnValue key_value,
    size_t line
) {
    (void)line;
    PtnArrayPathSegment segment = { 0, key_value };
    ptn_runtime_array_path_unset(runtime, name, &segment, 1, line);
}

static PTN_UNUSED void ptn_runtime_array_path_unset(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (ptn_runtime_is_globals_name(name)) {
        ptn_runtime_globals_array_path_unset(runtime, segments, segment_count, line);
        return;
    }
    (void)line;
    if (segment_count == 0) {
        return;
    }

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot == NULL) {
        return;
    }
    PtnValue *value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (value->type == PTN_STRING) {
        ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        return;
    }
    if (value->type != PTN_ARRAY) {
        return;
    }

    PtnArray *array = ptn_array_detach_value(value);
    for (size_t i = 0; i + 1 < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return;
        }
        PtnArrayKey key = ptn_array_key_from_value(segment->value);
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
        ptn_array_key_free(key);
        if (entry == NULL) {
            return;
        }
        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        if (entry_value->type == PTN_STRING) {
            ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
            return;
        }
        if (entry_value->type != PTN_ARRAY) {
            return;
        }
        array = ptn_array_detach_value(entry_value);
    }

    const PtnArrayPathSegment *leaf = &segments[segment_count - 1];
    if (leaf->append) {
        return;
    }
    PtnArrayKey key = ptn_array_key_from_value(leaf->value);
    (void)ptn_array_unset_entry(array, key);
}

static PTN_UNUSED PtnValue ptn_array_current_value(PtnArray *array) {
    if (array->current_index >= array->len) {
        return ptn_bool(0);
    }
    return ptn_value_share(array->entries[array->current_index].value);
}

static PTN_UNUSED PtnValue ptn_array_current_key_value(PtnArray *array) {
    if (array->current_index >= array->len) {
        return ptn_null();
    }
    return ptn_array_key_value(array->entries[array->current_index].key);
}

static PTN_UNUSED PtnValue ptn_array_next_value(PtnArray *array) {
    if (array->len == 0 || array->current_index + 1 >= array->len) {
        array->current_index = array->len;
        return ptn_bool(0);
    }
    array->current_index++;
    return ptn_value_share(array->entries[array->current_index].value);
}

static PTN_UNUSED PtnValue ptn_array_reset_value(PtnArray *array) {
    array->current_index = 0;
    return ptn_array_current_value(array);
}

static PTN_UNUSED PtnValue ptn_array_end_value(PtnArray *array) {
    if (array->len == 0) {
        array->current_index = 0;
        return ptn_bool(0);
    }
    array->current_index = array->len - 1;
    return ptn_array_current_value(array);
}

static PTN_UNUSED PtnValue ptn_array_prev_value(PtnArray *array) {
    if (array->len == 0 || array->current_index == 0 || array->current_index >= array->len) {
        array->current_index = array->len;
        return ptn_bool(0);
    }
    array->current_index--;
    return ptn_array_current_value(array);
}

static PTN_UNUSED PtnValue ptn_array_pop_value(PtnArray *array) {
    if (array->len == 0) {
        array->current_index = 0;
        return ptn_null();
    }

    size_t removed_index = array->len - 1;
    PtnValue removed = array->entries[removed_index].value;
    ptn_array_key_free(array->entries[removed_index].key);
    array->len--;
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    return removed;
}

static PTN_UNUSED PtnValue ptn_array_shift_value(PtnArray *array) {
    if (array->len == 0) {
        array->current_index = 0;
        return ptn_null();
    }

    PtnValue removed = array->entries[0].value;
    ptn_array_key_free(array->entries[0].key);
    for (size_t i = 1; i < array->len; i++) {
        array->entries[i - 1] = array->entries[i];
    }
    array->len--;

    int64_t next_integer_key = 0;
    for (size_t i = 0; i < array->len; i++) {
        if (array->entries[i].key.type == PTN_ARRAY_KEY_INT) {
            array->entries[i].key.as.integer = next_integer_key;
            if (next_integer_key < INT64_MAX) {
                next_integer_key++;
            }
        }
    }

    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    return removed;
}

static PTN_UNUSED int64_t ptn_array_unshift_values(PtnArray *array, size_t argc, const PtnValue *values) {
    if (argc > SIZE_MAX - array->len) {
        ptn_abort_out_of_memory();
    }
    size_t new_len = array->len + argc;
    if (new_len > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }

    PtnArrayEntry *new_entries = NULL;
    if (new_len != 0) {
        if (new_len > SIZE_MAX / sizeof(PtnArrayEntry)) {
            ptn_abort_out_of_memory();
        }
        new_entries = malloc(new_len * sizeof(PtnArrayEntry));
        if (new_entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }

    size_t out = 0;
    int64_t next_integer_key = 0;
    for (size_t i = 0; i < argc; i++) {
        new_entries[out].key = ptn_array_int_key(next_integer_key);
        new_entries[out].value = ptn_value_clone(values[i]);
        out++;
        if (next_integer_key < INT64_MAX) {
            next_integer_key++;
        }
    }

    for (size_t i = 0; i < array->len; i++) {
        if (array->entries[i].key.type == PTN_ARRAY_KEY_INT) {
            new_entries[out].key = ptn_array_int_key(next_integer_key);
            ptn_array_key_free(array->entries[i].key);
            if (next_integer_key < INT64_MAX) {
                next_integer_key++;
            }
        } else {
            new_entries[out].key = array->entries[i].key;
        }
        new_entries[out].value = array->entries[i].value;
        out++;
    }

    free(array->entries);
    array->entries = new_entries;
    array->len = new_len;
    array->capacity = new_len;
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    return (int64_t)array->len;
}

static PTN_UNUSED uint64_t ptn_random_u64(void) {
    static uint64_t state = 0;
    if (state == 0) {
        uint64_t seed = (uint64_t)time(NULL) ^ ((uint64_t)(uintptr_t)&state << 1);
#if defined(_WIN32)
        seed ^= (uint64_t)_getpid();
#else
        seed ^= (uint64_t)getpid();
#endif
        state = seed == 0 ? 0x9e3779b97f4a7c15ULL : seed;
    }
    state += 0x9e3779b97f4a7c15ULL;
    uint64_t z = state;
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}

static PTN_UNUSED size_t ptn_random_bounded_index(size_t upper_inclusive) {
    if (upper_inclusive == (size_t)UINT64_MAX) {
        return (size_t)ptn_random_u64();
    }
    return (size_t)(ptn_random_u64() % ((uint64_t)upper_inclusive + 1ULL));
}

static int ptn_array_key_compare_ascending(PtnArrayKey left, PtnArrayKey right) {
    if (left.type == PTN_ARRAY_KEY_INT && right.type == PTN_ARRAY_KEY_INT) {
        if (left.as.integer < right.as.integer) {
            return -1;
        }
        if (left.as.integer > right.as.integer) {
            return 1;
        }
        return 0;
    }
    if (left.type == PTN_ARRAY_KEY_STRING && right.type == PTN_ARRAY_KEY_STRING) {
        return ptn_compare_string_bytes(
            (const unsigned char *)left.as.string,
            left.string_len,
            (const unsigned char *)right.as.string,
            right.string_len
        );
    }
    return left.type == PTN_ARRAY_KEY_INT ? -1 : 1;
}

static PTN_UNUSED void ptn_array_ksort_entries(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_key_compare_ascending(array->entries[j - 1].key, moving.key) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static int ptn_array_key_compare_descending(PtnArrayKey left, PtnArrayKey right) {
    return -ptn_array_key_compare_ascending(left, right);
}

static PTN_UNUSED void ptn_array_krsort_entries(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_key_compare_descending(array->entries[j - 1].key, moving.key) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static int ptn_array_value_compare_ascending(PtnValue left, PtnValue right) {
    int compared = ptn_compare_order(left, right);
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_GREATER) {
        return 1;
    }
    return 0;
}

static int ptn_array_value_compare_descending(PtnValue left, PtnValue right) {
    return -ptn_array_value_compare_ascending(left, right);
}

static int ptn_ascii_is_natural_digit(unsigned char byte) {
    return byte >= (unsigned char)'0' && byte <= (unsigned char)'9';
}

static int ptn_ascii_is_natural_space(unsigned char byte) {
    return byte == (unsigned char)' ' ||
        byte == (unsigned char)'\t' ||
        byte == (unsigned char)'\n' ||
        byte == (unsigned char)'\r' ||
        byte == (unsigned char)'\v' ||
        byte == (unsigned char)'\f';
}

static unsigned char ptn_ascii_natural_case_fold(unsigned char byte) {
    if (byte >= (unsigned char)'A' && byte <= (unsigned char)'Z') {
        return (unsigned char)(byte + ((unsigned char)'a' - (unsigned char)'A'));
    }
    return byte;
}

static int ptn_compare_natural_digit_run_left(
    const unsigned char *left,
    size_t left_len,
    size_t left_offset,
    const unsigned char *right,
    size_t right_len,
    size_t right_offset
) {
    while (1) {
        int left_digit = left_offset < left_len && ptn_ascii_is_natural_digit(left[left_offset]);
        int right_digit = right_offset < right_len && ptn_ascii_is_natural_digit(right[right_offset]);
        if (!left_digit && !right_digit) {
            return 0;
        }
        if (!left_digit) {
            return -1;
        }
        if (!right_digit) {
            return 1;
        }
        if (left[left_offset] < right[right_offset]) {
            return -1;
        }
        if (left[left_offset] > right[right_offset]) {
            return 1;
        }
        left_offset++;
        right_offset++;
    }
}

static int ptn_compare_natural_digit_run_right(
    const unsigned char *left,
    size_t left_len,
    size_t left_offset,
    const unsigned char *right,
    size_t right_len,
    size_t right_offset
) {
    int bias = 0;
    while (1) {
        int left_digit = left_offset < left_len && ptn_ascii_is_natural_digit(left[left_offset]);
        int right_digit = right_offset < right_len && ptn_ascii_is_natural_digit(right[right_offset]);
        if (!left_digit && !right_digit) {
            return bias;
        }
        if (!left_digit) {
            return -1;
        }
        if (!right_digit) {
            return 1;
        }
        if (left[left_offset] < right[right_offset]) {
            if (bias == 0) {
                bias = -1;
            }
        } else if (left[left_offset] > right[right_offset]) {
            if (bias == 0) {
                bias = 1;
            }
        }
        left_offset++;
        right_offset++;
    }
}

static int ptn_compare_natural_string_operands(PtnStringOperand left_operand, PtnStringOperand right_operand, int case_insensitive) {
    const unsigned char *left = (const unsigned char *)left_operand.data;
    const unsigned char *right = (const unsigned char *)right_operand.data;
    size_t left_offset = 0;
    size_t right_offset = 0;

    while (1) {
        while (left_offset < left_operand.len && ptn_ascii_is_natural_space(left[left_offset])) {
            left_offset++;
        }
        while (right_offset < right_operand.len && ptn_ascii_is_natural_space(right[right_offset])) {
            right_offset++;
        }

        int left_digit = left_offset < left_operand.len && ptn_ascii_is_natural_digit(left[left_offset]);
        int right_digit = right_offset < right_operand.len && ptn_ascii_is_natural_digit(right[right_offset]);
        if (left_digit && right_digit) {
            int compared = left[left_offset] == (unsigned char)'0' || right[right_offset] == (unsigned char)'0'
                ? ptn_compare_natural_digit_run_left(
                    left,
                    left_operand.len,
                    left_offset,
                    right,
                    right_operand.len,
                    right_offset
                )
                : ptn_compare_natural_digit_run_right(
                    left,
                    left_operand.len,
                    left_offset,
                    right,
                    right_operand.len,
                    right_offset
                );
            if (compared != 0) {
                return compared;
            }
        }

        if (left_offset >= left_operand.len && right_offset >= right_operand.len) {
            return 0;
        }
        if (left_offset >= left_operand.len) {
            return -1;
        }
        if (right_offset >= right_operand.len) {
            return 1;
        }
        unsigned char left_byte = left[left_offset];
        unsigned char right_byte = right[right_offset];
        if (case_insensitive) {
            left_byte = ptn_ascii_natural_case_fold(left_byte);
            right_byte = ptn_ascii_natural_case_fold(right_byte);
        }
        if (left_byte < right_byte) {
            return -1;
        }
        if (left_byte > right_byte) {
            return 1;
        }
        left_offset++;
        right_offset++;
    }
}

static int ptn_array_value_compare_natural(PtnValue left, PtnValue right) {
    PtnStringOperand left_string = ptn_value_to_string_operand(left);
    PtnStringOperand right_string = ptn_value_to_string_operand(right);
    int compared = ptn_compare_natural_string_operands(left_string, right_string, 0);
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return compared;
}

static int ptn_array_value_compare_natural_case(PtnValue left, PtnValue right) {
    PtnStringOperand left_string = ptn_value_to_string_operand(left);
    PtnStringOperand right_string = ptn_value_to_string_operand(right);
    int compared = ptn_compare_natural_string_operands(left_string, right_string, 1);
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return compared;
}

static PTN_UNUSED void ptn_array_sort_values(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_ascending(array->entries[j - 1].value, moving.value) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_key_free(array->entries[i].key);
        array->entries[i].key = ptn_array_int_key((int64_t)i);
    }
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_rsort_values(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_descending(array->entries[j - 1].value, moving.value) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_key_free(array->entries[i].key);
        array->entries[i].key = ptn_array_int_key((int64_t)i);
    }
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_asort_values(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_ascending(array->entries[j - 1].value, moving.value) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_arsort_values(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_descending(array->entries[j - 1].value, moving.value) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_natsort_values(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_natural(array->entries[j - 1].value, moving.value) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_natcasesort_values(PtnArray *array) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0 && ptn_array_value_compare_natural_case(array->entries[j - 1].value, moving.value) > 0) {
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_shuffle_values(PtnArray *array) {
    if (array->len > 1) {
        for (size_t i = array->len - 1; i > 0; i--) {
            size_t j = ptn_random_bounded_index(i);
            PtnArrayEntry tmp = array->entries[i];
            array->entries[i] = array->entries[j];
            array->entries[j] = tmp;
        }
    }
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_key_free(array->entries[i].key);
        array->entries[i].key = ptn_array_int_key((int64_t)i);
    }
    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED int64_t ptn_array_push_values(PtnArray *array, size_t argc, const PtnValue *values) {
