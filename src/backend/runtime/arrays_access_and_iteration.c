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
static PTN_UNUSED int ptn_try_object_to_string_operand(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnStringOperand *out
);
static PTN_UNUSED void ptn_string_operand_free(PtnStringOperand operand);
static PTN_UNUSED PtnNumber ptn_to_number(PtnValue value);
static PTN_UNUSED int ptn_float_to_int_loses_precision(double value);
static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    double value,
    const char *path,
    size_t line
);
static PTN_UNUSED PtnArray *ptn_runtime_array_detach_variable(PtnRuntime *runtime, const char *name);
static PTN_UNUSED PtnArray *ptn_value_replace_with_empty_array(PtnValue *value);
static PTN_UNUSED int ptn_arrayaccess_can_dispatch(
    PtnRuntime *runtime,
    PtnValue container,
    const char *method_name
);
static PTN_UNUSED PtnValue ptn_arrayaccess_read(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
);

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

static PTN_UNUSED int ptn_class_name_is_datetime(const char *class_name) {
    const char *datetime = "DateTime";
    while (*class_name != '\0' && *datetime != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*datetime)) {
            return 0;
        }
        class_name++;
        datetime++;
    }
    return *class_name == '\0' && *datetime == '\0';
}

static PTN_UNUSED int ptn_class_name_is_generator(const char *class_name) {
    const char *generator = "Generator";
    while (*class_name != '\0' && *generator != '\0') {
        if (tolower((unsigned char)*class_name) != tolower((unsigned char)*generator)) {
            return 0;
        }
        class_name++;
        generator++;
    }
    return *class_name == '\0' && *generator == '\0';
}

static PTN_UNUSED PtnValue ptn_new_exception_object(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    const char *declaring_class = ptn_exception_constructor_declaring_class(runtime, class_name);
    int is_error_exception = ptn_exception_name_equal(declaring_class, "ErrorException");
    size_t max_args = ptn_exception_constructor_max_args(declaring_class);
    if (argc > max_args) {
        ptn_throw_exception(
            runtime,
            "ArgumentCountError",
            is_error_exception
                ? "ErrorException constructor expects at most 6 arguments"
                : "Exception constructor expects at most 3 arguments"
        );
        return ptn_null();
    }
    PtnStringOperand message = ptn_exception_constructor_message(
        runtime,
        declaring_class,
        argc,
        args,
        line
    );
    int64_t code = 0;
    if (argc >= 2) {
        PtnValue code_value = ptn_value_deref(args[1]);
        if (code_value.type == PTN_INT) {
            code = code_value.as.integer;
        } else if (code_value.type == PTN_BOOL) {
            code = code_value.as.boolean ? 1 : 0;
        } else if (code_value.type == PTN_FLOAT) {
            code = (int64_t)code_value.as.floating;
        }
    }
    int64_t severity = PTN_E_ERROR;
    if (is_error_exception && argc >= 3) {
        PtnValue severity_value = ptn_value_deref(args[2]);
        if (severity_value.type == PTN_INT) {
            severity = severity_value.as.integer;
        } else if (severity_value.type == PTN_BOOL) {
            severity = severity_value.as.boolean ? 1 : 0;
        } else if (severity_value.type == PTN_FLOAT) {
            severity = (int64_t)severity_value.as.floating;
        }
    }
    const char *exception_path = runtime->source_path;
    int has_error_exception_path_argument = 0;
    if (is_error_exception && argc >= 4 && ptn_value_deref(args[3]).type != PTN_NULL) {
        PtnStringOperand path_string = ptn_value_to_string_operand_with_runtime(runtime, args[3], line);
        if (runtime->exceptions->active_exception != NULL) {
            ptn_string_operand_free(message);
            ptn_string_operand_free(path_string);
            return ptn_null();
        }
        exception_path = ptn_duplicate_string_len(path_string.data, path_string.len);
        has_error_exception_path_argument = 1;
        ptn_string_operand_free(path_string);
    }
    size_t exception_line = has_error_exception_path_argument ? 0 : line;
    if (is_error_exception && argc >= 5 && ptn_value_deref(args[4]).type != PTN_NULL) {
        PtnValue line_value = ptn_value_deref(args[4]);
        if (line_value.type == PTN_INT && line_value.as.integer >= 0) {
            exception_line = (size_t)line_value.as.integer;
        } else if (line_value.type == PTN_FLOAT && line_value.as.floating >= 0.0) {
            exception_line = (size_t)line_value.as.floating;
        }
    }
    PtnValue previous = ptn_null();
    size_t previous_index = is_error_exception ? 5 : 2;
    if (argc > previous_index) {
        PtnValue previous_value = ptn_value_deref(args[previous_index]);
        if (previous_value.type == PTN_EXCEPTION ||
            (previous_value.type == PTN_OBJECT && ptn_object_is_declared_throwable(runtime, previous_value.as.object))) {
            previous = previous_value;
        }
    }
    return ptn_exception_value(ptn_exception_new_owned(
        runtime,
        class_name,
        message.owned,
        message.len,
        code,
        previous,
        severity,
        exception_path,
        exception_line
    ));
}

static PTN_UNUSED PtnValue ptn_new_object(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    const char *lookup_class_name = ptn_symbol_name_without_leading_slash(class_name);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (ptn_internal_class_name_is_reflection_class(lookup_class_name)) {
        return ptn_reflection_class_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_reflection_function(lookup_class_name)) {
        return ptn_reflection_function_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_array_iterator(lookup_class_name)) {
        return ptn_array_iterator_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_array_object(lookup_class_name)) {
        return ptn_array_object_new(runtime, argc, args, line);
    }
    if (ptn_internal_class_name_is_iterator_iterator(lookup_class_name)) {
        return ptn_iterator_iterator_new(runtime, argc, args, line);
    }
#endif
    const char *exception_class_name = ptn_builtin_exception_class_name(lookup_class_name);
    if (exception_class_name != NULL) {
        return ptn_new_exception_object(runtime, exception_class_name, argc, args, line);
    }
    if (ptn_class_name_is_datetime(lookup_class_name)) {
        if (argc > 1) {
            ptn_throw_exception(runtime, "ArgumentCountError", "DateTime constructor expects at most 1 argument");
            return ptn_null();
        }
        return ptn_object_new_shell(runtime, "DateTime");
    }
    if (ptn_class_name_is_generator(lookup_class_name)) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "The \"Generator\" class is reserved for internal use and cannot be manually instantiated",
            runtime->source_path,
            line
        );
        return ptn_null();
    }
    if (!ptn_class_name_is_stdclass(lookup_class_name)) {
        char message[192];
        int written = snprintf(message, sizeof(message), "Class \"%s\" not found", lookup_class_name);
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
            metadata->set_visibility,
            metadata->is_readonly
        );
        PtnObjectPropertyMetadata *cloned_metadata = NULL;
        for (size_t metadata_index = 0; metadata_index < cloned->property_metadata_len; metadata_index++) {
            if (strcmp(cloned->property_metadata[metadata_index].storage_name, metadata->storage_name) == 0) {
                cloned_metadata = &cloned->property_metadata[metadata_index];
                break;
            }
        }
        if (cloned_metadata != NULL) {
            cloned_metadata->is_unset = metadata->is_unset;
            if (metadata->last_type_name != NULL) {
                cloned_metadata->last_type_name = ptn_duplicate_string(metadata->last_type_name);
            }
        }
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

static PTN_UNUSED PtnValue ptn_cast_array(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        return ptn_value_clone(value);
    }

    PtnValue array_value = ptn_array_from_literal_entries(0, NULL);
    PtnArray *array = array_value.as.array;
    if (value.type == PTN_NULL) {
        return array_value;
    }

    if (value.type == PTN_OBJECT) {
        PtnArray *properties = value.as.object->properties;
        for (size_t i = 0; i < properties->len; i++) {
            PtnArrayEntry *entry = &properties->entries[i];
            ptn_array_set_entry(
                array,
                ptn_array_key_clone(entry->key),
                ptn_value_clone_deref(entry->value)
            );
        }
        return array_value;
    }

    if (value.type == PTN_CLOSURE || value.type == PTN_EXCEPTION) {
        return array_value;
    }

    ptn_array_set_entry(array, ptn_array_int_key(0), ptn_value_clone(value));
    return array_value;
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

static PTN_UNUSED int ptn_object_property_storage_initialized(
    PtnObject *object,
    const char *storage_name
) {
    if (object == NULL || storage_name == NULL) {
        return 0;
    }
    PtnArrayKey key = ptn_array_string_key(storage_name);
    PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
    ptn_array_key_free(key);
    return entry != NULL;
}

static PTN_UNUSED PtnObjectPropertyMetadata *ptn_object_mutable_property_metadata(
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

static PTN_UNUSED PtnObjectPropertyMetadata *ptn_object_metadata_for_display_name(
    PtnObject *object,
    const char *property
) {
    if (object == NULL || property == NULL) {
        return NULL;
    }
    for (size_t i = 0; i < object->property_metadata_len; i++) {
        if (strcmp(object->property_metadata[i].display_name, property) == 0) {
            return &object->property_metadata[i];
        }
    }
    return NULL;
}

static PTN_UNUSED PtnObjectPropertyMetadata *ptn_object_blocked_magic_metadata(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int for_write
) {
    PtnObjectPropertyMetadata *metadata =
        ptn_object_metadata_for_display_name(object, property);
    if (metadata == NULL) {
        return NULL;
    }
    PtnPropertyVisibility visibility =
        for_write ? metadata->set_visibility : metadata->read_visibility;
    if (ptn_property_visibility_allows(
        runtime,
        visibility,
        metadata->declaring_class,
        access_scope
    )) {
        return NULL;
    }
    return metadata;
}

static PTN_UNUSED const char *ptn_property_value_type_name(PtnValue value) {
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
        case PTN_EXCEPTION:
            return "object";
        case PTN_REFERENCE:
            return "reference";
    }
    return "mixed";
}

