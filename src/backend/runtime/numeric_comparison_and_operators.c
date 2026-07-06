static PTN_UNUSED PtnValue ptn_array_reindexing_internal_value(PtnValue value) {
    if (value.type == PTN_REFERENCE && value.as.reference->refcount == 1) {
        return ptn_value_deref(value);
    }
    return value;
}

static PTN_UNUSED int ptn_try_object_to_string_operand(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnStringOperand *out
);

static PTN_UNUSED PtnValue ptn_array_union(PtnArray *left, PtnArray *right) {
    int right_adds_key = 0;
    for (size_t i = 0; i < right->len; i++) {
        if (ptn_array_find_key(left, right->entries[i].key) >= left->len) {
            right_adds_key = 1;
            break;
        }
    }
    if (!right_adds_key) {
        return ptn_value_clone(ptn_array(left));
    }

    PtnValue union_value = ptn_array_from_literal_entries(0, NULL);
    PtnArray *union_array = union_value.as.array;

    for (size_t i = 0; i < left->len; i++) {
        ptn_array_set_entry(
            union_array,
            ptn_array_key_clone(left->entries[i].key),
            ptn_value_clone(left->entries[i].value)
        );
    }

    for (size_t i = 0; i < right->len; i++) {
        if (ptn_array_find_key(union_array, right->entries[i].key) < union_array->len) {
            continue;
        }
        ptn_array_set_entry(
            union_array,
            ptn_array_key_clone(right->entries[i].key),
            ptn_value_clone(right->entries[i].value)
        );
    }

    return union_value;
}

typedef struct {
    PtnArray **items;
    size_t len;
    size_t capacity;
} PtnComparisonArrayStackSide;

typedef struct {
    PtnComparisonArrayStackSide left;
    PtnComparisonArrayStackSide right;
    size_t depth;
} PtnComparisonArrayStack;

static void ptn_comparison_array_stack_side_init(PtnComparisonArrayStackSide *side) {
    side->items = NULL;
    side->len = 0;
    side->capacity = 0;
}

static void ptn_comparison_array_stack_init(PtnComparisonArrayStack *stack) {
    ptn_comparison_array_stack_side_init(&stack->left);
    ptn_comparison_array_stack_side_init(&stack->right);
    stack->depth = 0;
}

static void ptn_comparison_array_stack_side_free(PtnComparisonArrayStackSide *side) {
    free(side->items);
    side->items = NULL;
    side->len = 0;
    side->capacity = 0;
}

static void ptn_comparison_array_stack_free(PtnComparisonArrayStack *stack) {
    ptn_comparison_array_stack_side_free(&stack->left);
    ptn_comparison_array_stack_side_free(&stack->right);
}

static int ptn_comparison_array_stack_side_contains(PtnComparisonArrayStackSide *side, PtnArray *array) {
    for (size_t i = 0; i < side->len; i++) {
        if (side->items[i] == array) {
            return 1;
        }
    }
    return 0;
}

static void ptn_comparison_array_stack_side_push(PtnComparisonArrayStackSide *side, PtnArray *array) {
    if (side->len == side->capacity) {
        size_t new_capacity = side->capacity == 0 ? 8 : side->capacity * 2;
        if (new_capacity < side->capacity) {
            ptn_abort_out_of_memory();
        }
        PtnArray **new_items = realloc(side->items, new_capacity * sizeof(PtnArray *));
        if (new_items == NULL) {
            ptn_abort_out_of_memory();
        }
        side->items = new_items;
        side->capacity = new_capacity;
    }
    side->items[side->len++] = array;
}

static void ptn_comparison_array_stack_side_pop(PtnComparisonArrayStackSide *side) {
    if (side->len > 0) {
        side->len--;
    }
}

static void ptn_compare_throw_recursive_dependency(PtnRuntime *runtime, size_t line) {
    if (runtime == NULL) {
        return;
    }
    ptn_throw_exception_at(
        runtime,
        "Error",
        "Nesting level too deep - recursive dependency?",
        runtime->source_path,
        line
    );
}

static int ptn_compare_arrays_enter(
    PtnRuntime *runtime,
    PtnArray *left,
    PtnArray *right,
    size_t line,
    PtnComparisonArrayStack *stack,
    int *right_pushed
) {
    if (stack->depth >= 4096) {
        ptn_compare_throw_recursive_dependency(runtime, line);
        return 0;
    }
    if (
        ptn_comparison_array_stack_side_contains(&stack->left, left) ||
        ptn_comparison_array_stack_side_contains(&stack->right, right)
    ) {
        ptn_compare_throw_recursive_dependency(runtime, line);
        return 0;
    }
    ptn_comparison_array_stack_side_push(&stack->left, left);
    *right_pushed = 0;
    if (right != left) {
        ptn_comparison_array_stack_side_push(&stack->right, right);
        *right_pushed = 1;
    }
    stack->depth++;
    return 1;
}

static void ptn_compare_arrays_leave(PtnComparisonArrayStack *stack, int right_pushed) {
    if (stack->depth > 0) {
        stack->depth--;
    }
    if (right_pushed) {
        ptn_comparison_array_stack_side_pop(&stack->right);
    }
    ptn_comparison_array_stack_side_pop(&stack->left);
}

static int ptn_compare_equal_inner(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
);

static int ptn_compare_identical_inner(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
);

static int ptn_compare_order_inner(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
);

static PTN_UNUSED int ptn_compare_arrays_equal(
    PtnRuntime *runtime,
    PtnArray *left,
    PtnArray *right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    if (left == right) {
        return 1;
    }
    int right_pushed = 0;
    if (!ptn_compare_arrays_enter(runtime, left, right, line, stack, &right_pushed)) {
        return 0;
    }
    int result = 1;
    if (left->len != right->len) {
        result = 0;
    }
    for (size_t i = 0; result && i < left->len; i++) {
        PtnArrayEntry *right_entry = ptn_array_entry_for_key(right, left->entries[i].key);
        if (right_entry == NULL ||
            !ptn_compare_equal_inner(runtime, left->entries[i].value, right_entry->value, line, stack)) {
            result = 0;
            break;
        }
    }
    ptn_compare_arrays_leave(stack, right_pushed);
    return result;
}

static PTN_UNUSED int ptn_compare_arrays_identical(
    PtnRuntime *runtime,
    PtnArray *left,
    PtnArray *right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    if (left == right) {
        return 1;
    }
    int right_pushed = 0;
    if (!ptn_compare_arrays_enter(runtime, left, right, line, stack, &right_pushed)) {
        return 0;
    }
    int result = 1;
    if (left->len != right->len) {
        result = 0;
    }
    for (size_t i = 0; result && i < left->len; i++) {
        if (!ptn_array_keys_equal(left->entries[i].key, right->entries[i].key) ||
            !ptn_compare_identical_inner(runtime, left->entries[i].value, right->entries[i].value, line, stack)) {
            result = 0;
            break;
        }
    }
    ptn_compare_arrays_leave(stack, right_pushed);
    return result;
}

typedef struct PtnDateTimeData {
    time_t timestamp;
    int microsecond;
    char *timezone;
    int timezone_type;
} PtnDateTimeData;

static int ptn_compare_datetime_objects_order(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line,
    int *compared
);
static int ptn_compare_datetimezone_objects_order(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line,
    int *compared
);
static int ptn_compare_dateinterval_objects_warn(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line
);

static PTN_UNUSED int ptn_compare_objects_equal(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    if (left != NULL && left->lazy_uninitialized) {
        PtnValue value = ptn_value_borrow(ptn_object(left));
        if (!ptn_lazy_object_initialize(runtime, value, line)) {
            return 0;
        }
    }
    if (right != NULL && right->lazy_uninitialized) {
        PtnValue value = ptn_value_borrow(ptn_object(right));
        if (!ptn_lazy_object_initialize(runtime, value, line)) {
            return 0;
        }
    }
    if (ptn_compare_dateinterval_objects_warn(runtime, left, right, line)) {
        return 0;
    }
    if (left == right) {
        return 1;
    }
    int datetime_compared = PTN_COMPARE_UNORDERED;
    if (ptn_compare_datetime_objects_order(runtime, left, right, line, &datetime_compared)) {
        return datetime_compared == PTN_COMPARE_EQUAL;
    }
    int timezone_compared = PTN_COMPARE_UNORDERED;
    if (ptn_compare_datetimezone_objects_order(runtime, left, right, line, &timezone_compared)) {
        return timezone_compared == PTN_COMPARE_EQUAL;
    }
    if (strcmp(left->class_name, right->class_name) != 0) {
        return 0;
    }
    return ptn_compare_arrays_equal(runtime, left->properties, right->properties, line, stack);
}

static int ptn_compare_optional_names_equal(const char *left, const char *right) {
    if (left == NULL || right == NULL) {
        return left == right;
    }
    return ptn_ascii_case_equal(left, right);
}

static int ptn_compare_value_strings_case_equal(PtnString left, PtnString right) {
    if (left.len != right.len) {
        return 0;
    }
    for (size_t i = 0; i < left.len; i++) {
        if (tolower((unsigned char)left.data[i]) != tolower((unsigned char)right.data[i])) {
            return 0;
        }
    }
    return 1;
}

static int ptn_compare_object_is_datetime(PtnObject *object) {
    return object != NULL &&
        (ptn_declared_class_is_same_or_descendant(object->class_name, "DateTime") ||
         ptn_declared_class_is_same_or_descendant(object->class_name, "DateTimeImmutable"));
}

static int ptn_compare_object_is_datetimezone(PtnObject *object) {
    return object != NULL &&
        ptn_declared_class_is_same_or_descendant(object->class_name, "DateTimeZone");
}

static int ptn_compare_object_is_dateinterval(PtnObject *object) {
    return object != NULL &&
        ptn_declared_class_is_same_or_descendant(object->class_name, "DateInterval");
}

static int ptn_compare_dateinterval_objects_warn(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line
) {
    if (!ptn_compare_object_is_dateinterval(left) || !ptn_compare_object_is_dateinterval(right)) {
        return 0;
    }
    ptn_emit_warning(&runtime->diagnostics, "Cannot compare DateInterval objects", line);
    return 1;
}

static PtnArrayEntry *ptn_compare_object_string_property_entry(PtnObject *object, const char *name) {
    if (object == NULL || object->properties == NULL) {
        return NULL;
    }
    PtnArrayKey key = ptn_array_string_key(name);
    PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
    ptn_array_key_free(key);
    if (entry == NULL) {
        return NULL;
    }
    PtnValue value = ptn_value_deref(entry->value);
    return value.type == PTN_STRING ? entry : NULL;
}

