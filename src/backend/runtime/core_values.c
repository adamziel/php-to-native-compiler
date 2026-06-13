#ifndef _POSIX_C_SOURCE
#define _POSIX_C_SOURCE 200809L
#endif

#include <ctype.h>
#include <errno.h>
#include <limits.h>
#include <locale.h>
#include <math.h>
#include <stdarg.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#if defined(_WIN32)
#include <direct.h>
#include <process.h>
#else
#include <dirent.h>
#include <regex.h>
#include <sys/utsname.h>
#include <unistd.h>
#endif

#if !defined(_WIN32)
extern char *realpath(const char *path, char *resolved_path);
#endif

#if defined(__GNUC__) || defined(__clang__)
#define PTN_UNUSED __attribute__((unused))
#else
#define PTN_UNUSED
#endif

#if defined(__GNUC__) && !defined(__clang__)
#pragma GCC diagnostic ignored "-Wclobbered"
#endif

#if defined(_WIN32)
#define REG_EXTENDED 0
#define REG_ICASE 0
typedef struct {
    size_t re_nsub;
} regex_t;
typedef struct {
    int rm_so;
    int rm_eo;
} regmatch_t;
#endif

#define PTN_PHP_VERSION "8.4.0"
#define PTN_PHP_MAJOR_VERSION 8
#define PTN_PHP_MINOR_VERSION 4
#define PTN_PHP_RELEASE_VERSION 0
#define PTN_PHP_EXTRA_VERSION ""
#define PTN_PHP_VERSION_ID 80400
#define PTN_PHP_ZTS 0
#define PTN_PHP_DEBUG 0
#define PTN_PHP_SAPI_NAME "cli"
#define PTN_ZEND_VERSION "4.4.0"
#define PTN_PHP_EXTENSION_DIR "."
#if defined(_WIN32)
#define PTN_PHP_OS "WINNT"
#define PTN_PHP_OS_FAMILY "Windows"
#define PTN_PHP_SHLIB_SUFFIX "dll"
#elif defined(__APPLE__)
#define PTN_PHP_OS "Darwin"
#define PTN_PHP_OS_FAMILY "Darwin"
#define PTN_PHP_SHLIB_SUFFIX "dylib"
#elif defined(__linux__)
#define PTN_PHP_OS "Linux"
#define PTN_PHP_OS_FAMILY "Linux"
#define PTN_PHP_SHLIB_SUFFIX "so"
#elif defined(__FreeBSD__)
#define PTN_PHP_OS "FreeBSD"
#define PTN_PHP_OS_FAMILY "BSD"
#define PTN_PHP_SHLIB_SUFFIX "so"
#else
#define PTN_PHP_OS "Unknown"
#define PTN_PHP_OS_FAMILY "Unknown"
#define PTN_PHP_SHLIB_SUFFIX "so"
#endif
#define PTN_ARRAY_INDEX_MIN_ENTRIES 16
#define PTN_SYMBOL_INDEX_MIN_ENTRIES 16
#define PTN_E_ERROR 1
#define PTN_E_WARNING 2
#define PTN_E_PARSE 4
#define PTN_E_NOTICE 8
#define PTN_E_CORE_ERROR 16
#define PTN_E_CORE_WARNING 32
#define PTN_E_COMPILE_ERROR 64
#define PTN_E_COMPILE_WARNING 128
#define PTN_E_USER_ERROR 256
#define PTN_E_USER_WARNING 512
#define PTN_E_USER_NOTICE 1024
#define PTN_E_STRICT 2048
#define PTN_E_RECOVERABLE_ERROR 4096
#define PTN_E_DEPRECATED 8192
#define PTN_E_USER_DEPRECATED 16384
#define PTN_E_ALL 32767
#define PTN_ARRAY_FILTER_USE_BOTH 1
#define PTN_ARRAY_FILTER_USE_KEY 2
#define PTN_STR_PAD_LEFT 0
#define PTN_STR_PAD_RIGHT 1
#define PTN_STR_PAD_BOTH 2
#define PTN_COUNT_NORMAL 0
#define PTN_COUNT_RECURSIVE 1
#define PTN_PATHINFO_DIRNAME 1
#define PTN_PATHINFO_BASENAME 2
#define PTN_PATHINFO_EXTENSION 4
#define PTN_PATHINFO_FILENAME 8
#define PTN_PATHINFO_ALL 15
#define PTN_LC_CTYPE 0
#define PTN_LC_NUMERIC 1
#define PTN_LC_TIME 2
#define PTN_LC_COLLATE 3
#define PTN_LC_MONETARY 4
#define PTN_LC_MESSAGES 5
#define PTN_LC_ALL 6
#define PTN_DEFAULT_PRECISION 14

