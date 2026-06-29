static void ptn_declared_class_property_hook_deprecation(PtnRuntime *runtime, const char *class_name, const char *property_name, int hook_type, size_t line);
static PTN_UNUSED int ptn_internal_class_name_is_caching_iterator(const char *class_name);

static PTN_UNUSED void ptn_runtime_init_function_frame(PtnRuntime *runtime, PtnRuntime *caller_runtime) {
    ptn_symbols_init(&runtime->symbols);
    runtime->global_symbols = caller_runtime->global_symbols;
    ptn_symbols_init(&runtime->owned_constants);
    runtime->constants = caller_runtime->constants;
    ptn_symbols_init(&runtime->owned_constant_sources);
    runtime->constant_sources = caller_runtime->constant_sources;
    ptn_symbols_init(&runtime->owned_class_aliases);
    runtime->class_aliases = caller_runtime->class_aliases;
    ptn_symbols_init(&runtime->owned_dynamic_classes);
    runtime->dynamic_classes = caller_runtime->dynamic_classes;
    ptn_symbols_init(&runtime->owned_class_constants);
    runtime->class_constants = caller_runtime->class_constants;
    ptn_symbols_init(&runtime->owned_class_constant_deprecations);
    runtime->class_constant_deprecations = caller_runtime->class_constant_deprecations;
    ptn_symbols_init(&runtime->owned_class_constant_initializing);
    runtime->class_constant_initializing = caller_runtime->class_constant_initializing;
    runtime->current_class_constant_initializing_class_name =
        caller_runtime->current_class_constant_initializing_class_name;
    runtime->current_class_constant_initializing_key_class_name =
        caller_runtime->current_class_constant_initializing_key_class_name;
    runtime->current_class_constant_initializing_constant_name =
        caller_runtime->current_class_constant_initializing_constant_name;
    runtime->current_class_constant_source_path =
        caller_runtime->current_class_constant_source_path;
    runtime->class_constant_deprecation_suppress_class =
        caller_runtime->class_constant_deprecation_suppress_class;
    runtime->class_constant_deprecation_suppress_constant =
        caller_runtime->class_constant_deprecation_suppress_constant;
    runtime->dynamic_property_deprecation_suppress_object = NULL;
    runtime->dynamic_property_deprecation_suppress_property = NULL;
    ptn_symbols_init(&runtime->owned_static_properties);
    runtime->static_properties = caller_runtime->static_properties;
    ptn_symbols_init(&runtime->owned_static_property_initialized);
    runtime->static_property_initialized = caller_runtime->static_property_initialized;
    ptn_symbols_init(&runtime->owned_static_property_read_visibility);
    runtime->static_property_read_visibility = caller_runtime->static_property_read_visibility;
    ptn_symbols_init(&runtime->owned_static_property_set_visibility);
    runtime->static_property_set_visibility = caller_runtime->static_property_set_visibility;
    ptn_symbols_init(&runtime->owned_static_property_type_kind);
    runtime->static_property_type_kind = caller_runtime->static_property_type_kind;
    ptn_symbols_init(&runtime->owned_static_property_type_class_name);
    runtime->static_property_type_class_name = caller_runtime->static_property_type_class_name;
    ptn_symbols_init(&runtime->owned_static_property_type_text);
    runtime->static_property_type_text = caller_runtime->static_property_type_text;
    ptn_symbols_init(&runtime->owned_static_property_type_allows_null);
    runtime->static_property_type_allows_null = caller_runtime->static_property_type_allows_null;
    ptn_diagnostics_init(&runtime->diagnostics, NULL);
    runtime->diagnostics.runtime = runtime;
    runtime->diagnostics.error_reporting = caller_runtime->diagnostics.error_reporting;
    runtime->diagnostics.emitted_deprecation = caller_runtime->diagnostics.emitted_deprecation;
    runtime->diagnostics.emitted_warning = caller_runtime->diagnostics.emitted_warning;
    runtime->diagnostics.suppressed = caller_runtime->diagnostics.suppressed;
    runtime->date_timezone_startup_warning_emitted =
        caller_runtime->date_timezone_startup_warning_emitted;
    runtime->owned_exceptions.active_exception = NULL;
    runtime->owned_exceptions.try_frame = NULL;
    ptn_exception_handlers_init(&runtime->owned_exceptions);
    runtime->exceptions = caller_runtime->exceptions;
    runtime->native_argc = caller_runtime->native_argc;
    runtime->native_argv = caller_runtime->native_argv;
    runtime->owned_call_frame.argc = 0;
    runtime->owned_call_frame.args = NULL;
    runtime->owned_call_frame.arg_names = NULL;
    runtime->owned_call_frame.parameter_count = 0;
    runtime->owned_call_frame.parameter_names = NULL;
    runtime->owned_call_frame.has_current_closure = 0;
    runtime->owned_call_frame.current_closure = ptn_null();
    runtime->call_frame = NULL;
    runtime->next_call_arg_names = NULL;
    runtime->owned_trace_frame.runtime = NULL;
    runtime->owned_trace_frame.function_name = NULL;
    runtime->owned_trace_frame.file = NULL;
    runtime->owned_trace_frame.line = 0;
    runtime->owned_trace_frame.argc = 0;
    runtime->owned_trace_frame.args = NULL;
    runtime->owned_trace_frame.arg_names = NULL;
    runtime->owned_trace_frame.parameter_count = 0;
    runtime->owned_trace_frame.parameter_names = NULL;
    runtime->owned_trace_frame.sensitive_parameter_count = 0;
    runtime->owned_trace_frame.sensitive_parameters = NULL;
    runtime->owned_trace_frame.sensitive_variadic_position = (size_t)-1;
    runtime->owned_trace_frame.has_receiver = 0;
    runtime->owned_trace_frame.receiver = ptn_null();
    runtime->owned_trace_frame.previous = NULL;
    runtime->trace_frame = caller_runtime->trace_frame;
    runtime->lifecycle_root = caller_runtime->lifecycle_root == NULL
        ? caller_runtime
        : caller_runtime->lifecycle_root;
    runtime->live_objects = NULL;
    runtime->live_objects_len = 0;
    runtime->live_objects_capacity = 0;
    runtime->live_closures = NULL;
    runtime->live_closures_len = 0;
    runtime->live_closures_capacity = 0;
    runtime->first_class_callable_cache_values = NULL;
    runtime->first_class_callable_cache_names = NULL;
    runtime->first_class_callable_cache_len = 0;
    runtime->first_class_callable_cache_capacity = 0;
    runtime->live_arrays = NULL;
    runtime->live_arrays_len = 0;
    runtime->live_arrays_capacity = 0;
    runtime->live_references = NULL;
    runtime->live_references_len = 0;
    runtime->live_references_capacity = 0;
    runtime->temporary_roots = NULL;
    runtime->temporary_roots_len = 0;
    runtime->temporary_roots_capacity = 0;
    runtime->static_local_slots = NULL;
    runtime->static_local_slots_len = 0;
    runtime->static_local_slots_capacity = 0;
    runtime->next_object_id = 0;
    runtime->free_object_ids = NULL;
    runtime->free_object_ids_len = 0;
    runtime->free_object_ids_capacity = 0;
    runtime->deferred_free_object_id = 0;
    runtime->has_deferred_free_object_id = 0;
    runtime->output_buffers = NULL;
    runtime->output_buffers_len = 0;
    runtime->output_buffers_capacity = 0;
    runtime->output_buffer_callback_depth = 0;
    runtime->output_buffer_callback_function_name = NULL;
    runtime->output_buffer_callback_handler_name = NULL;
    runtime->output_buffer_callback_line = 0;
    runtime->output_buffer_callback_output_warned = 0;
    runtime->output_buffer_callback_passthrough_output = 0;
    runtime->output_buffer_callback_skip_buffers = 0;
    runtime->output_at_line_start = 1;
    runtime->output_has_started = 0;
    runtime->http_response_code_initialized = 0;
    runtime->http_response_code = 0;
    runtime->header_callback_registered = 0;
    runtime->header_callback_running = 0;
    runtime->header_callback_completed = 0;
    runtime->header_callback = ptn_null();
    runtime->shutdown_functions = NULL;
    runtime->shutdown_functions_len = 0;
    runtime->shutdown_functions_capacity = 0;
    runtime->shutdown_function_index = 0;
    runtime->shutdown_functions_running = 0;
    runtime->shutdown_functions_completed = 0;
    runtime->shutdown_in_progress = 0;
    runtime->tick_enabled = caller_runtime->tick_enabled;
    runtime->tick_functions = NULL;
    runtime->tick_functions_len = 0;
    runtime->tick_functions_capacity = 0;
    runtime->tick_functions_running = 0;
    runtime->defer_uncaught_exception_emit = 0;
    runtime->method_dispatch = caller_runtime->method_dispatch;
    runtime->reflected_method_dispatch = caller_runtime->reflected_method_dispatch;
    runtime->declared_method_exists = caller_runtime->declared_method_exists;
    runtime->declared_method_metadata = caller_runtime->declared_method_metadata;
    runtime->declared_method_visible = caller_runtime->declared_method_visible;
    runtime->declared_method_visibility_metadata = caller_runtime->declared_method_visibility_metadata;
    runtime->class_scope_allows = caller_runtime->class_scope_allows;
    runtime->declared_class_is_readonly = caller_runtime->declared_class_is_readonly;
    runtime->declared_class_allows_dynamic_properties =
        caller_runtime->declared_class_allows_dynamic_properties;
    runtime->magic_property_read = caller_runtime->magic_property_read;
    runtime->magic_property_isset = caller_runtime->magic_property_isset;
    runtime->declared_user_functions = caller_runtime->declared_user_functions;
    runtime->declared_user_classes = caller_runtime->declared_user_classes;
    runtime->declared_user_traits = caller_runtime->declared_user_traits;
    runtime->magic_property_get = caller_runtime->magic_property_get;
    runtime->magic_property_get_exists = caller_runtime->magic_property_get_exists;
    runtime->magic_property_set = caller_runtime->magic_property_set;
    runtime->magic_property_unset = caller_runtime->magic_property_unset;
    runtime->magic_debug_info = caller_runtime->magic_debug_info;
    runtime->property_hook_get = caller_runtime->property_hook_get;
    runtime->property_hook_set = caller_runtime->property_hook_set;
    runtime->active_property_hook_class = NULL;
    runtime->active_property_hook_property = NULL;
    runtime->class_constant_initializer = caller_runtime->class_constant_initializer;
    runtime->static_property_initializer = caller_runtime->static_property_initializer;
    runtime->new_instance_without_constructor = caller_runtime->new_instance_without_constructor;
    runtime->in_magic_property_dispatch = caller_runtime->in_magic_property_dispatch;
    runtime->active_spl_object_storage_get_hash_depth = 0;
    runtime->magic_property_frames = NULL;
    runtime->magic_property_frame_len = caller_runtime->magic_property_frame_len;
    runtime->magic_property_frame_capacity = caller_runtime->magic_property_frame_len;
    if (runtime->magic_property_frame_len != 0) {
        runtime->magic_property_frames =
            malloc(runtime->magic_property_frame_len * sizeof(PtnMagicPropertyFrame));
        if (runtime->magic_property_frames == NULL) {
            ptn_abort_out_of_memory();
        }
        for (size_t i = 0; i < runtime->magic_property_frame_len; i++) {
            runtime->magic_property_frames[i].object_id =
                caller_runtime->magic_property_frames[i].object_id;
            runtime->magic_property_frames[i].effective_object_id =
                caller_runtime->magic_property_frames[i].effective_object_id;
            runtime->magic_property_frames[i].property_len =
                caller_runtime->magic_property_frames[i].property_len;
            runtime->magic_property_frames[i].operation =
                caller_runtime->magic_property_frames[i].operation;
            runtime->magic_property_frames[i].property =
                ptn_duplicate_string_len(
                    caller_runtime->magic_property_frames[i].property,
                    caller_runtime->magic_property_frames[i].property_len
                );
        }
    }
    runtime->source_path = caller_runtime->source_path;
    runtime->source_snapshot_data = caller_runtime->source_snapshot_data;
    runtime->source_snapshot_len = caller_runtime->source_snapshot_len;
    runtime->compiled_include_depth = caller_runtime->compiled_include_depth;
    runtime->in_preload = caller_runtime->in_preload;
    runtime->current_function_name = NULL;
    runtime->current_class_name = NULL;
    runtime->current_called_class_name = NULL;
    runtime->called_class_name_override = NULL;
    runtime->forward_static_called_class_name = NULL;
    runtime->destructor_access_scope = NULL;
    runtime->destructor_shutdown_phase = 0;
    runtime->current_generator = NULL;
    runtime->pending_generator_assignment_name =
        caller_runtime->pending_generator_assignment_name;
    runtime->pending_yield_from_generator =
        caller_runtime->pending_yield_from_generator;
    runtime->pending_yield_from_line = caller_runtime->pending_yield_from_line;
    runtime->implicit_generator_foreach_rewind =
        caller_runtime->implicit_generator_foreach_rewind;
    runtime->implicit_generator_foreach_source_path =
        caller_runtime->implicit_generator_foreach_source_path;
    runtime->implicit_generator_foreach_line =
        caller_runtime->implicit_generator_foreach_line;
    runtime->generator_aborted_after_yield = 0;
    runtime->generator_aborted_rethrow_on_rewind = 0;
    runtime->generator_chained_exception_during_unwind = 0;
    runtime->defer_unreferenced_destructors_for_catch = 0;
    runtime->deferred_yield_from_iterator_object = ptn_null();
    runtime->suppress_generator_rewind_trace_frame =
        caller_runtime->suppress_generator_rewind_trace_frame;
    runtime->current_fiber = caller_runtime->current_fiber;
    runtime->has_current_receiver = 0;
    runtime->current_receiver = ptn_null();
    runtime->by_ref_argument_function_name_override =
        caller_runtime->by_ref_argument_function_name_override;
    runtime->by_ref_argument_notice_pending = caller_runtime->by_ref_argument_notice_pending;
    runtime->by_ref_argument_notice_emitted = caller_runtime->by_ref_argument_notice_emitted;
    runtime->by_ref_argument_notice_line = caller_runtime->by_ref_argument_notice_line;
    runtime->suppress_scoped_callable_deprecation = 0;
    runtime->include_path = NULL;
    runtime->included_files = NULL;
    runtime->included_files_len = 0;
    runtime->included_files_capacity = 0;
    runtime->autoload_callbacks = NULL;
    runtime->autoload_callback_scope_class_names = NULL;
    runtime->autoload_callback_called_class_names = NULL;
    runtime->autoload_callbacks_len = 0;
    runtime->autoload_callbacks_capacity = 0;
    runtime->spl_autoload_extensions = NULL;
    runtime->autoloading_class_names = NULL;
    runtime->autoloading_class_names_len = 0;
    runtime->autoloading_class_names_capacity = 0;
    runtime->last_opened_directory = NULL;
    runtime->open_basedir = NULL;
    runtime->memory_limit = NULL;
    runtime->max_memory_limit = NULL;
    runtime->default_charset = NULL;
    runtime->arg_separator_input = NULL;
    runtime->arg_separator_output = NULL;
    runtime->output_handler = NULL;
    runtime->filter_default = NULL;
    runtime->internal_encoding = NULL;
    runtime->input_encoding = NULL;
    runtime->output_encoding = NULL;
    runtime->variables_order = NULL;
    runtime->register_argc_argv = NULL;
    runtime->enable_post_data_reading = NULL;
    runtime->native_argc = caller_runtime->native_argc;
    runtime->native_argv = caller_runtime->native_argv;
    runtime->file_uploads = NULL;
    runtime->max_input_vars = NULL;
    runtime->max_input_nesting_level = NULL;
    runtime->post_max_size = NULL;
    runtime->always_populate_raw_post_data = NULL;
    runtime->upload_tmp_dir = NULL;
    runtime->expose_php = NULL;
    runtime->docref_root = NULL;
    runtime->unserialize_callback_func = NULL;
    runtime->unserialize_max_depth = caller_runtime->unserialize_max_depth;
    runtime->request_body = NULL;
    runtime->request_body_len = 0;
    ptn_symbols_init(&runtime->session_ini);
    runtime->session_id = NULL;
    runtime->session_active = caller_runtime->session_active;
    runtime->session_was_started = caller_runtime->session_was_started;
    runtime->session_auto_started = caller_runtime->session_auto_started;
    runtime->session_start_path = caller_runtime->session_start_path;
    runtime->session_start_line = caller_runtime->session_start_line;
    runtime->session_save_handler_kind = 0;
    runtime->session_save_handler_object = ptn_null();
    for (size_t i = 0; i < sizeof(runtime->session_save_handler_callbacks) / sizeof(runtime->session_save_handler_callbacks[0]); i++) {
        runtime->session_save_handler_callbacks[i] = ptn_null();
    }
    runtime->session_save_handler_register_shutdown = caller_runtime->session_save_handler_register_shutdown;
    runtime->session_save_handler_in_callback = 0;
    runtime->session_save_handler_shutdown_warning_pending = 0;
    runtime->session_parent_handler_open = caller_runtime->session_parent_handler_open;
    runtime->session_parent_save_handler = caller_runtime->session_parent_save_handler == NULL
        ? NULL
        : ptn_duplicate_string(caller_runtime->session_parent_save_handler);
    runtime->session_lazy_write = caller_runtime->session_lazy_write;
    runtime->session_last_data = NULL;
    runtime->session_last_data_len = 0;
    runtime->session_last_data_valid = 0;
    runtime->precision = caller_runtime->precision;
    runtime->serialize_precision = caller_runtime->serialize_precision;
    runtime->initial_precision = caller_runtime->initial_precision;
    runtime->initial_serialize_precision = caller_runtime->initial_serialize_precision;
    runtime->bcmath_scale = caller_runtime->bcmath_scale;
    runtime->initial_bcmath_scale = caller_runtime->initial_bcmath_scale;
    runtime->exception_ignore_args = caller_runtime->exception_ignore_args;
    runtime->exception_string_param_max_len = caller_runtime->exception_string_param_max_len;
    runtime->strict_types = caller_runtime->strict_types;
    runtime->tick_enabled = caller_runtime->tick_enabled;
    runtime->initial_zend_assertions = caller_runtime->initial_zend_assertions;
    runtime->zend_assertions = caller_runtime->zend_assertions;
    runtime->assert_active = caller_runtime->assert_active;
    runtime->assert_warning = caller_runtime->assert_warning;
    runtime->assert_bail = caller_runtime->assert_bail;
    runtime->assert_callback_ini = NULL;
    runtime->assert_callback = ptn_null();
    runtime->assert_exception = caller_runtime->assert_exception;
    runtime->disabled_functions = caller_runtime->disabled_functions;
    runtime->call_site_line = 0;
    runtime->suppress_user_call_frame_location =
        caller_runtime->suppress_user_call_frame_location;
    runtime->suppress_user_argument_count_location =
        caller_runtime->suppress_user_argument_count_location;
    runtime->warn_by_ref_argument_mismatch = caller_runtime->warn_by_ref_argument_mismatch;
    runtime->throw_argument_count_errors = caller_runtime->throw_argument_count_errors;
    runtime->gc_enabled = caller_runtime->gc_enabled;
    runtime->gc_running = caller_runtime->gc_running;
    runtime->gc_mark_epoch = caller_runtime->gc_mark_epoch;
    runtime->gc_runs = caller_runtime->gc_runs;
    runtime->gc_collected = caller_runtime->gc_collected;
    runtime->gc_roots = caller_runtime->gc_roots;
    runtime->active_serialize_state = caller_runtime->active_serialize_state;
    runtime->active_unserialize_state = caller_runtime->active_unserialize_state;
    runtime->strtok_string = NULL;
    runtime->strtok_len = 0;
    runtime->strtok_offset = 0;
    runtime->strtok_has_state = 0;
    runtime->json_last_error = caller_runtime->json_last_error;
    runtime->json_last_error_line = caller_runtime->json_last_error_line;
    runtime->json_last_error_column = caller_runtime->json_last_error_column;
    runtime->pcre_last_error = caller_runtime->pcre_last_error;
    runtime->pcre_utf8_cache_data = NULL;
    runtime->pcre_utf8_cache_len = 0;
    runtime->pcre_utf8_cache_known = 0;
    runtime->pcre_utf8_cache_valid = 0;
    runtime->intl_error_level = caller_runtime->intl_error_level;
    runtime->intl_use_exceptions = caller_runtime->intl_use_exceptions;
    runtime->intl_last_error_message = NULL;
}

static PTN_UNUSED void ptn_runtime_set_call_frame(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    const char *const *arg_names,
    size_t parameter_count,
    const char *const *parameter_names,
    size_t sensitive_parameter_count,
    const unsigned char *sensitive_parameters,
    size_t sensitive_variadic_position
) {
    size_t limit = 0;
    if (ptn_runtime_memory_limit_bytes(runtime, &limit) && limit != 0) {
        const size_t estimated_frame_bytes = 8192;
        size_t depth = 1;
        for (PtnTraceFrame *frame = runtime->trace_frame;
             frame != NULL;
             frame = frame->previous) {
            if (depth > SIZE_MAX / estimated_frame_bytes) {
                depth = SIZE_MAX / estimated_frame_bytes;
                break;
            }
            depth++;
        }
        size_t estimated_usage = depth * estimated_frame_bytes;
        if (estimated_usage > limit) {
            char message[192];
            int written = snprintf(
                message,
                sizeof(message),
                "Allowed memory size of %zu bytes exhausted (tried to allocate %zu bytes)",
                limit,
                estimated_frame_bytes
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_emit_fatal_error_at(
                runtime,
                message,
                runtime->source_path,
                runtime->call_site_line
            );
        }
    }
    runtime->owned_call_frame.argc = argc;
    runtime->owned_call_frame.args = args;
    runtime->owned_call_frame.arg_names = arg_names;
    runtime->owned_call_frame.parameter_count = parameter_count;
    runtime->owned_call_frame.parameter_names = parameter_names;
    runtime->owned_call_frame.has_current_closure = 0;
    runtime->owned_call_frame.current_closure = ptn_null();
    runtime->call_frame = &runtime->owned_call_frame;
    runtime->owned_trace_frame.runtime = runtime;
    runtime->owned_trace_frame.function_name = runtime->current_function_name;
    runtime->owned_trace_frame.file =
        runtime->suppress_user_call_frame_location ? NULL : runtime->source_path;
    runtime->owned_trace_frame.line =
        runtime->suppress_user_call_frame_location ? 0 : runtime->call_site_line;
    runtime->owned_trace_frame.argc = argc;
    runtime->owned_trace_frame.args = args;
    runtime->owned_trace_frame.arg_names = arg_names;
    runtime->owned_trace_frame.parameter_count = parameter_count;
    runtime->owned_trace_frame.parameter_names = parameter_names;
    runtime->owned_trace_frame.sensitive_parameter_count = sensitive_parameter_count;
    runtime->owned_trace_frame.sensitive_parameters = sensitive_parameters;
    runtime->owned_trace_frame.sensitive_variadic_position = sensitive_variadic_position;
    runtime->owned_trace_frame.has_receiver = runtime->has_current_receiver;
    runtime->owned_trace_frame.receiver = runtime->current_receiver;
    runtime->owned_trace_frame.previous = runtime->trace_frame;
    runtime->trace_frame = &runtime->owned_trace_frame;
    runtime->suppress_user_call_frame_location = 0;
}

static PTN_UNUSED void ptn_runtime_drop_call_frame_arguments(PtnRuntime *runtime) {
    if (runtime == NULL || runtime->call_frame == NULL || runtime->call_frame->args == NULL) {
        return;
    }
    /* Call-frame arguments are borrowed from the caller; only detach stale frame pointers. */
    runtime->owned_call_frame.argc = 0;
    runtime->owned_call_frame.args = NULL;
    runtime->owned_trace_frame.argc = 0;
    runtime->owned_trace_frame.args = NULL;
}

static PTN_UNUSED void ptn_runtime_push_trace_frame(
    PtnRuntime *runtime,
    PtnTraceFrame *frame,
    const char *function_name,
    const char *file,
    size_t line,
    size_t argc,
    const PtnValue *args
) {
    frame->runtime = runtime;
    frame->function_name = function_name;
    frame->file = file;
    frame->line = line;
    frame->argc = argc;
    frame->args = args;
    frame->arg_names = runtime->next_call_arg_names;
    frame->parameter_count = 0;
    frame->parameter_names = NULL;
    frame->sensitive_parameter_count = 0;
    frame->sensitive_parameters = NULL;
    frame->sensitive_variadic_position = (size_t)-1;
    frame->has_receiver = 0;
    frame->receiver = ptn_null();
    frame->previous = runtime->trace_frame;
    runtime->trace_frame = frame;
}

static PTN_UNUSED int ptn_runtime_shutdown_line_without_active_file(PtnRuntime *runtime, size_t line) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    return line == 0 && root != NULL && root->shutdown_functions_running;
}

static PTN_UNUSED const char *ptn_runtime_internal_call_path(PtnRuntime *runtime, size_t line) {
    if (ptn_runtime_shutdown_line_without_active_file(runtime, line)) {
        return "[no active file]";
    }
    return runtime == NULL ? NULL : runtime->source_path;
}

static PTN_UNUSED const char *ptn_runtime_internal_trace_file(PtnRuntime *runtime, size_t line) {
    if (ptn_runtime_shutdown_line_without_active_file(runtime, line)) {
        return NULL;
    }
    return runtime == NULL ? NULL : runtime->source_path;
}

static PTN_UNUSED void ptn_runtime_pop_trace_frame(PtnRuntime *runtime, PtnTraceFrame *frame) {
    if (runtime->trace_frame == frame) {
        runtime->trace_frame = frame->previous;
    }
}

static void ptn_shutdown_function_destroy(PtnShutdownFunction *function) {
    if (function == NULL) {
        return;
    }
    ptn_value_destroy(&function->callback);
    for (size_t i = 0; i < function->argc; i++) {
        ptn_value_destroy(&function->args[i]);
    }
    free(function->args);
    function->args = NULL;
    function->argc = 0;
}

static void ptn_tick_function_destroy(PtnTickFunction *function) {
    if (function == NULL) {
        return;
    }
    ptn_value_destroy(&function->callback);
    for (size_t i = 0; i < function->argc; i++) {
        ptn_value_destroy(&function->args[i]);
    }
    free(function->args);
    function->args = NULL;
    function->argc = 0;
}

static PTN_UNUSED void ptn_runtime_register_shutdown_function(
    PtnRuntime *runtime,
    PtnValue callback,
    size_t argc,
    const PtnValue *args
) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (root->shutdown_functions_len == root->shutdown_functions_capacity) {
        size_t new_capacity = root->shutdown_functions_capacity == 0
            ? 4
            : root->shutdown_functions_capacity * 2;
        PtnShutdownFunction *new_functions = realloc(
            root->shutdown_functions,
            new_capacity * sizeof(PtnShutdownFunction)
        );
        if (new_functions == NULL) {
            ptn_abort_out_of_memory();
        }
        root->shutdown_functions = new_functions;
        root->shutdown_functions_capacity = new_capacity;
    }
    PtnShutdownFunction *function =
        &root->shutdown_functions[root->shutdown_functions_len++];
    function->callback = ptn_value_clone(callback);
    function->argc = argc;
    function->args = NULL;
    if (argc != 0) {
        function->args = malloc(argc * sizeof(PtnValue));
        if (function->args == NULL) {
            ptn_abort_out_of_memory();
        }
        for (size_t i = 0; i < argc; i++) {
            function->args[i] = ptn_value_clone_deref(args[i]);
        }
    }
}

static PTN_UNUSED void ptn_runtime_register_tick_function(
    PtnRuntime *runtime,
    PtnValue callback,
    size_t argc,
    const PtnValue *args
) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (root->tick_functions_len == root->tick_functions_capacity) {
        size_t new_capacity = root->tick_functions_capacity == 0
            ? 4
            : root->tick_functions_capacity * 2;
        if (new_capacity < root->tick_functions_capacity ||
            new_capacity > SIZE_MAX / sizeof(PtnTickFunction)) {
            ptn_abort_out_of_memory();
        }
        PtnTickFunction *new_functions = realloc(
            root->tick_functions,
            new_capacity * sizeof(PtnTickFunction)
        );
        if (new_functions == NULL) {
            ptn_abort_out_of_memory();
        }
        root->tick_functions = new_functions;
        root->tick_functions_capacity = new_capacity;
    }
    PtnTickFunction *function = &root->tick_functions[root->tick_functions_len++];
    function->callback = ptn_value_clone(callback);
    function->argc = argc;
    function->args = NULL;
    if (argc != 0) {
        function->args = malloc(argc * sizeof(PtnValue));
        if (function->args == NULL) {
            ptn_abort_out_of_memory();
        }
        for (size_t i = 0; i < argc; i++) {
            function->args[i] = ptn_value_clone_deref(args[i]);
        }
    }
}

static PTN_UNUSED void ptn_runtime_tick(PtnRuntime *runtime, size_t line) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (
        runtime == NULL ||
        !runtime->tick_enabled ||
        root->tick_functions_len == 0 ||
        root->tick_functions_running
    ) {
        return;
    }
    root->tick_functions_running = 1;
    size_t limit = root->tick_functions_len;
    for (size_t i = 0; i < limit && i < root->tick_functions_len; i++) {
        PtnTickFunction *function = &root->tick_functions[i];
        PtnValue callback = ptn_value_clone(function->callback);
        size_t argc = function->argc;
        PtnValue *call_args = NULL;
        if (argc != 0) {
            call_args = malloc(argc * sizeof(PtnValue));
            if (call_args == NULL) {
                ptn_value_destroy(&callback);
                ptn_abort_out_of_memory();
            }
            for (size_t j = 0; j < argc; j++) {
                call_args[j] = ptn_value_clone(function->args[j]);
            }
        }
        PtnValue result = ptn_null();
        PtnTryFrame callback_frame;
        PtnTraceFrame *saved_trace_frame = runtime->trace_frame;
        int saved_suppress_user_call_frame_location =
            runtime->suppress_user_call_frame_location;
        int saved_warn_by_ref_argument_mismatch =
            runtime->warn_by_ref_argument_mismatch;
        int saved_throw_argument_count_errors =
            runtime->throw_argument_count_errors;
        ptn_try_frame_push(runtime, &callback_frame);
        if (setjmp(callback_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &callback_frame);
            runtime->trace_frame = saved_trace_frame;
            runtime->suppress_user_call_frame_location =
                saved_suppress_user_call_frame_location;
            runtime->warn_by_ref_argument_mismatch =
                saved_warn_by_ref_argument_mismatch;
            runtime->throw_argument_count_errors =
                saved_throw_argument_count_errors;
            for (size_t j = 0; j < argc; j++) {
                ptn_value_destroy(&call_args[j]);
            }
            free(call_args);
            ptn_value_destroy(&callback);
            root->tick_functions_running = 0;
            ptn_rethrow_exception(runtime);
        }
        runtime->suppress_user_call_frame_location = 1;
        runtime->warn_by_ref_argument_mismatch = 1;
        runtime->throw_argument_count_errors = 1;
        result = ptn_call_callable(
            runtime,
            callback,
            argc,
            call_args,
            line,
            0
        );
        ptn_try_frame_pop(runtime, &callback_frame);
        runtime->trace_frame = saved_trace_frame;
        runtime->suppress_user_call_frame_location =
            saved_suppress_user_call_frame_location;
        runtime->warn_by_ref_argument_mismatch =
            saved_warn_by_ref_argument_mismatch;
        runtime->throw_argument_count_errors =
            saved_throw_argument_count_errors;
        for (size_t j = 0; j < argc; j++) {
            ptn_value_destroy(&call_args[j]);
        }
        free(call_args);
        ptn_value_destroy(&callback);
        ptn_value_destroy(&result);
        if (runtime->exceptions->active_exception != NULL) {
            root->tick_functions_running = 0;
            ptn_rethrow_exception(runtime);
        }
    }
    root->tick_functions_running = 0;
#else
    (void)runtime;
    (void)line;
#endif
}

static void ptn_runtime_run_shutdown_functions(PtnRuntime *runtime) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (
        root->shutdown_functions_completed ||
        root->shutdown_functions_running
    ) {
        return;
    }
    root->shutdown_functions_running = 1;
    while (root->shutdown_function_index < root->shutdown_functions_len) {
        PtnShutdownFunction *function =
            &root->shutdown_functions[root->shutdown_function_index++];
        PtnValue result = ptn_null();
        PtnTryFrame callback_frame;
        PtnTraceFrame *saved_trace_frame = runtime->trace_frame;
        int saved_suppress_user_call_frame_location =
            runtime->suppress_user_call_frame_location;
        int saved_warn_by_ref_argument_mismatch =
            runtime->warn_by_ref_argument_mismatch;
        int saved_throw_argument_count_errors =
            runtime->throw_argument_count_errors;
        ptn_try_frame_push(runtime, &callback_frame);
        if (setjmp(callback_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &callback_frame);
            runtime->trace_frame = saved_trace_frame;
            runtime->suppress_user_call_frame_location =
                saved_suppress_user_call_frame_location;
            runtime->warn_by_ref_argument_mismatch =
                saved_warn_by_ref_argument_mismatch;
            runtime->throw_argument_count_errors =
                saved_throw_argument_count_errors;
            root->shutdown_functions_running = 0;
            ptn_rethrow_exception(runtime);
        }
        runtime->suppress_user_call_frame_location = 1;
        runtime->warn_by_ref_argument_mismatch = 1;
        runtime->throw_argument_count_errors = 1;
        result = ptn_call_callable(
            runtime,
            function->callback,
            function->argc,
            function->args,
            0,
            0
        );
        ptn_try_frame_pop(runtime, &callback_frame);
        runtime->trace_frame = saved_trace_frame;
        runtime->suppress_user_call_frame_location =
            saved_suppress_user_call_frame_location;
        runtime->warn_by_ref_argument_mismatch =
            saved_warn_by_ref_argument_mismatch;
        runtime->throw_argument_count_errors =
            saved_throw_argument_count_errors;
        ptn_value_destroy(&result);
        if (runtime->exceptions->active_exception != NULL) {
            root->shutdown_functions_running = 0;
            ptn_rethrow_exception(runtime);
        }
    }
    root->shutdown_functions_running = 0;
    root->shutdown_functions_completed = 1;
#else
    (void)runtime;
#endif
}

static PTN_UNUSED void ptn_runtime_run_shutdown_functions_from(PtnRuntime *runtime, size_t start) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (root == NULL || root->shutdown_functions_running) {
        return;
    }
    if (start > root->shutdown_functions_len) {
        start = root->shutdown_functions_len;
    }

    size_t saved_index = root->shutdown_function_index;
    int saved_completed = root->shutdown_functions_completed;
    root->shutdown_function_index = start;
    root->shutdown_functions_completed = 0;
    root->shutdown_functions_running = 1;
    while (root->shutdown_function_index < root->shutdown_functions_len) {
        PtnShutdownFunction *function =
            &root->shutdown_functions[root->shutdown_function_index++];
        PtnValue result = ptn_null();
        PtnTryFrame callback_frame;
        PtnTraceFrame *saved_trace_frame = runtime->trace_frame;
        int saved_suppress_user_call_frame_location =
            runtime->suppress_user_call_frame_location;
        int saved_warn_by_ref_argument_mismatch =
            runtime->warn_by_ref_argument_mismatch;
        int saved_throw_argument_count_errors =
            runtime->throw_argument_count_errors;
        ptn_try_frame_push(runtime, &callback_frame);
        if (setjmp(callback_frame.jump) != 0) {
            ptn_try_frame_pop(runtime, &callback_frame);
            runtime->trace_frame = saved_trace_frame;
            runtime->suppress_user_call_frame_location =
                saved_suppress_user_call_frame_location;
            runtime->warn_by_ref_argument_mismatch =
                saved_warn_by_ref_argument_mismatch;
            runtime->throw_argument_count_errors =
                saved_throw_argument_count_errors;
            root->shutdown_functions_running = 0;
            root->shutdown_function_index = saved_index;
            root->shutdown_functions_completed = saved_completed;
            ptn_rethrow_exception(runtime);
        }
        runtime->suppress_user_call_frame_location = 1;
        runtime->warn_by_ref_argument_mismatch = 1;
        runtime->throw_argument_count_errors = 1;
        result = ptn_call_callable(
            runtime,
            function->callback,
            function->argc,
            function->args,
            0,
            0
        );
        ptn_try_frame_pop(runtime, &callback_frame);
        runtime->trace_frame = saved_trace_frame;
        runtime->suppress_user_call_frame_location =
            saved_suppress_user_call_frame_location;
        runtime->warn_by_ref_argument_mismatch =
            saved_warn_by_ref_argument_mismatch;
        runtime->throw_argument_count_errors =
            saved_throw_argument_count_errors;
        ptn_value_destroy(&result);
        if (runtime->exceptions->active_exception != NULL) {
            root->shutdown_functions_running = 0;
            root->shutdown_function_index = saved_index;
            root->shutdown_functions_completed = saved_completed;
            ptn_rethrow_exception(runtime);
        }
    }
    root->shutdown_functions_running = 0;
    for (size_t i = start; i < root->shutdown_functions_len; i++) {
        ptn_shutdown_function_destroy(&root->shutdown_functions[i]);
    }
    root->shutdown_functions_len = start;
    root->shutdown_function_index = saved_index > start ? start : saved_index;
    root->shutdown_functions_completed = saved_completed;
#else
    (void)runtime;
    (void)start;
#endif
}

static PTN_UNUSED void ptn_runtime_register_header_callback(PtnRuntime *runtime, PtnValue callback) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (root == NULL) {
        return;
    }
    if (root->header_callback_registered) {
        ptn_value_destroy(&root->header_callback);
    }
    root->header_callback = ptn_value_clone(callback);
    root->header_callback_registered = 1;
    root->header_callback_completed = root->output_has_started ? 1 : 0;
}