static PtnArrayEntry *ptn_compare_object_int_property_entry(PtnObject *object, const char *name) {
    if (object == NULL || object->properties == NULL) {
        return NULL;
    }
    PtnArrayKey key = ptn_array_string_key(name);
    PtnArrayEntry *entry = ptn_array_entry_for_key(object->properties, key);
    ptn_array_key_free(key);
    if (entry == NULL) {
        return NULL;
    }
    PtnValue value = ptn_value_deref(entry->value);
    return value.type == PTN_INT ? entry : NULL;
}

static int ptn_compare_datetime_objects_order(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line,
    int *compared
) {
    if (!ptn_compare_object_is_datetime(left) || !ptn_compare_object_is_datetime(right)) {
        return 0;
    }
    PtnDateTimeData *left_data = (PtnDateTimeData *)left->native_data;
    PtnDateTimeData *right_data = (PtnDateTimeData *)right->native_data;
    if (left_data == NULL || right_data == NULL) {
        ptn_throw_exception(
            runtime,
            "DateObjectError",
            "Trying to compare an incomplete DateTime or DateTimeImmutable object"
        );
        *compared = PTN_COMPARE_UNORDERED;
        return 1;
    }
    if (left_data->timestamp < right_data->timestamp) {
        *compared = PTN_COMPARE_LESS;
    } else if (left_data->timestamp > right_data->timestamp) {
        *compared = PTN_COMPARE_GREATER;
    } else if (left_data->microsecond < right_data->microsecond) {
        *compared = PTN_COMPARE_LESS;
    } else if (left_data->microsecond > right_data->microsecond) {
        *compared = PTN_COMPARE_GREATER;
    } else {
        *compared = PTN_COMPARE_EQUAL;
    }
    (void)line;
    return 1;
}

static int ptn_compare_datetimezone_objects_order(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line,
    int *compared
) {
    if (!ptn_compare_object_is_datetimezone(left) || !ptn_compare_object_is_datetimezone(right)) {
        return 0;
    }
    PtnArrayEntry *left_type_entry = ptn_compare_object_int_property_entry(left, "timezone_type");
    PtnArrayEntry *right_type_entry = ptn_compare_object_int_property_entry(right, "timezone_type");
    PtnArrayEntry *left_name_entry = ptn_compare_object_string_property_entry(left, "timezone");
    PtnArrayEntry *right_name_entry = ptn_compare_object_string_property_entry(right, "timezone");
    if (left_type_entry == NULL || right_type_entry == NULL ||
        left_name_entry == NULL || right_name_entry == NULL) {
        ptn_throw_exception(runtime, "DateObjectError", "Trying to compare uninitialized DateTimeZone objects");
        *compared = PTN_COMPARE_UNORDERED;
        return 1;
    }

    PtnValue left_type = ptn_value_deref(left_type_entry->value);
    PtnValue right_type = ptn_value_deref(right_type_entry->value);
    if (left_type.as.integer != right_type.as.integer) {
        ptn_throw_exception(runtime, "DateException", "Cannot compare two different kinds of DateTimeZone objects");
        *compared = PTN_COMPARE_UNORDERED;
        return 1;
    }

    PtnValue left_name = ptn_value_deref(left_name_entry->value);
    PtnValue right_name = ptn_value_deref(right_name_entry->value);
    *compared = ptn_compare_value_strings(left_name.as.string, right_name.as.string) == PTN_COMPARE_EQUAL
        ? PTN_COMPARE_EQUAL
        : PTN_COMPARE_UNORDERED;
    (void)line;
    return 1;
}

static int ptn_compare_array_callable_parts(
    PtnValue callable,
    PtnValue *receiver_out,
    PtnValue *method_out
) {
    callable = ptn_value_deref(callable);
    if (callable.type != PTN_ARRAY || callable.as.array->len != 2) {
        return 0;
    }

    PtnArrayKey receiver_key = ptn_array_int_key(0);
    PtnArrayKey method_key = ptn_array_int_key(1);
    PtnArrayEntry *receiver_entry = ptn_array_entry_for_key(callable.as.array, receiver_key);
    PtnArrayEntry *method_entry = ptn_array_entry_for_key(callable.as.array, method_key);
    ptn_array_key_free(receiver_key);
    ptn_array_key_free(method_key);

    if (receiver_entry == NULL || method_entry == NULL) {
        return 0;
    }
    *receiver_out = ptn_value_deref(receiver_entry->value);
    *method_out = ptn_value_deref(method_entry->value);
    return 1;
}

static int ptn_compare_callable_receiver_identity(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_compare_value_strings_case_equal(left.as.string, right.as.string);
    }
    if (left.type == PTN_OBJECT && right.type == PTN_OBJECT) {
        return left.as.object == right.as.object;
    }
    if (left.type == PTN_EXCEPTION && right.type == PTN_EXCEPTION) {
        return left.as.exception == right.as.exception;
    }
    if (left.type == PTN_CLOSURE && right.type == PTN_CLOSURE) {
        return left.as.closure == right.as.closure;
    }
    return ptn_compare_identical_inner(runtime, left, right, line, stack);
}

static int ptn_compare_callable_method_identity(PtnValue left, PtnValue right) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_compare_value_strings_case_equal(left.as.string, right.as.string);
    }
    return 0;
}

static int ptn_compare_wrapped_callable_identity(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        return ptn_compare_value_strings_case_equal(left.as.string, right.as.string);
    }

    PtnValue left_receiver;
    PtnValue left_method;
    PtnValue right_receiver;
    PtnValue right_method;
    if (ptn_compare_array_callable_parts(left, &left_receiver, &left_method) &&
        ptn_compare_array_callable_parts(right, &right_receiver, &right_method)) {
        return ptn_compare_callable_receiver_identity(runtime, left_receiver, right_receiver, line, stack) &&
            ptn_compare_callable_method_identity(left_method, right_method);
    }

    return ptn_compare_identical_inner(runtime, left, right, line, stack);
}

static int ptn_compare_closure_captures_identical(
    PtnRuntime *runtime,
    PtnClosure *left,
    PtnClosure *right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    if (left->captures.len != right->captures.len) {
        return 0;
    }
    for (size_t i = 0; i < left->captures.len; i++) {
        PtnSymbol *left_capture = &left->captures.items[i];
        PtnValue right_value;
        if (!ptn_symbols_get(&right->captures, left_capture->name, &right_value)) {
            return 0;
        }
        if (!ptn_compare_identical_inner(runtime, left_capture->value, right_value, line, stack)) {
            return 0;
        }
    }
    return 1;
}

static int ptn_compare_closures_equal(
    PtnRuntime *runtime,
    PtnClosure *left,
    PtnClosure *right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    if (left == right) {
        return 1;
    }
    if (!left->has_wrapped_callable || !right->has_wrapped_callable) {
        return 0;
    }
    return ptn_compare_optional_names_equal(left->scope_class_name, right->scope_class_name) &&
        ptn_compare_optional_names_equal(left->called_class_name, right->called_class_name) &&
        ptn_compare_optional_names_equal(left->bound_scope_name, right->bound_scope_name) &&
        ptn_compare_closure_captures_identical(runtime, left, right, line, stack) &&
        ptn_compare_wrapped_callable_identity(
            runtime,
            left->wrapped_callable,
            right->wrapped_callable,
            line,
            stack
        );
}

static PTN_UNUSED int ptn_compare_arrays_order(
    PtnRuntime *runtime,
    PtnArray *left,
    PtnArray *right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    if (left == right) {
        return PTN_COMPARE_EQUAL;
    }
    int right_pushed = 0;
    if (!ptn_compare_arrays_enter(runtime, left, right, line, stack, &right_pushed)) {
        return PTN_COMPARE_UNORDERED;
    }
    int result = PTN_COMPARE_EQUAL;
    if (left->len < right->len) {
        result = PTN_COMPARE_LESS;
    }
    if (left->len > right->len) {
        result = PTN_COMPARE_GREATER;
    }
    for (size_t i = 0; result == PTN_COMPARE_EQUAL && i < left->len; i++) {
        PtnArrayEntry *right_entry = ptn_array_entry_for_key(right, left->entries[i].key);
        if (right_entry == NULL) {
            result = PTN_COMPARE_UNORDERED;
            break;
        }
        int compared = ptn_compare_order_inner(runtime, left->entries[i].value, right_entry->value, line, stack);
        if (compared != PTN_COMPARE_EQUAL) {
            result = compared;
            break;
        }
    }
    ptn_compare_arrays_leave(stack, right_pushed);
    return result;
}

static PTN_UNUSED int ptn_compare_class_names(const char *left, const char *right) {
    while (*left != '\0' && *right != '\0') {
        int left_byte = tolower((unsigned char)*left);
        int right_byte = tolower((unsigned char)*right);
        if (left_byte != right_byte) {
            return left_byte < right_byte ? PTN_COMPARE_LESS : PTN_COMPARE_GREATER;
        }
        left++;
        right++;
    }
    if (*left == '\0' && *right == '\0') {
        return PTN_COMPARE_EQUAL;
    }
    return *left == '\0' ? PTN_COMPARE_LESS : PTN_COMPARE_GREATER;
}

static PTN_UNUSED int ptn_compare_objects_order(
    PtnRuntime *runtime,
    PtnObject *left,
    PtnObject *right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    if (left != NULL && left->lazy_uninitialized) {
        PtnValue value = ptn_value_borrow(ptn_object(left));
        if (!ptn_lazy_object_initialize(runtime, value, line)) {
            return PTN_COMPARE_UNORDERED;
        }
    }
    if (right != NULL && right->lazy_uninitialized) {
        PtnValue value = ptn_value_borrow(ptn_object(right));
        if (!ptn_lazy_object_initialize(runtime, value, line)) {
            return PTN_COMPARE_UNORDERED;
        }
    }
    if (ptn_compare_dateinterval_objects_warn(runtime, left, right, line)) {
        return PTN_COMPARE_UNORDERED;
    }
    if (left == right) {
        return PTN_COMPARE_EQUAL;
    }
    int datetime_compared = PTN_COMPARE_UNORDERED;
    if (ptn_compare_datetime_objects_order(runtime, left, right, line, &datetime_compared)) {
        return datetime_compared;
    }
    int timezone_compared = PTN_COMPARE_UNORDERED;
    if (ptn_compare_datetimezone_objects_order(runtime, left, right, line, &timezone_compared)) {
        return timezone_compared;
    }
    if (left->enum_case_name != NULL || right->enum_case_name != NULL) {
        if (
            left->enum_case_name != NULL &&
            right->enum_case_name != NULL &&
            ptn_compare_class_names(left->class_name, right->class_name) == PTN_COMPARE_EQUAL &&
            strcmp(left->enum_case_name, right->enum_case_name) == 0
        ) {
            return PTN_COMPARE_EQUAL;
        }
        return PTN_COMPARE_UNORDERED;
    }
    if (ptn_compare_class_names(left->class_name, right->class_name) != PTN_COMPARE_EQUAL) {
        return PTN_COMPARE_UNORDERED;
    }
    return ptn_compare_arrays_order(runtime, left->properties, right->properties, line, stack);
}