static PTN_UNUSED void ptn_object_metadata_remember_value_type(
    PtnObjectPropertyMetadata *metadata,
    PtnValue value
) {
    if (metadata == NULL) {
        return;
    }
    free(metadata->last_type_name);
    metadata->last_type_name = ptn_duplicate_string(ptn_property_value_type_name(value));
}

static PTN_UNUSED int ptn_magic_property_get(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    PtnValue *value_out
) {
    if (runtime == NULL ||
        runtime->magic_property_get == NULL ||
        runtime->in_magic_property_dispatch) {
        return 0;
    }
    return runtime->magic_property_get(runtime, receiver, property, line, value_out);
}

static PTN_UNUSED int ptn_magic_property_get_exists(PtnRuntime *runtime, PtnValue receiver) {
    if (runtime == NULL ||
        runtime->magic_property_get_exists == NULL ||
        runtime->in_magic_property_dispatch) {
        return 0;
    }
    return runtime->magic_property_get_exists(runtime, receiver);
}

static PTN_UNUSED int ptn_magic_property_isset(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    int *isset_out
) {
    if (runtime == NULL ||
        runtime->magic_property_isset == NULL ||
        runtime->in_magic_property_dispatch) {
        return 0;
    }
    return runtime->magic_property_isset(runtime, receiver, property, line, isset_out);
}

static PTN_UNUSED int ptn_magic_property_set(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnValue value,
    size_t line
) {
    if (runtime == NULL ||
        runtime->magic_property_set == NULL ||
        runtime->in_magic_property_dispatch) {
        return 0;
    }
    return runtime->magic_property_set(runtime, receiver, property, value, line);
}

static PTN_UNUSED int ptn_magic_property_unset(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line
) {
    if (runtime == NULL ||
        runtime->magic_property_unset == NULL ||
        runtime->in_magic_property_dispatch) {
        return 0;
    }
    return runtime->magic_property_unset(runtime, receiver, property, line);
}

static PTN_UNUSED void ptn_throw_overloaded_property_reference_error(
    PtnRuntime *runtime,
    size_t line
) {
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Cannot assign by reference to overloaded object",
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
}

static PTN_UNUSED void ptn_throw_readonly_property_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot modify readonly property %s::$%s",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_readonly_property_initialize_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    const char *access_scope
) {
    char message[320];
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot initialize readonly property %s::$%s from global scope",
            declaring_class,
            property
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot initialize readonly property %s::$%s from scope %s",
            declaring_class,
            property,
            access_scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_uninitialized_typed_property_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Typed property %s::$%s must not be accessed before initialization",
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_dynamic_property_readonly_class_error(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot create dynamic property %s::$%s",
        class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

#define PTN_PROPERTY_ACCESS_READ 0
#define PTN_PROPERTY_ACCESS_WRITE 1
#define PTN_PROPERTY_ACCESS_INDIRECT_WRITE 2
#define PTN_PROPERTY_ACCESS_UNSET 3

static PTN_UNUSED char *ptn_object_resolve_property_storage_key(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *property,
    const char *access_scope,
    int access_mode,
    int quiet
) {
    int for_write = access_mode != PTN_PROPERTY_ACCESS_READ;
    int indirect_write = access_mode == PTN_PROPERTY_ACCESS_INDIRECT_WRITE;
    int unset_write = access_mode == PTN_PROPERTY_ACCESS_UNSET;
    const PtnObjectPropertyMetadata *scoped_private =
        ptn_object_private_property_for_scope(object, property, access_scope);
    if (scoped_private != NULL) {
        PtnPropertyVisibility visibility = (for_write || unset_write)
            ? scoped_private->set_visibility
            : scoped_private->read_visibility;
        int readonly_initialized = (for_write || unset_write) &&
            scoped_private->is_readonly &&
            ptn_object_property_storage_initialized(object, scoped_private->storage_name);
        if (readonly_initialized) {
            return ptn_duplicate_string(scoped_private->storage_name);
        }
        if (!ptn_property_visibility_allows(
            runtime,
            visibility,
            scoped_private->declaring_class,
            access_scope
        )) {
            if (quiet) {
                return NULL;
            }
            if (for_write && scoped_private->is_readonly) {
                ptn_throw_readonly_property_initialize_error(
                    runtime,
                    scoped_private->declaring_class,
                    property,
                    access_scope
                );
            } else if (for_write && scoped_private->set_visibility != scoped_private->read_visibility) {
                if (indirect_write) {
                    ptn_throw_property_indirect_set_visibility_error(
                        runtime,
                        scoped_private->set_visibility,
                        scoped_private->declaring_class,
                        property,
                        access_scope
                    );
                } else if (unset_write) {
                    ptn_throw_property_unset_visibility_error(
                        runtime,
                        scoped_private->set_visibility,
                        scoped_private->declaring_class,
                        property,
                        access_scope,
                        1
                    );
                } else {
                    ptn_throw_property_set_visibility_error(
                        runtime,
                        scoped_private->set_visibility,
                        scoped_private->declaring_class,
                        property,
                        access_scope
                    );
                }
            } else if (unset_write && scoped_private->set_visibility != scoped_private->read_visibility) {
                ptn_throw_property_unset_visibility_error(
                    runtime,
                    scoped_private->set_visibility,
                    scoped_private->declaring_class,
                    property,
                    access_scope,
                    1
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
        PtnPropertyVisibility visibility = (for_write || unset_write)
            ? shared_property->set_visibility
            : shared_property->read_visibility;
        int readonly_initialized = (for_write || unset_write) &&
            shared_property->is_readonly &&
            ptn_object_property_storage_initialized(object, shared_property->storage_name);
        if (readonly_initialized) {
            return ptn_duplicate_string(shared_property->storage_name);
        }
        if (!ptn_property_visibility_allows(
            runtime,
            visibility,
            shared_property->declaring_class,
            access_scope
        )) {
            if (quiet) {
                return NULL;
            }
            if (for_write && shared_property->is_readonly) {
                ptn_throw_readonly_property_initialize_error(
                    runtime,
                    shared_property->declaring_class,
                    property,
                    access_scope
                );
            } else if (for_write && shared_property->set_visibility != shared_property->read_visibility) {
                if (indirect_write) {
                    ptn_throw_property_indirect_set_visibility_error(
                        runtime,
                        shared_property->set_visibility,
                        shared_property->declaring_class,
                        property,
                        access_scope
                    );
                } else if (unset_write) {
                    ptn_throw_property_unset_visibility_error(
                        runtime,
                        shared_property->set_visibility,
                        shared_property->declaring_class,
                        property,
                        access_scope,
                        1
                    );
                } else {
                    ptn_throw_property_set_visibility_error(
                        runtime,
                        shared_property->set_visibility,
                        shared_property->declaring_class,
                        property,
                        access_scope
                    );
                }
            } else if (unset_write && shared_property->set_visibility != shared_property->read_visibility) {
                ptn_throw_property_unset_visibility_error(
                    runtime,
                    shared_property->set_visibility,
                    shared_property->declaring_class,
                    property,
                    access_scope,
                    1
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
        if (
            for_write &&
            runtime != NULL &&
            runtime->declared_class_is_readonly != NULL &&
            runtime->declared_class_is_readonly(object->class_name)
        ) {
            if (!quiet) {
                ptn_throw_dynamic_property_readonly_class_error(
                    runtime,
                    object->class_name,
                    property
                );
            }
            return NULL;
        }
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
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 0);
    if (blocked_metadata != NULL) {
        PtnValue magic_value = ptn_null();
        if (ptn_magic_property_get(runtime, receiver, property, line, &magic_value)) {
            return magic_value;
        }
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1
    );
    if (storage_key == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(runtime, receiver, property, line, 0, &magic_value)
        ) {
            return magic_value;
        }
        storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            0
        );
    }
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        if (metadata != NULL && metadata->is_readonly) {
            ptn_throw_uninitialized_typed_property_error(
                runtime,
                metadata->declaring_class,
                metadata->display_name
            );
            return ptn_null();
        }
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(runtime, receiver, property, line, 0, &magic_value)
        ) {
            return magic_value;
        }
        ptn_emit_undefined_property_warning(runtime, receiver.as.object, property, line);
        return ptn_null();
    }
    return ptn_value_clone_deref(entry->value);
}

static PTN_UNUSED PtnValue ptn_object_read_property_no_magic(
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
        PTN_PROPERTY_ACCESS_READ,
        0
    );
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        if (metadata != NULL && metadata->is_readonly) {
            ptn_throw_uninitialized_typed_property_error(
                runtime,
                metadata->declaring_class,
                metadata->display_name
            );
            return ptn_null();
        }
        if (metadata == NULL || metadata->is_unset) {
            PtnValue magic_value = ptn_null();
            if (ptn_magic_property_get(runtime, receiver, property, line, &magic_value)) {
                return magic_value;
            }
        }
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
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return ptn_lookup_missing();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1
    );
    if (storage_key == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(runtime, receiver, property, line, 1, &magic_value)
        ) {
            return ptn_lookup_found(magic_value);
        }
        return ptn_lookup_missing();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(runtime, receiver, property, line, 1, &magic_value)
        ) {
            return ptn_lookup_found(magic_value);
        }
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
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return ptn_lookup_missing();
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1
    );
    if (storage_key == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(runtime, receiver, property, line, 1, &magic_value)
        ) {
            return ptn_lookup_found(magic_value);
        }
        return ptn_lookup_missing();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        PtnValue magic_value;
        if (
            runtime != NULL &&
            runtime->magic_property_read != NULL &&
            runtime->magic_property_read(runtime, receiver, property, line, 1, &magic_value)
        ) {
            return ptn_lookup_found(magic_value);
        }
        return ptn_lookup_missing();
    }
    return ptn_lookup_found(ptn_value_clone_deref(entry->value));
}