static PTN_UNUSED void ptn_runtime_run_header_callback(PtnRuntime *runtime) {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        root = runtime;
    }
    if (
        root == NULL ||
        !root->header_callback_registered ||
        root->header_callback_completed ||
        root->header_callback_running
    ) {
        return;
    }

    PtnValue result = ptn_null();
    PtnTryFrame callback_frame;
    PtnTraceFrame *saved_trace_frame = runtime->trace_frame;
    int saved_suppress_user_call_frame_location =
        runtime->suppress_user_call_frame_location;
    int saved_warn_by_ref_argument_mismatch =
        runtime->warn_by_ref_argument_mismatch;
    int saved_throw_argument_count_errors =
        runtime->throw_argument_count_errors;
    int saved_passthrough_output =
        root->output_buffer_callback_passthrough_output;
    size_t saved_skip_buffers = root->output_buffer_callback_skip_buffers;

    root->header_callback_running = 1;
    root->header_callback_completed = 1;
    root->output_buffer_callback_passthrough_output = 1;
    root->output_buffer_callback_skip_buffers = root->output_buffers_len;
    ptn_try_frame_push(runtime, &callback_frame);
    if (setjmp(callback_frame.jump) != 0) {
        ptn_try_frame_pop(runtime, &callback_frame);
        runtime->trace_frame = saved_trace_frame;
        runtime->suppress_user_call_frame_location =
            saved_suppress_user_call_frame_location;
        runtime->warn_by_ref_argument_mismatch =
            saved_warn_by_ref_argument_mismatch;
        runtime->throw_argument_count_errors =
            saved_throw_argument_count_errors;
        root->output_buffer_callback_passthrough_output =
            saved_passthrough_output;
        root->output_buffer_callback_skip_buffers = saved_skip_buffers;
        root->header_callback_running = 0;
        ptn_rethrow_exception(runtime);
    }

    runtime->suppress_user_call_frame_location = 1;
    runtime->warn_by_ref_argument_mismatch = 1;
    runtime->throw_argument_count_errors = 1;
    result = ptn_call_callable(runtime, root->header_callback, 0, NULL, 0, 0);
    ptn_try_frame_pop(runtime, &callback_frame);
    runtime->trace_frame = saved_trace_frame;
    runtime->suppress_user_call_frame_location =
        saved_suppress_user_call_frame_location;
    runtime->warn_by_ref_argument_mismatch =
        saved_warn_by_ref_argument_mismatch;
    runtime->throw_argument_count_errors =
        saved_throw_argument_count_errors;
    root->output_buffer_callback_passthrough_output =
        saved_passthrough_output;
    root->output_buffer_callback_skip_buffers = saved_skip_buffers;
    root->header_callback_running = 0;
    ptn_value_destroy(&result);
    if (runtime->exceptions->active_exception != NULL) {
        ptn_rethrow_exception(runtime);
    }
#else
    (void)runtime;
#endif
}

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static void ptn_runtime_session_shutdown(PtnRuntime *runtime);
#else
static void ptn_runtime_session_shutdown(PtnRuntime *runtime) {
    (void)runtime;
}
#endif

static void ptn_runtime_free(PtnRuntime *runtime) {
    if (runtime->lifecycle_root == runtime) {
        if (runtime->shutdown_in_progress) {
            return;
        }
        runtime->shutdown_in_progress = 1;
    }
    free(runtime->dynamic_property_deprecation_suppress_property);
    runtime->dynamic_property_deprecation_suppress_property = NULL;
    runtime->dynamic_property_deprecation_suppress_object = NULL;
    if (runtime->lifecycle_root == runtime) {
        int session_shutdown_early =
            !runtime->session_active ||
            runtime->session_save_handler_kind == 0 ||
            runtime->session_save_handler_register_shutdown;
        if (session_shutdown_early) {
            ptn_runtime_session_shutdown(runtime);
        }
        ptn_runtime_run_header_callback(runtime);
        ptn_runtime_run_shutdown_functions(runtime);
        ptn_runtime_run_static_property_destructors(runtime);
        ptn_runtime_run_static_local_destructors(runtime);
        ptn_runtime_run_symbol_value_destructors(&runtime->symbols);
        ptn_runtime_run_object_destructors_until_output_buffer(runtime);
        ptn_runtime_release_static_locals(runtime);
        ptn_output_buffer_flush_all(runtime);
        ptn_runtime_run_object_destructors(runtime);
        if (!session_shutdown_early) {
            ptn_runtime_session_shutdown(runtime);
        }
        ptn_runtime_run_static_property_destructors(runtime);
        ptn_standard_streams_shutdown();
        ptn_diagnostics_clear_error_handler(&runtime->diagnostics);
        ptn_exception_handlers_clear(&runtime->owned_exceptions);
    }
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_property_type_allows_null, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_property_type_text, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_property_type_class_name, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_property_type_kind, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_property_set_visibility, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_property_read_visibility, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_property_initialized, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_static_properties, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_class_constant_initializing, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_class_constant_deprecations, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_class_constants, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_dynamic_classes, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_class_aliases, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_constant_sources, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->owned_constants, runtime);
    ptn_symbols_free_with_runtime_scope(&runtime->symbols, runtime);
    ptn_value_destroy_with_runtime_scope(runtime, &runtime->deferred_yield_from_iterator_object);
    if (runtime->lifecycle_root == runtime && runtime->last_opened_directory != NULL) {
        ptn_resource_release(runtime->last_opened_directory);
        runtime->last_opened_directory = NULL;
    }
    for (size_t i = 0; i < runtime->magic_property_frame_len; i++) {
        free(runtime->magic_property_frames[i].property);
    }
    free(runtime->magic_property_frames);
    runtime->magic_property_frames = NULL;
    runtime->magic_property_frame_len = 0;
    runtime->magic_property_frame_capacity = 0;
    if (runtime->lifecycle_root == runtime) {
        free(runtime->include_path);
        runtime->include_path = NULL;
        for (size_t i = 0; i < runtime->included_files_len; i++) {
            free(runtime->included_files[i]);
        }
        free(runtime->included_files);
        runtime->included_files = NULL;
        runtime->included_files_len = 0;
        runtime->included_files_capacity = 0;
        for (size_t i = 0; i < runtime->autoload_callbacks_len; i++) {
            ptn_value_destroy(&runtime->autoload_callbacks[i]);
            free(runtime->autoload_callback_scope_class_names[i]);
            free(runtime->autoload_callback_called_class_names[i]);
        }
        free(runtime->autoload_callbacks);
        free(runtime->autoload_callback_scope_class_names);
        free(runtime->autoload_callback_called_class_names);
        runtime->autoload_callbacks = NULL;
        runtime->autoload_callback_scope_class_names = NULL;
        runtime->autoload_callback_called_class_names = NULL;
        runtime->autoload_callbacks_len = 0;
        runtime->autoload_callbacks_capacity = 0;
        free(runtime->spl_autoload_extensions);
        runtime->spl_autoload_extensions = NULL;
        for (size_t i = 0; i < runtime->autoloading_class_names_len; i++) {
            free(runtime->autoloading_class_names[i]);
        }
        free(runtime->autoloading_class_names);
        runtime->autoloading_class_names = NULL;
        runtime->autoloading_class_names_len = 0;
        runtime->autoloading_class_names_capacity = 0;
        free(runtime->open_basedir);
        runtime->open_basedir = NULL;
        free(runtime->memory_limit);
        runtime->memory_limit = NULL;
        free(runtime->max_memory_limit);
        runtime->max_memory_limit = NULL;
        free(runtime->auto_detect_line_endings);
        runtime->auto_detect_line_endings = NULL;
        free(runtime->default_charset);
        runtime->default_charset = NULL;
        free(runtime->arg_separator_input);
        runtime->arg_separator_input = NULL;
        free(runtime->arg_separator_output);
        runtime->arg_separator_output = NULL;
        free(runtime->output_handler);
        runtime->output_handler = NULL;
        free(runtime->filter_default);
        runtime->filter_default = NULL;
        free(runtime->pcre_backtrack_limit);
        runtime->pcre_backtrack_limit = NULL;
        free(runtime->pcre_recursion_limit);
        runtime->pcre_recursion_limit = NULL;
        free(runtime->pcre_jit);
        runtime->pcre_jit = NULL;
        free(runtime->opcache_blacklist_filename);
        runtime->opcache_blacklist_filename = NULL;
        free(runtime->opcache_enable);
        runtime->opcache_enable = NULL;
        free(runtime->opcache_enable_cli);
        runtime->opcache_enable_cli = NULL;
        free(runtime->opcache_fast_shutdown);
        runtime->opcache_fast_shutdown = NULL;
        free(runtime->opcache_file_cache_only);
        runtime->opcache_file_cache_only = NULL;
        free(runtime->opcache_file_update_protection);
        runtime->opcache_file_update_protection = NULL;
        free(runtime->opcache_interned_strings_buffer);
        runtime->opcache_interned_strings_buffer = NULL;
        free(runtime->opcache_log_verbosity_level);
        runtime->opcache_log_verbosity_level = NULL;
        free(runtime->opcache_optimization_level);
        runtime->opcache_optimization_level = NULL;
        free(runtime->opcache_opt_debug_level);
        runtime->opcache_opt_debug_level = NULL;
        free(runtime->opcache_preload);
        runtime->opcache_preload = NULL;
        free(runtime->opcache_preload_user);
        runtime->opcache_preload_user = NULL;
        free(runtime->opcache_save_comments);
        runtime->opcache_save_comments = NULL;
        free(runtime->opcache_validate_timestamps);
        runtime->opcache_validate_timestamps = NULL;
        free(runtime->phar_readonly);
        runtime->phar_readonly = NULL;
        free(runtime->phar_require_hash);
        runtime->phar_require_hash = NULL;
        free(runtime->phar_cache_list);
        runtime->phar_cache_list = NULL;
        free(runtime->internal_encoding);
        runtime->internal_encoding = NULL;
        free(runtime->input_encoding);
        runtime->input_encoding = NULL;
        free(runtime->output_encoding);
        runtime->output_encoding = NULL;
        free(runtime->iconv_internal_encoding);
        runtime->iconv_internal_encoding = NULL;
        free(runtime->iconv_input_encoding);
        runtime->iconv_input_encoding = NULL;
        free(runtime->iconv_output_encoding);
        runtime->iconv_output_encoding = NULL;
        free(runtime->variables_order);
        runtime->variables_order = NULL;
        free(runtime->register_argc_argv);
        runtime->register_argc_argv = NULL;
        free(runtime->enable_post_data_reading);
        runtime->enable_post_data_reading = NULL;
        free(runtime->file_uploads);
        runtime->file_uploads = NULL;
        free(runtime->max_input_vars);
        runtime->max_input_vars = NULL;
        free(runtime->max_input_nesting_level);
        runtime->max_input_nesting_level = NULL;
        free(runtime->post_max_size);
        runtime->post_max_size = NULL;
        free(runtime->always_populate_raw_post_data);
        runtime->always_populate_raw_post_data = NULL;
        free(runtime->upload_tmp_dir);
        runtime->upload_tmp_dir = NULL;
        free(runtime->expose_php);
        runtime->expose_php = NULL;
        free(runtime->docref_root);
        runtime->docref_root = NULL;
        free(runtime->user_agent);
        runtime->user_agent = NULL;
        free(runtime->unserialize_callback_func);
        runtime->unserialize_callback_func = NULL;
        free(runtime->assert_callback_ini);
        runtime->assert_callback_ini = NULL;
        ptn_value_destroy(&runtime->assert_callback);
        runtime->assert_callback = ptn_null();
        free(runtime->disabled_functions);
        runtime->disabled_functions = NULL;
        free(runtime->request_body);
        runtime->request_body = NULL;
        runtime->request_body_len = 0;
        ptn_symbols_free(&runtime->session_ini);
        free(runtime->session_id);
        runtime->session_id = NULL;
        runtime->session_active = 0;
        runtime->session_was_started = 0;
        runtime->session_auto_started = 0;
        runtime->session_start_path = NULL;
        runtime->session_start_line = 0;
        ptn_value_destroy(&runtime->session_save_handler_object);
        runtime->session_save_handler_object = ptn_null();
        for (size_t i = 0; i < sizeof(runtime->session_save_handler_callbacks) / sizeof(runtime->session_save_handler_callbacks[0]); i++) {
            ptn_value_destroy(&runtime->session_save_handler_callbacks[i]);
            runtime->session_save_handler_callbacks[i] = ptn_null();
        }
        runtime->session_save_handler_kind = 0;
        runtime->session_save_handler_register_shutdown = 1;
        runtime->session_save_handler_in_callback = 0;
        runtime->session_save_handler_shutdown_warning_pending = 0;
        runtime->session_parent_handler_open = 0;
        free(runtime->session_parent_save_handler);
        runtime->session_parent_save_handler = NULL;
        runtime->session_lazy_write = 1;
        free(runtime->session_last_data);
        runtime->session_last_data = NULL;
        runtime->session_last_data_len = 0;
        runtime->session_last_data_valid = 0;
        free(runtime->live_objects);
        runtime->live_objects = NULL;
        runtime->live_objects_len = 0;
        runtime->live_objects_capacity = 0;
        free(runtime->live_closures);
        runtime->live_closures = NULL;
        runtime->live_closures_len = 0;
        runtime->live_closures_capacity = 0;
        for (size_t i = 0; i < runtime->first_class_callable_cache_len; i++) {
            free(runtime->first_class_callable_cache_names[i]);
            ptn_value_destroy(&runtime->first_class_callable_cache_values[i]);
        }
        free(runtime->first_class_callable_cache_names);
        free(runtime->first_class_callable_cache_values);
        runtime->first_class_callable_cache_names = NULL;
        runtime->first_class_callable_cache_values = NULL;
        runtime->first_class_callable_cache_len = 0;
        runtime->first_class_callable_cache_capacity = 0;
        free(runtime->live_arrays);
        runtime->live_arrays = NULL;
        runtime->live_arrays_len = 0;
        runtime->live_arrays_capacity = 0;
        free(runtime->live_references);
        runtime->live_references = NULL;
        runtime->live_references_len = 0;
        runtime->live_references_capacity = 0;
        ptn_runtime_clear_temporary_roots(runtime);
        free(runtime->temporary_roots);
        runtime->temporary_roots = NULL;
        runtime->temporary_roots_len = 0;
        runtime->temporary_roots_capacity = 0;
        free(runtime->static_local_slots);
        runtime->static_local_slots = NULL;
        runtime->static_local_slots_len = 0;
        runtime->static_local_slots_capacity = 0;
        free(runtime->free_object_ids);
        runtime->free_object_ids = NULL;
        runtime->free_object_ids_len = 0;
        runtime->free_object_ids_capacity = 0;
        runtime->deferred_free_object_id = 0;
        runtime->has_deferred_free_object_id = 0;
        free(runtime->output_buffers);
        runtime->output_buffers = NULL;
        runtime->output_buffers_len = 0;
        runtime->output_buffers_capacity = 0;
        runtime->output_buffer_callback_depth = 0;
        runtime->output_buffer_callback_function_name = NULL;
        free(runtime->output_buffer_callback_handler_name);
        runtime->output_buffer_callback_handler_name = NULL;
        runtime->output_buffer_callback_line = 0;
        runtime->output_buffer_callback_output_warned = 0;
        runtime->output_buffer_callback_passthrough_output = 0;
        runtime->output_buffer_callback_skip_buffers = 0;
        runtime->output_at_line_start = 1;
        runtime->output_has_started = 0;
        if (runtime->header_callback_registered) {
            ptn_value_destroy(&runtime->header_callback);
        }
        runtime->header_callback = ptn_null();
        runtime->header_callback_registered = 0;
        runtime->header_callback_running = 0;
        runtime->header_callback_completed = 0;
        for (size_t i = 0; i < runtime->shutdown_functions_len; i++) {
            ptn_shutdown_function_destroy(&runtime->shutdown_functions[i]);
        }
        free(runtime->shutdown_functions);
        runtime->shutdown_functions = NULL;
        runtime->shutdown_functions_len = 0;
        runtime->shutdown_functions_capacity = 0;
        runtime->shutdown_function_index = 0;
        runtime->shutdown_functions_running = 0;
        runtime->shutdown_functions_completed = 0;
        for (size_t i = 0; i < runtime->tick_functions_len; i++) {
            ptn_tick_function_destroy(&runtime->tick_functions[i]);
        }
        free(runtime->tick_functions);
        runtime->tick_functions = NULL;
        runtime->tick_functions_len = 0;
        runtime->tick_functions_capacity = 0;
        runtime->tick_functions_running = 0;
        free(runtime->strtok_string);
        runtime->strtok_string = NULL;
        runtime->strtok_len = 0;
        runtime->strtok_offset = 0;
        runtime->strtok_has_state = 0;
        free(runtime->intl_last_error_message);
        runtime->intl_last_error_message = NULL;
    }
}

static PTN_UNUSED PtnLookupResult ptn_object_property_lookup_quiet(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    const char *access_scope,
    size_t line
);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED void ptn_string_buffer_init(PtnStringBuffer *buffer);
static PTN_UNUSED void ptn_string_buffer_append(PtnStringBuffer *buffer, const char *value);
static PTN_UNUSED void ptn_string_buffer_append_len(
    PtnStringBuffer *buffer,
    const char *value,
    size_t len
);
static PTN_UNUSED void ptn_string_buffer_append_char(PtnStringBuffer *buffer, char value);
static PTN_UNUSED void ptn_string_buffer_append_format(
    PtnStringBuffer *buffer,
    const char *format,
    ...
);

static PTN_UNUSED PtnSymbolTable *ptn_runtime_global_symbol_table(PtnRuntime *runtime) {
    return runtime->global_symbols == NULL ? &runtime->symbols : runtime->global_symbols;
}

static PTN_UNUSED int ptn_runtime_is_auto_global_symbol_name(const char *name) {
    return strcmp(name, "_SERVER") == 0 ||
        strcmp(name, "_GET") == 0 ||
        strcmp(name, "_POST") == 0 ||
        strcmp(name, "_FILES") == 0 ||
        strcmp(name, "_COOKIE") == 0 ||
        strcmp(name, "_SESSION") == 0 ||
        strcmp(name, "_REQUEST") == 0 ||
        strcmp(name, "_ENV") == 0;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_variable_symbol_table(PtnRuntime *runtime, const char *name) {
    return ptn_runtime_is_auto_global_symbol_name(name)
        ? ptn_runtime_global_symbol_table(runtime)
        : &runtime->symbols;
}

static PTN_UNUSED PtnValue ptn_runtime_globals_snapshot(PtnRuntime *runtime) {
    PtnSymbolTable *globals = ptn_runtime_global_symbol_table(runtime);
    PtnArrayLiteralEntry *entries = NULL;
    if (globals->len != 0) {
        entries = malloc(globals->len * sizeof(PtnArrayLiteralEntry));
        if (entries == NULL) {
            ptn_abort_out_of_memory();
        }
    }

    size_t entry_count = 0;
    for (size_t i = 0; i < globals->len; i++) {
        if (strcmp(globals->items[i].name, "GLOBALS") == 0) {
            continue;
        }
        entries[entry_count].has_key = 1;
        entries[entry_count].key = ptn_string(globals->items[i].name);
        entries[entry_count].value = globals->items[i].value;
        entry_count++;
    }

    PtnValue snapshot = ptn_array_from_literal_entries(entry_count, entries);
    free(entries);
    return snapshot;
}

static PTN_UNUSED PtnValue ptn_runtime_write_variable_result_at(PtnRuntime *runtime, const char *name, PtnValue value, size_t line) {
    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    PtnValue current;
    if (ptn_symbols_get(symbols, name, &current) && current.type == PTN_REFERENCE) {
        PtnValue result = ptn_null();
        if (ptn_reference_assign_result_with_context_at(
            runtime,
            current.as.reference,
            value,
            1,
            line,
            &result
        )) {
            return result;
        }
        return ptn_value_clone(current.as.reference->value);
    }
    PtnValue result = ptn_value_clone_deref(value);
    ptn_symbols_set_with_runtime_scope_at(symbols, name, result, runtime, line);
    return result;
}

static PTN_UNUSED PtnValue ptn_runtime_write_variable_result(PtnRuntime *runtime, const char *name, PtnValue value) {
    return ptn_runtime_write_variable_result_at(runtime, name, value, 0);
}

static PTN_UNUSED void ptn_runtime_write_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnValue result = ptn_runtime_write_variable_result(runtime, name, value);
    ptn_value_destroy(&result);
}

static PTN_UNUSED PtnValue ptn_runtime_write_global_variable_result(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnSymbolTable *globals = ptn_runtime_global_symbol_table(runtime);
    PtnValue current;
    if (ptn_symbols_get(globals, name, &current) && current.type == PTN_REFERENCE) {
        if (ptn_reference_assign(runtime, current.as.reference, value)) {
            return ptn_value_clone(current.as.reference->value);
        }
        return ptn_value_clone(current.as.reference->value);
    }
    PtnValue result = ptn_value_clone_deref(value);
    ptn_symbols_set_with_runtime_scope(globals, name, result, runtime);
    return result;
}

static PTN_UNUSED void ptn_runtime_write_global_variable(PtnRuntime *runtime, const char *name, PtnValue value) {
    PtnValue result = ptn_runtime_write_global_variable_result(runtime, name, value);
    ptn_value_destroy(&result);
}

static PTN_UNUSED void ptn_runtime_bind_variable_reference(PtnRuntime *runtime, const char *name, PtnValue reference) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    ptn_gc_attach_value_runtime(runtime, reference, 0);
    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    PtnSymbol *symbol = ptn_symbols_slot_for_write(symbols, name);
    PtnValue old_value = symbol->value;
    ptn_array_note_value_replacement(old_value, reference);
    symbol->value = ptn_value_clone(reference);
    symbols->mutation_epoch++;
    ptn_runtime_unwrap_reference_slots_if_unaliased(runtime, old_value, 2);
    ptn_value_destroy(&old_value);
}

static PTN_UNUSED void ptn_runtime_bind_global_variable(PtnRuntime *runtime, const char *name) {
    PtnValue reference = ptn_symbols_reference_for_variable(ptn_runtime_global_symbol_table(runtime), name);
    ptn_gc_attach_value_runtime(runtime, reference, 0);
    ptn_symbols_bind_reference(&runtime->symbols, name, reference);
    ptn_value_destroy(&reference);
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_variable(PtnRuntime *runtime, const char *name) {
    PtnValue reference = ptn_symbols_reference_for_variable(ptn_runtime_variable_symbol_table(runtime, name), name);
    ptn_gc_attach_value_runtime(runtime, reference, 0);
    return reference;
}

static PTN_UNUSED PtnValue *ptn_runtime_global_variable_slot(PtnRuntime *runtime, const char *name) {
    return ptn_symbols_value_slot(ptn_runtime_global_symbol_table(runtime), name);
}

static PTN_UNUSED PtnValue *ptn_runtime_global_variable_slot_for_write(PtnRuntime *runtime, const char *name) {
    return &ptn_symbols_slot_for_write(ptn_runtime_global_symbol_table(runtime), name)->value;
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_global_variable_quiet(PtnRuntime *runtime, const char *name) {
    PtnValue value;
    if (ptn_symbols_get(ptn_runtime_global_symbol_table(runtime), name, &value)) {
        return ptn_lookup_found(ptn_value_deref(value));
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED void ptn_runtime_unset_global_variable(PtnRuntime *runtime, const char *name) {
    ptn_symbols_unset_with_runtime_scope(ptn_runtime_global_symbol_table(runtime), name, runtime);
}

static PTN_UNUSED void ptn_abort_by_reference_argument_error(
    const char *function_name,
    size_t position,
    const char *parameter_name
) {
    const int has_parameter_name = parameter_name != NULL && parameter_name[0] != '\0';
    fprintf(
        stderr,
        has_parameter_name
            ? "Fatal error: %s(): Argument #%zu ($%s) cannot be passed by reference\n"
            : "Fatal error: %s(): Argument #%zu cannot be passed by reference\n",
        function_name,
        position,
        has_parameter_name ? parameter_name : ""
    );
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception_at(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
);

static PTN_UNUSED void ptn_throw_by_reference_argument_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    size_t line
) {
    char message[256];
    const int has_parameter_name = parameter_name != NULL && parameter_name[0] != '\0';
    int written;
    if (has_parameter_name) {
        written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu ($%s) could not be passed by reference",
            function_name,
            position,
            parameter_name
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "%s(): Argument #%zu could not be passed by reference",
            function_name,
            position
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED PtnValue ptn_runtime_by_reference_argument_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    size_t line
) {
    ptn_throw_by_reference_argument_error(runtime, function_name, position, parameter_name, line);
    return ptn_null();
}

static PTN_UNUSED const char *ptn_by_reference_argument_function_name(
    PtnRuntime *runtime,
    const char *fallback
) {
    if (runtime != NULL && runtime->by_ref_argument_function_name_override != NULL) {
        return runtime->by_ref_argument_function_name_override;
    }
    return fallback;
}

static PTN_UNUSED void ptn_emit_by_reference_argument_warning(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name,
    size_t line
) {
    if (
        runtime != NULL &&
        runtime->by_ref_argument_notice_pending &&
        !runtime->by_ref_argument_notice_emitted
    ) {
        size_t notice_line = runtime->by_ref_argument_notice_line == 0
            ? line
            : runtime->by_ref_argument_notice_line;
        ptn_emit_only_variables_passed_by_reference_notice_at(runtime, notice_line);
        runtime->by_ref_argument_notice_emitted = 1;
    }
    const int has_parameter_name = parameter_name != NULL && parameter_name[0] != '\0';
    int needed = snprintf(
        NULL,
        0,
        has_parameter_name
            ? "%s(): Argument #%zu ($%s) must be passed by reference, value given"
            : "%s(): Argument #%zu must be passed by reference, value given",
        function_name,
        position,
        has_parameter_name ? parameter_name : ""
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
        has_parameter_name
            ? "%s(): Argument #%zu ($%s) must be passed by reference, value given"
            : "%s(): Argument #%zu must be passed by reference, value given",
        function_name,
        position,
        has_parameter_name ? parameter_name : ""
    );
    ptn_emit_warning_with_handler_frame(
        &runtime->diagnostics,
        message,
        line,
        runtime != NULL &&
            (runtime->suppress_user_call_frame_location || runtime->warn_by_ref_argument_mismatch)
    );
    free(message);
}

static PTN_UNUSED void ptn_abort_by_reference_return_error(void) {
    fputs("Fatal error: by-reference return did not produce a reference\n", stderr);
    exit(255);
}

static PTN_UNUSED PtnValue ptn_reference_source_or_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    if (value.type == PTN_REFERENCE) {
        return ptn_value_clone(value);
    }
    if (!ptn_value_is_return_reference_fallback(value)) {
        ptn_emit_only_variable_references_returned_by_reference_notice_at(runtime, line);
    }
    return ptn_reference_value(ptn_reference_new_owned(ptn_value_clone(value)));
}

static PTN_UNUSED PtnValue ptn_return_reference_source_or_value(PtnRuntime *runtime, PtnValue value, size_t line) {
    if (value.type == PTN_REFERENCE) {
        return ptn_value_clone(value);
    }
    if (!ptn_value_is_return_reference_fallback(value)) {
        ptn_emit_only_variable_references_returned_by_reference_notice_at(runtime, line);
    }
    return ptn_value_mark_return_reference_fallback(
        ptn_value_clone_deref_preserve_return_reference_fallback(value)
    );
}

static PTN_UNUSED PtnValue ptn_return_reference_source_or_plain_value(PtnValue value) {
    if (value.type == PTN_REFERENCE) {
        return ptn_value_clone(value);
    }
    return ptn_value_clone_deref_preserve_return_reference_fallback(value);
}

static PTN_UNUSED PtnValue ptn_call_result_for_value_context(PtnValue value) {
    if (value.type != PTN_REFERENCE) {
        return value;
    }
    PtnValue result = ptn_value_clone_deref(value);
    ptn_value_destroy(&value);
    return result;
}

static PTN_UNUSED PtnValue ptn_by_ref_argument_source_or_temporary(PtnRuntime *runtime, PtnValue value, size_t line) {
    if (ptn_value_is_by_ref_argument_source(value)) {
        return ptn_value_clone(value);
    }
    if (!ptn_value_is_return_reference_fallback(value)) {
        ptn_emit_only_variables_passed_by_reference_notice_at(runtime, line);
    }
    return ptn_reference_value(ptn_reference_new_owned(ptn_value_deep_clone(ptn_value_deref(value))));
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    if (strcmp(name, "GLOBALS") == 0) {
        return ptn_runtime_globals_snapshot(runtime);
    }
    PtnValue value;
    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    if (ptn_symbols_get(symbols, name, &value)) {
        return ptn_value_deref(value);
    }
    if (strcmp(name, "this") == 0) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Using $this when not in object context",
            path,
            line
        );
        return ptn_null();
    }
    if (ptn_runtime_is_auto_global_symbol_name(name)) {
        ptn_emit_undefined_global_variable_warning(&runtime->diagnostics, name, path, line);
        return ptn_null();
    }
    ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable_for_increment(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    if (strcmp(name, "GLOBALS") == 0) {
        return ptn_runtime_globals_snapshot(runtime);
    }
    PtnValue value;
    PtnSymbolTable *symbols = ptn_runtime_variable_symbol_table(runtime, name);
    if (ptn_symbols_get(symbols, name, &value)) {
        return ptn_value_clone(value);
    }
    if (strcmp(name, "this") == 0) {
        ptn_throw_exception_at(
            runtime,
            "Error",
            "Using $this when not in object context",
            path,
            line
        );
        return ptn_null();
    }
    if (ptn_runtime_is_auto_global_symbol_name(name)) {
        ptn_emit_undefined_global_variable_warning(&runtime->diagnostics, name, path, line);
        return ptn_null();
    }
    ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_read_variable_for_array_mutation(
    PtnRuntime *runtime,
    const char *name,
    const char *path,
    size_t line
) {
    PtnValue *slot = ptn_symbols_get_slot(ptn_runtime_variable_symbol_table(runtime, name), name);
    if (slot == NULL) {
        if (strcmp(name, "this") == 0) {
            ptn_throw_exception_at(
                runtime,
                "Error",
                "Using $this when not in object context",
                path,
                line
            );
            return ptn_null();
        }
        if (ptn_runtime_is_auto_global_symbol_name(name)) {
            ptn_emit_undefined_global_variable_warning(&runtime->diagnostics, name, path, line);
            return ptn_null();
        }
        ptn_emit_undefined_variable_warning(&runtime->diagnostics, name, path, line);
        return ptn_null();
    }
    if (slot->type == PTN_ARRAY) {
        (void)ptn_value_detach_array(slot);
    }
    return ptn_value_borrow(*slot);
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_variable_quiet(PtnRuntime *runtime, const char *name) {
    if (strcmp(name, "GLOBALS") == 0) {
        return ptn_lookup_found(ptn_runtime_globals_snapshot(runtime));
    }
    PtnValue value;
    if (ptn_symbols_get(ptn_runtime_variable_symbol_table(runtime, name), name, &value)) {
        return ptn_lookup_found(ptn_value_deref(value));
    }
    return ptn_lookup_missing();
}

static PTN_UNUSED int ptn_runtime_variable_is_set(PtnRuntime *runtime, const char *name) {
    if (strcmp(name, "GLOBALS") == 0) {
        return 1;
    }
    PtnValue value;
    return ptn_symbols_get(ptn_runtime_variable_symbol_table(runtime, name), name, &value) && ptn_value_deref(value).type != PTN_NULL;
}

static PTN_UNUSED int ptn_runtime_variable_is_empty(PtnRuntime *runtime, const char *name) {
    if (strcmp(name, "GLOBALS") == 0) {
        PtnValue globals = ptn_runtime_globals_snapshot(runtime);
        int result = !ptn_is_truthy(ptn_value_deref(globals));
        ptn_value_destroy(&globals);
        return result;
    }
    PtnValue value;
    return !ptn_symbols_get(ptn_runtime_variable_symbol_table(runtime, name), name, &value) || !ptn_is_truthy(ptn_value_deref(value));
}

static PTN_UNUSED int ptn_call_frame_has_parameter(PtnCallFrame *frame, const char *name) {
    if (frame == NULL || frame->parameter_names == NULL) {
        return 0;
    }
    for (size_t i = 0; i < frame->parameter_count; i++) {
        if (strcmp(frame->parameter_names[i], name) == 0) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_runtime_unset_variable(PtnRuntime *runtime, const char *name) {
    if (ptn_runtime_is_auto_global_symbol_name(name)) {
        ptn_runtime_unset_global_variable(runtime, name);
        return;
    }
    if (ptn_call_frame_has_parameter(runtime->call_frame, name)) {
        ptn_symbols_set_with_runtime_scope(&runtime->symbols, name, ptn_null(), runtime);
        return;
    }
    ptn_symbols_unset_with_runtime_scope(&runtime->symbols, name, runtime);
}

typedef struct {
    PtnArray **arrays;
    size_t len;
    size_t capacity;
} PtnTraceSnapshotSeen;

static PTN_UNUSED void ptn_trace_snapshot_seen_init(PtnTraceSnapshotSeen *seen) {
    seen->arrays = NULL;
    seen->len = 0;
    seen->capacity = 0;
}

static PTN_UNUSED void ptn_trace_snapshot_seen_free(PtnTraceSnapshotSeen *seen) {
    free(seen->arrays);
    seen->arrays = NULL;
    seen->len = 0;
    seen->capacity = 0;
}

static PTN_UNUSED int ptn_trace_snapshot_seen_contains(PtnTraceSnapshotSeen *seen, PtnArray *array) {
    for (size_t i = 0; i < seen->len; i++) {
        if (seen->arrays[i] == array) {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED void ptn_trace_snapshot_seen_push(PtnTraceSnapshotSeen *seen, PtnArray *array) {
    if (seen->len == seen->capacity) {
        size_t new_capacity = seen->capacity == 0 ? 8 : seen->capacity * 2;
        if (new_capacity < seen->capacity || new_capacity > SIZE_MAX / sizeof(PtnArray *)) {
            ptn_abort_out_of_memory();
        }
        PtnArray **new_arrays = realloc(seen->arrays, new_capacity * sizeof(PtnArray *));
        if (new_arrays == NULL) {
            ptn_abort_out_of_memory();
        }
        seen->arrays = new_arrays;
        seen->capacity = new_capacity;
    }
    seen->arrays[seen->len++] = array;
}

static PTN_UNUSED void ptn_trace_snapshot_seen_pop(PtnTraceSnapshotSeen *seen) {
    if (seen->len > 0) {
        seen->len--;
    }
}

static PTN_UNUSED PtnValue ptn_trace_value_snapshot_depth(
    PtnValue value,
    size_t depth,
    PtnTraceSnapshotSeen *seen
) {
    value = ptn_value_deref(value);
    if (value.type != PTN_ARRAY || depth > 64) {
        return ptn_value_deep_clone(value);
    }
    if (ptn_trace_snapshot_seen_contains(seen, value.as.array)) {
        return ptn_array_from_literal_entries(0, NULL);
    }

    PtnValue snapshot = ptn_array_from_literal_entries(0, NULL);
    ptn_trace_snapshot_seen_push(seen, value.as.array);
    for (size_t i = 0; i < value.as.array->len; i++) {
        ptn_array_set_entry(
            snapshot.as.array,
            ptn_array_key_clone(value.as.array->entries[i].key),
            ptn_trace_value_snapshot_depth(value.as.array->entries[i].value, depth + 1, seen)
        );
    }
    ptn_trace_snapshot_seen_pop(seen);
    snapshot.as.array->next_auto_key = value.as.array->next_auto_key;
    snapshot.as.array->current_index = value.as.array->current_index <= snapshot.as.array->len
        ? value.as.array->current_index
        : snapshot.as.array->len;
    snapshot.as.array->has_iterator_current_index = 0;
    snapshot.as.array->iterator_current_index = 0;
    snapshot.as.array->iterator_mutation_resume_index = 0;
    snapshot.as.array->iterator_mutation_epoch = 0;
    snapshot.as.array->mutation_epoch = 0;
    return snapshot;
}

static PTN_UNUSED PtnValue ptn_trace_value_snapshot(PtnValue value) {
    PtnTraceSnapshotSeen seen;
    ptn_trace_snapshot_seen_init(&seen);
    PtnValue snapshot = ptn_trace_value_snapshot_depth(value, 0, &seen);
    ptn_trace_snapshot_seen_free(&seen);
    return snapshot;
}

static PTN_UNUSED int ptn_trace_frame_arg_is_sensitive(PtnTraceFrame *frame, size_t position) {
    if (frame->sensitive_parameters == NULL) {
        return 0;
    }
    if (
        frame->sensitive_variadic_position != (size_t)-1 &&
        position >= frame->sensitive_variadic_position
    ) {
        return 1;
    }
    return position < frame->sensitive_parameter_count && frame->sensitive_parameters[position] != 0;
}

static PTN_UNUSED PtnValue ptn_trace_frame_arg_value(PtnTraceFrame *frame, size_t position) {
    int is_sensitive = ptn_trace_frame_arg_is_sensitive(frame, position);
    PtnValue result = ptn_null();
    if (
        frame->runtime != NULL &&
        frame->parameter_names != NULL &&
        position < frame->parameter_count
    ) {
        PtnValue value;
        if (ptn_symbols_get(&frame->runtime->symbols, frame->parameter_names[position], &value)) {
            result = ptn_trace_value_snapshot(value);
            goto wrap_if_sensitive;
        }
    }
    if (position < frame->argc) {
        result = ptn_trace_value_snapshot(frame->args[position]);
        goto wrap_if_sensitive;
    }

wrap_if_sensitive:
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (is_sensitive && frame->runtime != NULL) {
        PtnValue wrapper_args[1] = { result };
        PtnValue wrapped = ptn_sensitive_parameter_value_new(
            frame->runtime,
            1,
            wrapper_args,
            frame->line
        );
        ptn_value_destroy(&result);
        return wrapped;
    }
#else
    (void)is_sensitive;
#endif
    return result;
}

static PTN_UNUSED PtnValue ptn_trace_frame_args_array(PtnTraceFrame *frame) {
    PtnValue args = ptn_array_from_literal_entries(0, NULL);
    for (size_t i = 0; i < frame->argc; i++) {
        if (i > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        PtnArrayKey key = frame->arg_names != NULL && frame->arg_names[i] != NULL
            ? ptn_array_string_key(frame->arg_names[i])
            : ptn_array_int_key((int64_t)i);
        ptn_array_set_entry(
            args.as.array,
            key,
            ptn_trace_frame_arg_value(frame, i)
        );
    }
    return args;
}

static const char *ptn_trace_frame_method_separator(const char *function_name);

static PTN_UNUSED PtnValue ptn_trace_frame_array(PtnTraceFrame *frame) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (frame->file != NULL && frame->line != 0) {
        if (frame->line > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("file"),
            ptn_owned_string(ptn_duplicate_string(frame->file))
        );
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("line"),
            ptn_int((int64_t)frame->line)
        );
    }
    if (frame->function_name != NULL) {
        const char *separator = ptn_trace_frame_method_separator(frame->function_name);
        if (separator != NULL && separator != frame->function_name && separator[2] != '\0') {
            size_t class_len = (size_t)(separator - frame->function_name);
            const char *method_name = separator + 2;
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("function"),
                ptn_owned_string(ptn_duplicate_string(method_name))
            );
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("class"),
                ptn_owned_string_len(ptn_duplicate_string_len(frame->function_name, class_len), class_len)
            );
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("type"),
                ptn_string(separator[0] == '-' ? "->" : "::")
            );
        } else {
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("function"),
                ptn_owned_string(ptn_duplicate_string(frame->function_name))
            );
        }
    }
    ptn_array_set_entry(
        result.as.array,
        ptn_array_string_key("args"),
        ptn_trace_frame_args_array(frame)
    );
    return result;
}

static const char *ptn_trace_frame_method_separator(const char *function_name) {
    const char *object_separator = strstr(function_name, "->");
    if (object_separator != NULL) {
        return object_separator;
    }
    return strstr(function_name, "::");
}

static PTN_UNUSED PtnValue ptn_debug_backtrace_frame_array(PtnTraceFrame *frame, int64_t options) {
    PtnValue result = ptn_array_from_literal_entries(0, NULL);
    if (frame->file != NULL && frame->line != 0) {
        if (frame->line > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("file"),
            ptn_owned_string(ptn_duplicate_string(frame->file))
        );
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("line"),
            ptn_int((int64_t)frame->line)
        );
    }
    if (frame->function_name != NULL) {
        const char *separator = ptn_trace_frame_method_separator(frame->function_name);
        if (separator != NULL && separator != frame->function_name && separator[2] != '\0') {
            size_t class_len = (size_t)(separator - frame->function_name);
            const char *method_name = separator + 2;
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("function"),
                ptn_owned_string(ptn_duplicate_string(method_name))
            );
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("class"),
                ptn_owned_string_len(ptn_duplicate_string_len(frame->function_name, class_len), class_len)
            );
            if (
                (options & PTN_DEBUG_BACKTRACE_PROVIDE_OBJECT) != 0 &&
                frame->has_receiver
            ) {
                PtnValue receiver = ptn_value_deref(frame->receiver);
                if (receiver.type == PTN_OBJECT) {
                    ptn_array_set_entry(
                        result.as.array,
                        ptn_array_string_key("object"),
                        ptn_trace_value_snapshot(receiver)
                    );
                }
            }
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("type"),
                ptn_string(separator[0] == '-' ? "->" : "::")
            );
        } else {
            ptn_array_set_entry(
                result.as.array,
                ptn_array_string_key("function"),
                ptn_owned_string(ptn_duplicate_string(frame->function_name))
            );
        }
    }
    if ((options & PTN_DEBUG_BACKTRACE_IGNORE_ARGS) == 0) {
        ptn_array_set_entry(
            result.as.array,
            ptn_array_string_key("args"),
            ptn_trace_frame_args_array(frame)
        );
    }
    return result;
}

static PTN_UNUSED size_t ptn_runtime_exception_string_param_max_len(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return 15;
    }
    return root->exception_string_param_max_len;
}