static PTN_UNUSED int ptn_value_is_comparison_object(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_OBJECT ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION;
}

static PTN_UNUSED const char *ptn_comparison_object_class_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_OBJECT:
            return value.as.object->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_REFERENCE:
            return ptn_offset_container_type_name(value);
    }
    return ptn_offset_container_type_name(value);
}

static PtnString ptn_comparison_string_from_operand(PtnStringOperand operand) {
    PtnString string;
    string.data = (const unsigned char *)operand.data;
    string.len = operand.len;
    string.payload = NULL;
    return string;
}

static PTN_UNUSED int ptn_compare_object_and_string(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    int *compared
) {
    if (runtime == NULL) {
        return 0;
    }
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_OBJECT && right.type == PTN_STRING) {
        PtnStringOperand left_string;
        if (!ptn_try_object_to_string_operand(runtime, left, line, &left_string)) {
            return 0;
        }
        *compared = ptn_compare_strings_loose(
            ptn_comparison_string_from_operand(left_string),
            right.as.string
        );
        ptn_string_operand_free(left_string);
        return 1;
    }
    if (left.type == PTN_STRING && right.type == PTN_OBJECT) {
        PtnStringOperand right_string;
        if (!ptn_try_object_to_string_operand(runtime, right, line, &right_string)) {
            return 0;
        }
        *compared = ptn_compare_strings_loose(
            left.as.string,
            ptn_comparison_string_from_operand(right_string)
        );
        ptn_string_operand_free(right_string);
        return 1;
    }
    return 0;
}

static PTN_UNUSED int ptn_value_is_enum_case_object(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_OBJECT && value.as.object->enum_case_name != NULL;
}

