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
#include <io.h>
#include <process.h>
#include <sys/utime.h>
#else
#include <dirent.h>
#include <fnmatch.h>
#include <glob.h>
#include <langinfo.h>
#include <pwd.h>
#include <regex.h>
#include <sys/file.h>
#include <sys/time.h>
#include <sys/statvfs.h>
#include <sys/utsname.h>
#include <sys/wait.h>
#include <utime.h>
#include <unistd.h>
#endif

#if !defined(_WIN32)
extern char *realpath(const char *path, char *resolved_path);
extern char **environ;
#define PTN_ENVIRON environ
#else
extern char **_environ;
#define PTN_ENVIRON _environ
#endif

#if defined(__GNUC__) || defined(__clang__)
#define PTN_UNUSED __attribute__((unused))
#else
#define PTN_UNUSED
#endif

#ifndef R_OK
#define R_OK 4
#endif
#ifndef W_OK
#define W_OK 2
#endif
#ifndef X_OK
#define X_OK 1
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
#ifdef PATH_MAX
#define PTN_PHP_MAXPATHLEN PATH_MAX
#else
#define PTN_PHP_MAXPATHLEN 4096
#endif
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
#define PTN_ARRAY_MAX_ALLOC_ENTRIES 1048576ULL
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
#define PTN_EXTR_OVERWRITE 0
#define PTN_EXTR_SKIP 1
#define PTN_EXTR_PREFIX_SAME 2
#define PTN_EXTR_PREFIX_ALL 3
#define PTN_EXTR_PREFIX_INVALID 4
#define PTN_EXTR_PREFIX_IF_EXISTS 5
#define PTN_EXTR_IF_EXISTS 6
#define PTN_EXTR_REFS 256
#define PTN_ARRAY_FILTER_USE_BOTH 1
#define PTN_ARRAY_FILTER_USE_KEY 2
#define PTN_SORT_REGULAR 0
#define PTN_SORT_NUMERIC 1
#define PTN_SORT_STRING 2
#define PTN_SORT_DESC 3
#define PTN_SORT_ASC 4
#define PTN_SORT_LOCALE_STRING 5
#define PTN_SORT_NATURAL 6
#define PTN_SORT_FLAG_CASE 8
#define PTN_HTML_SPECIALCHARS 0
#define PTN_HTML_ENTITIES 1
#define PTN_ENT_NOQUOTES 0
#define PTN_ENT_COMPAT 2
#define PTN_ENT_QUOTES 3
#define PTN_ENT_IGNORE 4
#define PTN_ENT_SUBSTITUTE 8
#define PTN_ENT_HTML401 0
#define PTN_ENT_XML1 16
#define PTN_ENT_XHTML 32
#define PTN_ENT_HTML5 48
#define PTN_ENT_DISALLOWED 128
#define PTN_CRYPT_SALT_LENGTH 123
#define PTN_CRYPT_STD_DES 1
#define PTN_CRYPT_EXT_DES 1
#define PTN_CRYPT_MD5 1
#define PTN_CRYPT_BLOWFISH 1
#define PTN_CRYPT_SHA256 1
#define PTN_CRYPT_SHA512 1
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
#define PTN_PHP_URL_SCHEME 0
#define PTN_PHP_URL_HOST 1
#define PTN_PHP_URL_PORT 2
#define PTN_PHP_URL_USER 3
#define PTN_PHP_URL_PASS 4
#define PTN_PHP_URL_PATH 5
#define PTN_PHP_URL_QUERY 6
#define PTN_PHP_URL_FRAGMENT 7
#define PTN_PHP_QUERY_RFC1738 1
#define PTN_PHP_QUERY_RFC3986 2
#define PTN_FILE_USE_INCLUDE_PATH 1
#define PTN_FILE_IGNORE_NEW_LINES 2
#define PTN_FILE_SKIP_EMPTY_LINES 4
#define PTN_LOCK_SH 1
#define PTN_LOCK_EX 2
#define PTN_LOCK_UN 3
#define PTN_LOCK_NB 4
#define PTN_FNM_NOESCAPE 1
#define PTN_FNM_PATHNAME 2
#define PTN_FNM_PERIOD 4
#define PTN_FNM_CASEFOLD 16
#define PTN_GLOB_MARK 2
#define PTN_GLOB_NOSORT 4
#define PTN_GLOB_NOCHECK 16
#define PTN_GLOB_NOESCAPE 64
#define PTN_GLOB_BRACE 1024
#define PTN_GLOB_ONLYDIR 8192
#define PTN_GLOB_ERR 1
#define PTN_SEEK_SET SEEK_SET
#define PTN_SEEK_CUR SEEK_CUR
#define PTN_SEEK_END SEEK_END
#define PTN_STREAM_FILTER_READ 1
#define PTN_STREAM_FILTER_WRITE 2
#define PTN_STREAM_FILTER_ALL 3
#define PTN_DEBUG_BACKTRACE_PROVIDE_OBJECT 1
#define PTN_DEBUG_BACKTRACE_IGNORE_ARGS 2
#define PTN_LC_CTYPE 0
#define PTN_LC_NUMERIC 1
#define PTN_LC_TIME 2
#define PTN_LC_COLLATE 3
#define PTN_LC_MONETARY 4
#define PTN_LC_MESSAGES 5
#define PTN_LC_ALL 6
#define PTN_ABDAY_1 131072
#define PTN_ABDAY_2 131073
#define PTN_ABDAY_3 131074
#define PTN_ABDAY_4 131075
#define PTN_ABDAY_5 131076
#define PTN_ABDAY_6 131077
#define PTN_ABDAY_7 131078
#define PTN_DAY_1 131079
#define PTN_DAY_2 131080
#define PTN_DAY_3 131081
#define PTN_DAY_4 131082
#define PTN_DAY_5 131083
#define PTN_DAY_6 131084
#define PTN_DAY_7 131085
#define PTN_ABMON_1 131086
#define PTN_ABMON_2 131087
#define PTN_ABMON_3 131088
#define PTN_ABMON_4 131089
#define PTN_ABMON_5 131090
#define PTN_ABMON_6 131091
#define PTN_ABMON_7 131092
#define PTN_ABMON_8 131093
#define PTN_ABMON_9 131094
#define PTN_ABMON_10 131095
#define PTN_ABMON_11 131096
#define PTN_ABMON_12 131097
#define PTN_MON_1 131098
#define PTN_MON_2 131099
#define PTN_MON_3 131100
#define PTN_MON_4 131101
#define PTN_MON_5 131102
#define PTN_MON_6 131103
#define PTN_MON_7 131104
#define PTN_MON_8 131105
#define PTN_MON_9 131106
#define PTN_MON_10 131107
#define PTN_MON_11 131108
#define PTN_MON_12 131109
#define PTN_RADIXCHAR 65536
#define PTN_THOUSEP 65537
#define PTN_YESEXPR 327680
#define PTN_NOEXPR 327681
#define PTN_CODESET 14
#define PTN_DEFAULT_PRECISION 14
#define PTN_JSON_ERROR_NONE 0
#define PTN_JSON_ERROR_DEPTH 1
#define PTN_JSON_ERROR_STATE_MISMATCH 2
#define PTN_JSON_ERROR_CTRL_CHAR 3
#define PTN_JSON_ERROR_SYNTAX 4
#define PTN_JSON_ERROR_UTF8 5
#define PTN_JSON_ERROR_RECURSION 6
#define PTN_JSON_ERROR_INF_OR_NAN 7
#define PTN_JSON_ERROR_UNSUPPORTED_TYPE 8
#define PTN_JSON_ERROR_INVALID_PROPERTY_NAME 9
#define PTN_JSON_ERROR_UTF16 10
#define PTN_JSON_ERROR_NON_BACKED_ENUM 11
#define PTN_JSON_OBJECT_AS_ARRAY 1
#define PTN_JSON_BIGINT_AS_STRING 2
#define PTN_JSON_HEX_TAG 1
#define PTN_JSON_HEX_AMP 2
#define PTN_JSON_HEX_APOS 4
#define PTN_JSON_HEX_QUOT 8
#define PTN_JSON_FORCE_OBJECT 16
#define PTN_JSON_NUMERIC_CHECK 32
#define PTN_JSON_UNESCAPED_SLASHES 64
#define PTN_JSON_PRETTY_PRINT 128
#define PTN_JSON_UNESCAPED_UNICODE 256
#define PTN_JSON_PARTIAL_OUTPUT_ON_ERROR 512
#define PTN_JSON_PRESERVE_ZERO_FRACTION 1024
#define PTN_JSON_UNESCAPED_LINE_TERMINATORS 2048
#define PTN_JSON_INVALID_UTF8_IGNORE 1048576
#define PTN_JSON_INVALID_UTF8_SUBSTITUTE 2097152
#define PTN_JSON_THROW_ON_ERROR 4194304
#define PTN_PREG_PATTERN_ORDER 1
#define PTN_PREG_SET_ORDER 2
#define PTN_PREG_OFFSET_CAPTURE 256
#define PTN_PREG_UNMATCHED_AS_NULL 512
#define PTN_PREG_SPLIT_NO_EMPTY 1
#define PTN_PREG_SPLIT_DELIM_CAPTURE 2
#define PTN_PREG_SPLIT_OFFSET_CAPTURE 4
#define PTN_PREG_GREP_INVERT 1
#define PTN_PREG_NO_ERROR 0
#define PTN_PREG_INTERNAL_ERROR 1
#define PTN_PREG_BACKTRACK_LIMIT_ERROR 2
#define PTN_PREG_RECURSION_LIMIT_ERROR 3
#define PTN_PREG_BAD_UTF8_ERROR 4
#define PTN_PREG_BAD_UTF8_OFFSET_ERROR 5
#define PTN_PREG_JIT_STACKLIMIT_ERROR 6