static PTN_UNUSED int ptn_runtime_exception_ignore_args(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return 0;
    }
    return root->exception_ignore_args;
}

static PTN_UNUSED void ptn_runtime_set_exception_ignore_args(
    PtnRuntime *runtime,
    int value
) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return;
    }
    root->exception_ignore_args = value ? 1 : 0;
}

static PTN_UNUSED void ptn_runtime_set_exception_string_param_max_len(
    PtnRuntime *runtime,
    size_t value
) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return;
    }
    root->exception_string_param_max_len = value;
}

static PtnValue *ptn_trace_array_string_slot(PtnValue frame, const char *key) {
    frame = ptn_value_deref(frame);
    if (frame.type != PTN_ARRAY || frame.as.array == NULL) {
        return NULL;
    }
    size_t key_len = strlen(key);
    for (size_t i = 0; i < frame.as.array->len; i++) {
        PtnArrayEntry *entry = &frame.as.array->entries[i];
        if (
            entry->key.type == PTN_ARRAY_KEY_STRING &&
            entry->key.string_len == key_len &&
            memcmp(entry->key.as.string, key, key_len) == 0
        ) {
            return &entry->value;
        }
    }
    return NULL;
}

static void ptn_trace_append_quoted_string(
    PtnStringBuffer *buffer,
    const unsigned char *data,
    size_t len,
    size_t max_len
) {
    ptn_string_buffer_append_char(buffer, '\'');
    size_t display_len = len;
    int append_ellipsis = 0;
    if (len > max_len) {
        display_len = max_len;
        append_ellipsis = 1;
    }
    for (size_t i = 0; i < display_len; i++) {
        unsigned char byte = data[i];
        switch (byte) {
            case '\n':
                ptn_string_buffer_append(buffer, "\\n");
                break;
            case '\r':
                ptn_string_buffer_append(buffer, "\\r");
                break;
            case '\t':
                ptn_string_buffer_append(buffer, "\\t");
                break;
            case '\\':
                ptn_string_buffer_append(buffer, "\\\\");
                break;
            case '\'':
                ptn_string_buffer_append(buffer, "\\'");
                break;
            default:
                if (byte < 0x20 || byte >= 0x7f) {
                    ptn_string_buffer_append_format(buffer, "\\x%02X", (unsigned int)byte);
                } else {
                    ptn_string_buffer_append_char(buffer, (char)byte);
                }
                break;
        }
    }
    if (append_ellipsis) {
        ptn_string_buffer_append(buffer, "...");
    }
    ptn_string_buffer_append_char(buffer, '\'');
}

static void ptn_trace_append_arg(PtnStringBuffer *buffer, PtnValue value, size_t max_string_len) {
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
            char formatted[128];
            ptn_format_scalar_float(value.as.floating, formatted, sizeof(formatted));
            if (
                isfinite(value.as.floating) &&
                !ptn_formatted_float_has_decimal_or_exponent(formatted)
            ) {
                size_t formatted_len = strlen(formatted);
                if (formatted_len + 2 >= sizeof(formatted)) {
                    ptn_abort_out_of_memory();
                }
                formatted[formatted_len] = '.';
                formatted[formatted_len + 1] = '0';
                formatted[formatted_len + 2] = '\0';
            }
            ptn_string_buffer_append(buffer, formatted);
            break;
        }
        case PTN_STRING:
            ptn_trace_append_quoted_string(
                buffer,
                value.as.string.data,
                value.as.string.len,
                max_string_len
            );
            break;
        case PTN_ARRAY:
            ptn_string_buffer_append(buffer, "Array");
            break;
        case PTN_OBJECT:
            if (value.as.object->enum_case_name != NULL) {
                ptn_string_buffer_append_format(
                    buffer,
                    "%s::%s",
                    value.as.object->class_name,
                    value.as.object->enum_case_name
                );
            } else {
                ptn_string_buffer_append_format(buffer, "Object(%s)", value.as.object->class_name);
            }
            break;
        case PTN_CLOSURE:
            ptn_string_buffer_append(buffer, "Object(Closure)");
            break;
        case PTN_EXCEPTION:
            ptn_string_buffer_append_format(buffer, "Object(%s)", value.as.exception->class_name);
            break;
        case PTN_RESOURCE: {
            const char *curl_class_name = ptn_resource_curl_class_name(value.as.resource);
            if (curl_class_name != NULL) {
                ptn_string_buffer_append_format(buffer, "Object(%s)", curl_class_name);
                break;
            }
            ptn_string_buffer_append_format(buffer, "Resource id #%lld", (long long)value.as.resource->id);
            break;
        }
        case PTN_REFERENCE:
            ptn_string_buffer_append(buffer, "NULL");
            break;
    }
}

static void ptn_exception_append_display_function(
    PtnStringBuffer *buffer,
    const char *function_name
) {
    const char *constructor_separator = strstr(function_name, "::__construct");
    if (constructor_separator != NULL && constructor_separator[13] == '\0') {
        ptn_string_buffer_append_len(
            buffer,
            function_name,
            (size_t)(constructor_separator - function_name)
        );
        ptn_string_buffer_append(buffer, "->__construct");
        return;
    }
    ptn_string_buffer_append(buffer, function_name);
}

static PTN_UNUSED int ptn_trace_function_omits_printed_args(const char *function_name) {
    return function_name != NULL &&
        (ptn_ascii_case_equal(function_name, "include") ||
         ptn_ascii_case_equal(function_name, "include_once") ||
         ptn_ascii_case_equal(function_name, "require") ||
         ptn_ascii_case_equal(function_name, "require_once"));
}

static void ptn_exception_trace_warning(PtnRuntime *runtime, const char *message, size_t line) {
    if (runtime != NULL) {
        ptn_emit_warning(&runtime->diagnostics, message, line);
    }
}

static int ptn_exception_append_trace_frame(
    PtnRuntime *runtime,
    PtnStringBuffer *buffer,
    size_t index,
    PtnValue frame,
    size_t max_string_len,
    size_t line
) {
    ptn_string_buffer_append_format(buffer, "#%zu ", index);
    frame = ptn_value_deref(frame);
    if (frame.type != PTN_ARRAY || frame.as.array == NULL) {
        ptn_exception_trace_warning(runtime, "Expected array for frame 0", line);
        ptn_string_buffer_append(buffer, "{main}");
        return 1;
    }

    PtnValue *file_slot = ptn_trace_array_string_slot(frame, "file");
    PtnValue *line_slot = ptn_trace_array_string_slot(frame, "line");
    PtnValue file_value = file_slot == NULL ? ptn_null() : ptn_value_deref(*file_slot);
    PtnValue line_value = line_slot == NULL ? ptn_null() : ptn_value_deref(*line_slot);
    if (file_value.type == PTN_STRING && line_value.type == PTN_INT) {
        ptn_string_buffer_append_len(
            buffer,
            (const char *)file_value.as.string.data,
            file_value.as.string.len
        );
        ptn_string_buffer_append_format(buffer, "(%lld): ", (long long)line_value.as.integer);
    } else if (file_slot != NULL) {
        if (file_value.type != PTN_STRING) {
            ptn_exception_trace_warning(runtime, "File name is not a string", line);
        }
        ptn_string_buffer_append(buffer, "[unknown file]: ");
    } else {
        ptn_string_buffer_append(buffer, "[internal function]: ");
    }

    PtnValue *class_slot = ptn_trace_array_string_slot(frame, "class");
    PtnValue *type_slot = ptn_trace_array_string_slot(frame, "type");
    PtnValue *function_slot = ptn_trace_array_string_slot(frame, "function");
    PtnValue class_value = class_slot == NULL ? ptn_null() : ptn_value_deref(*class_slot);
    PtnValue type_value = type_slot == NULL ? ptn_null() : ptn_value_deref(*type_slot);
    PtnValue function_value = function_slot == NULL ? ptn_null() : ptn_value_deref(*function_slot);
    if (class_slot != NULL) {
        if (class_value.type == PTN_STRING) {
            ptn_string_buffer_append_len(
                buffer,
                (const char *)class_value.as.string.data,
                class_value.as.string.len
            );
        } else {
            ptn_exception_trace_warning(runtime, "Value for class is not a string", line);
            ptn_string_buffer_append(buffer, "[unknown]");
        }
    }
    if (type_slot != NULL) {
        if (type_value.type == PTN_STRING) {
            ptn_string_buffer_append_len(
                buffer,
                (const char *)type_value.as.string.data,
                type_value.as.string.len
            );
        } else {
            ptn_exception_trace_warning(runtime, "Value for type is not a string", line);
            ptn_string_buffer_append(buffer, "[unknown]");
        }
    }
    if (function_value.type == PTN_STRING) {
        char *function_name = ptn_duplicate_string_len(
            (const char *)function_value.as.string.data,
            function_value.as.string.len
        );
        ptn_exception_append_display_function(buffer, function_name);
        free(function_name);
    } else if (function_slot != NULL) {
        ptn_exception_trace_warning(runtime, "Value for function is not a string", line);
        ptn_string_buffer_append(buffer, "[unknown]");
    }
    ptn_string_buffer_append_char(buffer, '(');
    PtnValue *args_slot = ptn_trace_array_string_slot(frame, "args");
    PtnValue args_value = args_slot == NULL ? ptn_null() : ptn_value_deref(*args_slot);
    if (args_value.type == PTN_ARRAY && args_value.as.array != NULL) {
        for (size_t i = 0; i < args_value.as.array->len; i++) {
            if (i != 0) {
                ptn_string_buffer_append(buffer, ", ");
            }
            PtnArrayEntry *entry = &args_value.as.array->entries[i];
            if (entry->key.type == PTN_ARRAY_KEY_STRING) {
                ptn_string_buffer_append_len(buffer, entry->key.as.string, entry->key.string_len);
                ptn_string_buffer_append(buffer, ": ");
            }
            ptn_trace_append_arg(buffer, entry->value, max_string_len);
        }
    } else if (args_slot != NULL) {
        ptn_exception_trace_warning(runtime, "args element is not an array", line);
    }
    ptn_string_buffer_append_char(buffer, ')');
    return 0;
}

static PTN_UNUSED PtnStringOperand ptn_exception_trace_as_string_operand(
    PtnRuntime *runtime,
    PtnException *exception
) {
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    size_t max_string_len = ptn_runtime_exception_string_param_max_len(runtime);
    PtnValue trace = exception == NULL ? ptn_null() : ptn_value_deref(exception->trace);
    size_t index = 0;
    if (trace.type == PTN_ARRAY && trace.as.array != NULL) {
        for (size_t i = 0; i < trace.as.array->len; i++) {
            if (index != 0) {
                ptn_string_buffer_append_char(&buffer, '\n');
            }
            if (ptn_exception_append_trace_frame(
                runtime,
                &buffer,
                index,
                trace.as.array->entries[i].value,
                max_string_len,
                exception == NULL ? 0 : exception->line
            )) {
                return (PtnStringOperand) { buffer.data, buffer.data, buffer.len };
            }
            index++;
        }
    }
    if (index != 0) {
        ptn_string_buffer_append_char(&buffer, '\n');
    }
    ptn_string_buffer_append_format(&buffer, "#%zu {main}", index);
    return (PtnStringOperand) { buffer.data, buffer.data, buffer.len };
}

static PTN_UNUSED void ptn_exception_append_to_string_chain(
    PtnRuntime *runtime,
    PtnStringBuffer *buffer,
    PtnException *exception,
    int *first
) {
    if (exception == NULL) {
        return;
    }
    if (exception->previous.type == PTN_EXCEPTION) {
        ptn_exception_append_to_string_chain(
            runtime,
            buffer,
            exception->previous.as.exception,
            first
        );
    }
    if (*first) {
        *first = 0;
    } else {
        ptn_string_buffer_append(buffer, "\n\nNext ");
    }
    ptn_string_buffer_append(buffer, exception->class_name);
    if (exception->message_len != 0) {
        ptn_string_buffer_append(buffer, ": ");
        ptn_string_buffer_append_len(buffer, exception->message, exception->message_len);
        if (
            exception->message_defined_at_location &&
            exception->path != NULL &&
            exception->line != 0
        ) {
            ptn_string_buffer_append_format(
                buffer,
                " and defined in %s:%zu",
                exception->path,
                exception->line
            );
        }
    }
    if (!exception->message_defined_at_location) {
        ptn_string_buffer_append(buffer, " in ");
        ptn_string_buffer_append(buffer, exception->path == NULL ? "ptn" : exception->path);
        ptn_string_buffer_append_format(buffer, ":%zu", exception->line);
    }
    ptn_string_buffer_append(buffer, "\nStack trace:\n");
    PtnStringOperand trace = ptn_exception_trace_as_string_operand(runtime, exception);
    ptn_string_buffer_append_len(buffer, trace.data, trace.len);
    free(trace.owned);
}

static PTN_UNUSED PtnStringOperand ptn_exception_to_string_operand(
    PtnRuntime *runtime,
    PtnException *exception
) {
    PtnStringBuffer buffer;
    ptn_string_buffer_init(&buffer);
    int first = 1;
    ptn_exception_append_to_string_chain(runtime, &buffer, exception, &first);
    return (PtnStringOperand) { buffer.data, buffer.data, buffer.len };
}

static PTN_UNUSED void ptn_exception_trace_append_frame(
    PtnValue trace,
    PtnTraceFrame *frame,
    size_t *index
) {
    if (*index > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_array_set_entry(
        trace.as.array,
        ptn_array_int_key((int64_t)*index),
        ptn_trace_frame_array(frame)
    );
    (*index)++;
}

static PTN_UNUSED void ptn_exception_trace_append_value_frame(
    PtnValue trace,
    size_t *index,
    PtnValue frame
) {
    if (*index > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_array_set_entry(trace.as.array, ptn_array_int_key((int64_t)*index), frame);
    (*index)++;
}

static PTN_UNUSED void ptn_exception_trace_copy_location(PtnValue target, PtnValue source) {
    source = ptn_value_deref(source);
    if (source.type != PTN_ARRAY || source.as.array == NULL) {
        return;
    }
    PtnValue *file_slot = ptn_trace_array_string_slot(source, "file");
    PtnValue *line_slot = ptn_trace_array_string_slot(source, "line");
    PtnValue file_value = file_slot == NULL ? ptn_null() : ptn_value_deref(*file_slot);
    PtnValue line_value = line_slot == NULL ? ptn_null() : ptn_value_deref(*line_slot);
    if (file_value.type != PTN_STRING || line_value.type != PTN_INT) {
        return;
    }
    ptn_array_set_entry(
        target.as.array,
        ptn_array_string_key("file"),
        ptn_owned_string_len(
            ptn_duplicate_string_len((const char *)file_value.as.string.data, file_value.as.string.len),
            file_value.as.string.len
        )
    );
    ptn_array_set_entry(
        target.as.array,
        ptn_array_string_key("line"),
        ptn_int(line_value.as.integer)
    );
}

static PTN_UNUSED PtnValue ptn_exception_trace_current_function_frame(
    PtnRuntime *runtime,
    PtnValue location_source
) {
    PtnValue frame = ptn_array_from_literal_entries(0, NULL);
    ptn_exception_trace_copy_location(frame, location_source);
    const char *function_name =
        runtime != NULL && runtime->current_function_name != NULL
            ? runtime->current_function_name
            : "{unknown}";
    const char *separator = ptn_trace_frame_method_separator(function_name);
    if (separator != NULL && separator != function_name && separator[2] != '\0') {
        size_t class_len = (size_t)(separator - function_name);
        const char *method_name = separator + 2;
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("class"),
            ptn_owned_string_len(ptn_duplicate_string_len(function_name, class_len), class_len)
        );
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("type"),
            ptn_string(runtime != NULL && runtime->has_current_receiver ? "->" : (separator[0] == '-' ? "->" : "::"))
        );
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("function"),
            ptn_owned_string(ptn_duplicate_string(method_name))
        );
    } else {
        ptn_array_set_entry(
            frame.as.array,
            ptn_array_string_key("function"),
            ptn_owned_string(ptn_duplicate_string(function_name))
        );
    }
    ptn_array_set_entry(
        frame.as.array,
        ptn_array_string_key("args"),
        ptn_array_from_literal_entries(0, NULL)
    );
    return frame;
}

static PTN_UNUSED PtnValue ptn_exception_capture_deferred_destructor_trace(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (
        root == NULL ||
        !root->defer_uncaught_exception_emit ||
        !root->destructor_shutdown_phase ||
        runtime->exceptions == NULL ||
        runtime->exceptions->active_exception == NULL
    ) {
        return ptn_null();
    }
    PtnValue previous_trace = ptn_value_deref(runtime->exceptions->active_exception->trace);
    if (previous_trace.type != PTN_ARRAY || previous_trace.as.array == NULL || previous_trace.as.array->len == 0) {
        return ptn_null();
    }
    PtnValue trace = ptn_array_from_literal_entries(0, NULL);
    size_t index = 0;
    ptn_exception_trace_append_value_frame(
        trace,
        &index,
        ptn_exception_trace_current_function_frame(runtime, previous_trace.as.array->entries[0].value)
    );
    for (size_t i = 1; i < previous_trace.as.array->len; i++) {
        ptn_exception_trace_append_value_frame(
            trace,
            &index,
            ptn_value_clone_deref(previous_trace.as.array->entries[i].value)
        );
    }
    return trace;
}

static PTN_UNUSED PtnValue ptn_exception_capture_trace(PtnRuntime *runtime) {
    PtnValue deferred_destructor_trace = ptn_exception_capture_deferred_destructor_trace(runtime);
    if (ptn_value_deref(deferred_destructor_trace).type == PTN_ARRAY) {
        return deferred_destructor_trace;
    }
    ptn_value_destroy(&deferred_destructor_trace);
    PtnValue trace = ptn_array_from_literal_entries(0, NULL);
    if (
        runtime != NULL &&
        runtime->current_generator != NULL &&
        !runtime->generator_aborted_after_yield
    ) {
        int implicit_foreach_rewind = runtime->implicit_generator_foreach_rewind;
        const char *foreach_source_path = runtime->implicit_generator_foreach_source_path != NULL
            ? runtime->implicit_generator_foreach_source_path
            : runtime->source_path;
        size_t foreach_line = runtime->implicit_generator_foreach_line != 0
            ? runtime->implicit_generator_foreach_line
            : runtime->call_site_line;
        PtnValue generator_frame = ptn_array_from_literal_entries(0, NULL);
        if (implicit_foreach_rewind || runtime->suppress_generator_rewind_trace_frame) {
            ptn_generator_trace_set_file_line(generator_frame, foreach_source_path, foreach_line);
        }
        ptn_array_set_entry(
            generator_frame.as.array,
            ptn_array_string_key("function"),
            ptn_owned_string(ptn_duplicate_string(runtime->current_function_name == NULL ? "" : runtime->current_function_name))
        );
        ptn_array_set_entry(
            generator_frame.as.array,
            ptn_array_string_key("args"),
            ptn_array_from_literal_entries(0, NULL)
        );
        if (runtime->suppress_generator_rewind_trace_frame) {
            ptn_array_set_entry(trace.as.array, ptn_array_int_key(0), generator_frame);
            return trace;
        }
        ptn_array_set_entry(trace.as.array, ptn_array_int_key(0), generator_frame);

        if (implicit_foreach_rewind) {
            return trace;
        }

        PtnValue method_frame = ptn_array_from_literal_entries(0, NULL);
        if (runtime->source_path != NULL) {
            ptn_array_set_entry(
                method_frame.as.array,
                ptn_array_string_key("file"),
                ptn_owned_string(ptn_duplicate_string(runtime->source_path))
            );
        }
        if (runtime->call_site_line <= (size_t)INT64_MAX) {
            ptn_array_set_entry(
                method_frame.as.array,
                ptn_array_string_key("line"),
                ptn_int((int64_t)runtime->call_site_line)
            );
        }
        ptn_array_set_entry(
            method_frame.as.array,
            ptn_array_string_key("class"),
            ptn_string("Generator")
        );
        ptn_array_set_entry(
            method_frame.as.array,
            ptn_array_string_key("type"),
            ptn_string("->")
        );
        ptn_array_set_entry(
            method_frame.as.array,
            ptn_array_string_key("function"),
            ptn_string("rewind")
        );
        ptn_array_set_entry(
            method_frame.as.array,
            ptn_array_string_key("args"),
            ptn_array_from_literal_entries(0, NULL)
        );
        ptn_array_set_entry(trace.as.array, ptn_array_int_key(1), method_frame);
        return trace;
    }
    size_t index = 0;
    PtnTraceFrame *frame = runtime != NULL ? runtime->trace_frame : NULL;
    while (frame != NULL) {
        if (
            frame->previous != NULL &&
            frame->previous->function_name != NULL &&
            strcmp(frame->previous->function_name, "[constant expression]") == 0
        ) {
            PtnTraceFrame *constant_frame = frame->previous;
            ptn_exception_trace_append_frame(trace, constant_frame, &index);
            ptn_exception_trace_append_frame(trace, frame, &index);
            frame = constant_frame->previous;
            continue;
        }
        if (index > (size_t)INT64_MAX) {
            ptn_abort_out_of_memory();
        }
        ptn_exception_trace_append_frame(trace, frame, &index);
        frame = frame->previous;
    }
    return trace;
}

static PTN_UNUSED int ptn_try_object_to_string_operand(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line,
    PtnStringOperand *out
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
) {
    PtnException *exception = malloc(sizeof(PtnException));
    if (exception == NULL) {
        ptn_abort_out_of_memory();
    }
    exception->refcount = 1;
    exception->object_id = ptn_runtime_alloc_object_id(runtime);
    exception->lifecycle_runtime = ptn_runtime_root(runtime);
    exception->class_name = class_name;
    exception->message = message;
    exception->message_len = message_len;
    exception->uncaught_text = NULL;
    exception->uncaught_text_len = 0;
    exception->code = code;
    exception->path = path;
    exception->line = line;
    exception->message_defined_at_location = 0;
    exception->trace = ptn_exception_capture_trace(runtime);
    exception->previous = ptn_value_clone_deref(previous);
    exception->severity = severity;
    exception->dynamic_properties = ptn_array_from_literal_entries(0, NULL);
    exception->errors = ptn_ascii_case_equal(class_name, "Uri\\WhatWg\\InvalidUrlException")
        ? ptn_array_from_literal_entries(0, NULL)
        : ptn_null();
    exception->soap_fault_headerfault = ptn_null();
    return exception;
}

static PTN_UNUSED void ptn_exception_mark_message_defined_at_location(PtnException *exception) {
    if (exception != NULL) {
        exception->message_defined_at_location = 1;
    }
}

static PTN_UNUSED PtnException *ptn_exception_new_owned_cstr(
    PtnRuntime *runtime,
    const char *class_name,
    char *message,
    const char *path,
    size_t line
) {
    return ptn_exception_new_owned(
        runtime,
        class_name,
        message,
        strlen(message),
        0,
        ptn_null(),
        PTN_E_ERROR,
        path,
        line
    );
}

static PTN_UNUSED PtnException *ptn_exception_new(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
) {
    return ptn_exception_new_owned(
        runtime,
        class_name,
        ptn_duplicate_string(message),
        strlen(message),
        0,
        ptn_null(),
        PTN_E_ERROR,
        path,
        line
    );
}

static PTN_UNUSED PtnValue ptn_exception_previous_or_active(
    PtnRuntime *runtime,
    PtnValue previous
) {
    PtnValue resolved = ptn_value_deref(previous);
    if (resolved.type != PTN_NULL || runtime->exceptions->active_exception == NULL) {
        return previous;
    }
    return ptn_exception_borrow(runtime->exceptions->active_exception);
}

static PTN_UNUSED int ptn_exception_name_equal(const char *left, const char *right);

static PTN_UNUSED const char *ptn_builtin_exception_class_name(const char *class_name) {
    if (class_name[0] == '\\') {
        class_name++;
    }
    if (ptn_exception_name_equal(class_name, "Exception")) {
        return "Exception";
    }
    if (ptn_exception_name_equal(class_name, "ErrorException")) {
        return "ErrorException";
    }
    if (ptn_exception_name_equal(class_name, "ReflectionException")) {
        return "ReflectionException";
    }
    if (ptn_exception_name_equal(class_name, "SoapFault")) {
        return "SoapFault";
    }
    if (ptn_exception_name_equal(class_name, "PDOException")) {
        return "PDOException";
    }
    if (ptn_exception_name_equal(class_name, "PharException")) {
        return "PharException";
    }
    if (ptn_exception_name_equal(class_name, "JsonException")) {
        return "JsonException";
    }
    if (ptn_exception_name_equal(class_name, "IntlException")) {
        return "IntlException";
    }
    if (ptn_exception_name_equal(class_name, "RequestParseBodyException")) {
        return "RequestParseBodyException";
    }
    if (ptn_exception_name_equal(class_name, "LogicException")) {
        return "LogicException";
    }
    if (ptn_exception_name_equal(class_name, "BadFunctionCallException")) {
        return "BadFunctionCallException";
    }
    if (ptn_exception_name_equal(class_name, "BadMethodCallException")) {
        return "BadMethodCallException";
    }
    if (ptn_exception_name_equal(class_name, "RuntimeException")) {
        return "RuntimeException";
    }
    if (ptn_exception_name_equal(class_name, "InvalidArgumentException")) {
        return "InvalidArgumentException";
    }
    if (ptn_exception_name_equal(class_name, "UnexpectedValueException")) {
        return "UnexpectedValueException";
    }
    if (ptn_exception_name_equal(class_name, "OutOfBoundsException")) {
        return "OutOfBoundsException";
    }
    if (ptn_exception_name_equal(class_name, "OutOfRangeException")) {
        return "OutOfRangeException";
    }
    if (ptn_exception_name_equal(class_name, "Error")) {
        return "Error";
    }
    if (ptn_exception_name_equal(class_name, "UnhandledMatchError")) {
        return "UnhandledMatchError";
    }
    if (ptn_exception_name_equal(class_name, "TypeError")) {
        return "TypeError";
    }
    if (ptn_exception_name_equal(class_name, "ArgumentCountError")) {
        return "ArgumentCountError";
    }
    if (ptn_exception_name_equal(class_name, "ValueError")) {
        return "ValueError";
    }
    if (ptn_exception_name_equal(class_name, "FiberError")) {
        return "FiberError";
    }
    if (ptn_exception_name_equal(class_name, "Uri\\InvalidUriException")) {
        return "Uri\\InvalidUriException";
    }
    if (ptn_exception_name_equal(class_name, "Uri\\WhatWg\\InvalidUrlException")) {
        return "Uri\\WhatWg\\InvalidUrlException";
    }
    if (ptn_exception_name_equal(class_name, "DateRangeError")) {
        return "DateRangeError";
    }
    if (ptn_exception_name_equal(class_name, "DateObjectError")) {
        return "DateObjectError";
    }
    if (ptn_exception_name_equal(class_name, "DateMalformedStringException")) {
        return "DateMalformedStringException";
    }
    if (ptn_exception_name_equal(class_name, "DateMalformedIntervalStringException")) {
        return "DateMalformedIntervalStringException";
    }
    if (ptn_exception_name_equal(class_name, "DateMalformedPeriodStringException")) {
        return "DateMalformedPeriodStringException";
    }
    if (ptn_exception_name_equal(class_name, "ArithmeticError")) {
        return "ArithmeticError";
    }
    if (ptn_exception_name_equal(class_name, "DivisionByZeroError")) {
        return "DivisionByZeroError";
    }
    if (ptn_exception_name_equal(class_name, "AssertionError")) {
        return "AssertionError";
    }
    if (ptn_exception_name_equal(class_name, "ParseError")) {
        return "ParseError";
    }
    if (ptn_exception_name_equal(class_name, "UnhandledMatchError")) {
        return "UnhandledMatchError";
    }
    return NULL;
}

static PTN_UNUSED int ptn_exception_name_equal(const char *left, const char *right) {
    while (*left != '\0' && *right != '\0') {
        int left_byte = tolower((unsigned char)*left);
        int right_byte = tolower((unsigned char)*right);
        if (left_byte != right_byte) {
            return 0;
        }
        left++;
        right++;
    }
    return *left == '\0' && *right == '\0';
}