static PTN_UNUSED void ptn_emit_object_to_number_notice(
    PtnRuntime *runtime,
    PtnValue object,
    const char *target_type,
    size_t line
) {
    if (runtime == NULL || !ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_NOTICE)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Notice: Object of class ", stdout);
    fputs(ptn_comparison_object_class_name(object), stdout);
    fputs(" could not be converted to ", stdout);
    fputs(target_type, stdout);
    fputs(" in ", stdout);
    fputs(runtime->source_path != NULL ? runtime->source_path : "ptn", stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED int ptn_compare_object_and_number(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    int *compared
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (ptn_value_is_comparison_object(left) && ptn_is_number_type(right)) {
        if (ptn_value_is_enum_case_object(left)) {
            *compared = PTN_COMPARE_GREATER;
            return 1;
        }
        double right_number = right.type == PTN_INT ? (double)right.as.integer : right.as.floating;
        ptn_emit_object_to_number_notice(runtime, left, right.type == PTN_INT ? "int" : "float", line);
        *compared = ptn_compare_numbers(1.0, right_number);
        return 1;
    }
    if (ptn_is_number_type(left) && ptn_value_is_comparison_object(right)) {
        if (ptn_value_is_enum_case_object(right)) {
            *compared = PTN_COMPARE_LESS;
            return 1;
        }
        double left_number = left.type == PTN_INT ? (double)left.as.integer : left.as.floating;
        ptn_emit_object_to_number_notice(runtime, right, left.type == PTN_INT ? "int" : "float", line);
        *compared = ptn_compare_numbers(left_number, 1.0);
        return 1;
    }
    return 0;
}

static int ptn_compare_equal_inner(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    int internal_compared = 0;
    if (ptn_bcmath_number_compare(runtime, left, right, line, &internal_compared)) {
        return internal_compared == PTN_COMPARE_EQUAL;
    }
#endif
    if (left.type == right.type) {
        switch (left.type) {
            case PTN_NULL:
                return 1;
            case PTN_BOOL:
                return left.as.boolean == right.as.boolean;
            case PTN_INT:
                return left.as.integer == right.as.integer;
            case PTN_FLOAT:
                return ptn_compare_numbers(left.as.floating, right.as.floating) == PTN_COMPARE_EQUAL;
            case PTN_STRING:
                return ptn_compare_strings_loose(left.as.string, right.as.string) == PTN_COMPARE_EQUAL;
            case PTN_ARRAY:
                return ptn_compare_arrays_equal(runtime, left.as.array, right.as.array, line, stack);
            case PTN_OBJECT:
                return ptn_compare_objects_equal(runtime, left.as.object, right.as.object, line, stack);
            case PTN_CLOSURE:
                return ptn_compare_closures_equal(runtime, left.as.closure, right.as.closure, line, stack);
            case PTN_EXCEPTION:
                return left.as.exception == right.as.exception;
            case PTN_RESOURCE:
                return left.as.resource == right.as.resource;
            case PTN_REFERENCE:
                return 0;
        }
    }

    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_is_truthy(left) == ptn_is_truthy(right);
    }
    if (left.type == PTN_NULL || right.type == PTN_NULL) {
        if (left.type == PTN_NULL && right.type == PTN_NULL) {
            return 1;
        }
        PtnValue other = left.type == PTN_NULL ? right : left;
        switch (other.type) {
            case PTN_NULL:
                return 1;
            case PTN_BOOL:
                return ptn_is_truthy(other) == 0;
            case PTN_INT:
                return other.as.integer == 0;
            case PTN_FLOAT:
                return other.as.floating == 0.0;
            case PTN_STRING:
                return other.as.string.len == 0;
            case PTN_ARRAY:
                return other.as.array->len == 0;
            case PTN_OBJECT:
            case PTN_CLOSURE:
                return 0;
            case PTN_EXCEPTION:
                return 0;
            case PTN_RESOURCE:
                return 0;
            case PTN_REFERENCE:
                return 0;
        }
    }

    if (left.type == PTN_ARRAY || right.type == PTN_ARRAY) {
        if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
            return ptn_compare_arrays_equal(runtime, left.as.array, right.as.array, line, stack);
        }
        return 0;
    }

    int compared = 0;
    if (ptn_compare_object_and_string(runtime, left, right, line, &compared)) {
        return compared == PTN_COMPARE_EQUAL;
    }
    if (ptn_compare_object_and_number(runtime, left, right, line, &compared)) {
        return compared == PTN_COMPARE_EQUAL;
    }
    if (ptn_compare_number_types(left, right, &compared)) {
        return compared == PTN_COMPARE_EQUAL;
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number) == PTN_COMPARE_EQUAL;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_equal(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    PtnComparisonArrayStack stack;
    ptn_comparison_array_stack_init(&stack);
    int result = ptn_compare_equal_inner(runtime, left, right, line, &stack);
    ptn_comparison_array_stack_free(&stack);
    return result;
}

static int ptn_compare_identical_inner(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type != right.type) {
        return 0;
    }
    switch (left.type) {
        case PTN_NULL:
            return 1;
        case PTN_BOOL:
            return left.as.boolean == right.as.boolean;
        case PTN_INT:
            return left.as.integer == right.as.integer;
        case PTN_FLOAT:
            return left.as.floating == right.as.floating;
        case PTN_STRING:
            return ptn_compare_value_strings(left.as.string, right.as.string) == PTN_COMPARE_EQUAL;
        case PTN_ARRAY:
            if (left.as.array == right.as.array) {
                return 1;
            }
            return ptn_compare_arrays_identical(runtime, left.as.array, right.as.array, line, stack);
        case PTN_OBJECT:
            return left.as.object == right.as.object;
        case PTN_CLOSURE:
            return left.as.closure == right.as.closure;
        case PTN_EXCEPTION:
            return left.as.exception == right.as.exception;
        case PTN_RESOURCE:
            return left.as.resource == right.as.resource;
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_compare_identical(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    PtnComparisonArrayStack stack;
    ptn_comparison_array_stack_init(&stack);
    int result = ptn_compare_identical_inner(runtime, left, right, line, &stack);
    ptn_comparison_array_stack_free(&stack);
    return result;
}

static PTN_UNUSED int ptn_compare_not_identical(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    return !ptn_compare_identical(runtime, left, right, line);
}

static PTN_UNUSED int ptn_value_is_nan(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_FLOAT && isnan(value.as.floating);
}

static int ptn_compare_order_inner(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnComparisonArrayStack *stack
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    int internal_compared = PTN_COMPARE_UNORDERED;
    if (ptn_bcmath_number_compare(runtime, left, right, line, &internal_compared)) {
        return internal_compared;
    }
#endif
    if (left.type == right.type) {
        switch (left.type) {
            case PTN_NULL:
                return PTN_COMPARE_EQUAL;
            case PTN_BOOL:
                return ptn_compare_integers(left.as.boolean, right.as.boolean);
            case PTN_INT:
                return ptn_compare_integers(left.as.integer, right.as.integer);
            case PTN_FLOAT:
                return ptn_compare_numbers(left.as.floating, right.as.floating);
            case PTN_STRING:
                return ptn_compare_strings_loose(left.as.string, right.as.string);
            case PTN_ARRAY:
                return ptn_compare_arrays_order(runtime, left.as.array, right.as.array, line, stack);
            case PTN_OBJECT:
                return ptn_compare_objects_order(runtime, left.as.object, right.as.object, line, stack);
            case PTN_CLOSURE:
                return ptn_compare_closures_equal(runtime, left.as.closure, right.as.closure, line, stack)
                    ? PTN_COMPARE_EQUAL
                    : PTN_COMPARE_GREATER;
            case PTN_EXCEPTION:
                return left.as.exception == right.as.exception ? PTN_COMPARE_EQUAL : PTN_COMPARE_GREATER;
            case PTN_RESOURCE:
                return ptn_compare_integers(left.as.resource->id, right.as.resource->id);
            case PTN_REFERENCE:
                return PTN_COMPARE_UNORDERED;
        }
    }

    if (left.type == PTN_BOOL || right.type == PTN_BOOL) {
        return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
    }
    if (left.type == PTN_NULL && right.type == PTN_NULL) {
        return 0;
    }
    if (left.type == PTN_NULL) {
        if (ptn_value_is_nan(right)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (ptn_is_number_type(right)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (right.type == PTN_STRING) {
            return ptn_compare_string_bytes((const unsigned char *)"", 0, right.as.string.data, right.as.string.len);
        }
        if (right.type == PTN_ARRAY) {
            return ptn_compare_numbers(0.0, (double)ptn_is_truthy(right));
        }
    }
    if (right.type == PTN_NULL) {
        if (ptn_value_is_nan(left)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (ptn_is_number_type(left)) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
        }
        if (left.type == PTN_STRING) {
            return ptn_compare_string_bytes(left.as.string.data, left.as.string.len, (const unsigned char *)"", 0);
        }
        if (left.type == PTN_ARRAY) {
            return ptn_compare_numbers((double)ptn_is_truthy(left), 0.0);
        }
    }

    if (left.type == PTN_ARRAY || right.type == PTN_ARRAY) {
        if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
            return ptn_compare_arrays_order(runtime, left.as.array, right.as.array, line, stack);
        }
        if (ptn_value_is_comparison_object(left) || ptn_value_is_comparison_object(right)) {
            return ptn_value_is_comparison_object(left) ? PTN_COMPARE_GREATER : PTN_COMPARE_LESS;
        }
        return left.type == PTN_ARRAY ? PTN_COMPARE_GREATER : PTN_COMPARE_LESS;
    }

    int compared = 0;
    if (ptn_compare_object_and_string(runtime, left, right, line, &compared)) {
        return compared;
    }
    if (ptn_compare_object_and_number(runtime, left, right, line, &compared)) {
        return compared;
    }
    if (ptn_compare_number_types(left, right, &compared)) {
        return compared;
    }

    double left_number = 0.0;
    double right_number = 0.0;
    if (ptn_comparison_numeric_value(left, &left_number) &&
        ptn_comparison_numeric_value(right, &right_number)) {
        return ptn_compare_numbers(left_number, right_number);
    }
    if (ptn_is_number_type(left) && right.type == PTN_STRING) {
        if (ptn_value_is_nan(left)) {
            return PTN_COMPARE_UNORDERED;
        }
        return ptn_compare_number_and_string(runtime, left, right.as.string, 1);
    }
    if (left.type == PTN_STRING && ptn_is_number_type(right)) {
        if (ptn_value_is_nan(right)) {
            return PTN_COMPARE_UNORDERED;
        }
        return ptn_compare_number_and_string(runtime, right, left.as.string, 0);
    }
    if (ptn_value_is_comparison_object(left) || ptn_value_is_comparison_object(right)) {
        return ptn_value_is_comparison_object(left) ? PTN_COMPARE_GREATER : PTN_COMPARE_LESS;
    }
    return ptn_compare_numbers((double)ptn_is_truthy(left), (double)ptn_is_truthy(right));
}

static PTN_UNUSED int ptn_compare_order(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    PtnComparisonArrayStack stack;
    ptn_comparison_array_stack_init(&stack);
    int result = ptn_compare_order_inner(runtime, left, right, line, &stack);
    ptn_comparison_array_stack_free(&stack);
    return result;
}

static PTN_UNUSED int ptn_compare_less(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    return ptn_compare_order(runtime, left, right, line) == PTN_COMPARE_LESS;
}

static PTN_UNUSED int ptn_compare_less_equal(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int compared = ptn_compare_order(runtime, left, right, line);
    return compared == PTN_COMPARE_LESS || compared == PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_greater(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    return ptn_compare_order(runtime, left, right, line) == PTN_COMPARE_GREATER;
}

static PTN_UNUSED int ptn_compare_greater_equal(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int compared = ptn_compare_order(runtime, left, right, line);
    return compared == PTN_COMPARE_GREATER || compared == PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_spaceship(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    int compared = ptn_compare_order(runtime, left, right, line);
    if (compared == PTN_COMPARE_LESS) {
        return -1;
    }
    if (compared == PTN_COMPARE_EQUAL) {
        return 0;
    }
    return 1;
}

static PTN_UNUSED void ptn_emit_arithmetic_non_numeric_value_warning(PtnRuntime *runtime, size_t line) {
    if (!ptn_diagnostics_should_emit(&runtime->diagnostics, PTN_E_WARNING)) {
        return;
    }
    const char *message = "A non-numeric value encountered";
    const char *path = runtime != NULL && runtime->source_path != NULL ? runtime->source_path : "ptn";
    if (ptn_diagnostics_try_error_handler(&runtime->diagnostics, PTN_E_WARNING, message, path, line)) {
        return;
    }
    fputc('\n', stdout);
    fputs("Warning: ", stdout);
    fputs(message, stdout);
    fputs(" in ", stdout);
    fputs(path, stdout);
    fputs(" on line ", stdout);
    fprintf(stdout, "%zu", line);
    fputc('\n', stdout);
}

static PTN_UNUSED int ptn_arithmetic_string_to_number(
    PtnString string,
    PtnNumber *number,
    int *has_trailing_non_numeric_data
) {
    const char *data = (const char *)string.data;
    const char *limit = data + string.len;
    const char *start = data;
    while (start < limit && isspace((unsigned char)*start)) {
        start++;
    }
    if (!ptn_numeric_string_can_start(start, limit)) {
        return 0;
    }

    char *int_end = NULL;
    errno = 0;
    long long integer = strtoll(start, &int_end, 10);
    int int_errno = errno;

    char *float_end = NULL;
    errno = 0;
    double floating = strtod(start, &float_end);
    if (float_end == start) {
        return 0;
    }

    const char *end = float_end;
    while (end < limit && isspace((unsigned char)*end)) {
        end++;
    }
    *has_trailing_non_numeric_data = end < limit;

    if (int_end == float_end && int_errno != ERANGE && !ptn_contains_float_marker(start, int_end)) {
        *number = ptn_number_int((int64_t)integer);
    } else {
        *number = ptn_number_float(floating);
    }
    return 1;
}

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static int ptn_zend_test_numeric_castable_no_operations_number(PtnValue value, PtnNumber *out) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT ||
        !ptn_ascii_case_equal(value.as.object->class_name, "NumericCastableNoOperations")) {
        return 0;
    }
    char *storage_key = ptn_object_private_storage_key("NumericCastableNoOperations", "val");
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(value.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        return 0;
    }
    PtnValue stored = ptn_value_deref(entry->value);
    if (stored.type == PTN_INT) {
        *out = ptn_number_int(stored.as.integer);
        return 1;
    }
    if (stored.type == PTN_FLOAT) {
        *out = ptn_number_float(stored.as.floating);
        return 1;
    }
    return 0;
}
#endif

static PTN_UNUSED int ptn_arithmetic_number(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnNumber *number
) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            *number = ptn_number_int(0);
            return 1;
        case PTN_BOOL:
            *number = ptn_number_int(value.as.boolean ? 1 : 0);
            return 1;
        case PTN_INT:
            *number = ptn_number_int(value.as.integer);
            return 1;
        case PTN_FLOAT:
            *number = ptn_number_float(value.as.floating);
            return 1;
        case PTN_STRING: {
            int has_trailing_non_numeric_data = 0;
            if (!ptn_arithmetic_string_to_number(value.as.string, number, &has_trailing_non_numeric_data)) {
                return 0;
            }
            if (has_trailing_non_numeric_data) {
                ptn_emit_arithmetic_non_numeric_value_warning(runtime, line);
            }
            return 1;
        }
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_OBJECT:
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            if (ptn_zend_test_numeric_castable_no_operations_number(value, number)) {
                return 1;
            }
            if (ptn_simplexml_numeric_value(value, number)) {
                return 1;
            }
#endif
            return 0;
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED const char *ptn_arithmetic_operand_type_name(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_OBJECT:
            return value.as.object->class_name;
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_BOOL:
            return "bool";
        case PTN_NULL:
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

static PTN_UNUSED int ptn_numeric_operator_rejects_operand(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_ARRAY:
        case PTN_OBJECT: {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            PtnNumber ignored;
            if (ptn_zend_test_numeric_castable_no_operations_number(value, &ignored)) {
                return 0;
            }
            if (ptn_simplexml_numeric_value(value, &ignored)) {
                return 0;
            }
#endif
            return 1;
        }
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            return 1;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_RESOURCE:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_integer_operator_rejects_operand(PtnValue value) {
    value = ptn_value_deref(value);
    if (ptn_numeric_operator_rejects_operand(value)) {
        return 1;
    }
    if (value.type != PTN_STRING) {
        return 0;
    }

    PtnNumber number;
    int has_trailing_non_numeric_data = 0;
    return !ptn_arithmetic_string_to_number(
        value.as.string,
        &number,
        &has_trailing_non_numeric_data
    );
}

static PTN_UNUSED void ptn_throw_unsupported_operand_types(
    PtnRuntime *runtime,
    PtnValue left,
    const char *operator,
    PtnValue right,
    size_t line
) {
    const char *left_type = ptn_arithmetic_operand_type_name(left);
    const char *right_type = ptn_arithmetic_operand_type_name(right);
    int needed = snprintf(
        NULL,
        0,
        "Unsupported operand types: %s %s %s",
        left_type,
        operator,
        right_type
    );
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(
        message,
        (size_t)needed + 1,
        "Unsupported operand types: %s %s %s",
        left_type,
        operator,
        right_type
    );
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
    free(message);
}

static PTN_UNUSED void ptn_arithmetic_operands(
    PtnRuntime *runtime,
    PtnValue left,
    const char *operator,
    PtnValue right,
    size_t line,
    PtnNumber *left_number,
    PtnNumber *right_number
) {
    if (!ptn_arithmetic_number(runtime, left, line, left_number)) {
        ptn_throw_unsupported_operand_types(runtime, left, operator, right, line);
    }
    if (!ptn_arithmetic_number(runtime, right, line, right_number)) {
        ptn_throw_unsupported_operand_types(runtime, left, operator, right, line);
    }
}

static PTN_UNUSED PtnValue ptn_positive(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_int(integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(value.as.floating);
    }

    PtnNumber number;
    if (!ptn_arithmetic_number(runtime, value, line, &number)) {
        ptn_throw_unsupported_operand_types(runtime, value, "*", ptn_int(1), line);
        return ptn_null();
    }
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating);
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED PtnValue ptn_negate(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        if (integer == INT64_MIN) {
            return ptn_float(-(double)integer);
        }
        return ptn_int(-integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_float(-value.as.floating);
    }

    PtnNumber number;
    if (!ptn_arithmetic_number(runtime, value, line, &number)) {
        ptn_throw_unsupported_operand_types(runtime, value, "*", ptn_int(1), line);
        return ptn_null();
    }
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(-number.floating);
    }
    if (number.integer == INT64_MIN) {
        return ptn_float(-(double)number.integer);
    }
    return ptn_int(-number.integer);
}

static PTN_UNUSED int ptn_fast_numeric_pair(PtnValue left, PtnValue right, double *left_number, double *right_number) {
    return ptn_fast_scalar_double(left, left_number) && ptn_fast_scalar_double(right, right_number);
}

static PTN_UNUSED PtnValue ptn_add_integers(int64_t left, int64_t right) {
    if ((right > 0 && left > INT64_MAX - right) ||
        (right < 0 && left < INT64_MIN - right)) {
        return ptn_float((double)left + (double)right);
    }
    return ptn_int(left + right);
}

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED PtnValue ptn_multiply_integers(int64_t left, int64_t right);

static int ptn_zend_test_do_operation_no_cast_object_value(PtnValue value, int64_t *out) {
    value = ptn_value_deref(value);
    if (value.type != PTN_OBJECT ||
        !ptn_ascii_case_equal(value.as.object->class_name, "DoOperationNoCast")) {
        return 0;
    }
    char *storage_key = ptn_object_private_storage_key("DoOperationNoCast", "val");
    PtnArrayKey key = ptn_array_string_key(storage_key);
    PtnArrayEntry *entry = ptn_array_entry_for_key(value.as.object->properties, key);
    ptn_array_key_free(key);
    free(storage_key);
    if (entry == NULL) {
        return 0;
    }
    PtnValue stored = ptn_value_deref(entry->value);
    if (stored.type != PTN_INT) {
        return 0;
    }
    *out = stored.as.integer;
    return 1;
}

static int ptn_zend_test_do_operation_no_cast_operand_long(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    int64_t *out
) {
    if (ptn_zend_test_do_operation_no_cast_object_value(value, out)) {
        return 1;
    }

    PtnNumber number;
    if (!ptn_arithmetic_number(runtime, value, line, &number)) {
        return 0;
    }
    *out = number.type == PTN_NUMBER_FLOAT
        ? ptn_float_to_php_integer(number.floating)
        : number.integer;
    return 1;
}

static int ptn_zend_test_do_operation_no_cast_binary_op(
    PtnRuntime *runtime,
    const char *operator,
    PtnValue left,
    PtnValue right,
    size_t line,
    PtnValue *out
) {
    int64_t left_value = 0;
    int64_t right_value = 0;
    int64_t ignored = 0;
    if (!ptn_zend_test_do_operation_no_cast_object_value(left, &ignored) &&
        !ptn_zend_test_do_operation_no_cast_object_value(right, &ignored)) {
        return 0;
    }
    if (!ptn_zend_test_do_operation_no_cast_operand_long(runtime, left, line, &left_value) ||
        !ptn_zend_test_do_operation_no_cast_operand_long(runtime, right, line, &right_value)) {
        return 0;
    }
    PtnValue arg = ptn_ascii_case_equal(operator, "+")
        ? ptn_add_integers(left_value, right_value)
        : ptn_multiply_integers(left_value, right_value);
    PtnValue args[1] = { arg };
    *out = ptn_zend_test_do_operation_no_cast_new(runtime, 1, args, line);
    ptn_value_destroy(&arg);
    return 1;
}
#endif

static PTN_UNUSED PtnValue ptn_add(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_ARRAY && right.type == PTN_ARRAY) {
        return ptn_array_union(left.as.array, right.as.array);
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_result = ptn_null();
    if (ptn_zend_test_do_operation_no_cast_binary_op(runtime, "+", left, right, line, &internal_result)) {
        return internal_result;
    }
    if (ptn_bcmath_number_binary_op(runtime, "+", left, right, line, &internal_result)) {
        return internal_result;
    }
#endif

    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        return ptn_add_integers(left_integer, right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        return ptn_float(left_fast_number + right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "+", right, line, &left_number, &right_number);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating + right_number.floating);
    }

    return ptn_add_integers(left_number.integer, right_number.integer);
}

static PTN_UNUSED PtnValue ptn_subtract_integers(int64_t left, int64_t right) {
    if ((right < 0 && left > INT64_MAX + right) ||
        (right > 0 && left < INT64_MIN + right)) {
        return ptn_float((double)left - (double)right);
    }
    return ptn_int(left - right);
}

static PTN_UNUSED PtnValue ptn_subtract(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_result = ptn_null();
    if (ptn_bcmath_number_binary_op(runtime, "-", left, right, line, &internal_result)) {
        return internal_result;
    }
#endif

    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        return ptn_subtract_integers(left_integer, right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        return ptn_float(left_fast_number - right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "-", right, line, &left_number, &right_number);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating - right_number.floating);
    }

    return ptn_subtract_integers(left_number.integer, right_number.integer);
}

static PTN_UNUSED int ptn_multiply_overflows(int64_t left, int64_t right) {
    if (left == 0 || right == 0) {
        return 0;
    }
    if (left > 0) {
        if (right > 0) {
            return left > INT64_MAX / right;
        }
        return right < INT64_MIN / left;
    }
    if (right > 0) {
        return left < INT64_MIN / right;
    }
    return right < INT64_MAX / left;
}

static PTN_UNUSED PtnValue ptn_multiply_integers(int64_t left, int64_t right) {
    if (ptn_multiply_overflows(left, right)) {
        return ptn_float((double)left * (double)right);
    }
    return ptn_int(left * right);
}

static PTN_UNUSED PtnValue ptn_multiply(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_result = ptn_null();
    if (ptn_zend_test_do_operation_no_cast_binary_op(runtime, "*", left, right, line, &internal_result)) {
        return internal_result;
    }
    if (ptn_bcmath_number_binary_op(runtime, "*", left, right, line, &internal_result)) {
        return internal_result;
    }
#endif

    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        return ptn_multiply_integers(left_integer, right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        return ptn_float(left_fast_number * right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "*", right, line, &left_number, &right_number);
    if (left_number.type == PTN_NUMBER_FLOAT || right_number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(left_number.floating * right_number.floating);
    }

    return ptn_multiply_integers(left_number.integer, right_number.integer);
}

static PTN_UNUSED int ptn_integer_power_fits(int64_t base, int64_t exponent, int64_t *out) {
    if (exponent < 0) {
        return 0;
    }

    int64_t result = 1;
    int64_t factor = base;
    int64_t remaining = exponent;
    while (remaining > 0) {
        if ((remaining & 1) != 0) {
            if (ptn_multiply_overflows(result, factor)) {
                return 0;
            }
            result *= factor;
        }
        remaining >>= 1;
        if (remaining > 0) {
            if (ptn_multiply_overflows(factor, factor)) {
                return 0;
            }
            factor *= factor;
        }
    }

    *out = result;
    return 1;
}

static PTN_UNUSED PtnValue ptn_integer_power_float_fallback(int64_t base, int64_t exponent) {
    if (base < 0 && (exponent & 1) != 0) {
        return ptn_float(-pow(fabs((double)base), (double)exponent));
    }
    return ptn_float(pow((double)base, (double)exponent));
}

static PTN_UNUSED PtnValue ptn_power(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_result = ptn_null();
    if (ptn_bcmath_number_binary_op(runtime, "**", left, right, line, &internal_result)) {
        return internal_result;
    }
#endif

    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        if (left_integer == 0 && right_integer < 0) {
            ptn_emit_deprecation(
                &runtime->diagnostics,
                "Power of base 0 and negative exponent is deprecated",
                line
            );
        }
        int64_t integer_result = 0;
        if (ptn_integer_power_fits(left_integer, right_integer, &integer_result)) {
            return ptn_int(integer_result);
        }
        return ptn_integer_power_float_fallback(left_integer, right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        if (left_fast_number == 0.0 && right_fast_number < 0.0) {
            ptn_emit_deprecation(
                &runtime->diagnostics,
                "Power of base 0 and negative exponent is deprecated",
                line
            );
        }
        return ptn_float(pow(left_fast_number, right_fast_number));
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "**", right, line, &left_number, &right_number);
    if (left_number.floating == 0.0 && right_number.floating < 0.0) {
        ptn_emit_deprecation(
            &runtime->diagnostics,
            "Power of base 0 and negative exponent is deprecated",
            line
        );
    }
    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        int64_t integer_result = 0;
        if (ptn_integer_power_fits(left_number.integer, right_number.integer, &integer_result)) {
            return ptn_int(integer_result);
        }
        return ptn_integer_power_float_fallback(left_number.integer, right_number.integer);
    }
    return ptn_float(pow(left_number.floating, right_number.floating));
}

static PTN_UNUSED PtnValue ptn_divide(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_result = ptn_null();
    if (ptn_bcmath_number_binary_op(runtime, "/", left, right, line, &internal_result)) {
        return internal_result;
    }
#endif

    int64_t left_integer = 0;
    int64_t right_integer = 0;
    if (ptn_fast_integer_value(left, &left_integer) && ptn_fast_integer_value(right, &right_integer)) {
        if (right_integer == 0) {
            ptn_throw_exception_at(runtime, "DivisionByZeroError", "Division by zero", runtime->source_path, line);
            return ptn_null();
        }
        if (left_integer == INT64_MIN && right_integer == -1) {
            return ptn_float((double)left_integer / (double)right_integer);
        }
        if (left_integer % right_integer == 0) {
            return ptn_int(left_integer / right_integer);
        }
        return ptn_float((double)left_integer / (double)right_integer);
    }

    double left_fast_number = 0.0;
    double right_fast_number = 0.0;
    if (ptn_fast_numeric_pair(left, right, &left_fast_number, &right_fast_number)) {
        if (right_fast_number == 0.0) {
            ptn_throw_exception_at(runtime, "DivisionByZeroError", "Division by zero", runtime->source_path, line);
            return ptn_null();
        }
        return ptn_float(left_fast_number / right_fast_number);
    }

    PtnNumber left_number;
    PtnNumber right_number;
    ptn_arithmetic_operands(runtime, left, "/", right, line, &left_number, &right_number);
    if (right_number.floating == 0.0) {
        ptn_throw_exception_at(runtime, "DivisionByZeroError", "Division by zero", runtime->source_path, line);
        return ptn_null();
    }

    if (left_number.type == PTN_NUMBER_INT && right_number.type == PTN_NUMBER_INT) {
        if (left_number.integer == INT64_MIN && right_number.integer == -1) {
            return ptn_float((double)left_number.integer / (double)right_number.integer);
        }
        if (left_number.integer % right_number.integer == 0) {
            return ptn_int(left_number.integer / right_number.integer);
        }
    }
    return ptn_float(left_number.floating / right_number.floating);
}

static PTN_UNUSED int ptn_float_to_int_loses_precision(double value) {
    if (!isfinite(value) || value < -9223372036854775808.0 || value >= 9223372036854775808.0) {
        return 1;
    }
    int64_t integer = (int64_t)value;
    return (double)integer != value;
}

static PTN_UNUSED int ptn_float_to_int_out_of_range(double value) {
    return !isfinite(value) || value < -9223372036854775808.0 || value >= 9223372036854775808.0;
}

static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    double value,
    const char *path,
    size_t line
);

static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    const char *value,
    const char *path,
    size_t line
);