typedef struct PtnArray PtnArray;
typedef struct PtnClosure PtnClosure;
typedef struct PtnException PtnException;
typedef struct PtnGenerator PtnGenerator;
typedef struct PtnObject PtnObject;
typedef struct PtnReference PtnReference;
typedef struct PtnRuntime PtnRuntime;
typedef struct PtnResource PtnResource;
typedef struct PtnStreamFilter PtnStreamFilter;
typedef struct PtnTraceFrame PtnTraceFrame;
typedef struct PtnTryFrame PtnTryFrame;

static int ptn_builtin_class_implements_interface(const char *class_name, const char *interface_name);

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

typedef enum {
    PTN_STREAM_FILTER_STRING_ROT13,
    PTN_STREAM_FILTER_STRING_TOUPPER,
    PTN_STREAM_FILTER_STRING_TOLOWER
} PtnStreamFilterKind;

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
    size_t string_len;
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

typedef struct {
    const char *name;
    const char *type_name;
    const char *type_display_name;
    int type_allows_null;
    int type_is_builtin;
    int by_ref;
    int is_variadic;
    int can_be_passed_by_value;
} PtnParameterMetadata;

typedef struct {
    int found;
    const char *name;
    int is_internal;
    size_t parameter_count;
    size_t required_parameter_count;
    int is_variadic;
    const PtnParameterMetadata *parameters;
    int return_by_ref;
    const char *return_type_name;
    const char *return_type_display_name;
    int return_type_allows_null;
    int return_type_is_builtin;
} PtnFunctionMetadata;

struct PtnClosure {
    size_t refcount;
    size_t object_id;
    PtnRuntime *lifecycle_runtime;
    size_t function_index;
    const char *display_name;
    PtnFunctionMetadata metadata;
    char *scope_class_name;
    char *called_class_name;
    int is_static;
    int uses_this;
    PtnSymbolTable captures;
    int has_wrapped_callable;
    PtnValue wrapped_callable;
    char *bound_scope_name;
};

typedef enum {
    PTN_PROPERTY_PUBLIC,
    PTN_PROPERTY_PROTECTED,
    PTN_PROPERTY_PRIVATE
} PtnPropertyVisibility;

typedef enum {
    PTN_PROPERTY_TYPE_NONE,
    PTN_PROPERTY_TYPE_NULL,
    PTN_PROPERTY_TYPE_ARRAY,
    PTN_PROPERTY_TYPE_INT,
    PTN_PROPERTY_TYPE_FLOAT,
    PTN_PROPERTY_TYPE_STRING,
    PTN_PROPERTY_TYPE_BOOL,
    PTN_PROPERTY_TYPE_MIXED,
    PTN_PROPERTY_TYPE_OBJECT,
    PTN_PROPERTY_TYPE_CLASS
} PtnPropertyTypeKind;

