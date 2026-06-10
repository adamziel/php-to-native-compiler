#include <ctype.h>
#include <errno.h>
#include <math.h>
#include <stdarg.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#if defined(_WIN32)
#include <direct.h>
#include <process.h>
#else
#include <unistd.h>
#endif

#if defined(__GNUC__) || defined(__clang__)
#define PTN_UNUSED __attribute__((unused))
#else
#define PTN_UNUSED
#endif

#if defined(__GNUC__) && !defined(__clang__)
#pragma GCC diagnostic ignored "-Wclobbered"
#endif

#define PTN_PHP_VERSION "8.4.0"
#define PTN_PHP_SAPI_NAME "cli"
#define PTN_ARRAY_INDEX_MIN_ENTRIES 16
#define PTN_SYMBOL_INDEX_MIN_ENTRIES 16

typedef struct PtnArray PtnArray;
typedef struct PtnClosure PtnClosure;
typedef struct PtnException PtnException;
typedef struct PtnObject PtnObject;
typedef struct PtnReference PtnReference;
typedef struct PtnTryFrame PtnTryFrame;

typedef enum {
    PTN_NULL,
    PTN_BOOL,
    PTN_INT,
    PTN_FLOAT,
    PTN_STRING,
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
    size_t function_index;
    const char *display_name;
    PtnSymbolTable captures;
};

struct PtnReference {
    size_t refcount;
    PtnValue value;
};

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
    char *class_name;
    PtnArray *properties;
};

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
    const char *class_name;
    char *message;
    const char *path;
    size_t line;
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
} PtnDiagnosticSink;

typedef struct {
    PtnSymbolTable symbols;
    PtnSymbolTable *global_symbols;
    PtnSymbolTable owned_constants;
    PtnSymbolTable *constants;
    PtnSymbolTable owned_static_properties;
    PtnSymbolTable *static_properties;
    PtnDiagnosticSink diagnostics;
    PtnExceptionState owned_exceptions;
    PtnExceptionState *exceptions;
    PtnCallFrame owned_call_frame;
    PtnCallFrame *call_frame;
    const char *source_path;
    const char *current_function_name;
    size_t call_site_line;
} PtnRuntime;

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

static PTN_UNUSED PtnValue ptn_closure(size_t function_index, const char *display_name) {
    PtnClosure *closure = malloc(sizeof(PtnClosure));
    if (closure == NULL) {
        ptn_abort_out_of_memory();
    }
    closure->refcount = 1;
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