static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation(
    PtnDiagnosticSink *diagnostics,
    double value
) {
    ptn_emit_float_to_int_precision_deprecation_at(
        diagnostics,
        value,
        "ptn-generated-code",
        0
    );
}

static PTN_UNUSED void ptn_emit_float_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    double value,
    const char *path,
    size_t line
) {
    if (diagnostics != NULL && !ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    if (diagnostics != NULL) {
        diagnostics->emitted_deprecation = 1;
    }
    char formatted[128];
    ptn_format_scalar_shortest_float(value, formatted, sizeof(formatted));

    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "Implicit conversion from float %s to int loses precision",
        formatted
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    if (
        diagnostics != NULL &&
        ptn_diagnostics_try_error_handler(diagnostics, PTN_E_DEPRECATED, message, path, line)
    ) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nDeprecated: Implicit conversion from float %s to int loses precision in %s on line %zu\n",
        formatted,
        path,
        line
    );
}

static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation(
    PtnDiagnosticSink *diagnostics,
    const char *value
) {
    ptn_emit_float_string_to_int_precision_deprecation_at(
        diagnostics,
        value,
        "ptn-generated-code",
        0
    );
}

static PTN_UNUSED void ptn_emit_float_string_to_int_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    const char *value,
    const char *path,
    size_t line
) {
    if (diagnostics != NULL && !ptn_diagnostics_should_emit(diagnostics, PTN_E_DEPRECATED)) {
        return;
    }
    if (diagnostics != NULL) {
        diagnostics->emitted_deprecation = 1;
    }
    int needed = snprintf(
        NULL,
        0,
        "Implicit conversion from float-string \"%s\" to int loses precision",
        value
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
        "Implicit conversion from float-string \"%s\" to int loses precision",
        value
    );
    if (
        diagnostics != NULL &&
        ptn_diagnostics_try_error_handler(diagnostics, PTN_E_DEPRECATED, message, path, line)
    ) {
        free(message);
        return;
    }
    free(message);
    ptn_diagnostic_printf(
        diagnostics,
        "\nDeprecated: Implicit conversion from float-string \"%s\" to int loses precision in %s on line %zu\n",
        value,
        path,
        line
    );
}

