#ifndef PTN_RUNTIME_PHP_HASH_H
#define PTN_RUNTIME_PHP_HASH_H

#include <assert.h>
#include <ctype.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define PHP_HASH_HMAC 0x0001
#define PHP_HASH_SERIALIZE_MAGIC_SPEC 2
#define L64 INT64_C

#define PHPAPI
#define PHP_HASH_API
#define ZEND_ATTRIBUTE_UNUSED __attribute__((unused))
#define ZEND_SECURE_ZERO(ptr, len) memset((ptr), 0, (len))
#define ZEND_ASSERT(expr) assert(expr)
#define ZEND_SET_ALIGNED(alignment, declaration) declaration __attribute__((aligned(alignment)))
#define UNEXPECTED(expr) (expr)
#define EXPECTED(expr) (expr)

#define SUCCESS 0
#define FAILURE -1
#define E_DEPRECATED 8192
#define E_WARNING 2

typedef int zend_result;
typedef int64_t zend_long;
typedef struct _HashTable HashTable;
typedef struct _zend_object zend_object;
typedef struct _zend_module_entry zend_module_entry;
typedef struct _zend_class_entry zend_class_entry;

typedef enum {
    IS_UNDEF = 0,
    IS_NULL = 1,
    IS_FALSE = 2,
    IS_TRUE = 3,
    IS_LONG = 4,
    IS_DOUBLE = 5,
    IS_STRING = 6,
    IS_ARRAY = 7,
} ptn_zval_type;

typedef struct _zend_string {
    char *val;
    size_t len;
} zend_string;

typedef struct _zval {
    ptn_zval_type type;
    union {
        zend_long lval;
        zend_string *str;
    } value;
} zval;

#define Z_TYPE_P(zv) ((zv)->type)
#define Z_LVAL_P(zv) ((zv)->value.lval)
#define Z_STR_P(zv) ((zv)->value.str)
#define ZSTR_VAL(str) ((str)->val)
#define ZSTR_LEN(str) ((str)->len)
#define EG(name) (0)

static inline zval *zend_hash_str_find_deref(HashTable *ht, const char *key, size_t len) {
    (void)ht;
    (void)key;
    (void)len;
    return NULL;
}

static inline zend_string *zval_try_get_string(zval *value) {
    (void)value;
    return NULL;
}

static inline void zend_string_release(zend_string *value) {
    free(value);
}

static inline void php_error_docref(const char *docref, int type, const char *format, ...) {
    (void)docref;
    (void)type;
    (void)format;
}

static inline void zend_throw_error(void *ce, const char *format, ...) {
    (void)ce;
    (void)format;
}

static inline void *zend_mempcpy(void *dest, const void *src, size_t len) {
    memcpy(dest, src, len);
    return (char *)dest + len;
}

static inline void *ecalloc(size_t nmemb, size_t size) {
    return calloc(nmemb, size);
}

typedef enum {
    HASH_SPEC_SUCCESS = 0,
    HASH_SPEC_FAILURE = -1,
    WRONG_CONTEXT_SIZE = -999,
    BYTE_OFFSET_POS_ERROR = -1000,
    CONTEXT_VALIDATION_FAILURE = -2000,
} hash_spec_result;

typedef struct _php_hashcontext_object php_hashcontext_object;

typedef void (*php_hash_init_func_t)(void *context, HashTable *args);
typedef void (*php_hash_update_func_t)(void *context, const unsigned char *buf, size_t count);
typedef void (*php_hash_final_func_t)(unsigned char *digest, void *context);
typedef zend_result (*php_hash_copy_func_t)(const void *ops, const void *orig_context, void *dest_context);
typedef hash_spec_result (*php_hash_serialize_func_t)(const php_hashcontext_object *hash, zend_long *magic, zval *zv);
typedef hash_spec_result (*php_hash_unserialize_func_t)(php_hashcontext_object *hash, zend_long magic, const zval *zv);

typedef struct _php_hash_ops {
    const char *algo;
    php_hash_init_func_t hash_init;
    php_hash_update_func_t hash_update;
    php_hash_final_func_t hash_final;
    php_hash_copy_func_t hash_copy;
    php_hash_serialize_func_t hash_serialize;
    php_hash_unserialize_func_t hash_unserialize;
    const char *serialize_spec;
    size_t digest_size;
    size_t block_size;
    size_t context_size;
    unsigned is_crypto: 1;
} php_hash_ops;

struct _php_hashcontext_object {
    const php_hash_ops *ops;
    void *context;
    zend_long options;
    unsigned char *key;
};