static PTN_UNUSED int ptn_exception_type_matches_name(const char *class_name, const char *type_name) {
    if (type_name[0] == '\\') {
        type_name++;
    }
    if (ptn_exception_name_equal(class_name, type_name)) {
        return 1;
    }
    if (ptn_exception_name_equal(type_name, "Error")) {
        return ptn_exception_name_equal(class_name, "Error") ||
            ptn_exception_name_equal(class_name, "TypeError") ||
            ptn_exception_name_equal(class_name, "ArgumentCountError") ||
            ptn_exception_name_equal(class_name, "ValueError") ||
            ptn_exception_name_equal(class_name, "FiberError") ||
            ptn_exception_name_equal(class_name, "Uri\\InvalidUriException") ||
            ptn_exception_name_equal(class_name, "Uri\\WhatWg\\InvalidUrlException") ||
            ptn_exception_name_equal(class_name, "DateRangeError") ||
            ptn_exception_name_equal(class_name, "DateObjectError") ||
            ptn_exception_name_equal(class_name, "ArithmeticError") ||
            ptn_exception_name_equal(class_name, "DivisionByZeroError") ||
            ptn_exception_name_equal(class_name, "UnhandledMatchError") ||
            ptn_exception_name_equal(class_name, "AssertionError") ||
            ptn_exception_name_equal(class_name, "ParseError") ||
            ptn_exception_name_equal(class_name, "UnhandledMatchError");
    }
    if (ptn_exception_name_equal(type_name, "ValueError")) {
        return ptn_exception_name_equal(class_name, "Uri\\InvalidUriException") ||
            ptn_exception_name_equal(class_name, "Uri\\WhatWg\\InvalidUrlException") ||
            ptn_exception_name_equal(class_name, "DateRangeError");
    }
    if (ptn_exception_name_equal(type_name, "TypeError")) {
        return ptn_exception_name_equal(class_name, "ArgumentCountError");
    }
    if (ptn_exception_name_equal(type_name, "ArithmeticError")) {
        return ptn_exception_name_equal(class_name, "DivisionByZeroError");
    }
    if (ptn_exception_name_equal(type_name, "Exception")) {
        return ptn_exception_name_equal(class_name, "ErrorException") ||
            ptn_exception_name_equal(class_name, "ReflectionException") ||
            ptn_exception_name_equal(class_name, "SoapFault") ||
            ptn_exception_name_equal(class_name, "PharException") ||
            ptn_exception_name_equal(class_name, "JsonException") ||
            ptn_exception_name_equal(class_name, "RequestParseBodyException") ||
            ptn_exception_name_equal(class_name, "DateMalformedStringException") ||
            ptn_exception_name_equal(class_name, "DateMalformedIntervalStringException") ||
            ptn_exception_name_equal(class_name, "DateMalformedPeriodStringException") ||
            ptn_exception_name_equal(class_name, "LogicException") ||
            ptn_exception_name_equal(class_name, "BadFunctionCallException") ||
            ptn_exception_name_equal(class_name, "BadMethodCallException") ||
            ptn_exception_name_equal(class_name, "RuntimeException") ||
            ptn_exception_name_equal(class_name, "InvalidArgumentException") ||
            ptn_exception_name_equal(class_name, "UnexpectedValueException") ||
            ptn_exception_name_equal(class_name, "OutOfBoundsException") ||
            ptn_exception_name_equal(class_name, "OutOfRangeException");
    }
    if (ptn_exception_name_equal(type_name, "LogicException")) {
        return ptn_exception_name_equal(class_name, "BadFunctionCallException") ||
            ptn_exception_name_equal(class_name, "BadMethodCallException");
    }
    if (ptn_exception_name_equal(type_name, "BadFunctionCallException")) {
        return ptn_exception_name_equal(class_name, "BadMethodCallException");
    }
    if (ptn_exception_name_equal(type_name, "RuntimeException")) {
        return ptn_exception_name_equal(class_name, "InvalidArgumentException") ||
            ptn_exception_name_equal(class_name, "UnexpectedValueException") ||
            ptn_exception_name_equal(class_name, "OutOfBoundsException") ||
            ptn_exception_name_equal(class_name, "OutOfRangeException");
    }
    if (ptn_exception_name_equal(type_name, "Throwable")) {
        return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_try_frame_push(PtnRuntime *runtime, PtnTryFrame *frame) {
    frame->previous = runtime->exceptions->try_frame;
    frame->is_user_try = 0;
    runtime->exceptions->try_frame = frame;
}

static PTN_UNUSED void ptn_try_frame_pop(PtnRuntime *runtime, PtnTryFrame *frame) {
    if (runtime->exceptions->try_frame == frame) {
        runtime->exceptions->try_frame = frame->previous;
    }
}

static PtnDiagnosticSink *ptn_uncaught_exception_diagnostics(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return NULL;
    }
    PtnRuntime *root = ptn_runtime_root(runtime);
    return root == NULL ? &runtime->diagnostics : &root->diagnostics;
}

static void ptn_uncaught_exception_output_write(PtnRuntime *runtime, const char *data, size_t len) {
    if (data == NULL || len == 0) {
        return;
    }
    PtnDiagnosticSink *diagnostics = ptn_uncaught_exception_diagnostics(runtime);
    if (diagnostics == NULL) {
        fwrite(data, 1, len, stderr);
        return;
    }
    ptn_diagnostic_output_write(diagnostics, data, len);
}

static void ptn_uncaught_exception_output_cstr(PtnRuntime *runtime, const char *data) {
    ptn_uncaught_exception_output_write(runtime, data, data == NULL ? 0 : strlen(data));
}

static void ptn_uncaught_exception_output_flush(PtnRuntime *runtime) {
    PtnDiagnosticSink *diagnostics = ptn_uncaught_exception_diagnostics(runtime);
    if (diagnostics != NULL && diagnostics->stream != NULL) {
        fflush(diagnostics->stream);
        return;
    }
    fflush(stdout);
}

static void ptn_uncaught_exception_output_printf(PtnRuntime *runtime, const char *format, ...) {
    va_list args;
    va_start(args, format);
    va_list copy;
    va_copy(copy, args);
    int needed = vsnprintf(NULL, 0, format, args);
    va_end(args);
    if (needed < 0) {
        va_end(copy);
        ptn_abort_out_of_memory();
    }
    char *buffer = malloc((size_t)needed + 1);
    if (buffer == NULL) {
        va_end(copy);
        ptn_abort_out_of_memory();
    }
    int written = vsnprintf(buffer, (size_t)needed + 1, format, copy);
    va_end(copy);
    if (written < 0 || written != needed) {
        free(buffer);
        ptn_abort_out_of_memory();
    }
    ptn_uncaught_exception_output_write(runtime, buffer, (size_t)written);
    free(buffer);
}

static PTN_UNUSED void ptn_emit_uncaught_exception_chain_entry(
    PtnRuntime *runtime,
    PtnException *exception,
    int *first,
    const char *first_label
) {
    if (exception->previous.type == PTN_EXCEPTION) {
        ptn_emit_uncaught_exception_chain_entry(
            runtime,
            exception->previous.as.exception,
            first,
            first_label
        );
    }
    const char *display_path = exception->path != NULL ? exception->path : "[no active file]";
    size_t display_line = exception->line;
    if (*first) {
        ptn_uncaught_exception_output_cstr(runtime, "\n");
        ptn_uncaught_exception_output_printf(
            runtime,
            "%s: Uncaught %s",
            first_label,
            exception->class_name
        );
        if (exception->message_len != 0) {
            ptn_uncaught_exception_output_cstr(runtime, ": ");
            ptn_uncaught_exception_output_write(runtime, exception->message, exception->message_len);
        }
        if (exception->message_defined_at_location && exception->path != NULL && exception->line != 0) {
            ptn_uncaught_exception_output_printf(runtime, " and defined in %s:%zu\n", exception->path, exception->line);
        } else {
            ptn_uncaught_exception_output_printf(runtime, " in %s:%zu\n", display_path, display_line);
        }
        *first = 0;
    } else {
        ptn_uncaught_exception_output_printf(
            runtime,
            "\nNext %s",
            exception->class_name
        );
        if (exception->message_len != 0) {
            ptn_uncaught_exception_output_cstr(runtime, ": ");
            ptn_uncaught_exception_output_write(runtime, exception->message, exception->message_len);
        }
        if (exception->message_defined_at_location && exception->path != NULL && exception->line != 0) {
            ptn_uncaught_exception_output_printf(runtime, " and defined in %s:%zu\n", exception->path, exception->line);
        } else {
            ptn_uncaught_exception_output_printf(runtime, " in %s:%zu\n", display_path, display_line);
        }
    }
    ptn_uncaught_exception_output_cstr(runtime, "Stack trace:\n");
    PtnStringOperand trace = ptn_exception_trace_as_string_operand(runtime, exception);
    ptn_uncaught_exception_output_write(runtime, trace.data, trace.len);
    free(trace.owned);
    ptn_uncaught_exception_output_cstr(runtime, "\n");
}

static PTN_UNUSED void ptn_emit_uncaught_exception_with_label(
    PtnRuntime *runtime,
    PtnException *exception,
    const char *label
) {
    fflush(stdout);
    if (ptn_exception_handlers_try_uncaught(runtime, exception)) {
        return;
    }
    if (
        runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL &&
        runtime->exceptions->active_exception != exception
    ) {
        exception = runtime->exceptions->active_exception;
    }
    if (!runtime->diagnostics.display_errors) {
        return;
    }
    if (exception->previous.type == PTN_EXCEPTION) {
        int first = 1;
        ptn_emit_uncaught_exception_chain_entry(runtime, exception, &first, label);
        const char *display_path = exception->path != NULL ? exception->path : "[no active file]";
        ptn_uncaught_exception_output_printf(runtime, "  thrown in %s on line %zu\n", display_path, exception->line);
        ptn_uncaught_exception_output_flush(runtime);
        return;
    }
    const char *display_path = exception->path;
    size_t display_line = exception->line;
    PtnTraceFrame *frame = runtime != NULL ? runtime->trace_frame : NULL;
    if (
        (display_path == NULL || display_line == 0) &&
        runtime != NULL &&
        runtime->current_function_name == NULL &&
        frame != NULL &&
        frame->file != NULL &&
        frame->line != 0
    ) {
        display_path = frame->file;
        display_line = frame->line;
    }
    if (display_path == NULL) {
        ptn_uncaught_exception_output_printf(runtime, "%s: ", label);
        ptn_uncaught_exception_output_write(runtime, exception->message, exception->message_len);
        ptn_uncaught_exception_output_cstr(runtime, "\n");
        ptn_uncaught_exception_output_flush(runtime);
        return;
    }

    PtnDiagnosticSink *diagnostics = ptn_uncaught_exception_diagnostics(runtime);
    if (diagnostics != NULL && diagnostics->html_errors) {
        ptn_uncaught_exception_output_printf(runtime, "<br />\n<b>%s</b>:  Uncaught ", label);
        if (exception->uncaught_text != NULL) {
            ptn_uncaught_exception_output_write(runtime, exception->uncaught_text, exception->uncaught_text_len);
            ptn_uncaught_exception_output_cstr(runtime, "\n");
            ptn_uncaught_exception_output_printf(
                runtime,
                "  thrown in <b>%s</b> on line <b>%zu</b><br />\n",
                display_path,
                display_line
            );
            ptn_uncaught_exception_output_flush(runtime);
            return;
        }
        ptn_uncaught_exception_output_cstr(runtime, exception->class_name);
        if (exception->message_len != 0) {
            ptn_uncaught_exception_output_cstr(runtime, ": ");
            ptn_uncaught_exception_output_write(runtime, exception->message, exception->message_len);
            if (
                exception->message_defined_at_location &&
                exception->path != NULL &&
                exception->line != 0
            ) {
                ptn_uncaught_exception_output_printf(
                    runtime,
                    " and defined in %s:%zu",
                    exception->path,
                    exception->line
                );
            }
        }
        if (!exception->message_defined_at_location) {
            ptn_uncaught_exception_output_printf(runtime, " in %s:%zu", display_path, display_line);
        }
        ptn_uncaught_exception_output_cstr(runtime, "\nStack trace:\n");
        PtnStringOperand trace = ptn_exception_trace_as_string_operand(runtime, exception);
        ptn_uncaught_exception_output_write(runtime, trace.data, trace.len);
        free(trace.owned);
        ptn_uncaught_exception_output_printf(
            runtime,
            "\n  thrown in <b>%s</b> on line <b>%zu</b><br />\n",
            display_path,
            display_line
        );
        ptn_uncaught_exception_output_flush(runtime);
        return;
    }

    if (exception->uncaught_text != NULL) {
        ptn_uncaught_exception_output_cstr(runtime, "\n");
        ptn_uncaught_exception_output_printf(runtime, "%s: Uncaught ", label);
        ptn_uncaught_exception_output_write(runtime, exception->uncaught_text, exception->uncaught_text_len);
        ptn_uncaught_exception_output_cstr(runtime, "\n");
        ptn_uncaught_exception_output_printf(runtime, "  thrown in %s on line %zu\n", display_path, display_line);
        ptn_uncaught_exception_output_flush(runtime);
        return;
    }

    ptn_uncaught_exception_output_cstr(runtime, "\n");
    ptn_uncaught_exception_output_printf(runtime, "%s: Uncaught %s", label, exception->class_name);
    if (exception->message_len != 0) {
        ptn_uncaught_exception_output_cstr(runtime, ": ");
        ptn_uncaught_exception_output_write(runtime, exception->message, exception->message_len);
        if (
            exception->message_defined_at_location &&
            exception->path != NULL &&
            exception->line != 0
        ) {
            ptn_uncaught_exception_output_printf(runtime, " and defined in %s:%zu", exception->path, exception->line);
        }
    }
    if (!exception->message_defined_at_location) {
        ptn_uncaught_exception_output_printf(runtime, " in %s:%zu", display_path, display_line);
    }
    ptn_uncaught_exception_output_cstr(runtime, "\n");
    ptn_uncaught_exception_output_cstr(runtime, "Stack trace:\n");
    PtnStringOperand trace = ptn_exception_trace_as_string_operand(runtime, exception);
    ptn_uncaught_exception_output_write(runtime, trace.data, trace.len);
    free(trace.owned);
    ptn_uncaught_exception_output_cstr(runtime, "\n");
    ptn_uncaught_exception_output_printf(runtime, "  thrown in %s on line %zu\n", display_path, display_line);
    ptn_uncaught_exception_output_flush(runtime);
}

static PTN_UNUSED void ptn_emit_uncaught_exception(PtnRuntime *runtime, PtnException *exception) {
    ptn_emit_uncaught_exception_with_label(runtime, exception, "Fatal error");
}

static PTN_UNUSED void ptn_emit_uncaught_exception_warning(PtnRuntime *runtime, PtnException *exception) {
    ptn_emit_uncaught_exception_with_label(runtime, exception, "Warning");
}

static PTN_UNUSED void ptn_emit_inheritance_variance_uncaught_exception(
    PtnRuntime *runtime,
    PtnException *exception,
    const char *class_name,
    const char *source_path,
    size_t line
) {
    fflush(stdout);
    if (exception == NULL) {
        return;
    }
    if (ptn_exception_handlers_try_uncaught(runtime, exception)) {
        return;
    }
    if (
        runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL &&
        runtime->exceptions->active_exception != exception
    ) {
        exception = runtime->exceptions->active_exception;
    }
    if (!runtime->diagnostics.display_errors) {
        return;
    }
    const char *display_path = exception->path != NULL ? exception->path : source_path;
    size_t display_line = exception->line != 0 ? exception->line : line;
    fputc('\n', stderr);
    fprintf(
        stderr,
        "Fatal error: During inheritance of %s with variance dependencies: Uncaught %s",
        class_name,
        exception->class_name
    );
    if (exception->message_len != 0) {
        fputs(": ", stderr);
        fwrite(exception->message, 1, exception->message_len, stderr);
    }
    fprintf(stderr, " in %s:%zu\n", display_path, display_line);
    fputs("Stack trace:\n", stderr);
    PtnStringOperand trace = ptn_exception_trace_as_string_operand(runtime, exception);
    fwrite(trace.data, 1, trace.len, stderr);
    if (
        trace.len >= 6 &&
        memcmp(trace.data + trace.len - 6, "{main}", 6) == 0 &&
        source_path != NULL &&
        line != 0
    ) {
        fprintf(stderr, " in %s on line %zu", source_path, line);
    }
    free(trace.owned);
    fputc('\n', stderr);
}

static PTN_UNUSED void ptn_throw_exception_at(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
) {
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        class_name,
        ptn_duplicate_string(message),
        strlen(message),
        0,
        previous,
        PTN_E_ERROR,
        path,
        line
    );
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception_at_without_current_trace_frame(
    PtnRuntime *runtime,
    const char *class_name,
    const char *message,
    const char *path,
    size_t line
) {
    PtnTraceFrame *saved_trace_frame = runtime->trace_frame;
    if (saved_trace_frame != NULL) {
        runtime->trace_frame = saved_trace_frame->previous;
    }
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        class_name,
        ptn_duplicate_string(message),
        strlen(message),
        0,
        previous,
        PTN_E_ERROR,
        path,
        line
    );
    runtime->trace_frame = saved_trace_frame;
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception(PtnRuntime *runtime, const char *class_name, const char *message) {
    ptn_throw_exception_at(runtime, class_name, message, NULL, 0);
}

static PTN_UNUSED void ptn_throw_exception_owned_message(
    PtnRuntime *runtime,
    const char *class_name,
    char *message
) {
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        class_name,
        message,
        strlen(message),
        0,
        previous,
        PTN_E_ERROR,
        NULL,
        0
    );
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception_owned_message_at(
    PtnRuntime *runtime,
    const char *class_name,
    char *message,
    const char *path,
    size_t line
) {
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        class_name,
        message,
        strlen(message),
        0,
        previous,
        PTN_E_ERROR,
        path,
        line
    );
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception_owned_message_at_defined_location(
    PtnRuntime *runtime,
    const char *class_name,
    char *message,
    const char *path,
    size_t line
) {
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        class_name,
        message,
        strlen(message),
        0,
        previous,
        PTN_E_ERROR,
        path,
        line
    );
    ptn_exception_mark_message_defined_at_location(exception);
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED void ptn_throw_exception_owned_message_at_with_trace_frame(
    PtnRuntime *runtime,
    const char *class_name,
    char *message,
    const char *path,
    size_t line,
    const char *function_name,
    const char *frame_file,
    size_t frame_line,
    size_t argc,
    const PtnValue *args
) {
    PtnTraceFrame trace_frame;
    ptn_runtime_push_trace_frame(runtime, &trace_frame, function_name, frame_file, frame_line, argc, args);
    PtnValue previous = ptn_exception_previous_or_active(runtime, ptn_null());
    PtnException *exception = ptn_exception_new_owned(
        runtime,
        class_name,
        message,
        strlen(message),
        0,
        previous,
        PTN_E_ERROR,
        path,
        line
    );
    ptn_runtime_pop_trace_frame(runtime, &trace_frame);
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = exception;
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED const char *ptn_exception_constructor_declaring_class(
    PtnRuntime *runtime,
    const char *class_name
) {
    (void)runtime;
    if (ptn_declared_class_is_same_or_descendant(class_name, "ErrorException")) {
        return "ErrorException";
    }
    if (ptn_exception_name_equal(class_name, "SoapFault") ||
        ptn_declared_class_is_same_or_descendant(class_name, "SoapFault")) {
        return "SoapFault";
    }
    if (ptn_exception_name_equal(class_name, "Uri\\WhatWg\\InvalidUrlException") ||
        ptn_declared_class_is_same_or_descendant(class_name, "Uri\\WhatWg\\InvalidUrlException")) {
        return "Uri\\WhatWg\\InvalidUrlException";
    }
    if (ptn_declared_class_is_same_or_descendant(class_name, "Error")) {
        return "Error";
    }
    return "Exception";
}

static PTN_UNUSED size_t ptn_exception_constructor_max_args(const char *declaring_class) {
    if (ptn_exception_name_equal(declaring_class, "ErrorException") ||
        ptn_exception_name_equal(declaring_class, "SoapFault")) {
        return 6;
    }
    if (ptn_exception_name_equal(declaring_class, "Uri\\WhatWg\\InvalidUrlException")) {
        return 4;
    }
    return 3;
}

static const char *ptn_exception_constructor_given_type(PtnValue value) {
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
        case PTN_ARRAY:
            return "array";
        case PTN_OBJECT:
            return value.as.object->class_name;
        case PTN_CLOSURE:
            return "Closure";
        case PTN_EXCEPTION:
            return value.as.exception->class_name;
        case PTN_RESOURCE:
            return "resource";
        case PTN_REFERENCE:
            return "reference";
    }
    return "unknown";
}

static PTN_UNUSED int ptn_exception_validate_soap_fault_code(
    PtnRuntime *runtime,
    const char *declaring_class,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    if (!ptn_exception_name_equal(declaring_class, "SoapFault") || argc == 0) {
        return 1;
    }
    PtnValue code = ptn_value_deref(args[0]);
    if (code.type == PTN_NULL) {
        return 1;
    }
    if (code.type == PTN_STRING) {
        if (code.as.string.len != 0) {
            return 1;
        }
        ptn_throw_exception_at(
            runtime,
            "ValueError",
            "SoapFault::__construct(): Argument #1 ($code) is not a valid fault code",
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
        return 0;
    }
    if (code.type == PTN_ARRAY) {
        if (code.as.array != NULL && code.as.array->len == 2) {
            return 1;
        }
        ptn_throw_exception_at(
            runtime,
            "ValueError",
            "SoapFault::__construct(): Argument #1 ($code) is not a valid fault code",
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
        return 0;
    }
    const char *given = ptn_exception_constructor_given_type(code);
    int needed = snprintf(
        NULL,
        0,
        "SoapFault::__construct(): Argument #1 ($code) must be of type array|string|null, %s given",
        given
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
        "SoapFault::__construct(): Argument #1 ($code) must be of type array|string|null, %s given",
        given
    );
    ptn_throw_exception_owned_message_at(
        runtime,
        "TypeError",
        message,
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
    return 0;
}

static PTN_UNUSED PtnStringOperand ptn_exception_constructor_message(
    PtnRuntime *runtime,
    const char *declaring_class,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    size_t message_index = ptn_exception_name_equal(declaring_class, "SoapFault") ? 1 : 0;
    const char *message_parameter_name = ptn_exception_name_equal(declaring_class, "SoapFault")
        ? "string"
        : "message";
    if (argc <= message_index) {
        char *message = ptn_duplicate_string("");
        return (PtnStringOperand) { message, message, 0 };
    }

    PtnTraceFrame trace_frame;
    char trace_name[64];
    int written = snprintf(trace_name, sizeof(trace_name), "%s->__construct", declaring_class);
    if (written < 0 || (size_t)written >= sizeof(trace_name)) {
        ptn_abort_out_of_memory();
    }
    ptn_runtime_push_trace_frame(
        runtime,
        &trace_frame,
        trace_name,
        runtime != NULL ? runtime->source_path : NULL,
        line,
        argc,
        args
    );

    PtnValue value = ptn_value_deref(args[message_index]);
    if (
        value.type == PTN_ARRAY ||
        value.type == PTN_OBJECT ||
        value.type == PTN_CLOSURE ||
        value.type == PTN_EXCEPTION ||
        value.type == PTN_RESOURCE
    ) {
        const char *given = ptn_exception_constructor_given_type(value);
        int needed = snprintf(
            NULL,
            0,
            "%s::__construct(): Argument #%zu ($%s) must be of type string, %s given",
            declaring_class,
            message_index + 1,
            message_parameter_name,
            given
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
            "%s::__construct(): Argument #%zu ($%s) must be of type string, %s given",
            declaring_class,
            message_index + 1,
            message_parameter_name,
            given
        );
        ptn_throw_exception_owned_message_at(
            runtime,
            "TypeError",
            message,
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
    }

    char *message;
    size_t message_len;
    if (value.type == PTN_STRING) {
        message_len = value.as.string.len;
        message = ptn_duplicate_string_len((const char *)value.as.string.data, message_len);
    } else {
        message = ptn_value_to_string(value);
        message_len = strlen(message);
    }
    ptn_runtime_pop_trace_frame(runtime, &trace_frame);
    return (PtnStringOperand) { message, message, message_len };
}

static PTN_UNUSED int ptn_exception_is_soap_fault_class(const char *class_name) {
    return ptn_exception_name_equal(class_name, "SoapFault") ||
        ptn_declared_class_is_same_or_descendant(class_name, "SoapFault");
}

static PTN_UNUSED void ptn_exception_set_soap_fault_headerfault(
    PtnException *exception,
    size_t argc,
    const PtnValue *args
) {
    if (exception == NULL || !ptn_exception_is_soap_fault_class(exception->class_name)) {
        return;
    }
    PtnValue headerfault = argc >= 6 ? ptn_value_clone_deref(args[5]) : ptn_null();
    ptn_value_destroy(&exception->soap_fault_headerfault);
    exception->soap_fault_headerfault = headerfault;
}

static PTN_UNUSED void ptn_exception_set_soap_fault_property(
    PtnException *exception,
    const char *name,
    PtnValue value
) {
    if (exception == NULL || exception->dynamic_properties.type != PTN_ARRAY) {
        return;
    }
    ptn_array_set_entry(
        exception->dynamic_properties.as.array,
        ptn_array_string_key(name),
        ptn_value_clone_deref(value)
    );
}

static PTN_UNUSED void ptn_exception_set_soap_fault_properties(
    PtnException *exception,
    size_t argc,
    const PtnValue *args
) {
    if (exception == NULL || !ptn_exception_is_soap_fault_class(exception->class_name)) {
        return;
    }
    PtnValue faultstring = ptn_owned_string_len(
        ptn_duplicate_string_len(exception->message, exception->message_len),
        exception->message_len
    );
    ptn_exception_set_soap_fault_property(exception, "faultstring", faultstring);
    ptn_value_destroy(&faultstring);
    ptn_exception_set_soap_fault_property(exception, "faultcode", argc >= 1 ? args[0] : ptn_null());
    ptn_exception_set_soap_fault_property(
        exception,
        "faultcodens",
        ptn_string("http://schemas.xmlsoap.org/soap/envelope/")
    );
    ptn_exception_set_soap_fault_property(exception, "faultactor", argc >= 3 ? args[2] : ptn_null());
    ptn_exception_set_soap_fault_property(exception, "detail", argc >= 4 ? args[3] : ptn_null());
    ptn_exception_set_soap_fault_property(exception, "_name", argc >= 5 ? args[4] : ptn_null());
}

static PTN_UNUSED PtnValue ptn_exception_reconstruct(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type != PTN_EXCEPTION) {
        (void)runtime;
        (void)line;
        return ptn_null();
    }

    const char *declaring_class =
        ptn_exception_constructor_declaring_class(runtime, receiver.as.exception->class_name);
    size_t max_args = ptn_exception_constructor_max_args(declaring_class);
    if (argc > max_args) {
        char message[128];
        int written = snprintf(
            message,
            sizeof(message),
            "%s constructor expects at most %zu arguments",
            declaring_class,
            max_args
        );
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_throw_exception(runtime, "ArgumentCountError", message);
        return ptn_null();
    }

    if (!ptn_exception_validate_soap_fault_code(runtime, declaring_class, argc, args, line)) {
        return ptn_null();
    }

    PtnStringOperand message = ptn_exception_constructor_message(
        runtime,
        declaring_class,
        argc,
        args,
        line
    );
    if (runtime->exceptions->active_exception != NULL) {
        free(message.owned);
        return ptn_null();
    }

    PtnException *exception = receiver.as.exception;
    free(exception->message);
    exception->message = message.owned;
    exception->message_len = message.len;
    if (ptn_exception_name_equal(declaring_class, "Uri\\WhatWg\\InvalidUrlException")) {
        ptn_throw_exception(runtime, "Error", "Cannot modify readonly property Uri\\WhatWg\\InvalidUrlException::$errors");
        return ptn_null();
    }
    exception->code = 0;
    if (!ptn_exception_name_equal(declaring_class, "SoapFault") && argc >= 2) {
        PtnValue code_value = ptn_value_deref(args[1]);
        if (code_value.type == PTN_INT) {
            exception->code = code_value.as.integer;
        } else if (code_value.type == PTN_BOOL) {
            exception->code = code_value.as.boolean ? 1 : 0;
        } else if (code_value.type == PTN_FLOAT) {
            exception->code = (int64_t)code_value.as.floating;
        }
    }
    exception->severity = PTN_E_ERROR;
    if (ptn_exception_name_equal(declaring_class, "ErrorException") && argc >= 3) {
        PtnValue severity_value = ptn_value_deref(args[2]);
        if (severity_value.type == PTN_INT) {
            exception->severity = severity_value.as.integer;
        } else if (severity_value.type == PTN_BOOL) {
            exception->severity = severity_value.as.boolean ? 1 : 0;
        } else if (severity_value.type == PTN_FLOAT) {
            exception->severity = (int64_t)severity_value.as.floating;
        }
    }

    size_t previous_index = ptn_exception_name_equal(declaring_class, "ErrorException") ? 5 : 2;
    PtnValue previous = ptn_null();
    if (!ptn_exception_name_equal(declaring_class, "SoapFault") && argc > previous_index) {
        PtnValue previous_value = ptn_value_deref(args[previous_index]);
        if (previous_value.type == PTN_EXCEPTION) {
            previous = ptn_value_clone_deref(previous_value);
        }
    }
    ptn_value_destroy(&exception->previous);
    exception->previous = previous;
    ptn_exception_set_soap_fault_headerfault(exception, argc, args);
    ptn_exception_set_soap_fault_properties(exception, argc, args);
    return ptn_null();
}

static PTN_UNUSED int ptn_object_is_declared_throwable(PtnRuntime *runtime, PtnObject *object) {
    return runtime->class_scope_allows != NULL &&
        (runtime->class_scope_allows(object->class_name, "Exception") ||
            runtime->class_scope_allows(object->class_name, "Error"));
}

static PTN_UNUSED PtnStringOperand ptn_object_exception_message(
    PtnRuntime *runtime,
    PtnValue object,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(object);
    PtnLookupResult lookup = ptn_object_property_lookup_quiet(
        runtime,
        resolved,
        "message",
        resolved.as.object->class_name,
        line
    );
    if (!lookup.exists) {
        char *message = ptn_duplicate_string("");
        return (PtnStringOperand) { message, message, 0 };
    }
    PtnValue message_value = ptn_value_deref(lookup.value);
    char *message;
    size_t message_len;
    if (message_value.type == PTN_STRING) {
        message_len = message_value.as.string.len;
        message = ptn_duplicate_string_len((const char *)message_value.as.string.data, message_len);
    } else {
        message = ptn_value_to_string(message_value);
        message_len = strlen(message);
    }
    ptn_value_destroy(&lookup.value);
    return (PtnStringOperand) { message, message, message_len };
}

static PTN_UNUSED PtnLookupResult ptn_throwable_object_property(
    PtnRuntime *runtime,
    PtnValue object,
    const char *property,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(object);
    if (resolved.type != PTN_OBJECT) {
        return ptn_lookup_missing();
    }
    return ptn_object_property_lookup_quiet(
        runtime,
        resolved,
        property,
        resolved.as.object->class_name,
        line
    );
}

static PTN_UNUSED char *ptn_throwable_message_string(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_duplicate_string_len(
            receiver.as.exception->message,
            receiver.as.exception->message_len
        );
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, "message", line);
    if (!lookup.exists) {
        return ptn_duplicate_string("");
    }
    char *message = ptn_value_to_string(lookup.value);
    ptn_value_destroy(&lookup.value);
    return message;
}

static PTN_UNUSED int64_t ptn_throwable_int_property(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    int64_t fallback,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        if (ptn_exception_name_equal(property, "code")) {
            return receiver.as.exception->code;
        }
        if (ptn_exception_name_equal(property, "line")) {
            return receiver.as.exception->line > (size_t)INT64_MAX
                ? INT64_MAX
                : (int64_t)receiver.as.exception->line;
        }
        if (ptn_exception_name_equal(property, "severity")) {
            return receiver.as.exception->severity;
        }
        return fallback;
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, property, line);
    if (!lookup.exists) {
        return fallback;
    }
    PtnValue value = ptn_value_deref(lookup.value);
    int64_t result = fallback;
    if (value.type == PTN_INT) {
        result = value.as.integer;
    } else if (value.type == PTN_BOOL) {
        result = value.as.boolean ? 1 : 0;
    } else if (value.type == PTN_FLOAT) {
        result = (int64_t)value.as.floating;
    }
    ptn_value_destroy(&lookup.value);
    return result;
}

static PTN_UNUSED PtnValue ptn_throwable_file_value(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_owned_string(ptn_duplicate_string(receiver.as.exception->path != NULL ? receiver.as.exception->path : ""));
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, "file", line);
    if (!lookup.exists) {
        return ptn_owned_string(ptn_duplicate_string(runtime->source_path != NULL ? runtime->source_path : ""));
    }
    char *file = ptn_value_to_string(lookup.value);
    ptn_value_destroy(&lookup.value);
    return ptn_owned_string(file);
}

static PTN_UNUSED PtnValue ptn_throwable_previous_value(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_value_clone_deref(receiver.as.exception->previous);
    }
    PtnLookupResult lookup = ptn_throwable_object_property(runtime, receiver, "previous", line);
    if (!lookup.exists) {
        return ptn_null();
    }
    PtnValue previous = ptn_value_clone_deref(lookup.value);
    ptn_value_destroy(&lookup.value);
    return previous;
}

static PTN_UNUSED PtnValue ptn_throwable_trace_value(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        return ptn_value_clone(receiver.as.exception->trace);
    }
    (void)runtime;
    (void)line;
    return ptn_array_from_literal_entries(0, NULL);
}

static PTN_UNUSED PtnValue ptn_throwable_trace_string(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    (void)line;
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        PtnStringOperand trace = ptn_exception_trace_as_string_operand(
            runtime,
            receiver.as.exception
        );
        return ptn_owned_string_len(trace.owned, trace.len);
    }
    return ptn_owned_string(ptn_duplicate_string("#0 {main}"));
}

static PTN_UNUSED PtnValue ptn_throwable_to_string(PtnRuntime *runtime, PtnValue receiver, size_t line) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_EXCEPTION) {
        PtnStringOperand text = ptn_exception_to_string_operand(
            runtime,
            receiver.as.exception
        );
        return ptn_owned_string_len(text.owned, text.len);
    }
    const char *class_name = receiver.type == PTN_OBJECT ? receiver.as.object->class_name : "Exception";
    char *message = ptn_throwable_message_string(runtime, receiver, line);
    PtnValue file_value = ptn_throwable_file_value(runtime, receiver, line);
    char *file = ptn_value_to_string(file_value);
    ptn_value_destroy(&file_value);
    int64_t throwable_line = ptn_throwable_int_property(runtime, receiver, "line", 0, line);
    int needed = snprintf(
        NULL,
        0,
        "%s: %s in %s:%lld\nStack trace:\n#0 {main}",
        class_name,
        message,
        file,
        (long long)throwable_line
    );
    if (needed < 0) {
        free(file);
        free(message);
        ptn_abort_out_of_memory();
    }
    char *result = malloc((size_t)needed + 1);
    if (result == NULL) {
        free(file);
        free(message);
        ptn_abort_out_of_memory();
    }
    snprintf(
        result,
        (size_t)needed + 1,
        "%s: %s in %s:%lld\nStack trace:\n#0 {main}",
        class_name,
        message,
        file,
        (long long)throwable_line
    );
    free(file);
    free(message);
    return ptn_owned_string(result);
}

static PTN_UNUSED PtnValue ptn_throw_value(
    PtnRuntime *runtime,
    PtnValue value,
    const char *path,
    size_t line
) {
    PtnValue resolved = ptn_value_deref(value);
    if (resolved.type == PTN_OBJECT && ptn_object_is_declared_throwable(runtime, resolved.as.object)) {
        PtnStringOperand message = ptn_object_exception_message(runtime, resolved, line);
        int64_t code = ptn_throwable_int_property(runtime, resolved, "code", 0, line);
        int64_t severity = ptn_throwable_int_property(runtime, resolved, "severity", PTN_E_ERROR, line);
        PtnValue previous = ptn_throwable_previous_value(runtime, resolved, line);
        int chains_active_generator_exception =
            runtime != NULL &&
            runtime->current_generator != NULL &&
            runtime->exceptions != NULL &&
            runtime->exceptions->active_exception != NULL &&
            ptn_value_deref(previous).type == PTN_NULL;
        if (chains_active_generator_exception) {
            runtime->generator_chained_exception_during_unwind = 1;
            if (runtime->generator_aborted_after_yield) {
                runtime->generator_aborted_rethrow_on_rewind = 1;
            }
        }
        PtnValue chained_previous = ptn_exception_previous_or_active(runtime, previous);
        PtnValue file_value = ptn_throwable_file_value(runtime, resolved, line);
        char *exception_path = ptn_value_to_string(file_value);
        ptn_value_destroy(&file_value);
        int64_t stored_line = ptn_throwable_int_property(runtime, resolved, "line", (int64_t)line, line);
        ptn_exception_free(runtime->exceptions->active_exception);
        runtime->exceptions->active_exception = ptn_exception_new_owned(
            runtime,
            resolved.as.object->class_name,
            message.owned,
            message.len,
            code,
            chained_previous,
            severity,
            exception_path,
            stored_line < 0 ? line : (size_t)stored_line
        );
        if (
            runtime->exceptions->try_frame == NULL &&
            runtime->declared_method_exists != NULL &&
            runtime->declared_method_exists(resolved.as.object->class_name, "__toString")
        ) {
            PtnStringOperand uncaught_text;
            if (ptn_try_object_to_string_operand(runtime, resolved, line, &uncaught_text)) {
                runtime->exceptions->active_exception->uncaught_text_len = uncaught_text.len;
                if (uncaught_text.owned != NULL) {
                    runtime->exceptions->active_exception->uncaught_text = uncaught_text.owned;
                } else {
                    runtime->exceptions->active_exception->uncaught_text =
                        ptn_duplicate_string_len(uncaught_text.data, uncaught_text.len);
                }
            }
        }
        ptn_value_destroy(&previous);
        if (runtime->exceptions->try_frame != NULL) {
            longjmp(runtime->exceptions->try_frame->jump, 1);
        }
        ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
        ptn_runtime_shutdown_before_exit(runtime);
        exit(255);
        return ptn_null();
    }
    if (resolved.type != PTN_EXCEPTION) {
        ptn_throw_exception_at(runtime, "Error", "Can only throw objects", path, line);
        return ptn_null();
    }
    int chains_active_generator_exception =
        runtime != NULL &&
        runtime->current_generator != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL &&
        ptn_value_deref(resolved.as.exception->previous).type == PTN_NULL;
    if (chains_active_generator_exception) {
        runtime->generator_chained_exception_during_unwind = 1;
        if (runtime->generator_aborted_after_yield) {
            runtime->generator_aborted_rethrow_on_rewind = 1;
        }
    }
    PtnValue chained_previous = ptn_exception_previous_or_active(
        runtime,
        resolved.as.exception->previous
    );
    PtnValue chained_previous_resolved = ptn_value_deref(chained_previous);
    if (
        resolved.as.exception != runtime->exceptions->active_exception &&
        resolved.as.exception->previous.type == PTN_NULL &&
        chained_previous_resolved.type == PTN_EXCEPTION &&
        !ptn_exception_previous_chain_would_recurse(
            chained_previous_resolved.as.exception,
            resolved.as.exception
        )
    ) {
        ptn_value_destroy(&resolved.as.exception->previous);
        resolved.as.exception->previous = ptn_value_clone_deref(chained_previous);
    }
    ptn_exception_free(runtime->exceptions->active_exception);
    runtime->exceptions->active_exception = resolved.as.exception;
    ptn_exception_retain(runtime->exceptions->active_exception);
    if (runtime->exceptions->active_exception->path == NULL) {
        runtime->exceptions->active_exception->path = path;
        runtime->exceptions->active_exception->line = line;
    }
    ptn_value_destroy(&value);
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_emit_uncaught_exception(runtime, runtime->exceptions->active_exception);
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
    return ptn_null();
}

static PTN_UNUSED void ptn_throw_user_argument_count_error(
    PtnRuntime *callee_runtime,
    PtnRuntime *caller_runtime,
    const char *function_name,
    size_t expected,
    size_t passed,
    int exactly,
    size_t line,
    const char *declaration_path,
    size_t declaration_line
) {
    PtnRuntime *message_runtime = caller_runtime != NULL ? caller_runtime : callee_runtime;
    if (
        message_runtime != NULL &&
        (
            message_runtime->suppress_user_call_frame_location ||
            message_runtime->suppress_user_argument_count_location
        )
    ) {
        line = 0;
    }
    const char *mode = exactly ? "exactly" : "at least";
    const char *path =
        message_runtime != NULL && message_runtime->source_path != NULL
            ? message_runtime->source_path
            : "ptn";
    if (message_runtime != NULL && message_runtime->suppress_user_call_frame_location) {
        line = 0;
    }
    int needed = line == 0
        ? snprintf(
            NULL,
            0,
            "Too few arguments to function %s(), %zu passed and %s %zu expected",
            function_name,
            passed,
            mode,
            expected
        )
        : snprintf(
            NULL,
            0,
            "Too few arguments to function %s(), %zu passed in %s on line %zu and %s %zu expected",
            function_name,
            passed,
            path,
            line,
            mode,
            expected
        );
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (line == 0) {
        snprintf(
            message,
            (size_t)needed + 1,
            "Too few arguments to function %s(), %zu passed and %s %zu expected",
            function_name,
            passed,
            mode,
            expected
        );
    } else {
        snprintf(
            message,
            (size_t)needed + 1,
            "Too few arguments to function %s(), %zu passed in %s on line %zu and %s %zu expected",
            function_name,
            passed,
            path,
            line,
            mode,
            expected
        );
    }
    ptn_throw_exception_owned_message_at(
        callee_runtime,
        "ArgumentCountError",
        message,
        declaration_path,
        declaration_line
    );
}