typedef struct PtnArray PtnArray;
typedef struct PtnClosure PtnClosure;
typedef struct PtnException PtnException;
typedef struct PtnObject PtnObject;
typedef struct PtnReference PtnReference;
typedef struct PtnRuntime PtnRuntime;
typedef struct PtnResource PtnResource;
typedef struct PtnTryFrame PtnTryFrame;

typedef enum {
    PTN_NULL,
    PTN_BOOL,
    PTN_INT,
    PTN_FLOAT,
    PTN_STRING,
    PTN_RESOURCE,
    PTN_ARRAY,
    PTN_OBJECT,
    PTN_CLOSURE,
    PTN_EXCEPTION,
    PTN_REFERENCE
} PtnType;

typedef enum {
    PTN_ARRAY_KEY_INT,
    PTN_ARRAY_KEY_STRING
} PtnArrayKeyType;

typedef struct {
    size_t refcount;
    size_t len;
    unsigned char *data;
} PtnStringPayload;

typedef struct {
    const unsigned char *data;
    size_t len;
    PtnStringPayload *payload;
} PtnString;

typedef struct {
    PtnArrayKeyType type;
    union {
        int64_t integer;
        const char *string;
    } as;
} PtnArrayKey;

typedef struct {
    PtnType type;
    int owned;
    union {
        int boolean;
        int64_t integer;
        double floating;
        PtnString string;
        PtnArray *array;
        PtnObject *object;
        PtnClosure *closure;
        PtnException *exception;
        PtnResource *resource;
        PtnReference *reference;
    } as;
} PtnValue;

typedef struct {
    char *name;
    PtnValue value;
} PtnSymbol;

typedef struct {
    int occupied;
    uint64_t hash;
    size_t symbol_index;
} PtnSymbolIndexSlot;

typedef struct {
    PtnSymbol *items;
    size_t len;
    size_t capacity;
    PtnSymbolIndexSlot *index_slots;
    size_t index_capacity;
} PtnSymbolTable;

struct PtnClosure {
    size_t refcount;
    size_t object_id;
    size_t function_index;
    const char *display_name;
    PtnSymbolTable captures;
};

struct PtnReference {
    size_t refcount;
    PtnValue value;
};

typedef enum {
    PTN_PROPERTY_PUBLIC,
    PTN_PROPERTY_PROTECTED,
    PTN_PROPERTY_PRIVATE
} PtnPropertyVisibility;

typedef struct {
    char *storage_name;
    char *display_name;
    char *declaring_class;
    PtnPropertyVisibility visibility;
} PtnObjectPropertyMetadata;

typedef void (*PtnObjectNativeDataFree)(void *data);

typedef struct {
    int exists;
    PtnValue value;
} PtnLookupResult;

typedef struct {
    int append;
    PtnValue value;
} PtnArrayPathSegment;

typedef struct {
    PtnArrayKey key;
    PtnValue value;
} PtnArrayEntry;

typedef struct {
    int occupied;
    uint64_t hash;
    size_t entry_index;
} PtnArrayIndexSlot;

typedef struct {
    PtnArray *array;
    size_t index;
    size_t length;
    int valid;
    int live;
} PtnArrayIterator;

struct PtnArray {
    size_t refcount;
    size_t debug_hidden_refcount;
    size_t iterator_refcount;
    size_t len;
    size_t capacity;
    PtnArrayEntry *entries;
    PtnArrayIndexSlot *index_slots;
    size_t index_capacity;
    int64_t next_auto_key;
    size_t current_index;
};