static PTN_UNUSED int ptn_object_property_is_set(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_OBJECT) {
        return 0;
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_READ,
        1
    );
    if (storage_key == NULL) {
        int magic_isset = 0;
        if (ptn_magic_property_isset(runtime, receiver, property, line, &magic_isset)) {
            return magic_isset;
        }
        return 0;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        if (metadata == NULL || metadata->is_unset) {
            int magic_isset = 0;
            if (ptn_magic_property_isset(runtime, receiver, property, line, &magic_isset)) {
                return magic_isset;
            }
        }
        return 0;
    }
    return ptn_value_deref(entry->value).type != PTN_NULL;
}

static PTN_UNUSED PtnValue ptn_object_write_property_with_mode(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line,
    int indirect_write
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
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL && blocked_metadata->is_unset) {
        if (ptn_magic_property_set(runtime, receiver, property, value, line)) {
            return ptn_value_clone_deref(value);
        }
    }
    if (blocked_metadata == NULL &&
        ptn_object_metadata_for_display_name(receiver.as.object, property) == NULL &&
        ptn_magic_property_set(runtime, receiver, property, value, line)) {
        return ptn_value_clone_deref(value);
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        indirect_write ? PTN_PROPERTY_ACCESS_INDIRECT_WRITE : PTN_PROPERTY_ACCESS_WRITE,
        0
    );
    if (storage_key == NULL) {
        return ptn_null();
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    if (metadata != NULL && metadata->is_readonly && entry != NULL) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_readonly_property_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name
        );
        return ptn_null();
    }
    PtnValue stored = ptn_value_clone_deref(value);
    PtnValue result = ptn_value_clone(stored);
    PtnObjectPropertyMetadata *mutable_metadata =
        ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
    if (mutable_metadata != NULL) {
        mutable_metadata->is_unset = 0;
        ptn_object_metadata_remember_value_type(mutable_metadata, stored);
    }
    ptn_array_write_entry(receiver.as.object->properties, key, stored);
    free(storage_key);
    return result;
}

static PTN_UNUSED PtnValue ptn_object_write_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_object_write_property_with_mode(
        runtime,
        receiver,
        property,
        access_scope,
        value,
        line,
        0
    );
}

static PTN_UNUSED PtnValue ptn_object_write_property_indirect(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_object_write_property_with_mode(
        runtime,
        receiver,
        property,
        access_scope,
        value,
        line,
        1
    );
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
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL && ptn_magic_property_get_exists(runtime, receiver)) {
        ptn_throw_overloaded_property_reference_error(runtime, line);
        return;
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_WRITE,
        0
    );
    if (storage_key == NULL) {
        return;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    if (metadata != NULL && metadata->is_readonly && entry != NULL) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_readonly_property_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name
        );
        return;
    }
    ptn_array_set_entry(receiver.as.object->properties, key, ptn_value_clone(reference));
    PtnObjectPropertyMetadata *mutable_metadata =
        ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
    if (mutable_metadata != NULL) {
        mutable_metadata->is_unset = 0;
        ptn_object_metadata_remember_value_type(mutable_metadata, reference);
    }
    free(storage_key);
}

static PTN_UNUSED PtnValue ptn_object_reference_for_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
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
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL && ptn_magic_property_get_exists(runtime, receiver)) {
        ptn_throw_overloaded_property_reference_error(runtime, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
        1
    );
    if (storage_key == NULL) {
        char *read_storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            1
        );
        if (read_storage_key != NULL) {
            PtnArrayKey read_key = ptn_array_string_key(read_storage_key);
            PtnArrayEntry *read_entry =
                ptn_array_entry_for_key(receiver.as.object->properties, read_key);
            ptn_array_key_free(read_key);
            if (read_entry != NULL) {
                PtnValue current = ptn_value_clone_deref(read_entry->value);
                if (current.type == PTN_OBJECT) {
                    free(read_storage_key);
                    return ptn_reference_value(ptn_reference_new_owned(current));
                }
                ptn_value_destroy(&current);
            }
            free(read_storage_key);
        }
        storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            receiver.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_INDIRECT_WRITE,
            0
        );
        if (storage_key == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(receiver.as.object->properties, key);
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(receiver.as.object, storage_key);
    if (metadata != NULL && metadata->is_readonly && entry != NULL) {
        ptn_array_key_free(key);
        free(storage_key);
        ptn_throw_readonly_property_error(
            runtime,
            metadata->declaring_class,
            metadata->display_name
        );
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (entry == NULL) {
        PtnValue reference = ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        ptn_array_set_entry(receiver.as.object->properties, key, ptn_value_clone(reference));
        PtnObjectPropertyMetadata *mutable_metadata =
            ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
        if (mutable_metadata != NULL) {
            mutable_metadata->is_unset = 0;
            ptn_object_metadata_remember_value_type(mutable_metadata, reference);
        }
        free(storage_key);
        return reference;
    }
    ptn_array_key_free(key);
    free(storage_key);
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    return ptn_value_clone(entry->value);
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
    PtnObjectPropertyMetadata *blocked_metadata =
        ptn_object_blocked_magic_metadata(runtime, receiver.as.object, property, access_scope, 1);
    if (blocked_metadata != NULL && blocked_metadata->is_unset) {
        if (ptn_magic_property_unset(runtime, receiver, property, line)) {
            return;
        }
    }
    if (blocked_metadata != NULL &&
        blocked_metadata->set_visibility != blocked_metadata->read_visibility) {
        ptn_throw_property_unset_visibility_error(
            runtime,
            blocked_metadata->set_visibility,
            blocked_metadata->declaring_class,
            property,
            access_scope,
            1
        );
        return;
    }
    if (blocked_metadata == NULL &&
        ptn_object_metadata_for_display_name(receiver.as.object, property) == NULL &&
        ptn_magic_property_unset(runtime, receiver, property, line)) {
        return;
    }
    char *storage_key = ptn_object_resolve_property_storage_key(
        runtime,
        receiver.as.object,
        property,
        access_scope,
        PTN_PROPERTY_ACCESS_UNSET,
        0
    );
    if (storage_key == NULL) {
        return;
    }
    PtnArrayKey key = ptn_array_string_key(storage_key);
    ptn_array_unset_entry(receiver.as.object->properties, key);
    PtnObjectPropertyMetadata *mutable_metadata =
        ptn_object_mutable_property_metadata(receiver.as.object, storage_key);
    if (mutable_metadata != NULL) {
        mutable_metadata->is_unset = 1;
    }
    free(storage_key);
}

static PTN_UNUSED PtnValue ptn_object_declare_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *declaring_class,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    int is_readonly,
    int has_value,
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
            set_visibility,
            is_readonly
        );
    }
    if (!has_value) {
        return ptn_null();
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

static PTN_UNUSED int ptn_array_append_key_available(PtnRuntime *runtime, PtnArray *array) {
    for (size_t i = 0; i < array->len; i++) {
        if (array->entries[i].key.type == PTN_ARRAY_KEY_INT &&
            array->entries[i].key.as.integer == INT64_MAX) {
            goto unavailable;
        }
    }
    if (array->next_auto_key < INT64_MAX) {
        return 1;
    }
unavailable:
    ptn_throw_exception(
        runtime,
        "Error",
        "Cannot add element to the array as the next element is already occupied"
    );
    return 0;
}

static PTN_UNUSED void ptn_array_literal_append_entry(
    PtnRuntime *runtime,
    PtnArray *array,
    size_t line,
    int has_key,
    PtnValue key_value,
    PtnValue value
) {
    if (has_key) {
        if (ptn_value_deref(key_value).type == PTN_NULL) {
            ptn_emit_null_array_offset_deprecation(runtime, line);
        }
        PtnArrayKey key = ptn_array_key_from_value(key_value);
        ptn_array_set_entry(array, key, ptn_value_clone(value));
        return;
    }

    if (!ptn_array_append_key_available(runtime, array)) {
        return;
    }
    PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
    ptn_array_set_entry(array, key, ptn_value_clone(value));
}

static PTN_UNUSED void ptn_generator_data_free(void *data) {
    PtnGenerator *generator = (PtnGenerator *)data;
    if (generator == NULL) {
        return;
    }
    if (generator->values != NULL) {
        ptn_array_free(generator->values);
    }
    free(generator);
}

static PTN_UNUSED int ptn_object_is_generator(PtnObject *object) {
    return object != NULL &&
        object->native_data != NULL &&
        ptn_ascii_case_equal(object->class_name, "Generator");
}

static PTN_UNUSED PtnGenerator *ptn_generator_from_value(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT || !ptn_object_is_generator(value.as.object)) {
        return NULL;
    }
    return (PtnGenerator *)value.as.object->native_data;
}

static PTN_UNUSED PtnValue ptn_generator_new(PtnRuntime *runtime, int yields_by_ref) {
    PtnGenerator *generator = malloc(sizeof(PtnGenerator));
    if (generator == NULL) {
        ptn_abort_out_of_memory();
    }
    PtnValue values = ptn_array_from_literal_entries(0, NULL);
    generator->values = values.as.array;
    generator->yields_by_ref = yields_by_ref ? 1 : 0;

    PtnValue object = ptn_object_new_shell(runtime, "Generator");
    object.as.object->native_data = generator;
    object.as.object->native_data_free = ptn_generator_data_free;
    return object;
}