static PTN_UNUSED void ptn_throw_user_missing_argument_error(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *parameter_name
) {
    int needed = snprintf(
        NULL,
        0,
        "%s(): Argument #%zu ($%s) not passed",
        function_name,
        position,
        parameter_name
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
        "%s(): Argument #%zu ($%s) not passed",
        function_name,
        position,
        parameter_name
    );
    ptn_throw_exception_owned_message(runtime, "ArgumentCountError", message);
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_table(PtnRuntime *runtime) {
    return runtime->static_properties == NULL ? &runtime->owned_static_properties : runtime->static_properties;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_initialized_table(PtnRuntime *runtime) {
    return runtime->static_property_initialized == NULL
        ? &runtime->owned_static_property_initialized
        : runtime->static_property_initialized;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_read_visibility_table(PtnRuntime *runtime) {
    return runtime->static_property_read_visibility == NULL
        ? &runtime->owned_static_property_read_visibility
        : runtime->static_property_read_visibility;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_set_visibility_table(PtnRuntime *runtime) {
    return runtime->static_property_set_visibility == NULL
        ? &runtime->owned_static_property_set_visibility
        : runtime->static_property_set_visibility;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_type_kind_table(PtnRuntime *runtime) {
    return runtime->static_property_type_kind == NULL
        ? &runtime->owned_static_property_type_kind
        : runtime->static_property_type_kind;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_type_class_name_table(PtnRuntime *runtime) {
    return runtime->static_property_type_class_name == NULL
        ? &runtime->owned_static_property_type_class_name
        : runtime->static_property_type_class_name;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_type_text_table(PtnRuntime *runtime) {
    return runtime->static_property_type_text == NULL
        ? &runtime->owned_static_property_type_text
        : runtime->static_property_type_text;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_static_property_type_allows_null_table(PtnRuntime *runtime) {
    return runtime->static_property_type_allows_null == NULL
        ? &runtime->owned_static_property_type_allows_null
        : runtime->static_property_type_allows_null;
}

static PTN_UNUSED int ptn_runtime_static_property_declaration_exists(
    PtnRuntime *runtime,
    const char *key
) {
    return ptn_symbols_value_slot(ptn_runtime_static_property_table(runtime), key) != NULL ||
        ptn_symbols_value_slot(ptn_runtime_static_property_initialized_table(runtime), key) != NULL ||
        ptn_symbols_value_slot(ptn_runtime_static_property_read_visibility_table(runtime), key) != NULL ||
        ptn_symbols_value_slot(ptn_runtime_static_property_set_visibility_table(runtime), key) != NULL ||
        ptn_symbols_value_slot(ptn_runtime_static_property_type_kind_table(runtime), key) != NULL;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_class_constant_table(PtnRuntime *runtime) {
    return runtime->class_constants == NULL ? &runtime->owned_class_constants : runtime->class_constants;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_class_constant_deprecation_table(PtnRuntime *runtime) {
    return runtime->class_constant_deprecations == NULL
        ? &runtime->owned_class_constant_deprecations
        : runtime->class_constant_deprecations;
}

static PTN_UNUSED PtnSymbolTable *ptn_runtime_class_constant_initializing_table(PtnRuntime *runtime) {
    return runtime->class_constant_initializing == NULL
        ? &runtime->owned_class_constant_initializing
        : runtime->class_constant_initializing;
}

static PTN_UNUSED int ptn_property_class_names_equal(const char *left, const char *right) {
    if (left == NULL || right == NULL) {
        return 0;
    }
    while (*left != '\0' && *right != '\0') {
        if (tolower((unsigned char)*left) != tolower((unsigned char)*right)) {
            return 0;
        }
        left++;
        right++;
    }
    return *left == '\0' && *right == '\0';
}

static PTN_UNUSED int ptn_property_visibility_allows(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *access_scope
) {
    if (visibility == PTN_PROPERTY_PUBLIC) {
        return 1;
    }
    if (access_scope == NULL || declaring_class == NULL) {
        return 0;
    }
    if (visibility == PTN_PROPERTY_PRIVATE) {
        return ptn_property_class_names_equal(access_scope, declaring_class);
    }
    if (runtime->class_scope_allows != NULL) {
        return runtime->class_scope_allows(access_scope, declaring_class);
    }
    return ptn_property_class_names_equal(access_scope, declaring_class);
}

static PTN_UNUSED const char *ptn_property_visibility_name(PtnPropertyVisibility visibility) {
    if (visibility == PTN_PROPERTY_PRIVATE) {
        return "private";
    }
    if (visibility == PTN_PROPERTY_PROTECTED) {
        return "protected";
    }
    return "public";
}

static PTN_UNUSED const char *ptn_static_property_visibility_error_class(
    PtnPropertyVisibility visibility,
    const char *lookup_class,
    const char *declaring_class
) {
    if (
        visibility == PTN_PROPERTY_PRIVATE &&
        lookup_class != NULL &&
        declaring_class != NULL &&
        !ptn_property_class_names_equal(lookup_class, declaring_class)
    ) {
        return lookup_class;
    }
    return declaring_class;
}

static PTN_UNUSED void ptn_throw_property_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot access %s property %s::$%s",
        ptn_property_visibility_name(visibility),
        declaring_class,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_class_constant_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *class_name,
    const char *constant,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Cannot access %s constant %s::%s",
        ptn_property_visibility_name(visibility),
        class_name,
        constant
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_property_set_visibility_error_ex(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope,
    int is_readonly
) {
    char message[320];
    const char *scope = access_scope == NULL ? "global scope" : access_scope;
    const char *readonly = is_readonly ? " readonly" : "";
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot modify %s(set)%s property %s::$%s from %s",
            ptn_property_visibility_name(visibility),
            readonly,
            declaring_class,
            property,
            scope
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot modify %s(set)%s property %s::$%s from scope %s",
            ptn_property_visibility_name(visibility),
            readonly,
            declaring_class,
            property,
            scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_property_set_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope
) {
    ptn_throw_property_set_visibility_error_ex(
        runtime,
        visibility,
        declaring_class,
        property,
        access_scope,
        0
    );
}

static PTN_UNUSED void ptn_throw_readonly_property_set_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope
) {
    ptn_throw_property_set_visibility_error_ex(
        runtime,
        visibility,
        declaring_class,
        property,
        access_scope,
        1
    );
}

static PTN_UNUSED void ptn_throw_property_indirect_set_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope
) {
    char message[320];
    const char *scope = access_scope == NULL ? "global scope" : access_scope;
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot indirectly modify %s(set) property %s::$%s from %s",
            ptn_property_visibility_name(visibility),
            declaring_class,
            property,
            scope
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot indirectly modify %s(set) property %s::$%s from scope %s",
            ptn_property_visibility_name(visibility),
            declaring_class,
            property,
            scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED void ptn_throw_property_unset_visibility_error(
    PtnRuntime *runtime,
    PtnPropertyVisibility visibility,
    const char *declaring_class,
    const char *property,
    const char *access_scope,
    int asymmetric_set_visibility,
    int is_readonly
) {
    char message[320];
    const char *scope = access_scope == NULL ? "global scope" : access_scope;
    const char *set_suffix = asymmetric_set_visibility ? "(set)" : "";
    const char *readonly_suffix = is_readonly ? " readonly" : "";
    int written;
    if (access_scope == NULL) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot unset %s%s%s property %s::$%s from %s",
            ptn_property_visibility_name(visibility),
            set_suffix,
            readonly_suffix,
            declaring_class,
            property,
            scope
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot unset %s%s%s property %s::$%s from scope %s",
            ptn_property_visibility_name(visibility),
            set_suffix,
            readonly_suffix,
            declaring_class,
            property,
            scope
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception(runtime, "Error", message);
}

static PTN_UNUSED char *ptn_static_property_key(const char *class_name, const char *property) {
    size_t class_len = strlen(class_name);
    size_t property_len = strlen(property);
    size_t len = class_len + property_len + 4;
    char *key = malloc(len);
    if (key == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(key, class_name, class_len);
    memcpy(key + class_len, "::$", 3);
    memcpy(key + class_len + 3, property, property_len);
    key[len - 1] = '\0';
    return key;
}

static PTN_UNUSED char *ptn_runtime_resolve_static_property_key(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char **declaring_class_out
) {
    const char *lookup_class_name = class_name;
    while (lookup_class_name != NULL) {
        char *key = ptn_static_property_key(lookup_class_name, property);
        if (ptn_runtime_static_property_declaration_exists(runtime, key)) {
            if (declaring_class_out != NULL) {
                *declaring_class_out = lookup_class_name;
            }
            return key;
        }
        free(key);
        lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
    }
    if (declaring_class_out != NULL) {
        *declaring_class_out = NULL;
    }
    return NULL;
}

static PTN_UNUSED char *ptn_class_constant_key(const char *class_name, const char *constant) {
    size_t class_len = strlen(class_name);
    size_t constant_len = strlen(constant);
    size_t len = class_len + constant_len + 3;
    char *key = malloc(len);
    if (key == NULL) {
        ptn_abort_out_of_memory();
    }
    memcpy(key, class_name, class_len);
    memcpy(key + class_len, "::", 2);
    memcpy(key + class_len + 2, constant, constant_len);
    key[len - 1] = '\0';
    return key;
}

static PTN_UNUSED char ptn_ascii_lower_char(char ch) {
    return (ch >= 'A' && ch <= 'Z') ? (char)(ch - 'A' + 'a') : ch;
}

static PTN_UNUSED const char *ptn_symbol_name_without_leading_slash(const char *name) {
    return name[0] == '\\' ? name + 1 : name;
}

static char *ptn_class_alias_key(const char *class_name) {
    const char *lookup_name = ptn_symbol_name_without_leading_slash(class_name);
    size_t len = strlen(lookup_name);
    char *key = malloc(len + 1);
    if (key == NULL) {
        ptn_abort_out_of_memory();
    }
    for (size_t i = 0; i < len; i++) {
        key[i] = ptn_ascii_lower_char(lookup_name[i]);
    }
    key[len] = '\0';
    return key;
}

static PtnSymbolTable *ptn_runtime_class_alias_table(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root != NULL && root->class_aliases != NULL) {
        return root->class_aliases;
    }
    return runtime == NULL ? NULL : runtime->class_aliases;
}

static PtnSymbolTable *ptn_runtime_dynamic_class_table(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root != NULL && root->dynamic_classes != NULL) {
        return root->dynamic_classes;
    }
    return runtime == NULL ? NULL : runtime->dynamic_classes;
}

static PTN_UNUSED int ptn_runtime_dynamic_class_exists(PtnRuntime *runtime, const char *class_name) {
    if (class_name == NULL) {
        return 0;
    }
    PtnSymbolTable *classes = ptn_runtime_dynamic_class_table(runtime);
    if (classes == NULL) {
        return 0;
    }
    char *key = ptn_class_alias_key(class_name);
    PtnValue marker;
    int found = ptn_symbols_get(classes, key, &marker);
    free(key);
    return found;
}

static PtnValue ptn_runtime_dynamic_class_marker_field(PtnValue marker, const char *name) {
    marker = ptn_value_deref(marker);
    if (marker.type != PTN_ARRAY || marker.as.array == NULL) {
        return ptn_null();
    }
    size_t name_len = strlen(name);
    for (size_t i = 0; i < marker.as.array->len; i++) {
        PtnArrayEntry *entry = &marker.as.array->entries[i];
        if (entry->key.type == PTN_ARRAY_KEY_STRING &&
            entry->key.string_len == name_len &&
            memcmp(entry->key.as.string, name, name_len) == 0) {
            return ptn_value_deref(entry->value);
        }
    }
    return ptn_null();
}

static PTN_UNUSED void ptn_runtime_register_dynamic_class_ex(
    PtnRuntime *runtime,
    const char *class_name,
    const char *parent_name,
    int allows_dynamic_properties,
    PtnValue interfaces
) {
    if (class_name == NULL || *class_name == '\0') {
        return;
    }
    PtnSymbolTable *classes = ptn_runtime_dynamic_class_table(runtime);
    if (classes == NULL) {
        return;
    }
    char *key = ptn_class_alias_key(class_name);
    PtnValue parent_value = parent_name == NULL || *parent_name == '\0'
        ? ptn_null()
        : ptn_owned_string(ptn_duplicate_string(parent_name));
    PtnValue interfaces_value = ptn_value_deref(interfaces);
    int owns_interfaces_value = 0;
    if (interfaces_value.type != PTN_ARRAY) {
        interfaces_value = ptn_array_from_literal_entries(0, NULL);
        owns_interfaces_value = 1;
    }
    PtnArrayLiteralEntry entries[] = {
        { 1, ptn_string("parent"), parent_value },
        { 1, ptn_string("allow_dynamic"), ptn_bool(allows_dynamic_properties) },
        { 1, ptn_string("interfaces"), interfaces_value },
    };
    PtnValue marker = ptn_array_from_literal_entries(3, entries);
    ptn_symbols_set(classes, key, marker);
    ptn_value_destroy(&marker);
    if (owns_interfaces_value) {
        ptn_value_destroy(&interfaces_value);
    }
    ptn_value_destroy(&parent_value);
    free(key);
}

static PTN_UNUSED void ptn_runtime_register_dynamic_class(PtnRuntime *runtime, const char *class_name) {
    PtnValue interfaces = ptn_array_from_literal_entries(0, NULL);
    ptn_runtime_register_dynamic_class_ex(runtime, class_name, NULL, 0, interfaces);
    ptn_value_destroy(&interfaces);
}

static PTN_UNUSED void ptn_runtime_register_dynamic_class_with_parent(
    PtnRuntime *runtime,
    const char *class_name,
    const char *parent_name
) {
    PtnValue interfaces = ptn_array_from_literal_entries(0, NULL);
    ptn_runtime_register_dynamic_class_ex(runtime, class_name, parent_name, 0, interfaces);
    ptn_value_destroy(&interfaces);
}

static PTN_UNUSED const char *ptn_runtime_dynamic_class_parent_name(
    PtnRuntime *runtime,
    const char *class_name
) {
    PtnSymbolTable *classes = ptn_runtime_dynamic_class_table(runtime);
    if (classes == NULL || class_name == NULL) {
        return NULL;
    }
    char *key = ptn_class_alias_key(class_name);
    PtnValue parent;
    int found = ptn_symbols_get(classes, key, &parent);
    free(key);
    if (!found) {
        return NULL;
    }
    parent = ptn_value_deref(parent);
    if (parent.type == PTN_STRING) {
        return (const char *)parent.as.string.data;
    }
    PtnValue parent_field = ptn_runtime_dynamic_class_marker_field(parent, "parent");
    return parent_field.type == PTN_STRING ? (const char *)parent_field.as.string.data : NULL;
}

static PTN_UNUSED PtnValue ptn_runtime_dynamic_class_interfaces(
    PtnRuntime *runtime,
    const char *class_name
) {
    PtnSymbolTable *classes = ptn_runtime_dynamic_class_table(runtime);
    if (classes == NULL || class_name == NULL) {
        return ptn_null();
    }
    char *key = ptn_class_alias_key(class_name);
    PtnValue marker;
    int found = ptn_symbols_get(classes, key, &marker);
    free(key);
    if (!found) {
        return ptn_null();
    }
    marker = ptn_value_deref(marker);
    if (marker.type != PTN_ARRAY) {
        return ptn_null();
    }
    return ptn_runtime_dynamic_class_marker_field(marker, "interfaces");
}

static PTN_UNUSED int ptn_runtime_dynamic_class_allows_dynamic_properties_depth(
    PtnRuntime *runtime,
    const char *class_name,
    size_t depth
) {
    if (class_name == NULL || depth > 64) {
        return 0;
    }
    PtnSymbolTable *classes = ptn_runtime_dynamic_class_table(runtime);
    if (classes == NULL) {
        return 0;
    }
    char *key = ptn_class_alias_key(class_name);
    PtnValue marker;
    int found = ptn_symbols_get(classes, key, &marker);
    free(key);
    if (!found) {
        return 0;
    }
    marker = ptn_value_deref(marker);
    if (marker.type != PTN_ARRAY) {
        return 0;
    }
    if (ptn_is_truthy(ptn_runtime_dynamic_class_marker_field(marker, "allow_dynamic"))) {
        return 1;
    }
    PtnValue parent = ptn_runtime_dynamic_class_marker_field(marker, "parent");
    if (parent.type != PTN_STRING) {
        return 0;
    }
    const char *parent_name = (const char *)parent.as.string.data;
    if (ptn_ascii_case_equal(parent_name, "stdClass")) {
        return 1;
    }
    if (ptn_runtime_dynamic_class_allows_dynamic_properties_depth(runtime, parent_name, depth + 1)) {
        return 1;
    }
    return runtime != NULL &&
        runtime->declared_class_allows_dynamic_properties != NULL &&
        runtime->declared_class_allows_dynamic_properties(parent_name);
}

static PTN_UNUSED int ptn_runtime_dynamic_class_allows_dynamic_properties(
    PtnRuntime *runtime,
    const char *class_name
) {
    return ptn_runtime_dynamic_class_allows_dynamic_properties_depth(runtime, class_name, 0);
}

static PTN_UNUSED const char *ptn_runtime_declared_class_parent_name(
    PtnRuntime *runtime,
    const char *class_name
) {
    const char *dynamic_parent = ptn_runtime_dynamic_class_parent_name(runtime, class_name);
    return dynamic_parent != NULL ? dynamic_parent : ptn_declared_class_parent_name(class_name);
}

static PTN_UNUSED int ptn_runtime_declared_class_is_same_or_descendant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *ancestor_name
) {
    if (class_name == NULL || ancestor_name == NULL) {
        return 0;
    }
    const char *current = class_name;
    for (size_t depth = 0; current != NULL && depth < 64; depth++) {
        if (ptn_declared_class_is_same_or_descendant(current, ancestor_name)) {
            return 1;
        }
        current = ptn_runtime_dynamic_class_parent_name(runtime, current);
    }
    return 0;
}

static PTN_UNUSED const char *ptn_runtime_resolve_class_alias(
    PtnRuntime *runtime,
    const char *class_name
) {
    if (class_name == NULL) {
        return NULL;
    }
    const char *current = ptn_symbol_name_without_leading_slash(class_name);
    PtnSymbolTable *aliases = ptn_runtime_class_alias_table(runtime);
    if (aliases == NULL) {
        return current;
    }

    for (size_t depth = 0; depth < 32; depth++) {
        char *key = ptn_class_alias_key(current);
        PtnValue alias_value;
        int found = ptn_symbols_get(aliases, key, &alias_value);
        free(key);
        if (!found) {
            return current;
        }

        alias_value = ptn_value_deref(alias_value);
        if (alias_value.type != PTN_STRING) {
            return current;
        }
        const char *next = (const char *)alias_value.as.string.data;
        if (ptn_ascii_case_equal(current, next)) {
            return current;
        }
        current = next;
    }

    return current;
}

static PTN_UNUSED int ptn_ascii_case_equal_span_to_string(
    const char *left,
    size_t left_len,
    const char *right
) {
    size_t right_len = strlen(right);
    if (left_len != right_len) {
        return 0;
    }
    for (size_t i = 0; i < left_len; i++) {
        if (ptn_ascii_lower_char(left[i]) != ptn_ascii_lower_char(right[i])) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED const char *ptn_rounding_mode_case_name(const char *case_name) {
    static const char *const names[] = {
        "HalfAwayFromZero",
        "HalfTowardsZero",
        "HalfEven",
        "HalfOdd",
        "TowardsZero",
        "AwayFromZero",
        "NegativeInfinity",
        "PositiveInfinity",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_random_interval_boundary_case_name(const char *case_name) {
    static const char *const names[] = {
        "ClosedOpen",
        "ClosedClosed",
        "OpenClosed",
        "OpenOpen",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_stream_error_code_case_name(const char *case_name) {
    static const char *const names[] = {
        "None",
        "Generic",
        "ReadFailed",
        "WriteFailed",
        "SeekFailed",
        "SeekNotSupported",
        "FlushFailed",
        "TruncateFailed",
        "ConnectFailed",
        "BindFailed",
        "ListenFailed",
        "NotWritable",
        "NotReadable",
        "Disabled",
        "NotFound",
        "PermissionDenied",
        "AlreadyExists",
        "InvalidPath",
        "PathTooLong",
        "OpenFailed",
        "CreateFailed",
        "DupFailed",
        "UnlinkFailed",
        "RenameFailed",
        "MkdirFailed",
        "RmdirFailed",
        "StatFailed",
        "MetaFailed",
        "ChmodFailed",
        "ChownFailed",
        "CopyFailed",
        "TouchFailed",
        "InvalidMode",
        "InvalidMeta",
        "ModeNotSupported",
        "Readonly",
        "RecursionDetected",
        "NotImplemented",
        "NoOpener",
        "PersistentNotSupported",
        "WrapperNotFound",
        "WrapperDisabled",
        "ProtocolUnsupported",
        "WrapperRegistrationFailed",
        "WrapperUnregistrationFailed",
        "WrapperRestorationFailed",
        "FilterNotFound",
        "FilterFailed",
        "CastFailed",
        "CastNotSupported",
        "MakeSeekableFailed",
        "BufferedDataLost",
        "NetworkSendFailed",
        "NetworkRecvFailed",
        "SslNotSupported",
        "ResumptionFailed",
        "SocketPathTooLong",
        "OobNotSupported",
        "ProtocolError",
        "InvalidUrl",
        "InvalidResponse",
        "InvalidHeader",
        "InvalidParam",
        "RedirectLimit",
        "AuthFailed",
        "ArchivingFailed",
        "EncodingFailed",
        "DecodingFailed",
        "InvalidFormat",
        "AllocationFailed",
        "TemporaryFileFailed",
        "LockFailed",
        "LockNotSupported",
        "UserspaceNotImplemented",
        "UserspaceInvalidReturn",
        "UserspaceCallFailed",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_stream_error_mode_case_name(const char *case_name) {
    static const char *const names[] = {
        "Error",
        "Exception",
        "Silent",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_stream_error_store_case_name(const char *case_name) {
    static const char *const names[] = {
        "Auto",
        "None",
        "NonTerminating",
        "Terminating",
        "All",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_uri_url_host_type_case_name(const char *case_name) {
    static const char *const names[] = {
        "Domain",
        "IPv4",
        "IPv6",
        "Opaque",
        "Empty",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_uri_type_case_name(const char *case_name) {
    static const char *const names[] = {
        "AbsolutePathReference",
        "RelativePathReference",
        "NetworkPathReference",
        "Uri",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_uri_comparison_mode_case_name(const char *case_name) {
    if (strcmp(case_name, "IncludeFragment") == 0) {
        return "IncludeFragment";
    }
    if (strcmp(case_name, "ExcludeFragment") == 0) {
        return "ExcludeFragment";
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_uri_url_validation_error_type_case_name(const char *case_name) {
    static const char *const names[] = {
        "DomainInvalidCodePoint",
        "HostMissing",
        "HostInvalidCodePoint",
        "InvalidCredentials",
        "InvalidReverseSolidus",
        "InvalidUrlUnit",
        "PortInvalid",
        "PortOutOfRange",
        "MissingSchemeNonRelativeUrl",
        "Ipv4EmptyPart",
        "Ipv4TooManyParts",
        "Ipv4NonNumericPart",
        "Ipv4NonDecimalPart",
        "Ipv4OutOfRangePart",
        "Ipv6Unclosed",
        "Ipv6InvalidCompression",
        "Ipv6TooManyPieces",
        "Ipv6MultipleCompression",
        "Ipv6InvalidCodePoint",
        "Ipv6TooFewPieces",
        "Ipv4InIpv6TooManyPieces",
        "Ipv4InIpv6InvalidCodePoint",
        "Ipv4InIpv6OutOfRangePart",
        "Ipv4InIpv6TooFewParts",
    };
    for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); i++) {
        if (strcmp(case_name, names[i]) == 0) {
            return names[i];
        }
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_property_hook_type_case_name(const char *case_name) {
    if (strcmp(case_name, "Get") == 0) {
        return "Get";
    }
    if (strcmp(case_name, "Set") == 0) {
        return "Set";
    }
    return NULL;
}

static PTN_UNUSED const char *ptn_dom_adjacent_position_case_name(const char *case_name) {
    if (strcmp(case_name, "BeforeBegin") == 0) {
        return "BeforeBegin";
    }
    if (strcmp(case_name, "AfterBegin") == 0) {
        return "AfterBegin";
    }
    if (strcmp(case_name, "BeforeEnd") == 0) {
        return "BeforeEnd";
    }
    if (strcmp(case_name, "AfterEnd") == 0) {
        return "AfterEnd";
    }
    return NULL;
}

static PTN_UNUSED int ptn_builtin_class_constant_value_span(
    const char *class_name,
    size_t class_len,
    const char *constant,
    PtnValue *out
) {
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ArrayObject")) {
        if (strcmp(constant, "STD_PROP_LIST") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "ARRAY_AS_PROPS") == 0) {
            *out = ptn_int(2);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "DatePeriod")) {
        if (strcmp(constant, "EXCLUDE_START_DATE") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "INCLUDE_END_DATE") == 0) {
            *out = ptn_int(2);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "DirectoryIterator") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "FilesystemIterator") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "GlobIterator") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "RecursiveDirectoryIterator")) {
        if (strcmp(constant, "CURRENT_AS_FILEINFO") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "CURRENT_AS_SELF") == 0) {
            *out = ptn_int(16);
            return 1;
        }
        if (strcmp(constant, "CURRENT_AS_PATHNAME") == 0) {
            *out = ptn_int(32);
            return 1;
        }
        if (strcmp(constant, "CURRENT_MODE_MASK") == 0) {
            *out = ptn_int(240);
            return 1;
        }
        if (strcmp(constant, "KEY_AS_PATHNAME") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "KEY_AS_FILENAME") == 0) {
            *out = ptn_int(256);
            return 1;
        }
        if (strcmp(constant, "FOLLOW_SYMLINKS") == 0) {
            *out = ptn_int(512);
            return 1;
        }
        if (strcmp(constant, "KEY_MODE_MASK") == 0) {
            *out = ptn_int(3840);
            return 1;
        }
        if (strcmp(constant, "NEW_CURRENT_AND_KEY") == 0) {
            *out = ptn_int(256);
            return 1;
        }
        if (strcmp(constant, "SKIP_DOTS") == 0) {
            *out = ptn_int(4096);
            return 1;
        }
        if (strcmp(constant, "UNIX_PATHS") == 0) {
            *out = ptn_int(8192);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "XMLReader")) {
        if (strcmp(constant, "NONE") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "ELEMENT") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "ATTRIBUTE") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "TEXT") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "CDATA") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "ENTITY_REF") == 0) { *out = ptn_int(5); return 1; }
        if (strcmp(constant, "ENTITY") == 0) { *out = ptn_int(6); return 1; }
        if (strcmp(constant, "PI") == 0) { *out = ptn_int(7); return 1; }
        if (strcmp(constant, "COMMENT") == 0) { *out = ptn_int(8); return 1; }
        if (strcmp(constant, "DOC") == 0) { *out = ptn_int(9); return 1; }
        if (strcmp(constant, "DOC_TYPE") == 0) { *out = ptn_int(10); return 1; }
        if (strcmp(constant, "DOC_FRAGMENT") == 0) { *out = ptn_int(11); return 1; }
        if (strcmp(constant, "NOTATION") == 0) { *out = ptn_int(12); return 1; }
        if (strcmp(constant, "WHITESPACE") == 0) { *out = ptn_int(13); return 1; }
        if (strcmp(constant, "SIGNIFICANT_WHITESPACE") == 0) { *out = ptn_int(14); return 1; }
        if (strcmp(constant, "END_ELEMENT") == 0) { *out = ptn_int(15); return 1; }
        if (strcmp(constant, "END_ENTITY") == 0) { *out = ptn_int(16); return 1; }
        if (strcmp(constant, "XML_DECLARATION") == 0) { *out = ptn_int(17); return 1; }
        if (strcmp(constant, "LOADDTD") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "DEFAULTATTRS") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "VALIDATE") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "SUBST_ENTITIES") == 0) { *out = ptn_int(4); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "DOMNode") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "Dom\\Node")) {
        if (strcmp(constant, "DOCUMENT_POSITION_DISCONNECTED") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "DOCUMENT_POSITION_PRECEDING") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "DOCUMENT_POSITION_FOLLOWING") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "DOCUMENT_POSITION_CONTAINS") == 0) { *out = ptn_int(8); return 1; }
        if (strcmp(constant, "DOCUMENT_POSITION_CONTAINED_BY") == 0) { *out = ptn_int(16); return 1; }
        if (strcmp(constant, "DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC") == 0) { *out = ptn_int(32); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "PDO") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "Pdo\\Sqlite") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "Pdo\\Mysql") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "Pdo\\Pgsql")) {
        if (strcmp(constant, "PARAM_NULL") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "PARAM_INT") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "PARAM_STR") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "PARAM_LOB") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "PARAM_STMT") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "PARAM_BOOL") == 0) { *out = ptn_int(5); return 1; }
        if (strcmp(constant, "PARAM_INPUT_OUTPUT") == 0) { *out = ptn_int(0x80000000LL); return 1; }
        if (strcmp(constant, "FETCH_DEFAULT") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "FETCH_LAZY") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "FETCH_ASSOC") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "FETCH_NUM") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "FETCH_BOTH") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "FETCH_OBJ") == 0) { *out = ptn_int(5); return 1; }
        if (strcmp(constant, "FETCH_BOUND") == 0) { *out = ptn_int(6); return 1; }
        if (strcmp(constant, "FETCH_COLUMN") == 0) { *out = ptn_int(7); return 1; }
        if (strcmp(constant, "FETCH_CLASS") == 0) { *out = ptn_int(8); return 1; }
        if (strcmp(constant, "FETCH_INTO") == 0) { *out = ptn_int(9); return 1; }
        if (strcmp(constant, "FETCH_FUNC") == 0) { *out = ptn_int(10); return 1; }
        if (strcmp(constant, "FETCH_NAMED") == 0) { *out = ptn_int(11); return 1; }
        if (strcmp(constant, "FETCH_KEY_PAIR") == 0) { *out = ptn_int(12); return 1; }
        if (strcmp(constant, "FETCH_GROUP") == 0) { *out = ptn_int(65536); return 1; }
        if (strcmp(constant, "FETCH_UNIQUE") == 0) { *out = ptn_int(196608); return 1; }
        if (strcmp(constant, "FETCH_CLASSTYPE") == 0) { *out = ptn_int(262144); return 1; }
        if (strcmp(constant, "FETCH_SERIALIZE") == 0) { *out = ptn_int(524288); return 1; }
        if (strcmp(constant, "FETCH_PROPS_LATE") == 0) { *out = ptn_int(1048576); return 1; }
        if (strcmp(constant, "ATTR_AUTOCOMMIT") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "ATTR_PREFETCH") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "ATTR_TIMEOUT") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "ATTR_ERRMODE") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "ATTR_SERVER_VERSION") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "ATTR_CLIENT_VERSION") == 0) { *out = ptn_int(5); return 1; }
        if (strcmp(constant, "ATTR_SERVER_INFO") == 0) { *out = ptn_int(6); return 1; }
        if (strcmp(constant, "ATTR_CONNECTION_STATUS") == 0) { *out = ptn_int(7); return 1; }
        if (strcmp(constant, "ATTR_CASE") == 0) { *out = ptn_int(8); return 1; }
        if (strcmp(constant, "ATTR_CURSOR_NAME") == 0) { *out = ptn_int(9); return 1; }
        if (strcmp(constant, "ATTR_CURSOR") == 0) { *out = ptn_int(10); return 1; }
        if (strcmp(constant, "ATTR_ORACLE_NULLS") == 0) { *out = ptn_int(11); return 1; }
        if (strcmp(constant, "ATTR_PERSISTENT") == 0) { *out = ptn_int(12); return 1; }
        if (strcmp(constant, "ATTR_STATEMENT_CLASS") == 0) { *out = ptn_int(13); return 1; }
        if (strcmp(constant, "ATTR_DEFAULT_FETCH_MODE") == 0) { *out = ptn_int(19); return 1; }
        if (strcmp(constant, "ATTR_EMULATE_PREPARES") == 0) { *out = ptn_int(20); return 1; }
        if (strcmp(constant, "ATTR_STRINGIFY_FETCHES") == 0) { *out = ptn_int(17); return 1; }
        if (strcmp(constant, "ATTR_DRIVER_NAME") == 0) { *out = ptn_int(16); return 1; }
        if (strcmp(constant, "CURSOR_FWDONLY") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "CURSOR_SCROLL") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "ERRMODE_SILENT") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "ERRMODE_WARNING") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "ERRMODE_EXCEPTION") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "CASE_NATURAL") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "CASE_UPPER") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "CASE_LOWER") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "NULL_NATURAL") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "NULL_EMPTY_STRING") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "NULL_TO_STRING") == 0) { *out = ptn_int(2); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "Pdo\\Sqlite")) {
        if (strcmp(constant, "ATTR_TRANSACTION_MODE") == 0) { *out = ptn_int(1000); return 1; }
        if (strcmp(constant, "TRANSACTION_MODE_DEFERRED") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "TRANSACTION_MODE_IMMEDIATE") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "TRANSACTION_MODE_EXCLUSIVE") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "DETERMINISTIC") == 0) { *out = ptn_int(0x800); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "SQLite3")) {
        if (strcmp(constant, "OK") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "DENY") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "IGNORE") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "OPEN_READONLY") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "OPEN_READWRITE") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "OPEN_CREATE") == 0) { *out = ptn_int(4); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "Phar")) {
        if (strcmp(constant, "PHP") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "PHPS") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "PHAR") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "TAR") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "ZIP") == 0) {
            *out = ptn_int(3);
            return 1;
        }
        if (strcmp(constant, "NONE") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "COMPRESSED") == 0) {
            *out = ptn_int(0xf000);
            return 1;
        }
        if (strcmp(constant, "MD5") == 0) {
            *out = ptn_int(0x0001);
            return 1;
        }
        if (strcmp(constant, "SHA1") == 0) {
            *out = ptn_int(0x0002);
            return 1;
        }
        if (strcmp(constant, "SHA256") == 0) {
            *out = ptn_int(0x0003);
            return 1;
        }
        if (strcmp(constant, "SHA512") == 0) {
            *out = ptn_int(0x0004);
            return 1;
        }
        if (strcmp(constant, "OPENSSL") == 0) {
            *out = ptn_int(0x0010);
            return 1;
        }
        if (strcmp(constant, "OPENSSL_SHA256") == 0) {
            *out = ptn_int(0x0011);
            return 1;
        }
        if (strcmp(constant, "OPENSSL_SHA512") == 0) {
            *out = ptn_int(0x0012);
            return 1;
        }
        if (strcmp(constant, "GZ") == 0) {
            *out = ptn_int(0x1000);
            return 1;
        }
        if (strcmp(constant, "BZ2") == 0) {
            *out = ptn_int(0x2000);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ZipArchive")) {
        if (strcmp(constant, "CREATE") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "OVERWRITE") == 0) { *out = ptn_int(8); return 1; }
        if (strcmp(constant, "RDONLY") == 0) { *out = ptn_int(16); return 1; }
        if (strcmp(constant, "FL_ENC_RAW") == 0) { *out = ptn_int(64); return 1; }
        if (strcmp(constant, "FL_OVERWRITE") == 0) { *out = ptn_int(8192); return 1; }
        if (strcmp(constant, "ER_OK") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "ER_EXISTS") == 0) { *out = ptn_int(10); return 1; }
        if (strcmp(constant, "ER_CANCELLED") == 0) { *out = ptn_int(32); return 1; }
        if (strcmp(constant, "EM_AES_256") == 0) { *out = ptn_int(259); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "RecursiveArrayIterator")) {
        if (strcmp(constant, "CHILD_ARRAYS_ONLY") == 0) {
            *out = ptn_int(4);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "RegexIterator")) {
        if (strcmp(constant, "MATCH") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "GET_MATCH") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "ALL_MATCHES") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "SPLIT") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "REPLACE") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "USE_KEY") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "INVERT_MATCH") == 0) { *out = ptn_int(2); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "RecursiveIteratorIterator")) {
        if (strcmp(constant, "LEAVES_ONLY") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "SELF_FIRST") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "CHILD_FIRST") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "CATCH_GET_CHILD") == 0) {
            *out = ptn_int(16);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "MultipleIterator")) {
        if (strcmp(constant, "MIT_NEED_ANY") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "MIT_NEED_ALL") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "MIT_KEYS_NUMERIC") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "MIT_KEYS_ASSOC") == 0) {
            *out = ptn_int(2);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "SplDoublyLinkedList") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "SplQueue") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "SplStack")) {
        if (strcmp(constant, "IT_MODE_FIFO") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "IT_MODE_LIFO") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "IT_MODE_KEEP") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "IT_MODE_DELETE") == 0) {
            *out = ptn_int(1);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "SplFileObject")) {
        if (strcmp(constant, "DROP_NEW_LINE") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "READ_AHEAD") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "SKIP_EMPTY") == 0) {
            *out = ptn_int(4);
            return 1;
        }
        if (strcmp(constant, "READ_CSV") == 0) {
            *out = ptn_int(8);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "SplPriorityQueue")) {
        if (strcmp(constant, "EXTR_BOTH") == 0) {
            *out = ptn_int(3);
            return 1;
        }
        if (strcmp(constant, "EXTR_PRIORITY") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "EXTR_DATA") == 0) {
            *out = ptn_int(1);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlPartsIterator")) {
        if (strcmp(constant, "KEY_SEQUENTIAL") == 0) {
            *out = ptn_int(PTN_INTL_PARTS_KEY_SEQUENTIAL);
            return 1;
        }
        if (strcmp(constant, "KEY_LEFT") == 0) {
            *out = ptn_int(PTN_INTL_PARTS_KEY_LEFT);
            return 1;
        }
        if (strcmp(constant, "KEY_RIGHT") == 0) {
            *out = ptn_int(PTN_INTL_PARTS_KEY_RIGHT);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlBreakIterator")) {
        if (strcmp(constant, "DONE") == 0) {
            *out = ptn_int(PTN_INTL_BREAK_ITERATOR_DONE);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlDateFormatter")) {
        if (strcmp(constant, "FULL") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "LONG") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "MEDIUM") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "SHORT") == 0) {
            *out = ptn_int(3);
            return 1;
        }
        if (strcmp(constant, "NONE") == 0) {
            *out = ptn_int(-1);
            return 1;
        }
        if (strcmp(constant, "TRADITIONAL") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "GREGORIAN") == 0) {
            *out = ptn_int(1);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlListFormatter")) {
        if (strcmp(constant, "TYPE_AND") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "TYPE_OR") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "TYPE_UNITS") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "WIDTH_WIDE") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "WIDTH_SHORT") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "WIDTH_NARROW") == 0) { *out = ptn_int(2); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlCalendar") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlGregorianCalendar")) {
        if (strcmp(constant, "FIELD_ERA") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "FIELD_YEAR") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "FIELD_MONTH") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "FIELD_WEEK_OF_YEAR") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "FIELD_WEEK_OF_MONTH") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "FIELD_DAY_OF_MONTH") == 0) { *out = ptn_int(5); return 1; }
        if (strcmp(constant, "FIELD_DAY_OF_YEAR") == 0) { *out = ptn_int(6); return 1; }
        if (strcmp(constant, "FIELD_DAY_OF_WEEK") == 0) { *out = ptn_int(7); return 1; }
        if (strcmp(constant, "FIELD_DAY_OF_WEEK_IN_MONTH") == 0) { *out = ptn_int(8); return 1; }
        if (strcmp(constant, "FIELD_AM_PM") == 0) { *out = ptn_int(9); return 1; }
        if (strcmp(constant, "FIELD_HOUR") == 0) { *out = ptn_int(10); return 1; }
        if (strcmp(constant, "FIELD_HOUR_OF_DAY") == 0) { *out = ptn_int(11); return 1; }
        if (strcmp(constant, "FIELD_MINUTE") == 0) { *out = ptn_int(12); return 1; }
        if (strcmp(constant, "FIELD_SECOND") == 0) { *out = ptn_int(13); return 1; }
        if (strcmp(constant, "FIELD_MILLISECOND") == 0) { *out = ptn_int(14); return 1; }
        if (strcmp(constant, "FIELD_ZONE_OFFSET") == 0) { *out = ptn_int(15); return 1; }
        if (strcmp(constant, "FIELD_DST_OFFSET") == 0) { *out = ptn_int(16); return 1; }
        if (strcmp(constant, "FIELD_YEAR_WOY") == 0) { *out = ptn_int(17); return 1; }
        if (strcmp(constant, "FIELD_DOW_LOCAL") == 0) { *out = ptn_int(18); return 1; }
        if (strcmp(constant, "FIELD_EXTENDED_YEAR") == 0) { *out = ptn_int(19); return 1; }
        if (strcmp(constant, "FIELD_JULIAN_DAY") == 0) { *out = ptn_int(20); return 1; }
        if (strcmp(constant, "FIELD_MILLISECONDS_IN_DAY") == 0) { *out = ptn_int(21); return 1; }
        if (strcmp(constant, "FIELD_IS_LEAP_MONTH") == 0) { *out = ptn_int(22); return 1; }
        if (strcmp(constant, "DOW_SUNDAY") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "DOW_MONDAY") == 0) { *out = ptn_int(2); return 1; }
        if (strcmp(constant, "DOW_TUESDAY") == 0) { *out = ptn_int(3); return 1; }
        if (strcmp(constant, "DOW_WEDNESDAY") == 0) { *out = ptn_int(4); return 1; }
        if (strcmp(constant, "DOW_THURSDAY") == 0) { *out = ptn_int(5); return 1; }
        if (strcmp(constant, "DOW_FRIDAY") == 0) { *out = ptn_int(6); return 1; }
        if (strcmp(constant, "DOW_SATURDAY") == 0) { *out = ptn_int(7); return 1; }
        if (strcmp(constant, "WALLTIME_LAST") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "WALLTIME_FIRST") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "WALLTIME_NEXT_VALID") == 0) { *out = ptn_int(2); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlTimeZone")) {
        if (strcmp(constant, "TYPE_ANY") == 0) { *out = ptn_int(0); return 1; }
        if (strcmp(constant, "TYPE_CANONICAL") == 0) { *out = ptn_int(1); return 1; }
        if (strcmp(constant, "TYPE_CANONICAL_LOCATION") == 0) { *out = ptn_int(2); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "Locale")) {
        if (strcmp(constant, "LANG_TAG") == 0) {
            *out = ptn_string("language");
            return 1;
        }
        if (strcmp(constant, "REGION_TAG") == 0) {
            *out = ptn_string("region");
            return 1;
        }
        if (strcmp(constant, "ACTUAL_LOCALE") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "VALID_LOCALE") == 0) {
            *out = ptn_int(1);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "NumberFormatter")) {
        if (strcmp(constant, "PATTERN_DECIMAL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PATTERN_DECIMAL); return 1; }
        if (strcmp(constant, "DECIMAL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_DECIMAL); return 1; }
        if (strcmp(constant, "CURRENCY") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_CURRENCY); return 1; }
        if (strcmp(constant, "PERCENT") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PERCENT); return 1; }
        if (strcmp(constant, "SCIENTIFIC") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_SCIENTIFIC); return 1; }
        if (strcmp(constant, "SPELLOUT") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_SPELLOUT); return 1; }
        if (strcmp(constant, "ORDINAL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_ORDINAL); return 1; }
        if (strcmp(constant, "DURATION") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_DURATION); return 1; }
        if (strcmp(constant, "PATTERN_RULEBASED") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PATTERN_RULEBASED); return 1; }
        if (strcmp(constant, "CURRENCY_ACCOUNTING") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_CURRENCY_ACCOUNTING); return 1; }
        if (strcmp(constant, "DECIMAL_COMPACT_SHORT") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_DECIMAL_COMPACT_SHORT); return 1; }
        if (strcmp(constant, "DECIMAL_COMPACT_LONG") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_DECIMAL_COMPACT_LONG); return 1; }
        if (strcmp(constant, "TYPE_DEFAULT") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_TYPE_DEFAULT); return 1; }
        if (strcmp(constant, "TYPE_INT32") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_TYPE_INT32); return 1; }
        if (strcmp(constant, "TYPE_INT64") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_TYPE_INT64); return 1; }
        if (strcmp(constant, "TYPE_DOUBLE") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_TYPE_DOUBLE); return 1; }
        if (strcmp(constant, "TYPE_CURRENCY") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_TYPE_CURRENCY); return 1; }
        if (strcmp(constant, "DECIMAL_SEPARATOR_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_DECIMAL_SEPARATOR_SYMBOL); return 1; }
        if (strcmp(constant, "GROUPING_SEPARATOR_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_GROUPING_SEPARATOR_SYMBOL); return 1; }
        if (strcmp(constant, "PATTERN_SEPARATOR_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PATTERN_SEPARATOR_SYMBOL); return 1; }
        if (strcmp(constant, "PERCENT_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PERCENT_SYMBOL); return 1; }
        if (strcmp(constant, "ZERO_DIGIT_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_ZERO_DIGIT_SYMBOL); return 1; }
        if (strcmp(constant, "DIGIT_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_DIGIT_SYMBOL); return 1; }
        if (strcmp(constant, "MINUS_SIGN_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_MINUS_SIGN_SYMBOL); return 1; }
        if (strcmp(constant, "PLUS_SIGN_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PLUS_SIGN_SYMBOL); return 1; }
        if (strcmp(constant, "CURRENCY_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_CURRENCY_SYMBOL); return 1; }
        if (strcmp(constant, "INTL_CURRENCY_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_INTL_CURRENCY_SYMBOL); return 1; }
        if (strcmp(constant, "MONETARY_SEPARATOR_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_MONETARY_SEPARATOR_SYMBOL); return 1; }
        if (strcmp(constant, "EXPONENTIAL_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_EXPONENTIAL_SYMBOL); return 1; }
        if (strcmp(constant, "PERMILL_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PERMILL_SYMBOL); return 1; }
        if (strcmp(constant, "PAD_ESCAPE_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_PAD_ESCAPE_SYMBOL); return 1; }
        if (strcmp(constant, "INFINITY_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_INFINITY_SYMBOL); return 1; }
        if (strcmp(constant, "NAN_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_NAN_SYMBOL); return 1; }
        if (strcmp(constant, "SIGNIFICANT_DIGIT_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_SIGNIFICANT_DIGIT_SYMBOL); return 1; }
        if (strcmp(constant, "MONETARY_GROUPING_SEPARATOR_SYMBOL") == 0) { *out = ptn_int(PTN_NUMBER_FORMATTER_MONETARY_GROUPING_SEPARATOR_SYMBOL); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "IntlNumberRangeFormatter")) {
        if (strcmp(constant, "COLLAPSE_AUTO") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_COLLAPSE_AUTO); return 1; }
        if (strcmp(constant, "COLLAPSE_NONE") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_COLLAPSE_NONE); return 1; }
        if (strcmp(constant, "COLLAPSE_UNIT") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_COLLAPSE_UNIT); return 1; }
        if (strcmp(constant, "COLLAPSE_ALL") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_COLLAPSE_ALL); return 1; }
        if (strcmp(constant, "IDENTITY_FALLBACK_SINGLE_VALUE") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_SINGLE_VALUE); return 1; }
        if (strcmp(constant, "IDENTITY_FALLBACK_APPROXIMATELY_OR_SINGLE_VALUE") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_APPROXIMATELY_OR_SINGLE_VALUE); return 1; }
        if (strcmp(constant, "IDENTITY_FALLBACK_APPROXIMATELY") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_APPROXIMATELY); return 1; }
        if (strcmp(constant, "IDENTITY_FALLBACK_RANGE") == 0) { *out = ptn_int(PTN_INTL_NUMBER_RANGE_IDENTITY_FALLBACK_RANGE); return 1; }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "Collator")) {
        if (strcmp(constant, "PRIMARY") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "SECONDARY") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "TERTIARY") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "SORT_REGULAR") == 0) {
            *out = ptn_int(PTN_SORT_REGULAR);
            return 1;
        }
        if (strcmp(constant, "SORT_STRING") == 0) {
            *out = ptn_int(PTN_SORT_STRING);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "UConverter")) {
        if (strcmp(constant, "REASON_UNASSIGNED") == 0) {
            *out = ptn_int(0);
            return 1;
        }
        if (strcmp(constant, "REASON_ILLEGAL") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "REASON_IRREGULAR") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "REASON_RESET") == 0) {
            *out = ptn_int(3);
            return 1;
        }
        if (strcmp(constant, "REASON_CLOSE") == 0) {
            *out = ptn_int(4);
            return 1;
        }
        if (strcmp(constant, "REASON_CLONE") == 0) {
            *out = ptn_int(5);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ReflectionClass")) {
        if (strcmp(constant, "IS_IMPLICIT_ABSTRACT") == 0) {
            *out = ptn_int(16);
            return 1;
        }
        if (strcmp(constant, "IS_EXPLICIT_ABSTRACT") == 0) {
            *out = ptn_int(64);
            return 1;
        }
        if (strcmp(constant, "IS_FINAL") == 0) {
            *out = ptn_int(32);
            return 1;
        }
        if (strcmp(constant, "IS_READONLY") == 0) {
            *out = ptn_int(65536);
            return 1;
        }
        if (strcmp(constant, "SKIP_INITIALIZATION_ON_SERIALIZE") == 0) {
            *out = ptn_int(PTN_LAZY_OBJECT_SKIP_INITIALIZATION_ON_SERIALIZE);
            return 1;
        }
        if (strcmp(constant, "SKIP_DESTRUCTOR") == 0) {
            *out = ptn_int(PTN_LAZY_OBJECT_SKIP_DESTRUCTOR);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ReflectionMethod")) {
        if (strcmp(constant, "IS_PUBLIC") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "IS_PROTECTED") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "IS_PRIVATE") == 0) {
            *out = ptn_int(4);
            return 1;
        }
        if (strcmp(constant, "IS_STATIC") == 0) {
            *out = ptn_int(16);
            return 1;
        }
        if (strcmp(constant, "IS_FINAL") == 0) {
            *out = ptn_int(32);
            return 1;
        }
        if (strcmp(constant, "IS_ABSTRACT") == 0) {
            *out = ptn_int(64);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ReflectionProperty")) {
        if (strcmp(constant, "IS_PUBLIC") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "IS_PROTECTED") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "IS_PRIVATE") == 0) {
            *out = ptn_int(4);
            return 1;
        }
        if (strcmp(constant, "IS_STATIC") == 0) {
            *out = ptn_int(16);
            return 1;
        }
        if (strcmp(constant, "IS_READONLY") == 0) {
            *out = ptn_int(128);
            return 1;
        }
        if (strcmp(constant, "IS_ABSTRACT") == 0) {
            *out = ptn_int(64);
            return 1;
        }
        if (strcmp(constant, "IS_FINAL") == 0) {
            *out = ptn_int(32);
            return 1;
        }
        if (strcmp(constant, "IS_VIRTUAL") == 0) {
            *out = ptn_int(512);
            return 1;
        }
        if (strcmp(constant, "IS_PROTECTED_SET") == 0) {
            *out = ptn_int(2048);
            return 1;
        }
        if (strcmp(constant, "IS_PRIVATE_SET") == 0) {
            *out = ptn_int(4096);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "PropertyHookType")) {
        if (strcmp(constant, "Get") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "Set") == 0) {
            *out = ptn_int(2);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "DateTimeZone")) {
        if (strcmp(constant, "AFRICA") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "AMERICA") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "ANTARCTICA") == 0) {
            *out = ptn_int(4);
            return 1;
        }
        if (strcmp(constant, "ARCTIC") == 0) {
            *out = ptn_int(8);
            return 1;
        }
        if (strcmp(constant, "ASIA") == 0) {
            *out = ptn_int(16);
            return 1;
        }
        if (strcmp(constant, "ATLANTIC") == 0) {
            *out = ptn_int(32);
            return 1;
        }
        if (strcmp(constant, "AUSTRALIA") == 0) {
            *out = ptn_int(64);
            return 1;
        }
        if (strcmp(constant, "EUROPE") == 0) {
            *out = ptn_int(128);
            return 1;
        }
        if (strcmp(constant, "INDIAN") == 0) {
            *out = ptn_int(256);
            return 1;
        }
        if (strcmp(constant, "PACIFIC") == 0) {
            *out = ptn_int(512);
            return 1;
        }
        if (strcmp(constant, "UTC") == 0) {
            *out = ptn_int(1024);
            return 1;
        }
        if (strcmp(constant, "ALL") == 0) {
            *out = ptn_int(2047);
            return 1;
        }
        if (strcmp(constant, "ALL_WITH_BC") == 0) {
            *out = ptn_int(4095);
            return 1;
        }
        if (strcmp(constant, "PER_COUNTRY") == 0) {
            *out = ptn_int(4096);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "Uri\\UriComparisonMode")) {
        if (strcmp(constant, "IncludeFragment") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "ExcludeFragment") == 0) {
            *out = ptn_int(2);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ReflectionClassConstant")) {
        if (strcmp(constant, "IS_PUBLIC") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "IS_PROTECTED") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "IS_PRIVATE") == 0) {
            *out = ptn_int(4);
            return 1;
        }
        if (strcmp(constant, "IS_FINAL") == 0) {
            *out = ptn_int(32);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "ReflectionAttribute")) {
        if (strcmp(constant, "IS_INSTANCEOF") == 0) {
            *out = ptn_int(2);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "Attribute")) {
        if (strcmp(constant, "TARGET_CLASS") == 0) {
            *out = ptn_int(1);
            return 1;
        }
        if (strcmp(constant, "TARGET_FUNCTION") == 0) {
            *out = ptn_int(2);
            return 1;
        }
        if (strcmp(constant, "TARGET_METHOD") == 0) {
            *out = ptn_int(4);
            return 1;
        }
        if (strcmp(constant, "TARGET_PROPERTY") == 0) {
            *out = ptn_int(8);
            return 1;
        }
        if (strcmp(constant, "TARGET_CLASS_CONSTANT") == 0) {
            *out = ptn_int(16);
            return 1;
        }
        if (strcmp(constant, "TARGET_PARAMETER") == 0) {
            *out = ptn_int(32);
            return 1;
        }
        if (strcmp(constant, "TARGET_CONSTANT") == 0) {
            *out = ptn_int(64);
            return 1;
        }
        if (strcmp(constant, "TARGET_ALL") == 0) {
            *out = ptn_int(127);
            return 1;
        }
        if (strcmp(constant, "IS_REPEATABLE") == 0) {
            *out = ptn_int(128);
            return 1;
        }
    }
    if (ptn_ascii_case_equal_span_to_string(class_name, class_len, "DateTime") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "DateTimeImmutable") ||
        ptn_ascii_case_equal_span_to_string(class_name, class_len, "DateTimeInterface")) {
        if (strcmp(constant, "ATOM") == 0 || strcmp(constant, "RFC3339") == 0 || strcmp(constant, "W3C") == 0) {
            *out = ptn_string("Y-m-d\\TH:i:sP");
            return 1;
        }
        if (strcmp(constant, "COOKIE") == 0) {
            *out = ptn_string("l, d-M-Y H:i:s T");
            return 1;
        }
        if (strcmp(constant, "ISO8601") == 0) {
            *out = ptn_string("Y-m-d\\TH:i:sO");
            return 1;
        }
        if (strcmp(constant, "ISO8601_EXPANDED") == 0) {
            *out = ptn_string("X-m-d\\TH:i:sP");
            return 1;
        }
        if (strcmp(constant, "RFC3339_EXTENDED") == 0) {
            *out = ptn_string("Y-m-d\\TH:i:s.vP");
            return 1;
        }
        if (strcmp(constant, "RFC822") == 0) {
            *out = ptn_string("D, d M y H:i:s O");
            return 1;
        }
        if (strcmp(constant, "RFC850") == 0) {
            *out = ptn_string("l, d-M-y H:i:s T");
            return 1;
        }
        if (strcmp(constant, "RFC1036") == 0) {
            *out = ptn_string("D, d M y H:i:s O");
            return 1;
        }
        if (strcmp(constant, "RFC1123") == 0 || strcmp(constant, "RFC2822") == 0 || strcmp(constant, "RSS") == 0) {
            *out = ptn_string("D, d M Y H:i:s O");
            return 1;
        }
        if (strcmp(constant, "RFC7231") == 0) {
            *out = ptn_string("D, d M Y H:i:s \\G\\M\\T");
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_builtin_class_constant_value(
    const char *class_name,
    const char *constant,
    PtnValue *out
) {
    return ptn_builtin_class_constant_value_span(class_name, strlen(class_name), constant, out);
}

static PTN_UNUSED PtnValue ptn_builtin_enum_case_singleton(
    PtnRuntime *runtime,
    const char *class_name,
    const char *case_name
) {
    if (runtime == NULL) {
        return ptn_enum_case(runtime, class_name, case_name);
    }
    char *key = ptn_class_constant_key(class_name, case_name);
    PtnValue existing;
    if (ptn_symbols_get(ptn_runtime_class_constant_table(runtime), key, &existing)) {
        free(key);
        return ptn_value_clone_deref(existing);
    }
    PtnValue created = ptn_enum_case(runtime, class_name, case_name);
    PtnValue result = ptn_value_clone(created);
    ptn_symbols_set(ptn_runtime_class_constant_table(runtime), key, created);
    ptn_value_destroy(&created);
    free(key);
    return result;
}

static PTN_UNUSED PtnValue ptn_runtime_undeclared_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    size_t line
) {
    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Access to undeclared static property %s::$%s",
        class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
    return ptn_null();
}

static PTN_UNUSED const char *ptn_runtime_resolve_relative_static_member_class(
    PtnRuntime *runtime,
    const char *class_name
) {
    const char *lookup_class_name = ptn_symbol_name_without_leading_slash(class_name);
    if (ptn_ascii_case_equal(lookup_class_name, "static")) {
        const char *called = runtime == NULL ? NULL : runtime->current_called_class_name;
        if (called != NULL) {
            return called;
        }
        const char *current = runtime == NULL ? NULL : runtime->current_class_name;
        return current == NULL ? lookup_class_name : current;
    }
    if (ptn_ascii_case_equal(lookup_class_name, "self")) {
        const char *current = runtime == NULL ? NULL : runtime->current_class_name;
        return current == NULL ? lookup_class_name : current;
    }
    if (ptn_ascii_case_equal(lookup_class_name, "parent")) {
        const char *current = runtime == NULL ? NULL : runtime->current_class_name;
        const char *parent = current == NULL ? NULL : ptn_runtime_declared_class_parent_name(runtime, current);
        return parent == NULL ? lookup_class_name : parent;
    }
    return lookup_class_name;
}

static PTN_UNUSED const char *ptn_runtime_maybe_autoload_static_member_class(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
) {
    const char *lookup_class_name = ptn_runtime_resolve_relative_static_member_class(runtime, class_name);
    const char *resolved_class_name = ptn_runtime_resolve_class_alias(runtime, lookup_class_name);
    if (!ptn_declared_runtime_class_exists(runtime, resolved_class_name)
        && !ptn_declared_trait_exists(resolved_class_name)
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        && !ptn_internal_class_exists_name(resolved_class_name)
#endif
    ) {
        char *suspended_constant_key = NULL;
        int restore_suspended_constant = 0;
        if (
            runtime->current_class_constant_initializing_key_class_name != NULL &&
            runtime->current_class_constant_initializing_constant_name != NULL
        ) {
            suspended_constant_key = ptn_class_constant_key(
                runtime->current_class_constant_initializing_key_class_name,
                runtime->current_class_constant_initializing_constant_name
            );
            PtnValue initializing;
            if (
                ptn_symbols_get(
                    ptn_runtime_class_constant_initializing_table(runtime),
                    suspended_constant_key,
                    &initializing
                ) &&
                ptn_is_truthy(initializing)
            ) {
                ptn_symbols_unset(
                    ptn_runtime_class_constant_initializing_table(runtime),
                    suspended_constant_key
                );
                restore_suspended_constant = 1;
            }
        }
        ptn_runtime_autoload_class(runtime, resolved_class_name, line);
        if (restore_suspended_constant) {
            ptn_symbols_set(
                ptn_runtime_class_constant_initializing_table(runtime),
                suspended_constant_key,
                ptn_bool(1)
            );
        }
        free(suspended_constant_key);
        if (runtime->exceptions->active_exception != NULL) {
            return resolved_class_name;
        }
        resolved_class_name = ptn_runtime_resolve_class_alias(runtime, lookup_class_name);
    }
    return resolved_class_name;
}

static PTN_UNUSED int ptn_runtime_static_member_class_exists(
    PtnRuntime *runtime,
    const char *class_name
) {
    return ptn_declared_runtime_class_exists(runtime, class_name) ||
        ptn_declared_trait_exists(class_name)
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        || ptn_internal_class_exists_name(class_name)
#endif
    ;
}

static PTN_UNUSED void ptn_throw_class_not_found_error(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
);

static PTN_UNUSED void ptn_runtime_define_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    PtnPropertyVisibility read_visibility,
    PtnPropertyVisibility set_visibility,
    PtnPropertyTypeKind type_kind,
    const char *type_class_name,
    const char *type_text,
    int type_allows_null,
    PtnValue value,
    int initialized
) {
    char *key = ptn_static_property_key(class_name, property);
    ptn_symbols_set(ptn_runtime_static_property_table(runtime), key, ptn_value_deref(value));
    ptn_symbols_set(
        ptn_runtime_static_property_initialized_table(runtime),
        key,
        ptn_bool(initialized)
    );
    ptn_symbols_set(
        ptn_runtime_static_property_read_visibility_table(runtime),
        key,
        ptn_int((int64_t)read_visibility)
    );
    ptn_symbols_set(
        ptn_runtime_static_property_set_visibility_table(runtime),
        key,
        ptn_int((int64_t)set_visibility)
    );
    ptn_symbols_set(
        ptn_runtime_static_property_type_kind_table(runtime),
        key,
        ptn_int((int64_t)type_kind)
    );
    if (type_class_name != NULL) {
        ptn_symbols_set(
            ptn_runtime_static_property_type_class_name_table(runtime),
            key,
            ptn_string(type_class_name)
        );
    }
    if (type_text != NULL) {
        ptn_symbols_set(
            ptn_runtime_static_property_type_text_table(runtime),
            key,
            ptn_string(type_text)
        );
    }
    ptn_symbols_set(
        ptn_runtime_static_property_type_allows_null_table(runtime),
        key,
        ptn_bool(type_allows_null)
    );
    free(key);
}

static PTN_UNUSED int ptn_runtime_static_property_metadata(
    PtnRuntime *runtime,
    const char *key,
    const char *declaring_class,
    const char *property,
    PtnObjectPropertyMetadata *metadata
) {
    PtnValue type_kind_value;
    if (!ptn_symbols_get(ptn_runtime_static_property_type_kind_table(runtime), key, &type_kind_value)) {
        return 0;
    }
    type_kind_value = ptn_value_deref(type_kind_value);
    if (type_kind_value.type != PTN_INT ||
        (PtnPropertyTypeKind)type_kind_value.as.integer == PTN_PROPERTY_TYPE_NONE) {
        return 0;
    }

    memset(metadata, 0, sizeof(*metadata));
    metadata->storage_name = (char *)key;
    metadata->display_name = (char *)property;
    metadata->declaring_class = (char *)declaring_class;
    metadata->read_visibility = PTN_PROPERTY_PUBLIC;
    metadata->set_visibility = PTN_PROPERTY_PUBLIC;
    metadata->type_kind = (PtnPropertyTypeKind)type_kind_value.as.integer;

    PtnValue type_class_name;
    if (ptn_symbols_get(
        ptn_runtime_static_property_type_class_name_table(runtime),
        key,
        &type_class_name
    )) {
        type_class_name = ptn_value_deref(type_class_name);
        if (type_class_name.type == PTN_STRING) {
            metadata->type_class_name = (char *)type_class_name.as.string.data;
        }
    }

    PtnValue type_text;
    if (ptn_symbols_get(ptn_runtime_static_property_type_text_table(runtime), key, &type_text)) {
        type_text = ptn_value_deref(type_text);
        if (type_text.type == PTN_STRING) {
            metadata->type_text = (char *)type_text.as.string.data;
        }
    }

    PtnValue allows_null;
    if (ptn_symbols_get(
        ptn_runtime_static_property_type_allows_null_table(runtime),
        key,
        &allows_null
    )) {
        metadata->type_allows_null = ptn_is_truthy(allows_null);
    }

    return 1;
}

static PTN_UNUSED int ptn_runtime_static_property_type_coerce_assignment(
    PtnRuntime *runtime,
    const PtnObjectPropertyMetadata *metadata,
    PtnValue value,
    int reference_context,
    size_t line,
    PtnValue *out
) {
    if (metadata == NULL) {
        *out = ptn_value_clone_deref(value);
        return 1;
    }
    return ptn_property_type_coerce_assignment(
        runtime,
        metadata->type_kind,
        metadata->type_class_name,
        metadata->type_text,
        metadata->type_allows_null,
        metadata->declaring_class,
        metadata->display_name,
        value,
        reference_context,
        line,
        out
    );
}

static PTN_UNUSED int ptn_property_type_accepts_array_auto_initialization(
    PtnRuntime *runtime,
    PtnPropertyTypeKind kind,
    const char *type_class_name,
    const char *type_text,
    int allows_null
) {
    PtnValue array = ptn_array_from_literal_entries(0, NULL);
    PtnValue coerced = ptn_null();
    int accepts = ptn_property_type_try_coerce_assignment(
        runtime,
        kind,
        type_class_name,
        type_text,
        allows_null,
        array,
        &coerced
    );
    ptn_value_destroy(&coerced);
    ptn_value_destroy(&array);
    return accepts;
}

static PTN_UNUSED int ptn_property_metadata_accepts_array_auto_initialization(
    PtnRuntime *runtime,
    const PtnObjectPropertyMetadata *metadata
) {
    return metadata != NULL &&
        ptn_property_type_accepts_array_auto_initialization(
            runtime,
            metadata->type_kind,
            metadata->type_class_name,
            metadata->type_text,
            metadata->type_allows_null
        );
}

static PTN_UNUSED int ptn_runtime_static_property_typed_uninitialized_key(
    PtnRuntime *runtime,
    const char *key,
    const char *declaring_class,
    const char *property
) {
    PtnObjectPropertyMetadata metadata;
    if (!ptn_runtime_static_property_metadata(runtime, key, declaring_class, property, &metadata)) {
        return 0;
    }
    PtnValue initialized;
    return !(
        ptn_symbols_get(
            ptn_runtime_static_property_initialized_table(runtime),
            key,
            &initialized
        ) && ptn_is_truthy(initialized)
    );
}

static PTN_UNUSED int ptn_runtime_static_property_typed_uninitialized(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char **declaring_class_out
) {
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        return 0;
    }
    int result = ptn_runtime_static_property_typed_uninitialized_key(
        runtime,
        key,
        declaring_class,
        property
    );
    free(key);
    if (declaring_class_out != NULL) {
        *declaring_class_out = declaring_class;
    }
    return result;
}

static PTN_UNUSED void ptn_throw_property_array_auto_initialization_error(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    const char *type_text,
    int reference_context,
    size_t line
) {
    char message[512];
    const char *declared_type = type_text == NULL ? "mixed" : type_text;
    int written;
    if (reference_context) {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot auto-initialize an array inside a reference held by property %s::$%s of type %s",
            declaring_class,
            property,
            declared_type
        );
    } else {
        written = snprintf(
            message,
            sizeof(message),
            "Cannot auto-initialize an array inside property %s::$%s of type %s",
            declaring_class,
            property,
            declared_type
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "TypeError", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_uninitialized_typed_static_property_error(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    size_t line,
    int mention_static
) {
    char message[384];
    int written = snprintf(
        message,
        sizeof(message),
        mention_static
            ? "Typed static property %s::$%s must not be accessed before initialization"
            : "Typed property %s::$%s must not be accessed before initialization",
        class_name,
        property
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_throw_exception_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED int ptn_runtime_static_property_initialized(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property
) {
    char *key = ptn_runtime_resolve_static_property_key(runtime, class_name, property, NULL);
    if (key == NULL) {
        return 0;
    }
    PtnValue initialized;
    int result = ptn_symbols_get(
        ptn_runtime_static_property_initialized_table(runtime),
        key,
        &initialized
    ) && ptn_is_truthy(initialized);
    free(key);
    return result;
}

static PTN_UNUSED int ptn_runtime_ensure_static_property_initialized(
    PtnRuntime *runtime,
    const char *key,
    const char *declaring_class,
    const char *property
) {
    PtnValue initialized;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_initialized_table(runtime),
            key,
            &initialized
        ) &&
        ptn_is_truthy(initialized)
    ) {
        return 1;
    }
    if (runtime->static_property_initializer != NULL && declaring_class != NULL) {
        (void)runtime->static_property_initializer(runtime, declaring_class, property);
        if (runtime->exceptions != NULL && runtime->exceptions->active_exception != NULL) {
            return 0;
        }
    }
    return 1;
}

static PTN_UNUSED void ptn_runtime_define_class_constant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    PtnValue value
) {
    char *key = ptn_class_constant_key(class_name, constant);
    ptn_symbols_set(ptn_runtime_class_constant_table(runtime), key, ptn_value_deref(value));
    free(key);
}

static PTN_UNUSED void ptn_runtime_define_class_constant_deprecation(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *warning
) {
    if (warning == NULL || warning[0] == '\0') {
        return;
    }
    char *key = ptn_class_constant_key(class_name, constant);
    ptn_symbols_set(ptn_runtime_class_constant_deprecation_table(runtime), key, ptn_string(warning));
    free(key);
}

static PTN_UNUSED char *ptn_deprecated_warning_message_for_parts(
    const char *subject,
    const char *since,
    const char *message
) {
    int has_since = since != NULL && since[0] != '\0';
    int has_message = message != NULL && message[0] != '\0';
    size_t len = strlen(subject) + strlen(" is deprecated");
    if (has_since) {
        len += strlen(" since ") + strlen(since);
    }
    if (has_message) {
        len += strlen(", ") + strlen(message);
    }
    char *warning = malloc(len + 1);
    if (warning == NULL) {
        ptn_abort_out_of_memory();
    }
    warning[0] = '\0';
    strcat(warning, subject);
    strcat(warning, " is deprecated");
    if (has_since) {
        strcat(warning, " since ");
        strcat(warning, since);
    }
    if (has_message) {
        strcat(warning, ", ");
        strcat(warning, message);
    }
    return warning;
}

static PTN_UNUSED void ptn_runtime_define_class_constant_deprecation_from_value(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *subject,
    const char *since,
    PtnValue message_value
) {
    char *message = ptn_value_to_string(message_value);
    char *warning = ptn_deprecated_warning_message_for_parts(subject, since, message);
    ptn_runtime_define_class_constant_deprecation(runtime, class_name, constant, warning);
    free(warning);
    free(message);
}

static void ptn_runtime_emit_class_constant_deprecation(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    size_t line
) {
    if (runtime == NULL || line == 0) {
        return;
    }
    if (runtime->class_constant_deprecation_suppress_class != NULL &&
        runtime->class_constant_deprecation_suppress_constant != NULL &&
        ptn_ascii_case_equal(runtime->class_constant_deprecation_suppress_class, class_name) &&
        strcmp(runtime->class_constant_deprecation_suppress_constant, constant) == 0) {
        return;
    }
    char *key = ptn_class_constant_key(class_name, constant);
    PtnValue warning;
    if (ptn_symbols_get(ptn_runtime_class_constant_deprecation_table(runtime), key, &warning)) {
        char *message = ptn_value_to_string(warning);
        ptn_emit_user_deprecation(&runtime->diagnostics, message, line);
        free(message);
    }
    free(key);
}

static PTN_UNUSED PtnValue ptn_runtime_undefined_class_constant(
    PtnRuntime *runtime,
    const char *lookup_class_name,
    const char *message_class_name,
    const char *constant,
    size_t line
) {
    char message[256];
    int written;
    if (ptn_declared_trait_exists(lookup_class_name) &&
        !(
            message_class_name != NULL &&
            (
                ptn_ascii_case_equal(message_class_name, "self") ||
                ptn_ascii_case_equal(message_class_name, "static") ||
                ptn_ascii_case_equal(message_class_name, "parent")
            )
        )) {
        const char *display_class_name =
            message_class_name == NULL ? lookup_class_name : message_class_name;
        written = snprintf(
            message,
            sizeof(message),
            "Cannot access trait constant %s::%s directly",
            display_class_name,
            constant
        );
    } else if (!ptn_declared_runtime_class_exists(runtime, lookup_class_name) &&
        !ptn_declared_runtime_interface_exists(runtime, lookup_class_name)
        && !ptn_declared_trait_exists(lookup_class_name)
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
        && !ptn_internal_class_exists_name(lookup_class_name)
        && !ptn_internal_interface_exists_name(lookup_class_name)
#endif
    ) {
        written = snprintf(message, sizeof(message), "Class \"%s\" not found", lookup_class_name);
    } else {
        const char *display_class_name =
            message_class_name == NULL ? lookup_class_name : message_class_name;
        written = snprintf(
            message,
            sizeof(message),
            "Undefined constant %s::%s",
            display_class_name,
            constant
        );
    }
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    const char *source_path =
        runtime != NULL && runtime->current_class_constant_source_path != NULL
            ? runtime->current_class_constant_source_path
            : (runtime != NULL ? runtime->source_path : NULL);
    ptn_throw_exception_at(
        runtime,
        "Error",
        message,
        source_path,
        line
    );
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant_impl(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *message_class_name,
    const char *access_scope,
    int enforce_visibility,
    size_t line,
    size_t deprecation_line
);

static PTN_UNUSED int ptn_runtime_read_dynamic_interface_class_constant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *access_scope,
    int enforce_visibility,
    size_t line,
    size_t deprecation_line,
    PtnValue *out
) {
    PtnValue interfaces = ptn_runtime_dynamic_class_interfaces(runtime, class_name);
    interfaces = ptn_value_deref(interfaces);
    if (interfaces.type != PTN_ARRAY || interfaces.as.array == NULL) {
        return 0;
    }
    for (size_t i = 0; i < interfaces.as.array->len; i++) {
        PtnValue interface_value = ptn_value_deref(interfaces.as.array->entries[i].value);
        if (interface_value.type != PTN_STRING) {
            continue;
        }
        const char *interface_name = (const char *)interface_value.as.string.data;
        const char *metadata_declaring_class = interface_name;
        int metadata_visibility_int = (int)PTN_PROPERTY_PUBLIC;
        if (!ptn_declared_class_constant_metadata(
            interface_name,
            constant,
            &metadata_declaring_class,
            &metadata_visibility_int
        )) {
            continue;
        }
        (void)metadata_declaring_class;
        (void)metadata_visibility_int;
        *out = ptn_runtime_read_class_constant_impl(
            runtime,
            interface_name,
            constant,
            interface_name,
            access_scope,
            enforce_visibility,
            line,
            deprecation_line
        );
        return 1;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant_impl(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *message_class_name,
    const char *access_scope,
    int enforce_visibility,
    size_t line,
    size_t deprecation_line
) {
    const char *resolved_class_name =
        ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    const char *target_class_name = ptn_declared_class_canonical_name(resolved_class_name);
    const char *lookup_class_name = target_class_name;
    while (lookup_class_name != NULL) {
        const char *metadata_declaring_class = lookup_class_name;
        int metadata_visibility_int = (int)PTN_PROPERTY_PUBLIC;
        int lookup_has_metadata = ptn_declared_class_constant_metadata(
            lookup_class_name,
            constant,
            &metadata_declaring_class,
            &metadata_visibility_int
        );
        char *key = ptn_class_constant_key(metadata_declaring_class, constant);
        PtnValue value;
        if (ptn_symbols_get(ptn_runtime_class_constant_table(runtime), key, &value)) {
            const char *declaring_class = lookup_has_metadata ? metadata_declaring_class : lookup_class_name;
            int visibility_int = lookup_has_metadata ? metadata_visibility_int : (int)PTN_PROPERTY_PUBLIC;
            if (!lookup_has_metadata && !ptn_property_class_names_equal(target_class_name, lookup_class_name)) {
                free(key);
                lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                continue;
            }
            PtnPropertyVisibility visibility = (PtnPropertyVisibility)visibility_int;
            if (visibility == PTN_PROPERTY_PRIVATE &&
                !ptn_property_class_names_equal(target_class_name, declaring_class)) {
                free(key);
                lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                continue;
            }
            if (
                enforce_visibility &&
                !ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)
            ) {
                free(key);
                ptn_throw_class_constant_visibility_error(
                    runtime,
                    visibility,
                    target_class_name,
                    constant,
                    line
                );
                return ptn_null();
            }
            ptn_runtime_emit_class_constant_deprecation(
                runtime,
                lookup_class_name,
                constant,
                deprecation_line
            );
            free(key);
            return ptn_value_clone_deref(value);
        }
        PtnValue initializing;
        if (
            ptn_symbols_get(
                ptn_runtime_class_constant_initializing_table(runtime),
                key,
                &initializing
            ) &&
            ptn_is_truthy(initializing)
        ) {
            const char *message_class = runtime->current_class_constant_initializing_class_name == NULL
                ? (message_class_name == NULL ? lookup_class_name : message_class_name)
                : runtime->current_class_constant_initializing_class_name;
            const char *message_constant = runtime->current_class_constant_initializing_constant_name == NULL
                ? constant
                : runtime->current_class_constant_initializing_constant_name;
            char message[256];
            int written = snprintf(
                message,
                sizeof(message),
                "Cannot declare self-referencing constant %s::%s",
                message_class,
                message_constant
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                free(key);
                ptn_abort_out_of_memory();
            }
            free(key);
            const char *source_path =
                runtime != NULL && runtime->current_class_constant_source_path != NULL
                    ? runtime->current_class_constant_source_path
                    : (runtime != NULL ? runtime->source_path : NULL);
            ptn_throw_exception_at(
                runtime,
                "Error",
                message,
                source_path,
                line
            );
            return ptn_null();
        }
        if (runtime->class_constant_initializer != NULL) {
            const char *previous_initializing_class =
                runtime->current_class_constant_initializing_class_name;
            const char *previous_initializing_key_class =
                runtime->current_class_constant_initializing_key_class_name;
            const char *previous_initializing_constant =
                runtime->current_class_constant_initializing_constant_name;
            runtime->current_class_constant_initializing_class_name =
                message_class_name == NULL ? lookup_class_name : message_class_name;
            runtime->current_class_constant_initializing_key_class_name =
                metadata_declaring_class;
            runtime->current_class_constant_initializing_constant_name =
                constant;
            int initialized =
                runtime->class_constant_initializer(runtime, metadata_declaring_class, constant);
            runtime->current_class_constant_initializing_class_name =
                previous_initializing_class;
            runtime->current_class_constant_initializing_key_class_name =
                previous_initializing_key_class;
            runtime->current_class_constant_initializing_constant_name =
                previous_initializing_constant;
            if (initialized) {
            if (runtime->exceptions != NULL && runtime->exceptions->active_exception != NULL) {
                free(key);
                return ptn_null();
            }
            if (ptn_symbols_get(ptn_runtime_class_constant_table(runtime), key, &value)) {
                const char *declaring_class = lookup_has_metadata ? metadata_declaring_class : lookup_class_name;
                int visibility_int = lookup_has_metadata ? metadata_visibility_int : (int)PTN_PROPERTY_PUBLIC;
                if (!lookup_has_metadata && !ptn_property_class_names_equal(target_class_name, lookup_class_name)) {
                    free(key);
                    lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                    continue;
                }
                PtnPropertyVisibility visibility = (PtnPropertyVisibility)visibility_int;
                if (visibility == PTN_PROPERTY_PRIVATE &&
                    !ptn_property_class_names_equal(target_class_name, declaring_class)) {
                    free(key);
                    lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
                    continue;
                }
                if (
                    enforce_visibility &&
                    !ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)
                ) {
                    free(key);
                    ptn_throw_class_constant_visibility_error(
                        runtime,
                        visibility,
                        target_class_name,
                        constant,
                        line
                    );
                    return ptn_null();
                }
                ptn_runtime_emit_class_constant_deprecation(
                    runtime,
                    lookup_class_name,
                    constant,
                    deprecation_line
                );
                free(key);
                return ptn_value_clone_deref(value);
            }
            }
        }
        free(key);
        PtnValue dynamic_interface_value;
        if (ptn_runtime_read_dynamic_interface_class_constant(
            runtime,
            lookup_class_name,
            constant,
            access_scope,
            enforce_visibility,
            line,
            deprecation_line,
            &dynamic_interface_value
        )) {
            return dynamic_interface_value;
        }
        lookup_class_name = ptn_runtime_declared_class_parent_name(runtime, lookup_class_name);
    }
    PtnValue builtin_value;
    const char *rounding_case = ptn_ascii_case_equal(resolved_class_name, "RoundingMode")
        ? ptn_rounding_mode_case_name(constant)
        : NULL;
    if (rounding_case != NULL) {
        return ptn_enum_case(runtime, "RoundingMode", rounding_case);
    }
    const char *random_interval_boundary_case = ptn_ascii_case_equal(resolved_class_name, "Random\\IntervalBoundary")
        ? ptn_random_interval_boundary_case_name(constant)
        : NULL;
    if (random_interval_boundary_case != NULL) {
        return ptn_builtin_enum_case_singleton(runtime, "Random\\IntervalBoundary", random_interval_boundary_case);
    }
    const char *stream_error_code_case = ptn_ascii_case_equal(resolved_class_name, "StreamErrorCode")
        ? ptn_stream_error_code_case_name(constant)
        : NULL;
    if (stream_error_code_case != NULL) {
        return ptn_builtin_enum_case_singleton(runtime, "StreamErrorCode", stream_error_code_case);
    }
    const char *stream_error_mode_case = ptn_ascii_case_equal(resolved_class_name, "StreamErrorMode")
        ? ptn_stream_error_mode_case_name(constant)
        : NULL;
    if (stream_error_mode_case != NULL) {
        return ptn_builtin_enum_case_singleton(runtime, "StreamErrorMode", stream_error_mode_case);
    }
    const char *stream_error_store_case = ptn_ascii_case_equal(resolved_class_name, "StreamErrorStore")
        ? ptn_stream_error_store_case_name(constant)
        : NULL;
    if (stream_error_store_case != NULL) {
        return ptn_builtin_enum_case_singleton(runtime, "StreamErrorStore", stream_error_store_case);
    }
    const char *url_host_type_case = ptn_ascii_case_equal(resolved_class_name, "Uri\\WhatWg\\UrlHostType")
        ? ptn_uri_url_host_type_case_name(constant)
        : NULL;
    if (url_host_type_case != NULL) {
        return ptn_enum_case(runtime, "Uri\\WhatWg\\UrlHostType", url_host_type_case);
    }
    const char *uri_type_case = ptn_ascii_case_equal(resolved_class_name, "Uri\\Rfc3986\\UriType")
        ? ptn_uri_type_case_name(constant)
        : NULL;
    if (uri_type_case != NULL) {
        return ptn_enum_case(runtime, "Uri\\Rfc3986\\UriType", uri_type_case);
    }
    const char *comparison_mode_case = ptn_ascii_case_equal(resolved_class_name, "Uri\\UriComparisonMode")
        ? ptn_uri_comparison_mode_case_name(constant)
        : NULL;
    if (comparison_mode_case != NULL) {
        return ptn_enum_case(runtime, "Uri\\UriComparisonMode", comparison_mode_case);
    }
    const char *dom_adjacent_position_case = ptn_ascii_case_equal(resolved_class_name, "Dom\\AdjacentPosition")
        ? ptn_dom_adjacent_position_case_name(constant)
        : NULL;
    if (dom_adjacent_position_case != NULL) {
        return ptn_builtin_enum_case_singleton(runtime, "Dom\\AdjacentPosition", dom_adjacent_position_case);
    }
    const char *url_validation_error_type_case = ptn_ascii_case_equal(resolved_class_name, "Uri\\WhatWg\\UrlValidationErrorType")
        ? ptn_uri_url_validation_error_type_case_name(constant)
        : NULL;
    if (url_validation_error_type_case != NULL) {
        return ptn_builtin_enum_case_singleton(runtime, "Uri\\WhatWg\\UrlValidationErrorType", url_validation_error_type_case);
    }
    const char *property_hook_type_case = ptn_ascii_case_equal(resolved_class_name, "PropertyHookType")
        ? ptn_property_hook_type_case_name(constant)
        : NULL;
    if (property_hook_type_case != NULL) {
        return ptn_builtin_enum_case_singleton(runtime, "PropertyHookType", property_hook_type_case);
    }
    const char *builtin_lookup_class_name = resolved_class_name;
    while (builtin_lookup_class_name != NULL) {
        if (ptn_builtin_class_constant_value(builtin_lookup_class_name, constant, &builtin_value)) {
            return builtin_value;
        }
        builtin_lookup_class_name =
            ptn_runtime_declared_class_parent_name(runtime, builtin_lookup_class_name);
    }
    return ptn_runtime_undefined_class_constant(
        runtime,
        resolved_class_name,
        message_class_name,
        constant,
        line
    );
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    size_t line
) {
    return ptn_runtime_read_class_constant_impl(
        runtime,
        class_name,
        constant,
        NULL,
        NULL,
        0,
        line,
        line
    );
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant_with_scope(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *access_scope,
    size_t line
) {
    return ptn_runtime_read_class_constant_impl(
        runtime,
        class_name,
        constant,
        NULL,
        access_scope,
        1,
        line,
        line
    );
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant_suppress_deprecation(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    size_t line
) {
    return ptn_runtime_read_class_constant_impl(
        runtime,
        class_name,
        constant,
        NULL,
        NULL,
        0,
        line,
        0
    );
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant_with_scope_suppress_deprecation(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *access_scope,
    size_t line
) {
    return ptn_runtime_read_class_constant_impl(
        runtime,
        class_name,
        constant,
        NULL,
        access_scope,
        1,
        line,
        0
    );
}

static PTN_UNUSED PtnValue ptn_runtime_read_class_constant_with_scope_message_class(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant,
    const char *message_class_name,
    const char *access_scope,
    size_t line
) {
    return ptn_runtime_read_class_constant_impl(
        runtime,
        class_name,
        constant,
        message_class_name,
        access_scope,
        1,
        line,
        line
    );
}

static PTN_UNUSED const char *ptn_dynamic_class_name_fetch_type_name(PtnValue value) {
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
    return "unknown";
}

static PTN_UNUSED char *ptn_runtime_dynamic_class_constant_name(
    PtnRuntime *runtime,
    PtnValue value,
    size_t line
) {
    value = ptn_value_deref(value);
    if (value.type == PTN_STRING) {
        return ptn_duplicate_string_len((const char *)value.as.string.data, value.as.string.len);
    }

    const char *type_name = ptn_dynamic_class_name_fetch_type_name(value);
    int needed = snprintf(
        NULL,
        0,
        "Cannot use value of type %s as class constant name",
        type_name
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
        "Cannot use value of type %s as class constant name",
        type_name
    );
    ptn_throw_exception_owned_message_at(
        runtime,
        "Error",
        message,
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
    return NULL;
}

static PTN_UNUSED PtnValue ptn_runtime_fetch_dynamic_class_name(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    const char *class_name = NULL;
    if (receiver.type == PTN_OBJECT) {
        class_name = receiver.as.object->class_name;
    } else if (receiver.type == PTN_EXCEPTION) {
        class_name = receiver.as.exception->class_name;
    } else if (receiver.type == PTN_CLOSURE) {
        class_name = "Closure";
    }
    if (class_name != NULL) {
        return ptn_owned_string(ptn_duplicate_string(class_name));
    }

    if (receiver.type == PTN_NULL) {
        ptn_throw_exception_at(
            runtime,
            "TypeError",
            "Cannot use \"::class\" on null",
            runtime != NULL ? runtime->source_path : NULL,
            line
        );
        return ptn_null();
    }

    const char *type_name = ptn_dynamic_class_name_fetch_type_name(receiver);
    int needed = snprintf(NULL, 0, "Cannot use \"::class\" on value of type %s", type_name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(message, (size_t)needed + 1, "Cannot use \"::class\" on value of type %s", type_name);
    ptn_throw_exception_owned_message_at(
        runtime,
        "TypeError",
        message,
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
    return ptn_null();
}

static PTN_UNUSED PtnValue ptn_runtime_fetch_dynamic_static_member_class_name(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_STRING) {
        return ptn_value_clone_deref(receiver);
    }
    const char *class_name = NULL;
    if (receiver.type == PTN_OBJECT) {
        class_name = receiver.as.object->class_name;
    } else if (receiver.type == PTN_EXCEPTION) {
        class_name = receiver.as.exception->class_name;
    } else if (receiver.type == PTN_CLOSURE) {
        class_name = "Closure";
    }
    if (class_name != NULL) {
        return ptn_owned_string(ptn_duplicate_string(class_name));
    }

    ptn_throw_exception_at(
        runtime,
        "Error",
        "Class name must be a valid object or a string",
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
    return ptn_null();
}

static PTN_UNUSED int ptn_runtime_emit_static_trait_property_deprecation(
    PtnRuntime *runtime,
    const char *declaring_class,
    const char *property,
    size_t line
) {
    if (runtime == NULL || declaring_class == NULL || !ptn_declared_trait_exists(declaring_class)) {
        return 1;
    }
    int needed = snprintf(
        NULL,
        0,
        "Accessing static trait property %s::$%s is deprecated, it should only be accessed on a class using the trait",
        declaring_class,
        property
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
        "Accessing static trait property %s::$%s is deprecated, it should only be accessed on a class using the trait",
        declaring_class,
        property
    );
    ptn_emit_deprecation(&runtime->diagnostics, message, line);
    free(message);
    return runtime->exceptions == NULL || runtime->exceptions->active_exception == NULL;
}

static PTN_UNUSED int ptn_runtime_emit_static_trait_method_deprecation(
    PtnRuntime *runtime,
    const char *trait_name,
    const char *method_name,
    size_t line
) {
    if (runtime == NULL) {
        return 1;
    }
    int needed = snprintf(
        NULL,
        0,
        "Calling static trait method %s::%s is deprecated, it should only be called on a class using the trait",
        trait_name,
        method_name
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
        "Calling static trait method %s::%s is deprecated, it should only be called on a class using the trait",
        trait_name,
        method_name
    );
    ptn_emit_deprecation(&runtime->diagnostics, message, line);
    free(message);
    return runtime->exceptions == NULL || runtime->exceptions->active_exception == NULL;
}

static PTN_UNUSED PtnValue ptn_runtime_read_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line
) {
    class_name = ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    if (!ptn_runtime_static_member_class_exists(runtime, class_name)) {
        ptn_throw_class_not_found_error(runtime, class_name, line);
        return ptn_null();
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key != NULL) {
        if (!ptn_runtime_ensure_static_property_initialized(runtime, key, declaring_class, property)) {
            free(key);
            return ptn_null();
        }
        if (!ptn_runtime_emit_static_trait_property_deprecation(
                runtime,
                declaring_class,
                property,
                line
            )) {
            free(key);
            return ptn_null();
        }
        PtnObjectPropertyMetadata metadata;
        PtnObjectPropertyMetadata *metadata_ptr =
            ptn_runtime_static_property_metadata(runtime, key, declaring_class, property, &metadata)
                ? &metadata
                : NULL;
        PtnValue initialized;
        if (metadata_ptr != NULL &&
            (!ptn_symbols_get(
                ptn_runtime_static_property_initialized_table(runtime),
                key,
                &initialized
            ) || !ptn_is_truthy(initialized))) {
            free(key);
            ptn_throw_uninitialized_typed_static_property_error(
                runtime,
                declaring_class,
                property,
                line,
                1
            );
            return ptn_null();
        }
        PtnSymbolTable *static_properties = ptn_runtime_static_property_table(runtime);
        PtnValue value;
        if (!ptn_symbols_get(ptn_runtime_static_property_table(runtime), key, &value)) {
            ptn_symbols_set(static_properties, key, ptn_null());
            value = ptn_null();
        }
        PtnValue visibility_value;
        PtnPropertyVisibility visibility = PTN_PROPERTY_PUBLIC;
        if (
            ptn_symbols_get(
                ptn_runtime_static_property_read_visibility_table(runtime),
                key,
                &visibility_value
            ) &&
            ptn_value_deref(visibility_value).type == PTN_INT
        ) {
            visibility = (PtnPropertyVisibility)ptn_value_deref(visibility_value).as.integer;
        }
        if (!ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
            free(key);
            ptn_throw_property_visibility_error(
                runtime,
                visibility,
                ptn_static_property_visibility_error_class(visibility, class_name, declaring_class),
                property,
                line
            );
            return ptn_null();
        }
        free(key);
        return ptn_value_clone_deref(value);
    }
    free(key);
    return ptn_runtime_undeclared_static_property(runtime, class_name, property, line);
}

static PTN_UNUSED PtnValue ptn_runtime_read_static_property_for_indirect_write(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line
) {
    class_name = ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    if (!ptn_runtime_static_member_class_exists(runtime, class_name)) {
        ptn_throw_class_not_found_error(runtime, class_name, line);
        return ptn_null();
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        return ptn_runtime_undeclared_static_property(runtime, class_name, property, line);
    }
    if (!ptn_runtime_ensure_static_property_initialized(runtime, key, declaring_class, property)) {
        free(key);
        return ptn_null();
    }
    if (!ptn_runtime_emit_static_trait_property_deprecation(
            runtime,
            declaring_class,
            property,
            line
        )) {
        free(key);
        return ptn_null();
    }

    PtnPropertyVisibility read_visibility = PTN_PROPERTY_PUBLIC;
    PtnValue visibility_value;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_read_visibility_table(runtime),
            key,
            &visibility_value
        ) &&
        ptn_value_deref(visibility_value).type == PTN_INT
    ) {
        read_visibility = (PtnPropertyVisibility)ptn_value_deref(visibility_value).as.integer;
    }
    if (!ptn_property_visibility_allows(runtime, read_visibility, declaring_class, access_scope)) {
        free(key);
        ptn_throw_property_visibility_error(
            runtime,
            read_visibility,
            ptn_static_property_visibility_error_class(read_visibility, class_name, declaring_class),
            property,
            line
        );
        return ptn_null();
    }

    PtnObjectPropertyMetadata metadata;
    PtnObjectPropertyMetadata *metadata_ptr =
        ptn_runtime_static_property_metadata(runtime, key, declaring_class, property, &metadata)
            ? &metadata
            : NULL;
    PtnValue initialized;
    int is_initialized = ptn_symbols_get(
        ptn_runtime_static_property_initialized_table(runtime),
        key,
        &initialized
    ) && ptn_is_truthy(initialized);
    if (!is_initialized && metadata_ptr != NULL) {
        if (ptn_property_metadata_accepts_array_auto_initialization(runtime, metadata_ptr)) {
            free(key);
            return ptn_array_from_literal_entries(0, NULL);
        }
        free(key);
        ptn_throw_property_array_auto_initialization_error(
            runtime,
            metadata_ptr->declaring_class,
            metadata_ptr->display_name,
            metadata_ptr->type_text,
            0,
            line
        );
        return ptn_null();
    }

    PtnValue value;
    if (!ptn_symbols_get(ptn_runtime_static_property_table(runtime), key, &value)) {
        ptn_symbols_set(ptn_runtime_static_property_table(runtime), key, ptn_null());
        value = ptn_null();
    }
    PtnValue resolved = ptn_value_deref(value);
    if (metadata_ptr != NULL &&
        resolved.type == PTN_NULL &&
        !ptn_property_metadata_accepts_array_auto_initialization(runtime, metadata_ptr)) {
        free(key);
        ptn_throw_property_array_auto_initialization_error(
            runtime,
            metadata_ptr->declaring_class,
            metadata_ptr->display_name,
            metadata_ptr->type_text,
            0,
            line
        );
        return ptn_null();
    }
    free(key);
    return ptn_value_clone_deref(value);
}

static PTN_UNUSED PtnLookupResult ptn_runtime_read_static_property_quiet(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line
) {
    class_name = ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_lookup_missing();
    }
    if (!ptn_runtime_static_member_class_exists(runtime, class_name)) {
        ptn_throw_class_not_found_error(runtime, class_name, line);
        return ptn_lookup_missing();
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key != NULL) {
        if (!ptn_runtime_ensure_static_property_initialized(runtime, key, declaring_class, property)) {
            free(key);
            return ptn_lookup_missing();
        }
        if (ptn_runtime_static_property_typed_uninitialized_key(
                runtime,
                key,
                declaring_class,
                property
            )) {
            free(key);
            return ptn_lookup_missing();
        }
        PtnSymbolTable *static_properties = ptn_runtime_static_property_table(runtime);
        PtnValue value;
        if (!ptn_symbols_get(ptn_runtime_static_property_table(runtime), key, &value)) {
            ptn_symbols_set(static_properties, key, ptn_null());
            value = ptn_null();
        }
        PtnValue visibility_value;
        PtnPropertyVisibility visibility = PTN_PROPERTY_PUBLIC;
        if (
            ptn_symbols_get(
                ptn_runtime_static_property_read_visibility_table(runtime),
                key,
                &visibility_value
            ) &&
            ptn_value_deref(visibility_value).type == PTN_INT
        ) {
            visibility = (PtnPropertyVisibility)ptn_value_deref(visibility_value).as.integer;
        }
        if (!ptn_property_visibility_allows(runtime, visibility, declaring_class, access_scope)) {
            free(key);
            return ptn_lookup_missing();
        }
        free(key);
        return ptn_lookup_found(ptn_value_clone_deref(value));
    }
    free(key);
    return ptn_lookup_missing();
}

static PTN_UNUSED void ptn_throw_static_property_unset_error(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    size_t line
) {
    int needed = snprintf(
        NULL,
        0,
        "Attempt to unset static property %s::$%s",
        class_name,
        property
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
        "Attempt to unset static property %s::$%s",
        class_name,
        property
    );
    ptn_throw_exception_owned_message_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_throw_class_not_found_error(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
) {
    int needed = snprintf(NULL, 0, "Class \"%s\" not found", class_name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(message, (size_t)needed + 1, "Class \"%s\" not found", class_name);
    ptn_throw_exception_owned_message_at(runtime, "Error", message, runtime->source_path, line);
}

static PTN_UNUSED void ptn_runtime_unset_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line
) {
    (void)access_scope;
    const char *resolved_class_name =
        ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return;
    }
    if (!ptn_runtime_static_member_class_exists(runtime, resolved_class_name)) {
        ptn_throw_class_not_found_error(runtime, resolved_class_name, line);
        return;
    }
    ptn_throw_static_property_unset_error(runtime, resolved_class_name, property, line);
}

static PTN_UNUSED PtnValue ptn_runtime_reference_for_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line
) {
    if (runtime != NULL &&
        runtime->exceptions != NULL &&
        runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    class_name = ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    if (!ptn_runtime_static_member_class_exists(runtime, class_name)) {
        ptn_throw_class_not_found_error(runtime, class_name, line);
        return ptn_null();
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        return ptn_runtime_undeclared_static_property(runtime, class_name, property, line);
    }
    if (!ptn_runtime_ensure_static_property_initialized(runtime, key, declaring_class, property)) {
        free(key);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (!ptn_runtime_emit_static_trait_property_deprecation(
            runtime,
            declaring_class,
            property,
            line
        )) {
        free(key);
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    PtnValue read_visibility_value;
    PtnValue set_visibility_value;
    PtnPropertyVisibility read_visibility = PTN_PROPERTY_PUBLIC;
    PtnPropertyVisibility set_visibility = PTN_PROPERTY_PUBLIC;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_read_visibility_table(runtime),
            key,
            &read_visibility_value
        ) &&
        ptn_value_deref(read_visibility_value).type == PTN_INT
    ) {
        read_visibility = (PtnPropertyVisibility)ptn_value_deref(read_visibility_value).as.integer;
    }
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_set_visibility_table(runtime),
            key,
            &set_visibility_value
        ) &&
        ptn_value_deref(set_visibility_value).type == PTN_INT
    ) {
        set_visibility = (PtnPropertyVisibility)ptn_value_deref(set_visibility_value).as.integer;
    }
    if (!ptn_property_visibility_allows(runtime, read_visibility, declaring_class, access_scope)) {
        free(key);
        ptn_throw_property_visibility_error(
            runtime,
            read_visibility,
            ptn_static_property_visibility_error_class(read_visibility, class_name, declaring_class),
            property,
            line
        );
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }
    if (!ptn_property_visibility_allows(runtime, set_visibility, declaring_class, access_scope)) {
        free(key);
        if (set_visibility != read_visibility) {
            ptn_throw_property_indirect_set_visibility_error(
                runtime,
                set_visibility,
                declaring_class,
                property,
                access_scope
            );
        } else {
            ptn_throw_property_visibility_error(
                runtime,
                set_visibility,
                ptn_static_property_visibility_error_class(set_visibility, class_name, declaring_class),
                property,
                line
            );
        }
        return ptn_reference_value(ptn_reference_new_owned(ptn_null()));
    }

    PtnValue *slot = ptn_symbols_value_slot(ptn_runtime_static_property_table(runtime), key);
    if (slot == NULL) {
        PtnSymbolTable *static_properties = ptn_runtime_static_property_table(runtime);
        ptn_symbols_set(static_properties, key, ptn_null());
        slot = ptn_symbols_value_slot(static_properties, key);
        if (slot == NULL) {
            free(key);
            return ptn_runtime_undeclared_static_property(runtime, class_name, property, line);
        }
    }
    if (slot->type != PTN_REFERENCE) {
        PtnValue current = *slot;
        *slot = ptn_reference_value(ptn_reference_new_owned(current));
    }
    PtnObjectPropertyMetadata metadata;
    if (ptn_runtime_static_property_metadata(
        runtime,
        key,
        declaring_class,
        property,
        &metadata
    )) {
        ptn_reference_adopt_property_type(slot->as.reference, &metadata);
    }
    PtnValue reference = ptn_value_clone(*slot);
    free(key);
    return reference;
}

static PTN_UNUSED int ptn_runtime_validate_static_property_write(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    size_t line,
    int indirect_write
) {
    class_name = ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return 0;
    }
    if (!ptn_runtime_static_member_class_exists(runtime, class_name)) {
        ptn_throw_class_not_found_error(runtime, class_name, line);
        return 0;
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        PtnValue missing = ptn_runtime_undeclared_static_property(runtime, class_name, property, line);
        ptn_value_destroy(&missing);
        return 0;
    }

    PtnValue read_visibility_value;
    PtnValue set_visibility_value;
    PtnPropertyVisibility read_visibility = PTN_PROPERTY_PUBLIC;
    PtnPropertyVisibility set_visibility = PTN_PROPERTY_PUBLIC;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_read_visibility_table(runtime),
            key,
            &read_visibility_value
        ) &&
        ptn_value_deref(read_visibility_value).type == PTN_INT
    ) {
        read_visibility = (PtnPropertyVisibility)ptn_value_deref(read_visibility_value).as.integer;
    }
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_set_visibility_table(runtime),
            key,
            &set_visibility_value
        ) &&
        ptn_value_deref(set_visibility_value).type == PTN_INT
    ) {
        set_visibility = (PtnPropertyVisibility)ptn_value_deref(set_visibility_value).as.integer;
    }
    free(key);
    if (!ptn_property_visibility_allows(runtime, set_visibility, declaring_class, access_scope)) {
        if (set_visibility != read_visibility) {
            if (indirect_write) {
                ptn_throw_property_indirect_set_visibility_error(
                    runtime,
                    set_visibility,
                    declaring_class,
                    property,
                    access_scope
                );
            } else {
                ptn_throw_property_set_visibility_error(
                    runtime,
                    set_visibility,
                    declaring_class,
                    property,
                    access_scope
                );
            }
        } else {
            ptn_throw_property_visibility_error(
                runtime,
                set_visibility,
                ptn_static_property_visibility_error_class(set_visibility, class_name, declaring_class),
                property,
                line
            );
        }
        return 0;
    }
    return 1;
}

static PTN_UNUSED PtnValue ptn_runtime_write_static_property_impl(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line,
    int indirect_write,
    int reference_context
) {
    class_name = ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return ptn_null();
    }
    if (!ptn_runtime_static_member_class_exists(runtime, class_name)) {
        ptn_throw_class_not_found_error(runtime, class_name, line);
        return ptn_null();
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        return ptn_runtime_undeclared_static_property(runtime, class_name, property, line);
    }
    if (!ptn_runtime_emit_static_trait_property_deprecation(
            runtime,
            declaring_class,
            property,
            line
        )) {
        free(key);
        return ptn_null();
    }
    PtnValue read_visibility_value;
    PtnValue set_visibility_value;
    PtnPropertyVisibility read_visibility = PTN_PROPERTY_PUBLIC;
    PtnPropertyVisibility set_visibility = PTN_PROPERTY_PUBLIC;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_read_visibility_table(runtime),
            key,
            &read_visibility_value
        ) &&
        ptn_value_deref(read_visibility_value).type == PTN_INT
    ) {
        read_visibility = (PtnPropertyVisibility)ptn_value_deref(read_visibility_value).as.integer;
    }
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_set_visibility_table(runtime),
            key,
            &set_visibility_value
        ) &&
        ptn_value_deref(set_visibility_value).type == PTN_INT
    ) {
        set_visibility = (PtnPropertyVisibility)ptn_value_deref(set_visibility_value).as.integer;
    }
    if (!ptn_property_visibility_allows(runtime, set_visibility, declaring_class, access_scope)) {
        free(key);
        if (set_visibility != read_visibility) {
            if (indirect_write) {
                ptn_throw_property_indirect_set_visibility_error(
                    runtime,
                    set_visibility,
                    declaring_class,
                    property,
                    access_scope
                );
            } else {
                ptn_throw_property_set_visibility_error(
                    runtime,
                    set_visibility,
                    declaring_class,
                    property,
                    access_scope
                );
            }
        } else {
            ptn_throw_property_visibility_error(
                runtime,
                set_visibility,
                ptn_static_property_visibility_error_class(set_visibility, class_name, declaring_class),
                property,
                line
            );
        }
        return ptn_null();
    }
    PtnSymbolTable *static_properties = ptn_runtime_static_property_table(runtime);
    PtnObjectPropertyMetadata metadata;
    PtnObjectPropertyMetadata *metadata_ptr =
        ptn_runtime_static_property_metadata(runtime, key, declaring_class, property, &metadata)
            ? &metadata
            : NULL;
    PtnValue current;
    if (ptn_symbols_get(static_properties, key, &current) && current.type == PTN_REFERENCE) {
        if (metadata_ptr != NULL) {
            ptn_reference_adopt_property_type(current.as.reference, metadata_ptr);
        }
        if (!indirect_write) {
            if (!reference_context && metadata_ptr != NULL) {
                PtnValue directly_coerced = ptn_null();
                if (!ptn_runtime_static_property_type_coerce_assignment(
                    runtime,
                    metadata_ptr,
                    value,
                    0,
                    line,
                    &directly_coerced
                )) {
                    free(key);
                    return ptn_null();
                }
                ptn_value_destroy(&directly_coerced);
            }
            PtnValue result = ptn_null();
            if (ptn_reference_assign_publish_first_result_with_context(
                runtime,
                current.as.reference,
                value,
                1,
                &result
            )) {
                ptn_symbols_set(
                    ptn_runtime_static_property_initialized_table(runtime),
                    key,
                    ptn_bool(1)
                );
                free(key);
                return result;
            }
            free(key);
            return ptn_null();
        }
        PtnValue result = ptn_null();
        if (ptn_reference_assign_result(runtime, current.as.reference, value, &result)) {
            ptn_symbols_set(
                ptn_runtime_static_property_initialized_table(runtime),
                key,
                ptn_bool(1)
            );
            free(key);
            return result;
        }
        free(key);
        return ptn_null();
    }
    PtnValue result = ptn_null();
    if (!ptn_runtime_static_property_type_coerce_assignment(
        runtime,
        metadata_ptr,
        value,
        0,
        line,
        &result
    )) {
        free(key);
        return ptn_null();
    }
    ptn_symbols_set_with_runtime_scope(static_properties, key, result, runtime);
    ptn_symbols_set(
        ptn_runtime_static_property_initialized_table(runtime),
        key,
        ptn_bool(1)
    );
    free(key);
    return result;
}

static PTN_UNUSED PtnValue ptn_runtime_write_static_property(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_runtime_write_static_property_impl(
        runtime,
        class_name,
        property,
        access_scope,
        value,
        line,
        0,
        1
    );
}

static PTN_UNUSED PtnValue ptn_runtime_write_static_property_direct(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_runtime_write_static_property_impl(
        runtime,
        class_name,
        property,
        access_scope,
        value,
        line,
        0,
        0
    );
}

static PTN_UNUSED PtnValue ptn_runtime_write_static_property_indirect(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    PtnValue value,
    size_t line
) {
    return ptn_runtime_write_static_property_impl(
        runtime,
        class_name,
        property,
        access_scope,
        value,
        line,
        1,
        1
    );
}

static PTN_UNUSED void ptn_runtime_bind_static_property_reference(
    PtnRuntime *runtime,
    const char *class_name,
    const char *property,
    const char *access_scope,
    PtnValue reference,
    size_t line
) {
    if (reference.type != PTN_REFERENCE) {
        ptn_abort_out_of_memory();
    }
    class_name = ptn_runtime_maybe_autoload_static_member_class(runtime, class_name, line);
    if (runtime->exceptions->active_exception != NULL) {
        return;
    }
    if (!ptn_runtime_static_member_class_exists(runtime, class_name)) {
        ptn_throw_class_not_found_error(runtime, class_name, line);
        return;
    }
    const char *declaring_class = NULL;
    char *key = ptn_runtime_resolve_static_property_key(
        runtime,
        class_name,
        property,
        &declaring_class
    );
    if (key == NULL) {
        PtnValue missing = ptn_runtime_undeclared_static_property(runtime, class_name, property, line);
        ptn_value_destroy(&missing);
        return;
    }
    PtnValue read_visibility_value;
    PtnValue set_visibility_value;
    PtnPropertyVisibility read_visibility = PTN_PROPERTY_PUBLIC;
    PtnPropertyVisibility set_visibility = PTN_PROPERTY_PUBLIC;
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_read_visibility_table(runtime),
            key,
            &read_visibility_value
        ) &&
        ptn_value_deref(read_visibility_value).type == PTN_INT
    ) {
        read_visibility = (PtnPropertyVisibility)ptn_value_deref(read_visibility_value).as.integer;
    }
    if (
        ptn_symbols_get(
            ptn_runtime_static_property_set_visibility_table(runtime),
            key,
            &set_visibility_value
        ) &&
        ptn_value_deref(set_visibility_value).type == PTN_INT
    ) {
        set_visibility = (PtnPropertyVisibility)ptn_value_deref(set_visibility_value).as.integer;
    }
    if (!ptn_property_visibility_allows(runtime, read_visibility, declaring_class, access_scope)) {
        free(key);
        ptn_throw_property_visibility_error(
            runtime,
            read_visibility,
            ptn_static_property_visibility_error_class(read_visibility, class_name, declaring_class),
            property,
            line
        );
        return;
    }
    if (!ptn_property_visibility_allows(runtime, set_visibility, declaring_class, access_scope)) {
        free(key);
        if (set_visibility != read_visibility) {
            ptn_throw_property_indirect_set_visibility_error(
                runtime,
                set_visibility,
                declaring_class,
                property,
                access_scope
            );
        } else {
            ptn_throw_property_visibility_error(
                runtime,
                set_visibility,
                ptn_static_property_visibility_error_class(set_visibility, class_name, declaring_class),
                property,
                line
            );
        }
        return;
    }
    PtnObjectPropertyMetadata metadata;
    PtnObjectPropertyMetadata *metadata_ptr =
        ptn_runtime_static_property_metadata(runtime, key, declaring_class, property, &metadata)
            ? &metadata
            : NULL;
    if (metadata_ptr != NULL) {
        PtnValue coerced = ptn_null();
        if (!ptn_runtime_static_property_type_coerce_assignment(
            runtime,
            metadata_ptr,
            reference,
            0,
            line,
            &coerced
        )) {
            free(key);
            return;
        }
        if (reference.as.reference->property_type_kind != PTN_PROPERTY_TYPE_NONE) {
            PtnValue existing_coerced = ptn_null();
            if (!ptn_property_reference_coerce_assignment(
                runtime,
                reference.as.reference,
                reference,
                1,
                line,
                &existing_coerced
            )) {
                ptn_value_destroy(&coerced);
                free(key);
                return;
            }
            if (!ptn_compare_identical(runtime, existing_coerced, coerced, line)) {
                PtnReferencePropertyTypeSource existing =
                    ptn_reference_primary_property_type_source(reference.as.reference);
                ptn_throw_reference_property_bind_incompatibility(
                    runtime,
                    reference.as.reference->value,
                    &existing,
                    metadata_ptr
                );
                ptn_value_destroy(&existing_coerced);
                ptn_value_destroy(&coerced);
                free(key);
                return;
            }
            ptn_value_destroy(&existing_coerced);
        }
        ptn_value_destroy(&reference.as.reference->value);
        reference.as.reference->value = coerced;
        ptn_reference_adopt_property_type(reference.as.reference, metadata_ptr);
    }
    PtnValue *old_slot = ptn_symbols_value_slot(ptn_runtime_static_property_table(runtime), key);
    if (old_slot != NULL && old_slot->type == PTN_REFERENCE && metadata_ptr != NULL) {
        ptn_reference_forget_property_type(old_slot->as.reference, metadata_ptr);
    }
    ptn_symbols_set(ptn_runtime_static_property_table(runtime), key, reference);
    ptn_symbols_set(
        ptn_runtime_static_property_initialized_table(runtime),
        key,
        ptn_bool(1)
    );
    free(key);
}

static PTN_UNUSED int ptn_exception_matches(PtnRuntime *runtime, const char *type_name) {
    if (runtime->exceptions->active_exception == NULL) {
        return 0;
    }
    const char *class_name = runtime->exceptions->active_exception->class_name;
    if (ptn_exception_type_matches_name(class_name, type_name)) {
        return 1;
    }
    if (type_name[0] == '\\') {
        type_name++;
    }
    return ptn_declared_class_is_same_or_descendant(class_name, type_name);
}

static PTN_UNUSED PtnValue ptn_current_exception_value(PtnRuntime *runtime) {
    if (runtime->exceptions->active_exception == NULL) {
        return ptn_null();
    }
    return ptn_exception_borrow(runtime->exceptions->active_exception);
}

static PTN_UNUSED void ptn_clear_exception(PtnRuntime *runtime) {
    PtnException *exception = runtime->exceptions->active_exception;
    runtime->exceptions->active_exception = NULL;
    ptn_exception_free(exception);
    if (runtime->exceptions->active_exception == NULL) {
        runtime->generator_chained_exception_during_unwind = 0;
    }
}

static PTN_UNUSED int ptn_runtime_bind_catch_variable(PtnRuntime *runtime, const char *name, size_t line) {
    if (
        runtime == NULL ||
        runtime->exceptions == NULL ||
        runtime->exceptions->active_exception == NULL
    ) {
        return 1;
    }
    PtnException *caught_exception = runtime->exceptions->active_exception;
    ptn_exception_retain(caught_exception);
    runtime->exceptions->active_exception = NULL;
    ptn_exception_free(caught_exception);

    PtnTryFrame frame;
    int ok = 1;
    ptn_try_frame_push(runtime, &frame);
    if (setjmp(frame.jump) == 0) {
        PtnValue caught_value = ptn_exception_borrow(caught_exception);
        PtnValue result = ptn_runtime_write_variable_result_at(runtime, name, caught_value, line);
        ptn_value_destroy(&result);
        ptn_try_frame_pop(runtime, &frame);
    } else {
        ok = 0;
        ptn_try_frame_pop(runtime, &frame);
    }
    ptn_exception_free(caught_exception);
    return ok && runtime->exceptions->active_exception == NULL;
}

static PTN_UNUSED void ptn_rethrow_exception(PtnRuntime *runtime) {
    PtnException *exception = runtime->exceptions->active_exception;
    if (exception == NULL) {
        return;
    }
    if (runtime->exceptions->try_frame != NULL) {
        longjmp(runtime->exceptions->try_frame->jump, 1);
    }
    ptn_output_buffer_flush_all(runtime);
    ptn_emit_uncaught_exception(runtime, exception);
    PtnRuntime *root = runtime->lifecycle_root != NULL ? runtime->lifecycle_root : runtime;
    if (root->session_save_handler_shutdown_warning_pending) {
        root->session_save_handler_shutdown_warning_pending = 0;
        if (runtime->diagnostics.display_errors) {
            fputs(
                "\nWarning: PHP Request Shutdown: Cannot call session save handler in a recursive manner in Unknown on line 0\n",
                stderr
            );
        }
    }
    ptn_runtime_shutdown_before_exit(runtime);
    exit(255);
}

static PTN_UNUSED int ptn_object_is_internal_or_descendant(PtnValue receiver, const char *class_name) {
    return receiver.type == PTN_OBJECT &&
        ptn_declared_class_is_same_or_descendant(receiver.as.object->class_name, class_name);
}

static PTN_UNUSED const char *ptn_internal_no_discard_method_warning(PtnValue receiver, const char *method_name) {
    receiver = ptn_value_deref(receiver);
    const char *class_name = NULL;
    if (receiver.type == PTN_OBJECT) {
        class_name = receiver.as.object->class_name;
    } else if (receiver.type == PTN_EXCEPTION) {
        class_name = receiver.as.exception->class_name;
    } else if (receiver.type == PTN_CLOSURE) {
        class_name = "Closure";
    }
    if (
        class_name != NULL &&
        ptn_ascii_case_equal(class_name, "DateTimeImmutable") &&
        ptn_ascii_case_equal(method_name, "setTimestamp")
    ) {
        return "The return value of method DateTimeImmutable::setTimestamp() should either be used or intentionally ignored by casting it as (void), as DateTimeImmutable::setTimestamp() does not modify the object itself";
    }
    return NULL;
}

static PTN_UNUSED void ptn_emit_no_discard_for_internal_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name,
    size_t line
) {
    const char *message = ptn_internal_no_discard_method_warning(receiver, method_name);
    if (message != NULL) {
        ptn_emit_user_warning(&runtime->diagnostics, message, line);
    }
}

