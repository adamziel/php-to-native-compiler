#ifndef PTN_HASH_EXT_H
#define PTN_HASH_EXT_H

#include <stddef.h>

#ifndef PTN_RUNTIME_PHP_HASH_H
#define PHP_HASH_HMAC 0x0001
#define PHP_HASH_SERIALIZE_MAGIC_SPEC 2
#define SUCCESS 0
#define FAILURE -1

typedef enum {
    HASH_SPEC_SUCCESS = 0,
    HASH_SPEC_FAILURE = -1,
    WRONG_CONTEXT_SIZE = -999,
    BYTE_OFFSET_POS_ERROR = -1000,
    CONTEXT_VALIDATION_FAILURE = -2000,
} hash_spec_result;

typedef struct _php_hash_ops {
    const char *algo;
    void *hash_init;
    void *hash_update;
    void *hash_final;
    void *hash_copy;
    void *hash_serialize;
    void *hash_unserialize;
    const char *serialize_spec;
    size_t digest_size;
    size_t block_size;
    size_t context_size;
    unsigned is_crypto: 1;
} php_hash_ops;
#endif

size_t ptn_hash_ext_algo_count(void);
const char *ptn_hash_ext_algo_name(size_t index);
const php_hash_ops *ptn_hash_ext_fetch_ops(const char *algo);
void *ptn_hash_ext_alloc_context(const php_hash_ops *ops);
void ptn_hash_ext_free_context(void *context);
void ptn_hash_ext_init_context(const php_hash_ops *ops, void *context);
int ptn_hash_ext_copy_context(const php_hash_ops *ops, const void *source, void *dest);
void ptn_hash_ext_update(const php_hash_ops *ops, void *context, const unsigned char *data, size_t data_len);
void ptn_hash_ext_final(const php_hash_ops *ops, void *context, unsigned char *digest);
int ptn_hash_ext_hmac_init(const php_hash_ops *ops, void *context, const unsigned char *key, size_t key_len);
int ptn_hash_ext_hmac_final(const php_hash_ops *ops, const void *context, const unsigned char *key, size_t key_len, unsigned char *digest);
int ptn_hash_ext_digest(const php_hash_ops *ops, const unsigned char *data, size_t data_len, unsigned char *digest);
int ptn_hash_ext_hmac_digest(const php_hash_ops *ops, const unsigned char *key, size_t key_len, const unsigned char *data, size_t data_len, unsigned char *digest);
int ptn_hash_ext_validate_context(const php_hash_ops *ops, void *context);

#endif