static PTN_UNUSED PtnValue ptn_generator_yield(
    PtnRuntime *runtime,
    int has_key,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    PtnGenerator *generator = runtime == NULL ? NULL : runtime->current_generator;
    if (generator == NULL || generator->values == NULL) {
        return ptn_null();
    }

    PtnValue stored;
    if (generator->yields_by_ref) {
        if (value.type == PTN_REFERENCE) {
            stored = ptn_value_clone(value);
        } else {
            ptn_emit_only_variable_references_yielded_by_reference_notice(
                &runtime->diagnostics,
                line
            );
            stored = ptn_value_clone_deref(value);
        }
    } else {
        stored = ptn_value_clone_deref(value);
    }

    if (has_key) {
        PtnArrayKey key = ptn_array_key_from_value(key_value);
        ptn_array_set_entry(generator->values, key, stored);
    } else {
        if (!ptn_array_append_key_available(runtime, generator->values)) {
            ptn_value_destroy(&stored);
            return ptn_null();
        }
        PtnArrayKey key = ptn_array_int_key(generator->values->next_auto_key);
        ptn_array_set_entry(generator->values, key, stored);
    }
    return ptn_value_clone_deref(value);
}

static PTN_UNUSED PtnValue ptn_generator_current(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)runtime;
    (void)line;
    PtnGenerator *generator = ptn_generator_from_value(receiver);
    if (generator == NULL || generator->values == NULL || generator->values->len == 0) {
        return ptn_null();
    }
    return ptn_value_clone_deref(generator->values->entries[0].value);
}

static PTN_UNUSED void ptn_emit_unpack_traversable_by_ref_warning(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argument_index,
    size_t line
) {
    if (runtime == NULL || !ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Warning: Cannot pass by-reference argument ", stdout);
    fprintf(stdout, "%zu", argument_index);
    fputs(" of ", stdout);
    fputs(function_name != NULL ? function_name : "{closure}", stdout);
    fputs("() by unpacking a Traversable, passing by-value instead in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED void ptn_array_unpack_invalid_operand_message(
    PtnValue value,
    char *message,
    size_t message_len
) {
    const char *given = ptn_offset_container_type_name(value);
    PtnValue deref_value = ptn_value_deref(value);
    if (deref_value.type == PTN_OBJECT) {
        given = deref_value.as.object->class_name;
    } else if (deref_value.type == PTN_EXCEPTION) {
        given = deref_value.as.exception->class_name;
    } else if (deref_value.type == PTN_CLOSURE) {
        given = "Closure";
    }
    snprintf(
        message,
        message_len,
        "Only arrays and Traversables can be unpacked, %s given",
        given
    );
}

static PTN_UNUSED void ptn_array_unpack_invalid_operand_fatal(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    char message[160];
    ptn_array_unpack_invalid_operand_message(value, message, sizeof(message));
    if (runtime->diagnostics.display_errors) {
        FILE *stream = runtime->diagnostics.stream == NULL ? stderr : runtime->diagnostics.stream;
        fputs("Fatal error: ", stream);
        fputs(message, stream);
        fputs(" in ", stream);
        fputs(runtime->source_path, stream);
        fputs(" on line ", stream);
        fprintf(stream, "%zu", line);
        fputc('\n', stream);
    }
    exit(255);
}

static PTN_UNUSED void ptn_array_unpack_invalid_operand_throw(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    char message[160];
    ptn_array_unpack_invalid_operand_message(value, message, sizeof(message));
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_array_unpack_array_into(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnArray *source
) {
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        PtnArrayKey key = entry->key.type == PTN_ARRAY_KEY_INT
            ? ptn_array_int_key(target->next_auto_key)
            : ptn_array_key_clone(entry->key);
        if (entry->key.type == PTN_ARRAY_KEY_INT &&
            !ptn_array_append_key_available(runtime, target)) {
            ptn_array_key_free(key);
            return;
        }
        ptn_array_set_entry(target, key, ptn_value_clone_deref(entry->value));
    }
}

static PTN_UNUSED void ptn_array_unpack_into_common(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line,
    int fatal_on_invalid_operand
) {
    PtnValue source = ptn_value_deref(value);
    if (source.type != PTN_ARRAY) {
        if (fatal_on_invalid_operand) {
            ptn_array_unpack_invalid_operand_fatal(runtime, source, line);
        } else {
            ptn_array_unpack_invalid_operand_throw(runtime, source, line);
        }
        return;
    }

    ptn_array_unpack_array_into(runtime, target, source.as.array);
}

static PTN_UNUSED void ptn_array_unpack_into(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line
) {
    ptn_array_unpack_into_common(runtime, target, value, line, 0);
}

static PTN_UNUSED void ptn_array_unpack_const_invalid(PtnRuntime *runtime, size_t line) {
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Only arrays can be unpacked in constant expression",
        runtime->source_path,
        line
    );
}

static PTN_UNUSED void ptn_array_unpack_const_into(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line
) {
    PtnValue source = ptn_value_deref(value);
    if (source.type != PTN_ARRAY) {
        ptn_array_unpack_const_invalid(runtime, line);
        return;
    }
    ptn_array_unpack_array_into(runtime, target, source.as.array);
}

static PTN_UNUSED void ptn_array_unpack_into_or_fatal(
    PtnRuntime *runtime,
    PtnArray *target,
    PtnValue value,
    size_t line
) {
    ptn_array_unpack_into_common(runtime, target, value, line, 1);
}

typedef struct {
    size_t len;
    size_t capacity;
    PtnValue *values;
} PtnCallArguments;

static PTN_UNUSED void ptn_call_arguments_init(PtnCallArguments *arguments) {
    arguments->len = 0;
    arguments->capacity = 0;
    arguments->values = NULL;
}

static PTN_UNUSED void ptn_call_arguments_reserve(PtnCallArguments *arguments, size_t additional) {
    if (additional > SIZE_MAX - arguments->len) {
        ptn_abort_out_of_memory();
    }
    size_t needed = arguments->len + additional;
    if (needed <= arguments->capacity) {
        return;
    }
    size_t next_capacity = arguments->capacity == 0 ? 8 : arguments->capacity;
    while (next_capacity < needed) {
        if (next_capacity > SIZE_MAX / 2) {
            next_capacity = needed;
            break;
        }
        next_capacity *= 2;
    }
    PtnValue *values = realloc(arguments->values, next_capacity * sizeof(PtnValue));
    if (values == NULL) {
        ptn_abort_out_of_memory();
    }
    arguments->values = values;
    arguments->capacity = next_capacity;
}

static PTN_UNUSED void ptn_call_arguments_append_owned(PtnCallArguments *arguments, PtnValue value) {
    ptn_call_arguments_reserve(arguments, 1);
    arguments->values[arguments->len++] = value;
}

static PTN_UNUSED int ptn_call_argument_index_is_by_ref(
    size_t index,
    const size_t *by_ref_indices,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index
) {
    if (has_by_ref_variadic && index >= by_ref_variadic_index) {
        return 1;
    }
    for (size_t i = 0; i < by_ref_indices_len; i++) {
        if (by_ref_indices[i] == index) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_call_arguments_unpack(PtnRuntime *runtime, PtnCallArguments *arguments, PtnValue value, size_t line) {
    PtnValue source = ptn_value_deref(value);
    PtnArray *array = NULL;
    if (source.type == PTN_ARRAY) {
        array = source.as.array;
    } else if (source.type == PTN_OBJECT) {
        PtnGenerator *generator = ptn_generator_from_value(source);
        if (generator != NULL) {
            array = generator->values;
        }
    }
    if (array == NULL) {
        char message[160];
        ptn_array_unpack_invalid_operand_message(source, message, sizeof(message));
        ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
        return;
    }

    ptn_call_arguments_reserve(arguments, array->len);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        if (entry->key.type == PTN_ARRAY_KEY_STRING) {
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Cannot use positional argument after named argument during unpacking",
                runtime->source_path,
                line
            );
            return;
        }
        ptn_call_arguments_append_owned(arguments, ptn_value_clone_deref(entry->value));
    }
}

static PTN_UNUSED void ptn_call_arguments_unpack_array_with_parameter_modes(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    PtnValue *source_value,
    const size_t *by_ref_indices,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index,
    const char *function_name,
    size_t line
) {
    PtnValue *storage = source_value != NULL && source_value->type == PTN_REFERENCE
        ? &source_value->as.reference->value
        : source_value;
    PtnValue source = storage != NULL ? ptn_value_deref(*storage) : ptn_null();
    PtnArray *array = NULL;
    int source_is_traversable = 0;
    if (source.type == PTN_ARRAY) {
        array = source.as.array;
    } else if (source.type == PTN_OBJECT) {
        PtnGenerator *generator = ptn_generator_from_value(source);
        if (generator != NULL) {
            array = generator->values;
            source_is_traversable = 1;
        }
    }
    if (array == NULL) {
        char message[160];
        ptn_array_unpack_invalid_operand_message(source, message, sizeof(message));
        ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
        return;
    }

    int needs_by_ref_entry = 0;
    for (size_t i = 0; i < array->len; i++) {
        if (ptn_call_argument_index_is_by_ref(
            arguments->len + i,
            by_ref_indices,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index
        )) {
            needs_by_ref_entry = 1;
            break;
        }
    }
    if (needs_by_ref_entry && storage != NULL && !source_is_traversable) {
        PtnArray *detached = ptn_array_detach_value(storage);
        if (detached != NULL) {
            array = detached;
        } else {
            source = ptn_value_deref(*storage);
            if (source.type == PTN_ARRAY) {
                array = source.as.array;
            }
        }
    }

    ptn_call_arguments_reserve(arguments, array->len);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        if (entry->key.type == PTN_ARRAY_KEY_STRING) {
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Cannot use positional argument after named argument during unpacking",
                runtime->source_path,
                line
            );
            return;
        }

        if (ptn_call_argument_index_is_by_ref(
            arguments->len,
            by_ref_indices,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index
        )) {
            if (source_is_traversable) {
                ptn_emit_unpack_traversable_by_ref_warning(
                    runtime,
                    function_name,
                    arguments->len + 1,
                    line
                );
                ptn_call_arguments_append_owned(
                    arguments,
                    ptn_reference_value(ptn_reference_new_owned(ptn_value_clone_deref(entry->value)))
                );
                continue;
            }
            if (entry->value.type != PTN_REFERENCE) {
                PtnValue current = entry->value;
                entry->value = ptn_reference_value(ptn_reference_new_owned(current));
            }
            ptn_call_arguments_append_owned(arguments, ptn_value_clone(entry->value));
        } else {
            ptn_call_arguments_append_owned(arguments, ptn_value_clone_deref(entry->value));
        }
    }
}