static PTN_UNUSED void ptn_throw_undefined_method_for_receiver(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t line
) {
    receiver = ptn_value_deref(receiver);
    const char *class_name = NULL;
    if (receiver.type == PTN_OBJECT) {
        if (ptn_object_is_incomplete_class(receiver.as.object)) {
            ptn_throw_incomplete_object_method_call(runtime, receiver.as.object, line);
            return;
        }
        class_name = receiver.as.object->class_name;
    } else if (receiver.type == PTN_EXCEPTION) {
        class_name = receiver.as.exception->class_name;
    } else if (receiver.type == PTN_CLOSURE) {
        class_name = "Closure";
    }
    if (class_name == NULL) {
        ptn_throw_exception(runtime, "Error", "Call to undefined method");
        return;
    }
    int needed = snprintf(NULL, 0, "Call to undefined method %s::%s()", class_name, name);
    if (needed < 0) {
        ptn_abort_out_of_memory();
    }
    char *message = malloc((size_t)needed + 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    snprintf(message, (size_t)needed + 1, "Call to undefined method %s::%s()", class_name, name);
    ptn_throw_exception_owned_message_at(
        runtime,
        "Error",
        message,
        runtime != NULL ? runtime->source_path : NULL,
        line
    );
}

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static int64_t ptn_internal_expect_integer_arg(
    PtnRuntime *runtime,
    const char *function_name,
    size_t position,
    const char *argument_name,
    PtnValue value,
    size_t line
);

static PTN_UNUSED PtnValue ptn_datetime_immutable_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    (void)receiver;
    if (ptn_ascii_case_equal(name, "setTimestamp")) {
        if (argc != 1) {
            char message[128];
            int written = snprintf(
                message,
                sizeof(message),
                "DateTimeImmutable::setTimestamp() expects exactly 1 argument, %zu given",
                argc
            );
            if (written < 0 || (size_t)written >= sizeof(message)) {
                ptn_abort_out_of_memory();
            }
            ptn_throw_exception(runtime, "ArgumentCountError", message);
            return ptn_null();
        }
        (void)ptn_internal_expect_integer_arg(
            runtime,
            "DateTimeImmutable::setTimestamp",
            1,
            "timestamp",
            args[0],
            line
        );
        if (runtime->exceptions->active_exception != NULL) {
            return ptn_null();
        }
        return ptn_object_new_shell(runtime, "DateTimeImmutable");
    }
    ptn_throw_undefined_method_for_receiver(runtime, receiver, name, line);
    return ptn_null();
}
#endif