static PTN_UNUSED int ptn_string_has_trailing_non_numeric_data(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    char *end = NULL;
    (void)strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    return *end != '\0';
}

static PTN_UNUSED void ptn_emit_non_numeric_value_warning_at(
    PtnDiagnosticSink *diagnostics,
    const char *path,
    size_t line
) {
    if (diagnostics != NULL && !ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    const char *message = "A non-numeric value encountered";
    if (diagnostics != NULL &&
        ptn_diagnostics_try_error_handler(
            diagnostics,
            PTN_E_WARNING,
            message,
            path,
            line
        )) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nWarning: A non-numeric value encountered in %s on line %zu\n",
        path,
        line
    );
}

static PTN_UNUSED void ptn_emit_non_numeric_value_warning(PtnDiagnosticSink *diagnostics) {
    ptn_emit_non_numeric_value_warning_at(diagnostics, "ptn-generated-code", 0);
}

static PTN_UNUSED int64_t ptn_number_to_integer(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float_to_php_integer(number.floating);
    }
    return number.integer;
}

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    PtnValue value,
    const char *path,
    size_t line
);

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation(
    PtnDiagnosticSink *diagnostics,
    PtnValue value
) {
    return ptn_value_to_integer_with_precision_deprecation_at(
        diagnostics,
        value,
        "ptn-generated-code",
        0
    );
}

static PTN_UNUSED int64_t ptn_value_to_integer_with_precision_deprecation_at(
    PtnDiagnosticSink *diagnostics,
    PtnValue value,
    const char *path,
    size_t line
) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return integer;
    }
    if (value.type == PTN_FLOAT) {
        if (ptn_float_to_int_loses_precision(value.as.floating)) {
            ptn_emit_float_to_int_precision_deprecation_at(
                diagnostics,
                value.as.floating,
                path,
                line
            );
        }
        return ptn_float_to_php_integer(value.as.floating);
    }

    PtnNumber number = ptn_to_number(value);
    const char *string_data = value.type == PTN_STRING ? (const char *)value.as.string.data : "";
    if (value.type == PTN_STRING && ptn_string_has_trailing_non_numeric_data(string_data)) {
        ptn_emit_non_numeric_value_warning_at(diagnostics, path, line);
    }
    if (number.type == PTN_NUMBER_FLOAT && ptn_float_to_int_loses_precision(number.floating)) {
        if (value.type == PTN_STRING) {
            ptn_emit_float_string_to_int_precision_deprecation_at(
                diagnostics,
                string_data,
                path,
                line
            );
        } else {
            ptn_emit_float_to_int_precision_deprecation_at(
                diagnostics,
                number.floating,
                path,
                line
            );
        }
    }
    return ptn_number_to_integer(number);
}

static PTN_UNUSED int64_t ptn_value_to_modulo_integer(PtnRuntime *runtime, PtnValue value, size_t line) {
    value = ptn_value_deref(value);
    if (value.type == PTN_FLOAT && ptn_float_to_int_out_of_range(value.as.floating)) {
        ptn_emit_bitwise_float_out_of_range_warning(&runtime->diagnostics, value.as.floating, line);
        return ptn_float_to_php_integer(value.as.floating);
    }
    return ptn_value_to_integer_with_precision_deprecation_at(
        &runtime->diagnostics,
        value,
        runtime->source_path,
        line
    );
}

static PTN_UNUSED PtnValue ptn_modulo(PtnRuntime *runtime, PtnValue left, PtnValue right, size_t line) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnValue internal_result = ptn_null();
    if (ptn_bcmath_number_binary_op(runtime, "%", left, right, line, &internal_result)) {
        return internal_result;
    }
#endif
    if (ptn_integer_operator_rejects_operand(left) ||
        ptn_integer_operator_rejects_operand(right)) {
        ptn_throw_unsupported_operand_types(runtime, left, "%", right, line);
        return ptn_null();
    }

    int64_t left_fast_integer = 0;
    int64_t right_fast_integer = 0;
    if (ptn_fast_integer_value(left, &left_fast_integer) &&
        ptn_fast_integer_value(right, &right_fast_integer)) {
        if (right_fast_integer == 0) {
            ptn_throw_exception_at(runtime, "DivisionByZeroError", "Modulo by zero", runtime->source_path, line);
            return ptn_null();
        }
        if (left_fast_integer == INT64_MIN && right_fast_integer == -1) {
            return ptn_int(0);
        }
        return ptn_int(left_fast_integer % right_fast_integer);
    }

    int64_t left_integer = ptn_value_to_modulo_integer(runtime, left, line);
    int64_t right_integer = ptn_value_to_modulo_integer(runtime, right, line);
    if (right_integer == 0) {
        ptn_throw_exception_at(runtime, "DivisionByZeroError", "Modulo by zero", runtime->source_path, line);
        return ptn_null();
    }
    if (left_integer == INT64_MIN && right_integer == -1) {
        return ptn_int(0);
    }
    return ptn_int(left_integer % right_integer);
}

static PTN_UNUSED PtnValue ptn_increment_number(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating + 1.0);
    }
    return ptn_add_integers(number.integer, 1);
}

static PTN_UNUSED PtnValue ptn_decrement_number(PtnNumber number) {
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_float(number.floating - 1.0);
    }
    return ptn_subtract_integers(number.integer, 1);
}

static PTN_UNUSED int ptn_increment_string_byte_is_alnum(unsigned char byte) {
    return (byte >= '0' && byte <= '9') ||
        (byte >= 'a' && byte <= 'z') ||
        (byte >= 'A' && byte <= 'Z');
}

static PTN_UNUSED PtnValue ptn_increment_string(PtnRuntime *runtime, PtnString string, size_t line) {
    PtnNumber number;
    int has_trailing_non_numeric_data = 0;
    if (ptn_arithmetic_string_to_number(string, &number, &has_trailing_non_numeric_data) &&
        !has_trailing_non_numeric_data) {
        return ptn_increment_number(number);
    }

    ptn_emit_runtime_deprecation(
        runtime,
        "Increment on non-numeric string is deprecated, use str_increment() instead",
        line
    );

    if (string.len == 0) {
        return ptn_string_literal("1", 1);
    }

    if (!ptn_increment_string_byte_is_alnum(string.data[string.len - 1])) {
        return ptn_owned_string_len(
            ptn_duplicate_string_len((const char *)string.data, string.len),
            string.len
        );
    }

    char *result = ptn_duplicate_string_len((const char *)string.data, string.len);
    int carry = 0;
    char carry_prefix = '\0';
    for (size_t offset = string.len; offset > 0; offset--) {
        size_t index = offset - 1;
        unsigned char byte = (unsigned char)result[index];
        if (byte >= '0' && byte <= '8') {
            result[index] = (char)(byte + 1);
            carry = 0;
            break;
        }
        if (byte == '9') {
            result[index] = '0';
            carry = 1;
            carry_prefix = '1';
            continue;
        }
        if (byte >= 'a' && byte <= 'y') {
            result[index] = (char)(byte + 1);
            carry = 0;
            break;
        }
        if (byte == 'z') {
            result[index] = 'a';
            carry = 1;
            carry_prefix = 'a';
            continue;
        }
        if (byte >= 'A' && byte <= 'Y') {
            result[index] = (char)(byte + 1);
            carry = 0;
            break;
        }
        if (byte == 'Z') {
            result[index] = 'A';
            carry = 1;
            carry_prefix = 'A';
            continue;
        }
        carry = 0;
        break;
    }

    if (carry) {
        if (string.len == SIZE_MAX) {
            free(result);
            ptn_abort_out_of_memory();
        }
        char *prefixed = malloc(string.len + 2);
        if (prefixed == NULL) {
            free(result);
            ptn_abort_out_of_memory();
        }
        prefixed[0] = carry_prefix;
        memcpy(prefixed + 1, result, string.len + 1);
        free(result);
        return ptn_owned_string_len(prefixed, string.len + 1);
    }
    return ptn_owned_string_len(result, string.len);
}