static PTN_UNUSED void ptn_call_arguments_unpack_value_with_parameter_modes(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    PtnValue *value,
    const size_t *by_ref_indices,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index,
    const char *function_name,
    size_t line
) {
    ptn_call_arguments_unpack_array_with_parameter_modes(
        runtime,
        arguments,
        value,
        by_ref_indices,
        by_ref_indices_len,
        has_by_ref_variadic,
        by_ref_variadic_index,
        function_name,
        line
    );
}

static PTN_UNUSED void ptn_call_arguments_unpack_variable_with_parameter_modes(
    PtnRuntime *runtime,
    PtnCallArguments *arguments,
    const char *name,
    const size_t *by_ref_indices,
    size_t by_ref_indices_len,
    int has_by_ref_variadic,
    size_t by_ref_variadic_index,
    const char *function_name,
    size_t line
) {
    if (strcmp(name, "GLOBALS") == 0) {
        PtnValue globals = ptn_runtime_globals_snapshot(runtime);
        ptn_call_arguments_unpack_array_with_parameter_modes(
            runtime,
            arguments,
            &globals,
            by_ref_indices,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index,
            function_name,
            line
        );
        ptn_value_destroy(&globals);
        return;
    }

    PtnValue *slot = ptn_symbols_get_slot(&runtime->symbols, name);
    if (slot == NULL) {
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, runtime->source_path, line);
        PtnValue missing = ptn_null();
        ptn_call_arguments_unpack_array_with_parameter_modes(
            runtime,
            arguments,
            &missing,
            by_ref_indices,
            by_ref_indices_len,
            has_by_ref_variadic,
            by_ref_variadic_index,
            function_name,
            line
        );
        return;
    }

    ptn_call_arguments_unpack_array_with_parameter_modes(
        runtime,
        arguments,
        slot,
        by_ref_indices,
        by_ref_indices_len,
        has_by_ref_variadic,
        by_ref_variadic_index,
        function_name,
        line
    );
}

static PTN_UNUSED void ptn_call_arguments_destroy(PtnCallArguments *arguments) {
    for (size_t i = 0; i < arguments->len; i++) {
        ptn_value_destroy(&arguments->values[i]);
    }
    free(arguments->values);
    arguments->values = NULL;
    arguments->len = 0;
    arguments->capacity = 0;
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
    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot != NULL) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            PtnValue key = key_value == NULL ? ptn_null() : *key_value;
            PtnValue value = ptn_arrayaccess_read(runtime, slot_value, key, line);
            if (value.type == PTN_REFERENCE) {
                return value;
            }
            return ptn_reference_value(ptn_reference_new_owned(value));
        }
    }

    PtnArray *array = ptn_runtime_array_for_reference_write(runtime, name, path, line);
    if (array == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    if (key_value == NULL && !ptn_array_append_key_available(runtime, array)) {
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

    if (key_value == NULL && !ptn_array_append_key_available(runtime, array)) {
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
    if (key_value == NULL && !ptn_array_append_key_available(runtime, array)) {
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
    iterator.generator = NULL;
    iterator.runtime = NULL;
    iterator.access_scope = NULL;
    iterator.iterator_object = ptn_null();
    iterator.index = 0;
    iterator.length = 0;
    iterator.current_key = ptn_array_int_key(0);
    iterator.watched_slot = NULL;
    iterator.line = 0;
    iterator.has_current_key = 0;
    iterator.has_iterator_object = 0;
    iterator.protocol_iterator = 0;
    iterator.valid = 0;
    iterator.live = 0;
    return iterator;
}

static PTN_UNUSED void ptn_array_iterator_clear_current_key(PtnArrayIterator *iterator) {
    if (!iterator->has_current_key) {
        return;
    }
    ptn_array_key_free(iterator->current_key);
    iterator->current_key = ptn_array_int_key(0);
    iterator->has_current_key = 0;
}

static PTN_UNUSED void ptn_array_iterator_remember_current_key(PtnArrayIterator *iterator) {
    ptn_array_iterator_clear_current_key(iterator);
    if (
        iterator->array == NULL ||
        !iterator->valid ||
        iterator->index >= iterator->array->len
    ) {
        return;
    }
    iterator->current_key = ptn_array_key_clone(iterator->array->entries[iterator->index].key);
    iterator->has_current_key = 1;
}

static PTN_UNUSED int ptn_object_implements_builtin_interface(PtnObject *object, const char *interface_name) {
    return object != NULL &&
        (ptn_declared_class_implements_interface(object->class_name, interface_name) ||
         ptn_builtin_class_implements_interface(object->class_name, interface_name));
}

static PTN_UNUSED int ptn_object_has_iterator_method(
    PtnRuntime *runtime,
    PtnObject *object,
    const char *method_name
) {
    if (runtime == NULL || runtime->method_dispatch == NULL || object == NULL) {
        return 0;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (
        ptn_internal_class_exists_name(object->class_name) &&
        ptn_internal_class_method_exists(object->class_name, method_name)
    ) {
        return 1;
    }
#endif
    return runtime->declared_method_exists != NULL &&
        runtime->declared_method_exists(object->class_name, method_name);
}

static PTN_UNUSED PtnValue ptn_protocol_iterator_call(
    PtnArrayIterator *iterator,
    const char *method_name
) {
    if (
        iterator->runtime == NULL ||
        iterator->runtime->method_dispatch == NULL ||
        !iterator->has_iterator_object
    ) {
        return ptn_null();
    }
    return iterator->runtime->method_dispatch(
        iterator->runtime,
        iterator->iterator_object,
        method_name,
        0,
        NULL,
        iterator->line
    );
}

static PTN_UNUSED void ptn_protocol_iterator_refresh_valid(PtnArrayIterator *iterator) {
    PtnValue valid = ptn_protocol_iterator_call(iterator, "valid");
    iterator->valid = ptn_is_truthy(valid);
    ptn_value_destroy(&valid);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_array_snapshot(PtnArray *array) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (array == NULL) {
        return iterator;
    }
    iterator.array = array;
    iterator.length = array->len;
    ptn_array_retain(array);
    iterator.valid = iterator.length != 0;
    ptn_array_iterator_remember_current_key(&iterator);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_generator(
    PtnRuntime *runtime,
    PtnObject *object,
    int by_ref,
    const char *path,
    size_t line
) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (!ptn_object_is_generator(object)) {
        return iterator;
    }
    PtnGenerator *generator = (PtnGenerator *)object->native_data;
    if (by_ref && !generator->yields_by_ref) {
        ptn_throw_exception_at(
            runtime,
            "Exception",
            "You can only iterate a generator by-reference if it declared that it yields by-reference",
            path,
            line
        );
        return iterator;
    }
    iterator.array = generator->values;
    iterator.object = object;
    iterator.generator = generator;
    iterator.runtime = runtime;
    iterator.valid = iterator.array != NULL && iterator.array->len != 0;
    iterator.live = 1;
    ptn_array_iterator_retain(iterator.array);
    ptn_array_iterator_remember_current_key(&iterator);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_protocol_iterator(
    PtnRuntime *runtime,
    PtnValue iterator_value,
    const char *access_scope,
    size_t line
) {
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    iterator.runtime = runtime;
    iterator.access_scope = access_scope;
    iterator.iterator_object = ptn_value_clone_deref(iterator_value);
    iterator.has_iterator_object = 1;
    iterator.protocol_iterator = 1;
    iterator.line = line;

    if (ptn_object_has_iterator_method(runtime, iterator.iterator_object.as.object, "rewind")) {
        PtnValue rewind = ptn_protocol_iterator_call(&iterator, "rewind");
        ptn_value_destroy(&rewind);
    }
    ptn_protocol_iterator_refresh_valid(&iterator);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_traversable_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line,
    size_t depth
);

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
    ptn_array_iterator_remember_current_key(&iterator);
    return iterator;
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_traversable_object(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line,
    size_t depth
) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT) {
        return ptn_array_iterator_empty();
    }
    if (ptn_object_is_generator(value.as.object)) {
        return ptn_array_iterator_from_generator(runtime, value.as.object, 0, path, line);
    }

    if (
        ptn_object_implements_builtin_interface(value.as.object, "IteratorAggregate") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "getIterator")
    ) {
        if (depth > 16) {
            ptn_throw_exception(runtime, "Exception", "IteratorAggregate recursion limit exceeded");
            return ptn_array_iterator_empty();
        }
        PtnValue result = runtime->method_dispatch(runtime, value, "getIterator", 0, NULL, line);
        PtnValue resolved = ptn_value_deref(result);
        PtnArrayIterator iterator = ptn_array_iterator_empty();
        if (resolved.type == PTN_ARRAY) {
            iterator = ptn_array_iterator_from_array_snapshot(resolved.as.array);
        } else if (resolved.type == PTN_OBJECT && ptn_object_is_generator(resolved.as.object)) {
            iterator = ptn_array_iterator_from_generator(runtime, resolved.as.object, 0, path, line);
        } else if (
            resolved.type == PTN_OBJECT &&
            (
                ptn_object_implements_builtin_interface(resolved.as.object, "Iterator") ||
                ptn_object_implements_builtin_interface(resolved.as.object, "IteratorAggregate")
            )
        ) {
            iterator = ptn_array_iterator_from_traversable_object(
                runtime,
                resolved,
                access_scope,
                path,
                line,
                depth + 1
            );
        } else {
            ptn_emit_foreach_non_array_warning(resolved, path, line);
        }
        ptn_value_destroy(&result);
        return iterator;
    }

    if (
        ptn_object_implements_builtin_interface(value.as.object, "Iterator") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "valid") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "current") &&
        ptn_object_has_iterator_method(runtime, value.as.object, "next")
    ) {
        return ptn_array_iterator_from_protocol_iterator(runtime, value, access_scope, line);
    }

    return ptn_array_iterator_from_object_properties(runtime, value.as.object, access_scope);
}