static PTN_UNUSED PtnValue ptn_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
) {
    (void)args;
    receiver = ptn_value_deref(receiver);
    if (receiver.type == PTN_OBJECT && ptn_object_is_generator(receiver.as.object)) {
        if (ptn_ascii_case_equal(name, "current")) {
            if (argc != 0) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::current() expects exactly 0 arguments"
                );
                return ptn_null();
            }
            return ptn_generator_current(runtime, receiver, line);
        }
        if (ptn_ascii_case_equal(name, "getReturn")) {
            if (argc != 0) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::getReturn() expects exactly 0 arguments"
                );
                return ptn_null();
            }
            return ptn_generator_get_return(runtime, receiver, line);
        }
        if (ptn_ascii_case_equal(name, "key")) {
            if (argc != 0) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::key() expects exactly 0 arguments"
                );
                return ptn_null();
            }
            return ptn_generator_key(runtime, receiver, line);
        }
        if (ptn_ascii_case_equal(name, "next")) {
            if (argc != 0) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::next() expects exactly 0 arguments"
                );
                return ptn_null();
            }
            return ptn_generator_next(runtime, receiver, line);
        }
        if (ptn_ascii_case_equal(name, "rewind")) {
            if (argc != 0) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::rewind() expects exactly 0 arguments"
                );
                return ptn_null();
            }
            return ptn_generator_rewind(runtime, receiver, line);
        }
        if (ptn_ascii_case_equal(name, "send")) {
            if (argc != 1) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::send() expects exactly 1 argument"
                );
                return ptn_null();
            }
            return ptn_generator_send(runtime, receiver, args[0], line);
        }
        if (ptn_ascii_case_equal(name, "throw")) {
            if (argc != 1) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::throw() expects exactly 1 argument"
                );
                return ptn_null();
            }
            return ptn_generator_throw(runtime, receiver, args[0], line);
        }
        if (ptn_ascii_case_equal(name, "valid")) {
            if (argc != 0) {
                ptn_throw_exception(
                    runtime,
                    "ArgumentCountError",
                    "Generator::valid() expects exactly 0 arguments"
                );
                return ptn_null();
            }
            return ptn_generator_valid(runtime, receiver, line);
        }
    }
    int is_throwable_receiver = receiver.type == PTN_EXCEPTION ||
        (receiver.type == PTN_OBJECT && ptn_object_is_declared_throwable(runtime, receiver.as.object));
    if (receiver.type == PTN_EXCEPTION && ptn_exception_name_equal(name, "__construct")) {
        return ptn_exception_reconstruct(runtime, receiver, argc, args, line);
    }
    if (is_throwable_receiver && (
        ptn_exception_name_equal(name, "getMessage") ||
        ptn_exception_name_equal(name, "getCode") ||
        ptn_exception_name_equal(name, "getFile") ||
        ptn_exception_name_equal(name, "getLine") ||
        ptn_exception_name_equal(name, "getPrevious") ||
        ptn_exception_name_equal(name, "getTrace") ||
        ptn_exception_name_equal(name, "getTraceAsString") ||
        ptn_exception_name_equal(name, "getSeverity") ||
        ptn_exception_name_equal(name, "__toString")
    )) {
        if (argc != 0) {
            ptn_throw_exception(
                runtime,
                "ArgumentCountError",
                "Too many arguments to exception method"
            );
        }
        if (ptn_exception_name_equal(name, "getMessage")) {
            if (receiver.type == PTN_EXCEPTION) {
                return ptn_owned_string_len(
                    ptn_duplicate_string_len(
                        receiver.as.exception->message,
                        receiver.as.exception->message_len
                    ),
                    receiver.as.exception->message_len
                );
            }
            PtnStringOperand message = ptn_object_exception_message(runtime, receiver, line);
            return ptn_owned_string_len(message.owned, message.len);
        }
        if (ptn_exception_name_equal(name, "getCode")) {
            return ptn_int(ptn_throwable_int_property(runtime, receiver, "code", 0, line));
        }
        if (ptn_exception_name_equal(name, "getFile")) {
            return ptn_throwable_file_value(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getLine")) {
            return ptn_int(ptn_throwable_int_property(runtime, receiver, "line", 0, line));
        }
        if (ptn_exception_name_equal(name, "getPrevious")) {
            return ptn_throwable_previous_value(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getTrace")) {
            return ptn_throwable_trace_value(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getTraceAsString")) {
            return ptn_throwable_trace_string(runtime, receiver, line);
        }
        if (ptn_exception_name_equal(name, "getSeverity")) {
            return ptn_int(ptn_throwable_int_property(runtime, receiver, "severity", PTN_E_ERROR, line));
        }
        if (ptn_exception_name_equal(name, "__toString")) {
            return ptn_throwable_to_string(runtime, receiver, line);
        }
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_hash_context(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_hash_context_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_bcmath_number(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_bcmath_number_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_weak_map(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_weak_map_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_weak_reference(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_weak_reference_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_internal_class_name_is_reflection_class(receiver.as.object->class_name) ||
            ptn_internal_class_name_is_reflection_object(receiver.as.object->class_name))
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_class_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_function(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_function_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_generator(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_generator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_fiber(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_fiber_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_extension(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_extension_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_zend_extension(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_zend_extension_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_ascii_case_equal(receiver.as.object->class_name, "ReflectionClassConstant")
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_class_constant_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_ascii_case_equal(receiver.as.object->class_name, "ReflectionConstant")
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_constant_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_ascii_case_equal(receiver.as.object->class_name, "ReflectionAttribute")
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_attribute_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_parameter(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_parameter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_type(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_type_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_method(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_method_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_parameter(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_parameter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_type(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_type_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_property(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_property_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_reflection_reference(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_reflection_reference_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_sensitive_parameter_value(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_sensitive_parameter_value_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_internal_class_name_is_attribute(receiver.as.object->class_name) ||
            ptn_internal_class_name_is_deprecated(receiver.as.object->class_name) ||
            ptn_internal_class_name_is_no_discard(receiver.as.object->class_name))
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_attribute_metadata_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_object_is_internal_or_descendant(receiver, "ArrayIterator") ||
            ptn_object_is_internal_or_descendant(receiver, "RecursiveArrayIterator"))
        && ptn_internal_class_method_exists("ArrayIterator", name)
    ) {
        return ptn_array_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "AppendIterator")
        && ptn_internal_class_method_exists("AppendIterator", name)
    ) {
        return ptn_append_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "EmptyIterator")
        && ptn_internal_class_method_exists("EmptyIterator", name)
    ) {
        return ptn_empty_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "ArrayObject")
        && ptn_internal_class_method_exists("ArrayObject", name)
    ) {
        return ptn_array_object_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SplFixedArray")
        && ptn_internal_class_method_exists("SplFixedArray", name)
    ) {
        return ptn_spl_fixed_array_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SplObjectStorage")
        && ptn_internal_class_method_exists("SplObjectStorage", name)
    ) {
        return ptn_spl_object_storage_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_object_is_internal_or_descendant(receiver, "SplHeap") ||
            ptn_object_is_internal_or_descendant(receiver, "SplPriorityQueue"))
        && (ptn_internal_class_method_exists("SplHeap", name) ||
            ptn_internal_class_method_exists("SplPriorityQueue", name))
    ) {
        return ptn_spl_heap_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_object_is_internal_or_descendant(receiver, "SplDoublyLinkedList") ||
            ptn_object_is_internal_or_descendant(receiver, "SplQueue") ||
            ptn_object_is_internal_or_descendant(receiver, "SplStack"))
        && (ptn_internal_class_method_exists("SplDoublyLinkedList", name) ||
            ptn_internal_class_method_exists("SplQueue", name))
    ) {
        return ptn_spl_doubly_linked_list_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SplFileObject")
        && (ptn_internal_class_method_exists("SplFileObject", name) ||
            ptn_internal_class_method_exists("SplFileInfo", name))
    ) {
        return ptn_spl_file_object_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "DirectoryIterator")
        && ptn_internal_class_method_exists("DirectoryIterator", name)
    ) {
        return ptn_directory_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SplFileInfo")
        && ptn_internal_class_method_exists("SplFileInfo", name)
    ) {
        return ptn_spl_file_info_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_directory(receiver.as.object->class_name)
        && ptn_internal_class_method_exists("Directory", name)
    ) {
        return ptn_directory_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SessionHandler")
        && ptn_internal_class_method_exists("SessionHandler", name)
    ) {
        return ptn_session_handler_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_zip_archive(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_zip_archive_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_object_is_internal_or_descendant(receiver, "SoapClient") ||
            ptn_object_is_internal_or_descendant(receiver, "SoapServer") ||
            ptn_object_is_internal_or_descendant(receiver, "SoapHeader"))
        && (ptn_internal_class_method_exists("SoapClient", name) ||
            ptn_internal_class_method_exists("SoapServer", name) ||
            ptn_internal_class_method_exists("SoapHeader", name))
    ) {
        return ptn_soap_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "CallbackFilterIterator")
        && ptn_internal_class_method_exists("CallbackFilterIterator", name)
    ) {
        return ptn_callback_filter_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "LimitIterator")
        && ptn_internal_class_method_exists("LimitIterator", name)
    ) {
        return ptn_limit_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "RegexIterator")
        && ptn_internal_class_method_exists("RegexIterator", name)
    ) {
        return ptn_regex_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "CachingIterator")
        && (ptn_internal_class_method_exists("CachingIterator", name) ||
            ptn_internal_class_name_is_caching_iterator(receiver.as.object->class_name))
    ) {
        return ptn_iterator_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_object_is_internal_or_descendant(receiver, "FilterIterator") ||
            ptn_object_is_internal_or_descendant(receiver, "InfiniteIterator") ||
            ptn_object_is_internal_or_descendant(receiver, "NoRewindIterator") ||
            ptn_object_is_internal_or_descendant(receiver, "IteratorIterator"))
    ) {
        return ptn_iterator_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "MultipleIterator")
        && ptn_internal_class_method_exists("MultipleIterator", name)
    ) {
        return ptn_multiple_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "RecursiveIteratorIterator")
        && ptn_internal_class_method_exists("RecursiveIteratorIterator", name)
    ) {
        return ptn_recursive_iterator_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_object_is_internal_or_descendant(receiver, "IntlBreakIterator") ||
            ptn_object_is_internal_or_descendant(receiver, "IntlRuleBasedBreakIterator") ||
            ptn_object_is_internal_or_descendant(receiver, "IntlCodePointBreakIterator") ||
            ptn_object_is_internal_or_descendant(receiver, "IntlPartsIterator"))
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_intl_break_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "IntlDateFormatter")
        && ptn_internal_class_method_exists("IntlDateFormatter", name)
    ) {
        return ptn_intl_date_formatter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "IntlCalendar")
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_intl_calendar_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "IntlTimeZone")
        && ptn_internal_class_method_exists("IntlTimeZone", name)
    ) {
        return ptn_intl_timezone_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "IntlIterator")
        && ptn_internal_class_method_exists("IntlIterator", name)
    ) {
        return ptn_intl_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "MessageFormatter")
        && ptn_internal_class_method_exists("MessageFormatter", name)
    ) {
        return ptn_intl_message_formatter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "IntlListFormatter")
        && ptn_internal_class_method_exists("IntlListFormatter", name)
    ) {
        return ptn_intl_list_formatter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "NumberFormatter")
        && ptn_internal_class_method_exists("NumberFormatter", name)
    ) {
        return ptn_intl_number_formatter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "Collator")
        && ptn_internal_class_method_exists("Collator", name)
    ) {
        return ptn_intl_collator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "Spoofchecker")
        && ptn_internal_class_method_exists("Spoofchecker", name)
    ) {
        return ptn_intl_spoofchecker_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "UConverter")
        && ptn_internal_class_method_exists("UConverter", name)
    ) {
        return ptn_intl_uconverter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_object_is_internal_or_descendant(receiver, "DateTime") ||
            ptn_object_is_internal_or_descendant(receiver, "DateTimeImmutable"))
        && (ptn_internal_class_method_exists("DateTime", name) ||
            ptn_internal_class_method_exists("DateTimeImmutable", name))
    ) {
        return ptn_datetime_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "DateTimeZone")
        && ptn_internal_class_method_exists("DateTimeZone", name)
    ) {
        return ptn_datetime_zone_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "DateInterval")
        && ptn_internal_class_method_exists("DateInterval", name)
    ) {
        return ptn_date_interval_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_runtime_declared_class_is_same_or_descendant(
            runtime,
            receiver.as.object->class_name,
            "DatePeriod"
        )
        && ptn_internal_class_method_exists("DatePeriod", name)
    ) {
        return ptn_date_period_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_internal_iterator(receiver.as.object->class_name)
        && ptn_internal_class_method_exists("InternalIterator", name)
    ) {
        return ptn_internal_iterator_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_dom(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_dom_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_simplexml(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_simplexml_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "XMLReader")
        && ptn_internal_class_method_exists("XMLReader", name)
    ) {
        return ptn_xml_reader_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_internal_class_name_is_xml_writer(receiver.as.object->class_name)
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_xmlwriter_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && (ptn_internal_class_name_is_uri_rfc3986_uri(receiver.as.object->class_name) ||
            ptn_internal_class_name_is_uri_whatwg_url(receiver.as.object->class_name))
        && ptn_internal_class_method_exists(receiver.as.object->class_name, name)
    ) {
        return ptn_uri_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "PDO")
        && ptn_internal_class_method_exists("PDO", name)
    ) {
        return ptn_pdo_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "PDOStatement")
        && ptn_internal_class_method_exists("PDOStatement", name)
    ) {
        return ptn_pdo_statement_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SQLite3")
        && ptn_internal_class_method_exists("SQLite3", name)
    ) {
        return ptn_sqlite3_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SQLite3Stmt")
        && ptn_internal_class_method_exists("SQLite3Stmt", name)
    ) {
        return ptn_sqlite3_stmt_call_method(runtime, receiver, name, argc, args, line);
    }
    if (
        receiver.type == PTN_OBJECT
        && ptn_object_is_internal_or_descendant(receiver, "SQLite3Result")
        && ptn_internal_class_method_exists("SQLite3Result", name)
    ) {
        return ptn_sqlite3_result_call_method(runtime, receiver, name, argc, args, line);
    }