struct PtnObject {
    size_t refcount;
    size_t object_id;
    char *class_name;
    PtnArray *properties;
    PtnObjectPropertyMetadata *property_metadata;
    size_t property_metadata_len;
    size_t property_metadata_capacity;
    void *native_data;
    PtnObjectNativeDataFree native_data_free;
    PtnRuntime *lifecycle_runtime;
    int destructor_called;
};

typedef struct {
    int found;
    const char *name;
    int is_internal;
    size_t parameter_count;
    size_t required_parameter_count;
    int is_variadic;
} PtnFunctionMetadata;

typedef struct {
    int has_key;
    PtnValue key;
    PtnValue value;
} PtnArrayLiteralEntry;

typedef struct {
    char *data;
    size_t len;
    size_t capacity;
} PtnStringBuffer;

typedef enum {
    PTN_NUMBER_INT,
    PTN_NUMBER_FLOAT
} PtnNumberType;

typedef struct {
    PtnNumberType type;
    int64_t integer;
    double floating;
} PtnNumber;

typedef struct {
    const char *data;
    char *owned;
    size_t len;
} PtnStringOperand;

typedef struct {
    PtnValue value;
    size_t line;
} PtnConcatOperand;

struct PtnException {
    size_t object_id;
    const char *class_name;
    char *message;
    const char *path;
    size_t line;
};

struct PtnResource {
    size_t refcount;
    int64_t id;
    const char *type_name;
    FILE *stream;
    char *stream_uri;
    char *stream_mode;
    int persistent;
};

typedef struct {
    PtnException *active_exception;
    PtnTryFrame *try_frame;
} PtnExceptionState;

typedef struct {
    size_t argc;
    const PtnValue *args;
    size_t parameter_count;
    const char *const *parameter_names;
} PtnCallFrame;

struct PtnTryFrame {
    jmp_buf jump;
    PtnTryFrame *previous;
};

typedef struct {
    FILE *stream;
    int emitted_deprecation;
    int emitted_warning;
    int suppressed;
    int64_t error_reporting;
} PtnDiagnosticSink;

