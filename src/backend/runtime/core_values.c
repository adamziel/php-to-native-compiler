#include <ctype.h>
#include <errno.h>
#include <math.h>
#include <stdarg.h>
#include <setjmp.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
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
typedef struct PtnException PtnException;
typedef struct PtnTryFrame PtnTryFrame;

typedef enum {
    PTN_NULL,
    PTN_BOOL,
    PTN_INT,
    PTN_FLOAT,
    PTN_STRING,
    PTN_ARRAY,
    PTN_EXCEPTION
} PtnType;

typedef enum {
    PTN_ARRAY_KEY_INT,
    PTN_ARRAY_KEY_STRING
} PtnArrayKeyType;

typedef struct {
    const unsigned char *data;
    size_t len;
    char *owned;
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
        PtnException *exception;
    } as;
} PtnValue;

typedef struct {
    int exists;
    PtnValue value;
} PtnLookupResult;

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
    int valid;
} PtnArrayIterator;

struct PtnArray {
    size_t refcount;
    size_t len;
    size_t capacity;
    PtnArrayEntry *entries;
    PtnArrayIndexSlot *index_slots;
    size_t index_capacity;
    int64_t next_auto_key;
    size_t current_index;
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

typedef struct {
    FILE *stream;
    int emitted_deprecation;
    int emitted_warning;
} PtnDiagnosticSink;

typedef struct {
    PtnSymbolTable symbols;
    PtnSymbolTable owned_constants;
    PtnSymbolTable *constants;
    PtnDiagnosticSink diagnostics;
    PtnExceptionState owned_exceptions;
    PtnExceptionState *exceptions;
    PtnCallFrame owned_call_frame;
    PtnCallFrame *call_frame;
} PtnRuntime;

static PTN_UNUSED int ptn_is_truthy(PtnValue value);

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

static PTN_UNUSED PtnValue ptn_string_literal(const char *string, size_t len) {
    PtnValue value;
    value.type = PTN_STRING;
    value.owned = 0;
    value.as.string.data = (const unsigned char *)string;
    value.as.string.len = len;
    value.as.string.owned = NULL;
    return value;
}

static PTN_UNUSED PtnValue ptn_string(const char *string) {
    return ptn_string_literal(string, strlen(string));
}

static PTN_UNUSED PtnValue ptn_owned_string_len(char *string, size_t len) {
    PtnValue value;
    value.type = PTN_STRING;
    value.owned = 1;
    value.as.string.data = (const unsigned char *)string;
    value.as.string.len = len;
    value.as.string.owned = string;
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

static PTN_UNUSED char *ptn_duplicate_string(const char *string) {
    size_t len = strlen(string);