#endif
    ptn_throw_undefined_method_for_receiver(runtime, receiver, name, line);
    return ptn_null();
}

static PTN_UNUSED void ptn_runtime_define_constant_len(
    PtnRuntime *runtime,
    const char *name,
    size_t name_len,
    PtnValue value
) {
    PtnStringOperand key = ptn_runtime_global_constant_key_len(name, name_len);
    ptn_symbols_set_len(runtime->constants, key.data, key.len, value);
    ptn_string_operand_free(key);
}

static PTN_UNUSED void ptn_runtime_define_constant(PtnRuntime *runtime, const char *name, PtnValue value) {
    ptn_runtime_define_constant_len(runtime, name, strlen(name), value);
}

static PTN_UNUSED void ptn_runtime_record_constant_source_len(
    PtnRuntime *runtime,
    const char *name,
    size_t name_len,
    const char *source_path
) {
    if (runtime == NULL || source_path == NULL || source_path[0] == '\0') {
        return;
    }
    PtnValue source = ptn_string(source_path);
    PtnStringOperand key = ptn_runtime_global_constant_key_len(name, name_len);
    ptn_symbols_set_len(runtime->constant_sources, key.data, key.len, source);
    ptn_string_operand_free(key);
}

static PTN_UNUSED void ptn_runtime_record_constant_source(
    PtnRuntime *runtime,
    const char *name,
    const char *source_path
) {
    ptn_runtime_record_constant_source_len(runtime, name, strlen(name), source_path);
}

static PTN_UNUSED void ptn_runtime_define_constant_with_source_len(
    PtnRuntime *runtime,
    const char *name,
    size_t name_len,
    PtnValue value,
    const char *source_path
) {
    ptn_runtime_define_constant_len(runtime, name, name_len, value);
    ptn_runtime_record_constant_source_len(runtime, name, name_len, source_path);
}

static PTN_UNUSED void ptn_runtime_define_constant_with_source(
    PtnRuntime *runtime,
    const char *name,
    PtnValue value,
    const char *source_path
) {
    ptn_runtime_define_constant_with_source_len(runtime, name, strlen(name), value, source_path);
}

static PTN_UNUSED PtnValue ptn_runtime_constant_source_file(PtnRuntime *runtime, const char *name) {
    PtnValue source;
    PtnStringOperand key = ptn_runtime_global_constant_key_len(name, strlen(name));
    int found =
        runtime != NULL &&
        runtime->constant_sources != NULL &&
        ptn_symbols_get_len(runtime->constant_sources, key.data, key.len, &source);
    ptn_string_operand_free(key);
    if (
        found
    ) {
        return ptn_value_clone_deref(source);
    }
    return ptn_bool(0);
}

static PTN_UNUSED PtnNumber ptn_number_int(int64_t integer) {
    PtnNumber number;
    number.type = PTN_NUMBER_INT;
    number.integer = integer;
    number.floating = (double)integer;
    return number;
}

static PTN_UNUSED PtnNumber ptn_number_float(double floating) {
    PtnNumber number;
    number.type = PTN_NUMBER_FLOAT;
    number.integer = 0;
    number.floating = floating;
    return number;
}

static PTN_UNUSED int ptn_contains_float_marker(const char *start, const char *end) {
    for (const char *cursor = start; cursor < end; cursor++) {
        if (*cursor == '.' || *cursor == 'e' || *cursor == 'E') {
            return 1;
        }
    }
    return 0;
}

static PTN_UNUSED int ptn_string_has_embedded_nul(PtnString string) {
    return memchr(string.data, '\0', string.len) != NULL;
}

static PTN_UNUSED int ptn_numeric_string_can_start(const char *start, const char *limit) {
    if (start >= limit) {
        return 0;
    }
    if (*start == '+' || *start == '-') {
        start++;
        if (start >= limit) {
            return 0;
        }
    }
    if (isdigit((unsigned char)*start)) {
        return 1;
    }
    return *start == '.' && start + 1 < limit && isdigit((unsigned char)start[1]);
}

static PTN_UNUSED PtnNumber ptn_string_to_number(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    const char *numeric_start = start;
    if (*numeric_start == '+' || *numeric_start == '-') {
        numeric_start++;
    }
    if (numeric_start[0] == '0' && (numeric_start[1] == 'x' || numeric_start[1] == 'X')) {
        return ptn_number_int(0);
    }
    if (*start == '\0') {
        return ptn_number_int(0);
    }
    if (!ptn_numeric_string_can_start(start, start + strlen(start))) {
        return ptn_number_int(0);
    }

    char *int_end = NULL;
    errno = 0;
    long long integer = strtoll(start, &int_end, 10);
    int int_errno = errno;

    char *float_end = NULL;
    errno = 0;
    double floating = strtod(start, &float_end);
    if (float_end == start) {
        return ptn_number_int(0);
    }

    if (int_end == float_end && int_errno != ERANGE && !ptn_contains_float_marker(start, int_end)) {
        return ptn_number_int((int64_t)integer);
    }
    return ptn_number_float(floating);
}

static PTN_UNUSED PtnNumber ptn_to_number(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return ptn_number_int(0);
        case PTN_BOOL:
            return ptn_number_int(value.as.boolean ? 1 : 0);
        case PTN_INT:
            return ptn_number_int(value.as.integer);
        case PTN_FLOAT:
            return ptn_number_float(value.as.floating);
        case PTN_STRING:
            if (ptn_string_has_embedded_nul(value.as.string)) {
                return ptn_number_int(0);
            }
            return ptn_string_to_number((const char *)value.as.string.data);
        case PTN_ARRAY:
            return ptn_number_int(value.as.array->len == 0 ? 0 : 1);
        case PTN_OBJECT: {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            PtnNumber simplexml_number;
            if (ptn_simplexml_numeric_value(value, &simplexml_number)) {
                return simplexml_number;
            }
#endif
            return ptn_number_int(1);
        }
        case PTN_CLOSURE:
            return ptn_number_int(1);
        case PTN_EXCEPTION:
            return ptn_number_int(1);
        case PTN_RESOURCE:
            return ptn_number_int(value.as.resource->id);
        case PTN_REFERENCE:
            return ptn_number_int(0);
    }
    return ptn_number_int(0);
}

static PTN_UNUSED int ptn_fast_integer_value(PtnValue value, int64_t *integer) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            *integer = 0;
            return 1;
        case PTN_BOOL:
            *integer = value.as.boolean ? 1 : 0;
            return 1;
        case PTN_INT:
            *integer = value.as.integer;
            return 1;
        case PTN_FLOAT:
        case PTN_STRING:
        case PTN_RESOURCE:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED int ptn_fast_scalar_double(PtnValue value, double *number) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        *number = (double)integer;
        return 1;
    }
    if (value.type == PTN_FLOAT) {
        *number = value.as.floating;
        return 1;
    }
    return 0;
}

static PTN_UNUSED int ptn_is_truthy(PtnValue value) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_NULL:
            return 0;
        case PTN_BOOL:
            return value.as.boolean != 0;
        case PTN_INT:
            return value.as.integer != 0;
        case PTN_FLOAT:
            return value.as.floating != 0.0;
        case PTN_STRING:
            return value.as.string.len != 0 &&
                !(value.as.string.len == 1 && value.as.string.data[0] == '0');
        case PTN_ARRAY:
            return value.as.array->len != 0;
        case PTN_OBJECT: {
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
            int truthy = 1;
            if (ptn_bcmath_number_is_truthy(value, &truthy)) {
                return truthy;
            }
            if (ptn_simplexml_is_truthy(value, &truthy)) {
                return truthy;
            }
#endif
            return 1;
        }
        case PTN_CLOSURE:
            return 1;
        case PTN_EXCEPTION:
            return 1;
        case PTN_RESOURCE:
            return 1;
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

static PTN_UNUSED PtnValue ptn_not(PtnValue value) {
    return ptn_bool(!ptn_is_truthy(value));
}

static PTN_UNUSED int ptn_float_to_int_out_of_range(double value);
static PTN_UNUSED void ptn_emit_bitwise_float_out_of_range_warning(
    PtnDiagnosticSink *diagnostics,
    double value,
    size_t line
);

static PTN_UNUSED void ptn_emit_nan_coercion_warning(PtnRuntime *runtime, const char *type_name, size_t line) {
    if (runtime == NULL) {
        return;
    }
    char message[96];
    int written = snprintf(
        message,
        sizeof(message),
        "unexpected NAN value was coerced to %s",
        type_name
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_spaced_warning(&runtime->diagnostics, message, line);
}

static PTN_UNUSED int64_t ptn_float_string_to_php_integer(double value) {
    if (!isfinite(value)) {
        return 0;
    }
    if (value >= 9223372036854775808.0) {
        return INT64_MAX;
    }
    if (value < -9223372036854775808.0) {
        return INT64_MIN;
    }
    return ptn_float_to_php_integer(value);
}

static PTN_UNUSED PtnValue ptn_cast_int(PtnValue value) {
    value = ptn_value_deref(value);
    int64_t integer = 0;
    if (ptn_fast_integer_value(value, &integer)) {
        return ptn_int(integer);
    }
    if (value.type == PTN_FLOAT) {
        return ptn_int(ptn_float_to_php_integer(value.as.floating));
    }

    if (value.type == PTN_STRING) {
        if (ptn_string_has_embedded_nul(value.as.string)) {
            return ptn_int(0);
        }
        PtnNumber string_number = ptn_string_to_number((const char *)value.as.string.data);
        if (string_number.type == PTN_NUMBER_FLOAT) {
            return ptn_int(ptn_float_string_to_php_integer(string_number.floating));
        }
        return ptn_int(string_number.integer);
    }

    PtnNumber number = ptn_to_number(value);
    if (number.type == PTN_NUMBER_FLOAT) {
        return ptn_int(ptn_float_to_php_integer(number.floating));
    }
    return ptn_int(number.integer);
}

static PTN_UNUSED const char *ptn_numeric_cast_object_class_name(PtnValue value) {
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
            return NULL;
    }
    return NULL;
}

static PTN_UNUSED void ptn_emit_object_numeric_cast_warning(
    PtnRuntime *runtime,
    PtnValue value,
    const char *target_type,
    size_t line
) {
    if (runtime == NULL) {
        return;
    }
    const char *class_name = ptn_numeric_cast_object_class_name(value);
    if (class_name == NULL) {
        return;
    }
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
    PtnNumber simplexml_number;
    if (ptn_simplexml_numeric_value(value, &simplexml_number)) {
        return;
    }
#endif

    char message[256];
    int written = snprintf(
        message,
        sizeof(message),
        "Object of class %s could not be converted to %s",
        class_name,
        target_type
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_emit_spaced_warning(&runtime->diagnostics, message, line);
}

static PTN_UNUSED PtnValue ptn_cast_int_with_runtime(PtnRuntime *runtime, PtnValue value, size_t line) {
    ptn_emit_object_numeric_cast_warning(runtime, value, "int", line);
    PtnValue resolved = ptn_value_deref(value);
    if (runtime != NULL &&
        resolved.type == PTN_FLOAT &&
        ptn_float_to_int_out_of_range(resolved.as.floating)) {
        ptn_emit_bitwise_float_out_of_range_warning(&runtime->diagnostics, resolved.as.floating, line);
    }
    return ptn_cast_int(value);
}

static PTN_UNUSED PtnValue ptn_cast_float(PtnValue value) {
    value = ptn_value_deref(value);
    double fast_number = 0.0;
    if (ptn_fast_scalar_double(value, &fast_number)) {
        return ptn_float(fast_number);
    }

    PtnNumber number = ptn_to_number(value);
    return ptn_float(number.floating);
}

static PTN_UNUSED PtnValue ptn_cast_float_with_runtime(PtnRuntime *runtime, PtnValue value, size_t line) {
    ptn_emit_object_numeric_cast_warning(runtime, value, "float", line);
    return ptn_cast_float(value);
}

static PTN_UNUSED void ptn_abort_arithmetic_error(const char *message) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(1);
}

static PTN_UNUSED void ptn_abort_type_error_at(const char *message, const char *path, size_t line) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputs(" in ", stderr);
    fputs(path, stderr);
    fputs(" on line ", stderr);
    fprintf(stderr, "%zu", line);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED void ptn_abort_control_error(const char *message, const char *path, size_t line) {
    fputs("Fatal error: ", stderr);
    fputs(message, stderr);
    fputs(" in ", stderr);
    fputs(path, stderr);
    fputs(" on line ", stderr);
    fprintf(stderr, "%zu", line);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED int ptn_is_number_type(PtnValue value) {
    value = ptn_value_deref(value);
    return value.type == PTN_INT || value.type == PTN_FLOAT;
}

static PTN_UNUSED int ptn_string_may_be_numeric(const char *string) {
    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }
    if (isdigit((unsigned char)*start) || *start == '+' || *start == '-' || *start == '.') {
        return 1;
    }
    return *start == 'i' || *start == 'I' || *start == 'n' || *start == 'N';
}

static PTN_UNUSED int ptn_is_numeric_string(const char *string, double *number) {
    if (!ptn_string_may_be_numeric(string)) {
        return 0;
    }

    const char *start = string;
    while (isspace((unsigned char)*start)) {
        start++;
    }
    if (*start == '\0') {
        return 0;
    }

    const char *numeric_start = start;
    if (*numeric_start == '+' || *numeric_start == '-') {
        numeric_start++;
    }
    if (numeric_start[0] == '0' && (numeric_start[1] == 'x' || numeric_start[1] == 'X')) {
        return 0;
    }

    char *end = NULL;
    double parsed = strtod(start, &end);
    if (end == start) {
        return 0;
    }
    while (isspace((unsigned char)*end)) {
        end++;
    }
    if (*end != '\0') {
        return 0;
    }
    *number = parsed;
    return 1;
}

static PTN_UNUSED int ptn_comparison_numeric_value(PtnValue value, double *number) {
    value = ptn_value_deref(value);
    switch (value.type) {
        case PTN_INT:
            *number = (double)value.as.integer;
            return 1;
        case PTN_FLOAT:
            *number = value.as.floating;
            return 1;
        case PTN_STRING:
            if (ptn_string_has_embedded_nul(value.as.string)) {
                return 0;
            }
            return ptn_is_numeric_string((const char *)value.as.string.data, number);
        case PTN_RESOURCE:
        case PTN_NULL:
        case PTN_BOOL:
        case PTN_ARRAY:
        case PTN_OBJECT:
        case PTN_CLOSURE:
        case PTN_EXCEPTION:
        case PTN_REFERENCE:
            return 0;
    }
    return 0;
}

enum {
    PTN_COMPARE_LESS = -1,
    PTN_COMPARE_EQUAL = 0,
    PTN_COMPARE_GREATER = 1,
    PTN_COMPARE_UNORDERED = 2
};

static PTN_UNUSED int ptn_compare_numbers(double left, double right) {
    if (isnan(left) || isnan(right)) {
        return PTN_COMPARE_UNORDERED;
    }
    if (left < right) {
        return PTN_COMPARE_LESS;
    }
    if (left > right) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_integers(int64_t left, int64_t right) {
    if (left < right) {
        return PTN_COMPARE_LESS;
    }
    if (left > right) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_strings(const char *left, const char *right) {
    int compared = strcmp(left, right);
    return compared < 0 ? -1 : (compared > 0 ? 1 : 0);
}

static PTN_UNUSED int ptn_compare_string_bytes(
    const unsigned char *left,
    size_t left_len,
    const unsigned char *right,
    size_t right_len
) {
    size_t shared_len = left_len < right_len ? left_len : right_len;
    int compared = shared_len == 0 ? 0 : memcmp(left, right, shared_len);
    if (compared < 0) {
        return PTN_COMPARE_LESS;
    }
    if (compared > 0) {
        return PTN_COMPARE_GREATER;
    }
    if (left_len < right_len) {
        return PTN_COMPARE_LESS;
    }
    if (left_len > right_len) {
        return PTN_COMPARE_GREATER;
    }
    return PTN_COMPARE_EQUAL;
}

static PTN_UNUSED int ptn_compare_value_strings(PtnString left, PtnString right) {
    return ptn_compare_string_bytes(left.data, left.len, right.data, right.len);
}

static PTN_UNUSED void ptn_number_value_to_string(PtnValue value, char *buffer, size_t buffer_len) {
    if (value.type == PTN_INT) {