typedef PtnValue (*PtnMethodDispatchHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
typedef int (*PtnDeclaredMethodExistsHandler)(const char *class_name, const char *method_name);

struct PtnRuntime {
    PtnSymbolTable symbols;
    PtnSymbolTable *global_symbols;
    PtnSymbolTable owned_constants;
    PtnSymbolTable *constants;
    PtnSymbolTable owned_class_constants;
    PtnSymbolTable *class_constants;
    PtnSymbolTable owned_static_properties;
    PtnSymbolTable *static_properties;
    PtnDiagnosticSink diagnostics;
    PtnExceptionState owned_exceptions;
    PtnExceptionState *exceptions;
    PtnCallFrame owned_call_frame;
    PtnCallFrame *call_frame;
    PtnRuntime *lifecycle_root;
    PtnObject **live_objects;
    size_t live_objects_len;
    size_t live_objects_capacity;
    size_t next_object_id;
    PtnMethodDispatchHandler method_dispatch;
    PtnDeclaredMethodExistsHandler declared_method_exists;
    const char *source_path;
    const char *current_function_name;
    size_t call_site_line;
    int warn_by_ref_argument_mismatch;
};

typedef struct {
    size_t string_allocs;
    size_t string_frees;
    size_t string_clones;
    size_t string_retain;
    size_t string_release;
    size_t string_detaches;
    size_t array_allocs;
    size_t array_retain;
    size_t array_release;
    size_t array_frees;
    size_t array_clones;
    size_t array_detaches;
    size_t array_detach_skips;
} PtnCowDebugCounters;

static PtnCowDebugCounters ptn_cow_debug_counters;

static PTN_UNUSED int ptn_is_truthy(PtnValue value);
static void ptn_abort_out_of_memory(void);
static PTN_UNUSED char *ptn_duplicate_string(const char *string);
static void ptn_symbols_free(PtnSymbolTable *symbols);
static PTN_UNUSED void ptn_cow_debug_note_string_alloc(void);
static PTN_UNUSED void ptn_cow_debug_note_string_free(void);
static PTN_UNUSED void ptn_cow_debug_note_string_clone(void);
static PTN_UNUSED void ptn_cow_debug_note_string_retain(void);
static PTN_UNUSED void ptn_cow_debug_note_string_release(void);
static PTN_UNUSED void ptn_cow_debug_note_string_detach(void);
static PTN_UNUSED void ptn_cow_debug_note_array_alloc(void);
static PTN_UNUSED void ptn_cow_debug_note_array_retain(void);
static PTN_UNUSED void ptn_cow_debug_note_array_release(void);
static PTN_UNUSED void ptn_cow_debug_note_array_free(void);
static PTN_UNUSED void ptn_cow_debug_note_array_clone(void);
static PTN_UNUSED void ptn_cow_debug_note_array_detach(void);
static PTN_UNUSED void ptn_cow_debug_note_array_detach_skip(void);
static PTN_UNUSED void ptn_cow_debug_assert_string_refcount(size_t *refcount, const char *operation);
static PTN_UNUSED void ptn_cow_debug_assert_array_refcount(PtnArray *array, const char *operation);
static PTN_UNUSED void ptn_cow_debug_reset(void);
static PTN_UNUSED int ptn_cow_debug_counter(const char *name, size_t *out);
static PTN_UNUSED void ptn_cow_debug_assert_named_counter(const char *name, int64_t expected);
static PTN_UNUSED void ptn_cow_debug_assert_balanced(void);

typedef PtnValue (*PtnInternalFunctionHandler)(PtnRuntime *runtime, size_t argc, const PtnValue *args, size_t line);

typedef struct {
    const char *name;
    size_t min_args;
    size_t max_args;
    PtnInternalFunctionHandler handler;
} PtnInternalFunction;

#define PTN_VARIADIC_ARGS ((size_t)-1)

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_not_found(void) {
    PtnFunctionMetadata metadata;
    metadata.found = 0;
    metadata.name = NULL;
    metadata.is_internal = 0;
    metadata.parameter_count = 0;
    metadata.required_parameter_count = 0;
    metadata.is_variadic = 0;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_found(
    const char *name,
    int is_internal,
    size_t parameter_count,
    size_t required_parameter_count,
    int is_variadic
) {
    PtnFunctionMetadata metadata;
    metadata.found = 1;
    metadata.name = name;
    metadata.is_internal = is_internal;
    metadata.parameter_count = parameter_count;
    metadata.required_parameter_count = required_parameter_count;
    metadata.is_variadic = is_variadic;
    return metadata;
}

static PTN_UNUSED size_t ptn_runtime_alloc_object_id(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return 0;
    }
    PtnRuntime *root = runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
    if (root->next_object_id == 0) {
        root->next_object_id = 1;
    }
    if (root->next_object_id > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return root->next_object_id++;
}

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED int ptn_internal_class_name_is_reflection_function(const char *class_name);
static PTN_UNUSED int ptn_internal_class_method_exists(const char *class_name, const char *method_name);
static PTN_UNUSED PtnValue ptn_reflection_function_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_function_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
#endif

static PTN_UNUSED int ptn_float_precision(void) {
    static int initialized = 0;
    static int precision = PTN_DEFAULT_PRECISION;
    if (!initialized) {
        const char *configured = getenv("PTN_PHP_PRECISION");
        if (configured != NULL && configured[0] != '\0') {
            char *end = NULL;
            errno = 0;
            long parsed = strtol(configured, &end, 10);
            if (errno == 0 && end != configured && *end == '\0' && parsed >= 0 && parsed <= 53) {
                precision = (int)parsed;
            }
        }
        initialized = 1;
    }
    return precision;
}

static PTN_UNUSED void ptn_normalize_scalar_float_exponent(char *buffer) {
    for (char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == 'e' || *cursor == 'E') {
            *cursor = 'E';
            cursor++;
            if (*cursor == '+' || *cursor == '-') {
                cursor++;
            }
            while (*cursor == '0' && isdigit((unsigned char)cursor[1])) {
                memmove(cursor, cursor + 1, strlen(cursor));
            }
            return;
        }
    }
}

static PTN_UNUSED void ptn_scalar_float_ensure_exponent_decimal(char *buffer) {
    for (char *cursor = buffer; *cursor != '\0'; cursor++) {
        if (*cursor == '.') {
            return;
        }
        if (*cursor == 'E') {
            size_t tail_len = strlen(cursor);
            memmove(cursor + 2, cursor, tail_len + 1);
            cursor[0] = '.';
            cursor[1] = '0';
            return;
        }
    }
}

static PTN_UNUSED void ptn_format_scalar_float(double value, char *buffer, size_t buffer_size) {
    int written;
    if (isnan(value)) {
        written = snprintf(buffer, buffer_size, "NAN");
    } else if (isinf(value)) {
        written = snprintf(buffer, buffer_size, signbit(value) ? "-INF" : "INF");
    } else {
        written = snprintf(buffer, buffer_size, "%.*g", ptn_float_precision(), value);
    }
    if (written < 0 || (size_t)written >= buffer_size) {
        ptn_abort_out_of_memory();
    }
    ptn_normalize_scalar_float_exponent(buffer);
    ptn_scalar_float_ensure_exponent_decimal(buffer);
}

static PTN_UNUSED PtnValue ptn_null(void) {
    PtnValue value;
    value.type = PTN_NULL;
    value.owned = 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_bool(int boolean) {
    PtnValue value;
    value.type = PTN_BOOL;
    value.owned = 0;
    value.as.boolean = boolean ? 1 : 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_int(int64_t integer) {
    PtnValue value;
    value.type = PTN_INT;
    value.owned = 0;
    value.as.integer = integer;
    return value;
}

static PTN_UNUSED PtnValue ptn_float(double floating) {
    PtnValue value;
    value.type = PTN_FLOAT;
    value.owned = 0;
    value.as.floating = floating;
    return value;
}

static PTN_UNUSED PtnStringPayload *ptn_string_payload_from_owned(char *string, size_t len) {
    PtnStringPayload *payload = malloc(sizeof(PtnStringPayload));
    if (payload == NULL) {
        free(string);
        ptn_abort_out_of_memory();
    }
    payload->refcount = 1;
    payload->len = len;
    payload->data = (unsigned char *)string;
    payload->data[len] = '\0';
    ptn_cow_debug_note_string_alloc();
    return payload;
}

static PTN_UNUSED void ptn_string_payload_retain(PtnStringPayload *payload) {
    if (payload == NULL) {
        return;
    }
    ptn_cow_debug_assert_string_refcount(&payload->refcount, "retain");
    if (payload->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_note_string_clone();
    ptn_cow_debug_note_string_retain();
    payload->refcount++;
}

static PTN_UNUSED void ptn_string_payload_release(PtnStringPayload *payload) {
    if (payload == NULL) {
        return;
    }
    if (payload->refcount == 0) {
        return;
    }
    ptn_cow_debug_assert_string_refcount(&payload->refcount, "release");
    ptn_cow_debug_note_string_release();
    payload->refcount--;
    if (payload->refcount != 0) {
        return;
    }
    ptn_cow_debug_note_string_free();
    free(payload->data);
    free(payload);
}

static PTN_UNUSED void ptn_string_value_refresh(PtnValue *value) {
    if (value == NULL || value->type != PTN_STRING || value->as.string.payload == NULL) {
        return;
    }
    value->as.string.data = value->as.string.payload->data;
    value->as.string.len = value->as.string.payload->len;
}

static PTN_UNUSED void ptn_string_value_resize(PtnValue *value, size_t new_len) {
    if (value == NULL ||
        value->type != PTN_STRING ||
        !value->owned ||
        value->as.string.payload == NULL ||
        value->as.string.payload->refcount != 1) {
        ptn_abort_out_of_memory();
    }
    if (new_len == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }

    PtnStringPayload *payload = value->as.string.payload;
    size_t old_len = payload->len;
    unsigned char *data = realloc(payload->data, new_len + 1);
    if (data == NULL) {
        ptn_abort_out_of_memory();
    }
    if (new_len > old_len) {
        memset(data + old_len, ' ', new_len - old_len);
    }
    data[new_len] = '\0';
    payload->data = data;
    payload->len = new_len;
    ptn_string_value_refresh(value);
}

static PTN_UNUSED PtnValue ptn_string_literal(const char *string, size_t len) {
    PtnValue value;
    value.type = PTN_STRING;
    value.owned = 0;
    value.as.string.data = (const unsigned char *)string;
    value.as.string.len = len;
    value.as.string.payload = NULL;
    return value;
}

static PTN_UNUSED PtnValue ptn_string(const char *string) {
    return ptn_string_literal(string, strlen(string));
}

static PTN_UNUSED PtnValue ptn_owned_string_len(char *string, size_t len) {
    PtnStringPayload *payload = ptn_string_payload_from_owned(string, len);
    PtnValue value;
    value.type = PTN_STRING;
    value.owned = 1;
    value.as.string.data = payload->data;
    value.as.string.len = len;
    value.as.string.payload = payload;
    return value;
}

static PTN_UNUSED PtnValue ptn_owned_string(char *string) {
    return ptn_owned_string_len(string, strlen(string));
}

static PTN_UNUSED PtnValue ptn_array(PtnArray *array) {
    PtnValue value;
    value.type = PTN_ARRAY;
    value.owned = 1;
    value.as.array = array;
    return value;
}

static PTN_UNUSED PtnValue ptn_object(PtnObject *object) {
    PtnValue value;
    value.type = PTN_OBJECT;
    value.owned = 1;
    value.as.object = object;
    return value;
}

static PTN_UNUSED PtnValue ptn_closure(PtnRuntime *runtime, size_t function_index, const char *display_name) {
    PtnClosure *closure = malloc(sizeof(PtnClosure));
    if (closure == NULL) {
        ptn_abort_out_of_memory();
    }
    closure->refcount = 1;
    closure->object_id = ptn_runtime_alloc_object_id(runtime);
    closure->function_index = function_index;
    closure->display_name = display_name;
    closure->captures.items = NULL;
    closure->captures.len = 0;
    closure->captures.capacity = 0;
    closure->captures.index_slots = NULL;
    closure->captures.index_capacity = 0;
    PtnValue value;
    value.type = PTN_CLOSURE;
    value.owned = 1;
    value.as.closure = closure;
    return value;
}

static PTN_UNUSED PtnValue ptn_exception_value(PtnException *exception) {
    PtnValue value;
    value.type = PTN_EXCEPTION;
    value.owned = 1;
    value.as.exception = exception;
    return value;
}

static PTN_UNUSED PtnValue ptn_exception_borrow(PtnException *exception) {
    PtnValue value = ptn_exception_value(exception);
    value.owned = 0;
    return value;
}

static int64_t ptn_next_resource_id = 5;

static PTN_UNUSED PtnResource *ptn_resource_new_stream(FILE *stream, const char *uri, const char *mode) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
        if (stream != NULL) {
            fclose(stream);
        }
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = "stream";
    resource->stream = stream;
    resource->stream_uri = uri == NULL ? NULL : ptn_duplicate_string(uri);
    resource->stream_mode = mode == NULL ? NULL : ptn_duplicate_string(mode);
    resource->persistent = 0;
    return resource;
}

static PTN_UNUSED void ptn_resource_retain(PtnResource *resource) {
    if (resource == NULL) {
        return;
    }
    if (resource->persistent) {
        return;
    }
    if (resource->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount++;
}

static PTN_UNUSED void ptn_resource_close(PtnResource *resource) {
    if (resource == NULL || resource->stream == NULL) {
        return;
    }
    if (resource->persistent) {
        return;
    }
    fclose(resource->stream);
    resource->stream = NULL;
}

static PTN_UNUSED void ptn_resource_release(PtnResource *resource) {
    if (resource == NULL) {
        return;
    }
    if (resource->persistent) {
        return;
    }
    if (resource->refcount == 0) {
        return;
    }
    resource->refcount--;
    if (resource->refcount != 0) {
        return;
    }
    ptn_resource_close(resource);
    free(resource->stream_uri);
    free(resource->stream_mode);
    free(resource);
}

static PTN_UNUSED PtnValue ptn_resource(PtnResource *resource) {
    PtnValue value;
    value.type = PTN_RESOURCE;
    value.owned = 1;
    value.as.resource = resource;
    return value;
}

static PTN_UNUSED PtnValue ptn_standard_stream_resource_value(int64_t id) {
    static PtnResource stdin_resource = { SIZE_MAX, 1, "stream", NULL, NULL, NULL, 1 };
    static PtnResource stdout_resource = { SIZE_MAX, 2, "stream", NULL, NULL, NULL, 1 };
    static PtnResource stderr_resource = { SIZE_MAX, 3, "stream", NULL, NULL, NULL, 1 };
    PtnResource *resource = &stdin_resource;
    if (id == 2) {
        resource = &stdout_resource;
    } else if (id == 3) {
        resource = &stderr_resource;
    }
    resource->stream = id == 1 ? stdin : (id == 2 ? stdout : stderr);
    resource->stream_uri = id == 1 ? "php://stdin" : (id == 2 ? "php://stdout" : "php://stderr");
    resource->stream_mode = id == 1 ? "r" : "w";
    PtnValue value = ptn_resource(resource);
    value.owned = 0;
    return value;
}

static PTN_UNUSED PtnValue ptn_reference_value(PtnReference *reference) {
    PtnValue value;
    value.type = PTN_REFERENCE;
    value.owned = 1;
    value.as.reference = reference;
    return value;
}

static PTN_UNUSED PtnLookupResult ptn_lookup_missing(void) {
    PtnLookupResult result;
    result.exists = 0;
    result.value = ptn_null();
    return result;
}

static PTN_UNUSED PtnLookupResult ptn_lookup_found(PtnValue value) {
    PtnLookupResult result;
    result.exists = 1;
    result.value = value;
    return result;
}

static void ptn_abort_out_of_memory(void) {
    fputs("Fatal error: out of memory\n", stderr);
    exit(1);
}

static PTN_UNUSED void ptn_cow_debug_abort(const char *message) {
    fputs("Fatal error: COW debug assertion failed: ", stderr);
    fputs(message, stderr);
    fputc('\n', stderr);
    exit(255);
}

static PTN_UNUSED void ptn_cow_debug_abort_counter(const char *name, size_t actual, int64_t expected) {
    char message[192];
    int written = snprintf(
        message,
        sizeof(message),
        "%s expected %lld, got %zu",
        name,
        (long long)expected,
        actual
    );
    if (written < 0 || (size_t)written >= sizeof(message)) {
        ptn_abort_out_of_memory();
    }
    ptn_cow_debug_abort(message);
}

static PTN_UNUSED void ptn_cow_debug_increment(size_t *counter) {
    if (*counter == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    (*counter)++;
}

static PTN_UNUSED void ptn_cow_debug_note_string_alloc(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_allocs);
}

static PTN_UNUSED void ptn_cow_debug_note_string_free(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_frees);
}

static PTN_UNUSED void ptn_cow_debug_note_string_clone(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_clones);
}

static PTN_UNUSED void ptn_cow_debug_note_string_retain(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_retain);
}

static PTN_UNUSED void ptn_cow_debug_note_string_release(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_release);
}

