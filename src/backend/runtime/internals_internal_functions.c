    ptn_runtime_define_constant(runtime, name, value);
    return 1;
}

static PTN_UNUSED PtnValue ptn_read_constant(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_runtime_constant_value(runtime, name, &value)) {
        return value;
    }
    ptn_emit_undefined_constant_error(&runtime->diagnostics, name);
    exit(255);
    return ptn_null();
}

static PTN_UNUSED void ptn_echo(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            break;
        case PTN_BOOL:
            if (value.as.boolean) {
                fputs("1", stdout);
            }
            break;
        case PTN_INT:
            printf("%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[128];
            ptn_format_scalar_float(value.as.floating, formatted, sizeof(formatted));
            fputs(formatted, stdout);
            break;
        }
        case PTN_STRING:
            fwrite(value.as.string.data, 1, value.as.string.len, stdout);
            break;
        case PTN_ARRAY:
            fputs("Array", stdout);
            break;
        case PTN_OBJECT: {
            PtnStringOperand object_string;
            if (ptn_try_object_to_string_operand(runtime, value, line, &object_string)) {
                fwrite(object_string.data, 1, object_string.len, stdout);
                ptn_string_operand_free(object_string);
                break;
            }
            fputs("Object", stdout);
            break;
        }
        case PTN_CLOSURE:
            fputs("Object", stdout);
            break;
        case PTN_EXCEPTION:
            fputs("Object", stdout);
            break;
        case PTN_RESOURCE:
            printf("Resource id #%lld", (long long)value.as.resource->id);
            break;
        case PTN_REFERENCE:
            break;
    }
}

/* PTN_DIRECT_INTERNAL_HELPERS_START */
static PTN_UNUSED const char *ptn_count_operand_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_BOOL) {
        return value.as.boolean ? "true" : "false";
    }
    return ptn_offset_container_type_name(value);
}

typedef struct {
    PtnArray **items;
    size_t len;
    size_t capacity;
} PtnCountSeenArrays;

static PTN_UNUSED void ptn_count_seen_arrays_init(PtnCountSeenArrays *seen) {
    seen->items = NULL;
    seen->len = 0;
    seen->capacity = 0;
}

static PTN_UNUSED void ptn_count_seen_arrays_free(PtnCountSeenArrays *seen) {
    free(seen->items);
    seen->items = NULL;
    seen->len = 0;
    seen->capacity = 0;
}

static PTN_UNUSED int ptn_count_seen_arrays_contains(PtnCountSeenArrays *seen, PtnArray *array) {
    for (size_t i = 0; i < seen->len; i++) {
        if (seen->items[i] == array) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_count_seen_arrays_push(PtnCountSeenArrays *seen, PtnArray *array) {
    if (seen->len == seen->capacity) {
        size_t new_capacity = seen->capacity == 0 ? 8 : seen->capacity * 2;
        if (new_capacity < seen->capacity || new_capacity > SIZE_MAX / sizeof(PtnArray *)) {
            ptn_abort_out_of_memory();
        }
        PtnArray **new_items = realloc(seen->items, new_capacity * sizeof(PtnArray *));
        if (new_items == NULL) {
            ptn_abort_out_of_memory();
        }
        seen->items = new_items;
        seen->capacity = new_capacity;
    }
    seen->items[seen->len++] = array;
}

static PTN_UNUSED int64_t ptn_count_array_recursive(
    PtnRuntime *runtime,
    const char *function_name,
    PtnArray *array,
    PtnCountSeenArrays *seen,
    size_t line
) {
    if (ptn_count_seen_arrays_contains(seen, array)) {
        char message[96];
        int written = snprintf(message, sizeof(message), "%s(): Recursion detected", function_name);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_warning(&runtime->diagnostics, message, line);
        return 0;
    }

    ptn_count_seen_arrays_push(seen, array);
    if (array->len > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    int64_t count = (int64_t)array->len;
    for (size_t i = 0; i < array->len; i++) {
        PtnValue value = ptn_value_deref(array->entries[i].value);
        if (value.type != PTN_ARRAY) {
            continue;
        }
        int64_t nested = ptn_count_array_recursive(runtime, function_name, value.as.array, seen, line);
        if (nested > INT64_MAX - count) {
            ptn_abort_out_of_memory();
        }
        count += nested;
    }
    seen->len--;
    return count;
}

static PTN_UNUSED PtnValue ptn_count_value(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue value,
    int64_t mode,
    size_t line
) {
    if (mode != PTN_COUNT_NORMAL && mode != PTN_COUNT_RECURSIVE) {
        char message[128];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #2 ($mode) must be either COUNT_NORMAL or COUNT_RECURSIVE",
            function_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return ptn_null();
    }

    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        if (mode == PTN_COUNT_NORMAL) {
            return ptn_int((int64_t)value.as.array->len);
        }

        PtnCountSeenArrays seen;
        ptn_count_seen_arrays_init(&seen);
        int64_t count = ptn_count_array_recursive(runtime, function_name, value.as.array, &seen, line);
        ptn_count_seen_arrays_free(&seen);
        return ptn_int(count);
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #1 ($value) must be of type Countable|array, %s given",
        function_name,
        ptn_count_operand_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_array_key_exists_value(PtnRuntime *runtime, PtnValue key_value, PtnValue array_value, size_t line) {
    key_value = ptn_value_deref(key_value);
    array_value = ptn_value_deref(array_value);
    if (array_value.type != PTN_ARRAY) {
        const char *given = array_value.type == PTN_OBJECT
            ? array_value.as.object->class_name
            : ptn_offset_container_type_name(array_value);
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "array_key_exists(): Argument #2 ($array) must be of type array, %s given",
            given
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    if (key_value.type == PTN_NULL) {
        if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_DEPRECATED)) {
            fputc('\n', stdout);
            runtime->diagnostics.emitted_deprecation = 1;
            fputs(
                "Deprecated: Using null as the key parameter for array_key_exists() is deprecated, use an empty string instead in ptn on line ",
                stdout
            );
            fprintf(stdout, "%zu", line);
            fputc('\n', stdout);
        }
    }
    if (key_value.type == PTN_ARRAY || key_value.type == PTN_OBJECT || key_value.type == PTN_CLOSURE || key_value.type == PTN_EXCEPTION) {
        const char *type_name = key_value.type == PTN_OBJECT
            ? key_value.as.object->class_name
            : ptn_offset_container_type_name(key_value);
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "Cannot access offset of type %s on array",
            type_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    if (key_value.type == PTN_RESOURCE) {
        ptn_emit_resource_offset_warning(runtime, key_value.as.resource, line);
    }
    PtnArrayKey key = ptn_array_key_from_value(key_value);
    int exists = ptn_array_entry_for_key(array_value.as.array, key) != NULL;
    ptn_array_key_free(key);
    return ptn_bool(exists);
}
/* PTN_DIRECT_INTERNAL_HELPERS_END */

/* PTN_INTERNAL_FUNCTIONS_START */
static PTN_UNUSED PtnValue ptn_call_function(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line);
static PTN_UNUSED PtnValue ptn_call_callable(PtnRuntime *runtime, PtnValue callable, size_t argc, const PtnValue *args, size_t line);
static int ptn_callable_is_valid(PtnValue callable, int syntax_only);

static char *ptn_invalid_callback_message(
    const char *function_name,
    size_t position,
    const char *parameter_name,
    PtnValue callback,
    int accepts_null
) {
    PtnValue resolved = ptn_value_deref(callback);
    char *reason = NULL;
    if (resolved.type == PTN_STRING) {
        char *name = ptn_value_to_string(resolved);
        int needed = snprintf(
            NULL,
            0,
            "function \"%s\" not found or invalid function name",
            name
        );
        if (needed < 0) {
            ptn_abort_out_of_memory();
        }
        reason = malloc((size_t)needed + 1);
        if (reason == NULL) {
            ptn_abort_out_of_memory();
        }
        snprintf(
            reason,
            (size_t)needed + 1,
            "function \"%s\" not found or invalid function name",
            name
        );
        free(name);
    } else if (resolved.type == PTN_ARRAY && resolved.as.array->len != 2) {
        reason = ptn_duplicate_string("array callback must have exactly two members");
    } else {
        char *name = ptn_callable_output_name(callback);
        int needed = snprintf(
            NULL,
            0,
            "callback \"%s\" not found or invalid function name",
            name
        );
        if (needed < 0) {
            ptn_abort_out_of_memory();
        }
        reason = malloc((size_t)needed + 1);
        if (reason == NULL) {
            ptn_abort_out_of_memory();
        }
        snprintf(
            reason,
            (size_t)needed + 1,
            "callback \"%s\" not found or invalid function name",
            name
        );
        free(name);
    }

    const char *callback_requirement = accepts_null
        ? "must be a valid callback or null"
        : "must be a valid callback";
    int named_parameter = parameter_name != NULL && parameter_name[0] != '\0';
    int needed = named_parameter
        ? snprintf(
            NULL,
            0,
            "%s(): Argument #%zu ($%s) %s, %s",
            function_name,
            position,
            parameter_name,
            callback_requirement,
            reason
        )
        : snprintf(
            NULL,
            0,
            "%s(): Argument #%zu %s, %s",
            function_name,
            position,
            callback_requirement,
            reason
        );
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (named_parameter) {
        snprintf(
            message,
            (size_t)needed + 1,
            "%s(): Argument #%zu ($%s) %s, %s",
            function_name,
            position,
            parameter_name,
            callback_requirement,
            reason
        );
    } else {
        snprintf(
            message,
            (size_t)needed + 1,
            "%s(): Argument #%zu %s, %s",
            function_name,
            position,
            callback_requirement,
            reason
        );
    }
    free(reason);
    return message;
}

static PtnValue ptn_internal_expect_callback_arg_impl(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    PtnValue callback,
    int accepts_null
) {
    PtnValue checked = ptn_value_clone_deref(callback);
    if (ptn_callable_is_valid(checked, 0)) {
        return checked;
    }
    char *message = ptn_invalid_callback_message(
        function_name,
        position,
        parameter_name,
        checked,
        accepts_null
    );
    ptn_value_destroy(&checked);
    ptn_throw_exception_owned_message(runtime, "TypeError", message);
    return ptn_null();
}

static PtnValue ptn_internal_expect_callback_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    PtnValue callback
) {
    return ptn_internal_expect_callback_arg_impl(
        runtime,
        function_name,
        position,
        parameter_name,
        callback,
        0
    );
}

static PtnValue ptn_internal_expect_nullable_callback_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    PtnValue callback
) {
    return ptn_internal_expect_callback_arg_impl(
        runtime,
        function_name,
        position,
        parameter_name,
        callback,
        1
    );
}

static void ptn_var_dump_indent(size_t indent) {
    for (size_t i = 0; i < indent; i++) {
        fputs("  ", stdout);
    }
}

typedef struct {
    PtnArray **items;
    size_t len;
    size_t capacity;
    PtnObject **objects;
    size_t object_len;
    size_t object_capacity;
} PtnDumpSeenArrays;

static void ptn_dump_seen_arrays_init(PtnDumpSeenArrays *seen) {
    seen->items = NULL;
    seen->len = 0;
    seen->capacity = 0;
    seen->objects = NULL;
    seen->object_len = 0;
    seen->object_capacity = 0;
}

static void ptn_dump_seen_arrays_free(PtnDumpSeenArrays *seen) {
    free(seen->items);
    seen->items = NULL;
    seen->len = 0;
    seen->capacity = 0;
    free(seen->objects);
    seen->objects = NULL;
    seen->object_len = 0;
    seen->object_capacity = 0;
}

static int ptn_dump_seen_arrays_contains(PtnDumpSeenArrays *seen, PtnArray *array) {
    for (size_t i = 0; i < seen->len; i++) {
        if (seen->items[i] == array) {
            return 1;
        }
    }
    return 0;
}

static void ptn_dump_seen_arrays_push(PtnDumpSeenArrays *seen, PtnArray *array) {
    if (seen->len == seen->capacity) {
        size_t new_capacity = seen->capacity == 0 ? 8 : seen->capacity * 2;
        if (new_capacity < seen->capacity) {
            ptn_abort_out_of_memory();
        }
        PtnArray **new_items = realloc(seen->items, new_capacity * sizeof(PtnArray *));
        if (new_items == NULL) {
            ptn_abort_out_of_memory();
        }
        seen->items = new_items;
        seen->capacity = new_capacity;
    }
    seen->items[seen->len++] = array;
}

static void ptn_dump_seen_arrays_pop(PtnDumpSeenArrays *seen) {
    if (seen->len > 0) {
        seen->len--;
    }
}

static int ptn_dump_seen_objects_contains(PtnDumpSeenArrays *seen, PtnObject *object) {
    for (size_t i = 0; i < seen->object_len; i++) {
        if (seen->objects[i] == object) {
            return 1;
        }
    }
    return 0;
}

static void ptn_dump_seen_objects_push(PtnDumpSeenArrays *seen, PtnObject *object) {
    if (seen->object_len == seen->object_capacity) {
        size_t new_capacity = seen->object_capacity == 0 ? 8 : seen->object_capacity * 2;
        if (new_capacity < seen->object_capacity) {
            ptn_abort_out_of_memory();
        }
        PtnObject **new_objects = realloc(seen->objects, new_capacity * sizeof(PtnObject *));
        if (new_objects == NULL) {
            ptn_abort_out_of_memory();
        }
        seen->objects = new_objects;
        seen->object_capacity = new_capacity;
    }
    seen->objects[seen->object_len++] = object;
}

static void ptn_dump_seen_objects_pop(PtnDumpSeenArrays *seen) {
    if (seen->object_len > 0) {
        seen->object_len--;
    }
}

static void ptn_var_dump_object_property_key(PtnObject *object, PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        printf("[%lld]=>\n", (long long)key.as.integer);
        return;
    }
    const PtnObjectPropertyMetadata *metadata =
        ptn_object_property_metadata(object, key.as.string);
    const char *display_name = metadata == NULL ? key.as.string : metadata->display_name;
    if (metadata == NULL || metadata->read_visibility == PTN_PROPERTY_PUBLIC) {
        printf("[\"%s\"]=>\n", display_name);
        return;
    }
    if (metadata->read_visibility == PTN_PROPERTY_PROTECTED) {
        printf("[\"%s\":protected]=>\n", display_name);
        return;
    }
    printf(
        "[\"%s\":\"%s\":private]=>\n",
        display_name,
        metadata->declaring_class
    );
}

static const char *ptn_internal_function_parameter_name(const char *name, size_t index) {
    if (name == NULL) {
        return NULL;
    }
    if (index == 0) {
        if (ptn_ascii_case_equal(name, "strlen") ||
            ptn_ascii_case_equal(name, "strrev") ||
            ptn_ascii_case_equal(name, "strtolower") ||
            ptn_ascii_case_equal(name, "strtoupper")) {
            return "string";
        }
        if (ptn_ascii_case_equal(name, "sprintf") ||
            ptn_ascii_case_equal(name, "printf") ||
            ptn_ascii_case_equal(name, "vprintf") ||
            ptn_ascii_case_equal(name, "vsprintf")) {
            return "format";
        }
    }
    if (index == 1) {
        if (ptn_ascii_case_equal(name, "sprintf") ||
            ptn_ascii_case_equal(name, "printf")) {
            return "values";
        }
        if (ptn_ascii_case_equal(name, "vprintf") ||
            ptn_ascii_case_equal(name, "vsprintf")) {
            return "values";
        }
    }
    return NULL;
}

static const char *ptn_function_metadata_parameter_name(PtnFunctionMetadata metadata, size_t index, char *fallback, size_t fallback_len) {
    if (metadata.parameter_names != NULL && index < metadata.parameter_count && metadata.parameter_names[index] != NULL) {
        return metadata.parameter_names[index];
    }
    if (metadata.is_internal) {
        const char *name = ptn_internal_function_parameter_name(metadata.name, index);
        if (name != NULL) {
            return name;
        }
    }
    int written = snprintf(fallback, fallback_len, "param%zu", index + 1);
    if (written < 0 || (size_t)written >= fallback_len) {
        ptn_abort_out_of_memory();
    }
    return fallback;
}

typedef struct {
    int found;
    const char *file;
    size_t file_len;
    size_t line;
} PtnClosureSourceLocation;

static int ptn_parse_size_t_digits(const char *start, size_t len, size_t *result) {
    if (len == 0) {
        return 0;
    }
    size_t value = 0;
    for (size_t i = 0; i < len; i++) {
        unsigned char c = (unsigned char)start[i];
        if (c < '0' || c > '9') {
            return 0;
        }
        size_t digit = (size_t)(c - '0');
        if (value > (SIZE_MAX - digit) / 10) {
            return 0;
        }
        value = value * 10 + digit;
    }
    *result = value;
    return 1;
}

static PtnClosureSourceLocation ptn_closure_source_location(PtnClosure *closure) {
    PtnClosureSourceLocation location;
    location.found = 0;
    location.file = NULL;
    location.file_len = 0;
    location.line = 0;
    if (closure == NULL || closure->display_name == NULL) {
        return location;
    }
    const char *prefix = "{closure:";
    size_t prefix_len = strlen(prefix);
    const char *name = closure->display_name;
    size_t name_len = strlen(name);
    if (
        name_len <= prefix_len + 1 ||
        strncmp(name, prefix, prefix_len) != 0 ||
        name[name_len - 1] != '}'
    ) {
        return location;
    }

    const char *body_start = name + prefix_len;
    const char *body_end = name + name_len - 1;
    const char *line_start = body_end;
    while (line_start > body_start && line_start[-1] != ':') {
        line_start--;
    }
    if (line_start == body_start) {
        return location;
    }
    const char *separator = line_start - 1;
    size_t file_len = (size_t)(separator - body_start);
    size_t line_len = (size_t)(body_end - line_start);
    size_t line = 0;
    if (file_len == 0 || !ptn_parse_size_t_digits(line_start, line_len, &line)) {
        return location;
    }

    location.found = 1;
    location.file = body_start;
    location.file_len = file_len;
    location.line = line;
    return location;
}

static void ptn_var_dump_string_property(
    size_t indent,
    const char *name,
    const char *value,
    size_t value_len
) {
    ptn_var_dump_indent(indent + 1);
    printf("[\"%s\"]=>\n", name);
    ptn_var_dump_indent(indent + 1);
    printf("string(%zu) \"", value_len);
    fwrite(value, 1, value_len, stdout);
    fputs("\"\n", stdout);
}

static size_t ptn_closure_dump_field_count(PtnClosure *closure) {
    PtnFunctionMetadata metadata = closure->metadata;
    PtnClosureSourceLocation location = ptn_closure_source_location(closure);
    size_t count = 0;
    if (location.found) {
        count += 3;
    }
    if (closure->has_wrapped_callable && metadata.found && metadata.name != NULL) {
        count++;
    }
    if (metadata.found && metadata.parameter_count > 0) {
        count++;
    }
    return count;
}

static void ptn_var_dump_closure_parameters(PtnFunctionMetadata metadata, size_t indent) {
    ptn_var_dump_indent(indent);
    printf("array(%zu) {\n", metadata.parameter_count);
    for (size_t i = 0; i < metadata.parameter_count; i++) {
        char fallback[32];
        const char *parameter_name = ptn_function_metadata_parameter_name(
            metadata,
            i,
            fallback,
            sizeof(fallback)
        );
        const char *requiredness = i < metadata.required_parameter_count
            ? "<required>"
            : "<optional>";
        ptn_var_dump_indent(indent + 1);
        printf("[\"$%s\"]=>\n", parameter_name);
        ptn_var_dump_indent(indent + 1);
        printf("string(10) \"%s\"\n", requiredness);
    }
    ptn_var_dump_indent(indent);
    fputs("}\n", stdout);
}

static void ptn_var_dump_closure(PtnClosure *closure, size_t indent) {
    PtnFunctionMetadata metadata = closure->metadata;
    PtnClosureSourceLocation location = ptn_closure_source_location(closure);
    size_t field_count = ptn_closure_dump_field_count(closure);
    printf("object(Closure)#%zu (%zu) {\n", closure->object_id, field_count);
    if (location.found) {
        ptn_var_dump_string_property(
            indent,
            "name",
            closure->display_name,
            strlen(closure->display_name)
        );
        ptn_var_dump_string_property(indent, "file", location.file, location.file_len);
        ptn_var_dump_indent(indent + 1);
        fputs("[\"line\"]=>\n", stdout);
        ptn_var_dump_indent(indent + 1);
        printf("int(%zu)\n", location.line);
    }
    if (closure->has_wrapped_callable && metadata.found && metadata.name != NULL) {
        ptn_var_dump_indent(indent + 1);
        fputs("[\"function\"]=>\n", stdout);
        ptn_var_dump_indent(indent + 1);
        printf("string(%zu) \"%s\"\n", strlen(metadata.name), metadata.name);
    }
    if (metadata.found && metadata.parameter_count > 0) {
        ptn_var_dump_indent(indent + 1);
        fputs("[\"parameter\"]=>\n", stdout);
        ptn_var_dump_closure_parameters(metadata, indent + 1);
    }
    ptn_var_dump_indent(indent);
    fputs("}\n", stdout);
}

static void ptn_var_dump_value_indented(PtnValue value, size_t indent, PtnDumpSeenArrays *seen) {
    int print_reference = value.type == PTN_REFERENCE && value.as.reference->refcount > 1;
    if (value.type == PTN_REFERENCE) {
        value = ptn_value_deref(value);
    }
    if (value.type == PTN_ARRAY && ptn_dump_seen_arrays_contains(seen, value.as.array)) {
        ptn_var_dump_indent(indent);
        fputs("*RECURSION*\n", stdout);
        return;
    }
    if (value.type == PTN_OBJECT && ptn_dump_seen_objects_contains(seen, value.as.object)) {
        ptn_var_dump_indent(indent);
        fputs("*RECURSION*\n", stdout);
        return;
    }
    ptn_var_dump_indent(indent);
    if (print_reference) {
        fputs("&", stdout);
    }
    switch (value.type) {
        case PTN_NULL:
            fputs("NULL\n", stdout);
            break;
        case PTN_BOOL:
            fputs(value.as.boolean ? "bool(true)\n" : "bool(false)\n", stdout);
            break;
        case PTN_INT:
            printf("int(%lld)\n", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[64];
            ptn_format_var_dump_float(value.as.floating, formatted, sizeof(formatted));
            printf("float(%s)\n", formatted);
            break;
        }
        case PTN_STRING:
            printf("string(%zu) \"", value.as.string.len);
            fwrite(value.as.string.data, 1, value.as.string.len, stdout);
            fputs("\"\n", stdout);
            break;
        case PTN_ARRAY: {
            PtnArray *array = value.as.array;
            printf("array(%zu) {\n", array->len);
            ptn_dump_seen_arrays_push(seen, array);
            for (size_t i = 0; i < array->len; i++) {
                ptn_var_dump_indent(indent + 1);
                PtnArrayKey key = array->entries[i].key;
                if (key.type == PTN_ARRAY_KEY_INT) {
                    printf("[%lld]=>\n", (long long)key.as.integer);
                } else {
                    fputs("[\"", stdout);
                    fwrite(key.as.string, 1, key.string_len, stdout);
                    fputs("\"]=>\n", stdout);
                }
                ptn_var_dump_value_indented(array->entries[i].value, indent + 1, seen);
            }
            ptn_dump_seen_arrays_pop(seen);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        }
        case PTN_OBJECT: {
            PtnObject *object = value.as.object;
            PtnArray *properties = object->properties;
            printf("object(%s)#%zu (%zu) {\n", object->class_name, object->object_id, properties->len);
            ptn_dump_seen_objects_push(seen, object);
            for (size_t i = 0; i < properties->len; i++) {
                ptn_var_dump_indent(indent + 1);
                PtnArrayKey key = properties->entries[i].key;
                ptn_var_dump_object_property_key(object, key);
                ptn_var_dump_value_indented(properties->entries[i].value, indent + 1, seen);
            }
            ptn_dump_seen_objects_pop(seen);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        }
        case PTN_CLOSURE:
            ptn_var_dump_closure(value.as.closure, indent);
            break;
        case PTN_EXCEPTION:
            printf("object(%s)#%zu (1) {\n", value.as.exception->class_name, value.as.exception->object_id);
            ptn_var_dump_indent(indent + 1);
            fputs("[\"message\"]=>\n", stdout);
            ptn_var_dump_indent(indent + 1);
            printf("string(%zu) \"", strlen(value.as.exception->message));
            fputs(value.as.exception->message, stdout);
            fputs("\"\n", stdout);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        case PTN_RESOURCE:
            printf("resource(%lld) of type (%s)\n", (long long)value.as.resource->id, value.as.resource->type_name);
            break;
        case PTN_REFERENCE:
            fputs("NULL\n", stdout);
            break;
    }
}

static void ptn_var_dump_value(PtnValue value) {
    if (value.type == PTN_REFERENCE) {
        value = ptn_value_deref(value);
    }
    PtnDumpSeenArrays seen;
    ptn_dump_seen_arrays_init(&seen);
    ptn_var_dump_value_indented(value, 0, &seen);
    ptn_dump_seen_arrays_free(&seen);
}

static PtnValue ptn_internal_var_dump(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    for (size_t i = 0; i < argc; i++) {
        ptn_var_dump_value(args[i]);
    }
    return ptn_null();
}

static void ptn_debug_zval_dump_value_indented(PtnValue value, size_t indent, PtnDumpSeenArrays *seen) {
    ptn_var_dump_indent(indent);
    switch (value.type) {
        case PTN_REFERENCE:
            printf("reference refcount(%zu) {\n", value.as.reference->refcount);
            ptn_debug_zval_dump_value_indented(value.as.reference->value, indent + 1, seen);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        case PTN_ARRAY: {
            PtnArray *array = value.as.array;
            if (ptn_dump_seen_arrays_contains(seen, array)) {
                fputs("*RECURSION*\n", stdout);
                break;
            }
            if (array->len == 0) {
                fputs("array(0) interned {\n", stdout);
            } else {
                printf(
                    "array(%zu) packed refcount(%zu){\n",
                    array->len,
                    ptn_array_debug_visible_refcount(array)
                );
            }
            ptn_dump_seen_arrays_push(seen, array);
            for (size_t i = 0; i < array->len; i++) {
                ptn_var_dump_indent(indent + 1);
                PtnArrayKey key = array->entries[i].key;
                if (key.type == PTN_ARRAY_KEY_INT) {
                    printf("[%lld]=>\n", (long long)key.as.integer);
                } else {
                    printf("[\"%s\"]=>\n", key.as.string);
                }
                ptn_debug_zval_dump_value_indented(array->entries[i].value, indent + 1, seen);
            }
            ptn_dump_seen_arrays_pop(seen);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        }
        case PTN_OBJECT: {
            PtnObject *object = value.as.object;
            if (ptn_dump_seen_objects_contains(seen, object)) {
                fputs("*RECURSION*\n", stdout);
                break;
            }
            PtnArray *properties = object->properties;
            printf(
                "object(%s)#%zu (%zu) refcount(%zu){\n",
                object->class_name,
                object->object_id,
                properties->len,
                object->refcount
            );
            ptn_dump_seen_objects_push(seen, object);
            for (size_t i = 0; i < properties->len; i++) {
                ptn_var_dump_indent(indent + 1);
                PtnArrayKey key = properties->entries[i].key;
                ptn_var_dump_object_property_key(object, key);
                ptn_debug_zval_dump_value_indented(properties->entries[i].value, indent + 1, seen);
            }
            ptn_dump_seen_objects_pop(seen);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        }
        case PTN_NULL:
            fputs("NULL\n", stdout);
            break;
        case PTN_BOOL:
            fputs(value.as.boolean ? "bool(true)\n" : "bool(false)\n", stdout);
            break;
        case PTN_INT:
            printf("int(%lld)\n", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[64];
            ptn_format_var_dump_float(value.as.floating, formatted, sizeof(formatted));
            printf("float(%s)\n", formatted);
            break;
        }
        case PTN_STRING:
            printf("string(%zu) \"", value.as.string.len);
            fwrite(value.as.string.data, 1, value.as.string.len, stdout);
            fputs("\"\n", stdout);
            break;
        case PTN_CLOSURE:
            ptn_var_dump_closure(value.as.closure, indent);
            break;
        case PTN_EXCEPTION:
            printf("object(%s)#%zu (1) {\n", value.as.exception->class_name, value.as.exception->object_id);
            ptn_var_dump_indent(indent + 1);
            fputs("[\"message\"]=>\n", stdout);
            ptn_var_dump_indent(indent + 1);
            printf("string(%zu) \"", strlen(value.as.exception->message));
            fputs(value.as.exception->message, stdout);
            fputs("\"\n", stdout);
            ptn_var_dump_indent(indent);
            fputs("}\n", stdout);
            break;
        case PTN_RESOURCE:
            printf("resource(%lld) of type (%s)\n", (long long)value.as.resource->id, value.as.resource->type_name);
            break;
    }
}

static PtnValue ptn_internal_debug_zval_dump(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    for (size_t i = 0; i < argc; i++) {
        PtnDumpSeenArrays seen;
        ptn_dump_seen_arrays_init(&seen);
        ptn_debug_zval_dump_value_indented(args[i], 0, &seen);
        ptn_dump_seen_arrays_free(&seen);
    }
    return ptn_null();
}

static PtnValue ptn_internal__ptn_cow_debug_reset(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    ptn_cow_debug_reset();
    return ptn_null();
}

static PtnValue ptn_internal__ptn_cow_debug_counter(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    size_t counter = 0;
    int found = ptn_cow_debug_counter(name, &counter);
    free(name);
    if (!found) {
        ptn_cow_debug_abort("unknown counter");
    }
    if (counter > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_int((int64_t)counter);
}

static PtnValue ptn_internal__ptn_cow_debug_assert_counter(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    PtnValue expected = ptn_cast_int(args[1]);
    ptn_cow_debug_assert_named_counter(name, expected.as.integer);
    free(name);
    return ptn_bool(1);
}

static PtnValue ptn_internal__ptn_cow_debug_assert_balanced(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    ptn_cow_debug_assert_balanced();
    return ptn_bool(1);
}

static void ptn_print_r_value_indented(PtnStringBuffer *buffer, PtnValue value, size_t indent);

static void ptn_print_r_key(PtnStringBuffer *buffer, PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        ptn_string_buffer_append_format(buffer, "%lld", (long long)key.as.integer);
    } else {
        ptn_string_buffer_append(buffer, key.as.string);
    }
}

static void ptn_print_r_array(PtnStringBuffer *buffer, PtnArray *array, size_t indent) {
    ptn_string_buffer_append(buffer, "Array\n");
    ptn_string_buffer_append_indent(buffer, indent);
    ptn_string_buffer_append(buffer, "(\n");
    for (size_t i = 0; i < array->len; i++) {
        ptn_string_buffer_append_indent(buffer, indent + 4);
        ptn_string_buffer_append_char(buffer, '[');
        ptn_print_r_key(buffer, array->entries[i].key);
        ptn_string_buffer_append(buffer, "] => ");
        PtnValue entry_value = ptn_value_deref(array->entries[i].value);
        if (entry_value.type == PTN_ARRAY) {
            ptn_print_r_value_indented(buffer, entry_value, indent + 8);
            ptn_string_buffer_append_char(buffer, '\n');
        } else {
            ptn_print_r_value_indented(buffer, entry_value, indent);
            ptn_string_buffer_append_char(buffer, '\n');
        }
    }
    ptn_string_buffer_append_indent(buffer, indent);
    ptn_string_buffer_append(buffer, ")\n");
}

static void ptn_print_r_value_indented(PtnStringBuffer *buffer, PtnValue value, size_t indent) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            break;
        case PTN_BOOL:
            if (value.as.boolean) {
                ptn_string_buffer_append_char(buffer, '1');
            }
            break;
        case PTN_INT:
            ptn_string_buffer_append_format(buffer, "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[128];
            ptn_format_scalar_float(value.as.floating, formatted, sizeof(formatted));
            ptn_string_buffer_append(buffer, formatted);
            break;
        }
        case PTN_STRING:
            ptn_string_buffer_append_len(
                buffer,
                (const char *)value.as.string.data,
                value.as.string.len
            );
            break;
        case PTN_ARRAY:
            ptn_print_r_array(buffer, value.as.array, indent);
            break;
        case PTN_OBJECT:
        case PTN_CLOSURE:
            ptn_string_buffer_append(buffer, "Object");
            break;
        case PTN_EXCEPTION:
            ptn_string_buffer_append(buffer, "Object");
            break;
        case PTN_RESOURCE:
            ptn_string_buffer_append_format(buffer, "Resource id #%lld", (long long)value.as.resource->id);
            break;
        case PTN_REFERENCE:
            break;
    }
}

static PtnValue ptn_internal_print_r(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    int return_output = argc >= 2 && ptn_is_truthy(args[1]);
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    ptn_print_r_value_indented(&buffer, args[0], 0);
    if (return_output) {
        return ptn_owned_string_len(buffer.data, buffer.len);
    }
    fwrite(buffer.data, 1, buffer.len, stdout);
    free(buffer.data);
    return ptn_bool(1);
}

static void ptn_var_export_append_value(PtnStringBuffer *buffer, PtnValue value, size_t indent);

static void ptn_var_export_append_single_quoted_string(
    PtnStringBuffer *buffer,
    const char *data,
    size_t len
) {
    ptn_string_buffer_append_char(buffer, '\'');
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)data[i];
        if (byte == '\\' || byte == '\'') {
            ptn_string_buffer_append_char(buffer, '\\');
        }
        ptn_string_buffer_append_char(buffer, (char)byte);
    }
    ptn_string_buffer_append_char(buffer, '\'');
}

static void ptn_var_export_append_string(PtnStringBuffer *buffer, const char *data, size_t len) {
    size_t segment_start = 0;
    for (size_t i = 0; i < len; i++) {
        if (data[i] != '\0') {
            continue;
        }
        ptn_var_export_append_single_quoted_string(buffer, data + segment_start, i - segment_start);
        ptn_string_buffer_append(buffer, " . \"\\0\" . ");
        segment_start = i + 1;
    }
    size_t segment_len = len - segment_start;
    ptn_var_export_append_single_quoted_string(
        buffer,
        segment_len == 0 ? data : data + segment_start,
        segment_len
    );
}

static void ptn_var_export_append_key(PtnStringBuffer *buffer, PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        ptn_string_buffer_append_format(buffer, "%lld", (long long)key.as.integer);
    } else {
        ptn_var_export_append_string(buffer, key.as.string, key.string_len);
    }
}

static int ptn_var_export_is_complex_value(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_ARRAY || value.type == PTN_OBJECT;
}

static void ptn_var_export_append_array(PtnStringBuffer *buffer, PtnArray *array, size_t indent) {
    ptn_string_buffer_append(buffer, "array (\n");
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        PtnValue entry_value = ptn_value_deref(entry->value);
        ptn_string_buffer_append_indent(buffer, indent + 2);
        ptn_var_export_append_key(buffer, entry->key);
        ptn_string_buffer_append(buffer, " => ");
        if (ptn_var_export_is_complex_value(entry_value)) {
            ptn_string_buffer_append_char(buffer, '\n');
            ptn_string_buffer_append_indent(buffer, indent + 2);
        }
        ptn_var_export_append_value(buffer, entry_value, indent + 2);
        ptn_string_buffer_append(buffer, ",\n");
    }
    ptn_string_buffer_append_indent(buffer, indent);
    ptn_string_buffer_append_char(buffer, ')');
}

static void ptn_var_export_append_object_state_array(
    PtnStringBuffer *buffer,
    PtnObject *object,
    size_t indent
) {
    PtnArray *properties = object->properties;
    ptn_string_buffer_append(buffer, "array(\n");
    for (size_t i = 0; i < properties->len; i++) {
        PtnArrayEntry *entry = &properties->entries[i];
        PtnValue entry_value = ptn_value_deref(entry->value);
        ptn_string_buffer_append_indent(buffer, indent + 3);
        PtnArrayKey display_key = entry->key;
        int free_display_key = 0;
        if (entry->key.type == PTN_ARRAY_KEY_STRING) {
            const PtnObjectPropertyMetadata *metadata =
                ptn_object_property_metadata(object, entry->key.as.string);
            if (metadata != NULL) {
                display_key = ptn_array_string_key(metadata->display_name);
                free_display_key = 1;
            }
        }
        ptn_var_export_append_key(buffer, display_key);
        if (free_display_key) {
            ptn_array_key_free(display_key);
        }
        ptn_string_buffer_append(buffer, " => ");
        if (ptn_var_export_is_complex_value(entry_value)) {
            ptn_string_buffer_append_char(buffer, '\n');
            ptn_string_buffer_append_indent(buffer, indent + 2);
        }
        ptn_var_export_append_value(buffer, entry_value, indent + 2);
        ptn_string_buffer_append(buffer, ",\n");
    }
    ptn_string_buffer_append_indent(buffer, indent);
    ptn_string_buffer_append_char(buffer, ')');
}

static void ptn_var_export_append_object(PtnStringBuffer *buffer, PtnObject *object, size_t indent) {
    if (strcmp(object->class_name, "stdClass") == 0) {
        ptn_string_buffer_append(buffer, "(object) ");
    } else {
        ptn_string_buffer_append_char(buffer, '\\');
        ptn_string_buffer_append(buffer, object->class_name);
        ptn_string_buffer_append(buffer, "::__set_state(");
    }
    ptn_var_export_append_object_state_array(buffer, object, indent);
    if (strcmp(object->class_name, "stdClass") != 0) {
        ptn_string_buffer_append_char(buffer, ')');
    }
}

static void ptn_var_export_append_value(PtnStringBuffer *buffer, PtnValue value, size_t indent) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            ptn_string_buffer_append(buffer, "NULL");
            break;
        case PTN_BOOL:
            ptn_string_buffer_append(buffer, value.as.boolean ? "true" : "false");
            break;
        case PTN_INT:
            ptn_string_buffer_append_format(buffer, "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            char formatted[64];
            ptn_format_var_dump_float(value.as.floating, formatted, sizeof(formatted));
            ptn_string_buffer_append(buffer, formatted);
            break;
        }
        case PTN_STRING:
            ptn_var_export_append_string(
                buffer,
                (const char *)value.as.string.data,
                value.as.string.len
            );
            break;
        case PTN_RESOURCE:
            ptn_string_buffer_append(buffer, "NULL");
            break;
        case PTN_ARRAY:
            ptn_var_export_append_array(buffer, value.as.array, indent);
            break;
        case PTN_OBJECT:
            ptn_var_export_append_object(buffer, value.as.object, indent);
            break;
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            ptn_string_buffer_append(buffer, "NULL");
            break;
    }
}

static PtnValue ptn_internal_var_export(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    int return_output = argc >= 2 && ptn_is_truthy(args[1]);
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    ptn_var_export_append_value(&buffer, args[0], 0);
    if (return_output) {
        return ptn_owned_string_len(buffer.data, buffer.len);
    }
    fwrite(buffer.data, 1, buffer.len, stdout);
    free(buffer.data);
    return ptn_bool(1);
}

static void ptn_json_encode_append_value(
    PtnStringBuffer *buffer,
    PtnValue value,
    PtnDumpSeenArrays *seen,
    size_t depth,
    int *ok
);

static void ptn_json_encode_append_hex4(PtnStringBuffer *buffer, unsigned char byte) {
    static const char hex[] = "0123456789abcdef";
    ptn_string_buffer_append(buffer, "\\u00");
    ptn_string_buffer_append_char(buffer, hex[(byte >> 4) & 0xf]);
    ptn_string_buffer_append_char(buffer, hex[byte & 0xf]);
}

static void ptn_json_encode_append_string(PtnStringBuffer *buffer, const unsigned char *data, size_t len) {
    ptn_string_buffer_append_char(buffer, '"');
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = data[i];
        switch (byte) {
            case '"':
                ptn_string_buffer_append(buffer, "\\\"");
                break;
            case '\\':
                ptn_string_buffer_append(buffer, "\\\\");
                break;
            case '/':
                ptn_string_buffer_append(buffer, "\\/");
                break;
            case '\b':
                ptn_string_buffer_append(buffer, "\\b");
                break;
            case '\f':
                ptn_string_buffer_append(buffer, "\\f");
                break;
            case '\n':
                ptn_string_buffer_append(buffer, "\\n");
                break;
            case '\r':
                ptn_string_buffer_append(buffer, "\\r");
                break;
            case '\t':
                ptn_string_buffer_append(buffer, "\\t");
                break;
            default:
                if (byte < 0x20) {
                    ptn_json_encode_append_hex4(buffer, byte);
                } else {
                    ptn_string_buffer_append_char(buffer, (char)byte);
                }
                break;
        }
    }
    ptn_string_buffer_append_char(buffer, '"');
}

static int ptn_json_array_is_list(PtnArray *array) {
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            return 0;
        }
        PtnArrayKey key = array->entries[i].key;
        if (key.type != PTN_ARRAY_KEY_INT || key.as.integer != (int64_t)i) {
            return 0;
        }
    }
    return 1;
}

static void ptn_json_encode_append_array(
    PtnStringBuffer *buffer,
    PtnArray *array,
    PtnDumpSeenArrays *seen,
    size_t depth,
    int *ok
) {
    if (depth == 0 || ptn_dump_seen_arrays_contains(seen, array)) {
        *ok = 0;
        return;
    }
    ptn_dump_seen_arrays_push(seen, array);
    int list = ptn_json_array_is_list(array);
    ptn_string_buffer_append_char(buffer, list ? '[' : '{');
    for (size_t i = 0; i < array->len; i++) {
        if (i != 0) {
            ptn_string_buffer_append_char(buffer, ',');
        }
        if (!list) {
            PtnArrayKey key = array->entries[i].key;
            if (key.type == PTN_ARRAY_KEY_INT) {
                char key_buffer[64];
                int written = snprintf(key_buffer, sizeof(key_buffer), "%lld", (long long)key.as.integer);
                if (written < 0 || (size_t)written >= sizeof(key_buffer)) {
                    ptn_abort_out_of_memory();
                }
                ptn_json_encode_append_string(buffer, (const unsigned char *)key_buffer, (size_t)written);
            } else {
                ptn_json_encode_append_string(buffer, (const unsigned char *)key.as.string, key.string_len);
            }
            ptn_string_buffer_append_char(buffer, ':');
        }
        ptn_json_encode_append_value(buffer, array->entries[i].value, seen, depth - 1, ok);
        if (!*ok) {
            ptn_dump_seen_arrays_pop(seen);
            return;
        }
    }
    ptn_string_buffer_append_char(buffer, list ? ']' : '}');
    ptn_dump_seen_arrays_pop(seen);
}

static void ptn_json_encode_append_object(
    PtnStringBuffer *buffer,
    PtnObject *object,
    PtnDumpSeenArrays *seen,
    size_t depth,
    int *ok
) {
    if (depth == 0) {
        *ok = 0;
        return;
    }
    ptn_string_buffer_append_char(buffer, '{');
    size_t emitted = 0;
    for (size_t i = 0; i < object->properties->len; i++) {
        PtnArrayEntry *entry = &object->properties->entries[i];
        if (entry->key.type != PTN_ARRAY_KEY_STRING) {
            continue;
        }
        const PtnObjectPropertyMetadata *metadata =
            ptn_object_property_metadata(object, entry->key.as.string);
        if (metadata != NULL && metadata->read_visibility != PTN_PROPERTY_PUBLIC) {
            continue;
        }
        if (emitted != 0) {
            ptn_string_buffer_append_char(buffer, ',');
        }
        ptn_json_encode_append_string(
            buffer,
            (const unsigned char *)entry->key.as.string,
            entry->key.string_len
        );
        ptn_string_buffer_append_char(buffer, ':');
        ptn_json_encode_append_value(buffer, entry->value, seen, depth - 1, ok);
        if (!*ok) {
            return;
        }
        emitted++;
    }
    ptn_string_buffer_append_char(buffer, '}');
}

static void ptn_json_encode_append_value(
    PtnStringBuffer *buffer,
    PtnValue value,
    PtnDumpSeenArrays *seen,
    size_t depth,
    int *ok
) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            ptn_string_buffer_append(buffer, "null");
            break;
        case PTN_BOOL:
            ptn_string_buffer_append(buffer, value.as.boolean ? "true" : "false");
            break;
        case PTN_INT:
            ptn_string_buffer_append_format(buffer, "%lld", (long long)value.as.integer);
            break;
        case PTN_FLOAT: {
            if (!isfinite(value.as.floating)) {
                *ok = 0;
                break;
            }
            char formatted[128];
            ptn_format_scalar_float(value.as.floating, formatted, sizeof(formatted));
            ptn_string_buffer_append(buffer, formatted);
            break;
        }
        case PTN_STRING:
            ptn_json_encode_append_string(buffer, value.as.string.data, value.as.string.len);
            break;
        case PTN_ARRAY:
            ptn_json_encode_append_array(buffer, value.as.array, seen, depth, ok);
            break;
        case PTN_OBJECT:
            ptn_json_encode_append_object(buffer, value.as.object, seen, depth, ok);
            break;
        case PTN_RESOURCE:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            *ok = 0;
            break;
    }
}

static PtnValue ptn_internal_json_encode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    size_t depth = 512;
    if (argc >= 3) {
        int64_t requested_depth = ptn_value_to_integer(args[2]);
        if (requested_depth <= 0) {
            ptn_throw_exception(
                runtime,
                "ValueError",
                "json_encode(): Argument #3 ($depth) must be greater than 0"
            );
            return ptn_null();
        }
        depth = (size_t)requested_depth;
    }

    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    PtnDumpSeenArrays seen;
    ptn_dump_seen_arrays_init(&seen);
    int ok = 1;
    ptn_json_encode_append_value(&buffer, args[0], &seen, depth, &ok);
    ptn_dump_seen_arrays_free(&seen);
    if (!ok) {
        free(buffer.data);
        return ptn_bool(0);
    }
    return ptn_owned_string_len(buffer.data, buffer.len);
}

static PtnArray *ptn_internal_expect_array_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        return value.as.array;
    }

    char message[192];
    int written = argument_name != NULL && argument_name[0] != '\0'
        ? snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) must be of type array, %s given",
            function_name,
            position,
            argument_name,
            ptn_offset_container_type_name(value)
        )
        : snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu must be of type array, %s given",
            function_name,
            position,
            ptn_offset_container_type_name(value)
        );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return NULL;
}

static PtnArray *ptn_internal_expect_mutable_array_variable_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    const char *variable_name,
    PtnValue value
) {
    PtnArray *array = ptn_internal_expect_array_arg(
        runtime,
        function_name,
        position,
        argument_name,
        value
    );
    size_t index = ptn_symbols_find(&runtime->symbols, variable_name);
    if (index >= runtime->symbols.len || runtime->symbols.items[index].value.type != PTN_ARRAY) {
        return array;
    }
    return ptn_array_detach_value(&runtime->symbols.items[index].value);
}

static PtnArray *ptn_internal_expect_mutable_array_path_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    const char *variable_name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnValue null_value = ptn_null();
    if (segment_count == 0) {
        PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, variable_name);
        PtnValue value = slot == NULL ? null_value : *slot;
        return ptn_internal_expect_mutable_array_variable_arg(
            runtime,
            function_name,
            position,
            argument_name,
            variable_name,
            value
        );
    }

    PtnValue *slot = ptn_symbols_value_slot(&runtime->symbols, variable_name);
    if (slot == NULL) {
        return ptn_internal_expect_array_arg(runtime, function_name, position, argument_name, null_value);
    }

    PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, function_name, position, argument_name, *slot_value);
    array = ptn_array_detach_value(slot_value);

    for (size_t i = 0; i < segment_count; i++) {
        const PtnArrayPathSegment *segment = &segments[i];
        if (segment->append) {
            return ptn_internal_expect_array_arg(runtime, function_name, position, argument_name, null_value);
        }
        ptn_array_path_emit_null_key_deprecation(runtime, segment, line, 1);
        PtnArrayKey key = ptn_array_key_from_value(segment->value);
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
        ptn_array_key_free(key);
        if (entry == NULL) {
            return ptn_internal_expect_array_arg(runtime, function_name, position, argument_name, null_value);
        }

        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        if (i + 1 == segment_count) {
            PtnArray *leaf = ptn_internal_expect_array_arg(
                runtime,
                function_name,
                position,
                argument_name,
                *entry_value
            );
            (void)leaf;
            return ptn_array_detach_value(entry_value);
        }

        array = ptn_internal_expect_array_arg(
            runtime,
            function_name,
            position,
            argument_name,
            *entry_value
        );
        array = ptn_array_detach_value(entry_value);
    }

    return ptn_internal_expect_array_arg(runtime, function_name, position, argument_name, null_value);
}

static PTN_UNUSED PtnValue ptn_runtime_array_pop_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_pop",
        1,
        "array",
        name,
        value
    );
    return ptn_array_pop_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_pop_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnArray *array = ptn_internal_expect_mutable_array_path_arg(
        runtime,
        "array_pop",
        1,
        "array",
        name,
        segments,
        segment_count,
        line
    );
    return ptn_array_pop_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_push_variable(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t value_count,
    const PtnValue *values
) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_push",
        1,
        "array",
        name,
        value
    );
    return ptn_int(ptn_array_push_values(array, value_count, values));
}

static PTN_UNUSED PtnValue ptn_runtime_array_shift_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_shift",
        1,
        "array",
        name,
        value
    );
    return ptn_array_shift_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_shift_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnArray *array = ptn_internal_expect_mutable_array_path_arg(
        runtime,
        "array_shift",
        1,
        "array",
        name,
        segments,
        segment_count,
        line
    );
    return ptn_array_shift_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_ksort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "ksort",
        1,
        "array",
        name,
        value
    );
    ptn_array_ksort_entries(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_krsort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "krsort",
        1,
        "array",
        name,
        value
    );
    ptn_array_krsort_entries(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_asort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "asort",
        1,
        "array",
        name,
        value
    );
    ptn_array_asort_values(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_arsort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "arsort",
        1,
        "array",
        name,
        value
    );
    ptn_array_arsort_values(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_natsort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "natsort",
        1,
        "array",
        name,
        value
    );
    ptn_array_natsort_values(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_natcasesort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "natcasesort",
        1,
        "array",
        name,
        value
    );
    ptn_array_natcasesort_values(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_sort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "sort",
        1,
        "array",
        name,
        value
    );
    ptn_array_sort_values(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_rsort_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "rsort",
        1,
        "array",
        name,
        value
    );
    ptn_array_rsort_values(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_shuffle_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "shuffle",
        1,
        "array",
        name,
        value
    );
    ptn_array_shuffle_values(array);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_unshift_variable(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    size_t value_count,
    const PtnValue *values
) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "array_unshift",
        1,
        "array",
        name,
        value
    );
    return ptn_int(ptn_array_unshift_values(array, value_count, values));
}

static PTN_UNUSED PtnValue ptn_runtime_array_next_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "next",
        1,
        "array",
        name,
        value
    );
    return ptn_array_next_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_next_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnArray *array = ptn_internal_expect_mutable_array_path_arg(
        runtime,
        "next",
        1,
        "array",
        name,
        segments,
        segment_count,
        line
    );
    return ptn_array_next_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_end_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "end",
        1,
        "array",
        name,
        value
    );
    return ptn_array_end_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_end_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnArray *array = ptn_internal_expect_mutable_array_path_arg(
        runtime,
        "end",
        1,
        "array",
        name,
        segments,
        segment_count,
        line
    );
    return ptn_array_end_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_prev_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "prev",
        1,
        "array",
        name,
        value
    );
    return ptn_array_prev_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_prev_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnArray *array = ptn_internal_expect_mutable_array_path_arg(
        runtime,
        "prev",
        1,
        "array",
        name,
        segments,
        segment_count,
        line
    );
    return ptn_array_prev_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_reset_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnArray *array = ptn_internal_expect_mutable_array_variable_arg(
        runtime,
        "reset",
        1,
        "array",
        name,
        value
    );
    return ptn_array_reset_value(array);
}

static PTN_UNUSED PtnValue ptn_runtime_array_reset_path(
    PtnRuntime *runtime,
    const char *name,
    const PtnArrayPathSegment *segments,
    size_t segment_count,
    size_t line
) {
    PtnArray *array = ptn_internal_expect_mutable_array_path_arg(
        runtime,
        "reset",
        1,
        "array",
        name,
        segments,
        segment_count,
        line
    );
    return ptn_array_reset_value(array);
}

static PtnArray *ptn_array_walk_slot_array_for_write(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue *slot,
    PtnValue value
) {
    PtnValue current_value = slot == NULL ? value : *slot;
    PtnArray *array = ptn_internal_expect_array_arg(
        runtime,
        function_name,
        1,
        "array",
        current_value
    );
    if (slot == NULL) {
        return array;
    }

    PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    if (slot_value->type != PTN_ARRAY) {
        return array;
    }

    PtnArray *detached = ptn_array_detach_value(slot_value);
    return detached == NULL ? slot_value->as.array : detached;
}

static PtnArray *ptn_array_walk_slot_current_array(PtnValue *slot) {
    if (slot == NULL) {
        return NULL;
    }
    PtnValue *slot_value = slot->type == PTN_REFERENCE ? &slot->as.reference->value : slot;
    return slot_value->type == PTN_ARRAY ? slot_value->as.array : NULL;
}

static PtnValue *ptn_array_walk_current_slot(
    PtnRuntime *runtime,
    const char *name,
    PtnValue *local_slot
) {
    if (name == NULL) {
        return local_slot;
    }
    return ptn_symbols_value_slot(&runtime->symbols, name);
}

static void ptn_array_walk_call_function(
    PtnRuntime *runtime,
    PtnValue callback,
    PtnArray *array,
    size_t index,
    int has_userdata,
    PtnValue userdata,
    size_t line
) {
    PtnArrayEntry *entry = &array->entries[index];
    if (entry->value.type != PTN_REFERENCE) {
        PtnValue current = entry->value;
        entry->value = ptn_reference_value(ptn_reference_new_owned(current));
    }

    PtnValue value_reference = ptn_value_clone(entry->value);
    PtnValue key = ptn_array_key_value(entry->key);
    PtnValue callback_args[3] = {
        value_reference,
        key,
        has_userdata ? ptn_value_clone_deref(userdata) : ptn_null()
    };
    PtnValue callback_result = ptn_call_callable(
        runtime,
        callback,
        has_userdata ? 3 : 2,
        callback_args,
        line
    );
    ptn_value_destroy(&callback_args[0]);
    ptn_value_destroy(&callback_args[1]);
    if (has_userdata) {
        ptn_value_destroy(&callback_args[2]);
    }
    ptn_value_destroy(&callback_result);
}

static PtnArrayKey *ptn_array_walk_snapshot_keys(PtnArray *array, size_t *count_out) {
    *count_out = array->len;
    if (array->len == 0) {
        return NULL;
    }
    PtnArrayKey *keys = malloc(array->len * sizeof(PtnArrayKey));
    if (keys == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < array->len; i++) {
        keys[i] = ptn_array_key_clone(array->entries[i].key);
    }
    return keys;
}

static void ptn_array_walk_free_snapshot_keys(PtnArrayKey *keys, size_t count) {
    if (keys == NULL) {
        return;
    }
    for (size_t i = 0; i < count; i++) {
        ptn_array_key_free(keys[i]);
    }
    free(keys);
}

static PtnValue ptn_array_walk_slot(
    PtnRuntime *runtime,
    const char *name,
    PtnValue *local_slot,
    PtnValue value,
    PtnValue callback,
    int has_userdata,
    PtnValue userdata,
    size_t line
) {
    PtnArray *last_array = NULL;
    PtnArrayKey *snapshot_keys = NULL;
    size_t snapshot_count = 0;
    size_t snapshot_index = 0;

    for (;;) {
        PtnValue *slot = ptn_array_walk_current_slot(runtime, name, local_slot);
        PtnArray *array = ptn_array_walk_slot_array_for_write(runtime, "array_walk", slot, value);
        if (array != last_array) {
            ptn_array_walk_free_snapshot_keys(snapshot_keys, snapshot_count);
            last_array = array;
            snapshot_keys = ptn_array_walk_snapshot_keys(array, &snapshot_count);
            snapshot_index = 0;
        }
        if (snapshot_index >= snapshot_count) {
            break;
        }

        PtnArrayKey key = ptn_array_key_clone(snapshot_keys[snapshot_index]);
        snapshot_index++;
        size_t entry_index = ptn_array_find_key(array, key);
        ptn_array_key_free(key);
        if (entry_index < array->len) {
            ptn_array_walk_call_function(
                runtime,
                callback,
                array,
                entry_index,
                has_userdata,
                userdata,
                line
            );
        }

        PtnValue *after_slot = ptn_array_walk_current_slot(runtime, name, local_slot);
        PtnArray *after_array = ptn_array_walk_slot_current_array(after_slot);
        if (after_array != array && after_slot != NULL) {
            last_array = NULL;
        }
    }

    ptn_array_walk_free_snapshot_keys(snapshot_keys, snapshot_count);
    return ptn_bool(1);
}

static PtnValue ptn_array_walk_checked(
    PtnRuntime *runtime,
    const char *name,
    PtnValue *local_slot,
    PtnValue value,
    PtnValue callback,
    int has_userdata,
    PtnValue userdata,
    size_t line
) {
    PtnValue checked_callback = ptn_internal_expect_callback_arg(
        runtime,
        "array_walk",
        2,
        "callback",
        callback
    );
    int previous_warn_by_ref_argument_mismatch = runtime->warn_by_ref_argument_mismatch;
    int previous_throw_argument_count_errors = runtime->throw_argument_count_errors;
    runtime->warn_by_ref_argument_mismatch = 1;
    runtime->throw_argument_count_errors = 1;
    PtnValue result = ptn_array_walk_slot(
        runtime,
        name,
        local_slot,
        value,
        checked_callback,
        has_userdata,
        userdata,
        line
    );
    runtime->throw_argument_count_errors = previous_throw_argument_count_errors;
    runtime->warn_by_ref_argument_mismatch = previous_warn_by_ref_argument_mismatch;
    ptn_value_destroy(&checked_callback);
    return result;
}

static PTN_UNUSED PtnValue ptn_runtime_array_walk_variable(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    PtnValue callback,
    int has_userdata,
    PtnValue userdata,
    size_t line
) {
    PtnValue trace_args[3] = {
        value,
        callback,
        userdata
    };
    PtnTraceFrame trace_frame;
    ptn_runtime_push_trace_frame(
        runtime,
        &trace_frame,
        "array_walk",
        runtime->source_path,
        line,
        has_userdata ? 3 : 2,
        trace_args
    );
    PtnValue result = ptn_array_walk_checked(runtime, name, NULL, value, callback, has_userdata, userdata, line);
    ptn_runtime_pop_trace_frame(runtime, &trace_frame);
    return result;
}

static void ptn_array_walk_recursive_array(
    PtnRuntime *runtime,
    PtnArray *array,
    PtnValue callback,
    int has_userdata,
    PtnValue userdata,
    size_t line
) {
    size_t snapshot_count = 0;
    PtnArrayKey *snapshot_keys = ptn_array_walk_snapshot_keys(array, &snapshot_count);

    for (size_t i = 0; i < snapshot_count; i++) {
        PtnArrayEntry *entry = ptn_array_entry_for_key(array, snapshot_keys[i]);
        if (entry == NULL) {
            continue;
        }

        PtnValue *entry_value = entry->value.type == PTN_REFERENCE
            ? &entry->value.as.reference->value
            : &entry->value;
        PtnValue resolved = ptn_value_deref(*entry_value);
        if (resolved.type == PTN_ARRAY) {
            PtnArray *child = ptn_array_detach_value(entry_value);
            ptn_array_walk_recursive_array(
                runtime,
                child == NULL ? resolved.as.array : child,
                callback,
                has_userdata,
                userdata,
                line
            );
            continue;
        }

        size_t current_index = (size_t)(entry - array->entries);
        ptn_array_walk_call_function(
            runtime,
            callback,
            array,
            current_index,
            has_userdata,
            userdata,
            line
        );
    }

    ptn_array_walk_free_snapshot_keys(snapshot_keys, snapshot_count);
}

static PtnValue ptn_array_walk_recursive_checked(
    PtnRuntime *runtime,
    const char *name,
    PtnValue *local_slot,
    PtnValue value,
    PtnValue callback,
    int has_userdata,
    PtnValue userdata,
    size_t line
) {
    PtnValue checked_callback = ptn_internal_expect_callback_arg(
        runtime,
        "array_walk_recursive",
        2,
        "callback",
        callback
    );
    PtnValue *slot = ptn_array_walk_current_slot(runtime, name, local_slot);
    PtnArray *array = ptn_array_walk_slot_array_for_write(
        runtime,
        "array_walk_recursive",
        slot,
        value
    );
    int previous_warn_by_ref_argument_mismatch = runtime->warn_by_ref_argument_mismatch;
    int previous_throw_argument_count_errors = runtime->throw_argument_count_errors;
    runtime->warn_by_ref_argument_mismatch = 1;
    runtime->throw_argument_count_errors = 1;
    ptn_array_walk_recursive_array(
        runtime,
        array,
        checked_callback,
        has_userdata,
        userdata,
        line
    );
    runtime->throw_argument_count_errors = previous_throw_argument_count_errors;
    runtime->warn_by_ref_argument_mismatch = previous_warn_by_ref_argument_mismatch;
    ptn_value_destroy(&checked_callback);
    return ptn_bool(1);
}

static PTN_UNUSED PtnValue ptn_runtime_array_walk_recursive_variable(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    PtnValue callback,
    int has_userdata,
    PtnValue userdata,
    size_t line
) {
    PtnValue trace_args[3] = {
        value,
        callback,
        userdata
    };
    PtnTraceFrame trace_frame;
    ptn_runtime_push_trace_frame(
        runtime,
        &trace_frame,
        "array_walk_recursive",
        runtime->source_path,
        line,
        has_userdata ? 3 : 2,
        trace_args
    );
    PtnValue result = ptn_array_walk_recursive_checked(runtime, name, NULL, value, callback, has_userdata, userdata, line);
    ptn_runtime_pop_trace_frame(runtime, &trace_frame);
    return result;
}

static PtnValue ptn_internal_array_pop(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_pop", 1, "array", args[0]);
    return ptn_array_pop_value(array);
}

static PtnValue ptn_internal_array_push(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_push", 1, "array", args[0]);
    return ptn_int(ptn_array_push_values(array, argc - 1, args + 1));
}

static PtnValue ptn_internal_array_shift(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_shift", 1, "array", args[0]);
    return ptn_array_shift_value(array);
}

static void ptn_internal_throw_sort_flags_unsupported(PtnRuntime *runtime, const char *name) {
    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "%s() flags are unsupported; default regular value sorting is supported",
        name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_throw_exception(runtime, "Error", "sort flags are unsupported; default regular value sorting is supported");
        return;
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PtnValue ptn_internal_ksort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    if (argc >= 2) {
        ptn_internal_throw_sort_flags_unsupported(runtime, "ksort");
        return ptn_null();
    }
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "ksort", 1, "array", args[0]);
    ptn_array_ksort_entries(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_krsort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    if (argc >= 2) {
        ptn_internal_throw_sort_flags_unsupported(runtime, "krsort");
        return ptn_null();
    }
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "krsort", 1, "array", args[0]);
    ptn_array_krsort_entries(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_asort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    if (argc >= 2) {
        ptn_internal_throw_sort_flags_unsupported(runtime, "asort");
        return ptn_null();
    }
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "asort", 1, "array", args[0]);
    ptn_array_asort_values(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_arsort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    if (argc >= 2) {
        ptn_internal_throw_sort_flags_unsupported(runtime, "arsort");
        return ptn_null();
    }
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "arsort", 1, "array", args[0]);
    ptn_array_arsort_values(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_shuffle(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "shuffle", 1, "array", args[0]);
    ptn_array_shuffle_values(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_natsort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "natsort", 1, "array", args[0]);
    ptn_array_natsort_values(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_natcasesort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "natcasesort", 1, "array", args[0]);
    ptn_array_natcasesort_values(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_sort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    if (argc >= 2) {
        ptn_internal_throw_sort_flags_unsupported(runtime, "sort");
        return ptn_null();
    }
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "sort", 1, "array", args[0]);
    ptn_array_sort_values(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_rsort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    if (argc >= 2) {
        ptn_internal_throw_sort_flags_unsupported(runtime, "rsort");
        return ptn_null();
    }
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "rsort", 1, "array", args[0]);
    ptn_array_rsort_values(array);
    return ptn_bool(1);
}

static PtnValue ptn_internal_array_multisort(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    for (size_t i = 0; i < argc; i++) {
        PtnValue value = ptn_value_deref(args[i]);
        if (value.type != PTN_ARRAY) {
            char message[192];
            int written = snprintf(
                message,
                sizeof(message),
                "array_multisort(): Argument #%zu ($array) must be of type array, %s given",
                i + 1,
                ptn_offset_container_type_name(value)
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_type_error(&runtime->diagnostics, message);
            exit(255);
        }
        if (args[i].type == PTN_REFERENCE && args[i].as.reference->value.type == PTN_ARRAY) {
            ptn_array_sort_values(args[i].as.reference->value.as.array);
        }
    }
    return ptn_bool(1);
}

static const char *ptn_array_aggregate_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT) {
        return value.as.object->class_name;
    }
    if (value.type == PTN_EXCEPTION) {
        return value.as.exception->class_name;
    }
    if (value.type == PTN_CLOSURE) {
        return "Closure";
    }
    return ptn_offset_container_type_name(value);
}

static void ptn_array_aggregate_warn_unsupported(
    PtnRuntime *runtime,
    const char *function_name,
    const char *operation_name,
    PtnValue value,
    size_t line
) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): %s is not supported on type %s",
        function_name,
        operation_name,
        ptn_array_aggregate_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        fputc('\n', stdout);
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
}

static PtnNumber ptn_array_aggregate_number(
    PtnRuntime *runtime,
    const char *function_name,
    const char *operation_name,
    PtnValue value,
    int product_mode,
    size_t line
) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
            return ptn_to_number(value);
        case PTN_STRING:
            if (!ptn_string_has_embedded_nul(value.as.string)) {
                double unused = 0.0;
                if (ptn_is_numeric_string((const char *)value.as.string.data, &unused)) {
                    return ptn_string_to_number((const char *)value.as.string.data);
                }
            }
            ptn_array_aggregate_warn_unsupported(runtime, function_name, operation_name, value, line);
            return ptn_number_int(0);
        case PTN_RESOURCE:
            ptn_array_aggregate_warn_unsupported(runtime, function_name, operation_name, value, line);
            return ptn_number_int(value.as.resource->id);
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            ptn_array_aggregate_warn_unsupported(runtime, function_name, operation_name, value, line);
            return ptn_number_int(product_mode ? 1 : 0);
        case PTN_REFERENCE:
            return ptn_number_int(0);
    }
    return ptn_number_int(0);
}

static int ptn_int64_add_overflows(int64_t left, int64_t right, int64_t *result) {
    if ((right > 0 && left > INT64_MAX - right) || (right < 0 && left < INT64_MIN - right)) {
        return 1;
    }
    *result = left + right;
    return 0;
}

static int ptn_int64_multiply_overflows(int64_t left, int64_t right, int64_t *result) {
    long double product = (long double)left * (long double)right;
    if (product > (long double)INT64_MAX || product < (long double)INT64_MIN) {
        return 1;
    }
    *result = left * right;
    return 0;
}

static PtnValue ptn_internal_array_sum(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_sum", 1, "array", args[0]);
    int use_float = 0;
    int64_t integer_sum = 0;
    double float_sum = 0.0;
    for (size_t i = 0; i < array->len; i++) {
        PtnNumber number = ptn_array_aggregate_number(runtime, "array_sum", "Addition", array->entries[i].value, 0, line);
        if (number.type == PTN_NUMBER_FLOAT) {
            if (!use_float) {
                float_sum = (double)integer_sum;
                use_float = 1;
            }
            float_sum += number.floating;
        } else if (use_float) {
            float_sum += (double)number.integer;
        } else {
            int64_t next_sum = 0;
            if (ptn_int64_add_overflows(integer_sum, number.integer, &next_sum)) {
                float_sum = (double)integer_sum + (double)number.integer;
                use_float = 1;
            } else {
                integer_sum = next_sum;
            }
        }
    }
    return use_float ? ptn_float(float_sum) : ptn_int(integer_sum);
}

static PtnValue ptn_internal_array_product(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_product", 1, "array", args[0]);
    int use_float = 0;
    int64_t integer_product = 1;
    double float_product = 1.0;
    for (size_t i = 0; i < array->len; i++) {
        PtnNumber number = ptn_array_aggregate_number(runtime, "array_product", "Multiplication", array->entries[i].value, 1, line);
        if (number.type == PTN_NUMBER_FLOAT) {
            if (!use_float) {
                float_product = (double)integer_product;
                use_float = 1;
            }
            float_product *= number.floating;
        } else if (use_float) {
            float_product *= (double)number.integer;
        } else {
            int64_t next_product = 0;
            if (ptn_int64_multiply_overflows(integer_product, number.integer, &next_product)) {
                float_product = (double)integer_product * (double)number.integer;
                use_float = 1;
            } else {
                integer_product = next_product;
            }
        }
    }
    return use_float ? ptn_float(float_product) : ptn_int(integer_product);
}

static PtnArrayKey ptn_array_change_key_case_key(PtnArrayKey source, int uppercase) {
    if (source.type == PTN_ARRAY_KEY_INT) {
        return ptn_array_int_key(source.as.integer);
    }

    size_t len = source.string_len;
    char *changed = malloc(len + 1);
    if (changed == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)source.as.string[i];
        changed[i] = (char)(uppercase ? toupper(byte) : tolower(byte));
    }
    changed[len] = '\0';

    PtnArrayKey key;
    key.type = PTN_ARRAY_KEY_STRING;
    key.string_len = len;
    key.as.string = changed;
    return key;
}

static PtnValue ptn_internal_array_change_key_case(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_change_key_case", 1, "array", args[0]);
    int64_t case_value = argc >= 2 ? ptn_value_to_integer(args[1]) : 0;
    if (case_value != 0 && case_value != 1) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_change_key_case(): Argument #2 ($case) must be either CASE_LOWER or CASE_UPPER"
        );
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    int uppercase = case_value == 1;
    for (size_t i = 0; i < array->len; i++) {
        ptn_array_set_entry(
            result.as.array,
            ptn_array_change_key_case_key(array->entries[i].key, uppercase),
            ptn_value_clone_deref(array->entries[i].value)
        );
    }
    return result;
}

static void ptn_array_chunk_append_result(PtnValue *result, PtnValue chunk, int64_t *chunk_index) {
    ptn_array_set_entry(
        result->as.array,
        ptn_array_int_key(*chunk_index),
        chunk
    );
    if (*chunk_index == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    (*chunk_index)++;
}

static PtnValue ptn_internal_array_chunk(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_chunk", 1, "array", args[0]);
    int64_t length = ptn_value_to_integer(args[1]);
    if (length < 1) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_chunk(): Argument #2 ($length) must be greater than 0"
        );
    }

    int preserve_keys = argc >= 3 && ptn_is_truthy(args[2]);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    PtnValue chunk = ptn_null();
    int64_t chunk_index = 0;
    int64_t chunk_len = 0;

    for (size_t i = 0; i < array->len; i++) {
        if (chunk_len == 0) {
            chunk = ptn_array_from_literal_entries(0, NULL);
        }

        PtnArrayEntry *source = &array->entries[i];
        PtnArrayKey key = preserve_keys
            ? ptn_array_key_clone(source->key)
            : ptn_array_int_key(chunk.as.array->next_auto_key);
        PtnValue value = ptn_array_reindexing_internal_value(source->value);
        ptn_array_set_entry(chunk.as.array, key, ptn_value_clone(value));

        chunk_len++;
        if (chunk_len == length) {
            ptn_array_chunk_append_result(&result, chunk, &chunk_index);
            chunk = ptn_null();
            chunk_len = 0;
        }
    }

    if (chunk_len != 0) {
        ptn_array_chunk_append_result(&result, chunk, &chunk_index);
    }

    return result;
}

typedef struct {
    int is_null;
    PtnArrayKey array_key;
    char *property_name;
} PtnArrayColumnKey;

static char *ptn_array_column_integer_property_name(int64_t integer) {
    char buffer[32];
    int written = snprintf(buffer, sizeof(buffer), "%lld", (long long)integer);
    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_duplicate_string(buffer);
}

static const char *ptn_array_column_argument_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT) {
        return value.as.object->class_name;
    }
    return ptn_offset_container_type_name(value);
}

static PtnArrayColumnKey ptn_array_column_key_from_arg(
    PtnRuntime *runtime,
    size_t position,
    const char *argument_name,
    PtnValue value
) {
    value = ptn_value_deref(value);
    PtnArrayColumnKey key;
    key.is_null = 0;
    key.property_name = NULL;

    int64_t integer = 0;
    switch (value.type) {
        case PTN_NULL:
            key.is_null = 1;
            key.array_key = ptn_array_int_key(0);
            return key;
        case PTN_BOOL:
            integer = value.as.boolean ? 1 : 0;
            key.array_key = ptn_array_int_key(integer);
            key.property_name = ptn_array_column_integer_property_name(integer);
            return key;
        case PTN_INT:
            key.array_key = ptn_array_int_key(value.as.integer);
            key.property_name = ptn_array_column_integer_property_name(value.as.integer);
            return key;
        case PTN_FLOAT:
            integer = ptn_value_to_integer(value);
            key.array_key = ptn_array_int_key(integer);
            key.property_name = ptn_array_column_integer_property_name(integer);
            return key;
        case PTN_STRING:
            key.array_key = ptn_array_key_from_value(value);
            key.property_name = ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len);
            return key;
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE: {
            char message[192];
            int written = snprintf(
                message,
                sizeof(message),
                "array_column(): Argument #%zu ($%s) must be of type string|int|null, %s given",
                position,
                argument_name,
                ptn_array_column_argument_type_name(value)
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "TypeError", message);
            key.is_null = 1;
            key.array_key = ptn_array_int_key(0);
            return key;
        }
    }

    key.is_null = 1;
    key.array_key = ptn_array_int_key(0);
    return key;
}

static void ptn_array_column_key_free(PtnArrayColumnKey key) {
    if (key.is_null) {
        return;
    }
    ptn_array_key_free(key.array_key);
    free(key.property_name);
}

static int ptn_array_column_lookup(PtnValue row, PtnArrayColumnKey key, PtnValue *value_out) {
    row = ptn_value_deref(row);
    if (key.is_null) {
        *value_out = ptn_value_clone_deref(row);
        return 1;
    }

    if (row.type == PTN_ARRAY) {
        PtnArrayEntry *entry = ptn_array_entry_for_key(row.as.array, key.array_key);
        if (entry == NULL) {
            return 0;
        }
        *value_out = ptn_value_clone_deref(entry->value);
        return 1;
    }

    if (row.type == PTN_OBJECT) {
        PtnArrayKey property_key = ptn_array_string_key(key.property_name);
        PtnArrayEntry *entry = ptn_array_entry_for_key(row.as.object->properties, property_key);
        ptn_array_key_free(property_key);
        if (entry == NULL) {
            return 0;
        }
        *value_out = ptn_value_clone_deref(entry->value);
        return 1;
    }

    return 0;
}

static void ptn_array_column_add_value(PtnValue *result, PtnValue row, PtnArrayColumnKey index_key, PtnValue value) {
    if (!index_key.is_null) {
        PtnValue index_value;
        if (ptn_array_column_lookup(row, index_key, &index_value)) {
            PtnArrayKey result_key = ptn_array_key_from_value(index_value);
            ptn_value_destroy(&index_value);
            ptn_array_set_entry(result->as.array, result_key, value);
            return;
        }
    }

    PtnArrayKey next_key = ptn_array_int_key(result->as.array->next_auto_key);
    ptn_array_set_entry(result->as.array, next_key, value);
}

static PtnValue ptn_internal_array_column(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_column", 1, "array", args[0]);
    PtnArrayColumnKey column_key = ptn_array_column_key_from_arg(runtime, 2, "column_key", args[1]);
    PtnArrayColumnKey index_key = argc >= 3
        ? ptn_array_column_key_from_arg(runtime, 3, "index_key", args[2])
        : (PtnArrayColumnKey){ 1, { 0 }, NULL };

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < array->len; i++) {
        PtnValue value;
        if (!ptn_array_column_lookup(array->entries[i].value, column_key, &value)) {
            continue;
        }
        ptn_array_column_add_value(&result, array->entries[i].value, index_key, value);
    }

    ptn_array_column_key_free(column_key);
    ptn_array_column_key_free(index_key);
    return result;
}

static int ptn_array_count_values_key_from_value(PtnRuntime *runtime, PtnValue value, size_t line, PtnArrayKey *key_out) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_INT:
        case PTN_STRING:
            *key_out = ptn_array_key_from_value(value);
            return 1;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_FLOAT:
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            ptn_emit_spaced_warning(
                &runtime->diagnostics,
                "array_count_values(): Can only count string and integer values, entry skipped",
                line
            );
            return 0;
    }
    return 0;
}

static PtnValue ptn_internal_array_count_values(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_count_values", 1, "array", args[0]);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayKey key;
        if (!ptn_array_count_values_key_from_value(runtime, array->entries[i].value, line, &key)) {
            continue;
        }

        int64_t count = 1;
        PtnArrayEntry *existing = ptn_array_entry_for_key(result.as.array, key);
        if (existing != NULL) {
            PtnValue existing_value = ptn_value_deref(existing->value);
            count = existing_value.as.integer + 1;
        }
        ptn_array_set_entry(result.as.array, key, ptn_int(count));
    }
    return result;
}

static int ptn_array_flip_key_from_value(PtnRuntime *runtime, PtnValue value, size_t line, PtnArrayKey *key_out) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_INT:
        case PTN_STRING:
            *key_out = ptn_array_key_from_value(value);
            return 1;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_FLOAT:
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            ptn_emit_spaced_warning(
                &runtime->diagnostics,
                "array_flip(): Can only flip string and integer values, entry skipped",
                line
            );
            return 0;
    }
    return 0;
}

static PtnValue ptn_internal_array_flip(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_flip", 1, "array", args[0]);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayKey key;
        if (!ptn_array_flip_key_from_value(runtime, array->entries[i].value, line, &key)) {
            continue;
        }
        ptn_array_set_entry(result.as.array, key, ptn_array_key_value(array->entries[i].key));
    }
    return result;
}

static PtnValue ptn_internal_array_keys(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_keys", 1, "array", args[0]);
    int has_search_value = argc >= 2;
    int strict = argc >= 3 && ptn_is_truthy(args[2]);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        if (has_search_value) {
            int matched = strict
                ? ptn_compare_identical(args[1], entry->value)
                : ptn_compare_equal(args[1], entry->value);
            if (!matched) {
                continue;
            }
        }

        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key(result.as.array->next_auto_key),
            ptn_array_key_value(entry->key)
        );
    }
    return result;
}

static PtnValue ptn_internal_array_key_first(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_key_first", 1, "array", args[0]);
    if (array->len == 0) {
        return ptn_null();
    }
    return ptn_array_key_value(array->entries[0].key);
}

static PtnValue ptn_internal_array_key_last(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_key_last", 1, "array", args[0]);
    if (array->len == 0) {
        return ptn_null();
    }
    return ptn_array_key_value(array->entries[array->len - 1].key);
}

static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);

static PtnValue ptn_internal_array_rand(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_rand", 1, "array", args[0]);
    int64_t requested = argc >= 2
        ? ptn_internal_expect_integer_arg(runtime, "array_rand", 2, "num", args[1], line)
        : 1;
    if (array->len == 0) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_rand(): Argument #1 ($array) must not be empty"
        );
        return ptn_null();
    }
    if (requested < 1 || (uint64_t)requested > (uint64_t)array->len) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_rand(): Argument #2 ($num) must be between 1 and the number of elements in argument #1 ($array)"
        );
        return ptn_null();
    }
    if (requested == 1) {
        size_t index = ptn_random_bounded_index(array->len - 1);
        return ptn_array_key_value(array->entries[index].key);
    }

    size_t *indices = malloc(array->len * sizeof(size_t));
    if (indices == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < array->len; i++) {
        indices[i] = i;
    }
    size_t count = (size_t)requested;
    for (size_t i = 0; i < count; i++) {
        size_t j = i + ptn_random_bounded_index(array->len - i - 1);
        size_t tmp = indices[i];
        indices[i] = indices[j];
        indices[j] = tmp;
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < count; i++) {
        if (i > (size_t)INT64_MAX) {
            free(indices);
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key((int64_t)i),
            ptn_array_key_value(array->entries[indices[i]].key)
        );
    }
    free(indices);
    return result;
}

static PtnValue ptn_internal_array_first(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_first", 1, "array", args[0]);
    if (array->len == 0) {
        return ptn_null();
    }
    return ptn_value_clone(ptn_array_reindexing_internal_value(array->entries[0].value));
}

static PtnValue ptn_internal_array_last(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_last", 1, "array", args[0]);
    if (array->len == 0) {
        return ptn_null();
    }
    return ptn_value_clone(ptn_array_reindexing_internal_value(array->entries[array->len - 1].value));
}

static PtnValue ptn_internal_array_is_list(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnValue value = ptn_value_deref(args[0]);
    if (value.type != PTN_ARRAY) {
        const char *given = value.type == PTN_OBJECT
            ? value.as.object->class_name
            : ptn_count_operand_type_name(value);
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "array_is_list(): Argument #1 ($array) must be of type array, %s given",
            given
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }

    PtnArray *array = value.as.array;
    for (size_t i = 0; i < array->len; i++) {
        if (i > (size_t)INT64_MAX) {
            return ptn_bool(0);
        }
        PtnArrayKey key = array->entries[i].key;
        if (key.type != PTN_ARRAY_KEY_INT || key.as.integer != (int64_t)i) {
            return ptn_bool(0);
        }
    }
    return ptn_bool(1);
}

static int ptn_array_value_strings_equal(PtnValue left, PtnValue right) {
    PtnStringOperand left_string = ptn_value_to_string_operand(left);
    PtnStringOperand right_string = ptn_value_to_string_operand(right);
    int equal = left_string.len == right_string.len &&
        memcmp(left_string.data, right_string.data, left_string.len) == 0;
    ptn_string_operand_free(left_string);
    ptn_string_operand_free(right_string);
    return equal;
}

static void ptn_array_emit_string_conversion_warning_for_value(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_emit_spaced_warning(&runtime->diagnostics, "Array to string conversion", line);
    }
}

static void ptn_array_emit_set_operation_string_conversion_warnings(
    PtnRuntime *runtime,
    PtnArray *array,
    size_t line
) {
    for (size_t i = 0; i < array->len; i++) {
        ptn_array_emit_string_conversion_warning_for_value(runtime, array->entries[i].value, line);
    }
}

static int ptn_array_contains_value(PtnArray *array, PtnValue value) {
    for (size_t i = 0; i < array->len; i++) {
        if (ptn_array_value_strings_equal(value, array->entries[i].value)) {
            return 1;
        }
    }
    return 0;
}

static int ptn_array_contains_assoc_value(PtnArray *array, PtnArrayKey key, PtnValue value) {
    PtnArrayEntry *entry = ptn_array_entry_for_key(array, key);
    return entry != NULL && ptn_array_value_strings_equal(value, entry->value);
}

static int ptn_array_entry_matches_all(
    PtnArrayEntry *entry,
    size_t array_count,
    PtnArray **arrays,
    int compare_keys
) {
    for (size_t i = 0; i < array_count; i++) {
        int found = compare_keys
            ? ptn_array_contains_assoc_value(arrays[i], entry->key, entry->value)
            : ptn_array_contains_value(arrays[i], entry->value);
        if (!found) {
            return 0;
        }
    }
    return 1;
}

static int ptn_array_entry_matches_any(
    PtnArrayEntry *entry,
    size_t array_count,
    PtnArray **arrays,
    int compare_keys
) {
    for (size_t i = 0; i < array_count; i++) {
        int found = compare_keys
            ? ptn_array_contains_assoc_value(arrays[i], entry->key, entry->value)
            : ptn_array_contains_value(arrays[i], entry->value);
        if (found) {
            return 1;
        }
    }
    return 0;
}

static PtnValue ptn_array_intersect_or_diff(
    PtnRuntime *runtime,
    PtnArray *source,
    size_t array_count,
    PtnArray **arrays,
    int compare_keys,
    int keep_matches,
    size_t line
) {
    if (array_count != 0) {
        ptn_array_emit_set_operation_string_conversion_warnings(runtime, source, line);
        for (size_t i = 0; i < array_count; i++) {
            ptn_array_emit_set_operation_string_conversion_warnings(runtime, arrays[i], line);
        }
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        int keep = keep_matches
            ? ptn_array_entry_matches_all(entry, array_count, arrays, compare_keys)
            : !ptn_array_entry_matches_any(entry, array_count, arrays, compare_keys);
        if (keep) {
            ptn_array_set_entry(
                result.as.array,
                ptn_array_key_clone(entry->key),
                ptn_value_clone(ptn_array_reindexing_internal_value(entry->value))
            );
        }
    }
    return result;
}

static PtnArray **ptn_array_set_operation_array_args(
    PtnRuntime *runtime,
    const char *function_name,
    size_t array_count,
    const PtnValue *args
);

static PtnArray **ptn_array_set_operation_arrays(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args
) {
    size_t array_count = argc - 1;
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, function_name, array_count, args);
    return arrays;
}

static PtnArray **ptn_array_set_operation_array_args(
    PtnRuntime *runtime,
    const char *function_name,
    size_t array_count,
    const PtnValue *args
) {
    PtnArray **arrays = NULL;
    if (array_count != 0) {
        arrays = malloc(array_count * sizeof(PtnArray *));
        if (arrays == NULL) {
            ptn_abort_out_of_memory();
        }
    }

    for (size_t i = 0; i < array_count; i++) {
        arrays[i] = ptn_internal_expect_array_arg(runtime, function_name, i + 2, NULL, args[i + 1]);
    }
    return arrays;
}

static PtnValue ptn_internal_array_diff(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_diff", 1, "array", args[0]);
    PtnArray **arrays = ptn_array_set_operation_arrays(runtime, "array_diff", argc, args);
    PtnValue result = ptn_array_intersect_or_diff(runtime, source, argc - 1, arrays, 0, 0, line);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_diff_assoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_diff_assoc", 1, "array", args[0]);
    PtnArray **arrays = ptn_array_set_operation_arrays(runtime, "array_diff_assoc", argc, args);
    PtnValue result = ptn_array_intersect_or_diff(runtime, source, argc - 1, arrays, 1, 0, line);
    free(arrays);
    return result;
}

static int ptn_array_entry_key_matches_all(PtnArrayEntry *entry, size_t array_count, PtnArray **arrays) {
    for (size_t i = 0; i < array_count; i++) {
        if (ptn_array_entry_for_key(arrays[i], entry->key) == NULL) {
            return 0;
        }
    }
    return 1;
}

static int ptn_array_entry_key_matches_any(PtnArrayEntry *entry, size_t array_count, PtnArray **arrays) {
    for (size_t i = 0; i < array_count; i++) {
        if (ptn_array_entry_for_key(arrays[i], entry->key) != NULL) {
            return 1;
        }
    }
    return 0;
}

static PtnValue ptn_array_key_intersect_or_diff(
    PtnArray *source,
    size_t array_count,
    PtnArray **arrays,
    int keep_matches
) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        int keep = keep_matches
            ? ptn_array_entry_key_matches_all(entry, array_count, arrays)
            : !ptn_array_entry_key_matches_any(entry, array_count, arrays);
        if (keep) {
            ptn_array_set_entry(
                result.as.array,
                ptn_array_key_clone(entry->key),
                ptn_value_clone(ptn_array_reindexing_internal_value(entry->value))
            );
        }
    }
    return result;
}

static PtnValue ptn_internal_array_diff_key(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_diff_key", 1, "array", args[0]);
    PtnArray **arrays = ptn_array_set_operation_arrays(runtime, "array_diff_key", argc, args);
    PtnValue result = ptn_array_key_intersect_or_diff(source, argc - 1, arrays, 0);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_intersect(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_intersect", 1, "array", args[0]);
    PtnArray **arrays = ptn_array_set_operation_arrays(runtime, "array_intersect", argc, args);
    PtnValue result = ptn_array_intersect_or_diff(runtime, source, argc - 1, arrays, 0, 1, line);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_intersect_assoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_intersect_assoc", 1, "array", args[0]);
    PtnArray **arrays = ptn_array_set_operation_arrays(runtime, "array_intersect_assoc", argc, args);
    PtnValue result = ptn_array_intersect_or_diff(runtime, source, argc - 1, arrays, 1, 1, line);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_intersect_key(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_intersect_key", 1, "array", args[0]);
    PtnArray **arrays = ptn_array_set_operation_arrays(runtime, "array_intersect_key", argc, args);
    PtnValue result = ptn_array_key_intersect_or_diff(source, argc - 1, arrays, 1);
    free(arrays);
    return result;
}

static int ptn_array_user_compare(
    PtnRuntime *runtime,
    PtnValue callback,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    PtnValue callback_args[2] = {
        ptn_value_clone_deref(left),
        ptn_value_clone_deref(right)
    };
    PtnValue callback_result = ptn_call_callable(runtime, callback, 2, callback_args, line);
    int64_t compared = ptn_value_to_integer(callback_result);
    ptn_value_destroy(&callback_args[0]);
    ptn_value_destroy(&callback_args[1]);
    ptn_value_destroy(&callback_result);
    if (compared < 0) {
        return -1;
    }
    if (compared > 0) {
        return 1;
    }
    return 0;
}

typedef enum {
    PTN_ARRAY_SET_VALUE_NONE,
    PTN_ARRAY_SET_VALUE_STRING,
    PTN_ARRAY_SET_VALUE_CALLBACK
} PtnArraySetValueMode;

typedef enum {
    PTN_ARRAY_SET_KEY_NONE,
    PTN_ARRAY_SET_KEY_EXACT,
    PTN_ARRAY_SET_KEY_CALLBACK
} PtnArraySetKeyMode;

static int ptn_array_custom_keys_match(
    PtnRuntime *runtime,
    PtnArrayKey left,
    PtnArrayKey right,
    PtnArraySetKeyMode key_mode,
    PtnValue key_callback,
    size_t line
) {
    if (key_mode == PTN_ARRAY_SET_KEY_NONE) {
        return 1;
    }
    if (key_mode == PTN_ARRAY_SET_KEY_EXACT) {
        return ptn_array_keys_equal(left, right);
    }

    PtnValue left_key = ptn_array_key_value(left);
    PtnValue right_key = ptn_array_key_value(right);
    int matched = ptn_array_user_compare(runtime, key_callback, left_key, right_key, line) == 0;
    ptn_value_destroy(&left_key);
    ptn_value_destroy(&right_key);
    return matched;
}

static int ptn_array_custom_values_match(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    PtnArraySetValueMode value_mode,
    PtnValue value_callback,
    size_t line
) {
    if (value_mode == PTN_ARRAY_SET_VALUE_NONE) {
        return 1;
    }
    if (value_mode == PTN_ARRAY_SET_VALUE_STRING) {
        return ptn_array_value_strings_equal(left, right);
    }
    return ptn_array_user_compare(runtime, value_callback, left, right, line) == 0;
}

static int ptn_array_custom_entry_matches_candidate(
    PtnRuntime *runtime,
    PtnArrayEntry *entry,
    PtnArrayEntry *candidate,
    PtnArraySetValueMode value_mode,
    PtnValue value_callback,
    PtnArraySetKeyMode key_mode,
    PtnValue key_callback,
    size_t line
) {
    if (!ptn_array_custom_keys_match(runtime, entry->key, candidate->key, key_mode, key_callback, line)) {
        return 0;
    }
    return ptn_array_custom_values_match(runtime, entry->value, candidate->value, value_mode, value_callback, line);
}

static int ptn_array_custom_entry_matches_array(
    PtnRuntime *runtime,
    PtnArrayEntry *entry,
    PtnArray *array,
    PtnArraySetValueMode value_mode,
    PtnValue value_callback,
    PtnArraySetKeyMode key_mode,
    PtnValue key_callback,
    size_t line
) {
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *candidate = &array->entries[i];
        if (ptn_array_custom_entry_matches_candidate(
                runtime,
                entry,
                candidate,
                value_mode,
                value_callback,
                key_mode,
                key_callback,
                line
            )) {
            return 1;
        }
    }
    return 0;
}

static PtnValue ptn_array_custom_set_operation(
    PtnRuntime *runtime,
    PtnArray *source,
    size_t array_count,
    PtnArray **arrays,
    PtnArraySetValueMode value_mode,
    PtnValue value_callback,
    PtnArraySetKeyMode key_mode,
    PtnValue key_callback,
    int keep_matches,
    size_t line
) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        size_t matched_arrays = 0;
        for (size_t array_index = 0; array_index < array_count; array_index++) {
            int matched = ptn_array_custom_entry_matches_array(
                    runtime,
                    entry,
                    arrays[array_index],
                    value_mode,
                    value_callback,
                    key_mode,
                    key_callback,
                    line
                );
            if (keep_matches && !matched) {
                matched_arrays = 0;
                break;
            }
            if (matched) {
                matched_arrays++;
                if (!keep_matches) {
                    break;
                }
            }
        }
        int keep = keep_matches ? matched_arrays == array_count : matched_arrays == 0;
        if (keep) {
            ptn_array_set_entry(
                result.as.array,
                ptn_array_key_clone(entry->key),
                ptn_value_clone(ptn_array_reindexing_internal_value(entry->value))
            );
        }
    }
    return result;
}

static PtnValue ptn_internal_array_udiff(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_udiff", 1, "array", args[0]);
    PtnValue value_callback = ptn_internal_expect_callback_arg(runtime, "array_udiff", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_udiff", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_CALLBACK,
        value_callback,
        PTN_ARRAY_SET_KEY_NONE,
        ptn_null(),
        0,
        line
    );
    ptn_value_destroy(&value_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_udiff_assoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_udiff_assoc", 1, "array", args[0]);
    PtnValue value_callback = ptn_internal_expect_callback_arg(runtime, "array_udiff_assoc", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_udiff_assoc", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_CALLBACK,
        value_callback,
        PTN_ARRAY_SET_KEY_EXACT,
        ptn_null(),
        0,
        line
    );
    ptn_value_destroy(&value_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_udiff_uassoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 3;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_udiff_uassoc", 1, "array", args[0]);
    PtnValue value_callback = ptn_internal_expect_callback_arg(runtime, "array_udiff_uassoc", argc - 1, NULL, args[argc - 2]);
    PtnValue key_callback = ptn_internal_expect_callback_arg(runtime, "array_udiff_uassoc", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_udiff_uassoc", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_CALLBACK,
        value_callback,
        PTN_ARRAY_SET_KEY_CALLBACK,
        key_callback,
        0,
        line
    );
    ptn_value_destroy(&value_callback);
    ptn_value_destroy(&key_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_diff_uassoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_diff_uassoc", 1, "array", args[0]);
    PtnValue key_callback = ptn_internal_expect_callback_arg(runtime, "array_diff_uassoc", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_diff_uassoc", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_STRING,
        ptn_null(),
        PTN_ARRAY_SET_KEY_CALLBACK,
        key_callback,
        0,
        line
    );
    ptn_value_destroy(&key_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_diff_ukey(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_diff_ukey", 1, "array", args[0]);
    PtnValue key_callback = ptn_internal_expect_callback_arg(runtime, "array_diff_ukey", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_diff_ukey", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_NONE,
        ptn_null(),
        PTN_ARRAY_SET_KEY_CALLBACK,
        key_callback,
        0,
        line
    );
    ptn_value_destroy(&key_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_intersect_uassoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_intersect_uassoc", 1, "array", args[0]);
    PtnValue key_callback = ptn_internal_expect_callback_arg(runtime, "array_intersect_uassoc", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_intersect_uassoc", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_STRING,
        ptn_null(),
        PTN_ARRAY_SET_KEY_CALLBACK,
        key_callback,
        1,
        line
    );
    ptn_value_destroy(&key_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_intersect_ukey(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_intersect_ukey", 1, "array", args[0]);
    PtnValue key_callback = ptn_internal_expect_callback_arg(runtime, "array_intersect_ukey", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_intersect_ukey", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_NONE,
        ptn_null(),
        PTN_ARRAY_SET_KEY_CALLBACK,
        key_callback,
        1,
        line
    );
    ptn_value_destroy(&key_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_uintersect(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_uintersect", 1, "array", args[0]);
    PtnValue value_callback = ptn_internal_expect_callback_arg(runtime, "array_uintersect", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_uintersect", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_CALLBACK,
        value_callback,
        PTN_ARRAY_SET_KEY_NONE,
        ptn_null(),
        1,
        line
    );
    ptn_value_destroy(&value_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_uintersect_assoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 2;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_uintersect_assoc", 1, "array", args[0]);
    PtnValue value_callback = ptn_internal_expect_callback_arg(runtime, "array_uintersect_assoc", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_uintersect_assoc", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_CALLBACK,
        value_callback,
        PTN_ARRAY_SET_KEY_EXACT,
        ptn_null(),
        1,
        line
    );
    ptn_value_destroy(&value_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_uintersect_uassoc(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    size_t array_count = argc - 3;
    PtnArray *source = ptn_internal_expect_array_arg(runtime, "array_uintersect_uassoc", 1, "array", args[0]);
    PtnValue value_callback = ptn_internal_expect_callback_arg(runtime, "array_uintersect_uassoc", argc - 1, NULL, args[argc - 2]);
    PtnValue key_callback = ptn_internal_expect_callback_arg(runtime, "array_uintersect_uassoc", argc, NULL, args[argc - 1]);
    PtnArray **arrays = ptn_array_set_operation_array_args(runtime, "array_uintersect_uassoc", array_count, args);
    PtnValue result = ptn_array_custom_set_operation(
        runtime,
        source,
        array_count,
        arrays,
        PTN_ARRAY_SET_VALUE_CALLBACK,
        value_callback,
        PTN_ARRAY_SET_KEY_CALLBACK,
        key_callback,
        1,
        line
    );
    ptn_value_destroy(&value_callback);
    ptn_value_destroy(&key_callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_array_reduce(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_reduce", 1, "array", args[0]);
    PtnValue callback = ptn_internal_expect_callback_arg(runtime, "array_reduce", 2, "callback", args[1]);
    PtnValue carry = argc >= 3 ? ptn_value_clone_deref(args[2]) : ptn_null();

    for (size_t i = 0; i < array->len; i++) {
        PtnValue callback_args[2] = {
            carry,
            ptn_value_clone_deref(array->entries[i].value)
        };
        carry = ptn_null();
        ptn_value_debug_hide_ref(callback_args[0]);
        PtnValue callback_result = ptn_call_callable(runtime, callback, 2, callback_args, line);
        ptn_value_debug_unhide_ref(callback_args[0]);
        ptn_value_destroy(&callback_args[0]);
        ptn_value_destroy(&callback_args[1]);
        carry = ptn_value_clone_deref(callback_result);
        ptn_value_destroy(&callback_result);
    }

    ptn_value_destroy(&callback);
    return carry;
}

static PtnValue ptn_internal_array_walk(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue value = ptn_value_clone(args[0]);
    PtnValue result = ptn_array_walk_checked(
        runtime,
        NULL,
        &value,
        value,
        args[1],
        argc >= 3,
        argc >= 3 ? args[2] : ptn_null(),
        line
    );
    ptn_value_destroy(&value);
    return result;
}

static PtnValue ptn_internal_array_walk_recursive(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue value = ptn_value_clone(args[0]);
    PtnValue result = ptn_array_walk_recursive_checked(
        runtime,
        NULL,
        &value,
        value,
        args[1],
        argc >= 3,
        argc >= 3 ? args[2] : ptn_null(),
        line
    );
    ptn_value_destroy(&value);
    return result;
}

static PtnArray **ptn_array_map_arrays(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t *max_len_out) {
    size_t array_count = argc - 1;
    PtnArray **arrays = malloc(array_count * sizeof(PtnArray *));
    if (arrays == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t max_len = 0;
    for (size_t i = 0; i < array_count; i++) {
        arrays[i] = ptn_internal_expect_array_arg(runtime, "array_map", i + 2, "array", args[i + 1]);
        if (arrays[i]->len > max_len) {
            max_len = arrays[i]->len;
        }
    }
    *max_len_out = max_len;
    return arrays;
}

static PtnArrayKey ptn_array_map_result_key(PtnArray **arrays, size_t array_count, size_t index) {
    if (array_count == 1) {
        return ptn_array_key_clone(arrays[0]->entries[index].key);
    }
    if (index > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_array_int_key((int64_t)index);
}

static PtnValue ptn_array_map_argument_at(PtnArray *array, size_t index) {
    if (index >= array->len) {
        return ptn_null();
    }
    return ptn_value_clone_deref(array->entries[index].value);
}

static PtnValue ptn_array_map_null_callback_row(PtnArray **arrays, size_t array_count, size_t index) {
    if (array_count == 1) {
        return ptn_array_map_argument_at(arrays[0], index);
    }

    PtnValue row = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < array_count; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            row.as.array,
            ptn_array_int_key((int64_t)i),
            ptn_array_map_argument_at(arrays[i], index)
        );
    }
    return row;
}

static PtnValue ptn_internal_array_map(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int has_callback = ptn_value_deref(args[0]).type != PTN_NULL;
    PtnValue callback = has_callback
        ? ptn_internal_expect_nullable_callback_arg(runtime, "array_map", 1, "callback", args[0])
        : ptn_null();
    size_t max_len = 0;
    PtnArray **arrays = ptn_array_map_arrays(runtime, argc, args, &max_len);
    size_t array_count = argc - 1;
    PtnValue result = ptn_array_from_literal_entries(0, NULL);

    for (size_t i = 0; i < max_len; i++) {
        PtnValue mapped;
        if (has_callback) {
            PtnValue *callback_args = malloc(array_count * sizeof(PtnValue));
            if (callback_args == NULL) {
                ptn_abort_out_of_memory();
            }
            for (size_t arg_index = 0; arg_index < array_count; arg_index++) {
                callback_args[arg_index] = ptn_array_map_argument_at(arrays[arg_index], i);
            }

            PtnValue callback_result = ptn_call_callable(runtime, callback, array_count, callback_args, line);
            for (size_t arg_index = 0; arg_index < array_count; arg_index++) {
                ptn_value_destroy(&callback_args[arg_index]);
            }
            free(callback_args);
            mapped = ptn_value_clone_deref(callback_result);
            ptn_value_destroy(&callback_result);
        } else {
            mapped = ptn_array_map_null_callback_row(arrays, array_count, i);
        }

        ptn_array_set_entry(
            result.as.array,
            ptn_array_map_result_key(arrays, array_count, i),
            mapped
        );
    }

    ptn_value_destroy(&callback);
    free(arrays);
    return result;
}

static PtnValue ptn_internal_in_array(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "in_array", 2, "haystack", args[1]);
    int strict = argc >= 3 && ptn_is_truthy(args[2]);
    for (size_t i = 0; i < array->len; i++) {
        int matched = strict
            ? ptn_compare_identical(args[0], array->entries[i].value)
            : ptn_compare_equal(args[0], array->entries[i].value);
        if (matched) {
            return ptn_bool(1);
        }
    }
    return ptn_bool(0);
}

static PtnValue ptn_internal_array_search(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_search", 2, "haystack", args[1]);
    int strict = argc >= 3 && ptn_is_truthy(args[2]);
    for (size_t i = 0; i < array->len; i++) {
        int matched = strict
            ? ptn_compare_identical(args[0], array->entries[i].value)
            : ptn_compare_equal(args[0], array->entries[i].value);
        if (matched) {
            return ptn_array_key_value(array->entries[i].key);
        }
    }
    return ptn_bool(0);
}

static PtnValue ptn_internal_array_fill(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t start = ptn_value_to_integer(args[0]);
    int64_t count = ptn_value_to_integer(args[1]);
    if (count < 0) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_fill(): Argument #2 ($count) must be greater than or equal to 0"
        );
    }
    if (count > INT32_MAX) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_fill(): Argument #2 ($count) is too large"
        );
    }
    if ((uint64_t)count > PTN_ARRAY_MAX_ALLOC_ENTRIES) {
        ptn_emit_memory_allocation_overflow_error(
            runtime,
            (size_t)count,
            sizeof(PtnArrayEntry),
            sizeof(PtnArray),
            line
        );
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (int64_t i = 0; i < count; i++) {
        if (start > INT64_MAX - i) {
            ptn_value_destroy(&result);
            ptn_throw_exception(
                runtime,
                "Error",
                "Cannot add element to the array as the next element is already occupied"
            );
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key(start + i),
            ptn_value_clone(args[2])
        );
    }
    return result;
}

static PtnArrayKey ptn_array_key_from_key_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
    }

    PtnStringOperand string = ptn_value_to_string_operand(value);
    PtnValue key_value = ptn_string_literal(string.data, string.len);
    PtnArrayKey key = ptn_array_key_from_value(key_value);
    ptn_string_operand_free(string);
    return key;
}

static PtnValue ptn_internal_array_combine(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *keys = ptn_internal_expect_array_arg(runtime, "array_combine", 1, "keys", args[0]);
    PtnArray *values = ptn_internal_expect_array_arg(runtime, "array_combine", 2, "values", args[1]);
    if (keys->len != values->len) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements"
        );
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < keys->len; i++) {
        PtnArrayKey key = ptn_array_key_from_key_value(runtime, keys->entries[i].value, line);
        ptn_array_set_entry(result.as.array, key, ptn_value_clone(values->entries[i].value));
    }
    return result;
}

static PtnValue ptn_internal_array_fill_keys(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *keys = ptn_internal_expect_array_arg(runtime, "array_fill_keys", 1, "keys", args[0]);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < keys->len; i++) {
        PtnArrayKey key = ptn_array_key_from_key_value(runtime, keys->entries[i].value, line);
        ptn_array_set_entry(result.as.array, key, ptn_value_clone(args[1]));
    }
    return result;
}

static int ptn_array_callback_predicate_matches(
    PtnRuntime *runtime,
    PtnValue callback,
    PtnArrayEntry *entry,
    size_t line,
    int *matched_out
) {
    PtnValue callback_args[2] = {
        ptn_value_clone_deref(entry->value),
        ptn_array_key_value(entry->key)
    };
    PtnValue callback_result = ptn_call_callable(runtime, callback, 2, callback_args, line);
    ptn_value_destroy(&callback_args[0]);
    ptn_value_destroy(&callback_args[1]);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&callback_result);
        *matched_out = 0;
        return 0;
    }
    *matched_out = ptn_is_truthy(callback_result);
    ptn_value_destroy(&callback_result);
    return 1;
}

static PtnValue ptn_array_predicate_callback_arg(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue callback
) {
    return ptn_internal_expect_callback_arg(runtime, function_name, 2, "callback", callback);
}

static PtnValue ptn_internal_array_all(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_all", 1, "array", args[0]);
    PtnValue callback = ptn_array_predicate_callback_arg(runtime, "array_all", args[1]);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&callback);
        return ptn_null();
    }

    for (size_t i = 0; i < array->len; i++) {
        int matched = 0;
        if (!ptn_array_callback_predicate_matches(runtime, callback, &array->entries[i], line, &matched)) {
            ptn_value_destroy(&callback);
            return ptn_null();
        }
        if (!matched) {
            ptn_value_destroy(&callback);
            return ptn_bool(0);
        }
    }

    ptn_value_destroy(&callback);
    return ptn_bool(1);
}

static PtnValue ptn_internal_array_any(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_any", 1, "array", args[0]);
    PtnValue callback = ptn_array_predicate_callback_arg(runtime, "array_any", args[1]);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&callback);
        return ptn_null();
    }

    for (size_t i = 0; i < array->len; i++) {
        int matched = 0;
        if (!ptn_array_callback_predicate_matches(runtime, callback, &array->entries[i], line, &matched)) {
            ptn_value_destroy(&callback);
            return ptn_null();
        }
        if (matched) {
            ptn_value_destroy(&callback);
            return ptn_bool(1);
        }
    }

    ptn_value_destroy(&callback);
    return ptn_bool(0);
}

static PtnValue ptn_internal_array_find(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_find", 1, "array", args[0]);
    PtnValue callback = ptn_array_predicate_callback_arg(runtime, "array_find", args[1]);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&callback);
        return ptn_null();
    }

    for (size_t i = 0; i < array->len; i++) {
        int matched = 0;
        if (!ptn_array_callback_predicate_matches(runtime, callback, &array->entries[i], line, &matched)) {
            ptn_value_destroy(&callback);
            return ptn_null();
        }
        if (matched) {
            PtnValue found = ptn_value_clone_deref(array->entries[i].value);
            ptn_value_destroy(&callback);
            return found;
        }
    }

    ptn_value_destroy(&callback);
    return ptn_null();
}

static PtnValue ptn_internal_array_find_key(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_find_key", 1, "array", args[0]);
    PtnValue callback = ptn_array_predicate_callback_arg(runtime, "array_find_key", args[1]);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_value_destroy(&callback);
        return ptn_null();
    }

    for (size_t i = 0; i < array->len; i++) {
        int matched = 0;
        if (!ptn_array_callback_predicate_matches(runtime, callback, &array->entries[i], line, &matched)) {
            ptn_value_destroy(&callback);
            return ptn_null();
        }
        if (matched) {
            PtnValue found = ptn_array_key_value(array->entries[i].key);
            ptn_value_destroy(&callback);
            return found;
        }
    }

    ptn_value_destroy(&callback);
    return ptn_null();
}

static PtnValue ptn_internal_array_filter(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_filter", 1, "array", args[0]);
    PtnValue callback = argc >= 2 ? ptn_value_clone_deref(args[1]) : ptn_null();
    int has_callback = callback.type != PTN_NULL;
    if (has_callback) {
        ptn_value_destroy(&callback);
        callback = ptn_internal_expect_nullable_callback_arg(runtime, "array_filter", 2, "callback", args[1]);
    }
    int64_t mode = argc >= 3 ? ptn_value_to_integer(args[2]) : 0;
    if (mode != 0 && mode != PTN_ARRAY_FILTER_USE_BOTH && mode != PTN_ARRAY_FILTER_USE_KEY) {
        ptn_value_destroy(&callback);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_filter(): Argument #3 ($mode) must be one of ARRAY_FILTER_USE_VALUE, ARRAY_FILTER_USE_KEY, or ARRAY_FILTER_USE_BOTH"
        );
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        int keep = 0;
        if (has_callback) {
            PtnValue callback_args[2];
            size_t callback_argc = 1;
            if (mode == PTN_ARRAY_FILTER_USE_BOTH) {
                callback_argc = 2;
                callback_args[0] = ptn_value_clone_deref(entry->value);
                callback_args[1] = ptn_array_key_value(entry->key);
            } else if (mode == PTN_ARRAY_FILTER_USE_KEY) {
                callback_args[0] = ptn_array_key_value(entry->key);
            } else {
                callback_args[0] = ptn_value_clone_deref(entry->value);
            }

            PtnValue callback_result = ptn_call_callable(runtime, callback, callback_argc, callback_args, line);
            keep = ptn_is_truthy(callback_result);
            ptn_value_destroy(&callback_result);
            for (size_t arg_index = 0; arg_index < callback_argc; arg_index++) {
                ptn_value_destroy(&callback_args[arg_index]);
            }
        } else {
            keep = ptn_is_truthy(ptn_value_deref(entry->value));
        }

        if (keep) {
            ptn_array_set_entry(
                result.as.array,
                ptn_array_key_clone(entry->key),
                ptn_value_clone(ptn_array_reindexing_internal_value(entry->value))
            );
        }
    }

    ptn_value_destroy(&callback);
    return result;
}

static PtnValue ptn_internal_array_unshift(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_unshift", 1, "array", args[0]);
    return ptn_int(ptn_array_unshift_values(array, argc - 1, args + 1));
}

static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);

static int ptn_array_unique_contains_string_value(PtnArray *array, PtnValue value) {
    for (size_t i = 0; i < array->len; i++) {
        if (ptn_array_value_strings_equal(value, array->entries[i].value)) {
            return 1;
        }
    }
    return 0;
}

static PtnValue ptn_internal_array_unique(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_unique", 1, "array", args[0]);
    if (argc >= 2) {
        int64_t flags = ptn_internal_expect_integer_arg(runtime, "array_unique", 2, "flags", args[1], line);
        if (flags != PTN_SORT_STRING) {
            ptn_throw_exception(
                runtime,
                "Error",
                "array_unique() flags are unsupported; default string value comparison is supported"
            );
            return ptn_null();
        }
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *entry = &array->entries[i];
        if (ptn_array_unique_contains_string_value(result.as.array, entry->value)) {
            continue;
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_key_clone(entry->key),
            ptn_value_clone(ptn_array_reindexing_internal_value(entry->value))
        );
    }
    return result;
}

static PtnValue ptn_internal_array_values(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_values", 1, "array", args[0]);
    if (array->len == 0) {
        return ptn_array_from_literal_entries(0, NULL);
    }

    PtnArrayLiteralEntry *entries = malloc(array->len * sizeof(PtnArrayLiteralEntry));
    if (entries == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < array->len; i++) {
        entries[i].has_key = 0;
        entries[i].value = ptn_array_reindexing_internal_value(array->entries[i].value);
    }

    PtnValue result = ptn_array_from_literal_entries(array->len, entries);
    free(entries);
    return result;
}

static PtnValue ptn_internal_array_reverse(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_reverse", 1, "array", args[0]);
    int preserve_keys = argc >= 2 && ptn_is_truthy(args[1]);
    if (array->len == 0) {
        return ptn_array_from_literal_entries(0, NULL);
    }

    PtnArrayLiteralEntry *entries = malloc(array->len * sizeof(PtnArrayLiteralEntry));
    if (entries == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < array->len; i++) {
        PtnArrayEntry *source = &array->entries[array->len - i - 1];
        entries[i].has_key = preserve_keys || source->key.type == PTN_ARRAY_KEY_STRING;
        entries[i].key = entries[i].has_key ? ptn_array_key_value(source->key) : ptn_null();
        entries[i].value = ptn_array_reindexing_internal_value(source->value);
    }

    PtnValue result = ptn_array_from_literal_entries(array->len, entries);
    for (size_t i = 0; i < array->len; i++) {
        if (entries[i].has_key) {
            ptn_value_destroy(&entries[i].key);
        }
    }
    free(entries);
    return result;
}

static int ptn_array_slice_string_to_integer(PtnString string, int64_t *integer_out) {
    const char *data = (const char *)string.data;
    const char *limit = data + string.len;
    const char *start = data;
    while (start < limit && isspace((unsigned char)*start)) {
        start++;
    }
    if (start >= limit) {
        return 0;
    }

    char *end = NULL;
    errno = 0;
    double number = strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (end < limit && isspace((unsigned char)*end)) {
        end++;
    }
    if (end != limit) {
        return 0;
    }
    if (number > (double)INT64_MAX) {
        *integer_out = INT64_MAX;
        return 1;
    }
    if (number < (double)INT64_MIN) {
        *integer_out = INT64_MIN;
        return 1;
    }
    *integer_out = (int64_t)number;
    return 1;
}

static int64_t ptn_array_slice_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    int nullable,
    int *is_null
) {
    value = ptn_value_deref(value);
    if (is_null != NULL) {
        *is_null = 0;
    }

    switch (value.type) {
        case PTN_NULL:
            if (nullable) {
                if (is_null != NULL) {
                    *is_null = 1;
                }
                return 0;
            }
            return 0;
        case PTN_BOOL:
            return value.as.boolean ? 1 : 0;
        case PTN_INT:
            return value.as.integer;
        case PTN_FLOAT:
            return (int64_t)value.as.floating;
        case PTN_STRING: {
            int64_t integer = 0;
            if (ptn_array_slice_string_to_integer(value.as.string, &integer)) {
                return integer;
            }
            break;
        }
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            break;
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be of type %s, %s given",
        function_name,
        position,
        argument_name,
        nullable ? "?int" : "int",
        ptn_offset_container_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return 0;
}

static size_t ptn_array_slice_negative_distance(int64_t value, size_t limit) {
    uint64_t distance = value == INT64_MIN
        ? (uint64_t)INT64_MAX + 1ULL
        : (uint64_t)(-value);
    if (distance > (uint64_t)limit) {
        return limit;
    }
    return (size_t)distance;
}

static size_t ptn_array_slice_start_offset(size_t array_len, int64_t offset) {
    if (offset >= 0) {
        if ((uint64_t)offset > (uint64_t)array_len) {
            return array_len;
        }
        return (size_t)offset;
    }
    size_t distance = ptn_array_slice_negative_distance(offset, array_len);
    return array_len - distance;
}

static size_t ptn_array_slice_count(size_t array_len, size_t start, int has_length, int64_t length) {
    size_t available = array_len - start;
    if (!has_length) {
        return available;
    }
    if (length >= 0) {
        if ((uint64_t)length > (uint64_t)available) {
            return available;
        }
        return (size_t)length;
    }
    size_t drop = ptn_array_slice_negative_distance(length, available);
    return available - drop;
}

static PtnValue ptn_array_replacement_value(PtnValue value);

static PtnArrayKey ptn_array_splice_result_key(PtnArray *array, PtnArrayKey source_key) {
    if (source_key.type == PTN_ARRAY_KEY_STRING) {
        return ptn_array_key_clone(source_key);
    }
    return ptn_array_int_key(array->next_auto_key);
}

static void ptn_array_splice_append_source_entry(PtnArray *target, PtnArrayEntry *source) {
    ptn_array_set_entry(
        target,
        ptn_array_splice_result_key(target, source->key),
        ptn_array_replacement_value(source->value)
    );
}

static void ptn_array_splice_append_replacement_value(PtnArray *target, PtnValue value) {
    ptn_array_set_entry(
        target,
        ptn_array_int_key(target->next_auto_key),
        ptn_array_replacement_value(value)
    );
}

static void ptn_array_adopt_storage(PtnArray *target, PtnArray *source) {
    for (size_t i = 0; i < target->len; i++) {
        ptn_array_key_free(target->entries[i].key);
        ptn_value_destroy(&target->entries[i].value);
    }
    free(target->index_slots);
    free(target->entries);

    target->len = source->len;
    target->capacity = source->capacity;
    target->entries = source->entries;
    target->index_slots = source->index_slots;
    target->index_capacity = source->index_capacity;
    target->next_auto_key = source->next_auto_key;
    target->current_index = source->current_index <= source->len ? source->current_index : source->len;

    source->len = 0;
    source->capacity = 0;
    source->entries = NULL;
    source->index_slots = NULL;
    source->index_capacity = 0;
    source->next_auto_key = 0;
    source->current_index = 0;
}

static PtnValue ptn_array_splice_values(
    PtnArray *array,
    int64_t offset,
    int has_length,
    int64_t length,
    int has_replacement,
    PtnValue replacement
) {
    size_t start = ptn_array_slice_start_offset(array->len, offset);
    size_t count = ptn_array_slice_count(array->len, start, has_length, length);
    PtnValue removed = ptn_array_from_literal_entries(0, NULL);
    PtnValue rebuilt = ptn_array_from_literal_entries(0, NULL);
    PtnArray *replacement_array = NULL;
    PtnValue replacement_value = ptn_value_deref(replacement);
    if (has_replacement && replacement_value.type == PTN_ARRAY) {
        replacement_array = replacement_value.as.array;
    }

    for (size_t i = 0; i < start; i++) {
        ptn_array_splice_append_source_entry(rebuilt.as.array, &array->entries[i]);
    }

    for (size_t i = 0; i < count; i++) {
        ptn_array_splice_append_source_entry(removed.as.array, &array->entries[start + i]);
    }

    if (replacement_array != NULL) {
        for (size_t i = 0; i < replacement_array->len; i++) {
            ptn_array_splice_append_replacement_value(
                rebuilt.as.array,
                replacement_array->entries[i].value
            );
        }
    } else if (has_replacement && replacement_value.type != PTN_NULL) {
        ptn_array_splice_append_replacement_value(rebuilt.as.array, replacement);
    }

    for (size_t i = start + count; i < array->len; i++) {
        ptn_array_splice_append_source_entry(rebuilt.as.array, &array->entries[i]);
    }

    ptn_array_adopt_storage(array, rebuilt.as.array);
    free(rebuilt.as.array);
    return removed;
}

static PtnValue ptn_internal_array_slice(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_slice", 1, "array", args[0]);
    int64_t offset = ptn_array_slice_integer_arg(runtime, "array_slice", 2, "offset", args[1], 0, NULL);
    int length_is_null = 0;
    int64_t length = argc >= 3
        ? ptn_array_slice_integer_arg(runtime, "array_slice", 3, "length", args[2], 1, &length_is_null)
        : 0;
    int has_length = argc >= 3 && !length_is_null;
    int preserve_keys = argc >= 4 && ptn_is_truthy(args[3]);
    size_t start = ptn_array_slice_start_offset(array->len, offset);
    size_t count = ptn_array_slice_count(array->len, start, has_length, length);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);

    for (size_t i = 0; i < count; i++) {
        PtnArrayEntry *source = &array->entries[start + i];
        PtnArrayKey key = (preserve_keys || source->key.type == PTN_ARRAY_KEY_STRING)
            ? ptn_array_key_clone(source->key)
            : ptn_array_int_key(result.as.array->next_auto_key);
        ptn_array_set_entry(
            result.as.array,
            key,
            ptn_value_clone(ptn_array_reindexing_internal_value(source->value))
        );
    }

    return result;
}

static PtnValue ptn_internal_array_splice(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_splice", 1, "array", args[0]);
    if (args[0].type == PTN_REFERENCE && args[0].as.reference->value.type == PTN_ARRAY) {
        array = ptn_array_detach_value(&args[0].as.reference->value);
    }
    int64_t offset = ptn_array_slice_integer_arg(runtime, "array_splice", 2, "offset", args[1], 0, NULL);
    int length_is_null = 0;
    int64_t length = argc >= 3
        ? ptn_array_slice_integer_arg(runtime, "array_splice", 3, "length", args[2], 1, &length_is_null)
        : 0;
    int has_length = argc >= 3 && !length_is_null;
    PtnValue replacement = argc >= 4 ? args[3] : ptn_null();
    (void)line;
    return ptn_array_splice_values(
        array,
        offset,
        has_length,
        length,
        argc >= 4,
        replacement
    );
}

static PtnValue ptn_internal_range(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    int64_t start = ptn_value_to_integer(args[0]);
    int64_t end = ptn_value_to_integer(args[1]);
    int64_t step_value = argc >= 3 ? ptn_value_to_integer(args[2]) : 1;
    uint64_t step = step_value < 0 ? (uint64_t)(-(step_value + 1)) + 1 : (uint64_t)step_value;
    uint64_t distance = start <= end
        ? (uint64_t)end - (uint64_t)start
        : (uint64_t)start - (uint64_t)end;
    if (step == 0 || (distance != 0 && step > distance)) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "range(): Argument #3 ($step) must not exceed the specified range"
        );
    }

    uint64_t count = distance == 0 ? 1 : distance / step + 1;
    if (count > (uint64_t)INT64_MAX || count > (uint64_t)SIZE_MAX) {
        ptn_abort_out_of_memory();
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    int ascending = start <= end;
    int64_t current = start;
    for (uint64_t i = 0; i < count; i++) {
        ptn_array_set_entry(result.as.array, ptn_array_int_key((int64_t)i), ptn_int(current));
        if (i + 1 == count) {
            break;
        }
        if (step > (uint64_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        int64_t signed_step = (int64_t)step;
        if (ascending) {
            if (current > INT64_MAX - signed_step) {
                ptn_abort_out_of_memory();
            }
            current += signed_step;
        } else {
            if (current < INT64_MIN + signed_step) {
                ptn_abort_out_of_memory();
            }
            current -= signed_step;
        }
    }
    return result;
}

static PtnArrayKey ptn_array_pad_source_key(PtnArrayKey source_key, int preserve_integer_keys, int64_t *next_integer_key) {
    if (source_key.type == PTN_ARRAY_KEY_STRING || preserve_integer_keys) {
        return ptn_array_key_clone(source_key);
    }
    if (*next_integer_key == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    PtnArrayKey key = ptn_array_int_key(*next_integer_key);
    *next_integer_key += 1;
    return key;
}

static void ptn_array_pad_copy_source(
    PtnArray *target,
    PtnArray *source,
    int preserve_integer_keys,
    int64_t *next_integer_key
) {
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        ptn_array_set_entry(
            target,
            ptn_array_pad_source_key(entry->key, preserve_integer_keys, next_integer_key),
            ptn_value_clone(ptn_array_reindexing_internal_value(entry->value))
        );
    }
}

static void ptn_array_pad_append_values(
    PtnArray *target,
    size_t count,
    PtnValue value,
    int64_t *next_integer_key
) {
    for (size_t i = 0; i < count; i++) {
        if (*next_integer_key == INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            target,
            ptn_array_int_key(*next_integer_key),
            ptn_value_clone(value)
        );
        *next_integer_key += 1;
    }
}

static PtnValue ptn_internal_array_pad(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_pad", 1, "array", args[0]);
    int64_t length = ptn_value_to_integer(args[1]);
    uint64_t requested = length < 0 ? (uint64_t)(-(length + 1)) + 1 : (uint64_t)length;
    if (requested > 1048576ULL) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "array_pad(): Argument #2 ($length) must not exceed the maximum allowed array size"
        );
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (requested <= array->len) {
        int64_t next_integer_key = 0;
        ptn_array_pad_copy_source(result.as.array, array, 1, &next_integer_key);
        return result;
    }

    size_t padding = (size_t)(requested - array->len);
    int64_t next_integer_key = 0;
    if (length < 0) {
        ptn_array_pad_append_values(result.as.array, padding, args[2], &next_integer_key);
        ptn_array_pad_copy_source(result.as.array, array, 0, &next_integer_key);
    } else {
        ptn_array_pad_copy_source(result.as.array, array, 0, &next_integer_key);
        ptn_array_pad_append_values(result.as.array, padding, args[2], &next_integer_key);
    }
    return result;
}

static void ptn_array_merge_append(PtnRuntime *runtime, PtnArray *target, PtnValue value) {
    if (!ptn_array_append_key_available(runtime, target)) {
        return;
    }
    PtnArrayKey key = ptn_array_int_key(target->next_auto_key);
    ptn_array_set_entry(target, key, ptn_value_clone(ptn_array_reindexing_internal_value(value)));
}

static void ptn_array_merge_into(PtnRuntime *runtime, PtnArray *target, PtnArray *source) {
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        if (entry->key.type == PTN_ARRAY_KEY_INT) {
            ptn_array_merge_append(runtime, target, entry->value);
        } else {
            ptn_array_set_entry(
                target,
                ptn_array_key_clone(entry->key),
                ptn_value_clone(ptn_array_reindexing_internal_value(entry->value))
            );
        }
    }
}

static PtnValue ptn_internal_array_merge(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < argc; i++) {
        PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_merge", i + 1, "arrays", args[i]);
        ptn_array_merge_into(runtime, result.as.array, array);
    }
    (void)line;
    return result;
}

static void ptn_array_merge_recursive_into(PtnRuntime *runtime, PtnArray *target, PtnArray *source);

static void ptn_array_merge_recursive_append(PtnRuntime *runtime, PtnArray *target, PtnValue value) {
    if (!ptn_array_append_key_available(runtime, target)) {
        return;
    }
    PtnArrayKey key = ptn_array_int_key(target->next_auto_key);
    ptn_array_set_entry(target, key, ptn_value_clone_deref(value));
}

static void ptn_array_merge_recursive_collision(PtnRuntime *runtime, PtnArrayEntry *entry, PtnValue incoming) {
    PtnValue *entry_value = entry->value.type == PTN_REFERENCE
        ? &entry->value.as.reference->value
        : &entry->value;
    PtnValue existing = ptn_value_deref(*entry_value);
    PtnValue incoming_value = ptn_value_deref(incoming);

    if (existing.type == PTN_ARRAY) {
        PtnArray *target = ptn_array_detach_value(entry_value);
        if (target == NULL) {
            return;
        }
        if (incoming_value.type == PTN_ARRAY) {
            ptn_array_merge_recursive_into(runtime, target, incoming_value.as.array);
        } else {
            ptn_array_merge_recursive_append(runtime, target, incoming);
        }
        return;
    }

    PtnValue merged = ptn_array_from_literal_entries(0, NULL);
    ptn_array_merge_recursive_append(runtime, merged.as.array, existing);
    if (incoming_value.type == PTN_ARRAY) {
        ptn_array_merge_recursive_into(runtime, merged.as.array, incoming_value.as.array);
    } else {
        ptn_array_merge_recursive_append(runtime, merged.as.array, incoming);
    }
    ptn_value_destroy(entry_value);
    *entry_value = merged;
}

static void ptn_array_merge_recursive_entry(PtnRuntime *runtime, PtnArray *target, PtnArrayKey key, PtnValue value) {
    if (key.type == PTN_ARRAY_KEY_INT) {
        ptn_array_merge_recursive_append(runtime, target, value);
        return;
    }

    PtnArrayEntry *entry = ptn_array_entry_for_key(target, key);
    if (entry == NULL) {
        ptn_array_set_entry(target, ptn_array_key_clone(key), ptn_value_clone_deref(value));
        return;
    }
    ptn_array_merge_recursive_collision(runtime, entry, value);
}

static void ptn_array_merge_recursive_into(PtnRuntime *runtime, PtnArray *target, PtnArray *source) {
    for (size_t i = 0; i < source->len; i++) {
        ptn_array_merge_recursive_entry(runtime, target, source->entries[i].key, source->entries[i].value);
    }
}

static PtnValue ptn_internal_array_merge_recursive(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < argc; i++) {
        PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_merge_recursive", i + 1, "array", args[i]);
        ptn_array_merge_recursive_into(runtime, result.as.array, array);
    }
    (void)line;
    return result;
}

static PtnValue ptn_array_replacement_value(PtnValue value) {
    if (value.type == PTN_REFERENCE && value.as.reference->refcount <= 1) {
        return ptn_value_clone_deref(value);
    }
    return ptn_value_clone(value);
}

static void ptn_array_replace_into(PtnArray *target, PtnArray *source) {
    for (size_t i = 0; i < source->len; i++) {
        PtnArrayEntry *entry = &source->entries[i];
        ptn_array_set_entry(
            target,
            ptn_array_key_clone(entry->key),
            ptn_array_replacement_value(entry->value)
        );
    }
}

static PtnValue ptn_internal_array_replace(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < argc; i++) {
        PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_replace", i + 1, "array", args[i]);
        ptn_array_replace_into(result.as.array, array);
    }
    (void)line;
    return result;
}

static void ptn_array_replace_recursive_into(PtnArray *target, PtnArray *source);

static void ptn_array_replace_recursive_entry(PtnArray *target, PtnArrayKey key, PtnValue value) {
    PtnArrayEntry *entry = ptn_array_entry_for_key(target, key);
    if (entry == NULL) {
        ptn_array_set_entry(target, ptn_array_key_clone(key), ptn_array_replacement_value(value));
        return;
    }

    PtnValue *entry_value = entry->value.type == PTN_REFERENCE
        ? &entry->value.as.reference->value
        : &entry->value;
    PtnValue existing = ptn_value_deref(*entry_value);
    PtnValue incoming = ptn_value_deref(value);
    if (existing.type == PTN_ARRAY && incoming.type == PTN_ARRAY) {
        PtnValue separated = ptn_value_clone_deref(existing);
        ptn_value_destroy(&entry->value);
        entry->value = separated;
        PtnArray *target_child = ptn_array_detach_value(&entry->value);
        if (target_child != NULL) {
            ptn_array_replace_recursive_into(target_child, incoming.as.array);
        }
        return;
    }

    ptn_value_destroy(entry_value);
    *entry_value = ptn_array_replacement_value(value);
}

static void ptn_array_replace_recursive_into(PtnArray *target, PtnArray *source) {
    for (size_t i = 0; i < source->len; i++) {
        ptn_array_replace_recursive_entry(target, source->entries[i].key, source->entries[i].value);
    }
}

static PtnValue ptn_internal_array_replace_recursive(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < argc; i++) {
        PtnArray *array = ptn_internal_expect_array_arg(runtime, "array_replace_recursive", i + 1, "array", args[i]);
        ptn_array_replace_recursive_into(result.as.array, array);
    }
    (void)line;
    return result;
}

static PtnValue ptn_internal_current(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "current", 1, "array", args[0]);
    return ptn_array_current_value(array);
}

static PtnValue ptn_internal_key(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "key", 1, "array", args[0]);
    return ptn_array_current_key_value(array);
}

static PtnValue ptn_internal_next(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "next", 1, "array", args[0]);
    return ptn_array_next_value(array);
}

static PtnValue ptn_internal_end(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "end", 1, "array", args[0]);
    return ptn_array_end_value(array);
}

static PtnValue ptn_internal_prev(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "prev", 1, "array", args[0]);
    return ptn_array_prev_value(array);
}

static PtnValue ptn_internal_reset(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnArray *array = ptn_internal_expect_array_arg(runtime, "reset", 1, "array", args[0]);
    return ptn_array_reset_value(array);
}

static PtnStringOperand ptn_internal_expect_string_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);
static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);
static const char *ptn_internal_string_arg_type_name(PtnValue value);
static double ptn_value_to_double(PtnValue value);

static PtnValue ptn_internal_strlen(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "strlen", 1, "string", args[0], line);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_int((int64_t)len);
}

static size_t ptn_explode_find_separator(
    PtnStringOperand string,
    PtnStringOperand separator,
    size_t start
) {
    if (separator.len == 0 || start > string.len || separator.len > string.len - start) {
        return SIZE_MAX;
    }

    for (size_t offset = start; offset <= string.len - separator.len; offset++) {
        if (memcmp(string.data + offset, separator.data, separator.len) == 0) {
            return offset;
        }
    }
    return SIZE_MAX;
}

static size_t ptn_explode_piece_count(PtnStringOperand string, PtnStringOperand separator) {
    size_t count = 1;
    size_t start = 0;
    for (;;) {
        size_t offset = ptn_explode_find_separator(string, separator, start);
        if (offset == SIZE_MAX) {
            return count;
        }
        if (count == SIZE_MAX) {
            ptn_abort_out_of_memory();
        }
        count++;
        start = offset + separator.len;
    }
}

static size_t ptn_explode_negative_limit_omit_count(int64_t limit) {
    if (limit == INT64_MIN) {
        return SIZE_MAX;
    }
    return (size_t)(-limit);
}

static void ptn_explode_append_segment(
    PtnValue *result,
    PtnStringOperand string,
    size_t start,
    size_t len,
    size_t index
) {
    if (index > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    char *segment = ptn_duplicate_string_len(string.data + start, len);
    ptn_array_set_entry(
        result->as.array,
        ptn_array_int_key((int64_t)index),
        ptn_owned_string_len(segment, len)
    );
}

static PtnValue ptn_internal_explode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand separator = ptn_internal_expect_string_arg(runtime, "explode", 1, "separator", args[0], line);
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "explode", 2, "string", args[1], line);
    if (separator.len == 0) {
        ptn_string_operand_free(separator);
        ptn_string_operand_free(string);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "explode(): Argument #1 ($separator) must not be empty, use str_split() to split a string into characters"
        );
        return ptn_null();
    }

    int64_t limit = argc >= 3 ? ptn_value_to_integer(args[2]) : INT64_MAX;
    if (limit == 0) {
        limit = 1;
    }

    size_t piece_count = ptn_explode_piece_count(string, separator);
    size_t emit_count = piece_count;
    if (limit > 0 && (uint64_t)limit < (uint64_t)piece_count) {
        emit_count = (size_t)limit;
    } else if (limit < 0) {
        size_t omit_count = ptn_explode_negative_limit_omit_count(limit);
        emit_count = omit_count >= piece_count ? 0 : piece_count - omit_count;
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    size_t start = 0;
    for (size_t index = 0; index < emit_count; index++) {
        size_t segment_start = start;
        size_t segment_len = string.len - segment_start;
        if (limit < 0 || index + 1 < emit_count) {
            size_t offset = ptn_explode_find_separator(string, separator, segment_start);
            if (offset != SIZE_MAX) {
                segment_len = offset - segment_start;
                start = offset + separator.len;
            } else {
                start = string.len;
            }
        } else {
            start = string.len;
        }
        ptn_explode_append_segment(&result, string, segment_start, segment_len, index);
    }

    ptn_string_operand_free(separator);
    ptn_string_operand_free(string);
    return result;
}

static PtnValue ptn_internal_implode_named(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    PtnStringOperand separator = ptn_string_operand_borrowed("");
    PtnArray *array = NULL;
    if (argc == 1) {
        array = ptn_internal_expect_array_arg(runtime, function_name, 1, "array", args[0]);
    } else {
        separator = ptn_internal_expect_string_arg(runtime, function_name, 1, "separator", args[0], line);
        array = ptn_internal_expect_array_arg(runtime, function_name, 2, "array", args[1]);
    }

    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    for (size_t i = 0; i < array->len; i++) {
        if (i > 0) {
            ptn_string_buffer_append_len(&buffer, separator.data, separator.len);
        }

        PtnValue entry_value = ptn_value_deref(array->entries[i].value);
        if (entry_value.type == PTN_ARRAY) {
            ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
        }
        PtnStringOperand part = ptn_value_to_string_operand_with_runtime(runtime, entry_value, line);
        ptn_string_buffer_append_len(&buffer, part.data, part.len);
        ptn_string_operand_free(part);
    }

    if (argc >= 2) {
        ptn_string_operand_free(separator);
    }
    return ptn_owned_string_len(buffer.data, buffer.len);
}

static PtnValue ptn_internal_implode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_implode_named(runtime, "implode", argc, args, line);
}

static PtnValue ptn_internal_join(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_implode_named(runtime, "join", argc, args, line);
}

typedef struct {
    int left_adjust;
    int show_sign;
    int space_sign;
    int zero_pad;
    int alternate;
    int has_width;
    int width;
    int has_precision;
    int precision;
} PtnSprintfSpec;

static void ptn_sprintf_append_repeated(PtnStringBuffer *buffer, char byte, size_t count) {
    for (size_t i = 0; i < count; i++) {
        ptn_string_buffer_append_char(buffer, byte);
    }
}

static int ptn_sprintf_parse_decimal(const char *data, size_t len, size_t *offset) {
    int value = 0;
    while (*offset < len && isdigit((unsigned char)data[*offset])) {
        int digit = data[*offset] - '0';
        if (value > (INT_MAX - digit) / 10) {
            ptn_abort_out_of_memory();
        }
        value = value * 10 + digit;
        (*offset)++;
    }
    return value;
}

static void ptn_sprintf_build_numeric_format(char *format, size_t format_size, const PtnSprintfSpec *spec, char conversion, int use_long_long) {
    char *cursor = format;
    char *end = format + format_size;
    if (cursor >= end) {
        ptn_abort_out_of_memory();
    }
    *cursor++ = '%';
    if (spec->left_adjust && cursor < end) {
        *cursor++ = '-';
    }
    if (spec->show_sign && cursor < end) {
        *cursor++ = '+';
    }
    if (spec->space_sign && cursor < end) {
        *cursor++ = ' ';
    }
    if (spec->zero_pad && cursor < end) {
        *cursor++ = '0';
    }
    if (spec->alternate && cursor < end) {
        *cursor++ = '#';
    }
    if (spec->has_width) {
        int written = snprintf(cursor, (size_t)(end - cursor), "%d", spec->width);
        if (written < 0 || written >= end - cursor) {
            ptn_abort_out_of_memory();
        }
        cursor += written;
    }
    if (spec->has_precision) {
        if (cursor >= end) {
            ptn_abort_out_of_memory();
        }
        *cursor++ = '.';
        int written = snprintf(cursor, (size_t)(end - cursor), "%d", spec->precision);
        if (written < 0 || written >= end - cursor) {
            ptn_abort_out_of_memory();
        }
        cursor += written;
    }
    if (use_long_long) {
        if (end - cursor < 3) {
            ptn_abort_out_of_memory();
        }
        *cursor++ = 'l';
        *cursor++ = 'l';
    }
    if (end - cursor < 2) {
        ptn_abort_out_of_memory();
    }
    *cursor++ = conversion;
    *cursor = '\0';
}

static void ptn_sprintf_append_snprintf_signed(PtnStringBuffer *buffer, const char *format, long long value) {
    int needed = snprintf(NULL, 0, format, value);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *formatted = malloc((size_t)needed + 1);
    if (formatted == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(formatted, (size_t)needed + 1, format, value);
    if (written != needed) {
        free(formatted);
        ptn_abort_out_of_memory();
    }
    ptn_string_buffer_append_len(buffer, formatted, (size_t)needed);
    free(formatted);
}

static void ptn_sprintf_append_snprintf_unsigned(PtnStringBuffer *buffer, const char *format, unsigned long long value) {
    int needed = snprintf(NULL, 0, format, value);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *formatted = malloc((size_t)needed + 1);
    if (formatted == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(formatted, (size_t)needed + 1, format, value);
    if (written != needed) {
        free(formatted);
        ptn_abort_out_of_memory();
    }
    ptn_string_buffer_append_len(buffer, formatted, (size_t)needed);
    free(formatted);
}

static void ptn_sprintf_append_snprintf_double(PtnStringBuffer *buffer, const char *format, double value) {
    int needed = snprintf(NULL, 0, format, value);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *formatted = malloc((size_t)needed + 1);
    if (formatted == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(formatted, (size_t)needed + 1, format, value);
    if (written != needed) {
        free(formatted);
        ptn_abort_out_of_memory();
    }
    ptn_string_buffer_append_len(buffer, formatted, (size_t)needed);
    free(formatted);
}

static void ptn_sprintf_append_string(PtnStringBuffer *buffer, PtnStringOperand string, const PtnSprintfSpec *spec) {
    size_t len = string.len;
    if (spec->has_precision && spec->precision >= 0 && (size_t)spec->precision < len) {
        len = (size_t)spec->precision;
    }
    size_t width = spec->has_width && spec->width > 0 ? (size_t)spec->width : 0;
    size_t padding = width > len ? width - len : 0;
    if (!spec->left_adjust) {
        ptn_sprintf_append_repeated(buffer, ' ', padding);
    }
    ptn_string_buffer_append_len(buffer, string.data, len);
    if (spec->left_adjust) {
        ptn_sprintf_append_repeated(buffer, ' ', padding);
    }
}

static void ptn_sprintf_append_char(PtnStringBuffer *buffer, int64_t value, const PtnSprintfSpec *spec) {
    size_t width = spec->has_width && spec->width > 0 ? (size_t)spec->width : 0;
    size_t padding = width > 1 ? width - 1 : 0;
    char pad = spec->zero_pad && !spec->left_adjust ? '0' : ' ';
    if (!spec->left_adjust) {
        ptn_sprintf_append_repeated(buffer, pad, padding);
    }
    ptn_string_buffer_append_char(buffer, (char)((unsigned char)value));
    if (spec->left_adjust) {
        ptn_sprintf_append_repeated(buffer, ' ', padding);
    }
}

static void ptn_sprintf_append_binary(PtnStringBuffer *buffer, uint64_t value, const PtnSprintfSpec *spec) {
    char digits[65];
    size_t len = 0;
    if (value == 0) {
        digits[len++] = '0';
    } else {
        while (value != 0) {
            digits[len++] = (value & 1) ? '1' : '0';
            value >>= 1;
        }
        for (size_t i = 0; i < len / 2; i++) {
            char tmp = digits[i];
            digits[i] = digits[len - 1 - i];
            digits[len - 1 - i] = tmp;
        }
    }

    size_t precision_padding = 0;
    if (spec->has_precision && spec->precision > 0 && (size_t)spec->precision > len) {
        precision_padding = (size_t)spec->precision - len;
    }
    size_t value_len = len + precision_padding;
    size_t width = spec->has_width && spec->width > 0 ? (size_t)spec->width : 0;
    size_t width_padding = width > value_len ? width - value_len : 0;
    char pad = spec->zero_pad && !spec->left_adjust && !spec->has_precision ? '0' : ' ';
    if (!spec->left_adjust) {
        ptn_sprintf_append_repeated(buffer, pad, width_padding);
    }
    ptn_sprintf_append_repeated(buffer, '0', precision_padding);
    ptn_string_buffer_append_len(buffer, digits, len);
    if (spec->left_adjust) {
        ptn_sprintf_append_repeated(buffer, ' ', width_padding);
    }
}

static void ptn_sprintf_throw_unknown_specifier(PtnRuntime *runtime, char specifier) {
    char message[96];
    int written = snprintf(message, sizeof(message), "Unknown format specifier \"%c\"", specifier);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "ValueError", message);
}

static PtnValue ptn_internal_sprintf_named(PtnRuntime *runtime, const char *function_name, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand format = ptn_internal_expect_string_arg(runtime, function_name, 1, "format", args[0], line);
    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    size_t arg_index = 1;

    for (size_t i = 0; i < format.len; i++) {
        char byte = format.data[i];
        if (byte != '%') {
            ptn_string_buffer_append_char(&output, byte);
            continue;
        }
        if (i + 1 < format.len && format.data[i + 1] == '%') {
            ptn_string_buffer_append_char(&output, '%');
            i++;
            continue;
        }

        i++;
        PtnSprintfSpec spec = {0};
        while (i < format.len) {
            switch (format.data[i]) {
                case '-':
                    spec.left_adjust = 1;
                    i++;
                    continue;
                case '+':
                    spec.show_sign = 1;
                    i++;
                    continue;
                case ' ':
                    spec.space_sign = 1;
                    i++;
                    continue;
                case '0':
                    spec.zero_pad = 1;
                    i++;
                    continue;
                case '#':
                    spec.alternate = 1;
                    i++;
                    continue;
                default:
                    break;
            }
            break;
        }

        if (i < format.len && isdigit((unsigned char)format.data[i])) {
            spec.has_width = 1;
            spec.width = ptn_sprintf_parse_decimal(format.data, format.len, &i);
        }
        if (i < format.len && format.data[i] == '.') {
            i++;
            spec.has_precision = 1;
            spec.precision = ptn_sprintf_parse_decimal(format.data, format.len, &i);
        }
        while (i < format.len && strchr("hlLjzt", format.data[i]) != NULL) {
            i++;
        }
        if (i >= format.len) {
            ptn_string_operand_free(format);
            free(output.data);
            ptn_throw_exception(runtime, "ValueError", "Missing format specifier at end of string");
        }

        char conversion = format.data[i];
        if (arg_index >= argc) {
            ptn_string_operand_free(format);
            free(output.data);
            ptn_emit_argument_count_error(&runtime->diagnostics, function_name, arg_index + 1, argc);
            exit(255);
        }
        PtnValue arg = args[arg_index++];

        switch (conversion) {
            case 's': {
                PtnValue value = ptn_value_deref(arg);
                if (value.type == PTN_ARRAY) {
                    ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
                }
                PtnStringOperand string = ptn_value_to_string_operand_with_runtime(runtime, value, line);
                ptn_sprintf_append_string(&output, string, &spec);
                ptn_string_operand_free(string);
                break;
            }
            case 'c':
                ptn_sprintf_append_char(&output, ptn_value_to_integer(arg), &spec);
                break;
            case 'b':
                ptn_sprintf_append_binary(&output, (uint64_t)ptn_value_to_integer(arg), &spec);
                break;
            case 'd':
            case 'i': {
                char c_format[64];
                ptn_sprintf_build_numeric_format(c_format, sizeof(c_format), &spec, conversion, 1);
                ptn_sprintf_append_snprintf_signed(&output, c_format, (long long)ptn_value_to_integer(arg));
                break;
            }
            case 'u':
            case 'o':
            case 'x':
            case 'X': {
                char c_format[64];
                ptn_sprintf_build_numeric_format(c_format, sizeof(c_format), &spec, conversion, 1);
                ptn_sprintf_append_snprintf_unsigned(&output, c_format, (unsigned long long)((uint64_t)ptn_value_to_integer(arg)));
                break;
            }
            case 'f':
            case 'F':
            case 'e':
            case 'E':
            case 'g':
            case 'G': {
                char c_format[64];
                ptn_sprintf_build_numeric_format(c_format, sizeof(c_format), &spec, conversion, 0);
                ptn_sprintf_append_snprintf_double(&output, c_format, ptn_value_to_double(arg));
                break;
            }
            default:
                ptn_string_operand_free(format);
                free(output.data);
                ptn_sprintf_throw_unknown_specifier(runtime, conversion);
        }
    }

    ptn_string_operand_free(format);
    return ptn_owned_string_len(output.data, output.len);
}

static PtnValue ptn_internal_sprintf(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_sprintf_named(runtime, "sprintf", argc, args, line);
}

static PtnValue ptn_internal_printf(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue formatted = ptn_internal_sprintf_named(runtime, "printf", argc, args, line);
    if (ptn_value_deref(formatted).type != PTN_STRING) {
        return formatted;
    }
    PtnValue string_value = ptn_value_deref(formatted);
    fwrite(string_value.as.string.data, 1, string_value.as.string.len, stdout);
    if (string_value.as.string.len > (size_t)INT64_MAX) {
        ptn_value_drop(&formatted);
        ptn_abort_out_of_memory();
    }
    int64_t len = (int64_t)string_value.as.string.len;
    ptn_value_drop(&formatted);
    return ptn_int(len);
}

static PtnValue ptn_internal_write_formatted_stream(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue stream_value,
    PtnValue formatted,
    size_t line
) {
    PtnValue stream = ptn_value_deref(stream_value);
    if (stream.type != PTN_RESOURCE) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #1 ($stream) must be of type resource, %s given",
            function_name,
            ptn_offset_container_type_name(stream)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_value_drop(&formatted);
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    if (stream.as.resource->stream == NULL) {
        ptn_value_drop(&formatted);
        char message[96];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): supplied resource is not a valid stream resource",
            function_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }

    PtnValue string_value = ptn_value_deref(formatted);
    if (string_value.type != PTN_STRING) {
        return formatted;
    }
    size_t written = fwrite(string_value.as.string.data, 1, string_value.as.string.len, stream.as.resource->stream);
    if (written != string_value.as.string.len) {
        ptn_value_drop(&formatted);
        ptn_emit_warning(&runtime->diagnostics, "fprintf(): failed to write formatted stream output", line);
        return ptn_bool(0);
    }
    if (string_value.as.string.len > (size_t)INT64_MAX) {
        ptn_value_drop(&formatted);
        ptn_abort_out_of_memory();
    }
    int64_t len = (int64_t)string_value.as.string.len;
    ptn_value_drop(&formatted);
    return ptn_int(len);
}

static PtnValue ptn_internal_fprintf(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue formatted = ptn_internal_sprintf_named(runtime, "fprintf", argc - 1, args + 1, line);
    return ptn_internal_write_formatted_stream(runtime, "fprintf", args[0], formatted, line);
}

static PtnValue ptn_internal_vsprintf_named(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue format_value,
    PtnValue values_value,
    size_t line
) {
    values_value = ptn_value_deref(values_value);
    if (values_value.type != PTN_ARRAY) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #2 ($values) must be of type array, %s given",
            function_name,
            ptn_offset_container_type_name(values_value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    PtnArray *values = values_value.as.array;
    if (values->len > SIZE_MAX - 1) {
        ptn_abort_out_of_memory();
    }
    PtnValue *expanded = malloc((values->len + 1) * sizeof(PtnValue));
    if (expanded == NULL) {
        ptn_abort_out_of_memory();
    }
    expanded[0] = format_value;
    for (size_t i = 0; i < values->len; i++) {
        expanded[i + 1] = values->entries[i].value;
    }
    PtnValue result = ptn_internal_sprintf_named(runtime, function_name, values->len + 1, expanded, line);
    free(expanded);
    return result;
}

static PtnValue ptn_internal_vsprintf(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_vsprintf_named(runtime, "vsprintf", args[0], args[1], line);
}

static PtnValue ptn_internal_vprintf(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnValue formatted = ptn_internal_vsprintf_named(runtime, "vprintf", args[0], args[1], line);
    if (ptn_value_deref(formatted).type != PTN_STRING) {
        return formatted;
    }
    PtnValue string_value = ptn_value_deref(formatted);
    fwrite(string_value.as.string.data, 1, string_value.as.string.len, stdout);
    if (string_value.as.string.len > (size_t)INT64_MAX) {
        ptn_value_drop(&formatted);
        ptn_abort_out_of_memory();
    }
    int64_t len = (int64_t)string_value.as.string.len;
    ptn_value_drop(&formatted);
    return ptn_int(len);
}

static PtnValue ptn_internal_vfprintf(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnValue formatted = ptn_internal_vsprintf_named(runtime, "vfprintf", args[1], args[2], line);
    return ptn_internal_write_formatted_stream(runtime, "vfprintf", args[0], formatted, line);
}

static char *ptn_rot13_string(const char *string, size_t len) {
    char *rotated = malloc(len + 1);
    if (rotated == NULL) {
        ptn_abort_out_of_memory();
    }

    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)string[i];
        if (byte >= 'a' && byte <= 'z') {
            rotated[i] = (char)('a' + ((byte - 'a' + 13) % 26));
        } else if (byte >= 'A' && byte <= 'Z') {
            rotated[i] = (char)('A' + ((byte - 'A' + 13) % 26));
        } else {
            rotated[i] = (char)byte;
        }
    }
    rotated[len] = '\0';
    return rotated;
}

static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);

static PtnValue ptn_internal_str_rot13(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "str_rot13", 1, "string", args[0], line);
    char *rotated = ptn_rot13_string(string.data, string.len);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(rotated, len);
}

static PtnValue ptn_internal_str_shuffle(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "str_shuffle", 1, "string", args[0], line);
    char *shuffled = malloc(string.len + 1);
    if (shuffled == NULL) {
        ptn_abort_out_of_memory();
    }
    if (string.len != 0) {
        memcpy(shuffled, string.data, string.len);
    }
    shuffled[string.len] = '\0';
    if (string.len > 1) {
        for (size_t i = string.len - 1; i > 0; i--) {
            size_t j = ptn_random_bounded_index(i);
            char tmp = shuffled[i];
            shuffled[i] = shuffled[j];
            shuffled[j] = tmp;
        }
    }
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(shuffled, len);
}

static PtnValue ptn_internal_str_split(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "str_split", 1, "string", args[0], line);
    int64_t length = argc >= 2
        ? ptn_internal_expect_integer_arg(runtime, "str_split", 2, "length", args[1], line)
        : 1;
    if (length < 1) {
        ptn_string_operand_free(string);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "str_split(): Argument #2 ($length) must be greater than 0"
        );
        return ptn_null();
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    size_t chunk_len = (size_t)length;
    size_t index = 0;
    for (size_t offset = 0; offset < string.len; offset += chunk_len) {
        if (index > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        size_t remaining = string.len - offset;
        size_t len = remaining < chunk_len ? remaining : chunk_len;
        char *chunk = ptn_duplicate_string_len(string.data + offset, len);
        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key((int64_t)index),
            ptn_owned_string_len(chunk, len)
        );
        index++;
    }

    ptn_string_operand_free(string);
    return result;
}

static PtnValue ptn_internal_strrev(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "strrev", 1, "string", args[0], line);
    char *reversed = malloc(string.len + 1);
    if (reversed == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < string.len; i++) {
        reversed[i] = string.data[string.len - 1 - i];
    }
    reversed[string.len] = '\0';
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(reversed, len);
}

static char *ptn_first_char_case_string(const char *string, size_t len, int uppercase) {
    char *mapped = malloc(len + 1);
    if (mapped == NULL) {
        ptn_abort_out_of_memory();
    }
    if (len != 0) {
        memcpy(mapped, string, len);
        unsigned char first = (unsigned char)mapped[0];
        if (uppercase && first >= 'a' && first <= 'z') {
            mapped[0] = (char)('A' + (first - 'a'));
        } else if (!uppercase && first >= 'A' && first <= 'Z') {
            mapped[0] = (char)('a' + (first - 'A'));
        }
    }
    mapped[len] = '\0';
    return mapped;
}

static PtnValue ptn_internal_ucfirst(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "ucfirst", 1, "string", args[0], line);
    char *mapped = ptn_first_char_case_string(string.data, string.len, 1);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(mapped, len);
}

static PtnValue ptn_internal_lcfirst(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "lcfirst", 1, "string", args[0], line);
    char *mapped = ptn_first_char_case_string(string.data, string.len, 0);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(mapped, len);
}

static char *ptn_ascii_case_string(const char *string, size_t len, int uppercase) {
    char *mapped = malloc(len + 1);
    if (mapped == NULL) {
        ptn_abort_out_of_memory();
    }

    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)string[i];
        if (uppercase && byte >= 'a' && byte <= 'z') {
            mapped[i] = (char)('A' + (byte - 'a'));
        } else if (!uppercase && byte >= 'A' && byte <= 'Z') {
            mapped[i] = (char)('a' + (byte - 'A'));
        } else {
            mapped[i] = (char)byte;
        }
    }
    mapped[len] = '\0';
    return mapped;
}

static unsigned char ptn_ascii_lower_byte(unsigned char byte) {
    if (byte >= 'A' && byte <= 'Z') {
        return (unsigned char)('a' + (byte - 'A'));
    }
    return byte;
}

static int ptn_compare_string_bytes_ascii_case_insensitive(
    const unsigned char *left,
    size_t left_len,
    const unsigned char *right,
    size_t right_len
) {
    size_t shared_len = left_len < right_len ? left_len : right_len;
    for (size_t i = 0; i < shared_len; i++) {
        unsigned char left_byte = ptn_ascii_lower_byte(left[i]);
        unsigned char right_byte = ptn_ascii_lower_byte(right[i]);
        if (left_byte < right_byte) {
            return PTN_COMPARE_LESS;
        }
        if (left_byte > right_byte) {
            return PTN_COMPARE_GREATER;
        }
    }
    if (left_len < right_len) {
        return PTN_COMPARE_LESS;
    }
    if (left_len > right_len) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PtnValue ptn_internal_strtolower(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "strtolower", 1, "string", args[0], line);
    char *mapped = ptn_ascii_case_string(string.data, string.len, 0);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(mapped, len);
}

static PtnValue ptn_internal_strtoupper(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "strtoupper", 1, "string", args[0], line);
    char *mapped = ptn_ascii_case_string(string.data, string.len, 1);
    size_t len = string.len;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(mapped, len);
}

static PtnValue ptn_internal_strcmp(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand left = ptn_internal_expect_string_arg(runtime, "strcmp", 1, "string1", args[0], line);
    PtnStringOperand right = ptn_internal_expect_string_arg(runtime, "strcmp", 2, "string2", args[1], line);
    int compared = ptn_compare_string_bytes(
        (const unsigned char *)left.data,
        left.len,
        (const unsigned char *)right.data,
        right.len
    );
    ptn_string_operand_free(left);
    ptn_string_operand_free(right);
    if (compared < 0) {
        return ptn_int(-1);
    }
    if (compared > 0) {
        return ptn_int(1);
    }
    return ptn_int(0);
}

static PtnValue ptn_internal_strcasecmp(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand left = ptn_internal_expect_string_arg(runtime, "strcasecmp", 1, "string1", args[0], line);
    PtnStringOperand right = ptn_internal_expect_string_arg(runtime, "strcasecmp", 2, "string2", args[1], line);
    int compared = ptn_compare_string_bytes_ascii_case_insensitive(
        (const unsigned char *)left.data,
        left.len,
        (const unsigned char *)right.data,
        right.len
    );
    ptn_string_operand_free(left);
    ptn_string_operand_free(right);
    if (compared < 0) {
        return ptn_int(-1);
    }
    if (compared > 0) {
        return ptn_int(1);
    }
    return ptn_int(0);
}

static int ptn_compare_string_prefix_bytes(
    const unsigned char *left,
    size_t left_len,
    const unsigned char *right,
    size_t right_len,
    size_t limit
) {
    size_t left_prefix_len = left_len < limit ? left_len : limit;
    size_t right_prefix_len = right_len < limit ? right_len : limit;
    return ptn_compare_string_bytes(left, left_prefix_len, right, right_prefix_len);
}

static int ptn_compare_string_prefix_bytes_ascii_case_insensitive(
    const unsigned char *left,
    size_t left_len,
    const unsigned char *right,
    size_t right_len,
    size_t limit
) {
    size_t left_prefix_len = left_len < limit ? left_len : limit;
    size_t right_prefix_len = right_len < limit ? right_len : limit;
    return ptn_compare_string_bytes_ascii_case_insensitive(left, left_prefix_len, right, right_prefix_len);
}

static PtnValue ptn_internal_strncmp(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand left = ptn_internal_expect_string_arg(runtime, "strncmp", 1, "string1", args[0], line);
    PtnStringOperand right = ptn_internal_expect_string_arg(runtime, "strncmp", 2, "string2", args[1], line);
    int64_t length = ptn_internal_expect_integer_arg(runtime, "strncmp", 3, "length", args[2], line);
    if (length < 0) {
        ptn_string_operand_free(left);
        ptn_string_operand_free(right);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "strncmp(): Argument #3 ($length) must be greater than or equal to 0"
        );
    }
    int compared = ptn_compare_string_prefix_bytes(
        (const unsigned char *)left.data,
        left.len,
        (const unsigned char *)right.data,
        right.len,
        (size_t)length
    );
    ptn_string_operand_free(left);
    ptn_string_operand_free(right);
    if (compared < 0) {
        return ptn_int(-1);
    }
    if (compared > 0) {
        return ptn_int(1);
    }
    return ptn_int(0);
}

static PtnValue ptn_internal_strncasecmp(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand left = ptn_internal_expect_string_arg(runtime, "strncasecmp", 1, "string1", args[0], line);
    PtnStringOperand right = ptn_internal_expect_string_arg(runtime, "strncasecmp", 2, "string2", args[1], line);
    int64_t length = ptn_internal_expect_integer_arg(runtime, "strncasecmp", 3, "length", args[2], line);
    if (length < 0) {
        ptn_string_operand_free(left);
        ptn_string_operand_free(right);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "strncasecmp(): Argument #3 ($length) must be greater than or equal to 0"
        );
    }
    int compared = ptn_compare_string_prefix_bytes_ascii_case_insensitive(
        (const unsigned char *)left.data,
        left.len,
        (const unsigned char *)right.data,
        right.len,
        (size_t)length
    );
    ptn_string_operand_free(left);
    ptn_string_operand_free(right);
    if (compared < 0) {
        return ptn_int(-1);
    }
    if (compared > 0) {
        return ptn_int(1);
    }
    return ptn_int(0);
}

static PtnValue ptn_internal_str_contains(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, "str_contains", 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, "str_contains", 2, "needle", args[1], line);
    int contains = needle.len == 0;
    if (!contains && needle.len <= haystack.len) {
        for (size_t offset = 0; offset <= haystack.len - needle.len; offset++) {
            if (memcmp(haystack.data + offset, needle.data, needle.len) == 0) {
                contains = 1;
                break;
            }
        }
    }
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return ptn_bool(contains);
}

static PtnValue ptn_internal_str_starts_with(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, "str_starts_with", 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, "str_starts_with", 2, "needle", args[1], line);
    size_t haystack_len = haystack.len;
    size_t needle_len = needle.len;
    int starts = needle_len <= haystack_len && memcmp(haystack.data, needle.data, needle_len) == 0;
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return ptn_bool(starts);
}

static PtnValue ptn_internal_str_ends_with(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, "str_ends_with", 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, "str_ends_with", 2, "needle", args[1], line);
    size_t haystack_len = haystack.len;
    size_t needle_len = needle.len;
    int ends =
        needle_len <= haystack_len &&
        memcmp(haystack.data + haystack_len - needle_len, needle.data, needle_len) == 0;
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return ptn_bool(ends);
}

static PtnValue ptn_internal_strpbrk(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "strpbrk", 1, "string", args[0], line);
    PtnStringOperand characters = ptn_internal_expect_string_arg(runtime, "strpbrk", 2, "characters", args[1], line);

    if (characters.len == 0) {
        ptn_string_operand_free(string);
        ptn_string_operand_free(characters);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "strpbrk(): Argument #2 ($characters) must be a non-empty string"
        );
        return ptn_null();
    }

    unsigned char byte_set[256] = {0};
    for (size_t i = 0; i < characters.len; i++) {
        byte_set[(unsigned char)characters.data[i]] = 1;
    }

    for (size_t i = 0; i < string.len; i++) {
        if (byte_set[(unsigned char)string.data[i]]) {
            size_t result_len = string.len - i;
            char *result = ptn_duplicate_string_len(string.data + i, result_len);
            ptn_string_operand_free(string);
            ptn_string_operand_free(characters);
            return ptn_owned_string_len(result, result_len);
        }
    }

    ptn_string_operand_free(string);
    ptn_string_operand_free(characters);
    return ptn_bool(0);
}

static int ptn_match_bytes_at(
    const char *haystack,
    const char *needle,
    size_t needle_len,
    int case_insensitive
) {
    for (size_t i = 0; i < needle_len; i++) {
        unsigned char haystack_byte = (unsigned char)haystack[i];
        unsigned char needle_byte = (unsigned char)needle[i];
        if (case_insensitive) {
            haystack_byte = ptn_ascii_lower_byte(haystack_byte);
            needle_byte = ptn_ascii_lower_byte(needle_byte);
        }
        if (haystack_byte != needle_byte) {
            return 0;
        }
    }
    return 1;
}

static size_t ptn_find_bytes_from(
    const char *haystack,
    size_t haystack_len,
    const char *needle,
    size_t needle_len,
    size_t start,
    int case_insensitive
) {
    if (start > haystack_len) {
        return SIZE_MAX;
    }
    if (needle_len == 0) {
        return start;
    }
    if (needle_len > haystack_len - start) {
        return SIZE_MAX;
    }
    size_t last = haystack_len - needle_len;
    for (size_t offset = start; offset <= last; offset++) {
        if (ptn_match_bytes_at(haystack + offset, needle, needle_len, case_insensitive)) {
            return offset;
        }
    }
    return SIZE_MAX;
}

static size_t ptn_rfind_bytes_between(
    const char *haystack,
    size_t haystack_len,
    const char *needle,
    size_t needle_len,
    size_t start,
    size_t max_start,
    int case_insensitive
) {
    if (needle_len == 0) {
        return max_start <= haystack_len ? max_start : haystack_len;
    }
    if (start > haystack_len || needle_len > haystack_len) {
        return SIZE_MAX;
    }
    size_t last = haystack_len - needle_len;
    if (max_start > last) {
        max_start = last;
    }
    if (start > max_start) {
        return SIZE_MAX;
    }
    size_t cursor = max_start + 1;
    while (cursor > start) {
        size_t offset = cursor - 1;
        if (ptn_match_bytes_at(haystack + offset, needle, needle_len, case_insensitive)) {
            return offset;
        }
        cursor--;
    }
    return SIZE_MAX;
}

static int ptn_normalize_string_offset(
    PtnRuntime *runtime,
    const char *function_name,
    int64_t raw_offset,
    size_t string_len,
    size_t *offset_out
) {
    if (raw_offset < 0) {
        uint64_t offset_magnitude = raw_offset == INT64_MIN
            ? ((uint64_t)INT64_MAX + 1)
            : (uint64_t)(-raw_offset);
        if (offset_magnitude > (uint64_t)string_len) {
            char message[128];
            int written = snprintf(
                message,
                sizeof(message),
                "%s(): Argument #3 ($offset) must be contained in argument #1 ($haystack)",
                function_name
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "ValueError", message);
            return 0;
        }
        *offset_out = string_len - (size_t)offset_magnitude;
        return 1;
    }
    if ((uint64_t)raw_offset > (uint64_t)string_len) {
        char message[128];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #3 ($offset) must be contained in argument #1 ($haystack)",
            function_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ValueError", message);
        return 0;
    }
    *offset_out = (size_t)raw_offset;
    return 1;
}

static PtnValue ptn_string_position_value(size_t offset) {
    if (offset > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_int((int64_t)offset);
}

static PtnValue ptn_internal_strpos_named(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    size_t line,
    int case_insensitive
) {
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, function_name, 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, function_name, 2, "needle", args[1], line);
    int64_t raw_offset = argc >= 3
        ? ptn_internal_expect_integer_arg(runtime, function_name, 3, "offset", args[2], line)
        : 0;
    size_t offset = 0;
    if (!ptn_normalize_string_offset(runtime, function_name, raw_offset, haystack.len, &offset)) {
        ptn_string_operand_free(haystack);
        ptn_string_operand_free(needle);
        return ptn_null();
    }

    size_t match = ptn_find_bytes_from(
        haystack.data,
        haystack.len,
        needle.data,
        needle.len,
        offset,
        case_insensitive
    );
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return match == SIZE_MAX ? ptn_bool(0) : ptn_string_position_value(match);
}

static PtnValue ptn_internal_strpos(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_strpos_named(runtime, "strpos", argc, args, line, 0);
}

static PtnValue ptn_internal_stripos(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_strpos_named(runtime, "stripos", argc, args, line, 1);
}

static PtnValue ptn_internal_strrpos_named(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    size_t line,
    int case_insensitive
) {
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, function_name, 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, function_name, 2, "needle", args[1], line);
    int64_t raw_offset = argc >= 3
        ? ptn_internal_expect_integer_arg(runtime, function_name, 3, "offset", args[2], line)
        : 0;
    size_t offset = 0;
    if (!ptn_normalize_string_offset(runtime, function_name, raw_offset, haystack.len, &offset)) {
        ptn_string_operand_free(haystack);
        ptn_string_operand_free(needle);
        return ptn_null();
    }

    size_t start = 0;
    size_t max_start = haystack.len;
    if (raw_offset >= 0) {
        start = offset;
    } else {
        max_start = offset;
    }

    size_t match = ptn_rfind_bytes_between(
        haystack.data,
        haystack.len,
        needle.data,
        needle.len,
        start,
        max_start,
        case_insensitive
    );
    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return match == SIZE_MAX ? ptn_bool(0) : ptn_string_position_value(match);
}

static PtnValue ptn_internal_strrpos(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_strrpos_named(runtime, "strrpos", argc, args, line, 0);
}

static PtnValue ptn_internal_strripos(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_strrpos_named(runtime, "strripos", argc, args, line, 1);
}

static PtnValue ptn_internal_strstr_named(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    size_t line,
    int case_insensitive
) {
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, function_name, 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, function_name, 2, "needle", args[1], line);
    int before_needle = argc >= 3 && ptn_is_truthy(args[2]);
    size_t match = ptn_find_bytes_from(
        haystack.data,
        haystack.len,
        needle.data,
        needle.len,
        0,
        case_insensitive
    );
    ptn_string_operand_free(needle);
    if (match == SIZE_MAX) {
        ptn_string_operand_free(haystack);
        return ptn_bool(0);
    }

    size_t start = before_needle ? 0 : match;
    size_t len = before_needle ? match : haystack.len - match;
    PtnValue result = ptn_owned_string_len(ptn_duplicate_string_len(haystack.data + start, len), len);
    ptn_string_operand_free(haystack);
    return result;
}

static PtnValue ptn_internal_strstr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_strstr_named(runtime, "strstr", argc, args, line, 0);
}

static PtnValue ptn_internal_stristr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_strstr_named(runtime, "stristr", argc, args, line, 1);
}

static PtnValue ptn_internal_substr_count(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, "substr_count", 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, "substr_count", 2, "needle", args[1], line);
    if (needle.len == 0) {
        ptn_string_operand_free(haystack);
        ptn_string_operand_free(needle);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "substr_count(): Argument #2 ($needle) must not be empty"
        );
        return ptn_null();
    }

    int64_t raw_offset = argc >= 3
        ? ptn_internal_expect_integer_arg(runtime, "substr_count", 3, "offset", args[2], line)
        : 0;
    size_t start = 0;
    if (!ptn_normalize_string_offset(runtime, "substr_count", raw_offset, haystack.len, &start)) {
        ptn_string_operand_free(haystack);
        ptn_string_operand_free(needle);
        return ptn_null();
    }

    size_t end = haystack.len;
    if (argc >= 4 && ptn_value_deref(args[3]).type != PTN_NULL) {
        int64_t raw_length = ptn_internal_expect_integer_arg(runtime, "substr_count", 4, "length", args[3], line);
        int valid_length = 1;
        if (raw_length >= 0) {
            valid_length = (uint64_t)raw_length <= (uint64_t)(haystack.len - start);
            if (valid_length) {
                end = start + (size_t)raw_length;
            }
        } else {
            uint64_t trim = raw_length == INT64_MIN
                ? ((uint64_t)INT64_MAX + 1)
                : (uint64_t)(-raw_length);
            valid_length = trim <= (uint64_t)haystack.len && haystack.len - (size_t)trim >= start;
            if (valid_length) {
                end = haystack.len - (size_t)trim;
            }
        }
        if (!valid_length) {
            ptn_string_operand_free(haystack);
            ptn_string_operand_free(needle);
            ptn_throw_exception(
                runtime,
                "ValueError",
                "substr_count(): Argument #4 ($length) must be contained in argument #1 ($haystack)"
            );
            return ptn_null();
        }
    }

    int64_t count = 0;
    size_t offset = start;
    while (offset <= end && needle.len <= end - offset) {
        size_t match = ptn_find_bytes_from(
            haystack.data,
            end,
            needle.data,
            needle.len,
            offset,
            0
        );
        if (match == SIZE_MAX || match + needle.len > end) {
            break;
        }
        count++;
        offset = match + needle.len;
    }

    ptn_string_operand_free(haystack);
    ptn_string_operand_free(needle);
    return ptn_int(count);
}

static PtnValue ptn_internal_strrchr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand haystack = ptn_internal_expect_string_arg(runtime, "strrchr", 1, "haystack", args[0], line);
    PtnStringOperand needle = ptn_internal_expect_string_arg(runtime, "strrchr", 2, "needle", args[1], line);
    int before_needle = argc >= 3 && ptn_is_truthy(args[2]);
    unsigned char byte = needle.len == 0 ? 0 : (unsigned char)needle.data[0];
    size_t match = haystack.len;
    for (size_t i = haystack.len; i > 0; i--) {
        if ((unsigned char)haystack.data[i - 1] == byte) {
            match = i - 1;
            break;
        }
    }
    ptn_string_operand_free(needle);
    if (match == haystack.len) {
        ptn_string_operand_free(haystack);
        return ptn_bool(0);
    }

    size_t start = before_needle ? 0 : match;
    size_t len = before_needle ? match : haystack.len - match;
    PtnValue result = ptn_owned_string_len(ptn_duplicate_string_len(haystack.data + start, len), len);
    ptn_string_operand_free(haystack);
    return result;
}

static void ptn_string_buffer_append_repeated_pattern(PtnStringBuffer *buffer, PtnStringOperand pattern, size_t len) {
    size_t remaining = len;
    while (remaining > 0) {
        size_t chunk_len = pattern.len < remaining ? pattern.len : remaining;
        ptn_string_buffer_append_len(buffer, pattern.data, chunk_len);
        remaining -= chunk_len;
    }
}

static PtnValue ptn_internal_str_pad(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "str_pad", 1, "string", args[0], line);
    int64_t length = ptn_value_to_integer(args[1]);
    PtnStringOperand pad_string = argc >= 3
        ? ptn_internal_expect_string_arg(runtime, "str_pad", 3, "pad_string", args[2], line)
        : ptn_string_operand_borrowed(" ");

    if (pad_string.len == 0) {
        ptn_string_operand_free(input);
        ptn_string_operand_free(pad_string);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "str_pad(): Argument #3 ($pad_string) must not be empty"
        );
        return ptn_null();
    }

    int64_t pad_type = argc >= 4 ? ptn_value_to_integer(args[3]) : PTN_STR_PAD_RIGHT;
    if (pad_type != PTN_STR_PAD_LEFT &&
        pad_type != PTN_STR_PAD_RIGHT &&
        pad_type != PTN_STR_PAD_BOTH) {
        ptn_string_operand_free(input);
        ptn_string_operand_free(pad_string);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "str_pad(): Argument #4 ($pad_type) must be STR_PAD_LEFT, STR_PAD_RIGHT, or STR_PAD_BOTH"
        );
        return ptn_null();
    }

    if (length <= 0 || (uint64_t)length <= (uint64_t)input.len) {
        char *copy = ptn_duplicate_string_len(input.data, input.len);
        size_t copy_len = input.len;
        ptn_string_operand_free(input);
        ptn_string_operand_free(pad_string);
        return ptn_owned_string_len(copy, copy_len);
    }

    size_t target_len = (size_t)length;
    size_t pad_len = target_len - input.len;
    size_t left_len = 0;
    size_t right_len = 0;
    if (pad_type == PTN_STR_PAD_LEFT) {
        left_len = pad_len;
    } else if (pad_type == PTN_STR_PAD_RIGHT) {
        right_len = pad_len;
    } else {
        left_len = pad_len / 2;
        right_len = pad_len - left_len;
    }

    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    ptn_string_buffer_reserve(&output, target_len);
    ptn_string_buffer_append_repeated_pattern(&output, pad_string, left_len);
    ptn_string_buffer_append_len(&output, input.data, input.len);
    ptn_string_buffer_append_repeated_pattern(&output, pad_string, right_len);

    ptn_string_operand_free(input);
    ptn_string_operand_free(pad_string);
    return ptn_owned_string_len(output.data, output.len);
}

static PtnValue ptn_internal_str_repeat(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "str_repeat", 1, "string", args[0], line);
    int64_t repeat = ptn_value_to_integer(args[1]);
    if (repeat < 0) {
        ptn_string_operand_free(input);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "str_repeat(): Argument #2 ($times) must be greater than or equal to 0"
        );
    }
    if (repeat == 0 || input.len == 0) {
        ptn_string_operand_free(input);
        char *empty = malloc(1);
        if (empty == NULL) {
            ptn_abort_out_of_memory();
        }
        empty[0] = '\0';
        return ptn_owned_string_len(empty, 0);
    }

    size_t times = (size_t)repeat;
    if (input.len > SIZE_MAX / times || input.len * times == SIZE_MAX) {
        ptn_string_operand_free(input);
        ptn_abort_out_of_memory();
    }
    size_t output_len = input.len * times;
    char *output = malloc(output_len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < times; i++) {
        memcpy(output + (i * input.len), input.data, input.len);
    }
    output[output_len] = '\0';
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static PtnStringOperand ptn_trim_default_charlist(void) {
    static const char bytes[] = { ' ', '\t', '\n', '\r', '\0', '\v' };
    return ptn_string_operand_borrowed_len(bytes, sizeof(bytes));
}

static void ptn_trim_charlist_table(PtnStringOperand charlist, unsigned char table[256]) {
    memset(table, 0, 256);
    for (size_t i = 0; i < charlist.len; i++) {
        unsigned char start = (unsigned char)charlist.data[i];
        if (
            i + 3 < charlist.len &&
            charlist.data[i + 1] == '.' &&
            charlist.data[i + 2] == '.'
        ) {
            unsigned char end = (unsigned char)charlist.data[i + 3];
            if (start <= end) {
                for (unsigned int byte = start; byte <= end; byte++) {
                    table[byte] = 1;
                }
                i += 3;
                continue;
            }
        }
        table[start] = 1;
    }
}

static PtnValue ptn_trim_string_value(
    PtnStringOperand input,
    PtnStringOperand charlist,
    int trim_left,
    int trim_right
) {
    unsigned char table[256];
    ptn_trim_charlist_table(charlist, table);

    size_t start = 0;
    size_t end = input.len;
    if (trim_left) {
        while (start < end && table[(unsigned char)input.data[start]]) {
            start++;
        }
    }
    if (trim_right) {
        while (end > start && table[(unsigned char)input.data[end - 1]]) {
            end--;
        }
    }

    size_t output_len = end - start;
    char *output = ptn_duplicate_string_len(input.data + start, output_len);
    return ptn_owned_string_len(output, output_len);
}

static PtnValue ptn_internal_trim_named(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    size_t line,
    int trim_left,
    int trim_right
) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, function_name, 1, "string", args[0], line);
    PtnStringOperand charlist = argc >= 2
        ? ptn_internal_expect_string_arg(runtime, function_name, 2, "characters", args[1], line)
        : ptn_trim_default_charlist();
    PtnValue result = ptn_trim_string_value(input, charlist, trim_left, trim_right);
    ptn_string_operand_free(input);
    ptn_string_operand_free(charlist);
    return result;
}

static PtnValue ptn_internal_trim(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_trim_named(runtime, "trim", argc, args, line, 1, 1);
}

static PtnValue ptn_internal_ltrim(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_trim_named(runtime, "ltrim", argc, args, line, 1, 0);
}

static PtnValue ptn_internal_rtrim(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_trim_named(runtime, "rtrim", argc, args, line, 0, 1);
}

static PtnValue ptn_internal_chop(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_trim_named(runtime, "chop", argc, args, line, 0, 1);
}

typedef struct {
    PtnStringOperand from;
    PtnStringOperand to;
    size_t order;
} PtnStrtrReplacement;

static PtnStringOperand ptn_array_key_string_operand(PtnArrayKey key) {
    if (key.type == PTN_ARRAY_KEY_STRING) {
        return ptn_string_operand_borrowed(key.as.string);
    }

    char buffer[64];
    int written = snprintf(buffer, sizeof(buffer), "%lld", (long long)key.as.integer);
    if (written < 0 || (size_t)written >= sizeof(buffer)) {
        ptn_abort_out_of_memory();
    }
    return ptn_string_operand_owned_len(
        ptn_duplicate_string_len(buffer, (size_t)written),
        (size_t)written
    );
}

static void ptn_strtr_replacements_free(PtnStrtrReplacement *replacements, size_t count) {
    for (size_t i = 0; i < count; i++) {
        ptn_string_operand_free(replacements[i].from);
        ptn_string_operand_free(replacements[i].to);
    }
    free(replacements);
}

static void ptn_strtr_replacements_sort(PtnStrtrReplacement *replacements, size_t count) {
    for (size_t i = 1; i < count; i++) {
        PtnStrtrReplacement current = replacements[i];
        size_t j = i;
        while (j > 0 &&
            (replacements[j - 1].from.len < current.from.len ||
                (replacements[j - 1].from.len == current.from.len &&
                    replacements[j - 1].order > current.order))) {
            replacements[j] = replacements[j - 1];
            j--;
        }
        replacements[j] = current;
    }
}

static PtnValue ptn_strtr_array(PtnStringOperand input, PtnArray *map) {
    PtnStrtrReplacement *replacements = NULL;
    if (map->len != 0) {
        replacements = malloc(map->len * sizeof(PtnStrtrReplacement));
        if (replacements == NULL) {
            ptn_abort_out_of_memory();
        }
    }

    size_t replacement_count = 0;
    for (size_t i = 0; i < map->len; i++) {
        PtnStringOperand from = ptn_array_key_string_operand(map->entries[i].key);
        if (from.len == 0) {
            ptn_string_operand_free(from);
            continue;
        }
        replacements[replacement_count].from = from;
        replacements[replacement_count].to = ptn_value_to_string_operand(map->entries[i].value);
        replacements[replacement_count].order = i;
        replacement_count++;
    }
    ptn_strtr_replacements_sort(replacements, replacement_count);

    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    size_t offset = 0;
    while (offset < input.len) {
        const PtnStrtrReplacement *matched = NULL;
        for (size_t i = 0; i < replacement_count; i++) {
            if (replacements[i].from.len <= input.len - offset &&
                memcmp(input.data + offset, replacements[i].from.data, replacements[i].from.len) == 0) {
                matched = &replacements[i];
                break;
            }
        }

        if (matched != NULL) {
            ptn_string_buffer_append_len(&output, matched->to.data, matched->to.len);
            offset += matched->from.len;
        } else {
            ptn_string_buffer_append_char(&output, input.data[offset]);
            offset++;
        }
    }

    ptn_strtr_replacements_free(replacements, replacement_count);
    return ptn_owned_string_len(output.data, output.len);
}

static PtnValue ptn_strtr_byte_map(PtnStringOperand input, PtnStringOperand from, PtnStringOperand to) {
    unsigned char map[256];
    for (size_t i = 0; i < sizeof(map); i++) {
        map[i] = (unsigned char)i;
    }
    size_t count = from.len < to.len ? from.len : to.len;
    for (size_t i = 0; i < count; i++) {
        map[(unsigned char)from.data[i]] = (unsigned char)to.data[i];
    }

    char *output = malloc(input.len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < input.len; i++) {
        output[i] = (char)map[(unsigned char)input.data[i]];
    }
    output[input.len] = '\0';
    return ptn_owned_string_len(output, input.len);
}

static PtnValue ptn_internal_strtr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "strtr", 1, "string", args[0], line);
    if (argc == 2) {
        PtnValue map_value = ptn_value_deref(args[1]);
        if (map_value.type != PTN_ARRAY) {
            ptn_string_operand_free(input);
            ptn_throw_exception(
                runtime,
                "TypeError",
                "strtr(): Argument #2 ($from) must be of type array when argument #3 ($to) is not provided"
            );
        }
        PtnValue result = ptn_strtr_array(input, map_value.as.array);
        ptn_string_operand_free(input);
        return result;
    }

    PtnStringOperand from = ptn_internal_expect_string_arg(runtime, "strtr", 2, "from", args[1], line);
    PtnStringOperand to = ptn_internal_expect_string_arg(runtime, "strtr", 3, "to", args[2], line);
    PtnValue result = ptn_strtr_byte_map(input, from, to);
    ptn_string_operand_free(input);
    ptn_string_operand_free(from);
    ptn_string_operand_free(to);
    return result;
}

static const char *ptn_find_bytes(
    const char *haystack,
    size_t haystack_len,
    const char *needle,
    size_t needle_len
) {
    size_t match = ptn_find_bytes_from(haystack, haystack_len, needle, needle_len, 0, 0);
    if (match == SIZE_MAX || needle_len == 0) {
        return NULL;
    }
    return haystack + match;
}

static void ptn_internal_throw_array_or_string_arg_type_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value
) {
    value = ptn_value_deref(value);
    char message[224];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be of type array|string, %s given",
        function_name,
        position,
        argument_name,
        ptn_internal_string_arg_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static PtnStringOperand ptn_internal_expect_str_replace_arg(
    PtnRuntime *runtime,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (
        value.type == PTN_RESOURCE ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION
    ) {
        ptn_internal_throw_array_or_string_arg_type_error(
            runtime,
            "str_replace",
            position,
            argument_name,
            value
        );
        return ptn_string_operand_borrowed("");
    }
    if (value.type == PTN_OBJECT) {
        PtnStringOperand object_string;
        if (ptn_try_object_to_string_operand(runtime, value, line, &object_string)) {
            return object_string;
        }
        ptn_internal_throw_array_or_string_arg_type_error(
            runtime,
            "str_replace",
            position,
            argument_name,
            value
        );
        return ptn_string_operand_borrowed("");
    }
    return ptn_value_to_string_operand_with_runtime(runtime, value, line);
}

typedef struct {
    PtnStringOperand *items;
    size_t len;
    int is_array;
} PtnStrReplaceStringList;

static void ptn_str_replace_string_list_free(PtnStrReplaceStringList *list) {
    for (size_t i = 0; i < list->len; i++) {
        ptn_string_operand_free(list->items[i]);
    }
    free(list->items);
    list->items = NULL;
    list->len = 0;
}

static PtnStringOperand ptn_str_replace_entry_to_string(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_ARRAY) {
        ptn_emit_warning(&runtime->diagnostics, "Array to string conversion", line);
    }
    return ptn_value_to_string_operand_with_runtime(runtime, value, line);
}

static PtnStrReplaceStringList ptn_str_replace_string_list_from_arg(
    PtnRuntime *runtime,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    PtnStrReplaceStringList list;
    list.items = NULL;
    list.len = 0;
    list.is_array = value.type == PTN_ARRAY;

    if (!list.is_array) {
        list.items = malloc(sizeof(PtnStringOperand));
        if (list.items == NULL) {
            ptn_abort_out_of_memory();
        }
        list.len = 1;
        list.items[0] = ptn_internal_expect_str_replace_arg(runtime, position, argument_name, value, line);
        return list;
    }

    if (value.as.array->len == 0) {
        return list;
    }

    list.items = malloc(value.as.array->len * sizeof(PtnStringOperand));
    if (list.items == NULL) {
        ptn_abort_out_of_memory();
    }
    list.len = value.as.array->len;
    for (size_t i = 0; i < value.as.array->len; i++) {
        list.items[i] = ptn_str_replace_entry_to_string(runtime, value.as.array->entries[i].value, line);
    }
    return list;
}

static PtnValue ptn_str_replace_apply_to_string(
    PtnStringOperand subject,
    const PtnStrReplaceStringList *search,
    const PtnStrReplaceStringList *replace,
    int64_t *replacement_count
) {
    PtnStringOperand current = ptn_string_operand_owned_len(
        ptn_duplicate_string_len(subject.data, subject.len),
        subject.len
    );

    for (size_t i = 0; i < search->len; i++) {
        PtnStringOperand needle = search->items[i];
        if (needle.len == 0) {
            continue;
        }

        PtnStringOperand replacement = ptn_string_operand_borrowed("");
        if (replace->is_array) {
            if (i < replace->len) {
                replacement = replace->items[i];
            }
        } else if (replace->len > 0) {
            replacement = replace->items[0];
        }

        PtnStringBuffer output;
        ptn_string_buffer_init(&output);
        size_t offset = 0;
        while (offset < current.len) {
            const char *matched = ptn_find_bytes(
                current.data + offset,
                current.len - offset,
                needle.data,
                needle.len
            );
            if (matched == NULL) {
                ptn_string_buffer_append_len(&output, current.data + offset, current.len - offset);
                break;
            }
            size_t prefix_len = (size_t)(matched - (current.data + offset));
            ptn_string_buffer_append_len(&output, current.data + offset, prefix_len);
            ptn_string_buffer_append_len(&output, replacement.data, replacement.len);
            offset += prefix_len + needle.len;
            (*replacement_count)++;
        }

        ptn_string_operand_free(current);
        current = ptn_string_operand_owned_len(output.data, output.len);
    }

    char *result = current.owned;
    size_t result_len = current.len;
    current.owned = NULL;
    return ptn_owned_string_len(result, result_len);
}

static PtnValue ptn_internal_str_replace(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnValue replace_value = ptn_value_deref(args[1]);

    PtnStrReplaceStringList search =
        ptn_str_replace_string_list_from_arg(runtime, 1, "search", args[0], line);
    if (!search.is_array && replace_value.type == PTN_ARRAY) {
        ptn_str_replace_string_list_free(&search);
        ptn_throw_exception(
            runtime,
            "TypeError",
            "str_replace(): Argument #2 ($replace) must be of type string when argument #1 ($search) is a string"
        );
        return ptn_null();
    }

    PtnStrReplaceStringList replace =
        ptn_str_replace_string_list_from_arg(runtime, 2, "replace", args[1], line);
    int64_t replacement_count = 0;

    PtnValue result;
    PtnValue subject_value = ptn_value_deref(args[2]);
    if (subject_value.type == PTN_ARRAY) {
        result = ptn_array_from_literal_entries(0, NULL);
        for (size_t i = 0; i < subject_value.as.array->len; i++) {
            PtnStringOperand subject =
                ptn_str_replace_entry_to_string(runtime, subject_value.as.array->entries[i].value, line);
            PtnValue replaced =
                ptn_str_replace_apply_to_string(subject, &search, &replace, &replacement_count);
            ptn_string_operand_free(subject);
            ptn_array_set_entry(
                result.as.array,
                ptn_array_key_clone(subject_value.as.array->entries[i].key),
                replaced
            );
        }
    } else {
        PtnStringOperand subject =
            ptn_internal_expect_str_replace_arg(runtime, 3, "subject", args[2], line);
        result = ptn_str_replace_apply_to_string(subject, &search, &replace, &replacement_count);
        ptn_string_operand_free(subject);
    }

    if (argc >= 4 && args[3].type == PTN_REFERENCE) {
        PtnValue count_value = ptn_int(replacement_count);
        ptn_reference_assign(args[3].as.reference, count_value);
    }

    ptn_str_replace_string_list_free(&search);
    ptn_str_replace_string_list_free(&replace);
    return result;
}

static PtnValue ptn_internal_nl2br(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "nl2br", 1, "string", args[0], line);
    const char *break_tag = argc >= 2 && !ptn_is_truthy(args[1]) ? "<br>" : "<br />";
    size_t break_tag_len = strlen(break_tag);

    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    for (size_t i = 0; i < input.len; i++) {
        char byte = input.data[i];
        if (byte == '\r' || byte == '\n') {
            ptn_string_buffer_append_len(&output, break_tag, break_tag_len);
            ptn_string_buffer_append_char(&output, byte);
            if (i + 1 < input.len) {
                char next = input.data[i + 1];
                if ((byte == '\r' && next == '\n') || (byte == '\n' && next == '\r')) {
                    ptn_string_buffer_append_char(&output, next);
                    i++;
                }
            }
        } else {
            ptn_string_buffer_append_char(&output, byte);
        }
    }

    size_t len = output.len;
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output.data, len);
}

static int ptn_regex_is_escaped(const char *data, size_t index) {
    size_t slash_count = 0;
    while (index > 0 && data[index - 1] == '\\') {
        slash_count++;
        index--;
    }
    return (slash_count % 2) != 0;
}

static int ptn_regex_delimiter_end(PtnStringOperand pattern, size_t *end_out) {
    if (pattern.len < 2) {
        return 0;
    }
    char delimiter = pattern.data[0];
    if (isalnum((unsigned char)delimiter) || delimiter == '\\' || isspace((unsigned char)delimiter)) {
        return 0;
    }
    for (size_t i = pattern.len - 1; i > 0; i--) {
        if (pattern.data[i] == delimiter && !ptn_regex_is_escaped(pattern.data, i)) {
            *end_out = i;
            return 1;
        }
    }
    return 0;
}

static int ptn_regex_char_is_posix_special(char byte) {
    switch (byte) {
        case '.':
        case '^':
        case '$':
        case '*':
        case '+':
        case '?':
        case '(':
        case ')':
        case '[':
        case ']':
        case '{':
        case '}':
        case '|':
        case '\\':
            return 1;
        default:
            return 0;
    }
}

static void ptn_capture_map_push(size_t **capture_map, size_t *capture_count, size_t group_index) {
    size_t new_count = *capture_count + 1;
    size_t *new_map = realloc(*capture_map, new_count * sizeof(size_t));
    if (new_map == NULL) {
        ptn_abort_out_of_memory();
    }
    new_map[*capture_count] = group_index;
    *capture_map = new_map;
    *capture_count = new_count;
}

static char *ptn_pcre_pattern_to_posix(
    PtnStringOperand pattern,
    int *flags_out,
    size_t **capture_map_out,
    size_t *capture_count_out
) {
    size_t end = 0;
    if (!ptn_regex_delimiter_end(pattern, &end)) {
        return NULL;
    }

    *flags_out = REG_EXTENDED;
    for (size_t i = end + 1; i < pattern.len; i++) {
        if (pattern.data[i] == 'i') {
            *flags_out |= REG_ICASE;
        }
    }

    PtnStringBuffer output;
    ptn_string_buffer_init(&output);
    size_t *capture_map = NULL;
    size_t capture_count = 0;
    size_t posix_group_index = 0;
    int in_char_class = 0;
    char delimiter = pattern.data[0];
    for (size_t i = 1; i < end; i++) {
        char byte = pattern.data[i];
        if (byte == '\\' && i + 1 < end) {
            char next = pattern.data[++i];
            switch (next) {
                case 'd':
                    ptn_string_buffer_append(&output, "[0-9]");
                    break;
                case 's':
                    ptn_string_buffer_append(&output, "[[:space:]]");
                    break;
                case 'w':
                    ptn_string_buffer_append(&output, "[A-Za-z0-9_]");
                    break;
                case 'n':
                    ptn_string_buffer_append_char(&output, '\n');
                    break;
                case 'r':
                    ptn_string_buffer_append_char(&output, '\r');
                    break;
                case 't':
                    ptn_string_buffer_append_char(&output, '\t');
                    break;
                default:
                    if (next == delimiter || !ptn_regex_char_is_posix_special(next)) {
                        ptn_string_buffer_append_char(&output, next);
                    } else {
                        ptn_string_buffer_append_char(&output, '\\');
                        ptn_string_buffer_append_char(&output, next);
                    }
                    break;
            }
            continue;
        }

        if (byte == '[') {
            in_char_class = 1;
            ptn_string_buffer_append_char(&output, byte);
            continue;
        }
        if (byte == ']') {
            in_char_class = 0;
            ptn_string_buffer_append_char(&output, byte);
            continue;
        }

        if (!in_char_class && byte == '(') {
            posix_group_index++;
            if (i + 2 < end && pattern.data[i + 1] == '?' && pattern.data[i + 2] == ':') {
                ptn_string_buffer_append_char(&output, '(');
                i += 2;
                continue;
            }
            ptn_capture_map_push(&capture_map, &capture_count, posix_group_index);
        }

        ptn_string_buffer_append_char(&output, byte);
    }
    *capture_map_out = capture_map;
    *capture_count_out = capture_count;
    return output.data;
}

static void ptn_preg_match_assign_matches(
    PtnValue matches_arg,
    const char *subject,
    regmatch_t *matches,
    size_t *capture_map,
    size_t capture_count
) {
    if (matches_arg.type != PTN_REFERENCE) {
        return;
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i <= capture_count; i++) {
        size_t match_index = i == 0 ? 0 : capture_map[i - 1];
        PtnValue value;
        if (matches[match_index].rm_so < 0 || matches[match_index].rm_eo < matches[match_index].rm_so) {
            value = ptn_string("");
        } else {
            size_t start = (size_t)matches[match_index].rm_so;
            size_t len = (size_t)(matches[match_index].rm_eo - matches[match_index].rm_so);
            value = ptn_owned_string_len(ptn_duplicate_string_len(subject + start, len), len);
        }
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(result.as.array, ptn_array_int_key((int64_t)i), value);
    }
    ptn_reference_assign(matches_arg.as.reference, result);
    ptn_value_destroy(&result);
}

static PtnValue ptn_internal_preg_match(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand pattern = ptn_value_to_string_operand(args[0]);
    PtnStringOperand subject = ptn_value_to_string_operand(args[1]);
    int regex_flags = 0;
    size_t *capture_map = NULL;
    size_t capture_count = 0;
    char *posix_pattern = ptn_pcre_pattern_to_posix(pattern, &regex_flags, &capture_map, &capture_count);
    if (posix_pattern == NULL) {
        ptn_string_operand_free(pattern);
        ptn_string_operand_free(subject);
        ptn_emit_warning(&runtime->diagnostics, "preg_match(): Compilation failed", line);
        return ptn_bool(0);
    }

#if defined(_WIN32)
    free(posix_pattern);
    free(capture_map);
    ptn_string_operand_free(pattern);
    ptn_string_operand_free(subject);
    ptn_emit_warning(&runtime->diagnostics, "preg_match(): regex matching is unsupported on this platform", line);
    return ptn_bool(0);
#else
    regex_t regex;
    int compile_result = regcomp(&regex, posix_pattern, regex_flags);
    free(posix_pattern);
    if (compile_result != 0) {
        free(capture_map);
        ptn_string_operand_free(pattern);
        ptn_string_operand_free(subject);
        ptn_emit_warning(&runtime->diagnostics, "preg_match(): Compilation failed", line);
        return ptn_bool(0);
    }

    size_t match_count = regex.re_nsub + 1;
    regmatch_t *matches = calloc(match_count, sizeof(regmatch_t));
    if (matches == NULL) {
        regfree(&regex);
        ptn_string_operand_free(pattern);
        ptn_string_operand_free(subject);
        ptn_abort_out_of_memory();
    }
    char *subject_c = ptn_duplicate_string_len(subject.data, subject.len);
    int exec_result = regexec(&regex, subject_c, match_count, matches, 0);
    int matched = exec_result == 0;
    if (argc >= 3) {
        if (matched) {
            ptn_preg_match_assign_matches(args[2], subject_c, matches, capture_map, capture_count);
        } else if (args[2].type == PTN_REFERENCE) {
            PtnValue empty_matches = ptn_array_from_literal_entries(0, NULL);
            ptn_reference_assign(args[2].as.reference, empty_matches);
            ptn_value_destroy(&empty_matches);
        }
    }

    free(subject_c);
    free(matches);
    free(capture_map);
    regfree(&regex);
    ptn_string_operand_free(pattern);
    ptn_string_operand_free(subject);
    return ptn_int(matched ? 1 : 0);
#endif
}

static int ptn_quotemeta_needs_escape(unsigned char byte) {
    switch (byte) {
        case '.':
        case '\\':
        case '+':
        case '*':
        case '?':
        case '[':
        case '^':
        case ']':
        case '(':
        case '$':
        case ')':
            return 1;
        default:
            return 0;
    }
}

static char *ptn_quotemeta_string(const char *input, size_t len, size_t *output_len_out) {
    size_t escape_count = 0;
    for (size_t i = 0; i < len; i++) {
        if (ptn_quotemeta_needs_escape((unsigned char)input[i])) {
            escape_count++;
        }
    }
    if (escape_count > SIZE_MAX - len - 1) {
        ptn_abort_out_of_memory();
    }

    char *output = malloc(len + escape_count + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)input[i];
        if (ptn_quotemeta_needs_escape(byte)) {
            output[out++] = '\\';
        }
        output[out++] = (char)byte;
    }
    output[out] = '\0';
    *output_len_out = out;
    return output;
}

static PtnValue ptn_internal_quotemeta(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "quotemeta", 1, "string", args[0], line);
    size_t output_len = 0;
    char *output = ptn_quotemeta_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static char *ptn_chunk_split_string(
    const char *input,
    size_t input_len,
    size_t chunk_len,
    const char *ending,
    size_t ending_len,
    size_t *output_len_out
) {
    if (input_len == 0) {
        if (ending_len == SIZE_MAX) {
            ptn_abort_out_of_memory();
        }
        char *output = malloc(ending_len + 1);
        if (output == NULL) {
            ptn_abort_out_of_memory();
        }
        memcpy(output, ending, ending_len);
        output[ending_len] = '\0';
        *output_len_out = ending_len;
        return output;
    }

    size_t chunk_count = ((input_len - 1) / chunk_len) + 1;
    if (chunk_count != 0 && ending_len > (SIZE_MAX - input_len) / chunk_count) {
        ptn_abort_out_of_memory();
    }
    size_t output_len = input_len + (chunk_count * ending_len);
    if (output_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }

    char *output = malloc(output_len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t input_offset = 0;
    size_t output_offset = 0;
    while (input_offset < input_len) {
        size_t remaining = input_len - input_offset;
        size_t copy_len = remaining < chunk_len ? remaining : chunk_len;
        memcpy(output + output_offset, input + input_offset, copy_len);
        input_offset += copy_len;
        output_offset += copy_len;
        memcpy(output + output_offset, ending, ending_len);
        output_offset += ending_len;
    }
    output[output_offset] = '\0';
    *output_len_out = output_offset;
    return output;
}

static PtnValue ptn_internal_chunk_split(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "chunk_split", 1, "string", args[0], line);
    int64_t chunk_len_value = argc >= 2 ? ptn_value_to_integer(args[1]) : 76;
    if (chunk_len_value <= 0) {
        ptn_string_operand_free(input);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "chunk_split(): Argument #2 ($length) must be greater than 0"
        );
    }
    PtnStringOperand ending;
    if (argc >= 3) {
        ending = ptn_internal_expect_string_arg(runtime, "chunk_split", 3, "separator", args[2], line);
    } else {
        ending.data = "\r\n";
        ending.owned = NULL;
        ending.len = 2;
    }
    size_t output_len = 0;
    char *output = ptn_chunk_split_string(
        input.data,
        input.len,
        (size_t)chunk_len_value,
        ending.data,
        ending.len,
        &output_len
    );
    ptn_string_operand_free(input);
    ptn_string_operand_free(ending);
    return ptn_owned_string_len(output, output_len);
}

static char *ptn_strip_tags_string(const char *input, size_t len, size_t *output_len_out) {
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t input_offset = 0;
    size_t output_offset = 0;
    while (input_offset < len) {
        if (input[input_offset] == '<') {
            if (input_offset + 1 < len && input[input_offset + 1] == '?') {
                size_t tag_end = input_offset + 2;
                while (tag_end + 1 < len && !(input[tag_end] == '?' && input[tag_end + 1] == '>')) {
                    tag_end++;
                }
                if (tag_end + 1 < len) {
                    input_offset = tag_end + 2;
                    continue;
                }
            } else if (input_offset + 1 < len && input[input_offset + 1] == '%') {
                size_t tag_end = input_offset + 2;
                while (tag_end + 1 < len && !(input[tag_end] == '%' && input[tag_end + 1] == '>')) {
                    tag_end++;
                }
                if (tag_end + 1 < len) {
                    input_offset = tag_end + 2;
                    continue;
                }
            } else if (input_offset + 3 < len && memcmp(input + input_offset, "<!--", 4) == 0) {
                size_t tag_end = input_offset + 4;
                while (tag_end + 2 < len && memcmp(input + tag_end, "-->", 3) != 0) {
                    tag_end++;
                }
                if (tag_end + 2 < len) {
                    input_offset = tag_end + 3;
                    continue;
                }
            } else {
                size_t tag_end = input_offset + 1;
                while (tag_end < len && input[tag_end] != '>') {
                    tag_end++;
                }
                if (tag_end < len) {
                    input_offset = tag_end + 1;
                    continue;
                }
            }
        }
        if (input[input_offset] == '\0') {
            input_offset++;
            continue;
        }
        output[output_offset++] = input[input_offset++];
    }
    output[output_offset] = '\0';
    *output_len_out = output_offset;
    return output;
}

static PtnValue ptn_internal_strip_tags(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "strip_tags", 1, "string", args[0], line);
    size_t output_len = 0;
    char *output = ptn_strip_tags_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static uint32_t ptn_rotate_left32(uint32_t value, uint32_t amount) {
    return (value << amount) | (value >> (32 - amount));
}

static char *ptn_digest_hex_string(const unsigned char *digest, size_t digest_len) {
    static const char hex_digits[] = "0123456789abcdef";
    if (digest_len > (SIZE_MAX - 1) / 2) {
        ptn_abort_out_of_memory();
    }
    char *hex = malloc((digest_len * 2) + 1);
    if (hex == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < digest_len; i++) {
        hex[i * 2] = hex_digits[digest[i] >> 4];
        hex[i * 2 + 1] = hex_digits[digest[i] & 0x0f];
    }
    hex[digest_len * 2] = '\0';
    return hex;
}

static char *ptn_digest_raw_string(const unsigned char *digest, size_t digest_len) {
    char *raw = malloc(digest_len + 1);
    if (raw == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(raw, digest, digest_len);
    raw[digest_len] = '\0';
    return raw;
}

static PtnValue ptn_digest_value(const unsigned char *digest, size_t digest_len, int raw_output) {
    if (raw_output) {
        return ptn_owned_string_len(ptn_digest_raw_string(digest, digest_len), digest_len);
    }
    return ptn_owned_string_len(ptn_digest_hex_string(digest, digest_len), digest_len * 2);
}

static void ptn_md5_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[16]) {
    static const uint32_t shifts[64] = {
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21
    };
    static const uint32_t constants[64] = {
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee,
        0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
        0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be,
        0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
        0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa,
        0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
        0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
        0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c,
        0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
        0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05,
        0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
        0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039,
        0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1,
        0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391
    };

    size_t padded_len = input_len + 1;
    while ((padded_len % 64) != 56) {
        padded_len++;
    }
    if (padded_len < input_len || padded_len > SIZE_MAX - 8) {
        ptn_abort_out_of_memory();
    }

    unsigned char *message = calloc(padded_len + 8, 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (input_len != 0) {
        memcpy(message, input, input_len);
    }
    message[input_len] = 0x80;

    uint64_t bit_len = (uint64_t)input_len * 8;
    for (size_t i = 0; i < 8; i++) {
        message[padded_len + i] = (unsigned char)(bit_len >> (8 * i));
    }

    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xefcdab89;
    uint32_t h2 = 0x98badcfe;
    uint32_t h3 = 0x10325476;

    for (size_t offset = 0; offset < padded_len; offset += 64) {
        uint32_t words[16];
        for (size_t i = 0; i < 16; i++) {
            size_t base = offset + i * 4;
            words[i] = (uint32_t)message[base]
                | ((uint32_t)message[base + 1] << 8)
                | ((uint32_t)message[base + 2] << 16)
                | ((uint32_t)message[base + 3] << 24);
        }

        uint32_t a = h0;
        uint32_t b = h1;
        uint32_t c = h2;
        uint32_t d = h3;

        for (uint32_t i = 0; i < 64; i++) {
            uint32_t f;
            uint32_t g;
            if (i < 16) {
                f = (b & c) | ((~b) & d);
                g = i;
            } else if (i < 32) {
                f = (d & b) | ((~d) & c);
                g = (5 * i + 1) % 16;
            } else if (i < 48) {
                f = b ^ c ^ d;
                g = (3 * i + 5) % 16;
            } else {
                f = c ^ (b | (~d));
                g = (7 * i) % 16;
            }

            uint32_t next = d;
            d = c;
            c = b;
            b = b + ptn_rotate_left32(a + f + constants[i] + words[g], shifts[i]);
            a = next;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
    }

    free(message);

    uint32_t words[4] = { h0, h1, h2, h3 };
    for (size_t i = 0; i < 4; i++) {
        digest[i * 4] = (unsigned char)words[i];
        digest[i * 4 + 1] = (unsigned char)(words[i] >> 8);
        digest[i * 4 + 2] = (unsigned char)(words[i] >> 16);
        digest[i * 4 + 3] = (unsigned char)(words[i] >> 24);
    }
}

static PtnValue ptn_internal_md5(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "md5", 1, "string", args[0], line);
    unsigned char digest[16];
    ptn_md5_digest_bytes((const unsigned char *)input.data, input.len, digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    ptn_string_operand_free(input);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static uint32_t ptn_crc32_bytes(const unsigned char *input, size_t input_len) {
    uint32_t crc = UINT32_C(0xffffffff);
    for (size_t i = 0; i < input_len; i++) {
        crc ^= (uint32_t)input[i];
        for (size_t bit = 0; bit < 8; bit++) {
            uint32_t mask = -(crc & UINT32_C(1));
            crc = (crc >> 1) ^ (UINT32_C(0xedb88320) & mask);
        }
    }
    return crc ^ UINT32_C(0xffffffff);
}

static PtnValue ptn_internal_crc32(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "crc32", 1, "string", args[0], line);
    uint32_t checksum = ptn_crc32_bytes((const unsigned char *)input.data, input.len);
    ptn_string_operand_free(input);
    return ptn_int((int64_t)checksum);
}

static void ptn_sha1_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[20]) {
    size_t padded_len = input_len + 1;
    while ((padded_len % 64) != 56) {
        padded_len++;
    }
    if (padded_len < input_len || padded_len > SIZE_MAX - 8) {
        ptn_abort_out_of_memory();
    }

    unsigned char *message = calloc(padded_len + 8, 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (input_len != 0) {
        memcpy(message, input, input_len);
    }
    message[input_len] = 0x80;

    uint64_t bit_len = (uint64_t)input_len * 8;
    for (size_t i = 0; i < 8; i++) {
        message[padded_len + 7 - i] = (unsigned char)(bit_len >> (8 * i));
    }

    uint32_t h0 = 0x67452301;
    uint32_t h1 = 0xefcdab89;
    uint32_t h2 = 0x98badcfe;
    uint32_t h3 = 0x10325476;
    uint32_t h4 = 0xc3d2e1f0;

    for (size_t offset = 0; offset < padded_len; offset += 64) {
        uint32_t words[80];
        for (size_t i = 0; i < 16; i++) {
            size_t base = offset + i * 4;
            words[i] = ((uint32_t)message[base] << 24)
                | ((uint32_t)message[base + 1] << 16)
                | ((uint32_t)message[base + 2] << 8)
                | (uint32_t)message[base + 3];
        }
        for (size_t i = 16; i < 80; i++) {
            words[i] = ptn_rotate_left32(words[i - 3] ^ words[i - 8] ^ words[i - 14] ^ words[i - 16], 1);
        }

        uint32_t a = h0;
        uint32_t b = h1;
        uint32_t c = h2;
        uint32_t d = h3;
        uint32_t e = h4;

        for (size_t i = 0; i < 80; i++) {
            uint32_t f;
            uint32_t k;
            if (i < 20) {
                f = (b & c) | ((~b) & d);
                k = 0x5a827999;
            } else if (i < 40) {
                f = b ^ c ^ d;
                k = 0x6ed9eba1;
            } else if (i < 60) {
                f = (b & c) | (b & d) | (c & d);
                k = 0x8f1bbcdc;
            } else {
                f = b ^ c ^ d;
                k = 0xca62c1d6;
            }

            uint32_t temp = ptn_rotate_left32(a, 5) + f + e + k + words[i];
            e = d;
            d = c;
            c = ptn_rotate_left32(b, 30);
            b = a;
            a = temp;
        }

        h0 += a;
        h1 += b;
        h2 += c;
        h3 += d;
        h4 += e;
    }

    free(message);

    uint32_t words[5] = { h0, h1, h2, h3, h4 };
    for (size_t i = 0; i < 5; i++) {
        digest[i * 4] = (unsigned char)(words[i] >> 24);
        digest[i * 4 + 1] = (unsigned char)(words[i] >> 16);
        digest[i * 4 + 2] = (unsigned char)(words[i] >> 8);
        digest[i * 4 + 3] = (unsigned char)words[i];
    }
}

static PtnValue ptn_internal_sha1(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "sha1", 1, "string", args[0], line);
    unsigned char digest[20];
    ptn_sha1_digest_bytes((const unsigned char *)input.data, input.len, digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    ptn_string_operand_free(input);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static PtnValue ptn_internal_pow(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_power(runtime, args[0], args[1], line);
}

static char *ptn_path_operand_to_c_string(PtnStringOperand path) {
    if (memchr(path.data, '\0', path.len) != NULL) {
        return NULL;
    }
    return ptn_duplicate_string_len(path.data, path.len);
}

static void ptn_emit_file_warning(
    PtnRuntime *runtime,
    const char *function_name,
    const char *path,
    const char *detail,
    size_t line
) {
    int needed = snprintf(NULL, 0, "%s(%s): %s", function_name, path, detail);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(message, (size_t)needed + 1, "%s(%s): %s", function_name, path, detail);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        fputc('\n', stdout);
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
    free(message);
}

static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);

static PtnValue ptn_internal_fopen(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_internal_expect_string_arg(runtime, "fopen", 1, "filename", args[0], line);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "fopen(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    PtnStringOperand mode_operand = ptn_internal_expect_string_arg(runtime, "fopen", 2, "mode", args[1], line);
    char *mode = ptn_path_operand_to_c_string(mode_operand);
    ptn_string_operand_free(mode_operand);
    if (mode == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "fopen(): Argument #2 ($mode) must not contain any null bytes", line);
        free(path);
        return ptn_bool(0);
    }

    FILE *stream = fopen(path, mode);
    if (stream == NULL) {
        char detail[192];
        int needed = snprintf(detail, sizeof(detail), "Failed to open stream: %s", strerror(errno));
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "fopen", path, detail, line);
        free(mode);
        free(path);
        return ptn_bool(0);
    }

    PtnValue resource = ptn_resource(ptn_resource_new_stream(stream, path, mode));
    free(mode);
    free(path);
    return resource;
}

static PtnValue ptn_internal_fclose(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnValue value = ptn_value_deref(args[0]);
    if (value.type != PTN_RESOURCE) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "fclose(): Argument #1 ($stream) must be of type resource, %s given",
            ptn_offset_container_type_name(value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    if (value.as.resource->stream == NULL) {
        return ptn_bool(0);
    }
    ptn_resource_close(value.as.resource);
    (void)line;
    return ptn_bool(1);
}

static PtnResource *ptn_internal_expect_open_stream_arg(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type != PTN_RESOURCE) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #1 ($stream) must be of type resource, %s given",
            function_name,
            ptn_offset_container_type_name(value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return NULL;
    }
    if (value.as.resource->stream == NULL) {
        char message[128];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #1 ($stream) must be an open stream resource",
            function_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return NULL;
    }
    (void)line;
    return value.as.resource;
}

static PtnValue ptn_internal_fwrite_named(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    PtnResource *resource = ptn_internal_expect_open_stream_arg(runtime, function_name, args[0], line);
    if (resource == NULL) {
        return ptn_null();
    }
    PtnStringOperand data = ptn_internal_expect_string_arg(runtime, function_name, 2, "data", args[1], line);
    size_t length = data.len;
    if (argc >= 3 && ptn_value_deref(args[2]).type != PTN_NULL) {
        int64_t requested = ptn_internal_expect_integer_arg(runtime, function_name, 3, "length", args[2], line);
        if (requested < 0) {
            ptn_string_operand_free(data);
            char message[128];
            int written = snprintf(
                message,
                sizeof(message),
                "%s(): Argument #3 ($length) must be greater than or equal to 0",
                function_name
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "ValueError", message);
            return ptn_null();
        }
        if ((uint64_t)requested < length) {
            length = (size_t)requested;
        }
    }

    size_t written = fwrite(data.data, 1, length, resource->stream);
    ptn_string_operand_free(data);
    if (written != length && ferror(resource->stream)) {
        clearerr(resource->stream);
        return ptn_bool(0);
    }
    if (written > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_int((int64_t)written);
}

static PtnValue ptn_internal_fwrite(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_fwrite_named(runtime, "fwrite", argc, args, line);
}

static PtnValue ptn_internal_fputs(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    return ptn_internal_fwrite_named(runtime, "fputs", argc, args, line);
}

static void ptn_stream_meta_set(PtnArray *array, const char *key, PtnValue value) {
    ptn_array_set_entry(array, ptn_array_string_key(key), value);
}

static PtnValue ptn_internal_stream_get_meta_data(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnValue value = ptn_value_deref(args[0]);
    if (value.type != PTN_RESOURCE) {
        char message[224];
        int written = snprintf(
            message,
            sizeof(message),
            "stream_get_meta_data(): Argument #1 ($stream) must be of type resource, %s given",
            ptn_offset_container_type_name(value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }
    if (value.as.resource->stream == NULL) {
        ptn_throw_exception(
            runtime,
            "TypeError",
            "stream_get_meta_data(): Argument #1 ($stream) must be an open stream resource"
        );
        return ptn_null();
    }

    PtnResource *resource = value.as.resource;
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    ptn_stream_meta_set(result.as.array, "timed_out", ptn_bool(0));
    ptn_stream_meta_set(result.as.array, "blocked", ptn_bool(1));
    ptn_stream_meta_set(result.as.array, "eof", ptn_bool(feof(resource->stream) != 0));
    ptn_stream_meta_set(result.as.array, "wrapper_type", ptn_string("plainfile"));
    ptn_stream_meta_set(result.as.array, "stream_type", ptn_string("STDIO"));
    ptn_stream_meta_set(
        result.as.array,
        "mode",
        ptn_owned_string(ptn_duplicate_string(resource->stream_mode == NULL ? "" : resource->stream_mode))
    );
    ptn_stream_meta_set(result.as.array, "unread_bytes", ptn_int(0));
    ptn_stream_meta_set(result.as.array, "seekable", ptn_bool(1));
    ptn_stream_meta_set(
        result.as.array,
        "uri",
        ptn_owned_string(ptn_duplicate_string(resource->stream_uri == NULL ? "" : resource->stream_uri))
    );
    return result;
}

static PtnValue ptn_internal_file_put_contents(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "file_put_contents(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    PtnStringOperand data = ptn_value_to_string_operand(args[1]);
    FILE *stream = fopen(path, "wb");
    if (stream == NULL) {
        char detail[192];
        int needed = snprintf(detail, sizeof(detail), "Failed to open stream: %s", strerror(errno));
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "file_put_contents", path, detail, line);
        free(path);
        ptn_string_operand_free(data);
        return ptn_bool(0);
    }

    size_t written = fwrite(data.data, 1, data.len, stream);
    int failed = written != data.len || fclose(stream) != 0;
    if (failed) {
        char detail[192];
        int needed = snprintf(detail, sizeof(detail), "Failed to write %zu bytes: %s", data.len, strerror(errno));
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "file_put_contents", path, detail, line);
        free(path);
        ptn_string_operand_free(data);
        return ptn_bool(0);
    }

    if (data.len > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    PtnValue result = ptn_int((int64_t)data.len);
    free(path);
    ptn_string_operand_free(data);
    return result;
}

static int ptn_read_file_bytes(const char *path, unsigned char **data_out, size_t *len_out);
static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);

static PtnValue ptn_internal_file_get_contents(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "file_get_contents(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    int64_t offset = argc >= 4
        ? ptn_internal_expect_integer_arg(runtime, "file_get_contents", 4, "offset", args[3], line)
        : 0;
    int has_length = 0;
    int64_t length = 0;
    if (argc >= 5 && ptn_value_deref(args[4]).type != PTN_NULL) {
        length = ptn_internal_expect_integer_arg(runtime, "file_get_contents", 5, "length", args[4], line);
        if (length < 0) {
            ptn_throw_exception(
                runtime,
                "ValueError",
                "file_get_contents(): Argument #5 ($length) must be greater than or equal to 0"
            );
            free(path);
            return ptn_null();
        }
        has_length = 1;
    }

    unsigned char *data = NULL;
    size_t data_len = 0;
    int read_result = ptn_read_file_bytes(path, &data, &data_len);
    if (read_result <= 0) {
        char detail[192];
        int needed = snprintf(
            detail,
            sizeof(detail),
            "%s: %s",
            read_result == 0 ? "Failed to open stream" : "Failed to read stream",
            strerror(errno)
        );
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "file_get_contents", path, detail, line);
        free(path);
        free(data);
        return ptn_bool(0);
    }
    free(path);

    int64_t start_offset = offset;
    if (start_offset < 0) {
        uint64_t distance = start_offset == INT64_MIN
            ? (uint64_t)INT64_MAX + 1
            : (uint64_t)(-start_offset);
        if (distance > data_len) {
            start_offset = 0;
        } else {
            start_offset = (int64_t)data_len + start_offset;
        }
    }
    size_t start = start_offset <= 0 ? 0 : (size_t)start_offset;
    if (start > data_len) {
        start = data_len;
    }

    size_t available = data_len - start;
    size_t result_len = available;
    if (has_length && (uint64_t)length < result_len) {
        result_len = (size_t)length;
    }
    char *copy = ptn_duplicate_string_len((const char *)data + start, result_len);
    free(data);
    return ptn_owned_string_len(copy, result_len);
}

static int ptn_read_file_bytes(const char *path, unsigned char **data_out, size_t *len_out) {
    FILE *stream = fopen(path, "rb");
    if (stream == NULL) {
        return 0;
    }

    unsigned char *data = NULL;
    size_t len = 0;
    size_t capacity = 0;
    unsigned char chunk[4096];
    for (;;) {
        size_t read_len = fread(chunk, 1, sizeof(chunk), stream);
        if (read_len != 0) {
            if (read_len > SIZE_MAX - len) {
                fclose(stream);
                free(data);
                ptn_abort_out_of_memory();
            }
            size_t required = len + read_len;
            if (required > capacity) {
                size_t new_capacity = capacity == 0 ? 4096 : capacity;
                while (new_capacity < required) {
                    if (new_capacity > SIZE_MAX / 2) {
                        fclose(stream);
                        free(data);
                        ptn_abort_out_of_memory();
                    }
                    new_capacity *= 2;
                }
                unsigned char *new_data = realloc(data, new_capacity);
                if (new_data == NULL) {
                    fclose(stream);
                    free(data);
                    ptn_abort_out_of_memory();
                }
                data = new_data;
                capacity = new_capacity;
            }
            memcpy(data + len, chunk, read_len);
            len = required;
        }
        if (read_len < sizeof(chunk)) {
            if (ferror(stream)) {
                fclose(stream);
                free(data);
                return -1;
            }
            break;
        }
    }

    if (fclose(stream) != 0) {
        free(data);
        return -1;
    }
    *data_out = data;
    *len_out = len;
    return 1;
}

static PtnValue ptn_internal_sha1_file(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "sha1_file(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    unsigned char *data = NULL;
    size_t data_len = 0;
    int read_result = ptn_read_file_bytes(path, &data, &data_len);
    if (read_result <= 0) {
        char detail[192];
        int needed = snprintf(
            detail,
            sizeof(detail),
            "%s: %s",
            read_result == 0 ? "Failed to open stream" : "Failed to read stream",
            strerror(errno)
        );
        if (needed < 0 || (size_t)needed >= sizeof(detail)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_file_warning(runtime, "sha1_file", path, detail, line);
        free(path);
        free(data);
        return ptn_bool(0);
    }

    unsigned char digest[20];
    ptn_sha1_digest_bytes(data, data_len, digest);
    int raw_output = argc >= 2 && ptn_is_truthy(args[1]);
    free(path);
    free(data);
    return ptn_digest_value(digest, sizeof(digest), raw_output);
}

static PtnValue ptn_internal_unlink(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "unlink(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    if (remove(path) == 0) {
        free(path);
        return ptn_bool(1);
    }

    ptn_emit_file_warning(runtime, "unlink", path, strerror(errno), line);
    free(path);
    return ptn_bool(0);
}

static int ptn_path_is_separator(char byte) {
    if (byte == '/') {
        return 1;
    }
#if defined(_WIN32)
    if (byte == '\\') {
        return 1;
    }
#endif
    return 0;
}

static PtnValue ptn_internal_basename(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path = ptn_internal_expect_string_arg(runtime, "basename", 1, "path", args[0], line);
    PtnStringOperand suffix = argc >= 2
        ? ptn_internal_expect_string_arg(runtime, "basename", 2, "suffix", args[1], line)
        : ptn_string_operand_borrowed("");

    size_t end = path.len;
    while (end > 0 && ptn_path_is_separator(path.data[end - 1])) {
        end--;
    }

    size_t start = end;
    while (start > 0 && !ptn_path_is_separator(path.data[start - 1])) {
        start--;
    }

    size_t basename_len = end - start;
    if (
        suffix.len != 0 &&
        suffix.len < basename_len &&
        memcmp(path.data + start + basename_len - suffix.len, suffix.data, suffix.len) == 0
    ) {
        basename_len -= suffix.len;
    }

    char *basename = ptn_duplicate_string_len(path.data + start, basename_len);
    ptn_string_operand_free(suffix);
    ptn_string_operand_free(path);
    return ptn_owned_string_len(basename, basename_len);
}

static int ptn_stat_path(const char *path, struct stat *info) {
    return stat(path, info);
}

static int ptn_lstat_path(const char *path, struct stat *info) {
#if defined(_WIN32)
    return stat(path, info);
#else
    return lstat(path, info);
#endif
}

static int ptn_path_exists_c(const char *path) {
    struct stat info;
    return ptn_stat_path(path, &info) == 0;
}

static int ptn_path_is_directory_c(const char *path) {
    struct stat info;
    return ptn_stat_path(path, &info) == 0 && S_ISDIR(info.st_mode);
}

static int ptn_path_is_regular_file_c(const char *path) {
    struct stat info;
    return ptn_stat_path(path, &info) == 0 && S_ISREG(info.st_mode);
}

static int ptn_path_is_link_c(const char *path) {
    struct stat info;
    if (ptn_lstat_path(path, &info) != 0) {
        return 0;
    }
#if defined(S_ISLNK)
    return S_ISLNK(info.st_mode);
#else
    return 0;
#endif
}

static int ptn_platform_access(const char *path, int mode) {
#if defined(_WIN32)
    return _access(path, mode) == 0;
#else
    return access(path, mode) == 0;
#endif
}

static int ptn_path_is_readable_c(const char *path) {
    return ptn_platform_access(path, R_OK);
}

static int ptn_path_is_writable_c(const char *path) {
    return ptn_platform_access(path, W_OK);
}

static int ptn_path_is_executable_c(const char *path) {
    return ptn_platform_access(path, X_OK);
}

static int ptn_platform_mkdir(const char *path, int64_t mode) {
#if defined(_WIN32)
    (void)mode;
    return _mkdir(path);
#else
    return mkdir(path, (mode_t)mode);
#endif
}

static int64_t ptn_mkdir_mode_from_args(size_t argc, const PtnValue *args) {
    if (argc < 2) {
        return 0777;
    }
    return ptn_value_to_integer(args[1]);
}

static int ptn_mkdir_existing_parent_ok(const char *path, int64_t mode) {
    if (ptn_platform_mkdir(path, mode) == 0) {
        return 1;
    }
    int saved_errno = errno;
    if (saved_errno == EEXIST && ptn_path_is_directory_c(path)) {
        return 1;
    }
    errno = saved_errno;
    return 0;
}

static void ptn_trim_trailing_path_separators(char *path) {
    size_t len = strlen(path);
    while (len > 1 && ptn_path_is_separator(path[len - 1])) {
        path[len - 1] = '\0';
        len--;
    }
}

static int ptn_mkdir_recursive(const char *path, int64_t mode) {
    char *work = ptn_duplicate_string(path);
    ptn_trim_trailing_path_separators(work);
    size_t len = strlen(work);
    size_t index = 0;

#if defined(_WIN32)
    if (len >= 2 && work[1] == ':') {
        index = 2;
    }
#endif
    while (index < len && ptn_path_is_separator(work[index])) {
        index++;
    }

    for (; index < len; index++) {
        if (!ptn_path_is_separator(work[index])) {
            continue;
        }
        char separator = work[index];
        work[index] = '\0';
        if (work[0] != '\0' && !ptn_mkdir_existing_parent_ok(work, mode)) {
            int saved_errno = errno;
            free(work);
            errno = saved_errno;
            return 0;
        }
        work[index] = separator;
        while (index + 1 < len && ptn_path_is_separator(work[index + 1])) {
            index++;
        }
    }

    int result = ptn_platform_mkdir(work, mode) == 0;
    int saved_errno = errno;
    free(work);
    errno = saved_errno;
    return result;
}

static PtnValue ptn_path_predicate(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue path_value,
    size_t line,
    int (*predicate)(const char *)
) {
    PtnStringOperand path_operand = ptn_value_to_string_operand(path_value);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        char message[96];
        int written = snprintf(message, sizeof(message), "%s(): Filename contains null byte", function_name);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_warning(&runtime->diagnostics, message, line);
        return ptn_bool(0);
    }

    int result = predicate(path);
    free(path);
    return ptn_bool(result);
}

static PtnValue ptn_internal_file_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "file_exists", args[0], line, ptn_path_exists_c);
}

static void ptn_emit_stat_warning(
    PtnRuntime *runtime,
    const char *function_name,
    const char *failure_kind,
    const char *path,
    size_t line
) {
    int needed = snprintf(NULL, 0, "%s(): %s failed for %s", function_name, failure_kind, path);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(message, (size_t)needed + 1, "%s(): %s failed for %s", function_name, failure_kind, path);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        fputc('\n', stdout);
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
    free(message);
}

static void ptn_emit_function_warning(
    PtnRuntime *runtime,
    const char *function_name,
    const char *detail,
    size_t line
) {
    int needed = snprintf(NULL, 0, "%s(): %s", function_name, detail);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(message, (size_t)needed + 1, "%s(): %s", function_name, detail);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        fputc('\n', stdout);
    }
    ptn_emit_warning(&runtime->diagnostics, message, line);
    free(message);
}

typedef enum {
    PTN_STAT_FIELD_DEV,
    PTN_STAT_FIELD_INO,
    PTN_STAT_FIELD_MODE,
    PTN_STAT_FIELD_NLINK,
    PTN_STAT_FIELD_UID,
    PTN_STAT_FIELD_GID,
    PTN_STAT_FIELD_RDEV,
    PTN_STAT_FIELD_SIZE,
    PTN_STAT_FIELD_ATIME,
    PTN_STAT_FIELD_MTIME,
    PTN_STAT_FIELD_CTIME,
    PTN_STAT_FIELD_BLKSIZE,
    PTN_STAT_FIELD_BLOCKS
} PtnStatField;

static int64_t ptn_stat_field_value(const struct stat *info, PtnStatField field) {
    switch (field) {
        case PTN_STAT_FIELD_DEV:
            return (int64_t)info->st_dev;
        case PTN_STAT_FIELD_INO:
            return (int64_t)info->st_ino;
        case PTN_STAT_FIELD_MODE:
            return (int64_t)info->st_mode;
        case PTN_STAT_FIELD_NLINK:
            return (int64_t)info->st_nlink;
        case PTN_STAT_FIELD_UID:
            return (int64_t)info->st_uid;
        case PTN_STAT_FIELD_GID:
            return (int64_t)info->st_gid;
        case PTN_STAT_FIELD_RDEV:
            return (int64_t)info->st_rdev;
        case PTN_STAT_FIELD_SIZE:
            return (int64_t)info->st_size;
        case PTN_STAT_FIELD_ATIME:
            return (int64_t)info->st_atime;
        case PTN_STAT_FIELD_MTIME:
            return (int64_t)info->st_mtime;
        case PTN_STAT_FIELD_CTIME:
            return (int64_t)info->st_ctime;
        case PTN_STAT_FIELD_BLKSIZE:
#if defined(_WIN32)
            return -1;
#else
            return (int64_t)info->st_blksize;
#endif
        case PTN_STAT_FIELD_BLOCKS:
#if defined(_WIN32)
            return -1;
#else
            return (int64_t)info->st_blocks;
#endif
    }
    return 0;
}

static void ptn_stat_array_set(PtnValue *array, int64_t index, const char *name, int64_t value) {
    ptn_array_set_entry(array->as.array, ptn_array_int_key(index), ptn_int(value));
    ptn_array_set_entry(array->as.array, ptn_array_string_key(name), ptn_int(value));
}

static PtnValue ptn_stat_array_from_info(const struct stat *info) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    ptn_stat_array_set(&result, 0, "dev", ptn_stat_field_value(info, PTN_STAT_FIELD_DEV));
    ptn_stat_array_set(&result, 1, "ino", ptn_stat_field_value(info, PTN_STAT_FIELD_INO));
    ptn_stat_array_set(&result, 2, "mode", ptn_stat_field_value(info, PTN_STAT_FIELD_MODE));
    ptn_stat_array_set(&result, 3, "nlink", ptn_stat_field_value(info, PTN_STAT_FIELD_NLINK));
    ptn_stat_array_set(&result, 4, "uid", ptn_stat_field_value(info, PTN_STAT_FIELD_UID));
    ptn_stat_array_set(&result, 5, "gid", ptn_stat_field_value(info, PTN_STAT_FIELD_GID));
    ptn_stat_array_set(&result, 6, "rdev", ptn_stat_field_value(info, PTN_STAT_FIELD_RDEV));
    ptn_stat_array_set(&result, 7, "size", ptn_stat_field_value(info, PTN_STAT_FIELD_SIZE));
    ptn_stat_array_set(&result, 8, "atime", ptn_stat_field_value(info, PTN_STAT_FIELD_ATIME));
    ptn_stat_array_set(&result, 9, "mtime", ptn_stat_field_value(info, PTN_STAT_FIELD_MTIME));
    ptn_stat_array_set(&result, 10, "ctime", ptn_stat_field_value(info, PTN_STAT_FIELD_CTIME));
    ptn_stat_array_set(&result, 11, "blksize", ptn_stat_field_value(info, PTN_STAT_FIELD_BLKSIZE));
    ptn_stat_array_set(&result, 12, "blocks", ptn_stat_field_value(info, PTN_STAT_FIELD_BLOCKS));
    return result;
}

static int ptn_stat_path_from_value(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue path_value,
    size_t line,
    int use_lstat,
    const char *failure_kind,
    struct stat *info
) {
    PtnStringOperand path_operand = ptn_value_to_string_operand(path_value);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        char message[96];
        int written = snprintf(message, sizeof(message), "%s(): Filename contains null byte", function_name);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
            fputc('\n', stdout);
        }
        ptn_emit_warning(&runtime->diagnostics, message, line);
        return 0;
    }
    if (path[0] == '\0') {
        free(path);
        return 0;
    }

    int ok = use_lstat ? ptn_lstat_path(path, info) == 0 : ptn_stat_path(path, info) == 0;
    if (!ok) {
        ptn_emit_stat_warning(runtime, function_name, failure_kind, path, line);
        free(path);
        return 0;
    }
    free(path);
    return 1;
}

static PtnValue ptn_internal_stat_named(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue path_value,
    size_t line,
    int use_lstat,
    const char *failure_kind
) {
    struct stat info;
    if (!ptn_stat_path_from_value(runtime, function_name, path_value, line, use_lstat, failure_kind, &info)) {
        return ptn_bool(0);
    }
    return ptn_stat_array_from_info(&info);
}

static PtnValue ptn_internal_file_stat_field(
    PtnRuntime *runtime,
    const char *function_name,
    PtnValue path_value,
    size_t line,
    int use_lstat,
    const char *failure_kind,
    PtnStatField field
) {
    struct stat info;
    if (!ptn_stat_path_from_value(runtime, function_name, path_value, line, use_lstat, failure_kind, &info)) {
        return ptn_bool(0);
    }
    return ptn_int(ptn_stat_field_value(&info, field));
}

static const char *ptn_filetype_from_mode(mode_t mode) {
    if (S_ISREG(mode)) {
        return "file";
    }
    if (S_ISDIR(mode)) {
        return "dir";
    }
#if defined(S_ISLNK)
    if (S_ISLNK(mode)) {
        return "link";
    }
#endif
#if defined(S_ISCHR)
    if (S_ISCHR(mode)) {
        return "char";
    }
#endif
#if defined(S_ISBLK)
    if (S_ISBLK(mode)) {
        return "block";
    }
#endif
#if defined(S_ISFIFO)
    if (S_ISFIFO(mode)) {
        return "fifo";
    }
#endif
#if defined(S_ISSOCK)
    if (S_ISSOCK(mode)) {
        return "socket";
    }
#endif
    return "unknown";
}

static PtnValue ptn_internal_filetype(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    struct stat info;
    if (!ptn_stat_path_from_value(runtime, "filetype", args[0], line, 1, "Lstat", &info)) {
        return ptn_bool(0);
    }
    return ptn_string(ptn_filetype_from_mode(info.st_mode));
}

static PtnValue ptn_internal_stat(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_stat_named(runtime, "stat", args[0], line, 0, "stat");
}

static PtnValue ptn_internal_lstat(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_stat_named(runtime, "lstat", args[0], line, 1, "Lstat");
}

static PtnValue ptn_internal_fileatime(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "fileatime", args[0], line, 0, "stat", PTN_STAT_FIELD_ATIME);
}

static PtnValue ptn_internal_filectime(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "filectime", args[0], line, 0, "stat", PTN_STAT_FIELD_CTIME);
}

static PtnValue ptn_internal_filegroup(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "filegroup", args[0], line, 0, "stat", PTN_STAT_FIELD_GID);
}

static PtnValue ptn_internal_fileinode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "fileinode", args[0], line, 0, "stat", PTN_STAT_FIELD_INO);
}

static PtnValue ptn_internal_filemtime(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "filemtime", args[0], line, 0, "stat", PTN_STAT_FIELD_MTIME);
}

static PtnValue ptn_internal_fileowner(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "fileowner", args[0], line, 0, "stat", PTN_STAT_FIELD_UID);
}

static PtnValue ptn_internal_fileperms(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "fileperms", args[0], line, 0, "stat", PTN_STAT_FIELD_MODE);
}

static PtnValue ptn_internal_filesize(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_internal_file_stat_field(runtime, "filesize", args[0], line, 0, "stat", PTN_STAT_FIELD_SIZE);
}

static PtnValue ptn_internal_clearstatcache(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_null();
}

static PtnValue ptn_internal_chmod(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "chmod(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    int64_t permissions = ptn_internal_expect_integer_arg(runtime, "chmod", 2, "permissions", args[1], line);
#if defined(_WIN32)
    int ok = _chmod(path, (int)permissions) == 0;
#else
    int ok = chmod(path, (mode_t)permissions) == 0;
#endif
    if (ok) {
        free(path);
        return ptn_bool(1);
    }

    ptn_emit_function_warning(runtime, "chmod", strerror(errno), line);
    free(path);
    return ptn_bool(0);
}

static int ptn_platform_touch_times(const char *path, int64_t atime_value, int64_t mtime_value) {
#if defined(_WIN32)
    struct _utimbuf times;
    times.actime = (time_t)atime_value;
    times.modtime = (time_t)mtime_value;
    return _utime(path, &times) == 0;
#else
    struct utimbuf times;
    times.actime = (time_t)atime_value;
    times.modtime = (time_t)mtime_value;
    return utime(path, &times) == 0;
#endif
}

static PtnValue ptn_internal_touch(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "touch(): Filename contains null byte", line);
        return ptn_bool(0);
    }
    if (path[0] == '\0') {
        free(path);
        return ptn_bool(0);
    }

    int64_t mtime_value = argc >= 2 && ptn_value_deref(args[1]).type != PTN_NULL
        ? ptn_internal_expect_integer_arg(runtime, "touch", 2, "mtime", args[1], line)
        : (int64_t)time(NULL);
    int64_t atime_value = argc >= 3 && ptn_value_deref(args[2]).type != PTN_NULL
        ? ptn_internal_expect_integer_arg(runtime, "touch", 3, "atime", args[2], line)
        : mtime_value;

    if (ptn_path_exists_c(path)) {
        if (ptn_platform_touch_times(path, atime_value, mtime_value)) {
            free(path);
            return ptn_bool(1);
        }
        ptn_emit_file_warning(runtime, "touch", path, strerror(errno), line);
        free(path);
        return ptn_bool(0);
    }

    FILE *stream = fopen(path, "ab");
    if (stream == NULL) {
        int needed = snprintf(NULL, 0, "touch(): Unable to create file %s because %s", path, strerror(errno));
        if (needed < 0) {
            ptn_abort_out_of_memory();
        }
        char *message = malloc((size_t)needed + 1);
        if (message == NULL) {
            ptn_abort_out_of_memory();
        }
        int written = snprintf(message, (size_t)needed + 1, "touch(): Unable to create file %s because %s", path, strerror(errno));
        if (written < 0 || written != needed) {
            free(message);
            ptn_abort_out_of_memory();
        }
        if (ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
            fputc('\n', stdout);
        }
        ptn_emit_warning(&runtime->diagnostics, message, line);
        free(message);
        free(path);
        return ptn_bool(0);
    }
    fclose(stream);
    if (!ptn_platform_touch_times(path, atime_value, mtime_value)) {
        ptn_emit_file_warning(runtime, "touch", path, strerror(errno), line);
        free(path);
        return ptn_bool(0);
    }
    free(path);
    return ptn_bool(1);
}

static char *ptn_platform_getcwd_alloc(void) {
    size_t capacity = 256;
    for (;;) {
        if (capacity > (size_t)INT_MAX) {
            errno = ERANGE;
            return NULL;
        }
        char *buffer = malloc(capacity);
        if (buffer == NULL) {
            ptn_abort_out_of_memory();
        }
#if defined(_WIN32)
        if (_getcwd(buffer, (int)capacity) != NULL) {
            return buffer;
        }
#else
        if (getcwd(buffer, capacity) != NULL) {
            return buffer;
        }
#endif
        int saved_errno = errno;
        free(buffer);
        if (saved_errno == ERANGE && capacity <= SIZE_MAX / 2) {
            capacity *= 2;
            continue;
        }
        errno = saved_errno;
        return NULL;
    }
}

static int ptn_platform_chdir(const char *path) {
#if defined(_WIN32)
    return _chdir(path);
#else
    return chdir(path);
#endif
}

static PtnValue ptn_internal_getcwd(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    char *cwd = ptn_platform_getcwd_alloc();
    if (cwd == NULL) {
        return ptn_bool(0);
    }
    return ptn_owned_string(cwd);
}

static PtnValue ptn_internal_chdir(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_internal_expect_string_arg(runtime, "chdir", 1, "directory", args[0], line);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "chdir(): Path must not contain any null bytes", line);
        return ptn_bool(0);
    }

    if (ptn_platform_chdir(path) == 0) {
        free(path);
        return ptn_bool(1);
    }

    ptn_emit_file_warning(runtime, "chdir", path, strerror(errno), line);
    free(path);
    return ptn_bool(0);
}

static PtnValue ptn_internal_realpath(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "realpath(): Filename contains null byte", line);
        return ptn_bool(0);
    }

#if defined(_WIN32)
    char *resolved = _fullpath(NULL, path, 0);
#else
    char *resolved = realpath(path, NULL);
#endif
    free(path);
    if (resolved == NULL) {
        return ptn_bool(0);
    }
    return ptn_owned_string(resolved);
}

static PtnValue ptn_internal_is_dir(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "is_dir", args[0], line, ptn_path_is_directory_c);
}

static PtnValue ptn_internal_is_executable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "is_executable", args[0], line, ptn_path_is_executable_c);
}

static PtnValue ptn_internal_is_file(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "is_file", args[0], line, ptn_path_is_regular_file_c);
}

static PtnValue ptn_internal_is_link(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "is_link", args[0], line, ptn_path_is_link_c);
}

static PtnValue ptn_internal_is_readable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "is_readable", args[0], line, ptn_path_is_readable_c);
}

static PtnValue ptn_internal_is_writable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "is_writable", args[0], line, ptn_path_is_writable_c);
}

static PtnValue ptn_internal_is_writeable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_path_predicate(runtime, "is_writeable", args[0], line, ptn_path_is_writable_c);
}

static int ptn_scandir_name_compare(const void *left, const void *right) {
    const char *left_name = *(const char * const *)left;
    const char *right_name = *(const char * const *)right;
    return strcmp(left_name, right_name);
}

static PtnValue ptn_internal_scandir(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "scandir(): Filename contains null byte", line);
        return ptn_bool(0);
    }

#if defined(_WIN32)
    ptn_emit_file_warning(runtime, "scandir", path, "directory scanning is unsupported on this platform", line);
    free(path);
    return ptn_bool(0);
#else
    DIR *dir = opendir(path);
    if (dir == NULL) {
        ptn_emit_file_warning(runtime, "scandir", path, strerror(errno), line);
        free(path);
        return ptn_bool(0);
    }

    char **names = NULL;
    size_t len = 0;
    size_t capacity = 0;
    struct dirent *entry;
    while ((entry = readdir(dir)) != NULL) {
        if (len == capacity) {
            size_t new_capacity = capacity == 0 ? 16 : capacity * 2;
            if (new_capacity < capacity) {
                closedir(dir);
                free(path);
                ptn_abort_out_of_memory();
            }
            char **new_names = realloc(names, new_capacity * sizeof(char *));
            if (new_names == NULL) {
                closedir(dir);
                free(path);
                ptn_abort_out_of_memory();
            }
            names = new_names;
            capacity = new_capacity;
        }
        names[len++] = ptn_duplicate_string(entry->d_name);
    }
    closedir(dir);
    free(path);
    qsort(names, len, sizeof(char *), ptn_scandir_name_compare);

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < len; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(result.as.array, ptn_array_int_key((int64_t)i), ptn_owned_string(names[i]));
    }
    free(names);
    return result;
#endif
}

static PtnValue ptn_internal_mkdir(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "mkdir(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    int64_t mode = ptn_mkdir_mode_from_args(argc, args);
    int recursive = argc >= 3 && ptn_is_truthy(args[2]);
    int created = recursive ? ptn_mkdir_recursive(path, mode) : ptn_platform_mkdir(path, mode) == 0;
    if (created) {
        free(path);
        return ptn_bool(1);
    }

    ptn_emit_file_warning(runtime, "mkdir", path, strerror(errno), line);
    free(path);
    return ptn_bool(0);
}

static PtnValue ptn_internal_rmdir(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path_operand = ptn_value_to_string_operand(args[0]);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "rmdir(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    if (rmdir(path) == 0) {
        free(path);
        return ptn_bool(1);
    }

    ptn_emit_file_warning(runtime, "rmdir", path, strerror(errno), line);
    free(path);
    return ptn_bool(0);
}

static size_t ptn_substr_clamped_positive(int64_t value, size_t limit) {
    if (value <= 0) {
        return 0;
    }
    uint64_t unsigned_value = (uint64_t)value;
    if (unsigned_value > (uint64_t)limit) {
        return limit;
    }
    return (size_t)unsigned_value;
}

static size_t ptn_substr_clamped_negative_distance(int64_t value, size_t limit) {
    if (value >= 0) {
        return 0;
    }
    if (value == INT64_MIN) {
        return limit;
    }
    uint64_t distance = (uint64_t)(-value);
    if (distance > (uint64_t)limit) {
        return limit;
    }
    return (size_t)distance;
}

static size_t ptn_substr_start_offset(size_t string_len, int64_t start) {
    if (start >= 0) {
        return ptn_substr_clamped_positive(start, string_len);
    }
    size_t distance = ptn_substr_clamped_negative_distance(start, string_len);
    return string_len - distance;
}

static char *ptn_substr_copy(const char *string, size_t start, size_t len) {
    if (len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    char *result = malloc(len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(result, string + start, len);
    result[len] = '\0';
    return result;
}

static PtnValue ptn_internal_substr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "substr", 1, "string", args[0], line);
    size_t string_len = string.len;
    size_t start = ptn_substr_start_offset(string_len, ptn_value_to_integer(args[1]));
    size_t end = string_len;

    if (argc >= 3 && args[2].type != PTN_NULL) {
        int64_t length = ptn_value_to_integer(args[2]);
        if (length >= 0) {
            size_t requested_len = ptn_substr_clamped_positive(length, string_len);
            size_t available_len = string_len - start;
            if (requested_len > available_len) {
                requested_len = available_len;
            }
            end = start + requested_len;
        } else {
            size_t truncate_len = ptn_substr_clamped_negative_distance(length, string_len);
            end = string_len - truncate_len;
            if (end < start) {
                end = start;
            }
        }
    }

    char *substring = ptn_substr_copy(string.data, start, end - start);
    size_t substring_len = end - start;
    ptn_string_operand_free(string);
    return ptn_owned_string_len(substring, substring_len);
}

static int ptn_is_path_separator(char byte) {
#ifdef _WIN32
    return byte == '/' || byte == '\\';
#else
    return byte == '/';
#endif
}

static char *ptn_dirname_string(const char *path, size_t len, size_t *dirname_len) {
    if (len == 0) {
        *dirname_len = 0;
        return ptn_duplicate_string("");
    }
    while (len > 1 && ptn_is_path_separator(path[len - 1])) {
        len--;
    }

    size_t end = len;
    while (end > 0 && !ptn_is_path_separator(path[end - 1])) {
        end--;
    }
    if (end == 0) {
        *dirname_len = 1;
        return ptn_duplicate_string(".");
    }
    while (end > 1 && ptn_is_path_separator(path[end - 1])) {
        end--;
    }

    char *dirname = malloc(end + 1);
    if (dirname == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(dirname, path, end);
    dirname[end] = '\0';
    *dirname_len = end;
    return dirname;
}

static char *ptn_dirname_string_levels(const char *path, size_t len, int64_t levels, size_t *dirname_len) {
    char *current = ptn_duplicate_string_len(path, len);
    size_t current_len = len;

    for (int64_t level = 0; level < levels; level++) {
        size_t next_len = 0;
        char *next = ptn_dirname_string(current, current_len, &next_len);
        if (next_len == current_len && memcmp(next, current, current_len) == 0) {
            free(current);
            *dirname_len = next_len;
            return next;
        }
        free(current);
        current = next;
        current_len = next_len;
    }

    *dirname_len = current_len;
    return current;
}

typedef struct {
    char *dirname;
    size_t dirname_len;
    size_t basename_start;
    size_t basename_len;
    size_t extension_start;
    size_t extension_len;
    size_t filename_len;
    int has_extension;
} PtnPathInfoParts;

static PtnPathInfoParts ptn_pathinfo_parts(PtnStringOperand path) {
    PtnPathInfoParts parts;
    parts.dirname = ptn_dirname_string(path.data, path.len, &parts.dirname_len);

    size_t end = path.len;
    while (end > 0 && ptn_is_path_separator(path.data[end - 1])) {
        end--;
    }

    size_t start = end;
    while (start > 0 && !ptn_is_path_separator(path.data[start - 1])) {
        start--;
    }
    parts.basename_start = start;
    parts.basename_len = end - start;
    parts.has_extension = 0;
    parts.extension_start = start + parts.basename_len;
    parts.extension_len = 0;
    parts.filename_len = parts.basename_len;

    size_t dot = SIZE_MAX;
    for (size_t i = 0; i < parts.basename_len; i++) {
        if (path.data[start + i] == '.') {
            dot = i;
        }
    }
    if (dot != SIZE_MAX) {
        parts.has_extension = 1;
        parts.extension_start = start + dot + 1;
        parts.extension_len = parts.basename_len - dot - 1;
        parts.filename_len = dot;
    }

    return parts;
}

static PtnValue ptn_pathinfo_owned_string(const char *data, size_t len) {
    return ptn_owned_string_len(ptn_duplicate_string_len(data, len), len);
}

static void ptn_pathinfo_array_set_string(PtnValue *result, const char *key, const char *data, size_t len) {
    ptn_array_set_entry(
        result->as.array,
        ptn_array_string_key(key),
        ptn_pathinfo_owned_string(data, len)
    );
}

static PtnValue ptn_internal_pathinfo(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path = ptn_internal_expect_string_arg(runtime, "pathinfo", 1, "path", args[0], line);
    int64_t flags = argc >= 2
        ? ptn_internal_expect_integer_arg(runtime, "pathinfo", 2, "flags", args[1], line)
        : PTN_PATHINFO_ALL;

    if (flags < PTN_PATHINFO_DIRNAME || flags > PTN_PATHINFO_ALL) {
        ptn_string_operand_free(path);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "pathinfo(): Argument #2 ($flags) must be one of the PATHINFO_* constants"
        );
        return ptn_null();
    }
    if (
        flags != PTN_PATHINFO_ALL &&
        flags != PTN_PATHINFO_DIRNAME &&
        flags != PTN_PATHINFO_BASENAME &&
        flags != PTN_PATHINFO_EXTENSION &&
        flags != PTN_PATHINFO_FILENAME
    ) {
        ptn_string_operand_free(path);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "pathinfo(): Argument #2 ($flags) must be only one of the PATHINFO_* constants"
        );
        return ptn_null();
    }

    PtnPathInfoParts parts = ptn_pathinfo_parts(path);
    if (flags == PTN_PATHINFO_DIRNAME) {
        PtnValue result = ptn_owned_string_len(parts.dirname, parts.dirname_len);
        ptn_string_operand_free(path);
        return result;
    }
    if (flags == PTN_PATHINFO_BASENAME) {
        PtnValue result = ptn_pathinfo_owned_string(path.data + parts.basename_start, parts.basename_len);
        free(parts.dirname);
        ptn_string_operand_free(path);
        return result;
    }
    if (flags == PTN_PATHINFO_EXTENSION) {
        PtnValue result = ptn_pathinfo_owned_string(path.data + parts.extension_start, parts.extension_len);
        free(parts.dirname);
        ptn_string_operand_free(path);
        return result;
    }
    if (flags == PTN_PATHINFO_FILENAME) {
        PtnValue result = ptn_pathinfo_owned_string(path.data + parts.basename_start, parts.filename_len);
        free(parts.dirname);
        ptn_string_operand_free(path);
        return result;
    }

    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (parts.dirname_len > 0) {
        ptn_pathinfo_array_set_string(&result, "dirname", parts.dirname, parts.dirname_len);
    }
    ptn_pathinfo_array_set_string(
        &result,
        "basename",
        path.data + parts.basename_start,
        parts.basename_len
    );
    if (parts.has_extension) {
        ptn_pathinfo_array_set_string(
            &result,
            "extension",
            path.data + parts.extension_start,
            parts.extension_len
        );
    }
    ptn_pathinfo_array_set_string(
        &result,
        "filename",
        path.data + parts.basename_start,
        parts.filename_len
    );

    free(parts.dirname);
    ptn_string_operand_free(path);
    return result;
}

static const char *ptn_internal_string_arg_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_OBJECT:
            return value.as.object->class_name;
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            return ptn_offset_container_type_name(value);
    }
    return ptn_offset_container_type_name(value);
}

static void ptn_internal_throw_string_arg_type_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value
) {
    value = ptn_value_deref(value);
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be of type string, %s given",
        function_name,
        position,
        argument_name,
        ptn_internal_string_arg_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static PtnStringOperand ptn_internal_expect_string_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_NULL) {
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "%s(): Passing null to parameter #%zu ($%s) of type string is deprecated",
            function_name,
            position,
            argument_name
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_emit_deprecation(&runtime->diagnostics, message, line);
    } else if (value.type == PTN_OBJECT) {
        PtnStringOperand object_string;
        if (ptn_try_object_to_string_operand(runtime, value, line, &object_string)) {
            return object_string;
        }
        ptn_internal_throw_string_arg_type_error(runtime, function_name, position, argument_name, value);
        return ptn_string_operand_borrowed("");
    } else if (
        value.type == PTN_ARRAY ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION ||
        value.type == PTN_RESOURCE
    ) {
        ptn_internal_throw_string_arg_type_error(runtime, function_name, position, argument_name, value);
        return ptn_string_operand_borrowed("");
    }
    return ptn_value_to_string_operand_with_runtime(runtime, value, line);
}

static void ptn_highlight_append_escaped(PtnStringBuffer *buffer, const char *data, size_t len) {
    for (size_t i = 0; i < len; i++) {
        switch ((unsigned char)data[i]) {
            case '&':
                ptn_string_buffer_append(buffer, "&amp;");
                break;
            case '<':
                ptn_string_buffer_append(buffer, "&lt;");
                break;
            case '>':
                ptn_string_buffer_append(buffer, "&gt;");
                break;
            default:
                ptn_string_buffer_append_char(buffer, data[i]);
                break;
        }
    }
}

static PtnValue ptn_highlight_string_value(PtnStringOperand input) {
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    ptn_string_buffer_append(&buffer, "<code><span style=\"color: #000000\">\n");
    ptn_highlight_append_escaped(&buffer, input.data, input.len);
    ptn_string_buffer_append(&buffer, "</span>\n</code>");
    return ptn_owned_string_len(buffer.data, buffer.len);
}

static PtnValue ptn_internal_highlight_string(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "highlight_string", 1, "string", args[0], line);
    int return_output = argc >= 2 && ptn_is_truthy(args[1]);
    PtnValue highlighted = ptn_highlight_string_value(input);
    ptn_string_operand_free(input);
    if (return_output) {
        return highlighted;
    }
    fwrite(highlighted.as.string.data, 1, highlighted.as.string.len, stdout);
    ptn_value_destroy(&highlighted);
    return ptn_bool(1);
}

static void ptn_emit_highlight_file_open_warnings(
    PtnRuntime *runtime,
    const char *path,
    const char *reason,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    const char *source_path = runtime->source_path == NULL ? "ptn" : runtime->source_path;
    printf(
        "Warning: highlight_file(%s): Failed to open stream: %s in %s on line %zu\n\n",
        path,
        reason,
        source_path,
        line
    );
    printf(
        "Warning: highlight_file(): Failed opening '%s' for highlighting in %s on line %zu\n",
        path,
        source_path,
        line
    );
}

static PtnValue ptn_internal_highlight_file(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path_operand = ptn_internal_expect_string_arg(runtime, "highlight_file", 1, "filename", args[0], line);
    char *path = ptn_path_operand_to_c_string(path_operand);
    ptn_string_operand_free(path_operand);
    if (path == NULL) {
        ptn_emit_warning(&runtime->diagnostics, "highlight_file(): Filename contains null byte", line);
        return ptn_bool(0);
    }

    unsigned char *data = NULL;
    size_t data_len = 0;
    int read_result = ptn_read_file_bytes(path, &data, &data_len);
    if (read_result <= 0) {
        const char *reason = read_result == 0 ? strerror(errno) : "Failed to read stream";
        ptn_emit_highlight_file_open_warnings(runtime, path, reason, line);
        free(path);
        free(data);
        return ptn_bool(0);
    }

    PtnStringOperand input = ptn_string_operand_owned_len((char *)data, data_len);
    int return_output = argc >= 2 && ptn_is_truthy(args[1]);
    PtnValue highlighted = ptn_highlight_string_value(input);
    ptn_string_operand_free(input);
    free(path);
    if (return_output) {
        return highlighted;
    }
    fwrite(highlighted.as.string.data, 1, highlighted.as.string.len, stdout);
    ptn_value_destroy(&highlighted);
    return ptn_bool(1);
}

static PtnValue ptn_internal_dirname(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    PtnStringOperand path = ptn_internal_expect_string_arg(runtime, "dirname", 1, "path", args[0], line);
    int64_t levels = argc >= 2
        ? ptn_internal_expect_integer_arg(runtime, "dirname", 2, "levels", args[1], line)
        : 1;
    if (levels < 1) {
        ptn_string_operand_free(path);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "dirname(): Argument #2 ($levels) must be greater than or equal to 1"
        );
        return ptn_null();
    }

    size_t dirname_len = 0;
    char *dirname = ptn_dirname_string_levels(path.data, path.len, levels, &dirname_len);
    ptn_string_operand_free(path);
    return ptn_owned_string_len(dirname, dirname_len);
}

static PtnValue ptn_internal_gettype(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_gettype_value(args[0]);
}

static PtnValue ptn_internal_is_null(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_NULL);
}

static PtnValue ptn_internal_is_array(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_ARRAY);
}

static PtnValue ptn_internal_is_object(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_object(args[0]);
}

static PtnValue ptn_internal_is_countable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_ARRAY);
}

static PtnValue ptn_internal_is_iterable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_ARRAY);
}

static PtnValue ptn_internal_is_bool(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_BOOL);
}

static PtnValue ptn_internal_is_int(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_INT);
}

static PtnValue ptn_internal_is_float(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_FLOAT);
}

static PtnValue ptn_internal_is_string(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_type(args[0], PTN_STRING);
}

static PtnValue ptn_internal_is_resource(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnValue value = ptn_value_deref(args[0]);
    return ptn_bool(value.type == PTN_RESOURCE && value.as.resource->stream != NULL);
}

static PtnValue ptn_internal_is_scalar(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_is_scalar(args[0]);
}

static int ptn_is_numeric_function_string(PtnString string) {
    if (ptn_string_has_embedded_nul(string)) {
        return 0;
    }

    const char *cursor = (const char *)string.data;
    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }
    if (*cursor == '+' || *cursor == '-') {
        cursor++;
    }

    int digits = 0;
    while (isdigit((unsigned char)*cursor)) {
        digits = 1;
        cursor++;
    }
    if (*cursor == '.') {
        cursor++;
        while (isdigit((unsigned char)*cursor)) {
            digits = 1;
            cursor++;
        }
    }
    if (!digits) {
        return 0;
    }

    if (*cursor == 'e' || *cursor == 'E') {
        const char *exponent = cursor + 1;
        if (*exponent == '+' || *exponent == '-') {
            exponent++;
        }
        int exponent_digits = 0;
        while (isdigit((unsigned char)*exponent)) {
            exponent_digits = 1;
            exponent++;
        }
        if (!exponent_digits) {
            return 0;
        }
        cursor = exponent;
    }

    while (isspace((unsigned char)*cursor)) {
        cursor++;
    }
    return *cursor == '\0';
}

static PtnValue ptn_internal_is_numeric(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnValue value = ptn_value_deref(args[0]);
    if (value.type == PTN_INT || value.type == PTN_FLOAT) {
        return ptn_bool(1);
    }
    if (value.type == PTN_STRING) {
        return ptn_bool(ptn_is_numeric_function_string(value.as.string));
    }
    return ptn_bool(0);
}

static PtnValue ptn_internal_is_finite(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    if (args[0].type != PTN_FLOAT) {
        return ptn_bool(1);
    }
    return ptn_bool(isfinite(args[0].as.floating));
}

static PtnValue ptn_internal_is_infinite(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_bool(args[0].type == PTN_FLOAT && isinf(args[0].as.floating));
}

static PtnValue ptn_internal_is_nan(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_bool(args[0].type == PTN_FLOAT && isnan(args[0].as.floating));
}

static PtnValue ptn_internal_bin2hex(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    static const char hex_digits[] = "0123456789abcdef";
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "bin2hex", 1, "string", args[0], line);
    size_t len = string.len;
    if (len > (SIZE_MAX - 1) / 2) {
        ptn_abort_out_of_memory();
    }
    char *hex = malloc((len * 2) + 1);
    if (hex == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)string.data[i];
        hex[i * 2] = hex_digits[byte >> 4];
        hex[(i * 2) + 1] = hex_digits[byte & 0x0f];
    }
    hex[len * 2] = '\0';
    ptn_string_operand_free(string);
    return ptn_owned_string_len(hex, len * 2);
}

static int ptn_hex_nibble(unsigned char byte) {
    if (byte >= '0' && byte <= '9') {
        return (int)(byte - '0');
    }
    if (byte >= 'a' && byte <= 'f') {
        return 10 + (int)(byte - 'a');
    }
    if (byte >= 'A' && byte <= 'F') {
        return 10 + (int)(byte - 'A');
    }
    return -1;
}

static int ptn_octal_nibble(unsigned char byte) {
    if (byte >= '0' && byte <= '7') {
        return (int)(byte - '0');
    }
    return -1;
}

static void ptn_cslashes_charlist_mask(const char *charlist, size_t len, unsigned char mask[256]) {
    memset(mask, 0, 256);
    for (size_t i = 0; i < len; i++) {
        unsigned char start = (unsigned char)charlist[i];
        if (i + 3 < len && charlist[i + 1] == '.' && charlist[i + 2] == '.') {
            unsigned char end = (unsigned char)charlist[i + 3];
            if (start <= end) {
                for (unsigned int byte = start; byte <= end; byte++) {
                    mask[byte] = 1;
                }
                i += 3;
                continue;
            }
        }
        mask[start] = 1;
    }
}

static size_t ptn_addcslashes_escape_len(unsigned char byte) {
    switch (byte) {
        case '\0':
            return 4;
        case '\n':
        case '\r':
        case '\t':
        case '\v':
        case '\f':
        case '\a':
        case '\b':
            return 2;
        default:
            if (byte < 32 || byte > 126) {
                return 4;
            }
            return 2;
    }
}

static void ptn_addcslashes_write_escape(char *output, size_t *out, unsigned char byte) {
    output[(*out)++] = '\\';
    switch (byte) {
        case '\0':
            output[(*out)++] = '0';
            output[(*out)++] = '0';
            output[(*out)++] = '0';
            break;
        case '\n':
            output[(*out)++] = 'n';
            break;
        case '\r':
            output[(*out)++] = 'r';
            break;
        case '\t':
            output[(*out)++] = 't';
            break;
        case '\v':
            output[(*out)++] = 'v';
            break;
        case '\f':
            output[(*out)++] = 'f';
            break;
        case '\a':
            output[(*out)++] = 'a';
            break;
        case '\b':
            output[(*out)++] = 'b';
            break;
        default:
            if (byte < 32 || byte > 126) {
                static const char octal_digits[] = "01234567";
                output[(*out)++] = octal_digits[(byte >> 6) & 0x07];
                output[(*out)++] = octal_digits[(byte >> 3) & 0x07];
                output[(*out)++] = octal_digits[byte & 0x07];
            } else {
                output[(*out)++] = (char)byte;
            }
            break;
    }
}

static char *ptn_addcslashes_string(
    const char *input,
    size_t input_len,
    const char *charlist,
    size_t charlist_len,
    size_t *output_len_out
) {
    unsigned char mask[256];
    ptn_cslashes_charlist_mask(charlist, charlist_len, mask);

    size_t output_len = 0;
    for (size_t i = 0; i < input_len; i++) {
        unsigned char byte = (unsigned char)input[i];
        size_t add_len = mask[byte] ? ptn_addcslashes_escape_len(byte) : 1;
        if (add_len > SIZE_MAX - output_len - 1) {
            ptn_abort_out_of_memory();
        }
        output_len += add_len;
    }

    char *output = malloc(output_len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < input_len; i++) {
        unsigned char byte = (unsigned char)input[i];
        if (mask[byte]) {
            ptn_addcslashes_write_escape(output, &out, byte);
        } else {
            output[out++] = (char)byte;
        }
    }
    output[out] = '\0';
    *output_len_out = out;
    return output;
}

static PtnValue ptn_internal_addcslashes(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "addcslashes", 1, "string", args[0], line);
    PtnStringOperand charlist = ptn_internal_expect_string_arg(runtime, "addcslashes", 2, "characters", args[1], line);
    size_t output_len = 0;
    char *output = ptn_addcslashes_string(
        input.data,
        input.len,
        charlist.data,
        charlist.len,
        &output_len
    );
    ptn_string_operand_free(input);
    ptn_string_operand_free(charlist);
    return ptn_owned_string_len(output, output_len);
}

static int ptn_addslashes_needs_escape(unsigned char byte) {
    return byte == '\0' || byte == '\'' || byte == '"' || byte == '\\';
}

static size_t ptn_addslashes_escape_len(unsigned char byte) {
    return ptn_addslashes_needs_escape(byte) ? 2 : 1;
}

static void ptn_addslashes_write_escape(char *output, size_t *out, unsigned char byte) {
    if (byte == '\0') {
        output[(*out)++] = '\\';
        output[(*out)++] = '0';
        return;
    }
    if (ptn_addslashes_needs_escape(byte)) {
        output[(*out)++] = '\\';
    }
    output[(*out)++] = (char)byte;
}

static char *ptn_addslashes_string(const char *input, size_t len, size_t *output_len_out) {
    size_t output_len = 0;
    for (size_t i = 0; i < len; i++) {
        size_t add_len = ptn_addslashes_escape_len((unsigned char)input[i]);
        if (add_len > SIZE_MAX - output_len - 1) {
            ptn_abort_out_of_memory();
        }
        output_len += add_len;
    }

    char *output = malloc(output_len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        ptn_addslashes_write_escape(output, &out, (unsigned char)input[i]);
    }
    output[out] = '\0';
    *output_len_out = out;
    return output;
}

static PtnValue ptn_internal_addslashes(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "addslashes", 1, "string", args[0], line);
    size_t output_len = 0;
    char *output = ptn_addslashes_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static char *ptn_stripcslashes_string(const char *input, size_t len, size_t *output_len_out) {
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        unsigned char byte = (unsigned char)input[i];
        if (byte != '\\') {
            output[out++] = (char)byte;
            continue;
        }
        if (i + 1 >= len) {
            output[out++] = '\\';
            continue;
        }

        unsigned char escaped = (unsigned char)input[++i];
        switch (escaped) {
            case 'n':
                output[out++] = '\n';
                break;
            case 'r':
                output[out++] = '\r';
                break;
            case 't':
                output[out++] = '\t';
                break;
            case 'v':
                output[out++] = '\v';
                break;
            case 'f':
                output[out++] = '\f';
                break;
            case 'a':
                output[out++] = '\a';
                break;
            case 'b':
                output[out++] = '\b';
                break;
            case '\\':
                output[out++] = '\\';
                break;
            case 'x': {
                unsigned int value = 0;
                size_t digits = 0;
                while (digits < 2 && i + 1 < len) {
                    int nibble = ptn_hex_nibble((unsigned char)input[i + 1]);
                    if (nibble < 0) {
                        break;
                    }
                    value = (value << 4) | (unsigned int)nibble;
                    digits++;
                    i++;
                }
                output[out++] = digits == 0 ? 'x' : (char)(value & 0xff);
                break;
            }
            default: {
                int octal = ptn_octal_nibble(escaped);
                if (octal < 0) {
                    output[out++] = (char)escaped;
                    break;
                }
                unsigned int value = (unsigned int)octal;
                size_t digits = 1;
                while (digits < 3 && i + 1 < len) {
                    int next = ptn_octal_nibble((unsigned char)input[i + 1]);
                    if (next < 0) {
                        break;
                    }
                    value = (value << 3) | (unsigned int)next;
                    digits++;
                    i++;
                }
                output[out++] = (char)(value & 0xff);
                break;
            }
        }
    }

    output[out] = '\0';
    *output_len_out = out;
    return output;
}

static PtnValue ptn_internal_stripcslashes(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "stripcslashes", 1, "string", args[0], line);
    size_t output_len = 0;
    char *output = ptn_stripcslashes_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static char *ptn_stripslashes_string(const char *input, size_t len, size_t *output_len_out) {
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t input_offset = 0;
    size_t output_offset = 0;
    while (input_offset < len) {
        unsigned char byte = (unsigned char)input[input_offset++];
        if (byte != '\\') {
            output[output_offset++] = (char)byte;
            continue;
        }
        if (input_offset >= len) {
            continue;
        }
        byte = (unsigned char)input[input_offset++];
        output[output_offset++] = byte == '0' ? '\0' : (char)byte;
    }
    output[output_offset] = '\0';
    *output_len_out = output_offset;
    return output;
}

static PtnValue ptn_internal_stripslashes(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "stripslashes", 1, "string", args[0], line);
    size_t output_len = 0;
    char *output = ptn_stripslashes_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static PtnValue ptn_internal_hex2bin(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand hex = ptn_internal_expect_string_arg(runtime, "hex2bin", 1, "string", args[0], line);
    size_t len = hex.len;
    if ((len % 2) != 0) {
        ptn_emit_warning(
            &runtime->diagnostics,
            "hex2bin(): Hexadecimal input string must have an even length",
            line
        );
        ptn_string_operand_free(hex);
        return ptn_bool(0);
    }

    char *binary = malloc((len / 2) + 1);
    if (binary == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t output_len = 0;
    for (size_t i = 0; i < len; i += 2) {
        int high = ptn_hex_nibble((unsigned char)hex.data[i]);
        int low = ptn_hex_nibble((unsigned char)hex.data[i + 1]);
        if (high < 0 || low < 0) {
            ptn_emit_warning(
                &runtime->diagnostics,
                "hex2bin(): Input string must be hexadecimal string",
                line
            );
            free(binary);
            ptn_string_operand_free(hex);
            return ptn_bool(0);
        }
        binary[output_len++] = (char)((high << 4) | low);
    }
    binary[output_len] = '\0';
    ptn_string_operand_free(hex);
    return ptn_owned_string_len(binary, output_len);
}

static char *ptn_quoted_printable_decode_string(const char *input, size_t len, size_t *output_len_out) {
    char *output = malloc(len + 1);
    if (output == NULL) {
        ptn_abort_out_of_memory();
    }

    size_t out = 0;
    for (size_t i = 0; i < len; i++) {
        if (input[i] == '=') {
            if (i + 1 < len && input[i + 1] == '\n') {
                i += 1;
                continue;
            }
            if (i + 2 < len && input[i + 1] == '\r' && input[i + 2] == '\n') {
                i += 2;
                continue;
            }
            if (i + 2 < len) {
                int high = ptn_hex_nibble((unsigned char)input[i + 1]);
                int low = ptn_hex_nibble((unsigned char)input[i + 2]);
                if (high >= 0 && low >= 0) {
                    output[out++] = (char)((high << 4) | low);
                    i += 2;
                    continue;
                }
            }
        }
        output[out++] = input[i];
    }
    output[out] = '\0';
    *output_len_out = out;
    return output;
}

static PtnValue ptn_internal_quoted_printable_decode(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand input = ptn_internal_expect_string_arg(runtime, "quoted_printable_decode", 1, "string", args[0], line);
    size_t output_len = 0;
    char *output = ptn_quoted_printable_decode_string(input.data, input.len, &output_len);
    ptn_string_operand_free(input);
    return ptn_owned_string_len(output, output_len);
}

static int ptn_ascii_is_letter(unsigned char byte) {
    return (byte >= 'A' && byte <= 'Z') || (byte >= 'a' && byte <= 'z');
}

static unsigned char ptn_ascii_upper(unsigned char byte) {
    if (byte >= 'a' && byte <= 'z') {
        return (unsigned char)(byte - ('a' - 'A'));
    }
    return byte;
}

static char ptn_soundex_code(unsigned char byte) {
    switch (ptn_ascii_upper(byte)) {
        case 'B':
        case 'F':
        case 'P':
        case 'V':
            return '1';
        case 'C':
        case 'G':
        case 'J':
        case 'K':
        case 'Q':
        case 'S':
        case 'X':
        case 'Z':
            return '2';
        case 'D':
        case 'T':
            return '3';
        case 'L':
            return '4';
        case 'M':
        case 'N':
            return '5';
        case 'R':
            return '6';
        default:
            return '\0';
    }
}

static int ptn_soundex_resets_previous(unsigned char byte) {
    switch (ptn_ascii_upper(byte)) {
        case 'A':
        case 'E':
        case 'I':
        case 'O':
        case 'U':
        case 'Y':
        case 'H':
        case 'W':
            return 1;
        default:
            return 0;
    }
}

static PtnValue ptn_internal_soundex(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "soundex", 1, "string", args[0], line);
    char *result = malloc(5);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    result[0] = '0';
    result[1] = '0';
    result[2] = '0';
    result[3] = '0';
    result[4] = '\0';

    size_t first = 0;
    while (first < string.len && !ptn_ascii_is_letter((unsigned char)string.data[first])) {
        first++;
    }
    if (first == string.len) {
        ptn_string_operand_free(string);
        return ptn_owned_string_len(result, 4);
    }

    result[0] = (char)ptn_ascii_upper((unsigned char)string.data[first]);
    char previous = ptn_soundex_code((unsigned char)string.data[first]);
    size_t output_len = 1;
    for (size_t i = first + 1; i < string.len && output_len < 4; i++) {
        unsigned char byte = (unsigned char)string.data[i];
        char code = ptn_soundex_code(byte);
        if (code == '\0') {
            if (ptn_soundex_resets_previous(byte)) {
                previous = '\0';
            }
            continue;
        }
        if (code != previous) {
            result[output_len++] = code;
        }
        previous = code;
    }

    ptn_string_operand_free(string);
    return ptn_owned_string_len(result, 4);
}

static double ptn_value_to_double(PtnValue value) {
    double fast_number = 0.0;
    if (ptn_fast_scalar_double(value, &fast_number)) {
        return fast_number;
    }

    PtnNumber number = ptn_to_number(value);
    return number.floating;
}

static const char *ptn_numeric_arg_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_OBJECT) {
        return value.as.object->class_name;
    }
    if (value.type == PTN_EXCEPTION) {
        return value.as.exception->class_name;
    }
    if (value.type == PTN_CLOSURE) {
        return "Closure";
    }
    return ptn_offset_container_type_name(value);
}

static int ptn_numeric_arg_string_to_double(PtnString string, double *out) {
    const char *data = (const char *)string.data;
    const char *limit = data + string.len;
    const char *start = data;
    while (start < limit && isspace((unsigned char)*start)) {
        start++;
    }
    if (start >= limit) {
        return 0;
    }

    const char *cursor = start;
    if (*cursor == '+' || *cursor == '-') {
        cursor++;
        if (cursor >= limit) {
            return 0;
        }
    }
    if (cursor + 1 < limit && cursor[0] == '0' && (cursor[1] == 'x' || cursor[1] == 'X')) {
        return 0;
    }
    if (!isdigit((unsigned char)*cursor) && *cursor != '.') {
        return 0;
    }
    if (*cursor == '.' && (cursor + 1 >= limit || !isdigit((unsigned char)cursor[1]))) {
        return 0;
    }

    char *end = NULL;
    double number = strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (end < limit && isspace((unsigned char)*end)) {
        end++;
    }
    if (end != limit) {
        return 0;
    }

    *out = number;
    return 1;
}

static PtnNumber ptn_internal_expect_number_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    const char *expected_type,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL: {
            char message[192];
            int written = snprintf(
                message,
                sizeof(message),
                "%s(): Passing null to parameter #%zu ($%s) of type %s is deprecated",
                function_name,
                position,
                argument_name,
                expected_type
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_deprecation(&runtime->diagnostics, message, line);
            return ptn_number_int(0);
        }
        case PTN_BOOL:
            return ptn_number_int(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_number_int(value.as.integer);
        case PTN_FLOAT:
            return ptn_number_float(value.as.floating);
        case PTN_STRING: {
            double number = 0.0;
            if (ptn_numeric_arg_string_to_double(value.as.string, &number)) {
                return ptn_string_to_number((const char *)value.as.string.data);
            }
            break;
        }
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            break;
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be of type %s, %s given",
        function_name,
        position,
        argument_name,
        expected_type,
        ptn_numeric_arg_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return ptn_number_int(0);
}

static double ptn_internal_expect_numeric_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
) {
    return ptn_internal_expect_number_arg(
        runtime,
        function_name,
        position,
        argument_name,
        "int|float",
        value,
        line
    ).floating;
}

static double ptn_internal_expect_float_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
) {
    return ptn_internal_expect_number_arg(
        runtime,
        function_name,
        position,
        argument_name,
        "float",
        value,
        line
    ).floating;
}

static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);

static int64_t ptn_internal_expect_integer_arg_with_precision_location(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line,
    const char *precision_path,
    size_t precision_line
) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL: {
            char message[192];
            int written = snprintf(
                message,
                sizeof(message),
                "%s(): Passing null to parameter #%zu ($%s) of type int is deprecated",
                function_name,
                position,
                argument_name
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_deprecation(&runtime->diagnostics, message, line);
            return 0;
        }
        case PTN_BOOL:
            return value.as.boolean ? 1 : 0;
        case PTN_INT:
            return value.as.integer;
        case PTN_FLOAT:
            if (!isfinite(value.as.floating)) {
                break;
            }
            if (ptn_float_to_int_loses_precision(value.as.floating)) {
                if (precision_path == NULL) {
                    ptn_emit_float_to_int_precision_deprecation(
                        &runtime->diagnostics,
                        value.as.floating
                    );
                } else {
                    ptn_emit_float_to_int_precision_deprecation_at(
                        &runtime->diagnostics,
                        value.as.floating,
                        precision_path,
                        precision_line
                    );
                }
            }
            return (int64_t)value.as.floating;
        case PTN_STRING: {
            PtnNumber number;
            int has_trailing_non_numeric_data = 0;
            if (
                ptn_arithmetic_string_to_number(value.as.string, &number, &has_trailing_non_numeric_data) &&
                !has_trailing_non_numeric_data
            ) {
                if (number.type == PTN_NUMBER_FLOAT && !isfinite(number.floating)) {
                    break;
                }
                if (number.type == PTN_NUMBER_FLOAT && ptn_float_to_int_loses_precision(number.floating)) {
                    if (precision_path == NULL) {
                        ptn_emit_float_string_to_int_precision_deprecation(
                            &runtime->diagnostics,
                            (const char *)value.as.string.data
                        );
                    } else {
                        ptn_emit_float_string_to_int_precision_deprecation_at(
                            &runtime->diagnostics,
                            (const char *)value.as.string.data,
                            precision_path,
                            precision_line
                        );
                    }
                }
                return ptn_number_to_integer(number);
            }
            break;
        }
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            break;
    }

    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s(): Argument #%zu ($%s) must be of type int, %s given",
        function_name,
        position,
        argument_name,
        ptn_numeric_arg_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return 0;
}

static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
) {
    return ptn_internal_expect_integer_arg_with_precision_location(
        runtime,
        function_name,
        position,
        argument_name,
        value,
        line,
        NULL,
        0
    );
}

static PtnValue ptn_internal_ceil(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_float(ceil(ptn_internal_expect_numeric_arg(runtime, "ceil", 1, "num", args[0], line)));
}

static PtnValue ptn_internal_floor(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_float(floor(ptn_internal_expect_numeric_arg(runtime, "floor", 1, "num", args[0], line)));
}

static PtnValue ptn_internal_flush(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    fflush(stdout);
    return ptn_null();
}

static PtnValue ptn_internal_abs(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnNumber number = ptn_internal_expect_number_arg(
        runtime,
        "abs",
        1,
        "num",
        "int|float",
        args[0],
        line
    );
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(fabs(number.floating));
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(fabs((double)number.integer));
    }
    if (number.integer < 0) {
        return ptn_int(-number.integer);
    }
    return ptn_int(number.integer);
}

static PtnValue ptn_internal_sqrt(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_float(sqrt(ptn_internal_expect_float_arg(runtime, "sqrt", 1, "num", args[0], line)));
}

static PtnValue ptn_internal_minmax(
    PtnRuntime *runtime,
    const char *function_name,
    size_t argc,
    const PtnValue *args,
    int want_max
) {
    PtnValue first = ptn_value_deref(args[0]);
    if (argc == 1 && first.type == PTN_ARRAY) {
        if (first.as.array->len == 0) {
            char message[128];
            int written = snprintf(
                message,
                sizeof(message),
                "%s(): Argument #1 ($value) must contain at least one element",
                function_name
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "ValueError", message);
            return ptn_null();
        }
        PtnValue best = ptn_value_clone_deref(first.as.array->entries[0].value);
        for (size_t i = 1; i < first.as.array->len; i++) {
            PtnValue candidate = ptn_value_deref(first.as.array->entries[i].value);
            int compared = ptn_compare_order(candidate, best);
            if ((want_max && compared == PTN_COMPARE_GREATER) ||
                (!want_max && compared == PTN_COMPARE_LESS)) {
                ptn_value_destroy(&best);
                best = ptn_value_clone_deref(candidate);
            }
        }
        return best;
    }

    PtnValue best = ptn_value_clone_deref(args[0]);
    for (size_t i = 1; i < argc; i++) {
        PtnValue candidate = ptn_value_deref(args[i]);
        int compared = ptn_compare_order(candidate, best);
        if ((want_max && compared == PTN_COMPARE_GREATER) ||
            (!want_max && compared == PTN_COMPARE_LESS)) {
            ptn_value_destroy(&best);
            best = ptn_value_clone_deref(candidate);
        }
    }
    return best;
}

static PtnValue ptn_internal_max(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    return ptn_internal_minmax(runtime, "max", argc, args, 1);
}

static PtnValue ptn_internal_min(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    return ptn_internal_minmax(runtime, "min", argc, args, 0);
}

static PtnValue ptn_internal_fdiv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    double dividend = ptn_internal_expect_float_arg(runtime, "fdiv", 1, "num1", args[0], line);
    double divisor = ptn_internal_expect_float_arg(runtime, "fdiv", 2, "num2", args[1], line);
    return ptn_float(dividend / divisor);
}

static PtnValue ptn_internal_intdiv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t dividend = ptn_internal_expect_integer_arg(runtime, "intdiv", 1, "num1", args[0], line);
    int64_t divisor = ptn_internal_expect_integer_arg(runtime, "intdiv", 2, "num2", args[1], line);
    if (divisor == 0) {
        ptn_throw_exception(runtime, "DivisionByZeroError", "Division by zero");
        return ptn_null();
    }
    if (dividend == INT64_MIN && divisor == -1) {
        ptn_throw_exception(runtime, "ArithmeticError", "Division of PHP_INT_MIN by -1 is not an integer");
        return ptn_null();
    }
    return ptn_int(dividend / divisor);
}

static PtnValue ptn_internal_pi(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_float(3.14159265358979323846264338327950288);
}

static PtnValue ptn_internal_getrandmax(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_int(2147483647);
}

static PtnValue ptn_internal_getmypid(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
#if defined(_WIN32)
    return ptn_int((int64_t)_getpid());
#else
    return ptn_int((int64_t)getpid());
#endif
}

static int ptn_string_operand_ascii_case_equal(PtnStringOperand value, const char *literal) {
    size_t literal_len = strlen(literal);
    if (value.len != literal_len) {
        return 0;
    }
    for (size_t i = 0; i < literal_len; i++) {
        if (tolower((unsigned char)value.data[i]) != tolower((unsigned char)literal[i])) {
            return 0;
        }
    }
    return 1;
}

static int ptn_is_modeled_extension_operand(PtnStringOperand extension) {
    return ptn_string_operand_ascii_case_equal(extension, "Core") ||
        ptn_string_operand_ascii_case_equal(extension, "date") ||
        ptn_string_operand_ascii_case_equal(extension, "pcre") ||
        ptn_string_operand_ascii_case_equal(extension, "Reflection") ||
        ptn_string_operand_ascii_case_equal(extension, "standard");
}

static int ptn_ini_value(PtnStringOperand option, PtnValue *out) {
    if (ptn_string_operand_ascii_case_equal(option, "date.timezone")) {
        *out = ptn_string("UTC");
        return 1;
    }
    if (ptn_string_operand_ascii_case_equal(option, "display_errors")) {
        const char *configured = getenv("PTN_PHP_DISPLAY_ERRORS");
        *out = ptn_string(configured == NULL ? "1" : configured);
        return 1;
    }
    if (ptn_string_operand_ascii_case_equal(option, "extension_dir")) {
        *out = ptn_string(PTN_PHP_EXTENSION_DIR);
        return 1;
    }
    if (ptn_string_operand_ascii_case_equal(option, "include_path")) {
        *out = ptn_string(".");
        return 1;
    }
    if (ptn_string_operand_ascii_case_equal(option, "pcre.backtrack_limit")) {
        *out = ptn_string("1000000");
        return 1;
    }
    if (ptn_string_operand_ascii_case_equal(option, "precision")) {
        *out = ptn_string("14");
        return 1;
    }
    if (ptn_string_operand_ascii_case_equal(option, "zend.assertions")) {
        const char *configured = getenv("PTN_ZEND_ASSERTIONS");
        *out = ptn_string(configured == NULL ? "1" : configured);
        return 1;
    }
    return 0;
}

static const char *ptn_runtime_current_include_path(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || root->include_path == NULL) {
        return ".";
    }
    return root->include_path;
}

static void ptn_runtime_set_include_path(PtnRuntime *runtime, const char *path) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return;
    }
    char *copy = ptn_duplicate_string(path);
    free(root->include_path);
    root->include_path = copy;
}

static int ptn_environment_put_owned(char *assignment) {
#if defined(_WIN32)
    int result = _putenv(assignment);
    free(assignment);
    return result == 0;
#else
    char *equals = strchr(assignment, '=');
    if (equals == NULL || equals == assignment) {
        free(assignment);
        return 0;
    }
    *equals = '\0';
    int result = setenv(assignment, equals + 1, 1);
    free(assignment);
    return result == 0;
#endif
}

static int ptn_environment_unset(const char *name) {
#if defined(_WIN32)
    size_t name_len = strlen(name);
    if (name_len > SIZE_MAX - 2) {
        ptn_abort_out_of_memory();
    }
    char *assignment = malloc(name_len + 2);
    if (assignment == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(assignment, name, name_len);
    assignment[name_len] = '=';
    assignment[name_len + 1] = '\0';
    int result = _putenv(assignment);
    free(assignment);
    return result == 0;
#else
    return unsetenv(name) == 0;
#endif
}

static PtnValue ptn_environment_snapshot(void) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    char **environment = PTN_ENVIRON;
    if (environment == NULL) {
        return result;
    }
    for (char **entry = environment; *entry != NULL; entry++) {
        char *equals = strchr(*entry, '=');
        if (equals == NULL || equals == *entry) {
            continue;
        }
        size_t name_len = (size_t)(equals - *entry);
        char *name = ptn_duplicate_string_len(*entry, name_len);
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key(name),
            ptn_owned_string(ptn_duplicate_string(equals + 1))
        );
        free(name);
    }
    return result;
}

static PtnValue ptn_internal_get_include_path(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)args;
    (void)line;
    return ptn_owned_string(ptn_duplicate_string(ptn_runtime_current_include_path(runtime)));
}

static PtnValue ptn_internal_set_include_path(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand path = ptn_internal_expect_string_arg(runtime, "set_include_path", 1, "include_path", args[0], line);
    if (path.len == 0) {
        ptn_string_operand_free(path);
        return ptn_bool(0);
    }
    char *previous = ptn_duplicate_string(ptn_runtime_current_include_path(runtime));
    char *next = ptn_duplicate_string_len(path.data, path.len);
    ptn_runtime_set_include_path(runtime, next);
    free(next);
    ptn_string_operand_free(path);
    return ptn_owned_string(previous);
}

static PtnValue ptn_internal_ini_restore(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand option = ptn_internal_expect_string_arg(runtime, "ini_restore", 1, "option", args[0], line);
    if (ptn_string_operand_ascii_case_equal(option, "include_path")) {
        ptn_runtime_set_include_path(runtime, ".");
        ptn_string_operand_free(option);
        return ptn_null();
    }
    PtnValue value;
    int known = ptn_ini_value(option, &value);
    if (known) {
        ptn_value_destroy(&value);
    }
    ptn_string_operand_free(option);
    return known ? ptn_null() : ptn_bool(0);
}

static PtnValue ptn_internal_getenv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    if (argc == 0) {
        (void)runtime;
        (void)args;
        (void)line;
        return ptn_environment_snapshot();
    }

    PtnStringOperand name = ptn_internal_expect_string_arg(runtime, "getenv", 1, "name", args[0], line);
    if (memchr(name.data, '\0', name.len) != NULL) {
        ptn_string_operand_free(name);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "getenv(): Argument #1 ($name) must not contain any null bytes"
        );
        return ptn_null();
    }
    char *key = ptn_duplicate_string_len(name.data, name.len);
    ptn_string_operand_free(name);
    const char *value = getenv(key);
    free(key);
    if (value == NULL) {
        return ptn_bool(0);
    }
    return ptn_owned_string(ptn_duplicate_string(value));
}

static PtnValue ptn_internal_putenv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand assignment = ptn_internal_expect_string_arg(runtime, "putenv", 1, "assignment", args[0], line);
    if (memchr(assignment.data, '\0', assignment.len) != NULL) {
        ptn_string_operand_free(assignment);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "putenv(): Argument #1 ($assignment) must not contain any null bytes"
        );
        return ptn_null();
    }
    if (assignment.len == 0 || assignment.data[0] == '=') {
        ptn_string_operand_free(assignment);
        ptn_throw_exception(
            runtime,
            "ValueError",
            "putenv(): Argument #1 ($assignment) must have a valid syntax"
        );
        return ptn_null();
    }

    const char *equals = memchr(assignment.data, '=', assignment.len);
    int ok;
    if (equals == NULL) {
        char *name = ptn_duplicate_string_len(assignment.data, assignment.len);
        ok = ptn_environment_unset(name);
        free(name);
    } else {
        char *owned_assignment = ptn_duplicate_string_len(assignment.data, assignment.len);
        ok = ptn_environment_put_owned(owned_assignment);
    }
    ptn_string_operand_free(assignment);
    return ptn_bool(ok);
}

static PtnValue ptn_internal_php_sapi_name(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_string(PTN_PHP_SAPI_NAME);
}

static PtnValue ptn_internal_zend_version(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_string(PTN_ZEND_VERSION);
}

static PtnValue ptn_internal_phpversion(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    if (argc == 0) {
        return ptn_string(PTN_PHP_VERSION);
    }

    PtnStringOperand extension = ptn_value_to_string_operand(args[0]);
    int modeled_extension =
        extension.data[0] == '\0' ||
        ptn_is_modeled_extension_operand(extension);
    ptn_string_operand_free(extension);
    if (modeled_extension) {
        return ptn_string(PTN_PHP_VERSION);
    }
    return ptn_bool(0);
}

static PtnValue ptn_internal_php_uname(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    char mode = 'a';
    if (argc >= 1) {
        PtnStringOperand mode_operand = ptn_value_to_string_operand(args[0]);
        if (mode_operand.len != 1) {
            ptn_string_operand_free(mode_operand);
            ptn_throw_exception(
                runtime,
                "ValueError",
                "php_uname(): Argument #1 ($mode) must be a single character"
            );
            return ptn_null();
        }
        mode = (char)tolower((unsigned char)mode_operand.data[0]);
        ptn_string_operand_free(mode_operand);
        if (mode != 'a' && mode != 'm' && mode != 'n' && mode != 'r' && mode != 's' && mode != 'v') {
            ptn_throw_exception(
                runtime,
                "ValueError",
                "php_uname(): Argument #1 ($mode) must be one of \"a\", \"m\", \"n\", \"r\", \"s\", or \"v\""
            );
            return ptn_null();
        }
    }

#if defined(_WIN32)
    const char *system_name = PTN_PHP_OS;
    const char *node_name = "";
    const char *release = "";
    const char *version = "";
    const char *machine = "";
#else
    struct utsname info;
    if (uname(&info) != 0) {
        return ptn_string(PTN_PHP_OS);
    }
    const char *system_name = info.sysname;
    const char *node_name = info.nodename;
    const char *release = info.release;
    const char *version = info.version;
    const char *machine = info.machine;
#endif

    switch (mode) {
        case 's':
            return ptn_string(system_name);
        case 'n':
            return ptn_string(node_name);
        case 'r':
            return ptn_string(release);
        case 'v':
            return ptn_string(version);
        case 'm':
            return ptn_string(machine);
        case 'a':
        default: {
            PtnStringBuffer buffer;
            ptn_string_buffer_init(&buffer);
            ptn_string_buffer_append(&buffer, system_name);
            ptn_string_buffer_append_char(&buffer, ' ');
            ptn_string_buffer_append(&buffer, node_name);
            ptn_string_buffer_append_char(&buffer, ' ');
            ptn_string_buffer_append(&buffer, release);
            ptn_string_buffer_append_char(&buffer, ' ');
            ptn_string_buffer_append(&buffer, version);
            ptn_string_buffer_append_char(&buffer, ' ');
            ptn_string_buffer_append(&buffer, machine);
            return ptn_owned_string_len(buffer.data, buffer.len);
        }
    }
}

static PtnValue ptn_internal_ini_get(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnStringOperand option = ptn_value_to_string_operand(args[0]);
    PtnValue value;
    if (ptn_string_operand_ascii_case_equal(option, "include_path")) {
        value = ptn_owned_string(ptn_duplicate_string(ptn_runtime_current_include_path(runtime)));
        ptn_string_operand_free(option);
        return value;
    }
    int found = ptn_ini_value(option, &value);
    ptn_string_operand_free(option);
    return found ? value : ptn_bool(0);
}

static PtnValue ptn_internal_get_cfg_var(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnStringOperand option = ptn_value_to_string_operand(args[0]);
    if (ptn_string_operand_ascii_case_equal(option, "cfg_file_path")) {
        ptn_string_operand_free(option);
        return ptn_bool(0);
    }
    PtnValue value;
    if (ptn_string_operand_ascii_case_equal(option, "include_path")) {
        value = ptn_owned_string(ptn_duplicate_string(ptn_runtime_current_include_path(runtime)));
        ptn_string_operand_free(option);
        return value;
    }
    int found = ptn_ini_value(option, &value);
    ptn_string_operand_free(option);
    return found ? value : ptn_bool(0);
}

static PtnValue ptn_internal_php_ini_scanned_files(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_string("");
}

static PtnValue ptn_internal_get_loaded_extensions(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    int zend_extensions = argc >= 1 && ptn_is_truthy(args[0]);
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (zend_extensions) {
        return result;
    }
    ptn_array_set_entry(result.as.array, ptn_array_int_key(0), ptn_string("Core"));
    ptn_array_set_entry(result.as.array, ptn_array_int_key(1), ptn_string("date"));
    ptn_array_set_entry(result.as.array, ptn_array_int_key(2), ptn_string("pcre"));
    ptn_array_set_entry(result.as.array, ptn_array_int_key(3), ptn_string("Reflection"));
    ptn_array_set_entry(result.as.array, ptn_array_int_key(4), ptn_string("standard"));
    return result;
}

static PtnValue ptn_internal_extension_loaded(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnStringOperand extension = ptn_value_to_string_operand(args[0]);
    int loaded = ptn_is_modeled_extension_operand(extension);
    ptn_string_operand_free(extension);
    return ptn_bool(loaded);
}

static int ptn_setlocale_category(int64_t category, int *out) {
    switch (category) {
        case PTN_LC_CTYPE:
            *out = LC_CTYPE;
            return 1;
        case PTN_LC_NUMERIC:
            *out = LC_NUMERIC;
            return 1;
        case PTN_LC_TIME:
            *out = LC_TIME;
            return 1;
        case PTN_LC_COLLATE:
            *out = LC_COLLATE;
            return 1;
        case PTN_LC_MONETARY:
            *out = LC_MONETARY;
            return 1;
        case PTN_LC_MESSAGES:
#if defined(LC_MESSAGES)
            *out = LC_MESSAGES;
            return 1;
#else
            return 0;
#endif
        case PTN_LC_ALL:
            *out = LC_ALL;
            return 1;
        default:
            return 0;
    }
}

static char *ptn_setlocale_try_string(int category, const char *locale) {
    const char *result = NULL;
    if (strcmp(locale, "0") == 0) {
        result = setlocale(category, NULL);
    } else {
        result = setlocale(category, locale);
    }
    return result == NULL ? NULL : ptn_duplicate_string(result);
}

static char *ptn_setlocale_try_value(int category, PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_NULL) {
        return ptn_setlocale_try_string(category, "0");
    }
    if (value.type == PTN_ARRAY) {
        PtnArray *array = value.as.array;
        for (size_t i = 0; i < array->len; i++) {
            PtnValue candidate = ptn_value_deref(array->entries[i].value);
            char *result = NULL;
            if (candidate.type == PTN_NULL) {
                result = ptn_setlocale_try_string(category, "0");
            } else {
                char *locale = ptn_value_to_string(candidate);
                result = ptn_setlocale_try_string(category, locale);
                free(locale);
            }
            if (result != NULL) {
                return result;
            }
        }
        return NULL;
    }

    char *locale = ptn_value_to_string(value);
    char *result = ptn_setlocale_try_string(category, locale);
    free(locale);
    return result;
}

static PtnValue ptn_internal_setlocale(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t category_value = ptn_internal_expect_integer_arg(runtime, "setlocale", 1, "category", args[0], line);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }

    int category = 0;
    if (!ptn_setlocale_category(category_value, &category)) {
        return ptn_bool(0);
    }

    for (size_t i = 1; i < argc; i++) {
        char *result = ptn_setlocale_try_value(category, args[i]);
        if (result != NULL) {
            return ptn_owned_string(result);
        }
    }
    return ptn_bool(0);
}

static void ptn_localeconv_set_string(PtnArray *array, const char *key, const char *value) {
    ptn_array_set_entry(array, ptn_array_string_key(key), ptn_string(value == NULL ? "" : value));
}

static void ptn_localeconv_set_int(PtnArray *array, const char *key, char value) {
    ptn_array_set_entry(array, ptn_array_string_key(key), ptn_int((int64_t)(unsigned char)value));
}

static PtnValue ptn_localeconv_grouping_array(const char *grouping) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (grouping == NULL) {
        return result;
    }
    for (size_t i = 0; grouping[i] != '\0' && (unsigned char)grouping[i] != CHAR_MAX; i++) {
        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key(result.as.array->next_auto_key),
            ptn_int((int64_t)(unsigned char)grouping[i])
        );
    }
    return result;
}

static void ptn_localeconv_set_grouping(PtnArray *array, const char *key, const char *grouping) {
    ptn_array_set_entry(array, ptn_array_string_key(key), ptn_localeconv_grouping_array(grouping));
}

static PtnValue ptn_internal_localeconv(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    struct lconv *locale = localeconv();
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    ptn_localeconv_set_string(result.as.array, "decimal_point", locale->decimal_point);
    ptn_localeconv_set_string(result.as.array, "thousands_sep", locale->thousands_sep);
    ptn_localeconv_set_string(result.as.array, "int_curr_symbol", locale->int_curr_symbol);
    ptn_localeconv_set_string(result.as.array, "currency_symbol", locale->currency_symbol);
    ptn_localeconv_set_string(result.as.array, "mon_decimal_point", locale->mon_decimal_point);
    ptn_localeconv_set_string(result.as.array, "mon_thousands_sep", locale->mon_thousands_sep);
    ptn_localeconv_set_string(result.as.array, "positive_sign", locale->positive_sign);
    ptn_localeconv_set_string(result.as.array, "negative_sign", locale->negative_sign);
    ptn_localeconv_set_int(result.as.array, "int_frac_digits", locale->int_frac_digits);
    ptn_localeconv_set_int(result.as.array, "frac_digits", locale->frac_digits);
    ptn_localeconv_set_int(result.as.array, "p_cs_precedes", locale->p_cs_precedes);
    ptn_localeconv_set_int(result.as.array, "p_sep_by_space", locale->p_sep_by_space);
    ptn_localeconv_set_int(result.as.array, "n_cs_precedes", locale->n_cs_precedes);
    ptn_localeconv_set_int(result.as.array, "n_sep_by_space", locale->n_sep_by_space);
    ptn_localeconv_set_int(result.as.array, "p_sign_posn", locale->p_sign_posn);
    ptn_localeconv_set_int(result.as.array, "n_sign_posn", locale->n_sign_posn);
    ptn_localeconv_set_grouping(result.as.array, "grouping", locale->grouping);
    ptn_localeconv_set_grouping(result.as.array, "mon_grouping", locale->mon_grouping);
    return result;
}

static int ptn_digit_value_for_base(unsigned char byte, int base) {
    int value = -1;
    if (byte >= '0' && byte <= '9') {
        value = (int)(byte - '0');
    } else if (byte >= 'a' && byte <= 'z') {
        value = 10 + (int)(byte - 'a');
    } else if (byte >= 'A' && byte <= 'Z') {
        value = 10 + (int)(byte - 'A');
    }
    return value >= 0 && value < base ? value : -1;
}

static PtnValue ptn_base_string_to_number(
    PtnRuntime *runtime,
    const char *string,
    size_t string_len,
    int base,
    char prefix,
    size_t line
) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }

    const char *end = string + string_len;
    while (end > start && isspace((unsigned char)*(end - 1))) {
        end--;
    }

    if ((end - start) >= 2 && start[0] == '0' && tolower((unsigned char)start[1]) == prefix) {
        start += 2;
    }

    int saw_digit = 0;
    int saw_invalid = 0;
    int fits_integer = 1;
    int64_t integer = 0;
    double floating = 0.0;

    for (const char *cursor = start; cursor < end; cursor++) {
        int digit = ptn_digit_value_for_base((unsigned char)*cursor, base);
        if (digit < 0) {
            saw_invalid = 1;
            continue;
        }
        saw_digit = 1;
        floating = (floating * (double)base) + (double)digit;
        if (fits_integer) {
            if (integer > (INT64_MAX - digit) / base) {
                fits_integer = 0;
            } else {
                integer = (integer * base) + digit;
            }
        }
    }

    if (saw_invalid) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Invalid characters passed for attempted conversion, these have been ignored",
            line
        );
    }
    if (!saw_digit) {
        return ptn_int(0);
    }
    return fits_integer ? ptn_int(integer) : ptn_float(floating);
}

static int64_t ptn_intval_string_to_integer(const char *string, size_t string_len, int base) {
    if (base != 0 && (base < 2 || base > 36)) {
        return 0;
    }

    const char *cursor = string;
    const char *end = string + string_len;
    while (cursor < end && isspace((unsigned char)*cursor)) {
        cursor++;
    }

    int negative = 0;
    if (cursor < end && (*cursor == '+' || *cursor == '-')) {
        negative = *cursor == '-';
        cursor++;
    }

    if (base == 0) {
        if ((end - cursor) >= 2 && cursor[0] == '0' && (cursor[1] == 'x' || cursor[1] == 'X')) {
            base = 16;
            cursor += 2;
        } else if ((end - cursor) >= 2 && cursor[0] == '0' && (cursor[1] == 'b' || cursor[1] == 'B')) {
            base = 2;
            cursor += 2;
        } else if (cursor < end && cursor[0] == '0') {
            base = 8;
        } else {
            base = 10;
        }
    } else if (base == 16 && (end - cursor) >= 2 && cursor[0] == '0' && (cursor[1] == 'x' || cursor[1] == 'X')) {
        cursor += 2;
    } else if (base == 2 && (end - cursor) >= 2 && cursor[0] == '0' && (cursor[1] == 'b' || cursor[1] == 'B')) {
        cursor += 2;
    }

    uint64_t limit = negative ? ((uint64_t)INT64_MAX + 1u) : (uint64_t)INT64_MAX;
    uint64_t magnitude = 0;
    int saw_digit = 0;
    for (; cursor < end; cursor++) {
        int digit = ptn_digit_value_for_base((unsigned char)*cursor, base);
        if (digit < 0) {
            break;
        }
        saw_digit = 1;
        if (magnitude > (limit - (uint64_t)digit) / (uint64_t)base) {
            return negative ? INT64_MIN : INT64_MAX;
        }
        magnitude = (magnitude * (uint64_t)base) + (uint64_t)digit;
    }

    if (!saw_digit) {
        return 0;
    }
    if (negative && magnitude == limit) {
        return INT64_MIN;
    }
    int64_t value = (int64_t)magnitude;
    return negative ? -value : value;
}

static PtnValue ptn_internal_bindec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string.data, string.len, 2, 'b', line);
    ptn_string_operand_free(string);
    return value;
}

static PtnValue ptn_internal_hexdec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string.data, string.len, 16, 'x', line);
    ptn_string_operand_free(string);
    return value;
}

static PtnValue ptn_internal_octdec(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_value_to_string_operand(args[0]);
    PtnValue value = ptn_base_string_to_number(runtime, string.data, string.len, 8, 'o', line);
    ptn_string_operand_free(string);
    return value;
}

static PtnValue ptn_internal_boolval(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_cast_bool(args[0]);
}

static PtnValue ptn_internal_floatval(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    return ptn_cast_float(args[0]);
}

static PtnValue ptn_internal_intval(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    if (argc >= 2 && args[0].type == PTN_STRING) {
        int64_t base = ptn_number_to_integer(ptn_to_number(args[1]));
        return ptn_int(ptn_intval_string_to_integer(
            (const char *)args[0].as.string.data,
            args[0].as.string.len,
            (int)base
        ));
    }
    return ptn_cast_int(args[0]);
}

static PtnValue ptn_internal_chr(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    int64_t integer = ptn_internal_expect_integer_arg_with_precision_location(
        runtime,
        "chr",
        1,
        "codepoint",
        args[0],
        line,
        "ptn",
        line
    );
    if (integer < 0 || integer > 255) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "chr(): Providing a value not in-between 0 and 255 is deprecated, this is because a byte value must be in the [0, 255] interval. The value used will be constrained using % 256",
            line
        );
    }
    int64_t normalized = integer % 256;
    if (normalized < 0) {
        normalized += 256;
    }
    char *string = malloc(2);
    if (string == NULL) {
        ptn_abort_out_of_memory();
    }
    string[0] = (char)(unsigned char)normalized;
    string[1] = '\0';
    return ptn_owned_string_len(string, 1);
}

static PtnValue ptn_internal_ord(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnStringOperand string = ptn_internal_expect_string_arg(runtime, "ord", 1, "character", args[0], line);
    size_t len = string.len;
    int64_t byte = 0;
    if (len == 0) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "ord(): Providing an empty string is deprecated",
            line
        );
    } else {
        byte = (int64_t)(unsigned char)string.data[0];
        if (len != 1) {
            ptn_emit_deprecation(
                &runtime->diagnostics,
                "ord(): Providing a string that is not one byte long is deprecated. Use ord($str[0]) instead",
                line
            );
        }
    }
    ptn_string_operand_free(string);
    return ptn_int(byte);
}

static PtnValue ptn_internal_count(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t mode = argc >= 2 ? ptn_value_to_integer(args[1]) : PTN_COUNT_NORMAL;
    return ptn_count_value(runtime, "count", args[0], mode, line);
}

static PtnValue ptn_internal_sizeof(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int64_t mode = argc >= 2 ? ptn_value_to_integer(args[1]) : PTN_COUNT_NORMAL;
    return ptn_count_value(runtime, "sizeof", args[0], mode, line);
}

static PtnValue ptn_internal_error_reporting(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    int64_t previous_level = runtime->diagnostics.error_reporting;
    if (argc >= 1) {
        runtime->diagnostics.error_reporting = ptn_value_to_integer(args[0]);
    }
    return ptn_int(previous_level);
}

static PtnValue ptn_internal_ob_get_contents(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)args;
    (void)line;
    return ptn_bool(0);
}

static PtnCallFrame *ptn_current_call_frame(PtnRuntime *runtime, const char *function_name) {
    if (runtime->call_frame != NULL) {
        return runtime->call_frame;
    }
    char message[128];
    const char *format = strcmp(function_name, "func_num_args") == 0
        ? "%s() must be called from a function context"
        : "%s() cannot be called from the global scope";
    int written = snprintf(message, sizeof(message), format, function_name);
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
    return NULL;
}

static PtnValue ptn_call_frame_arg_value(PtnRuntime *runtime, PtnCallFrame *frame, size_t position) {
    if (position < frame->parameter_count) {
        PtnValue value;
        if (ptn_symbols_get(&runtime->symbols, frame->parameter_names[position], &value)) {
            return ptn_value_clone_deref(value);
        }
        return ptn_null();
    }
    return ptn_value_clone_deref(frame->args[position]);
}

static PtnValue ptn_internal_func_num_args(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)args;
    (void)line;
    PtnCallFrame *frame = ptn_current_call_frame(runtime, "func_num_args");
    if (frame->argc > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_int((int64_t)frame->argc);
}

static PtnValue ptn_internal_func_get_arg(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnCallFrame *frame = ptn_current_call_frame(runtime, "func_get_arg");
    int64_t position = ptn_value_to_integer(args[0]);
    if (position < 0) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "func_get_arg(): Argument #1 ($position) must be greater than or equal to 0"
        );
    }
    if ((uint64_t)position >= (uint64_t)frame->argc) {
        ptn_throw_exception(
            runtime,
            "ValueError",
            "func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function"
        );
    }
    return ptn_call_frame_arg_value(runtime, frame, (size_t)position);
}

static PtnValue ptn_internal_func_get_args(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)args;
    (void)line;
    PtnCallFrame *frame = ptn_current_call_frame(runtime, "func_get_args");
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < frame->argc; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_int_key((int64_t)i),
            ptn_call_frame_arg_value(runtime, frame, i)
        );
    }
    return result;
}

static PtnValue ptn_internal_call_user_func(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    int previous_warn_by_ref_argument_mismatch = runtime->warn_by_ref_argument_mismatch;
    runtime->warn_by_ref_argument_mismatch = 1;
    PtnValue result = ptn_call_callable(
        runtime,
        args[0],
        argc - 1,
        argc > 1 ? args + 1 : NULL,
        line
    );
    runtime->warn_by_ref_argument_mismatch = previous_warn_by_ref_argument_mismatch;
    return result;
}

static PtnValue ptn_internal_call_user_func_array(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnArray *arguments = ptn_internal_expect_array_arg(runtime, "call_user_func_array", 2, "args", args[1]);
    PtnValue *expanded = NULL;
    if (arguments->len != 0) {
        expanded = malloc(arguments->len * sizeof(PtnValue));
        if (expanded == NULL) {
            ptn_abort_out_of_memory();
        }
    }
    for (size_t i = 0; i < arguments->len; i++) {
        expanded[i] = ptn_value_clone(arguments->entries[i].value);
    }

    int previous_warn_by_ref_argument_mismatch = runtime->warn_by_ref_argument_mismatch;
    runtime->warn_by_ref_argument_mismatch = 1;
    PtnValue result = ptn_call_callable(runtime, args[0], arguments->len, expanded, line);
    runtime->warn_by_ref_argument_mismatch = previous_warn_by_ref_argument_mismatch;
    for (size_t i = 0; i < arguments->len; i++) {
        ptn_value_destroy(&expanded[i]);
    }
    free(expanded);
    return result;
}

static int ptn_spl_object_id(PtnValue value, size_t *id_out) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_OBJECT:
            *id_out = value.as.object->object_id;
            return 1;
        case PTN_CLOSURE:
            *id_out = value.as.closure->object_id;
            return 1;
        case PTN_EXCEPTION:
            *id_out = value.as.exception->object_id;
            return 1;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PtnValue ptn_internal_spl_object_id(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    size_t id = 0;
    if (ptn_spl_object_id(args[0], &id)) {
        return ptn_int((int64_t)id);
    }

    PtnValue value = ptn_value_deref(args[0]);
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "spl_object_id(): Argument #1 ($object) must be of type object, %s given",
        ptn_offset_container_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return ptn_null();
}

static PtnValue ptn_internal_spl_object_hash(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    size_t id = 0;
    if (!ptn_spl_object_id(args[0], &id)) {
        PtnValue value = ptn_value_deref(args[0]);
        char message[192];
        int written = snprintf(
            message,
            sizeof(message),
            "spl_object_hash(): Argument #1 ($object) must be of type object, %s given",
            ptn_offset_container_type_name(value)
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "TypeError", message);
        return ptn_null();
    }

    char buffer[33];
    int written = snprintf(buffer, sizeof(buffer), "%016llx0000000000000000", (unsigned long long)id);
    if (written < 0 || written != 32) {
        ptn_abort_out_of_memory();
    }
    return ptn_owned_string(ptn_duplicate_string(buffer));
}

static PtnValue ptn_internal_assert(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)line;
    if (ptn_is_truthy(args[0])) {
        return ptn_bool(1);
    }

    char *message = argc >= 2 ? ptn_value_to_string(args[1]) : ptn_duplicate_string("");
    ptn_throw_exception_owned_message(runtime, "AssertionError", message);
    return ptn_bool(0);
}

static PtnValue ptn_internal_date(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    char *format = ptn_value_to_string(args[0]);
    time_t timestamp = argc >= 2 ? (time_t)ptn_value_to_integer(args[1]) : time(NULL);
    struct tm *parts = localtime(&timestamp);
    if (parts == NULL) {
        free(format);
        return ptn_bool(0);
    }

    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    for (size_t i = 0; format[i] != '\0'; i++) {
        char ch = format[i];
        if (ch == '\\' && format[i + 1] != '\0') {
            i++;
            ptn_string_buffer_append_char(&buffer, format[i]);
            continue;
        }
        switch (ch) {
            case 'Y':
                ptn_string_buffer_append_format(&buffer, "%04d", parts->tm_year + 1900);
                break;
            case 'y':
                ptn_string_buffer_append_format(&buffer, "%02d", (parts->tm_year + 1900) % 100);
                break;
            case 'm':
                ptn_string_buffer_append_format(&buffer, "%02d", parts->tm_mon + 1);
                break;
            case 'n':
                ptn_string_buffer_append_format(&buffer, "%d", parts->tm_mon + 1);
                break;
            case 'd':
                ptn_string_buffer_append_format(&buffer, "%02d", parts->tm_mday);
                break;
            case 'j':
                ptn_string_buffer_append_format(&buffer, "%d", parts->tm_mday);
                break;
            case 'H':
                ptn_string_buffer_append_format(&buffer, "%02d", parts->tm_hour);
                break;
            case 'h': {
                int hour = parts->tm_hour % 12;
                if (hour == 0) {
                    hour = 12;
                }
                ptn_string_buffer_append_format(&buffer, "%02d", hour);
                break;
            }
            case 'i':
                ptn_string_buffer_append_format(&buffer, "%02d", parts->tm_min);
                break;
            case 's':
                ptn_string_buffer_append_format(&buffer, "%02d", parts->tm_sec);
                break;
            default:
                ptn_string_buffer_append_char(&buffer, ch);
                break;
        }
    }
    free(format);
    return ptn_owned_string_len(buffer.data, buffer.len);
}

static PtnValue ptn_internal_closure_from_callable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_define(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_constant(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static int ptn_user_function_exists(const char *name);
static PtnFunctionMetadata ptn_user_function_metadata(const char *name);
static int ptn_callable_is_valid(PtnValue callable, int syntax_only);
static int ptn_declared_class_exists(const char *name);
static int ptn_declared_class_method_exists(const char *class_name, const char *method_name);
static const char *ptn_declared_class_parent_name(const char *name);
static int ptn_declared_class_property_exists(const char *class_name, const char *property_name);
static const char *ptn_property_exists_target_type_name(PtnValue value);
static PtnValue ptn_internal_class_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_date(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_defined(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_function_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_get_called_class(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_get_class(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_get_parent_class(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_is_callable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_method_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_property_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_spl_object_hash(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_spl_object_id(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_array_key_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_fclose(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_fopen(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);
static PtnValue ptn_internal_stream_get_meta_data(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);

static const PtnInternalFunction *ptn_internal_functions(size_t *count) {
    /* Keep sorted by ASCII case-insensitive name for ptn_find_internal_function. */
    static const PtnInternalFunction functions[] = {
        { "_ptn_cow_debug_assert_balanced", 0, 0, ptn_internal__ptn_cow_debug_assert_balanced },
        { "_ptn_cow_debug_assert_counter", 2, 2, ptn_internal__ptn_cow_debug_assert_counter },
        { "_ptn_cow_debug_counter", 1, 1, ptn_internal__ptn_cow_debug_counter },
        { "_ptn_cow_debug_reset", 0, 0, ptn_internal__ptn_cow_debug_reset },
        { "abs", 1, 1, ptn_internal_abs },
        { "addcslashes", 2, 2, ptn_internal_addcslashes },
        { "addslashes", 1, 1, ptn_internal_addslashes },
        { "array_all", 2, 2, ptn_internal_array_all },
        { "array_any", 2, 2, ptn_internal_array_any },
        { "array_change_key_case", 1, 2, ptn_internal_array_change_key_case },
        { "array_chunk", 2, 3, ptn_internal_array_chunk },
        { "array_column", 2, 3, ptn_internal_array_column },
        { "array_combine", 2, 2, ptn_internal_array_combine },
        { "array_count_values", 1, 1, ptn_internal_array_count_values },
        { "array_diff", 1, PTN_VARIADIC_ARGS, ptn_internal_array_diff },
        { "array_diff_assoc", 1, PTN_VARIADIC_ARGS, ptn_internal_array_diff_assoc },
        { "array_diff_key", 1, PTN_VARIADIC_ARGS, ptn_internal_array_diff_key },
        { "array_diff_uassoc", 2, PTN_VARIADIC_ARGS, ptn_internal_array_diff_uassoc },
        { "array_diff_ukey", 2, PTN_VARIADIC_ARGS, ptn_internal_array_diff_ukey },
        { "array_fill", 3, 3, ptn_internal_array_fill },
        { "array_fill_keys", 2, 2, ptn_internal_array_fill_keys },
        { "array_filter", 1, 3, ptn_internal_array_filter },
        { "array_find", 2, 2, ptn_internal_array_find },
        { "array_find_key", 2, 2, ptn_internal_array_find_key },
        { "array_first", 1, 1, ptn_internal_array_first },
        { "array_flip", 1, 1, ptn_internal_array_flip },
        { "array_intersect", 1, PTN_VARIADIC_ARGS, ptn_internal_array_intersect },
        { "array_intersect_assoc", 1, PTN_VARIADIC_ARGS, ptn_internal_array_intersect_assoc },
        { "array_intersect_key", 1, PTN_VARIADIC_ARGS, ptn_internal_array_intersect_key },
        { "array_intersect_uassoc", 2, PTN_VARIADIC_ARGS, ptn_internal_array_intersect_uassoc },
        { "array_intersect_ukey", 2, PTN_VARIADIC_ARGS, ptn_internal_array_intersect_ukey },
        { "array_is_list", 1, 1, ptn_internal_array_is_list },
        { "array_key_exists", 2, 2, ptn_internal_array_key_exists },
        { "array_key_first", 1, 1, ptn_internal_array_key_first },
        { "array_key_last", 1, 1, ptn_internal_array_key_last },
        { "array_keys", 1, 3, ptn_internal_array_keys },
        { "array_last", 1, 1, ptn_internal_array_last },
        { "array_map", 2, PTN_VARIADIC_ARGS, ptn_internal_array_map },
        { "array_merge", 0, PTN_VARIADIC_ARGS, ptn_internal_array_merge },
        { "array_merge_recursive", 0, PTN_VARIADIC_ARGS, ptn_internal_array_merge_recursive },
        { "array_multisort", 1, PTN_VARIADIC_ARGS, ptn_internal_array_multisort },
        { "array_pad", 3, 3, ptn_internal_array_pad },
        { "array_pop", 1, 1, ptn_internal_array_pop },
        { "array_product", 1, 1, ptn_internal_array_product },
        { "array_push", 1, PTN_VARIADIC_ARGS, ptn_internal_array_push },
        { "array_rand", 1, 2, ptn_internal_array_rand },
        { "array_reduce", 2, 3, ptn_internal_array_reduce },
        { "array_replace", 1, PTN_VARIADIC_ARGS, ptn_internal_array_replace },
        { "array_replace_recursive", 1, PTN_VARIADIC_ARGS, ptn_internal_array_replace_recursive },
        { "array_reverse", 1, 2, ptn_internal_array_reverse },
        { "array_search", 2, 3, ptn_internal_array_search },
        { "array_shift", 1, 1, ptn_internal_array_shift },
        { "array_slice", 2, 4, ptn_internal_array_slice },
        { "array_splice", 2, 4, ptn_internal_array_splice },
        { "array_sum", 1, 1, ptn_internal_array_sum },
        { "array_udiff", 2, PTN_VARIADIC_ARGS, ptn_internal_array_udiff },
        { "array_udiff_assoc", 2, PTN_VARIADIC_ARGS, ptn_internal_array_udiff_assoc },
        { "array_udiff_uassoc", 3, PTN_VARIADIC_ARGS, ptn_internal_array_udiff_uassoc },
        { "array_uintersect", 2, PTN_VARIADIC_ARGS, ptn_internal_array_uintersect },
        { "array_uintersect_assoc", 2, PTN_VARIADIC_ARGS, ptn_internal_array_uintersect_assoc },
        { "array_uintersect_uassoc", 3, PTN_VARIADIC_ARGS, ptn_internal_array_uintersect_uassoc },
        { "array_unique", 1, 2, ptn_internal_array_unique },
        { "array_unshift", 1, PTN_VARIADIC_ARGS, ptn_internal_array_unshift },
        { "array_values", 1, 1, ptn_internal_array_values },
        { "array_walk", 2, 3, ptn_internal_array_walk },
        { "array_walk_recursive", 2, 3, ptn_internal_array_walk_recursive },
        { "arsort", 1, 2, ptn_internal_arsort },
        { "asort", 1, 2, ptn_internal_asort },
        { "assert", 1, 2, ptn_internal_assert },
        { "basename", 1, 2, ptn_internal_basename },
        { "bin2hex", 1, 1, ptn_internal_bin2hex },
        { "bindec", 1, 1, ptn_internal_bindec },
        { "boolval", 1, 1, ptn_internal_boolval },
        { "call_user_func", 1, PTN_VARIADIC_ARGS, ptn_internal_call_user_func },
        { "call_user_func_array", 2, 2, ptn_internal_call_user_func_array },
        { "ceil", 1, 1, ptn_internal_ceil },
        { "chdir", 1, 1, ptn_internal_chdir },
        { "chmod", 2, 2, ptn_internal_chmod },
        { "chop", 1, 2, ptn_internal_chop },
        { "chr", 1, 1, ptn_internal_chr },
        { "chunk_split", 1, 3, ptn_internal_chunk_split },
        { "class_exists", 1, 2, ptn_internal_class_exists },
        { "clearstatcache", 0, 2, ptn_internal_clearstatcache },
        { "Closure::fromCallable", 1, 1, ptn_internal_closure_from_callable },
        { "constant", 1, 1, ptn_internal_constant },
        { "count", 1, 2, ptn_internal_count },
        { "crc32", 1, 1, ptn_internal_crc32 },
        { "current", 1, 1, ptn_internal_current },
        { "date", 1, 2, ptn_internal_date },
        { "debug_zval_dump", 1, PTN_VARIADIC_ARGS, ptn_internal_debug_zval_dump },
        { "define", 2, 3, ptn_internal_define },
        { "defined", 1, 1, ptn_internal_defined },
        { "dirname", 1, 2, ptn_internal_dirname },
        { "doubleval", 1, 1, ptn_internal_floatval },
        { "end", 1, 1, ptn_internal_end },
        { "error_reporting", 0, 1, ptn_internal_error_reporting },
        { "explode", 2, 3, ptn_internal_explode },
        { "extension_loaded", 1, 1, ptn_internal_extension_loaded },
        { "fclose", 1, 1, ptn_internal_fclose },
        { "fdiv", 2, 2, ptn_internal_fdiv },
        { "file_exists", 1, 1, ptn_internal_file_exists },
        { "file_get_contents", 1, 5, ptn_internal_file_get_contents },
        { "file_put_contents", 2, 2, ptn_internal_file_put_contents },
        { "fileatime", 1, 1, ptn_internal_fileatime },
        { "filectime", 1, 1, ptn_internal_filectime },
        { "filegroup", 1, 1, ptn_internal_filegroup },
        { "fileinode", 1, 1, ptn_internal_fileinode },
        { "filemtime", 1, 1, ptn_internal_filemtime },
        { "fileowner", 1, 1, ptn_internal_fileowner },
        { "fileperms", 1, 1, ptn_internal_fileperms },
        { "filesize", 1, 1, ptn_internal_filesize },
        { "filetype", 1, 1, ptn_internal_filetype },
        { "floatval", 1, 1, ptn_internal_floatval },
        { "floor", 1, 1, ptn_internal_floor },
        { "flush", 0, 0, ptn_internal_flush },
        { "fopen", 2, 4, ptn_internal_fopen },
        { "fprintf", 2, PTN_VARIADIC_ARGS, ptn_internal_fprintf },
        { "fputs", 2, 3, ptn_internal_fputs },
        { "func_get_arg", 1, 1, ptn_internal_func_get_arg },
        { "func_get_args", 0, 0, ptn_internal_func_get_args },
        { "func_num_args", 0, 0, ptn_internal_func_num_args },
        { "function_exists", 1, 1, ptn_internal_function_exists },
        { "fwrite", 2, 3, ptn_internal_fwrite },
        { "get_called_class", 0, 0, ptn_internal_get_called_class },
        { "get_cfg_var", 1, 1, ptn_internal_get_cfg_var },
        { "get_class", 0, 1, ptn_internal_get_class },
        { "get_include_path", 0, 0, ptn_internal_get_include_path },
        { "get_loaded_extensions", 0, 1, ptn_internal_get_loaded_extensions },
        { "get_parent_class", 0, 1, ptn_internal_get_parent_class },
        { "getcwd", 0, 0, ptn_internal_getcwd },
        { "getenv", 0, 2, ptn_internal_getenv },
        { "getmypid", 0, 0, ptn_internal_getmypid },
        { "getrandmax", 0, 0, ptn_internal_getrandmax },
        { "gettype", 1, 1, ptn_internal_gettype },
        { "hex2bin", 1, 1, ptn_internal_hex2bin },
        { "hexdec", 1, 1, ptn_internal_hexdec },
        { "highlight_file", 1, 2, ptn_internal_highlight_file },
        { "highlight_string", 1, 2, ptn_internal_highlight_string },
        { "implode", 1, 2, ptn_internal_implode },
        { "in_array", 2, 3, ptn_internal_in_array },
        { "ini_get", 1, 1, ptn_internal_ini_get },
        { "ini_restore", 1, 1, ptn_internal_ini_restore },
        { "intdiv", 2, 2, ptn_internal_intdiv },
        { "intval", 1, 2, ptn_internal_intval },
        { "is_array", 1, 1, ptn_internal_is_array },
        { "is_bool", 1, 1, ptn_internal_is_bool },
        { "is_callable", 1, 3, ptn_internal_is_callable },
        { "is_countable", 1, 1, ptn_internal_is_countable },
        { "is_dir", 1, 1, ptn_internal_is_dir },
        { "is_double", 1, 1, ptn_internal_is_float },
        { "is_executable", 1, 1, ptn_internal_is_executable },
        { "is_file", 1, 1, ptn_internal_is_file },
        { "is_finite", 1, 1, ptn_internal_is_finite },
        { "is_float", 1, 1, ptn_internal_is_float },
        { "is_infinite", 1, 1, ptn_internal_is_infinite },
        { "is_int", 1, 1, ptn_internal_is_int },
        { "is_integer", 1, 1, ptn_internal_is_int },
        { "is_iterable", 1, 1, ptn_internal_is_iterable },
        { "is_link", 1, 1, ptn_internal_is_link },
        { "is_long", 1, 1, ptn_internal_is_int },
        { "is_nan", 1, 1, ptn_internal_is_nan },
        { "is_null", 1, 1, ptn_internal_is_null },
        { "is_numeric", 1, 1, ptn_internal_is_numeric },
        { "is_object", 1, 1, ptn_internal_is_object },
        { "is_readable", 1, 1, ptn_internal_is_readable },
        { "is_resource", 1, 1, ptn_internal_is_resource },
        { "is_scalar", 1, 1, ptn_internal_is_scalar },
        { "is_string", 1, 1, ptn_internal_is_string },
        { "is_writable", 1, 1, ptn_internal_is_writable },
        { "is_writeable", 1, 1, ptn_internal_is_writeable },
        { "join", 1, 2, ptn_internal_join },
        { "json_encode", 1, 3, ptn_internal_json_encode },
        { "key", 1, 1, ptn_internal_key },
        { "krsort", 1, 2, ptn_internal_krsort },
        { "ksort", 1, 2, ptn_internal_ksort },
        { "lcfirst", 1, 1, ptn_internal_lcfirst },
        { "localeconv", 0, 0, ptn_internal_localeconv },
        { "lstat", 1, 1, ptn_internal_lstat },
        { "ltrim", 1, 2, ptn_internal_ltrim },
        { "max", 1, PTN_VARIADIC_ARGS, ptn_internal_max },
        { "md5", 1, 2, ptn_internal_md5 },
        { "method_exists", 2, 2, ptn_internal_method_exists },
        { "min", 1, PTN_VARIADIC_ARGS, ptn_internal_min },
        { "mkdir", 1, 4, ptn_internal_mkdir },
        { "natcasesort", 1, 1, ptn_internal_natcasesort },
        { "natsort", 1, 1, ptn_internal_natsort },
        { "next", 1, 1, ptn_internal_next },
        { "nl2br", 1, 2, ptn_internal_nl2br },
        { "ob_get_contents", 0, 0, ptn_internal_ob_get_contents },
        { "octdec", 1, 1, ptn_internal_octdec },
        { "ord", 1, 1, ptn_internal_ord },
        { "pathinfo", 1, 2, ptn_internal_pathinfo },
        { "php_ini_scanned_files", 0, 0, ptn_internal_php_ini_scanned_files },
        { "php_sapi_name", 0, 0, ptn_internal_php_sapi_name },
        { "php_uname", 0, 1, ptn_internal_php_uname },
        { "phpversion", 0, 1, ptn_internal_phpversion },
        { "pi", 0, 0, ptn_internal_pi },
        { "pow", 2, 2, ptn_internal_pow },
        { "preg_match", 2, 5, ptn_internal_preg_match },
        { "prev", 1, 1, ptn_internal_prev },
        { "print_r", 1, 2, ptn_internal_print_r },
        { "printf", 1, PTN_VARIADIC_ARGS, ptn_internal_printf },
        { "property_exists", 2, 2, ptn_internal_property_exists },
        { "putenv", 1, 1, ptn_internal_putenv },
        { "quoted_printable_decode", 1, 1, ptn_internal_quoted_printable_decode },
        { "quotemeta", 1, 1, ptn_internal_quotemeta },
        { "range", 2, 3, ptn_internal_range },
        { "realpath", 1, 1, ptn_internal_realpath },
        { "reset", 1, 1, ptn_internal_reset },
        { "rmdir", 1, 2, ptn_internal_rmdir },
        { "rsort", 1, 2, ptn_internal_rsort },
        { "rtrim", 1, 2, ptn_internal_rtrim },
        { "scandir", 1, 3, ptn_internal_scandir },
        { "set_include_path", 1, 1, ptn_internal_set_include_path },
        { "setlocale", 2, PTN_VARIADIC_ARGS, ptn_internal_setlocale },
        { "sha1", 1, 2, ptn_internal_sha1 },
        { "sha1_file", 1, 2, ptn_internal_sha1_file },
        { "shuffle", 1, 1, ptn_internal_shuffle },
        { "sizeof", 1, 2, ptn_internal_sizeof },
        { "sort", 1, 2, ptn_internal_sort },
        { "soundex", 1, 1, ptn_internal_soundex },
        { "spl_object_hash", 1, 1, ptn_internal_spl_object_hash },
        { "spl_object_id", 1, 1, ptn_internal_spl_object_id },
        { "sprintf", 1, PTN_VARIADIC_ARGS, ptn_internal_sprintf },
        { "sqrt", 1, 1, ptn_internal_sqrt },
        { "stat", 1, 1, ptn_internal_stat },
        { "str_contains", 2, 2, ptn_internal_str_contains },
        { "str_ends_with", 2, 2, ptn_internal_str_ends_with },
        { "str_pad", 2, 4, ptn_internal_str_pad },
        { "str_repeat", 2, 2, ptn_internal_str_repeat },
        { "str_replace", 3, 4, ptn_internal_str_replace },
        { "str_rot13", 1, 1, ptn_internal_str_rot13 },
        { "str_shuffle", 1, 1, ptn_internal_str_shuffle },
        { "str_split", 1, 2, ptn_internal_str_split },
        { "str_starts_with", 2, 2, ptn_internal_str_starts_with },
        { "strcasecmp", 2, 2, ptn_internal_strcasecmp },
        { "strcmp", 2, 2, ptn_internal_strcmp },
        { "stream_get_meta_data", 1, 1, ptn_internal_stream_get_meta_data },
        { "strip_tags", 1, 1, ptn_internal_strip_tags },
        { "stripcslashes", 1, 1, ptn_internal_stripcslashes },
        { "stripos", 2, 3, ptn_internal_stripos },
        { "stripslashes", 1, 1, ptn_internal_stripslashes },
        { "stristr", 2, 3, ptn_internal_stristr },
        { "strlen", 1, 1, ptn_internal_strlen },
        { "strncasecmp", 3, 3, ptn_internal_strncasecmp },
        { "strncmp", 3, 3, ptn_internal_strncmp },
        { "strpbrk", 2, 2, ptn_internal_strpbrk },
        { "strpos", 2, 3, ptn_internal_strpos },
        { "strrchr", 2, 3, ptn_internal_strrchr },
        { "strrev", 1, 1, ptn_internal_strrev },
        { "strripos", 2, 3, ptn_internal_strripos },
        { "strrpos", 2, 3, ptn_internal_strrpos },
        { "strstr", 2, 3, ptn_internal_strstr },
        { "strtolower", 1, 1, ptn_internal_strtolower },
        { "strtoupper", 1, 1, ptn_internal_strtoupper },
        { "strtr", 2, 3, ptn_internal_strtr },
        { "substr", 2, 3, ptn_internal_substr },
        { "substr_count", 2, 4, ptn_internal_substr_count },
        { "touch", 1, 3, ptn_internal_touch },
        { "trim", 1, 2, ptn_internal_trim },
        { "ucfirst", 1, 1, ptn_internal_ucfirst },
        { "unlink", 1, 1, ptn_internal_unlink },
        { "var_dump", 1, PTN_VARIADIC_ARGS, ptn_internal_var_dump },
        { "var_export", 1, 2, ptn_internal_var_export },
        { "vfprintf", 3, 3, ptn_internal_vfprintf },
        { "vprintf", 2, 2, ptn_internal_vprintf },
        { "vsprintf", 2, 2, ptn_internal_vsprintf },
        { "zend_version", 0, 0, ptn_internal_zend_version },
    };
    *count = sizeof(functions) / sizeof(functions[0]);
    return functions;
}

static const PtnInternalFunction *ptn_find_internal_function(const char *name) {
    size_t count = 0;
    const PtnInternalFunction *functions = ptn_internal_functions(&count);
    size_t low = 0;
    size_t high = count;
    while (low < high) {
        size_t mid = low + ((high - low) / 2);
        int ordering = ptn_ascii_case_compare(name, functions[mid].name);
        if (ordering == 0) {
            return &functions[mid];
        }
        if (ordering < 0) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    return NULL;
}

static PtnFunctionMetadata ptn_internal_function_metadata(const PtnInternalFunction *function) {
    if (function == NULL) {
        return ptn_function_metadata_not_found();
    }
    int is_variadic = function->max_args == PTN_VARIADIC_ARGS;
    size_t parameter_count = is_variadic ? function->min_args : function->max_args;
    return ptn_function_metadata_found(
        function->name,
        1,
        parameter_count,
        function->min_args,
        is_variadic,
        NULL
    );
}

static PtnFunctionMetadata ptn_find_function_metadata(const char *name) {
    PtnFunctionMetadata metadata = ptn_user_function_metadata(name);
    if (metadata.found) {
        return metadata;
    }
    return ptn_internal_function_metadata(ptn_find_internal_function(name));
}

static PtnValue ptn_internal_closure_from_callable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    PtnValue callable = ptn_value_deref(args[0]);
    if (callable.type == PTN_CLOSURE) {
        return ptn_value_clone(callable);
    }
    if (!ptn_callable_is_valid(callable, 0)) {
        char *message = ptn_invalid_callback_message(
            "Closure::fromCallable",
            1,
            "callback",
            callable,
            0
        );
        ptn_throw_exception_owned_message(runtime, "TypeError", message);
        return ptn_null();
    }
    char *name = ptn_callable_output_name(callable);
    PtnFunctionMetadata metadata = ptn_find_function_metadata(name);
    free(name);
    return ptn_closure_wrap_callable(runtime, callable, metadata);
}

static PTN_UNUSED int ptn_internal_class_name_is_reflection_function(const char *class_name) {
    return ptn_ascii_case_equal(class_name, "ReflectionFunction");
}

static PTN_UNUSED int ptn_internal_class_name_is_closure(const char *class_name) {
    return ptn_ascii_case_equal(class_name, "Closure");
}

static int ptn_internal_class_exists_name(const char *class_name) {
    return ptn_internal_class_name_is_reflection_function(class_name)
        || ptn_internal_class_name_is_closure(class_name)
        || ptn_builtin_exception_class_name(class_name) != NULL;
}

static int ptn_reflection_function_method_exists(const char *method_name) {
    return ptn_ascii_case_equal(method_name, "getName")
        || ptn_ascii_case_equal(method_name, "getNamespaceName")
        || ptn_ascii_case_equal(method_name, "getNumberOfParameters")
        || ptn_ascii_case_equal(method_name, "getNumberOfRequiredParameters")
        || ptn_ascii_case_equal(method_name, "getShortName")
        || ptn_ascii_case_equal(method_name, "inNamespace")
        || ptn_ascii_case_equal(method_name, "isInternal")
        || ptn_ascii_case_equal(method_name, "isUserDefined")
        || ptn_ascii_case_equal(method_name, "isVariadic");
}

static int ptn_closure_method_exists(const char *method_name) {
    return ptn_ascii_case_equal(method_name, "__invoke")
        || ptn_ascii_case_equal(method_name, "bindTo")
        || ptn_ascii_case_equal(method_name, "fromCallable");
}

static int ptn_exception_method_exists(const char *method_name) {
    return ptn_exception_name_equal(method_name, "getMessage")
        || ptn_exception_name_equal(method_name, "getTrace");
}

static PTN_UNUSED int ptn_internal_class_method_exists(const char *class_name, const char *method_name) {
    if (ptn_internal_class_name_is_reflection_function(class_name)) {
        return ptn_reflection_function_method_exists(method_name);
    }
    if (ptn_internal_class_name_is_closure(class_name)) {
        return ptn_closure_method_exists(method_name);
    }
    if (ptn_builtin_exception_class_name(class_name) != NULL) {
        return ptn_exception_method_exists(method_name);
    }
    return ptn_declared_class_method_exists(class_name, method_name);
}

typedef struct {
    PtnFunctionMetadata metadata;
} PtnReflectionFunctionData;

static void ptn_reflection_function_data_free(void *data) {
    free(data);
}

static PtnReflectionFunctionData *ptn_reflection_function_data(PtnRuntime *runtime, PtnValue receiver) {
    receiver = ptn_value_deref(receiver);
    if (
        receiver.type != PTN_OBJECT
        || !ptn_internal_class_name_is_reflection_function(receiver.as.object->class_name)
        || receiver.as.object->native_data == NULL
    ) {
        ptn_throw_exception(runtime, "Error", "Invalid ReflectionFunction object");
        return NULL;
    }
    return (PtnReflectionFunctionData *)receiver.as.object->native_data;
}

static PtnValue ptn_reflection_function_string_before_last_namespace_separator(const char *name) {
    const char *last_separator = strrchr(name, '\\');
    if (last_separator == NULL) {
        return ptn_string("");
    }
    size_t len = (size_t)(last_separator - name);
    return ptn_owned_string_len(ptn_duplicate_string_len(name, len), len);
}

static PtnValue ptn_reflection_function_string_after_last_namespace_separator(const char *name) {
    const char *last_separator = strrchr(name, '\\');
    const char *short_name = last_separator == NULL ? name : last_separator + 1;
    return ptn_owned_string(ptn_duplicate_string(short_name));
}

static PtnValue ptn_reflection_function_size_result(size_t value) {
    if (value > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return ptn_int((int64_t)value);
}

static void ptn_reflection_function_check_no_arguments(PtnRuntime *runtime, const char *method_name, size_t argc) {
    if (argc == 0) {
        return;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "ReflectionFunction::%s() expects exactly 0 arguments, %zu given",
        method_name,
        argc
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "ArgumentCountError", message);
}

static PTN_UNUSED PtnValue ptn_reflection_function_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    (void)line;
    if (argc != 1) {
        char message[160];
        int written = snprintf(
            message,
            sizeof(message),
            "ReflectionFunction::__construct() expects exactly 1 argument, %zu given",
            argc
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ArgumentCountError", message);
        return ptn_null();
    }

    PtnValue target = ptn_value_deref(args[0]);
    char *name = NULL;
    PtnFunctionMetadata metadata = ptn_function_metadata_not_found();
    if (target.type == PTN_CLOSURE) {
        metadata = target.as.closure->metadata;
    } else {
        name = ptn_value_to_string(target);
        metadata = ptn_find_function_metadata(name);
    }
    if (!metadata.found) {
        char message[256];
        int written = snprintf(
            message,
            sizeof(message),
            "Function %s() does not exist",
            name == NULL ? "Closure" : name
        );
        free(name);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ReflectionException", message);
        return ptn_null();
    }
    free(name);

    PtnReflectionFunctionData *data = malloc(sizeof(PtnReflectionFunctionData));
    if (data == NULL) {
        ptn_abort_out_of_memory();
    }
    data->metadata = metadata;

    PtnValue object = ptn_object_new_shell(runtime, "ReflectionFunction");
    object.as.object->native_data = data;
    object.as.object->native_data_free = ptn_reflection_function_data_free;
    return object;
}

static PTN_UNUSED PtnValue ptn_reflection_function_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    (void)args;
    (void)line;
    ptn_reflection_function_check_no_arguments(runtime, name, argc);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    PtnReflectionFunctionData *data = ptn_reflection_function_data(runtime, receiver);
    if (data == NULL) {
        return ptn_null();
    }
    PtnFunctionMetadata metadata = data->metadata;
    if (ptn_ascii_case_equal(name, "getName")) {
        return ptn_owned_string(ptn_duplicate_string(metadata.name));
    }
    if (ptn_ascii_case_equal(name, "getNamespaceName")) {
        return ptn_reflection_function_string_before_last_namespace_separator(metadata.name);
    }
    if (ptn_ascii_case_equal(name, "getNumberOfParameters")) {
        return ptn_reflection_function_size_result(metadata.parameter_count);
    }
    if (ptn_ascii_case_equal(name, "getNumberOfRequiredParameters")) {
        return ptn_reflection_function_size_result(metadata.required_parameter_count);
    }
    if (ptn_ascii_case_equal(name, "getShortName")) {
        return ptn_reflection_function_string_after_last_namespace_separator(metadata.name);
    }
    if (ptn_ascii_case_equal(name, "inNamespace")) {
        return ptn_bool(strchr(metadata.name, '\\') != NULL);
    }
    if (ptn_ascii_case_equal(name, "isInternal")) {
        return ptn_bool(metadata.is_internal);
    }
    if (ptn_ascii_case_equal(name, "isUserDefined")) {
        return ptn_bool(!metadata.is_internal);
    }
    if (ptn_ascii_case_equal(name, "isVariadic")) {
        return ptn_bool(metadata.is_variadic);
    }
    ptn_throw_exception(runtime, "Error", "Call to undefined method");
    return ptn_null();
}

static PtnValue ptn_internal_define(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    char *name = ptn_value_to_string(args[0]);
    if (argc >= 3 && ptn_is_truthy(args[2])) {
        ptn_emit_define_case_insensitive_ignored_warning(&runtime->diagnostics, line);
    }
    int did_define = ptn_runtime_define_constant_if_absent(runtime, name, args[1], line);
    free(name);
    return ptn_bool(did_define);
}

static PtnValue ptn_internal_constant(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    PtnValue value = ptn_read_constant(runtime, name);
    free(name);
    return value;
}

static PtnValue ptn_internal_class_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    int exists = ptn_declared_class_exists(name) || ptn_internal_class_exists_name(name);
    free(name);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_defined(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    int exists = ptn_runtime_constant_is_defined(runtime, name);
    free(name);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_function_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    char *name = ptn_value_to_string(args[0]);
    int exists = ptn_user_function_exists(name) || ptn_find_internal_function(name) != NULL;
    free(name);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_get_called_class(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    (void)args;
    (void)line;
    if (runtime->current_called_class_name == NULL) {
        ptn_throw_exception(runtime, "Error", "get_called_class() must be called from a class");
        return ptn_null();
    }
    return ptn_owned_string(ptn_duplicate_string(runtime->current_called_class_name));
}

static PtnValue ptn_internal_get_class(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    if (argc == 0) {
        if (runtime->current_class_name == NULL) {
            ptn_throw_exception(
                runtime,
                "Error",
                "get_class() without arguments must be called from within a class"
            );
            return ptn_null();
        }
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Calling get_class() without arguments is deprecated",
            line
        );
        return ptn_owned_string(ptn_duplicate_string(runtime->current_class_name));
    }

    PtnValue value = ptn_value_deref(args[0]);
    switch (value.type) {
        case PTN_OBJECT:
            return ptn_owned_string(ptn_duplicate_string(value.as.object->class_name));
        case PTN_CLOSURE:
            return ptn_string("Closure");
        case PTN_EXCEPTION:
            return ptn_owned_string(ptn_duplicate_string(value.as.exception->class_name));
        default:
            break;
    }
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "get_class(): Argument #1 ($object) must be of type object, %s given",
        ptn_offset_container_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
    return ptn_null();
}

static void ptn_throw_get_parent_class_type_error(PtnRuntime *runtime, PtnValue value) {
    char message[224];
    int written = snprintf(
        message,
        sizeof(message),
        "get_parent_class(): Argument #1 ($object_or_class) must be an object or a valid class name, %s given",
        ptn_property_exists_target_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static PtnValue ptn_internal_get_parent_class(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    if (argc == 0) {
        if (runtime->current_class_name == NULL) {
            return ptn_bool(0);
        }
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Calling get_parent_class() without arguments is deprecated",
            line
        );
        const char *scope_parent_name = ptn_declared_class_parent_name(runtime->current_class_name);
        if (scope_parent_name == NULL) {
            return ptn_bool(0);
        }
        return ptn_owned_string(ptn_duplicate_string(scope_parent_name));
    }

    PtnValue target = ptn_value_deref(args[0]);
    char *class_name = NULL;
    if (target.type == PTN_OBJECT) {
        class_name = ptn_duplicate_string(target.as.object->class_name);
    } else if (target.type == PTN_STRING) {
        class_name = ptn_value_to_string(target);
        if (!ptn_declared_class_exists(class_name) && !ptn_internal_class_exists_name(class_name)) {
            free(class_name);
            ptn_throw_get_parent_class_type_error(runtime, target);
            return ptn_null();
        }
    } else {
        ptn_throw_get_parent_class_type_error(runtime, target);
        return ptn_null();
    }

    const char *parent_name = ptn_declared_class_parent_name(class_name);
    free(class_name);
    if (parent_name == NULL) {
        return ptn_bool(0);
    }
    return ptn_owned_string(ptn_duplicate_string(parent_name));
}

static PtnValue ptn_internal_is_callable(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)line;
    int syntax_only = argc >= 2 && ptn_is_truthy(args[1]);
    if (argc >= 3 && args[2].type == PTN_REFERENCE) {
        PtnValue callable_name = ptn_owned_string(ptn_callable_output_name(args[0]));
        ptn_reference_assign(args[2].as.reference, callable_name);
        ptn_value_destroy(&callable_name);
    }
    return ptn_bool(ptn_callable_is_valid(args[0], syntax_only));
}

static PtnValue ptn_internal_method_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)runtime;
    (void)argc;
    (void)line;
    PtnValue target = ptn_value_deref(args[0]);
    char *class_name = NULL;
    if (target.type == PTN_OBJECT) {
        class_name = ptn_duplicate_string(target.as.object->class_name);
    } else if (target.type == PTN_EXCEPTION) {
        class_name = ptn_duplicate_string(target.as.exception->class_name);
    } else {
        class_name = ptn_value_to_string(target);
    }
    char *method_name = ptn_value_to_string(args[1]);
    int exists = ptn_internal_class_method_exists(class_name, method_name);
    free(method_name);
    free(class_name);
    return ptn_bool(exists);
}

static const char *ptn_property_exists_target_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    if (value.type == PTN_BOOL) {
        return value.as.boolean ? "true" : "false";
    }
    return ptn_offset_container_type_name(value);
}

static void ptn_throw_property_exists_target_type_error(PtnRuntime *runtime, PtnValue value) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "property_exists(): Argument #1 ($object_or_class) must be of type object|string, %s given",
        ptn_property_exists_target_type_name(value)
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static PtnValue ptn_internal_property_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    PtnValue target = ptn_value_deref(args[0]);
    if (
        target.type != PTN_OBJECT &&
        target.type != PTN_STRING &&
        target.type != PTN_CLOSURE &&
        target.type != PTN_EXCEPTION
    ) {
        ptn_throw_property_exists_target_type_error(runtime, target);
        return ptn_null();
    }

    PtnStringOperand property_operand = ptn_internal_expect_string_arg(
        runtime,
        "property_exists",
        2,
        "property",
        args[1],
        line
    );
    if (runtime->exceptions->active_exception != NULL) {
        ptn_string_operand_free(property_operand);
        return ptn_null();
    }
    char *property_name = ptn_duplicate_string_len(property_operand.data, property_operand.len);

    int exists = 0;
    if (target.type == PTN_OBJECT) {
        exists = ptn_declared_class_property_exists(target.as.object->class_name, property_name) ||
            ptn_object_public_property_slot_exists(target.as.object, property_name);
    } else if (target.type == PTN_STRING) {
        char *class_name = ptn_value_to_string(target);
        exists = ptn_declared_class_property_exists(class_name, property_name);
        free(class_name);
    } else if (target.type == PTN_CLOSURE) {
        exists = ptn_declared_class_property_exists("Closure", property_name);
    } else if (target.type == PTN_EXCEPTION) {
        exists = ptn_declared_class_property_exists(target.as.exception->class_name, property_name);
    }

    free(property_name);
    ptn_string_operand_free(property_operand);
    return ptn_bool(exists);
}

static PtnValue ptn_internal_array_key_exists(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line) {
    (void)argc;
    return ptn_array_key_exists_value(runtime, args[0], args[1], line);
}

static void ptn_throw_internal_argument_count_error(
    PtnRuntime *runtime,
    const char *name,
    const char *limit,
    size_t expected,
    size_t argc
) {
    int needed = snprintf(
        NULL,
        0,
        "%s() expects %s %zu argument%s, %zu given",
        name,
        limit,
        expected,
        expected == 1 ? "" : "s",
        argc
    );
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(
        message,
        (size_t)needed + 1,
        "%s() expects %s %zu argument%s, %zu given",
        name,
        limit,
        expected,
        expected == 1 ? "" : "s",
        argc
    );
    ptn_throw_exception_owned_message(runtime, "ArgumentCountError", message);
}

static PTN_UNUSED PtnValue ptn_call_internal(PtnRuntime *runtime, const char *name, size_t argc, const PtnValue *args, size_t line) {
    const PtnInternalFunction *function = ptn_find_internal_function(name);
    if (function != NULL) {
        if (argc < function->min_args) {
            ptn_throw_internal_argument_count_error(
                runtime,
                name,
                "at least",
                function->min_args,
                argc
            );
            return ptn_null();
        }
        if (function->max_args != PTN_VARIADIC_ARGS && argc > function->max_args) {
            ptn_throw_internal_argument_count_error(
                runtime,
                name,
                "at most",
                function->max_args,
                argc
            );
            return ptn_null();
        }
        PtnTraceFrame trace_frame;
        ptn_runtime_push_trace_frame(
            runtime,
            &trace_frame,
            function->name,
            runtime->source_path,
            line,
            argc,
            args
        );
        PtnValue result = function->handler(runtime, argc, args, line);
        ptn_runtime_pop_trace_frame(runtime, &trace_frame);
        return result;
    }

    ptn_emit_undefined_function_error(&runtime->diagnostics, name);
    exit(255);
    return ptn_null();
}
/* PTN_INTERNAL_FUNCTIONS_END */