static PTN_UNUSED PtnArrayIterator ptn_array_iterator_from_value(
    PtnRuntime *runtime,
    PtnValue value,
    const char *access_scope,
    const char *path,
    size_t line
) {
    value = ptn_value_deref(value);
    PtnArrayIterator iterator = ptn_array_iterator_empty();
    if (value.type != PTN_ARRAY) {
        if (value.type == PTN_OBJECT) {
            if (ptn_object_is_generator(value.as.object)) {
                return ptn_array_iterator_from_generator(
                    runtime,
                    value.as.object,
                    0,
                    path,
                    line
                );
            }
            return ptn_array_iterator_from_traversable_object(runtime, value, access_scope, path, line, 0);
        }
        ptn_emit_foreach_non_array_warning(value, path, line);
        return iterator;
    }
    return ptn_array_iterator_from_array_snapshot(value.as.array);
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
        if (ptn_object_is_generator(value->as.object)) {
            return ptn_array_iterator_from_generator(
                runtime,
                value->as.object,
                1,
                path,
                line
            );
        }
        return ptn_array_iterator_from_traversable_object(runtime, *value, access_scope, path, line, 0);
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
    ptn_array_iterator_remember_current_key(&iterator);
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
    PtnArrayIterator iterator = ptn_array_iterator_by_ref_from_slot(runtime, slot, access_scope, path, line);
    if (iterator.object == NULL && iterator.array != NULL) {
        iterator.watched_slot = slot;
    }
    return iterator;
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
    if (iterator->protocol_iterator) {
        return ptn_protocol_iterator_call(iterator, "key");
    }
    PtnArrayKey key = iterator->array->entries[iterator->index].key;
    if (iterator->object != NULL && iterator->generator == NULL) {
        return ptn_object_foreach_key_value(iterator->object, key);
    }
    if (key.type == PTN_ARRAY_KEY_INT) {
        return ptn_int(key.as.integer);
    }
    return ptn_owned_string_len(ptn_duplicate_string_len(key.as.string, key.string_len), key.string_len);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_value(PtnArrayIterator *iterator) {
    if (iterator->protocol_iterator) {
        return ptn_protocol_iterator_call(iterator, "current");
    }
    return ptn_value_borrow(iterator->array->entries[iterator->index].value);
}

static PTN_UNUSED PtnValue ptn_array_iterator_current_reference(PtnArrayIterator *iterator) {
    if (iterator->protocol_iterator) {
        PtnValue current = ptn_protocol_iterator_call(iterator, "current");
        if (current.type == PTN_REFERENCE) {
            return current;
        }
        return ptn_reference_value(ptn_reference_new_owned(current));
    }
    PtnArrayEntry *entry = &iterator->array->entries[iterator->index];
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_array_iterator_release(PtnArray *array);

static PTN_UNUSED PtnArray *ptn_array_iterator_watched_slot_array(PtnArrayIterator *iterator) {
    if (iterator->watched_slot == NULL) {
        return NULL;
    }
    PtnValue *value = iterator->watched_slot->type == PTN_REFERENCE
        ? &iterator->watched_slot->as.reference->value
        : iterator->watched_slot;
    if (value->type != PTN_ARRAY) {
        return NULL;
    }
    return value->as.array;
}

static PTN_UNUSED void ptn_array_iterator_refresh_watched_array(PtnArrayIterator *iterator) {
    if (!iterator->live || iterator->watched_slot == NULL) {
        return;
    }
    PtnArray *array = ptn_array_iterator_watched_slot_array(iterator);
    if (array == NULL || array == iterator->array) {
        return;
    }
    ptn_array_iterator_retain(array);
    if (iterator->array != NULL) {
        ptn_array_iterator_release(iterator->array);
    }
    iterator->array = array;
    iterator->object = NULL;
}

static PTN_UNUSED void ptn_array_iterator_advance(PtnArrayIterator *iterator) {
    if (iterator->protocol_iterator) {
        PtnValue next = ptn_protocol_iterator_call(iterator, "next");
        ptn_value_destroy(&next);
        ptn_protocol_iterator_refresh_valid(iterator);
        return;
    }
    if (iterator->array == NULL) {
        iterator->valid = 0;
        ptn_array_iterator_clear_current_key(iterator);
        return;
    }

    size_t next_index = iterator->index + 1;
    if (iterator->has_current_key) {
        size_t current_index = ptn_array_find_key(iterator->array, iterator->current_key);
        if (current_index < iterator->array->len) {
            next_index = current_index + 1;
        } else {
            next_index = iterator->index;
        }
    }

    ptn_array_iterator_refresh_watched_array(iterator);
    ptn_array_iterator_clear_current_key(iterator);
    size_t limit = iterator->live ? iterator->array->len : iterator->length;
    if (limit > iterator->array->len) {
        limit = iterator->array->len;
    }
    iterator->index = next_index;
    iterator->valid = iterator->index < limit;
    ptn_array_iterator_skip_invisible_object_properties(iterator);
    ptn_array_iterator_remember_current_key(iterator);
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
    ptn_array_iterator_clear_current_key(iterator);
    if (iterator->array != NULL) {
        if (iterator->live) {
            ptn_array_iterator_release(iterator->array);
        } else {
            ptn_array_free(iterator->array);
        }
        iterator->array = NULL;
    }
    iterator->object = NULL;
    iterator->generator = NULL;
    iterator->runtime = NULL;
    iterator->access_scope = NULL;
    if (iterator->has_iterator_object) {
        ptn_value_destroy(&iterator->iterator_object);
        iterator->iterator_object = ptn_null();
        iterator->has_iterator_object = 0;
    }
    iterator->watched_slot = NULL;
    iterator->index = 0;
    iterator->length = 0;
    iterator->line = 0;
    iterator->valid = 0;
    iterator->protocol_iterator = 0;
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

static PTN_UNUSED int ptn_arrayaccess_can_dispatch(
    PtnRuntime *runtime,
    PtnValue container,
    const char *method_name
) {
    container = ptn_value_deref(container);
    if (container.type != PTN_OBJECT || runtime->method_dispatch == NULL) {
        return 0;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (
        ptn_internal_class_exists_name(container.as.object->class_name) &&
        ptn_internal_class_method_exists(container.as.object->class_name, method_name)
    ) {
        return 1;
    }
#endif
    return runtime->declared_method_exists != NULL &&
        runtime->declared_method_exists(container.as.object->class_name, method_name);
}

static PTN_UNUSED PtnValue ptn_arrayaccess_call(
    PtnRuntime *runtime,
    PtnValue container,
    const char *method_name,
    size_t argc,
    PtnValue *args,
    size_t line
) {
    return runtime->method_dispatch(runtime, ptn_value_deref(container), method_name, argc, args, line);
}

static PTN_UNUSED PtnValue ptn_arrayaccess_read(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    PtnValue args[1] = { ptn_value_clone_deref(key_value) };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetGet", 1, args, line);
    ptn_value_destroy(&args[0]);
    return result;
}

static PTN_UNUSED void ptn_arrayaccess_write(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    PtnValue value,
    size_t line
) {
    PtnValue args[2] = {
        ptn_value_clone_deref(key_value),
        ptn_value_clone_deref(value)
    };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetSet", 2, args, line);
    ptn_value_destroy(&result);
    ptn_value_destroy(&args[0]);
    ptn_value_destroy(&args[1]);
}

static PTN_UNUSED int ptn_arrayaccess_exists(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    PtnValue args[1] = { ptn_value_clone_deref(key_value) };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetExists", 1, args, line);
    int exists = ptn_is_truthy(ptn_value_deref(result));
    ptn_value_destroy(&result);
    ptn_value_destroy(&args[0]);
    return exists;
}

static PTN_UNUSED void ptn_arrayaccess_unset(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    PtnValue args[1] = { ptn_value_clone_deref(key_value) };
    PtnValue result = ptn_arrayaccess_call(runtime, container, "offsetUnset", 1, args, line);
    ptn_value_destroy(&result);
    ptn_value_destroy(&args[0]);
}

static PTN_UNUSED PtnLookupResult ptn_offset_lookup(PtnRuntime *runtime, PtnValue container, PtnValue key_value, size_t line, int quiet) {
    container = ptn_value_deref(container);
    key_value = ptn_value_deref(key_value);
    if (container.type == PTN_STRING) {
        return ptn_string_offset_lookup(runtime, container, key_value, line, quiet);
    }

    if (quiet && ptn_arrayaccess_can_dispatch(runtime, container, "offsetExists") &&
        !ptn_arrayaccess_exists(runtime, container, key_value, line)) {
        return ptn_lookup_missing();
    }

    if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetGet")) {
        return ptn_lookup_found(ptn_arrayaccess_read(runtime, container, key_value, line));
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
            ptn_emit_array_runtime_warning(runtime, message, line);
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

static PTN_UNUSED PtnValue ptn_array_read_for_list_destructure(
    PtnRuntime *runtime,
    PtnValue container,
    PtnValue key_value,
    size_t line
) {
    container = ptn_value_deref(container);
    key_value = ptn_value_deref(key_value);
    if (container.type == PTN_NULL) {
        return ptn_null();
    }
    if (container.type != PTN_ARRAY) {
        char message[128];
        int written = snprintf(
            message,
            sizeof(message),
            "Cannot use %s as array",
            ptn_offset_container_type_name(container)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_array_runtime_diagnostic("Warning", message, line);
        return ptn_null();
    }

    PtnArrayKey key = ptn_array_key_from_value(key_value);
    PtnArrayEntry *entry = ptn_array_entry_for_key(container.as.array, key);
    if (entry == NULL) {
        ptn_emit_undefined_array_key_warning(runtime, key, line);
        ptn_array_key_free(key);
        return ptn_null();
    }
    PtnValue value = ptn_value_clone_deref(entry->value);
    ptn_array_key_free(key);
    return value;
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

    if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetExists")) {
        return ptn_arrayaccess_exists(runtime, container, key_value, line);
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

    if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetExists")) {
        if (!ptn_arrayaccess_exists(runtime, container, key_value, line)) {
            return 1;
        }
        if (ptn_arrayaccess_can_dispatch(runtime, container, "offsetGet")) {
            PtnValue value = ptn_arrayaccess_read(runtime, container, key_value, line);
            int result = !ptn_is_truthy(ptn_value_deref(value));
            ptn_value_destroy(&value);
            return result;
        }
        return 0;
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
    PtnRuntime *runtime,
    PtnArray *array,
    const PtnArrayPathSegment *segment
) {
    if (segment->append) {
        if (!ptn_array_append_key_available(runtime, array)) {
            return ptn_array_int_key(0);
        }
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
    PtnArrayKey key = ptn_array_path_segment_key(runtime, array, segment);
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

static PTN_UNUSED PtnValue ptn_value_reference_for_array_path(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
);

static PTN_UNUSED void ptn_value_array_path_set_impl(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
);

static PTN_UNUSED PtnValue ptn_value_reference_for_array_path(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
);

static PTN_UNUSED PtnValue ptn_runtime_reference_for_array_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
) {
    if (ptn_runtime_is_globals_name(name)) {
        if (segment_count == 0) {
            return ptn_runtime_reference_for_variable(runtime, name);
        }

        char *global_name = ptn_runtime_global_name_from_segment(&segments[0]);
        if (global_name == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        PtnValue *slot = ptn_runtime_global_variable_slot_for_write(runtime, global_name);
        free(global_name);
        if (slot == NULL) {
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
        return ptn_value_reference_for_array_path(
            runtime,
            slot,
            segments + 1,
            segment_count - 1,
            path,
            line
        );
    }

    if (segment_count == 0) {
        return ptn_runtime_reference_for_variable(runtime, name);
    }

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, name);
    if (slot != NULL) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            PtnValue value = ptn_arrayaccess_read(runtime, slot_value, key, line);
            if (segment_count == 1 && value.type == PTN_REFERENCE) {
                return value;
            }
            if (segment_count == 1) {
                return ptn_reference_value(ptn_reference_new_owned(value));
            }
            if (value.type == PTN_REFERENCE) {
                PtnValue nested_reference = ptn_value_reference_for_array_path(
                    runtime,
                    &value,
                    segments + 1,
                    segment_count - 1,
                    path,
                    line
                );
                ptn_value_destroy(&value);
                return nested_reference;
            }
            ptn_value_destroy(&value);
            return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
        }
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
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
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
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return;
    }
    PtnArrayKey key = leaf->append
        ? ptn_array_int_key(array->next_auto_key)
        : ptn_array_key_from_value(leaf->value);
    ptn_array_set_entry(array, key, ptn_value_clone(reference));
}

static PTN_UNUSED PtnValue ptn_value_reference_for_array_path(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    const char *path,
    size_t line
) {
    if (target == NULL) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (segment_count == 0) {
        if (target->type != PTN_REFERENCE) {
            PtnValue current = *target;
            *target = ptn_reference_value(ptn_reference_new_owned(current));
        }
        return ptn_value_clone(*target);
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetGet")) {
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue value = ptn_arrayaccess_read(runtime, *target_value, key, line);
        if (segment_count == 1 && value.type == PTN_REFERENCE) {
            return value;
        }
        if (segment_count == 1) {
            return ptn_reference_value(ptn_reference_new_owned(value));
        }
        if (value.type == PTN_REFERENCE) {
            PtnValue nested_reference = ptn_value_reference_for_array_path(
                runtime,
                &value,
                segments + 1,
                segment_count - 1,
                path,
                line
            );
            ptn_value_destroy(&value);
            return nested_reference;
        }
        ptn_value_destroy(&value);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    PtnArray *array = NULL;
    if (target_value->type == PTN_ARRAY) {
        array = ptn_array_detach_value(target_value);
    } else if (target_value->type == PTN_STRING) {
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    } else if ((array = ptn_array_convertible_scalar_for_write(target_value, line)) != NULL) {
        /* false/null conversion handled by shared lvalue write semantics. */
    } else {
        ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
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
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    PtnArrayEntry *entry = ptn_array_reference_entry(array, leaf->append ? NULL : &leaf->value);
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }
    return ptn_value_clone(entry->value);
}

static PTN_UNUSED void ptn_value_bind_array_path_reference(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue reference,
    const char *path,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    if (target == NULL) {
        return;
    }
    if (segment_count == 0) {
        ptn_value_destroy(target);
        *target = ptn_value_clone(reference);
        return;
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    PtnArray *array = NULL;
    if (target_value->type == PTN_ARRAY) {
        array = ptn_array_detach_value(target_value);
    } else if (target_value->type == PTN_STRING) {
        ptn_throw_exception_at(runtime, "Error", "Cannot create references to/from string offsets", path, line);
        return;
    } else if ((array = ptn_array_convertible_scalar_for_write(target_value, line)) != NULL) {
        /* false/null conversion handled by shared lvalue write semantics. */
    } else {
        ptn_throw_exception(runtime, "Error", "Cannot use a scalar value as an array");
        return;
    }
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
    if (leaf->append && !ptn_array_append_key_available(runtime, array)) {
        return;
    }
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
    PtnArrayKey key = ptn_array_path_segment_key(runtime, array, segment);
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
    PtnArrayKey key = ptn_array_path_segment_key(runtime, array, segment);
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
    if (slot != NULL && segment_count == 1) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetSet")) {
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            ptn_arrayaccess_write(runtime, slot_value, key, value, line);
            return;
        }
    }
    if (slot != NULL && segment_count > 1) {
        PtnValue slot_value = ptn_value_deref(*slot);
        if (ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
            PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
            PtnValue nested = ptn_arrayaccess_read(runtime, slot_value, key, line);
            ptn_value_array_path_set_impl(
                runtime,
                &nested,
                segments + 1,
                segment_count - 1,
                value,
                line,
                emit_null_key_deprecation
            );
            ptn_value_destroy(&nested);
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
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        return ptn_arrayaccess_read(runtime, slot_value, key, line);
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

static PTN_UNUSED void ptn_value_array_path_set_impl(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line,
    int emit_null_key_deprecation
) {
    if (target == NULL || segment_count == 0) {
        return;
    }

    PtnValue *target_value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (segment_count == 1 && target_value->type == PTN_STRING) {
        if (segments[0].append) {
            ptn_throw_exception(runtime, "Error", "[] operator not supported for strings");
            return;
        }
        ptn_runtime_string_offset_set(runtime, target_value, segments[0].value, value, line);
        return;
    }
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetSet")) {
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        ptn_arrayaccess_write(runtime, *target_value, key, value, line);
        return;
    }
    if (segment_count > 1 && ptn_arrayaccess_can_dispatch(runtime, *target_value, "offsetGet")) {
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        PtnValue nested = ptn_arrayaccess_read(runtime, *target_value, key, line);
        ptn_value_array_path_set_impl(
            runtime,
            &nested,
            segments + 1,
            segment_count - 1,
            value,
            line,
            emit_null_key_deprecation
        );
        ptn_value_destroy(&nested);
        return;
    }

    PtnArray *array = ptn_array_root_slot_for_write(runtime, target, line);
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

static PTN_UNUSED void ptn_value_array_path_set(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    ptn_value_array_path_set_impl(runtime, target, segments, segment_count, value, line, 1);
}

static PTN_UNUSED void ptn_value_array_path_set_from_assign_op(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    PtnValue value,
    size_t line
) {
    ptn_value_array_path_set_impl(runtime, target, segments, segment_count, value, line, 0);
}

static PTN_UNUSED PtnValue ptn_value_array_path_read_for_assign_op(
    PtnRuntime *runtime,
    PtnValue target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (segment_count == 0) {
        return ptn_null();
    }

    PtnValue slot_value = ptn_value_deref(target);
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
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, slot_value, "offsetGet")) {
        PtnValue key = segments[0].append ? ptn_null() : segments[0].value;
        return ptn_arrayaccess_read(runtime, slot_value, key, line);
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

static PTN_UNUSED void ptn_value_array_path_unset(
    PtnRuntime *runtime,
    PtnValue *target,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    if (target == NULL || segment_count == 0) {
        return;
    }

    PtnValue *value = target->type == PTN_REFERENCE ? &target->as.reference->value : target;
    if (value->type == PTN_STRING) {
        ptn_throw_exception(runtime, "Error", "Cannot unset string offsets");
        return;
    }
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetUnset")) {
        if (segments[0].append) {
            return;
        }
        ptn_arrayaccess_unset(runtime, *value, segments[0].value, line);
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
    if (segment_count == 1 && ptn_arrayaccess_can_dispatch(runtime, *value, "offsetUnset")) {
        if (segments[0].append) {
            return;
        }
        ptn_arrayaccess_unset(runtime, *value, segments[0].value, line);
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
    PtnValue removed = ptn_value_clone_deref(array->entries[removed_index].value);
    ptn_value_destroy(&array->entries[removed_index].value);
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

    PtnValue removed = ptn_value_clone_deref(array->entries[0].value);
    ptn_value_destroy(&array->entries[0].value);
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
    PtnValue result = ptn_value_clone_deref(removed);
    ptn_value_destroy(&removed);
    return result;
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

static int ptn_array_value_compare_numeric(PtnValue left, PtnValue right) {
    PtnNumber left_number = ptn_to_number(left);
    PtnNumber right_number = ptn_to_number(right);
    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        if (left_number.integer < right_number.integer) {
            return -1;
        }
        if (left_number.integer > right_number.integer) {
            return 1;
        }
        return 0;
    }
    if (left_number.floating < right_number.floating) {
        return -1;
    }
    if (left_number.floating > right_number.floating) {
        return 1;
    }
    return 0;
}

static int ptn_array_value_compare_string(PtnValue left, PtnValue right) {
    PtnStringOperand left_string = ptn_value_to_string_operand(left);
    PtnStringOperand right_string = ptn_value_to_string_operand(right);
    int compared = ptn_compare_string_bytes(
        (const unsigned char *)left_string.data,
        left_string.len,
        (const unsigned char *)right_string.data,
        right_string.len
    );
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_GREATER) {
        return 1;
    }
    return 0;
}

static int ptn_array_value_compare_string_case(PtnValue left, PtnValue right) {
    PtnStringOperand left_string = ptn_value_to_string_operand(left);
    PtnStringOperand right_string = ptn_value_to_string_operand(right);
    const unsigned char *left_bytes = (const unsigned char *)left_string.data;
    const unsigned char *right_bytes = (const unsigned char *)right_string.data;
    size_t shared_len = left_string.len < right_string.len ? left_string.len : right_string.len;
    int compared = 0;
    for (size_t i = 0; i < shared_len; i++) {
        unsigned char left_byte = left_bytes[i];
        unsigned char right_byte = right_bytes[i];
        if (left_byte >= (unsigned char)'A' && left_byte <= (unsigned char)'Z') {
            left_byte = (unsigned char)(left_byte + ((unsigned char)'a' - (unsigned char)'A'));
        }
        if (right_byte >= (unsigned char)'A' && right_byte <= (unsigned char)'Z') {
            right_byte = (unsigned char)(right_byte + ((unsigned char)'a' - (unsigned char)'A'));
        }
        if (left_byte < right_byte) {
            compared = -1;
            break;
        }
        if (left_byte > right_byte) {
            compared = 1;
            break;
        }
    }
    if (compared == 0) {
        if (left_string.len < right_string.len) {
            compared = -1;
        } else if (left_string.len > right_string.len) {
            compared = 1;
        }
    }
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return compared;
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

static int ptn_array_sort_flags_case_insensitive(int64_t flags) {
    return flags == (PTN_SORT_STRING | PTN_SORT_FLAG_CASE) ||
        flags == (PTN_SORT_NATURAL | PTN_SORT_FLAG_CASE);
}

static int64_t ptn_array_sort_flags_base(int64_t flags) {
    return ptn_array_sort_flags_case_insensitive(flags)
        ? (flags & ~PTN_SORT_FLAG_CASE)
        : flags;
}

static int ptn_array_value_compare_by_sort_flags(PtnValue left, PtnValue right, int64_t flags) {
    int case_insensitive = ptn_array_sort_flags_case_insensitive(flags);
    switch (ptn_array_sort_flags_base(flags)) {
        case PTN_SORT_NUMERIC:
            return ptn_array_value_compare_numeric(left, right);
        case PTN_SORT_STRING:
        case PTN_SORT_LOCALE_STRING:
            return case_insensitive
                ? ptn_array_value_compare_string_case(left, right)
                : ptn_array_value_compare_string(left, right);
        case PTN_SORT_NATURAL:
            return case_insensitive
                ? ptn_array_value_compare_natural_case(left, right)
                : ptn_array_value_compare_natural(left, right);
        case PTN_SORT_REGULAR:
        default:
            return ptn_array_value_compare_ascending(left, right);
    }
}

static int ptn_array_key_compare_by_sort_flags(PtnArrayKey left, PtnArrayKey right, int64_t flags) {
    if (ptn_array_sort_flags_base(flags) == PTN_SORT_REGULAR) {
        return ptn_array_key_compare_ascending(left, right);
    }
    PtnValue left_value = ptn_array_key_value(left);
    PtnValue right_value = ptn_array_key_value(right);
    int compared = ptn_array_value_compare_by_sort_flags(left_value, right_value, flags);
    ptn_value_destroy(&left_value);
    ptn_value_destroy(&right_value);
    return compared;
}

static void ptn_array_reindex_after_sort(PtnArray *array) {
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_key_free(array->entries[i].key);
        array->entries[i].key = ptn_array_int_key((int64_t)i);
    }
    ptn_array_recompute_next_auto_key(array);
}

static void ptn_array_sort_entries_by_flags(PtnArray *array, int compare_keys, int descending, int reindex, int64_t flags) {
    for (size_t i = 1; i < array->len; i++) {
        PtnArrayEntry moving = array->entries[i];
        size_t j = i;
        while (j > 0) {
            int compared = compare_keys
                ? ptn_array_key_compare_by_sort_flags(array->entries[j - 1].key, moving.key, flags)
                : ptn_array_value_compare_by_sort_flags(array->entries[j - 1].value, moving.value, flags);
            if (descending) {
                compared = -compared;
            }
            if (compared <= 0) {
                break;
            }
            array->entries[j] = array->entries[j - 1];
            j--;
        }
        array->entries[j] = moving;
    }
    if (reindex) {
        ptn_array_reindex_after_sort(array);
    }
    array->current_index = 0;
    ptn_array_rebuild_index(array);
}

static PTN_UNUSED void ptn_array_sort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 0, 1, flags);
}

static PTN_UNUSED void ptn_array_rsort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 1, 1, flags);
}

static PTN_UNUSED void ptn_array_asort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 0, 0, flags);
}

static PTN_UNUSED void ptn_array_arsort_values_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 0, 1, 0, flags);
}

static PTN_UNUSED void ptn_array_ksort_entries_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 1, 0, 0, flags);
}