static PTN_UNUSED void ptn_cow_debug_note_string_detach(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.string_detaches);
}

static PTN_UNUSED void ptn_cow_debug_note_array_alloc(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_allocs);
}

static PTN_UNUSED void ptn_cow_debug_note_array_retain(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_retain);
}

static PTN_UNUSED void ptn_cow_debug_note_array_release(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_release);
}

static PTN_UNUSED void ptn_cow_debug_note_array_free(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_frees);
}

static PTN_UNUSED void ptn_cow_debug_note_array_clone(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_clones);
}

static PTN_UNUSED void ptn_cow_debug_note_array_detach(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_detaches);
}

static PTN_UNUSED void ptn_cow_debug_note_array_detach_skip(void) {
    ptn_cow_debug_increment(&ptn_cow_debug_counters.array_detach_skips);
}

static PTN_UNUSED void ptn_cow_debug_assert_array_refcount(PtnArray *array, const char *operation) {
    if (array == NULL) {
        return;
    }
    if (array->refcount == 0) {
        char message[128];
        int written = snprintf(message, sizeof(message), "array refcount underflow during %s", operation);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_cow_debug_abort(message);
    }
}

static PTN_UNUSED void ptn_cow_debug_assert_string_refcount(size_t *refcount, const char *operation) {
    if (refcount == NULL) {
        return;
    }
    if (*refcount == 0) {
        char message[128];
        int written = snprintf(message, sizeof(message), "string refcount underflow during %s", operation);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_cow_debug_abort(message);
    }
}

