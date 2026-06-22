#ifndef PTN_HASH_EXT_H
#define PTN_HASH_EXT_H

#include "php_hash.h"

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