static PTN_UNUSED PtnValue ptn_decrement_string(PtnRuntime *runtime, PtnString string, size_t line) {
    PtnNumber number;
    int has_trailing_non_numeric_data = 0;
    if (ptn_arithmetic_string_to_number(string, &number, &has_trailing_non_numeric_data) &&
        !has_trailing_non_numeric_data) {
        return ptn_decrement_number(number);
    }

    if (string.len == 0) {
        ptn_emit_runtime_deprecation(
            runtime,
            "Decrement on empty string is deprecated as non-numeric",
            line
        );
        return ptn_subtract_integers(0, 1);
    }

    ptn_emit_runtime_deprecation(
        runtime,
        "Decrement on non-numeric string has no effect and is deprecated",
        line
    );

    return ptn_owned_string_len(
        ptn_duplicate_string_len((const char *)string.data, string.len),
        string.len
    );
}

static PTN_UNUSED char *ptn_invalid_increment_decrement_message(
    const char *operation,
    PtnValue value
) {
    const char *type_name = ptn_arithmetic_operand_type_name(value);
    int needed = snprintf(NULL, 0, "Cannot %s %s", operation, type_name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    int written = snprintf(message, (size_t)needed + 1, "Cannot %s %s", operation, type_name);
    if (written < 0 || written != needed) {
        free(message);
        ptn_abort_out_of_memory();
    }
    return message;
}

static PTN_UNUSED int ptn_invalid_increment_decrement_type(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
            return 1;
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_INT:
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED void ptn_prepare_invalid_increment_decrement_previous(
    PtnRuntime *runtime,
    const char *operation,
    PtnValue value,
    size_t line
) {
    char *message = ptn_invalid_increment_decrement_message(operation, value);
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        "TypeError",
        message,
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

static PTN_UNUSED void ptn_throw_invalid_increment_decrement(
    PtnRuntime *runtime,
    const char *operation,
    PtnValue value,
    size_t line
) {
    char *message = ptn_invalid_increment_decrement_message(operation, value);
    ptn_throw_exception_owned_message_at(runtime, "TypeError", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_property_increment_overflow_error(
    PtnRuntime *runtime,
    int increment,
    int reference_context,
    const char *declaring_class,
    const char *property,
    const char *type_text
);

static PTN_UNUSED const PtnReferencePropertyTypeSource *ptn_reference_float_blocking_type_source(
    PtnReference *reference,
    PtnReferencePropertyTypeSource *primary
);

static PTN_UNUSED int ptn_reference_increment_overflow_guard(
    PtnRuntime *runtime,
    PtnValue value,
    int increment
) {
    if (value.type != PTN_REFERENCE) {
        return 0;
    }
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type != PTN_INT ||
        (increment && resolved.as.integer != INT64_MAX) ||
        (!increment && resolved.as.integer != INT64_MIN)) {
        return 0;
    }
    PtnReferencePropertyTypeSource primary;
    const PtnReferencePropertyTypeSource *source =
        ptn_reference_float_blocking_type_source(value.as.reference, &primary);
    if (source == NULL) {
        return 0;
    }
    ptn_throw_property_increment_overflow_error(
        runtime,
        increment,
        1,
        source->declaring_class,
        source->property_name,
        source->text
    );
    return 1;
}

static PTN_UNUSED PtnValue ptn_increment_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    if (ptn_reference_increment_overflow_guard(runtime, value, 1)) {
        return ptn_value_clone(ptn_value_deref(value));
    }
    value = ptn_value_deref(value);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnNumber zend_test_number;
    if (ptn_zend_test_numeric_castable_no_operations_number(value, &zend_test_number)) {
        return ptn_increment_number(zend_test_number);
    }
    PtnValue internal_result = ptn_null();
    if (ptn_bcmath_number_inc_dec(runtime, value, 1, line, &internal_result)) {
        return internal_result;
    }
#endif
    switch (value.type) {
        case PTN_NULL:
            return ptn_int(1);
        case PTN_BOOL:
            ptn_emit_warning(
                &runtime->diagnostics,
                "Increment on type bool has no effect, this will change in the next major version of PHP",
                line
            );
            return ptn_bool(value.as.boolean);
        case PTN_INT:
            return ptn_add_integers(value.as.integer, 1);
        case PTN_FLOAT:
            return ptn_float(value.as.floating + 1.0);
        case PTN_STRING: {
            PtnValue snapshot = ptn_value_clone(value);
            PtnValue result = ptn_increment_string(runtime, snapshot.as.string, line);
            ptn_value_destroy(&snapshot);
            return result;
        }
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            ptn_throw_invalid_increment_decrement(runtime, "increment", value, line);
            return ptn_null();
    }
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_decrement_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    if (ptn_reference_increment_overflow_guard(runtime, value, 0)) {
        return ptn_value_clone(ptn_value_deref(value));
    }
    value = ptn_value_deref(value);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnNumber zend_test_number;
    if (ptn_zend_test_numeric_castable_no_operations_number(value, &zend_test_number)) {
        return ptn_decrement_number(zend_test_number);
    }
    PtnValue internal_result = ptn_null();
    if (ptn_bcmath_number_inc_dec(runtime, value, 0, line, &internal_result)) {
        return internal_result;
    }
#endif
    switch (value.type) {
        case PTN_NULL:
            ptn_emit_warning(
                &runtime->diagnostics,
                "Decrement on type null has no effect, this will change in the next major version of PHP",
                line
            );
            return ptn_null();
        case PTN_BOOL:
            ptn_emit_warning(
                &runtime->diagnostics,
                "Decrement on type bool has no effect, this will change in the next major version of PHP",
                line
            );
            return ptn_bool(value.as.boolean);
        case PTN_INT:
            return ptn_subtract_integers(value.as.integer, 1);
        case PTN_FLOAT:
            return ptn_float(value.as.floating - 1.0);
        case PTN_STRING: {
            PtnValue snapshot = ptn_value_clone(value);
            PtnValue result = ptn_decrement_string(runtime, snapshot.as.string, line);
            ptn_value_destroy(&snapshot);
            return result;
        }
        case PTN_ARRAY:
        case PTN_RESOURCE:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            ptn_throw_invalid_increment_decrement(runtime, "decrement", value, line);
            return ptn_null();
    }
    return ptn_null();
}

static PTN_UNUSED void ptn_throw_property_increment_overflow_error(
    PtnRuntime *runtime,
    int increment,
    int reference_context,
    const char *declaring_class,
    const char *property,
    const char *type_text
) {
    char message[384];
    const char *operation = increment ? "increment" : "decrement";
    const char *boundary = increment ? "maximal" : "minimal";
    const char *declared_type = type_text == NULL ? "mixed" : type_text;
    int written;
    if (reference_context) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot %s a reference held by property %s::$%s of type %s past its %s value",
            operation,
            declaring_class,
            property,
            declared_type,
            boundary
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot %s property %s::$%s of type %s past its %s value",
            operation,
            declaring_class,
            property,
            declared_type,
            boundary
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "TypeError", message);
}

static PTN_UNUSED const PtnReferencePropertyTypeSource *ptn_reference_float_blocking_type_source(
    PtnReference *reference,
    PtnReferencePropertyTypeSource *primary
) {
    if (reference == NULL || reference->property_type_kind == PTN_PROPERTY_TYPE_NONE) {
        return NULL;
    }
    *primary = ptn_reference_primary_property_type_source(reference);
    if (!ptn_reference_property_type_source_allows_float(primary)) {
        return primary;
    }
    for (size_t i = 0; i < reference->property_type_source_len; i++) {
        if (!ptn_reference_property_type_source_allows_float(&reference->property_type_sources[i])) {
            return &reference->property_type_sources[i];
        }
    }
    return NULL;
}

static PTN_UNUSED PtnValue ptn_property_increment_value(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    PtnValue current,
    int increment,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(current);
    PtnValue object = ptn_value_deref(receiver);
    if (object.type == PTN_OBJECT) {
        char *storage_key = ptn_object_resolve_property_storage_key(
            runtime,
            object.as.object,
            property,
            access_scope,
            PTN_PROPERTY_ACCESS_READ,
            1,
            line
        );
        if (storage_key != NULL) {
            const PtnObjectPropertyMetadata *metadata =
                ptn_object_property_metadata(object.as.object, storage_key);
            PtnArrayKey key = ptn_array_string_key(storage_key);
            PtnArrayEntry *entry = ptn_array_entry_for_key(object.as.object->properties, key);
            ptn_array_key_free(key);
            if (metadata != NULL && metadata->is_readonly && entry != NULL) {
                PtnObjectPropertyMetadata *mutable_metadata =
                    ptn_object_mutable_property_metadata(object.as.object, storage_key);
                int readonly_clone_reinit =
                    object.as.object->readonly_clone_initializing &&
                    mutable_metadata != NULL &&
                    !mutable_metadata->readonly_clone_reinitialized;
                if (!readonly_clone_reinit) {
                    if (ptn_invalid_increment_decrement_type(resolved)) {
                        ptn_prepare_invalid_increment_decrement_previous(
                            runtime,
                            increment ? "increment" : "decrement",
                            resolved,
                            line
                        );
                    }
                    ptn_throw_readonly_property_error(
                        runtime,
                        object.as.object->class_name,
                        metadata->declaring_class,
                        metadata->display_name,
                        line
                    );
                    free(storage_key);
                    return ptn_value_clone(resolved);
                }
            }
            free(storage_key);
        }
    }
    int boundary = resolved.type == PTN_INT &&
        ((increment && resolved.as.integer == INT64_MAX) ||
            (!increment && resolved.as.integer == INT64_MIN));
    if (boundary) {
        if (object.type == PTN_OBJECT) {
            char *storage_key = ptn_object_resolve_property_storage_key(
                runtime,
                object.as.object,
                property,
                access_scope,
                PTN_PROPERTY_ACCESS_READ,
                1,
                line
            );
            if (storage_key != NULL) {
                const PtnObjectPropertyMetadata *metadata =
                    ptn_object_property_metadata(object.as.object, storage_key);
                PtnArrayKey key = ptn_array_string_key(storage_key);
                PtnArrayEntry *entry = ptn_array_entry_for_key(object.as.object->properties, key);
                if (entry != NULL && entry->value.type == PTN_REFERENCE) {
                    PtnReferencePropertyTypeSource primary;
                    const PtnReferencePropertyTypeSource *source =
                        ptn_reference_float_blocking_type_source(entry->value.as.reference, &primary);
                    if (source != NULL) {
                        ptn_throw_property_increment_overflow_error(
                            runtime,
                            increment,
                            1,
                            source->declaring_class,
                            source->property_name,
                            source->text
                        );
                        ptn_array_key_free(key);
                        free(storage_key);
                        return ptn_value_clone(resolved);
                    }
                }
                ptn_array_key_free(key);
                if (metadata != NULL &&
                    ptn_property_type_is_declared(metadata->type_kind) &&
                    !ptn_property_type_allows_float(metadata->type_kind, metadata->type_text)) {
                    ptn_throw_property_increment_overflow_error(
                        runtime,
                        increment,
                        0,
                        metadata->declaring_class,
                        metadata->display_name,
                        metadata->type_text
                    );
                    free(storage_key);
                    return ptn_value_clone(resolved);
                }
                free(storage_key);
            }
        }
    }

    return increment
        ? ptn_increment_value(runtime, resolved, line)
        : ptn_decrement_value(runtime, resolved, line);
}

static PTN_UNUSED PtnValue ptn_static_property_increment_value(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    PtnValue current,
    int increment,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(current);
    int boundary = resolved.type == PTN_INT &&
        ((increment && resolved.as.integer == INT64_MAX) ||
            (!increment && resolved.as.integer == INT64_MIN));
    if (boundary) {
        const char *declaring_class = NULL;
        char *key = ptn_runtime_resolve_static_property_key(
            runtime,
            class_name,
            property,
            &declaring_class
        );
        if (key != NULL) {
            PtnValue current_slot;
            if (ptn_symbols_get(ptn_runtime_static_property_table(runtime), key, &current_slot) &&
                current_slot.type == PTN_REFERENCE) {
                PtnReferencePropertyTypeSource primary;
                const PtnReferencePropertyTypeSource *source =
                    ptn_reference_float_blocking_type_source(current_slot.as.reference, &primary);
                if (source != NULL) {
                    ptn_throw_property_increment_overflow_error(
                        runtime,
                        increment,
                        1,
                        source->declaring_class,
                        source->property_name,
                        source->text
                    );
                    free(key);
                    return ptn_value_clone(resolved);
                }
            }
            PtnObjectPropertyMetadata metadata;
            if (ptn_runtime_static_property_metadata(
                    runtime,
                    key,
                    declaring_class,
                    property,
                    &metadata
                ) &&
                ptn_property_type_is_declared(metadata.type_kind) &&
                !ptn_property_type_allows_float(metadata.type_kind, metadata.type_text)) {
                ptn_throw_property_increment_overflow_error(
                    runtime,
                    increment,
                    0,
                    metadata.declaring_class,
                    metadata.display_name,
                    metadata.type_text
                );
                free(key);
                return ptn_value_clone(resolved);
            }
            free(key);
        }
    }

    return increment
        ? ptn_increment_value(runtime, resolved, line)
        : ptn_decrement_value(runtime, resolved, line);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_and(PtnStringOperand left, PtnStringOperand right) {
    size_t left_len = left.len;
    size_t right_len = right.len;
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left.data[i] & (unsigned char)right.data[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string_len(result, result_len);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_or(PtnStringOperand left, PtnStringOperand right) {
    size_t left_len = left.len;
    size_t right_len = right.len;
    size_t result_len = left_len > right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        unsigned char left_byte = i < left_len ? (unsigned char)left.data[i] : 0;
        unsigned char right_byte = i < right_len ? (unsigned char)right.data[i] : 0;
        result[i] = (char)(left_byte | right_byte);
    }
    result[result_len] = '\0';
    return ptn_owned_string_len(result, result_len);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_xor(PtnStringOperand left, PtnStringOperand right) {
    size_t left_len = left.len;
    size_t right_len = right.len;
    size_t result_len = left_len < right_len ? left_len : right_len;
    char *result = malloc(result_len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < result_len; i++) {
        result[i] = (char)((unsigned char)left.data[i] ^ (unsigned char)right.data[i]);
    }
    result[result_len] = '\0';
    return ptn_owned_string_len(result, result_len);
}

static PTN_UNUSED PtnValue ptn_bitwise_string_not(PtnStringOperand value) {
    size_t len = value.len;
    char *result = malloc(len + 1);
    if (result == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        result[i] = (char)(~(unsigned char)value.data[i]);
    }
    result[len] = '\0';
    return ptn_owned_string_len(result, len);
}

static PTN_UNUSED int64_t ptn_value_to_integer(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(NULL, value);
}

static PTN_UNUSED int64_t ptn_bitwise_integer_operand(PtnValue value) {
    return ptn_value_to_integer_with_precision_deprecation(NULL, value);
}

static PTN_UNUSED void ptn_format_bitwise_float_diagnostic(double value, char *buffer, size_t buffer_size) {
    if (isfinite(value)) {
        ptn_format_scalar_shortest_float(value, buffer, buffer_size);
    } else {
        ptn_format_scalar_float(value, buffer, buffer_size);
    }
}

static PTN_UNUSED void ptn_emit_bitwise_float_out_of_range_warning(
    PtnDiagnosticSink *diagnostics,
    double value,
    size_t line
) {
    if (!ptn_diagnostics_should_emit(diagnostics, PTN_E_WARNING)) {
        return;
    }
    char formatted[64];
    ptn_format_bitwise_float_diagnostic(value, formatted, sizeof(formatted));
    char message[128];
    int written = snprintf(
        message,
        sizeof(message),
        "The float %s is not representable as an int, cast occurred",
        formatted
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    diagnostics->emitted_warning = 1;
    if (ptn_diagnostics_try_error_handler(diagnostics, PTN_E_WARNING, message, NULL, line)) {
        return;
    }
    ptn_diagnostic_printf(
        diagnostics,
        "\nWarning: The float %s is not representable as an int, cast occurred in %s on line %zu\n",
        formatted,
        ptn_diagnostic_builtin_path(line),
        line
    );
}

static PTN_UNUSED int64_t ptn_bitwise_integer_operand_checked(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_FLOAT) {
        if (ptn_float_to_int_out_of_range(value.as.floating)) {
            ptn_emit_bitwise_float_out_of_range_warning(&runtime->diagnostics, value.as.floating, line);
            return ptn_float_to_php_integer(value.as.floating);
        }
        if (ptn_float_to_int_loses_precision(value.as.floating)) {
            ptn_emit_float_to_int_precision_deprecation_at(
                &runtime->diagnostics,
                value.as.floating,
                runtime->source_path,
                line
            );
        }
        return ptn_float_to_php_integer(value.as.floating);
    }
    return ptn_value_to_integer_with_precision_deprecation_at(
        &runtime->diagnostics,
        value,
        runtime->source_path,
        line
    );
}

static PTN_UNUSED PtnValue ptn_bitwise_and(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        PtnStringOperand left_string = {
            (const char *)left.as.string.data,
            NULL,
            left.as.string.len
        };
        PtnStringOperand right_string = {
            (const char *)right.as.string.data,
            NULL,
            right.as.string.len
        };
        return ptn_bitwise_string_and(left_string, right_string);
    }
    if (ptn_integer_operator_rejects_operand(left) ||
        ptn_integer_operator_rejects_operand(right)) {
        ptn_throw_unsupported_operand_types(runtime, left, "&", right, line);
        return ptn_null();
    }
    return ptn_int(
        ptn_bitwise_integer_operand_checked(runtime, left, line) &
        ptn_bitwise_integer_operand_checked(runtime, right, line)
    );
}

static PTN_UNUSED PtnValue ptn_bitwise_or(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        PtnStringOperand left_string = {
            (const char *)left.as.string.data,
            NULL,
            left.as.string.len
        };
        PtnStringOperand right_string = {
            (const char *)right.as.string.data,
            NULL,
            right.as.string.len
        };
        return ptn_bitwise_string_or(left_string, right_string);
    }
    if (ptn_integer_operator_rejects_operand(left) ||
        ptn_integer_operator_rejects_operand(right)) {
        ptn_throw_unsupported_operand_types(runtime, left, "|", right, line);
        return ptn_null();
    }
    return ptn_int(
        ptn_bitwise_integer_operand_checked(runtime, left, line) |
        ptn_bitwise_integer_operand_checked(runtime, right, line)
    );
}

static PTN_UNUSED PtnValue ptn_bitwise_xor(
    PtnRuntime *runtime,
    PtnValue left,
    PtnValue right,
    size_t line
) {
    left = ptn_value_deref(left);
    right = ptn_value_deref(right);
    if (left.type == PTN_STRING && right.type == PTN_STRING) {
        PtnStringOperand left_string = {
            (const char *)left.as.string.data,
            NULL,
            left.as.string.len
        };
        PtnStringOperand right_string = {
            (const char *)right.as.string.data,
            NULL,
            right.as.string.len
        };
        return ptn_bitwise_string_xor(left_string, right_string);
    }
    if (ptn_integer_operator_rejects_operand(left) ||
        ptn_integer_operator_rejects_operand(right)) {
        ptn_throw_unsupported_operand_types(runtime, left, "^", right, line);
        return ptn_null();
    }
    return ptn_int(
        ptn_bitwise_integer_operand_checked(runtime, left, line) ^
        ptn_bitwise_integer_operand_checked(runtime, right, line)
    );
}