static PTN_UNUSED size_t ptn_cow_debug_live_count(size_t allocs, size_t frees, const char *name) {
    if (frees > allocs) {
        char message[128];
        int written = snprintf(message, sizeof(message), "%s frees exceed allocs", name);
        if (written < 0 || (size_t)written >= sizeof(message)) {
            ptn_abort_out_of_memory();
        }
        ptn_cow_debug_abort(message);
    }
    return allocs - frees;
}

static PTN_UNUSED void ptn_cow_debug_reset(void) {
    memset(&ptn_cow_debug_counters, 0, sizeof(ptn_cow_debug_counters));
}

static PTN_UNUSED int ptn_cow_debug_counter(const char *name, size_t *out) {
    if (strcmp(name, "string.alloc") == 0) {
        *out = ptn_cow_debug_counters.string_allocs;
        return 1;
    }
    if (strcmp(name, "string.free") == 0) {
        *out = ptn_cow_debug_counters.string_frees;
        return 1;
    }
    if (strcmp(name, "string.clone") == 0) {
        *out = ptn_cow_debug_counters.string_clones;
        return 1;
    }
    if (strcmp(name, "string.retain") == 0) {
        *out = ptn_cow_debug_counters.string_retain;
        return 1;
    }
    if (strcmp(name, "string.release") == 0) {
        *out = ptn_cow_debug_counters.string_release;
        return 1;
    }
    if (strcmp(name, "string.detach") == 0) {
        *out = ptn_cow_debug_counters.string_detaches;
        return 1;
    }
    if (strcmp(name, "string.live") == 0) {
        *out = ptn_cow_debug_live_count(
            ptn_cow_debug_counters.string_allocs,
            ptn_cow_debug_counters.string_frees,
            name
        );
        return 1;
    }
    if (strcmp(name, "array.alloc") == 0) {
        *out = ptn_cow_debug_counters.array_allocs;
        return 1;
    }
    if (strcmp(name, "array.retain") == 0) {
        *out = ptn_cow_debug_counters.array_retain;
        return 1;
    }
    if (strcmp(name, "array.release") == 0) {
        *out = ptn_cow_debug_counters.array_release;
        return 1;
    }
    if (strcmp(name, "array.free") == 0) {
        *out = ptn_cow_debug_counters.array_frees;
        return 1;
    }
    if (strcmp(name, "array.clone") == 0) {
        *out = ptn_cow_debug_counters.array_clones;
        return 1;
    }
    if (strcmp(name, "array.detach") == 0) {
        *out = ptn_cow_debug_counters.array_detaches;
        return 1;
    }
    if (strcmp(name, "array.detach_skip") == 0) {
        *out = ptn_cow_debug_counters.array_detach_skips;
        return 1;
    }
    if (strcmp(name, "array.live") == 0) {
        *out = ptn_cow_debug_live_count(
            ptn_cow_debug_counters.array_allocs,
            ptn_cow_debug_counters.array_frees,
            name
        );
        return 1;
    }
    return 0;
}

static PTN_UNUSED void ptn_cow_debug_assert_named_counter(const char *name, int64_t expected) {
    if (expected < 0) {
        ptn_cow_debug_abort_counter(name, 0, expected);
    }
    size_t actual = 0;
    if (!ptn_cow_debug_counter(name, &actual)) {
        ptn_cow_debug_abort("unknown counter");
    }
    if (actual != (size_t)expected) {
        ptn_cow_debug_abort_counter(name, actual, expected);
    }
}

static PTN_UNUSED void ptn_cow_debug_assert_balanced(void) {
    ptn_cow_debug_assert_named_counter("string.live", 0);
    ptn_cow_debug_assert_named_counter("array.live", 0);
}

static PTN_UNUSED char *ptn_duplicate_string(const char *string) {
    size_t len = strlen(string);