extern const php_hash_ops php_hash_md2_ops;
extern const php_hash_ops php_hash_md4_ops;
extern const php_hash_ops php_hash_md5_ops;
extern const php_hash_ops php_hash_sha1_ops;
extern const php_hash_ops php_hash_sha224_ops;
extern const php_hash_ops php_hash_sha256_ops;
extern const php_hash_ops php_hash_sha384_ops;
extern const php_hash_ops php_hash_sha512_ops;
extern const php_hash_ops php_hash_sha512_256_ops;
extern const php_hash_ops php_hash_sha512_224_ops;
extern const php_hash_ops php_hash_sha3_224_ops;
extern const php_hash_ops php_hash_sha3_256_ops;
extern const php_hash_ops php_hash_sha3_384_ops;
extern const php_hash_ops php_hash_sha3_512_ops;
extern const php_hash_ops php_hash_ripemd128_ops;
extern const php_hash_ops php_hash_ripemd160_ops;
extern const php_hash_ops php_hash_ripemd256_ops;
extern const php_hash_ops php_hash_ripemd320_ops;
extern const php_hash_ops php_hash_whirlpool_ops;
extern const php_hash_ops php_hash_3tiger128_ops;
extern const php_hash_ops php_hash_3tiger160_ops;
extern const php_hash_ops php_hash_3tiger192_ops;
extern const php_hash_ops php_hash_4tiger128_ops;
extern const php_hash_ops php_hash_4tiger160_ops;
extern const php_hash_ops php_hash_4tiger192_ops;
extern const php_hash_ops php_hash_snefru_ops;
extern const php_hash_ops php_hash_gost_ops;
extern const php_hash_ops php_hash_gost_crypto_ops;
extern const php_hash_ops php_hash_adler32_ops;
extern const php_hash_ops php_hash_crc32_ops;
extern const php_hash_ops php_hash_crc32b_ops;
extern const php_hash_ops php_hash_crc32c_ops;
extern const php_hash_ops php_hash_fnv132_ops;
extern const php_hash_ops php_hash_fnv1a32_ops;
extern const php_hash_ops php_hash_fnv164_ops;
extern const php_hash_ops php_hash_fnv1a64_ops;
extern const php_hash_ops php_hash_joaat_ops;
extern const php_hash_ops php_hash_murmur3a_ops;
extern const php_hash_ops php_hash_murmur3c_ops;
extern const php_hash_ops php_hash_murmur3f_ops;
extern const php_hash_ops php_hash_xxh32_ops;
extern const php_hash_ops php_hash_xxh64_ops;
extern const php_hash_ops php_hash_xxh3_64_ops;
extern const php_hash_ops php_hash_xxh3_128_ops;

#define PHP_HASH_HAVAL_OPS(p, b) extern const php_hash_ops php_hash_##p##haval##b##_ops;
PHP_HASH_HAVAL_OPS(3, 128)
PHP_HASH_HAVAL_OPS(3, 160)
PHP_HASH_HAVAL_OPS(3, 192)
PHP_HASH_HAVAL_OPS(3, 224)
PHP_HASH_HAVAL_OPS(3, 256)
PHP_HASH_HAVAL_OPS(4, 128)
PHP_HASH_HAVAL_OPS(4, 160)
PHP_HASH_HAVAL_OPS(4, 192)
PHP_HASH_HAVAL_OPS(4, 224)
PHP_HASH_HAVAL_OPS(4, 256)
PHP_HASH_HAVAL_OPS(5, 128)
PHP_HASH_HAVAL_OPS(5, 160)
PHP_HASH_HAVAL_OPS(5, 192)
PHP_HASH_HAVAL_OPS(5, 224)
PHP_HASH_HAVAL_OPS(5, 256)
#undef PHP_HASH_HAVAL_OPS

zend_result php_hash_copy(const void *ops, const void *orig_context, void *dest_context);
hash_spec_result php_hash_serialize(const php_hashcontext_object *context, zend_long *magic, zval *zv);
hash_spec_result php_hash_unserialize(php_hashcontext_object *context, zend_long magic, const zval *zv);
hash_spec_result php_hash_serialize_spec(const php_hashcontext_object *context, zval *zv, const char *spec);
hash_spec_result php_hash_unserialize_spec(php_hashcontext_object *hash, const zval *zv, const char *spec);

static inline void php_hash_bin2hex(char *out, const unsigned char *in, size_t in_len) {
    static const char hexits[17] = "0123456789abcdef";
    for (size_t i = 0; i < in_len; i++) {
        out[i * 2] = hexits[in[i] >> 4];
        out[i * 2 + 1] = hexits[in[i] & 0x0f];
    }
}

#endif