struct PtnReference {
    size_t refcount;
    PtnValue value;
    PtnPropertyTypeKind property_type_kind;
    char *property_type_class_name;
    char *property_type_text;
    int property_type_allows_null;
    char *property_declaring_class;
    char *property_name;
};

typedef struct {
    char *storage_name;
    char *display_name;
    char *declaring_class;
    PtnPropertyVisibility read_visibility;
    PtnPropertyVisibility set_visibility;
    int is_readonly;
    int is_unset;
    char *last_type_name;
    PtnPropertyTypeKind type_kind;
    char *type_class_name;
    char *type_text;
    int type_allows_null;
} PtnObjectPropertyMetadata;

typedef void (*PtnObjectNativeDataFree)(void *data);

typedef struct {
    int exists;
    PtnValue value;
} PtnLookupResult;

static PTN_UNUSED int ptn_property_reference_coerce_assignment(
    PtnRuntime *runtime,
    const PtnReference *reference,
    PtnValue value,
    int reference_context,
    PtnValue *out
);
static PTN_UNUSED PtnValue ptn_cast_string(PtnValue value);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED int ptn_value_satisfies_class_type_hint(PtnRuntime *runtime, PtnValue value, const char *expected_class_name);

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
    PtnObject *object;
    PtnGenerator *generator;
    PtnRuntime *runtime;
    const char *access_scope;
    PtnValue iterator_object;
    size_t index;
    size_t length;
    PtnArrayKey current_key;
    PtnReference *current_reference;
    PtnValue *watched_slot;
    size_t line;
    uint64_t seen_mutation_epoch;
    int has_current_key;
    int has_iterator_object;
    int protocol_iterator;
    int valid;
    int live;
} PtnArrayIterator;

struct PtnGenerator {
    PtnArray *values;
    PtnArray *keys;
    PtnValue return_value;
    PtnArray *reference_notice_lines;
    size_t position;
    int64_t next_auto_key;
    int completed;
    int yields_by_ref;
};

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
    int has_iterator_current_index;
    size_t iterator_current_index;
    size_t iterator_mutation_resume_index;
    uint64_t iterator_mutation_epoch;
    uint64_t mutation_epoch;
};

struct PtnObject {
    size_t refcount;
    size_t object_id;
    char *class_name;
    char *enum_case_name;
    PtnArray *properties;
    PtnObjectPropertyMetadata *property_metadata;
    size_t property_metadata_len;
    size_t property_metadata_capacity;
    void *native_data;
    PtnObjectNativeDataFree native_data_free;
    PtnRuntime *lifecycle_runtime;
    int destructor_enabled;
    int destructor_called;
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

typedef struct {
    PtnStringBuffer buffer;
    int has_callback;
    PtnValue callback;
} PtnOutputBuffer;

typedef struct {
    PtnValue callback;
    PtnValue *args;
    size_t argc;
} PtnShutdownFunction;

typedef struct {
    PtnReference *reference;
} PtnStaticLocalSlot;

typedef struct {
    size_t object_id;
    char *property;
} PtnMagicPropertyFrame;

typedef enum {
    PTN_STREAM_BACKEND_FILE,
    PTN_STREAM_BACKEND_MEMORY,
    PTN_STREAM_BACKEND_INPUT,
    PTN_STREAM_BACKEND_TEMP
} PtnStreamBackend;

typedef struct {
    unsigned char *data;
    size_t len;
    size_t capacity;
    size_t position;
    size_t max_memory;
    int writable;
    int append;
    int spilled;
    int eof;
    int error;
} PtnMemoryStream;

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
    size_t refcount;
    size_t object_id;
    PtnRuntime *lifecycle_runtime;
    const char *class_name;
    char *message;
    size_t message_len;
    int64_t code;
    const char *path;
    size_t line;
    PtnValue trace;
    PtnValue previous;
    int64_t severity;
};

struct PtnResource {
    size_t refcount;
    int64_t id;
    const char *type_name;
    FILE *stream;
    void *directory;
    char *stream_uri;
    char *stream_mode;
    PtnStreamBackend stream_backend;
    PtnMemoryStream *memory_stream;
    PtnStreamFilter *read_filters;
    PtnStreamFilter *write_filters;
    int persistent;
};

struct PtnStreamFilter {
    PtnStreamFilterKind kind;
    char *name;
    PtnStreamFilter *next;
};

typedef struct {
    int has_handler;
    PtnValue handler;
    int64_t levels;
} PtnErrorHandlerFrame;

typedef struct {
    int has_handler;
    PtnValue handler;
} PtnExceptionHandlerFrame;

typedef struct {
    PtnException *active_exception;
    PtnTryFrame *try_frame;
    int has_exception_handler;
    PtnValue exception_handler;
    PtnExceptionHandlerFrame *exception_handler_stack;
    size_t exception_handler_stack_len;
    size_t exception_handler_stack_capacity;
    int in_exception_handler;
} PtnExceptionState;

typedef struct {
    size_t argc;
    const PtnValue *args;
    size_t parameter_count;
    const char *const *parameter_names;
} PtnCallFrame;

struct PtnTraceFrame {
    PtnRuntime *runtime;
    const char *function_name;
    const char *file;
    size_t line;
    size_t argc;
    const PtnValue *args;
    size_t parameter_count;
    const char *const *parameter_names;
    int has_receiver;
    PtnValue receiver;
    PtnTraceFrame *previous;
};

struct PtnTryFrame {
    jmp_buf jump;
    PtnTryFrame *previous;
};

typedef struct {
    PtnRuntime *runtime;
    FILE *stream;
    int emitted_deprecation;
    int emitted_warning;
    int suppressed;
    int64_t error_reporting;
    int display_errors;
    int has_error_handler;
    PtnValue error_handler;
    int64_t error_handler_levels;
    PtnErrorHandlerFrame *error_handler_stack;
    size_t error_handler_stack_len;
    size_t error_handler_stack_capacity;
} PtnDiagnosticSink;