static PTN_UNUSED void ptn_array_krsort_entries_with_flags(PtnArray *array, int64_t flags) {
    ptn_array_sort_entries_by_flags(array, 1, 1, 0, flags);
}

static PTN_UNUSED void ptn_array_sort_values(PtnArray *array) {
    ptn_array_sort_values_with_flags(array, PTN_SORT_REGULAR);
}

static PTN_UNUSED void ptn_array_rsort_values(PtnArray *array) {
    ptn_array_rsort_values_with_flags(array, PTN_SORT_REGULAR);
}

static PTN_UNUSED void ptn_array_asort_values(PtnArray *array) {
    ptn_array_asort_values_with_flags(array, PTN_SORT_REGULAR);
}

static PTN_UNUSED void ptn_array_arsort_values(PtnArray *array) {
    ptn_array_arsort_values_with_flags(array, PTN_SORT_REGULAR);
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

static PTN_UNUSED int64_t ptn_array_push_values(PtnRuntime *runtime, PtnArray *array, size_t argc, const PtnValue *values) {
    if (argc > SIZE_MAX - array->len) {
        ptn_abort_out_of_memory();
    }
    if (array->len + argc > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }

    for (size_t i = 0; i < argc; i++) {
        if (array->next_auto_key >= INT64_MAX) {
            goto unavailable;
        }
        for (size_t j = 0; j < array->len; j++) {
            if (array->entries[j].key.type == PTN_ARRAY_KEY_INT &&
                array->entries[j].key.as.integer == INT64_MAX) {
                goto unavailable;
            }
        }
        PtnArrayKey key = ptn_array_int_key(array->next_auto_key);
        ptn_array_set_entry(array, key, ptn_value_clone(values[i]));
    }

    array->current_index = 0;
    ptn_array_recompute_next_auto_key(array);
    ptn_array_rebuild_index(array);
    return (int64_t)array->len;

unavailable:
    ptn_throw_exception(
        runtime,
        "Error",
        "Cannot add element to the array as the next element is already occupied"
    );
    return (int64_t)array->len;
}