typedef PtnValue (*PtnMethodDispatchHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *method_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
typedef int (*PtnReflectedMethodDispatchHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *target_class_name,
    const char *method_name,
    const char *called_class_name,
    size_t argc,
    const PtnValue *args,
    size_t line,
    PtnValue *result_out
);
typedef int (*PtnDeclaredMethodExistsHandler)(const char *class_name, const char *method_name);
typedef int (*PtnClassScopeAllowsHandler)(
    const char *access_scope,
    const char *declaring_class
);
typedef int (*PtnDeclaredClassReadonlyHandler)(const char *class_name);
typedef int (*PtnMagicPropertyReadHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    int require_isset,
    PtnValue *value_out
);
typedef int (*PtnMagicPropertyIssetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    int *isset_out
);
typedef int (*PtnMagicPropertyGetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line,
    PtnValue *value_out
);
typedef int (*PtnMagicPropertyGetExistsHandler)(PtnRuntime *runtime, PtnValue receiver);
typedef int (*PtnMagicPropertySetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    PtnValue value,
    size_t line
);
typedef int (*PtnMagicPropertyUnsetHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *property,
    size_t line
);
typedef int (*PtnMagicDebugInfoHandler)(
    PtnRuntime *runtime,
    PtnValue receiver,
    size_t line,
    PtnValue *value_out
);
typedef int (*PtnClassConstantInitializerHandler)(
    PtnRuntime *runtime,
    const char *class_name,
    const char *constant_name
);
typedef PtnValue (*PtnNewInstanceWithoutConstructorHandler)(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
);

struct PtnRuntime {
    PtnSymbolTable symbols;
    PtnSymbolTable *global_symbols;
    PtnSymbolTable owned_constants;
    PtnSymbolTable *constants;
    PtnSymbolTable owned_class_aliases;
    PtnSymbolTable *class_aliases;
    PtnSymbolTable owned_class_constants;
    PtnSymbolTable *class_constants;
    PtnSymbolTable owned_class_constant_deprecations;
    PtnSymbolTable *class_constant_deprecations;
    const char *class_constant_deprecation_suppress_class;
    const char *class_constant_deprecation_suppress_constant;
    PtnSymbolTable owned_static_properties;
    PtnSymbolTable *static_properties;
    PtnSymbolTable owned_static_property_read_visibility;
    PtnSymbolTable *static_property_read_visibility;
    PtnSymbolTable owned_static_property_set_visibility;
    PtnSymbolTable *static_property_set_visibility;
    PtnDiagnosticSink diagnostics;
    PtnExceptionState owned_exceptions;
    PtnExceptionState *exceptions;
    PtnCallFrame owned_call_frame;
    PtnCallFrame *call_frame;
    PtnTraceFrame owned_trace_frame;
    PtnTraceFrame *trace_frame;
    PtnRuntime *lifecycle_root;
    PtnObject **live_objects;
    size_t live_objects_len;
    size_t live_objects_capacity;
    PtnStaticLocalSlot *static_local_slots;
    size_t static_local_slots_len;
    size_t static_local_slots_capacity;
    size_t next_object_id;
    size_t *free_object_ids;
    size_t free_object_ids_len;
    size_t free_object_ids_capacity;
    PtnOutputBuffer *output_buffers;
    size_t output_buffers_len;
    size_t output_buffers_capacity;
    size_t output_buffer_callback_depth;
    PtnShutdownFunction *shutdown_functions;
    size_t shutdown_functions_len;
    size_t shutdown_functions_capacity;
    size_t shutdown_function_index;
    int shutdown_functions_running;
    int shutdown_functions_completed;
    PtnMethodDispatchHandler method_dispatch;
    PtnReflectedMethodDispatchHandler reflected_method_dispatch;
    PtnDeclaredMethodExistsHandler declared_method_exists;
    PtnClassScopeAllowsHandler class_scope_allows;
    PtnDeclaredClassReadonlyHandler declared_class_is_readonly;
    PtnMagicPropertyReadHandler magic_property_read;
    PtnMagicPropertyIssetHandler magic_property_isset;
    int *declared_user_functions;
    PtnMagicPropertyGetHandler magic_property_get;
    PtnMagicPropertyGetExistsHandler magic_property_get_exists;
    PtnMagicPropertySetHandler magic_property_set;
    PtnMagicPropertyUnsetHandler magic_property_unset;
    PtnMagicDebugInfoHandler magic_debug_info;
    PtnClassConstantInitializerHandler class_constant_initializer;
    PtnNewInstanceWithoutConstructorHandler new_instance_without_constructor;
    int in_magic_property_dispatch;
    PtnMagicPropertyFrame *magic_property_frames;
    size_t magic_property_frame_len;
    size_t magic_property_frame_capacity;
    const char *source_path;
    const char *current_function_name;
    const char *current_class_name;
    const char *current_called_class_name;
    const char *called_class_name_override;
    PtnGenerator *current_generator;
    int has_current_receiver;
    PtnValue current_receiver;
    const char *by_ref_argument_function_name_override;
    int by_ref_argument_notice_pending;
    int by_ref_argument_notice_emitted;
    size_t by_ref_argument_notice_line;
    char *include_path;
    char **included_files;
    size_t included_files_len;
    size_t included_files_capacity;
    PtnValue *autoload_callbacks;
    size_t autoload_callbacks_len;
    size_t autoload_callbacks_capacity;
    char **autoloading_class_names;
    size_t autoloading_class_names_len;
    size_t autoloading_class_names_capacity;
    char *open_basedir;
    char *memory_limit;
    char *max_memory_limit;
    char *default_charset;
    char *arg_separator_input;
    char *arg_separator_output;
    char *output_handler;
    char *filter_default;
    char *pcre_backtrack_limit;
    char *pcre_jit;
    char *opcache_save_comments;
    char *internal_encoding;
    char *input_encoding;
    char *output_encoding;
    char *variables_order;
    char *register_argc_argv;
    char *enable_post_data_reading;
    char *file_uploads;
    char *max_input_vars;
    char *max_input_nesting_level;
    char *post_max_size;
    char *always_populate_raw_post_data;
    char *upload_tmp_dir;
    char *expose_php;
    char *user_agent;
    char *request_body;
    size_t request_body_len;
    int exception_ignore_args;
    size_t exception_string_param_max_len;
    int strict_types;
    int initial_zend_assertions;
    int zend_assertions;
    int assert_exception;
    size_t call_site_line;
    int suppress_user_call_frame_location;
    int warn_by_ref_argument_mismatch;
    int throw_argument_count_errors;
    void *active_serialize_state;
    void *active_unserialize_state;
    char *strtok_string;
    size_t strtok_len;
    size_t strtok_offset;
    int strtok_has_state;
    int json_last_error;
    size_t json_last_error_line;
    size_t json_last_error_column;
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
static PTN_UNUSED int ptn_ascii_case_equal(const char *left, const char *right);
static PTN_UNUSED int ptn_object_is_generator(PtnObject *object);
static PTN_UNUSED PtnValue ptn_generator_current(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED PtnValue ptn_generator_get_return(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED PtnValue ptn_generator_key(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED PtnValue ptn_generator_next(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED PtnValue ptn_generator_rewind(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED void ptn_generator_set_return_value(PtnGenerator *generator, PtnValue value);
static PTN_UNUSED PtnValue ptn_generator_valid(PtnRuntime *runtime, PtnValue receiver, size_t line);
static PTN_UNUSED char *ptn_duplicate_string(const char *string);
static PTN_UNUSED char *ptn_value_to_string(PtnValue value);
static PTN_UNUSED void ptn_output_write(PtnRuntime *runtime, const char *data, size_t len);
static PTN_UNUSED int ptn_declared_class_exists(const char *name);
static PTN_UNUSED const char *ptn_declared_class_canonical_name(const char *name);
static PTN_UNUSED const char *ptn_builtin_exception_class_name(const char *class_name);
static PTN_UNUSED void ptn_emit_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line);
static PTN_UNUSED void ptn_emit_user_warning(PtnDiagnosticSink *diagnostics, const char *message, size_t line);
static PTN_UNUSED void ptn_throw_exception(PtnRuntime *runtime, const char *class_name, const char *message);
static PTN_UNUSED void ptn_rethrow_exception(PtnRuntime *runtime);
static PTN_UNUSED void ptn_try_frame_push(PtnRuntime *runtime, PtnTryFrame *frame);
static PTN_UNUSED void ptn_try_frame_pop(PtnRuntime *runtime, PtnTryFrame *frame);
#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED PtnValue ptn_call_callable(PtnRuntime *runtime, PtnValue callable, size_t argc, const PtnValue *args, size_t line);
#endif
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
static PTN_UNUSED void ptn_output_buffer_flush_all(PtnRuntime *runtime);
static PTN_UNUSED const char *ptn_runtime_resolve_class_alias(
    PtnRuntime *runtime,
    const char *class_name
);
static PTN_UNUSED void ptn_runtime_autoload_class(
    PtnRuntime *runtime,
    const char *class_name,
    size_t line
);
static PtnSymbolTable *ptn_runtime_class_alias_table(PtnRuntime *runtime);

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
    metadata.parameters = NULL;
    metadata.return_by_ref = 0;
    metadata.return_type_name = NULL;
    metadata.return_type_display_name = NULL;
    metadata.return_type_allows_null = 0;
    metadata.return_type_is_builtin = 0;
    return metadata;
}

static PTN_UNUSED PtnFunctionMetadata ptn_function_metadata_found(
    const char *name,
    int is_internal,
    size_t parameter_count,
    size_t required_parameter_count,
    int is_variadic,
    const PtnParameterMetadata *parameters,
    int return_by_ref,
    const char *return_type_name,
    const char *return_type_display_name,
    int return_type_allows_null,
    int return_type_is_builtin
) {
    PtnFunctionMetadata metadata;
    metadata.found = 1;
    metadata.name = name;
    metadata.is_internal = is_internal;
    metadata.parameter_count = parameter_count;
    metadata.required_parameter_count = required_parameter_count;
    metadata.is_variadic = is_variadic;
    metadata.parameters = parameters;
    metadata.return_by_ref = return_by_ref;
    metadata.return_type_name = return_type_name;
    metadata.return_type_display_name = return_type_display_name;
    metadata.return_type_allows_null = return_type_allows_null;
    metadata.return_type_is_builtin = return_type_is_builtin;
    return metadata;
}

static PTN_UNUSED PtnRuntime *ptn_runtime_root(PtnRuntime *runtime) {
    if (runtime == NULL) {
        return NULL;
    }
    return runtime->lifecycle_root == NULL ? runtime : runtime->lifecycle_root;
}

static PTN_UNUSED void ptn_runtime_note_included_file(PtnRuntime *runtime, const char *path) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || path == NULL) {
        return;
    }
    for (size_t i = 0; i < root->included_files_len; i++) {
        if (strcmp(root->included_files[i], path) == 0) {
            return;
        }
    }
    if (root->included_files_len == root->included_files_capacity) {
        size_t new_capacity = root->included_files_capacity == 0
            ? 8
            : root->included_files_capacity * 2;
        if (new_capacity < root->included_files_capacity ||
            new_capacity > SIZE_MAX / sizeof(char *)) {
            ptn_abort_out_of_memory();
        }
        char **new_files = realloc(root->included_files, new_capacity * sizeof(char *));
        if (new_files == NULL) {
            ptn_abort_out_of_memory();
        }
        root->included_files = new_files;
        root->included_files_capacity = new_capacity;
    }
    root->included_files[root->included_files_len++] = ptn_duplicate_string(path);
}

static PTN_UNUSED size_t ptn_runtime_alloc_object_id(PtnRuntime *runtime) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL) {
        return 0;
    }
    if (root->free_object_ids_len > 0) {
        return root->free_object_ids[--root->free_object_ids_len];
    }
    if (root->next_object_id == 0) {
        root->next_object_id = 1;
    }
    if (root->next_object_id > (size_t)INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    return root->next_object_id++;
}

static PTN_UNUSED void ptn_runtime_release_object_id(PtnRuntime *runtime, size_t object_id) {
    PtnRuntime *root = ptn_runtime_root(runtime);
    if (root == NULL || object_id == 0) {
        return;
    }
    if (root->free_object_ids_len == root->free_object_ids_capacity) {
        size_t new_capacity = root->free_object_ids_capacity == 0
            ? 8
            : root->free_object_ids_capacity * 2;
        if (new_capacity < root->free_object_ids_capacity ||
            new_capacity > SIZE_MAX / sizeof(size_t)) {
            ptn_abort_out_of_memory();
        }
        size_t *new_ids = realloc(root->free_object_ids, new_capacity * sizeof(size_t));
        if (new_ids == NULL) {
            ptn_abort_out_of_memory();
        }
        root->free_object_ids = new_ids;
        root->free_object_ids_capacity = new_capacity;
    }
    root->free_object_ids[root->free_object_ids_len++] = object_id;
}

#ifdef PTN_HAS_INTERNAL_FUNCTION_DISPATCH
static PTN_UNUSED int ptn_internal_class_name_is_reflection_class(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_attribute(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_object(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_extension(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_function(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_method(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_class_constant(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_named_type(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_parameter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_property(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_constant(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_reflection_reference(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_array_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_array_object(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_callback_filter_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_filter_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_infinite_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_iterator_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_limit_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_recursive_array_iterator(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_sensitive_parameter(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_sensitive_parameter_value(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_attribute(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_deprecated(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_no_discard(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_datetime_immutable(const char *class_name);
static PTN_UNUSED int ptn_internal_class_name_is_datetime_zone(const char *class_name);
static int ptn_internal_class_exists_name(const char *class_name);
static int ptn_internal_interface_exists_name(const char *name);
static PTN_UNUSED int ptn_internal_class_method_exists(const char *class_name, const char *method_name);
static PTN_UNUSED int ptn_internal_class_static_method_exists(const char *class_name, const char *method_name);
static PTN_UNUSED int ptn_runtime_class_exists(PtnRuntime *runtime, const char *class_name);
static PTN_UNUSED int ptn_runtime_interface_exists(
    PtnRuntime *runtime,
    const char *interface_name
);
static PTN_UNUSED int ptn_runtime_class_or_interface_exists(
    PtnRuntime *runtime,
    const char *class_name
);
static PTN_UNUSED PtnValue ptn_reflection_class_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_object_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_extension_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_function_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_method_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_constant_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_constant_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_value_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_property_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_attribute_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_deprecated_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_no_discard_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_value_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_constant_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_constant_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_attribute_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_method_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_named_type_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_parameter_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_property_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_reference_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
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
static PTN_UNUSED PtnValue ptn_reflection_extension_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_sensitive_parameter_value_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_attribute_metadata_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_reflection_class_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_iterator_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_recursive_array_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_object_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_object_clone(
    PtnRuntime *runtime,
    PtnValue source,
    size_t line
);
static PTN_UNUSED PtnValue ptn_array_object_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED int ptn_internal_cast_array_object(PtnValue value, PtnValue *array_out);
static PTN_UNUSED PtnValue ptn_iterator_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_iterator_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_callback_filter_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_callback_filter_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_filter_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_infinite_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_limit_iterator_new(
    PtnRuntime *runtime,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_limit_iterator_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_datetime_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_datetime_zone_new(
    PtnRuntime *runtime,
    const char *class_name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_datetime_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED PtnValue ptn_datetime_zone_clone(PtnRuntime *runtime, PtnValue source, size_t line);
static PTN_UNUSED PtnValue ptn_datetime_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_datetime_zone_call_method(
    PtnRuntime *runtime,
    PtnValue receiver,
    const char *name,
    size_t argc,
    const PtnValue *args,
    size_t line
);
static PTN_UNUSED PtnValue ptn_internal_class_static_call_method(
    PtnRuntime *runtime,
    const char *class_name,
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
            if (errno == 0 && end != configured && *end == '\0' && parsed >= -1 && parsed <= 53) {
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

static PTN_UNUSED int ptn_same_scalar_double(double left, double right) {
    return memcmp(&left, &right, sizeof(double)) == 0;
}

static PTN_UNUSED void ptn_format_scalar_shortest_float(double value, char *buffer, size_t buffer_size) {
    for (int precision = 1; precision <= 17; precision++) {
        char candidate[64];
        char *end = NULL;
        double reparsed;
        snprintf(candidate, sizeof(candidate), "%.*g", precision, value);
        ptn_normalize_scalar_float_exponent(candidate);
        ptn_scalar_float_ensure_exponent_decimal(candidate);
        errno = 0;
        reparsed = strtod(candidate, &end);
        if (errno == 0 && end != NULL && *end == '\0' && ptn_same_scalar_double(reparsed, value)) {
            int written = snprintf(buffer, buffer_size, "%s", candidate);
            if (written < 0 || (size_t)written >= buffer_size) {
                ptn_abort_out_of_memory();
            }
            return;
        }
    }

    int written = snprintf(buffer, buffer_size, "%.17g", value);
    if (written < 0 || (size_t)written >= buffer_size) {
        ptn_abort_out_of_memory();
    }
    ptn_normalize_scalar_float_exponent(buffer);
    ptn_scalar_float_ensure_exponent_decimal(buffer);
}

static PTN_UNUSED void ptn_format_scalar_float(double value, char *buffer, size_t buffer_size) {
    int written;
    if (isnan(value)) {
        written = snprintf(buffer, buffer_size, "NAN");
    } else if (isinf(value)) {
        written = snprintf(buffer, buffer_size, signbit(value) ? "-INF" : "INF");
    } else if (ptn_float_precision() < 0) {
        ptn_format_scalar_shortest_float(value, buffer, buffer_size);
        return;
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

static PTN_UNUSED PtnValue ptn_missing(void) {
    PtnValue value;
    value.type = PTN_NULL;
    value.owned = -1;
    return value;
}

static PTN_UNUSED int ptn_value_is_missing(PtnValue value) {
    return value.type == PTN_NULL && value.owned == -1;
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

static PTN_UNUSED PtnValue ptn_closure(
    PtnRuntime *runtime,
    size_t function_index,
    const char *display_name,
    PtnFunctionMetadata metadata,
    int is_static,
    int uses_this
) {
    PtnClosure *closure = malloc(sizeof(PtnClosure));
    if (closure == NULL) {
        ptn_abort_out_of_memory();
    }
    closure->refcount = 1;
    closure->object_id = ptn_runtime_alloc_object_id(runtime);
    closure->lifecycle_runtime = ptn_runtime_root(runtime);
    closure->function_index = function_index;
    closure->display_name = display_name;
    closure->metadata = metadata;
    closure->scope_class_name = NULL;
    closure->called_class_name = NULL;
    closure->is_static = is_static;
    closure->uses_this = uses_this;
    closure->captures.items = NULL;
    closure->captures.len = 0;
    closure->captures.capacity = 0;
    closure->captures.index_slots = NULL;
    closure->captures.index_capacity = 0;
    closure->has_wrapped_callable = 0;
    closure->wrapped_callable = ptn_null();
    closure->bound_scope_name = NULL;
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

static PTN_UNUSED void ptn_exception_retain(PtnException *exception) {
    if (exception == NULL) {
        return;
    }
    if (exception->refcount == SIZE_MAX) {
        ptn_abort_out_of_memory();
    }
    exception->refcount++;
}

static int64_t ptn_next_resource_id = 5;

static PTN_UNUSED PtnMemoryStream *ptn_memory_stream_new(size_t max_memory, int writable, int append) {
    PtnMemoryStream *stream = malloc(sizeof(PtnMemoryStream));
    if (stream == NULL) {
        ptn_abort_out_of_memory();
    }
    stream->data = NULL;
    stream->len = 0;
    stream->capacity = 0;
    stream->position = 0;
    stream->max_memory = max_memory;
    stream->writable = writable;
    stream->append = append;
    stream->spilled = 0;
    stream->eof = 0;
    stream->error = 0;
    return stream;
}

static PTN_UNUSED void ptn_memory_stream_free(PtnMemoryStream *stream) {
    if (stream == NULL) {
        return;
    }
    free(stream->data);
    free(stream);
}

static PTN_UNUSED int ptn_resource_is_open(PtnResource *resource) {
    if (resource == NULL) {
        return 0;
    }
    return resource->stream != NULL ||
        resource->directory != NULL ||
        resource->memory_stream != NULL ||
        strcmp(resource->type_name, "stream") != 0;
}

static PTN_UNUSED int ptn_stream_resource_is_open(PtnResource *resource) {
    return resource != NULL && (resource->stream != NULL || resource->memory_stream != NULL);
}

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
    resource->directory = NULL;
    resource->stream_uri = uri == NULL ? NULL : ptn_duplicate_string(uri);
    resource->stream_mode = mode == NULL ? NULL : ptn_duplicate_string(mode);
    resource->stream_backend = PTN_STREAM_BACKEND_FILE;
    resource->memory_stream = NULL;
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->persistent = 0;
    return resource;
}

static PTN_UNUSED PtnResource *ptn_resource_new_memory_stream(
    const char *uri,
    const char *mode,
    PtnStreamBackend backend,
    size_t max_memory,
    int writable,
    int append
) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        free(resource);
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = "stream";
    resource->stream = NULL;
    resource->directory = NULL;
    resource->stream_uri = uri == NULL ? NULL : ptn_duplicate_string(uri);
    resource->stream_mode = mode == NULL ? NULL : ptn_duplicate_string(mode);
    resource->stream_backend = backend;
    resource->memory_stream = ptn_memory_stream_new(max_memory, writable, append);
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->persistent = 0;
    return resource;
}

static PTN_UNUSED PtnResource *ptn_resource_new_directory(void *directory, const char *uri) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
#if !defined(_WIN32)
        if (directory != NULL) {
            closedir((DIR *)directory);
        }
#endif
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = "stream";
    resource->stream = NULL;
    resource->directory = directory;
    resource->stream_uri = uri == NULL ? NULL : ptn_duplicate_string(uri);
    resource->stream_mode = ptn_duplicate_string("r");
    resource->stream_backend = PTN_STREAM_BACKEND_FILE;
    resource->memory_stream = NULL;
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->persistent = 0;
    return resource;
}

static PTN_UNUSED PtnResource *ptn_resource_new_named(const char *type_name) {
    PtnResource *resource = malloc(sizeof(PtnResource));
    if (resource == NULL) {
        ptn_abort_out_of_memory();
    }
    if (ptn_next_resource_id == INT64_MAX) {
        ptn_abort_out_of_memory();
    }
    resource->refcount = 1;
    resource->id = ptn_next_resource_id++;
    resource->type_name = type_name;
    resource->stream = NULL;
    resource->directory = NULL;
    resource->stream_uri = NULL;
    resource->stream_mode = NULL;
    resource->stream_backend = PTN_STREAM_BACKEND_FILE;
    resource->memory_stream = NULL;
    resource->read_filters = NULL;
    resource->write_filters = NULL;
    resource->persistent = 0;
    return resource;
}

static PTN_UNUSED int ptn_memory_stream_reserve(PtnMemoryStream *stream, size_t required) {
    if (required <= stream->capacity) {
        return 1;
    }
    size_t new_capacity = stream->capacity == 0 ? 128 : stream->capacity;
    while (new_capacity < required) {
        if (new_capacity > SIZE_MAX / 2) {
            return 0;
        }
        new_capacity *= 2;
    }
    unsigned char *new_data = realloc(stream->data, new_capacity);
    if (new_data == NULL) {
        return 0;
    }
    stream->data = new_data;
    stream->capacity = new_capacity;
    return 1;
}

static PTN_UNUSED void ptn_memory_stream_note_size(PtnResource *resource, PtnMemoryStream *stream) {
    if (
        resource->stream_backend == PTN_STREAM_BACKEND_TEMP &&
        stream->max_memory != SIZE_MAX &&
        stream->len > stream->max_memory
    ) {
        stream->spilled = 1;
    }
}

static PTN_UNUSED size_t ptn_stream_write_bytes(PtnResource *resource, const void *data, size_t len) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->memory_stream == NULL) {
        return fwrite(data, 1, len, resource->stream);
    }

    PtnMemoryStream *stream = resource->memory_stream;
    if (!stream->writable) {
        stream->error = 1;
        errno = EBADF;
        return 0;
    }
    if (stream->append) {
        stream->position = stream->len;
    }
    if (stream->position > SIZE_MAX - len) {
        ptn_abort_out_of_memory();
    }
    size_t end = stream->position + len;
    if (!ptn_memory_stream_reserve(stream, end)) {
        ptn_abort_out_of_memory();
    }
    if (stream->position > stream->len) {
        memset(stream->data + stream->len, 0, stream->position - stream->len);
    }
    if (len != 0) {
        memcpy(stream->data + stream->position, data, len);
    }
    stream->position = end;
    if (end > stream->len) {
        stream->len = end;
        ptn_memory_stream_note_size(resource, stream);
    }
    stream->error = 0;
    return len;
}

static PTN_UNUSED size_t ptn_stream_read_bytes(PtnResource *resource, void *buffer, size_t len) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->memory_stream == NULL) {
        return fread(buffer, 1, len, resource->stream);
    }

    PtnMemoryStream *stream = resource->memory_stream;
    if (len == 0) {
        return 0;
    }
    if (stream->position >= stream->len) {
        stream->eof = 1;
        return 0;
    }
    size_t available = stream->len - stream->position;
    size_t read_len = available < len ? available : len;
    memcpy(buffer, stream->data + stream->position, read_len);
    stream->position += read_len;
    stream->eof = read_len < len;
    stream->error = 0;
    return read_len;
}

static PTN_UNUSED int ptn_stream_get_byte(PtnResource *resource) {
    if (resource == NULL) {
        return EOF;
    }
    if (resource->memory_stream == NULL) {
        return fgetc(resource->stream);
    }
    unsigned char byte = 0;
    return ptn_stream_read_bytes(resource, &byte, 1) == 1 ? (int)byte : EOF;
}

static PTN_UNUSED int ptn_stream_unget_byte(PtnResource *resource, int byte) {
    if (resource == NULL) {
        return EOF;
    }
    if (resource->memory_stream == NULL) {
        return ungetc(byte, resource->stream);
    }
    PtnMemoryStream *stream = resource->memory_stream;
    if (stream->position == 0 || byte == EOF) {
        return EOF;
    }
    stream->position--;
    stream->eof = 0;
    stream->error = 0;
    return byte;
}

static PTN_UNUSED int ptn_stream_seek(PtnResource *resource, int64_t offset, int whence) {
    if (resource == NULL) {
        return -1;
    }
    if (resource->memory_stream == NULL) {
        return fseek(resource->stream, (long)offset, whence);
    }
    PtnMemoryStream *stream = resource->memory_stream;
    size_t base_size = 0;
    if (whence == SEEK_SET) {
        base_size = 0;
    } else if (whence == SEEK_CUR) {
        base_size = stream->position;
    } else if (whence == SEEK_END) {
        base_size = stream->len;
    } else {
        return -1;
    }
    if (base_size > (size_t)INT64_MAX) {
        return -1;
    }
    int64_t base = (int64_t)base_size;
    if (offset < 0) {
        if (offset == INT64_MIN || base < -offset) {
            return -1;
        }
    } else if (base > INT64_MAX - offset) {
        return -1;
    }
    int64_t target = base + offset;
    if (target < 0) {
        return -1;
    }
    stream->position = (size_t)target;
    stream->eof = 0;
    stream->error = 0;
    return 0;
}

static PTN_UNUSED int64_t ptn_stream_tell(PtnResource *resource) {
    if (resource == NULL) {
        return -1;
    }
    if (resource->memory_stream == NULL) {
        long position = ftell(resource->stream);
        return position < 0 ? -1 : (int64_t)position;
    }
    PtnMemoryStream *stream = resource->memory_stream;
    if (stream->position > (size_t)INT64_MAX) {
        return -1;
    }
    return (int64_t)stream->position;
}

static PTN_UNUSED int ptn_stream_flush(PtnResource *resource) {
    if (resource == NULL) {
        return -1;
    }
    if (resource->memory_stream == NULL) {
        return fflush(resource->stream);
    }
    resource->memory_stream->error = 0;
    return 0;
}

static PTN_UNUSED int ptn_stream_eof(PtnResource *resource) {
    if (resource == NULL) {
        return 1;
    }
    if (resource->memory_stream == NULL) {
        return feof(resource->stream) != 0;
    }
    return resource->memory_stream->eof != 0;
}

static PTN_UNUSED int ptn_stream_error(PtnResource *resource) {
    if (resource == NULL) {
        return 1;
    }
    if (resource->memory_stream == NULL) {
        return ferror(resource->stream) != 0;
    }
    return resource->memory_stream->error != 0;
}

static PTN_UNUSED void ptn_stream_clear_error(PtnResource *resource) {
    if (resource == NULL) {
        return;
    }
    if (resource->memory_stream == NULL) {
        clearerr(resource->stream);
        return;
    }
    resource->memory_stream->eof = 0;
    resource->memory_stream->error = 0;
}

static PTN_UNUSED int ptn_stream_truncate(PtnResource *resource, int64_t size) {
    if (resource == NULL) {
        return 0;
    }
    if (resource->memory_stream == NULL) {
        int descriptor = -1;
#if defined(_WIN32)
        descriptor = _fileno(resource->stream);
#else
        descriptor = fileno(resource->stream);
#endif
        if (descriptor < 0) {
            return 0;
        }
#if defined(_WIN32)
        return _chsize_s(descriptor, size) == 0;
#else
        return ftruncate(descriptor, (off_t)size) == 0;
#endif
    }

    PtnMemoryStream *stream = resource->memory_stream;
    if (!stream->writable) {
        stream->error = 1;
        errno = EBADF;
        return 0;
    }
    size_t new_len = (size_t)size;
    if (!ptn_memory_stream_reserve(stream, new_len)) {
        ptn_abort_out_of_memory();
    }
    if (new_len > stream->len) {
        memset(stream->data + stream->len, 0, new_len - stream->len);
    }
    stream->len = new_len;
    ptn_memory_stream_note_size(resource, stream);
    stream->eof = 0;
    stream->error = 0;
    return 1;
}

static PTN_UNUSED void ptn_stream_filter_chain_free(PtnStreamFilter *filter) {
    while (filter != NULL) {
        PtnStreamFilter *next = filter->next;
        free(filter->name);
        free(filter);
        filter = next;
    }
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
    if (resource == NULL) {
        return;
    }
    if (resource->persistent) {
        return;
    }
    if (resource->stream != NULL) {
        fclose(resource->stream);
        resource->stream = NULL;
    }
    if (resource->memory_stream != NULL) {
        ptn_memory_stream_free(resource->memory_stream);
        resource->memory_stream = NULL;
    }
#if !defined(_WIN32)
    if (resource->directory != NULL) {
        closedir((DIR *)resource->directory);
        resource->directory = NULL;
    }
#endif
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
    ptn_stream_filter_chain_free(resource->read_filters);
    ptn_stream_filter_chain_free(resource->write_filters);
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
    static PtnResource stdin_resource = {
        SIZE_MAX,
        1,
        "stream",
        NULL,
        NULL,
        NULL,
        NULL,
        PTN_STREAM_BACKEND_FILE,
        NULL,
        NULL,
        NULL,
        1
    };
    static PtnResource stdout_resource = {
        SIZE_MAX,
        2,
        "stream",
        NULL,
        NULL,
        NULL,
        NULL,
        PTN_STREAM_BACKEND_FILE,
        NULL,
        NULL,
        NULL,
        1
    };
    static PtnResource stderr_resource = {
        SIZE_MAX,
        3,
        "stream",
        NULL,
        NULL,
        NULL,
        NULL,
        PTN_STREAM_BACKEND_FILE,
        NULL,
        NULL,
        NULL,
        1
    };
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
