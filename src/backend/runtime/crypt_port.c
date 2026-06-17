/*
 * Portable crypt(3) algorithms adapted from php-src ext/standard crypt sources.
 * The PHP source files carry BSD-3-Clause or public-domain notices in their
 * copied blocks below. This wrapper keeps the code self-contained for PTN's
 * single-file generated C runtime.
 */

#include <stdbool.h>

#ifndef MIN
#define MIN(a, b) (((a) < (b)) ? (a) : (b))
#endif
#ifndef MAX
#define MAX(a, b) (((a) > (b)) ? (a) : (b))
#endif
#define PHPAPI
#define ZEND_TLS static
#define ZEND_NONSTRING
#define ZEND_ATTRIBUTE_UNUSED __attribute__((unused))
#define ZEND_SECURE_ZERO(ptr, len) memset((ptr), 0, (len))
#define ZEND_SET_ALIGNED(alignment, decl) decl __attribute__((aligned(alignment)))
#define ZEND_STRTOUL strtoull
typedef unsigned long long zend_ulong;
typedef struct _PtnCryptHashTable { int unused; } HashTable;
#define ALLOCA_FLAG(name) int name = 0
#define SET_ALLOCA_FLAG(name) ((name) = 0)
#define do_alloca(size, use_heap) ((use_heap) = 1, malloc(size))
#define free_alloca(ptr, use_heap) do { (void)(use_heap); free(ptr); } while (0)
static void *zend_mempcpy(void *dst, const void *src, size_t len) { memcpy(dst, src, len); return (char *)dst + len; }
static size_t ptn_crypt_strlcpy(char *dst, const char *src, size_t size) { size_t len = strlen(src); if (size != 0) { size_t copy = len >= size ? size - 1 : len; memcpy(dst, src, copy); dst[copy] = '\0'; } return len; }
#define strlcpy ptn_crypt_strlcpy

typedef struct {
    uint32_t lo, hi;
    uint32_t a, b, c, d;
    unsigned char buffer[64];
    uint32_t block[16];
} PHP_MD5_CTX;
#define PHP_MD5Init(ctx) PHP_MD5InitArgs(ctx, NULL)
void PHP_MD5InitArgs(PHP_MD5_CTX *context, HashTable *args);
void PHP_MD5Update(PHP_MD5_CTX *ctx, const void *data, size_t size);
void PHP_MD5Final(unsigned char *result, PHP_MD5_CTX *ctx);

#define MD5_HASH_MAX_LEN 120
struct php_crypt_extended_data {
    int initialized;
    uint32_t saltbits;
    uint32_t old_salt;
    uint32_t en_keysl[16], en_keysr[16];
    uint32_t de_keysl[16], de_keysr[16];
    uint32_t old_rawkey0, old_rawkey1;
    char output[21];
};
void _crypt_extended_init(void);
char *_crypt_extended_r(const unsigned char *key, const char *setting, struct php_crypt_extended_data *data);
char *php_crypt_blowfish_rn(const char *key, const char *setting, char *output, int size);
char *php_md5_crypt_r(const char *pw, const char *salt, char *out);
char *php_sha512_crypt_r(const char *key, const char *salt, char *buffer, int buflen);
char *php_sha256_crypt_r(const char *key, const char *salt, char *buffer, int buflen);
/*
 * This is an OpenSSL-compatible implementation of the RSA Data Security,
 * Inc. MD5 Message-Digest Algorithm (RFC 1321).
 *
 * Written by Solar Designer <solar at openwall.com> in 2001, and placed
 * in the public domain.  There's absolutely no warranty.
 *
 * This differs from Colin Plumb's older public domain implementation in
 * that no 32-bit integer data type is required, there's no compile-time
 * endianness configuration, and the function prototypes match OpenSSL's.
 * The primary goals are portability and ease of use.
 *
 * This implementation is meant to be fast, but not as fast as possible.
 * Some known optimizations are not included to reduce source code size
 * and avoid compile-time configuration.
 */

#include <string.h>

/*
 * The basic MD5 functions.
 *
 * F and G are optimized compared to their RFC 1321 definitions for
 * architectures that lack an AND-NOT instruction, just like in Colin Plumb's
 * implementation.
 */
#define F(x, y, z)			((z) ^ ((x) & ((y) ^ (z))))
#define G(x, y, z)			((y) ^ ((z) & ((x) ^ (y))))
#define H(x, y, z)			((x) ^ (y) ^ (z))
#define I(x, y, z)			((y) ^ ((x) | ~(z)))

/*
 * The MD5 transformation for all four rounds.
 */
#define STEP(f, a, b, c, d, x, t, s) \
	(a) += f((b), (c), (d)) + (x) + (t); \
	(a) = (((a) << (s)) | (((a) & 0xffffffff) >> (32 - (s)))); \
	(a) += (b);

/*
 * SET reads 4 input bytes in little-endian byte order and stores them
 * in a properly aligned word in host byte order.
 *
 * The check for little-endian architectures that tolerate unaligned
 * memory accesses is just an optimization.  Nothing will break if it
 * doesn't work.
 */
#if defined(__i386__) || defined(__x86_64__) || defined(__vax__)
typedef ZEND_SET_ALIGNED(1, uint32_t unaligned_uint32_t);
# define SET(n) \
	(*(unaligned_uint32_t *)&ptr[(n) * 4])
# define GET(n) \
	SET(n)
#else
# define SET(n) \
	(ctx->block[(n)] = \
	(uint32_t)ptr[(n) * 4] | \
	((uint32_t)ptr[(n) * 4 + 1] << 8) | \
	((uint32_t)ptr[(n) * 4 + 2] << 16) | \
	((uint32_t)ptr[(n) * 4 + 3] << 24))
# define GET(n) \
	(ctx->block[(n)])
#endif

/*
 * This processes one or more 64-byte data blocks, but does NOT update
 * the bit counters.  There are no alignment requirements.
 */
static const void *body(PHP_MD5_CTX *ctx, const void *data, size_t size)
{
	const unsigned char *ptr;
	uint32_t a, b, c, d;
	uint32_t saved_a, saved_b, saved_c, saved_d;

	ptr = data;

	a = ctx->a;
	b = ctx->b;
	c = ctx->c;
	d = ctx->d;

	do {
		saved_a = a;
		saved_b = b;
		saved_c = c;
		saved_d = d;

/* Round 1 */
		STEP(F, a, b, c, d, SET(0), 0xd76aa478, 7)
		STEP(F, d, a, b, c, SET(1), 0xe8c7b756, 12)
		STEP(F, c, d, a, b, SET(2), 0x242070db, 17)
		STEP(F, b, c, d, a, SET(3), 0xc1bdceee, 22)
		STEP(F, a, b, c, d, SET(4), 0xf57c0faf, 7)
		STEP(F, d, a, b, c, SET(5), 0x4787c62a, 12)
		STEP(F, c, d, a, b, SET(6), 0xa8304613, 17)
		STEP(F, b, c, d, a, SET(7), 0xfd469501, 22)
		STEP(F, a, b, c, d, SET(8), 0x698098d8, 7)
		STEP(F, d, a, b, c, SET(9), 0x8b44f7af, 12)
		STEP(F, c, d, a, b, SET(10), 0xffff5bb1, 17)
		STEP(F, b, c, d, a, SET(11), 0x895cd7be, 22)
		STEP(F, a, b, c, d, SET(12), 0x6b901122, 7)
		STEP(F, d, a, b, c, SET(13), 0xfd987193, 12)
		STEP(F, c, d, a, b, SET(14), 0xa679438e, 17)
		STEP(F, b, c, d, a, SET(15), 0x49b40821, 22)

/* Round 2 */
		STEP(G, a, b, c, d, GET(1), 0xf61e2562, 5)
		STEP(G, d, a, b, c, GET(6), 0xc040b340, 9)
		STEP(G, c, d, a, b, GET(11), 0x265e5a51, 14)
		STEP(G, b, c, d, a, GET(0), 0xe9b6c7aa, 20)
		STEP(G, a, b, c, d, GET(5), 0xd62f105d, 5)
		STEP(G, d, a, b, c, GET(10), 0x02441453, 9)
		STEP(G, c, d, a, b, GET(15), 0xd8a1e681, 14)
		STEP(G, b, c, d, a, GET(4), 0xe7d3fbc8, 20)
		STEP(G, a, b, c, d, GET(9), 0x21e1cde6, 5)
		STEP(G, d, a, b, c, GET(14), 0xc33707d6, 9)
		STEP(G, c, d, a, b, GET(3), 0xf4d50d87, 14)
		STEP(G, b, c, d, a, GET(8), 0x455a14ed, 20)
		STEP(G, a, b, c, d, GET(13), 0xa9e3e905, 5)
		STEP(G, d, a, b, c, GET(2), 0xfcefa3f8, 9)
		STEP(G, c, d, a, b, GET(7), 0x676f02d9, 14)
		STEP(G, b, c, d, a, GET(12), 0x8d2a4c8a, 20)

/* Round 3 */
		STEP(H, a, b, c, d, GET(5), 0xfffa3942, 4)
		STEP(H, d, a, b, c, GET(8), 0x8771f681, 11)
		STEP(H, c, d, a, b, GET(11), 0x6d9d6122, 16)
		STEP(H, b, c, d, a, GET(14), 0xfde5380c, 23)
		STEP(H, a, b, c, d, GET(1), 0xa4beea44, 4)
		STEP(H, d, a, b, c, GET(4), 0x4bdecfa9, 11)
		STEP(H, c, d, a, b, GET(7), 0xf6bb4b60, 16)
		STEP(H, b, c, d, a, GET(10), 0xbebfbc70, 23)
		STEP(H, a, b, c, d, GET(13), 0x289b7ec6, 4)
		STEP(H, d, a, b, c, GET(0), 0xeaa127fa, 11)
		STEP(H, c, d, a, b, GET(3), 0xd4ef3085, 16)
		STEP(H, b, c, d, a, GET(6), 0x04881d05, 23)
		STEP(H, a, b, c, d, GET(9), 0xd9d4d039, 4)
		STEP(H, d, a, b, c, GET(12), 0xe6db99e5, 11)
		STEP(H, c, d, a, b, GET(15), 0x1fa27cf8, 16)
		STEP(H, b, c, d, a, GET(2), 0xc4ac5665, 23)

/* Round 4 */
		STEP(I, a, b, c, d, GET(0), 0xf4292244, 6)
		STEP(I, d, a, b, c, GET(7), 0x432aff97, 10)
		STEP(I, c, d, a, b, GET(14), 0xab9423a7, 15)
		STEP(I, b, c, d, a, GET(5), 0xfc93a039, 21)
		STEP(I, a, b, c, d, GET(12), 0x655b59c3, 6)
		STEP(I, d, a, b, c, GET(3), 0x8f0ccc92, 10)
		STEP(I, c, d, a, b, GET(10), 0xffeff47d, 15)
		STEP(I, b, c, d, a, GET(1), 0x85845dd1, 21)
		STEP(I, a, b, c, d, GET(8), 0x6fa87e4f, 6)
		STEP(I, d, a, b, c, GET(15), 0xfe2ce6e0, 10)
		STEP(I, c, d, a, b, GET(6), 0xa3014314, 15)
		STEP(I, b, c, d, a, GET(13), 0x4e0811a1, 21)
		STEP(I, a, b, c, d, GET(4), 0xf7537e82, 6)
		STEP(I, d, a, b, c, GET(11), 0xbd3af235, 10)
		STEP(I, c, d, a, b, GET(2), 0x2ad7d2bb, 15)
		STEP(I, b, c, d, a, GET(9), 0xeb86d391, 21)

		a += saved_a;
		b += saved_b;
		c += saved_c;
		d += saved_d;

		ptr += 64;
	} while (size -= 64);

	ctx->a = a;
	ctx->b = b;
	ctx->c = c;
	ctx->d = d;

	return ptr;
}

PHPAPI void PHP_MD5InitArgs(PHP_MD5_CTX *ctx, ZEND_ATTRIBUTE_UNUSED HashTable *args)
{
	ctx->a = 0x67452301;
	ctx->b = 0xefcdab89;
	ctx->c = 0x98badcfe;
	ctx->d = 0x10325476;

	ctx->lo = 0;
	ctx->hi = 0;
}

PHPAPI void PHP_MD5Update(PHP_MD5_CTX *ctx, const void *data, size_t size)
{
	uint32_t saved_lo;
	uint32_t used, free;

	saved_lo = ctx->lo;
	if ((ctx->lo = (saved_lo + size) & 0x1fffffff) < saved_lo) {
		ctx->hi++;
	}
	ctx->hi += size >> 29;

	used = saved_lo & 0x3f;

	if (used) {
		free = 64 - used;

		if (size < free) {
			memcpy(&ctx->buffer[used], data, size);
			return;
		}

		memcpy(&ctx->buffer[used], data, free);
		data = (unsigned char *)data + free;
		size -= free;
		body(ctx, ctx->buffer, 64);
	}

	if (size >= 64) {
		data = body(ctx, data, size & ~(size_t)0x3f);
		size &= 0x3f;
	}

	memcpy(ctx->buffer, data, size);
}

PHPAPI void PHP_MD5Final(unsigned char *result, PHP_MD5_CTX *ctx)
{
	uint32_t used, free;

	used = ctx->lo & 0x3f;

	ctx->buffer[used++] = 0x80;

	free = 64 - used;

	if (free < 8) {
		memset(&ctx->buffer[used], 0, free);
		body(ctx, ctx->buffer, 64);
		used = 0;
		free = 64;
	}

	memset(&ctx->buffer[used], 0, free - 8);

	ctx->lo <<= 3;
	ctx->buffer[56] = ctx->lo;
	ctx->buffer[57] = ctx->lo >> 8;
	ctx->buffer[58] = ctx->lo >> 16;
	ctx->buffer[59] = ctx->lo >> 24;
	ctx->buffer[60] = ctx->hi;
	ctx->buffer[61] = ctx->hi >> 8;
	ctx->buffer[62] = ctx->hi >> 16;
	ctx->buffer[63] = ctx->hi >> 24;

	body(ctx, ctx->buffer, 64);

	result[0] = ctx->a;
	result[1] = ctx->a >> 8;
	result[2] = ctx->a >> 16;
	result[3] = ctx->a >> 24;
	result[4] = ctx->b;
	result[5] = ctx->b >> 8;
	result[6] = ctx->b >> 16;
	result[7] = ctx->b >> 24;
	result[8] = ctx->c;
	result[9] = ctx->c >> 8;
	result[10] = ctx->c >> 16;
	result[11] = ctx->c >> 24;
	result[12] = ctx->d;
	result[13] = ctx->d >> 8;
	result[14] = ctx->d >> 16;
	result[15] = ctx->d >> 24;

	ZEND_SECURE_ZERO(ctx, sizeof(*ctx));
}

#undef F
#undef G
#undef H
#undef I
#undef STEP
#undef SET
#undef GET
/*
 * This version is derived from the original implementation of FreeSec
 * (release 1.1) by David Burren.  I've reviewed the changes made in
 * OpenBSD (as of 2.7) and modified the original code in a similar way
 * where applicable.  I've also made it reentrant and made a number of
 * other changes.
 * - Solar Designer <solar at openwall.com>
 */

/*
 * FreeSec: libcrypt for NetBSD
 *
 * Copyright (c) 1994 David Burren
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the author nor the names of other contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 *
 *	$Owl: Owl/packages/glibc/crypt_freesec.c,v 1.4 2005/11/16 13:08:32 solar Exp $
 *
 * This is an original implementation of the DES and the crypt(3) interfaces
 * by David Burren <davidb at werj.com.au>.
 *
 * An excellent reference on the underlying algorithm (and related
 * algorithms) is:
 *
 *	B. Schneier, Applied Cryptography: protocols, algorithms,
 *	and source code in C, John Wiley & Sons, 1994.
 *
 * Note that in that book's description of DES the lookups for the initial,
 * pbox, and final permutations are inverted (this has been brought to the
 * attention of the author).  A list of errata for this book has been
 * posted to the sci.crypt newsgroup by the author and is available for FTP.
 *
 * ARCHITECTURE ASSUMPTIONS:
 *	This code used to have some nasty ones, but these have been removed
 *	by now.  The code requires a 32-bit integer type, though.
 */

#include <sys/types.h>
#include <string.h>

#ifdef TEST
#include <stdio.h>
#endif


#define _PASSWORD_EFMT1 '_'

static const uint8_t IP[64] = {
	58, 50, 42, 34, 26, 18, 10,  2, 60, 52, 44, 36, 28, 20, 12,  4,
	62, 54, 46, 38, 30, 22, 14,  6, 64, 56, 48, 40, 32, 24, 16,  8,
	57, 49, 41, 33, 25, 17,  9,  1, 59, 51, 43, 35, 27, 19, 11,  3,
	61, 53, 45, 37, 29, 21, 13,  5, 63, 55, 47, 39, 31, 23, 15,  7
};

static const uint8_t key_perm[56] = {
	57, 49, 41, 33, 25, 17,  9,  1, 58, 50, 42, 34, 26, 18,
	10,  2, 59, 51, 43, 35, 27, 19, 11,  3, 60, 52, 44, 36,
	63, 55, 47, 39, 31, 23, 15,  7, 62, 54, 46, 38, 30, 22,
	14,  6, 61, 53, 45, 37, 29, 21, 13,  5, 28, 20, 12,  4
};

static const uint8_t key_shifts[16] = {
	1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1
};

static const uint8_t comp_perm[48] = {
	14, 17, 11, 24,  1,  5,  3, 28, 15,  6, 21, 10,
	23, 19, 12,  4, 26,  8, 16,  7, 27, 20, 13,  2,
	41, 52, 31, 37, 47, 55, 30, 40, 51, 45, 33, 48,
	44, 49, 39, 56, 34, 53, 46, 42, 50, 36, 29, 32
};

/*
 *	No E box is used, as it's replaced by some ANDs, shifts, and ORs.
 */

static const uint8_t sbox[8][64] = {
	{
		14,  4, 13,  1,  2, 15, 11,  8,  3, 10,  6, 12,  5,  9,  0,  7,
		 0, 15,  7,  4, 14,  2, 13,  1, 10,  6, 12, 11,  9,  5,  3,  8,
		 4,  1, 14,  8, 13,  6,  2, 11, 15, 12,  9,  7,  3, 10,  5,  0,
		15, 12,  8,  2,  4,  9,  1,  7,  5, 11,  3, 14, 10,  0,  6, 13
	},
	{
		15,  1,  8, 14,  6, 11,  3,  4,  9,  7,  2, 13, 12,  0,  5, 10,
		 3, 13,  4,  7, 15,  2,  8, 14, 12,  0,  1, 10,  6,  9, 11,  5,
		 0, 14,  7, 11, 10,  4, 13,  1,  5,  8, 12,  6,  9,  3,  2, 15,
		13,  8, 10,  1,  3, 15,  4,  2, 11,  6,  7, 12,  0,  5, 14,  9
	},
	{
		10,  0,  9, 14,  6,  3, 15,  5,  1, 13, 12,  7, 11,  4,  2,  8,
		13,  7,  0,  9,  3,  4,  6, 10,  2,  8,  5, 14, 12, 11, 15,  1,
		13,  6,  4,  9,  8, 15,  3,  0, 11,  1,  2, 12,  5, 10, 14,  7,
		 1, 10, 13,  0,  6,  9,  8,  7,  4, 15, 14,  3, 11,  5,  2, 12
	},
	{
		 7, 13, 14,  3,  0,  6,  9, 10,  1,  2,  8,  5, 11, 12,  4, 15,
		13,  8, 11,  5,  6, 15,  0,  3,  4,  7,  2, 12,  1, 10, 14,  9,
		10,  6,  9,  0, 12, 11,  7, 13, 15,  1,  3, 14,  5,  2,  8,  4,
		 3, 15,  0,  6, 10,  1, 13,  8,  9,  4,  5, 11, 12,  7,  2, 14
	},
	{
		 2, 12,  4,  1,  7, 10, 11,  6,  8,  5,  3, 15, 13,  0, 14,  9,
		14, 11,  2, 12,  4,  7, 13,  1,  5,  0, 15, 10,  3,  9,  8,  6,
		 4,  2,  1, 11, 10, 13,  7,  8, 15,  9, 12,  5,  6,  3,  0, 14,
		11,  8, 12,  7,  1, 14,  2, 13,  6, 15,  0,  9, 10,  4,  5,  3
	},
	{
		12,  1, 10, 15,  9,  2,  6,  8,  0, 13,  3,  4, 14,  7,  5, 11,
		10, 15,  4,  2,  7, 12,  9,  5,  6,  1, 13, 14,  0, 11,  3,  8,
		 9, 14, 15,  5,  2,  8, 12,  3,  7,  0,  4, 10,  1, 13, 11,  6,
		 4,  3,  2, 12,  9,  5, 15, 10, 11, 14,  1,  7,  6,  0,  8, 13
	},
	{
		 4, 11,  2, 14, 15,  0,  8, 13,  3, 12,  9,  7,  5, 10,  6,  1,
		13,  0, 11,  7,  4,  9,  1, 10, 14,  3,  5, 12,  2, 15,  8,  6,
		 1,  4, 11, 13, 12,  3,  7, 14, 10, 15,  6,  8,  0,  5,  9,  2,
		 6, 11, 13,  8,  1,  4, 10,  7,  9,  5,  0, 15, 14,  2,  3, 12
	},
	{
		13,  2,  8,  4,  6, 15, 11,  1, 10,  9,  3, 14,  5,  0, 12,  7,
		 1, 15, 13,  8, 10,  3,  7,  4, 12,  5,  6, 11,  0, 14,  9,  2,
		 7, 11,  4,  1,  9, 12, 14,  2,  0,  6, 10, 13, 15,  3,  5,  8,
		 2,  1, 14,  7,  4, 10,  8, 13, 15, 12,  9,  0,  3,  5,  6, 11
	}
};

static const uint8_t pbox[32] = {
	16,  7, 20, 21, 29, 12, 28, 17,  1, 15, 23, 26,  5, 18, 31, 10,
	 2,  8, 24, 14, 32, 27,  3,  9, 19, 13, 30,  6, 22, 11,  4, 25
};

static const uint32_t bits32[32] =
{
	0x80000000, 0x40000000, 0x20000000, 0x10000000,
	0x08000000, 0x04000000, 0x02000000, 0x01000000,
	0x00800000, 0x00400000, 0x00200000, 0x00100000,
	0x00080000, 0x00040000, 0x00020000, 0x00010000,
	0x00008000, 0x00004000, 0x00002000, 0x00001000,
	0x00000800, 0x00000400, 0x00000200, 0x00000100,
	0x00000080, 0x00000040, 0x00000020, 0x00000010,
	0x00000008, 0x00000004, 0x00000002, 0x00000001
};

static const uint8_t bits8[8] = { 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01 };

static const unsigned char	ascii64[] =
	 "./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

static uint8_t m_sbox[4][4096];
static uint32_t psbox[4][256];
static uint32_t ip_maskl[8][256], ip_maskr[8][256];
static uint32_t fp_maskl[8][256], fp_maskr[8][256];
static uint32_t key_perm_maskl[8][128], key_perm_maskr[8][128];
static uint32_t comp_maskl[8][128], comp_maskr[8][128];

static inline int
ascii_to_bin(char ch)
{
	signed char sch = ch;
	int retval;

	retval = sch - '.';
	if (sch >= 'A') {
		retval = sch - ('A' - 12);
		if (sch >= 'a')
			retval = sch - ('a' - 38);
	}
	retval &= 0x3f;

	return(retval);
}

/*
 * When we choose to "support" invalid salts, nevertheless disallow those
 * containing characters that would violate the passwd file format.
 */
static inline int
ascii_is_unsafe(char ch)
{
	return !ch || ch == '\n' || ch == ':';
}

void
_crypt_extended_init(void)
{
	int i, j, b, k, inbit, obit;
	uint32_t *p, *il, *ir, *fl, *fr;
	const uint32_t *bits28, *bits24;
	uint8_t inv_key_perm[64];
	uint8_t inv_comp_perm[56];
	uint8_t init_perm[64], final_perm[64];
	uint8_t u_sbox[8][64];
	uint8_t un_pbox[32];

	bits24 = (bits28 = bits32 + 4) + 4;

	/*
	 * Invert the S-boxes, reordering the input bits.
	 */
	for (i = 0; i < 8; i++)
		for (j = 0; j < 64; j++) {
			b = (j & 0x20) | ((j & 1) << 4) | ((j >> 1) & 0xf);
			u_sbox[i][j] = sbox[i][b];
		}

	/*
	 * Convert the inverted S-boxes into 4 arrays of 8 bits.
	 * Each will handle 12 bits of the S-box input.
	 */
	for (b = 0; b < 4; b++)
		for (i = 0; i < 64; i++)
			for (j = 0; j < 64; j++)
				m_sbox[b][(i << 6) | j] =
					(u_sbox[(b << 1)][i] << 4) |
					u_sbox[(b << 1) + 1][j];

	/*
	 * Set up the initial & final permutations into a useful form, and
	 * initialise the inverted key permutation.
	 */
	for (i = 0; i < 64; i++) {
		init_perm[final_perm[i] = IP[i] - 1] = i;
		inv_key_perm[i] = 255;
	}

	/*
	 * Invert the key permutation and initialise the inverted key
	 * compression permutation.
	 */
	for (i = 0; i < 56; i++) {
		inv_key_perm[key_perm[i] - 1] = i;
		inv_comp_perm[i] = 255;
	}

	/*
	 * Invert the key compression permutation.
	 */
	for (i = 0; i < 48; i++) {
		inv_comp_perm[comp_perm[i] - 1] = i;
	}

	/*
	 * Set up the OR-mask arrays for the initial and final permutations,
	 * and for the key initial and compression permutations.
	 */
	for (k = 0; k < 8; k++) {
		for (i = 0; i < 256; i++) {
			*(il = &ip_maskl[k][i]) = 0;
			*(ir = &ip_maskr[k][i]) = 0;
			*(fl = &fp_maskl[k][i]) = 0;
			*(fr = &fp_maskr[k][i]) = 0;
			for (j = 0; j < 8; j++) {
				inbit = 8 * k + j;
				if (i & bits8[j]) {
					if ((obit = init_perm[inbit]) < 32)
						*il |= bits32[obit];
					else
						*ir |= bits32[obit-32];
					if ((obit = final_perm[inbit]) < 32)
						*fl |= bits32[obit];
					else
						*fr |= bits32[obit - 32];
				}
			}
		}
		for (i = 0; i < 128; i++) {
			*(il = &key_perm_maskl[k][i]) = 0;
			*(ir = &key_perm_maskr[k][i]) = 0;
			for (j = 0; j < 7; j++) {
				inbit = 8 * k + j;
				if (i & bits8[j + 1]) {
					if ((obit = inv_key_perm[inbit]) == 255)
						continue;
					if (obit < 28)
						*il |= bits28[obit];
					else
						*ir |= bits28[obit - 28];
				}
			}
			*(il = &comp_maskl[k][i]) = 0;
			*(ir = &comp_maskr[k][i]) = 0;
			for (j = 0; j < 7; j++) {
				inbit = 7 * k + j;
				if (i & bits8[j + 1]) {
					if ((obit=inv_comp_perm[inbit]) == 255)
						continue;
					if (obit < 24)
						*il |= bits24[obit];
					else
						*ir |= bits24[obit - 24];
				}
			}
		}
	}

	/*
	 * Invert the P-box permutation, and convert into OR-masks for
	 * handling the output of the S-box arrays setup above.
	 */
	for (i = 0; i < 32; i++)
		un_pbox[pbox[i] - 1] = i;

	for (b = 0; b < 4; b++)
		for (i = 0; i < 256; i++) {
			*(p = &psbox[b][i]) = 0;
			for (j = 0; j < 8; j++) {
				if (i & bits8[j])
					*p |= bits32[un_pbox[8 * b + j]];
			}
		}
}

static void
des_init_local(struct php_crypt_extended_data *data)
{
	data->old_rawkey0 = data->old_rawkey1 = 0;
	data->saltbits = 0;
	data->old_salt = 0;

	data->initialized = 1;
}

static void
setup_salt(uint32_t salt, struct php_crypt_extended_data *data)
{
	uint32_t	obit, saltbit, saltbits;
	int	i;

	if (salt == data->old_salt)
		return;
	data->old_salt = salt;

	saltbits = 0;
	saltbit = 1;
	obit = 0x800000;
	for (i = 0; i < 24; i++) {
		if (salt & saltbit)
			saltbits |= obit;
		saltbit <<= 1;
		obit >>= 1;
	}
	data->saltbits = saltbits;
}

static int
des_setkey(const char *key, struct php_crypt_extended_data *data)
{
	uint32_t	k0, k1, rawkey0, rawkey1;
	int	shifts, round;

	rawkey0 =
		(uint32_t)(uint8_t)key[3] |
		((uint32_t)(uint8_t)key[2] << 8) |
		((uint32_t)(uint8_t)key[1] << 16) |
		((uint32_t)(uint8_t)key[0] << 24);
	rawkey1 =
		(uint32_t)(uint8_t)key[7] |
		((uint32_t)(uint8_t)key[6] << 8) |
		((uint32_t)(uint8_t)key[5] << 16) |
		((uint32_t)(uint8_t)key[4] << 24);

	if ((rawkey0 | rawkey1)
	    && rawkey0 == data->old_rawkey0
	    && rawkey1 == data->old_rawkey1) {
		/*
		 * Already setup for this key.
		 * This optimisation fails on a zero key (which is weak and
		 * has bad parity anyway) in order to simplify the starting
		 * conditions.
		 */
		return(0);
	}
	data->old_rawkey0 = rawkey0;
	data->old_rawkey1 = rawkey1;

	/*
	 *	Do key permutation and split into two 28-bit subkeys.
	 */
	k0 = key_perm_maskl[0][rawkey0 >> 25]
	   | key_perm_maskl[1][(rawkey0 >> 17) & 0x7f]
	   | key_perm_maskl[2][(rawkey0 >> 9) & 0x7f]
	   | key_perm_maskl[3][(rawkey0 >> 1) & 0x7f]
	   | key_perm_maskl[4][rawkey1 >> 25]
	   | key_perm_maskl[5][(rawkey1 >> 17) & 0x7f]
	   | key_perm_maskl[6][(rawkey1 >> 9) & 0x7f]
	   | key_perm_maskl[7][(rawkey1 >> 1) & 0x7f];
	k1 = key_perm_maskr[0][rawkey0 >> 25]
	   | key_perm_maskr[1][(rawkey0 >> 17) & 0x7f]
	   | key_perm_maskr[2][(rawkey0 >> 9) & 0x7f]
	   | key_perm_maskr[3][(rawkey0 >> 1) & 0x7f]
	   | key_perm_maskr[4][rawkey1 >> 25]
	   | key_perm_maskr[5][(rawkey1 >> 17) & 0x7f]
	   | key_perm_maskr[6][(rawkey1 >> 9) & 0x7f]
	   | key_perm_maskr[7][(rawkey1 >> 1) & 0x7f];
	/*
	 *	Rotate subkeys and do compression permutation.
	 */
	shifts = 0;
	for (round = 0; round < 16; round++) {
		uint32_t	t0, t1;

		shifts += key_shifts[round];

		t0 = (k0 << shifts) | (k0 >> (28 - shifts));
		t1 = (k1 << shifts) | (k1 >> (28 - shifts));

		data->de_keysl[15 - round] =
		data->en_keysl[round] = comp_maskl[0][(t0 >> 21) & 0x7f]
				| comp_maskl[1][(t0 >> 14) & 0x7f]
				| comp_maskl[2][(t0 >> 7) & 0x7f]
				| comp_maskl[3][t0 & 0x7f]
				| comp_maskl[4][(t1 >> 21) & 0x7f]
				| comp_maskl[5][(t1 >> 14) & 0x7f]
				| comp_maskl[6][(t1 >> 7) & 0x7f]
				| comp_maskl[7][t1 & 0x7f];

		data->de_keysr[15 - round] =
		data->en_keysr[round] = comp_maskr[0][(t0 >> 21) & 0x7f]
				| comp_maskr[1][(t0 >> 14) & 0x7f]
				| comp_maskr[2][(t0 >> 7) & 0x7f]
				| comp_maskr[3][t0 & 0x7f]
				| comp_maskr[4][(t1 >> 21) & 0x7f]
				| comp_maskr[5][(t1 >> 14) & 0x7f]
				| comp_maskr[6][(t1 >> 7) & 0x7f]
				| comp_maskr[7][t1 & 0x7f];
	}
	return(0);
}

static int
do_des(uint32_t l_in, uint32_t r_in, uint32_t *l_out, uint32_t *r_out,
	int count, struct php_crypt_extended_data *data)
{
	/*
	 *	l_in, r_in, l_out, and r_out are in pseudo-"big-endian" format.
	 */
	uint32_t	l, r, *kl, *kr, *kl1, *kr1;
	uint32_t	f, r48l, r48r, saltbits;
	int	round;

	if (count == 0) {
		return(1);
	} else if (count > 0) {
		/*
		 * Encrypting
		 */
		kl1 = data->en_keysl;
		kr1 = data->en_keysr;
	} else {
		/*
		 * Decrypting
		 */
		count = -count;
		kl1 = data->de_keysl;
		kr1 = data->de_keysr;
	}

	/*
	 *	Do initial permutation (IP).
	 */
	l = ip_maskl[0][l_in >> 24]
	  | ip_maskl[1][(l_in >> 16) & 0xff]
	  | ip_maskl[2][(l_in >> 8) & 0xff]
	  | ip_maskl[3][l_in & 0xff]
	  | ip_maskl[4][r_in >> 24]
	  | ip_maskl[5][(r_in >> 16) & 0xff]
	  | ip_maskl[6][(r_in >> 8) & 0xff]
	  | ip_maskl[7][r_in & 0xff];
	r = ip_maskr[0][l_in >> 24]
	  | ip_maskr[1][(l_in >> 16) & 0xff]
	  | ip_maskr[2][(l_in >> 8) & 0xff]
	  | ip_maskr[3][l_in & 0xff]
	  | ip_maskr[4][r_in >> 24]
	  | ip_maskr[5][(r_in >> 16) & 0xff]
	  | ip_maskr[6][(r_in >> 8) & 0xff]
	  | ip_maskr[7][r_in & 0xff];

	saltbits = data->saltbits;
	while (count--) {
		/*
		 * Do each round.
		 */
		kl = kl1;
		kr = kr1;
		round = 16;
		while (round--) {
			/*
			 * Expand R to 48 bits (simulate the E-box).
			 */
			r48l	= ((r & 0x00000001) << 23)
				| ((r & 0xf8000000) >> 9)
				| ((r & 0x1f800000) >> 11)
				| ((r & 0x01f80000) >> 13)
				| ((r & 0x001f8000) >> 15);

			r48r	= ((r & 0x0001f800) << 7)
				| ((r & 0x00001f80) << 5)
				| ((r & 0x000001f8) << 3)
				| ((r & 0x0000001f) << 1)
				| ((r & 0x80000000) >> 31);
			/*
			 * Do salting for crypt() and friends, and
			 * XOR with the permuted key.
			 */
			f = (r48l ^ r48r) & saltbits;
			r48l ^= f ^ *kl++;
			r48r ^= f ^ *kr++;
			/*
			 * Do sbox lookups (which shrink it back to 32 bits)
			 * and do the pbox permutation at the same time.
			 */
			f = psbox[0][m_sbox[0][r48l >> 12]]
			  | psbox[1][m_sbox[1][r48l & 0xfff]]
			  | psbox[2][m_sbox[2][r48r >> 12]]
			  | psbox[3][m_sbox[3][r48r & 0xfff]];
			/*
			 * Now that we've permuted things, complete f().
			 */
			f ^= l;
			l = r;
			r = f;
		}
		r = l;
		l = f;
	}
	/*
	 * Do final permutation (inverse of IP).
	 */
	*l_out	= fp_maskl[0][l >> 24]
		| fp_maskl[1][(l >> 16) & 0xff]
		| fp_maskl[2][(l >> 8) & 0xff]
		| fp_maskl[3][l & 0xff]
		| fp_maskl[4][r >> 24]
		| fp_maskl[5][(r >> 16) & 0xff]
		| fp_maskl[6][(r >> 8) & 0xff]
		| fp_maskl[7][r & 0xff];
	*r_out	= fp_maskr[0][l >> 24]
		| fp_maskr[1][(l >> 16) & 0xff]
		| fp_maskr[2][(l >> 8) & 0xff]
		| fp_maskr[3][l & 0xff]
		| fp_maskr[4][r >> 24]
		| fp_maskr[5][(r >> 16) & 0xff]
		| fp_maskr[6][(r >> 8) & 0xff]
		| fp_maskr[7][r & 0xff];
	return(0);
}

static int
des_cipher(const char *in, char *out, uint32_t salt, int count,
	struct php_crypt_extended_data *data)
{
	uint32_t	l_out = 0, r_out = 0, rawl, rawr;
	int	retval;

	setup_salt(salt, data);

	rawl =
		(uint32_t)(uint8_t)in[3] |
		((uint32_t)(uint8_t)in[2] << 8) |
		((uint32_t)(uint8_t)in[1] << 16) |
		((uint32_t)(uint8_t)in[0] << 24);
	rawr =
		(uint32_t)(uint8_t)in[7] |
		((uint32_t)(uint8_t)in[6] << 8) |
		((uint32_t)(uint8_t)in[5] << 16) |
		((uint32_t)(uint8_t)in[4] << 24);

	retval = do_des(rawl, rawr, &l_out, &r_out, count, data);

	out[0] = l_out >> 24;
	out[1] = l_out >> 16;
	out[2] = l_out >> 8;
	out[3] = l_out;
	out[4] = r_out >> 24;
	out[5] = r_out >> 16;
	out[6] = r_out >> 8;
	out[7] = r_out;

	return(retval);
}

char *
_crypt_extended_r(const unsigned char *key, const char *setting,
	struct php_crypt_extended_data *data)
{
	int		i;
	uint32_t	count, salt, l, r0, r1, keybuf[2];
	uint8_t		*p, *q;

	if (!data->initialized)
		des_init_local(data);

	/*
	 * Copy the key, shifting each character up by one bit
	 * and padding with zeros.
	 */
	q = (uint8_t *) keybuf;
	while ((size_t)(q - (uint8_t *) keybuf) < sizeof(keybuf)) {
		*q++ = *key << 1;
		if (*key)
			key++;
	}
	if (des_setkey((char *) keybuf, data))
		return(NULL);

	if (*setting == _PASSWORD_EFMT1) {
		/*
		 * "new"-style:
		 *	setting - underscore, 4 chars of count, 4 chars of salt
		 *	key - unlimited characters
		 */
		for (i = 1, count = 0; i < 5; i++) {
			int value = ascii_to_bin(setting[i]);
			if (ascii64[value] != setting[i])
				return(NULL);
			count |= value << (i - 1) * 6;
		}
		if (!count)
			return(NULL);

		for (i = 5, salt = 0; i < 9; i++) {
			int value = ascii_to_bin(setting[i]);
			if (ascii64[value] != setting[i])
				return(NULL);
			salt |= value << (i - 5) * 6;
		}

		while (*key) {
			/*
			 * Encrypt the key with itself.
			 */
			if (des_cipher((char *) keybuf, (char *) keybuf,
			    0, 1, data))
				return(NULL);
			/*
			 * And XOR with the next 8 characters of the key.
			 */
			q = (uint8_t *) keybuf;
			while ((size_t)(q - (uint8_t *) keybuf) < sizeof(keybuf) && *key)
				*q++ ^= *key++ << 1;

			if (des_setkey((char *) keybuf, data))
				return(NULL);
		}
		memcpy(data->output, setting, 9);
		data->output[9] = '\0';
		p = (uint8_t *) data->output + 9;
	} else {
		/*
		 * "old"-style:
		 *	setting - 2 chars of salt
		 *	key - up to 8 characters
		 */
		count = 25;

		if (ascii_is_unsafe(setting[0]) || ascii_is_unsafe(setting[1]))
			return(NULL);

		salt = (ascii_to_bin(setting[1]) << 6)
		     |  ascii_to_bin(setting[0]);

		data->output[0] = setting[0];
		data->output[1] = setting[1];
		p = (uint8_t *) data->output + 2;
	}
	setup_salt(salt, data);
	/*
	 * Do it.
	 */
	if (do_des(0, 0, &r0, &r1, count, data))
		return(NULL);
	/*
	 * Now encode the result...
	 */
	l = (r0 >> 8);
	*p++ = ascii64[(l >> 18) & 0x3f];
	*p++ = ascii64[(l >> 12) & 0x3f];
	*p++ = ascii64[(l >> 6) & 0x3f];
	*p++ = ascii64[l & 0x3f];

	l = (r0 << 16) | ((r1 >> 16) & 0xffff);
	*p++ = ascii64[(l >> 18) & 0x3f];
	*p++ = ascii64[(l >> 12) & 0x3f];
	*p++ = ascii64[(l >> 6) & 0x3f];
	*p++ = ascii64[l & 0x3f];

	l = r1 << 2;
	*p++ = ascii64[(l >> 12) & 0x3f];
	*p++ = ascii64[(l >> 6) & 0x3f];
	*p++ = ascii64[l & 0x3f];
	*p = 0;

	return(data->output);
}

#ifdef TEST
static char *
_crypt_extended(const char *key, const char *setting)
{
	static int initialized = 0;
	static struct php_crypt_extended_data data;

	if (!initialized) {
		_crypt_extended_init();
		initialized = 1;
		data.initialized = 0;
	}
	return _crypt_extended_r(key, setting, &data);
}

#define crypt _crypt_extended

static const struct {
	const char *hash;
	const char *pw;
} tests[] = {
/* "new"-style */
	{"_J9..CCCCXBrJUJV154M", "U*U*U*U*"},
	{"_J9..CCCCXUhOBTXzaiE", "U*U***U"},
	{"_J9..CCCC4gQ.mB/PffM", "U*U***U*"},
	{"_J9..XXXXvlzQGqpPPdk", "*U*U*U*U"},
	{"_J9..XXXXsqM/YSSP..Y", "*U*U*U*U*"},
	{"_J9..XXXXVL7qJCnku0I", "*U*U*U*U*U*U*U*U"},
	{"_J9..XXXXAj8cFbP5scI", "*U*U*U*U*U*U*U*U*"},
	{"_J9..SDizh.vll5VED9g", "ab1234567"},
	{"_J9..SDizRjWQ/zePPHc", "cr1234567"},
	{"_J9..SDizxmRI1GjnQuE", "zxyDPWgydbQjgq"},
	{"_K9..SaltNrQgIYUAeoY", "726 even"},
	{"_J9..SDSD5YGyRCr4W4c", ""},
/* "old"-style, valid salts */
	{"CCNf8Sbh3HDfQ", "U*U*U*U*"},
	{"CCX.K.MFy4Ois", "U*U***U"},
	{"CC4rMpbg9AMZ.", "U*U***U*"},
	{"XXxzOu6maQKqQ", "*U*U*U*U"},
	{"SDbsugeBiC58A", ""},
	{"./xZjzHv5vzVE", "password"},
	{"0A2hXM1rXbYgo", "password"},
	{"A9RXdR23Y.cY6", "password"},
	{"ZziFATVXHo2.6", "password"},
	{"zZDDIZ0NOlPzw", "password"},
/* "old"-style, "reasonable" invalid salts, UFC-crypt behavior expected */
	{"\001\002wyd0KZo65Jo", "password"},
	{"a_C10Dk/ExaG.", "password"},
	{"~\377.5OTsRVjwLo", "password"},
/* The below are erroneous inputs, so NULL return is expected/required */
	{"", ""}, /* no salt */
	{" ", ""}, /* setting string is too short */
	{"a:", ""}, /* unsafe character */
	{"\na", ""}, /* unsafe character */
	{"_/......", ""}, /* setting string is too short for its type */
	{"_........", ""}, /* zero iteration count */
	{"_/!......", ""}, /* invalid character in count */
	{"_/......!", ""}, /* invalid character in salt */
	{NULL}
};

int main(void)
{
	int i;

	for (i = 0; tests[i].hash; i++) {
		char *hash = crypt(tests[i].pw, tests[i].hash);
		if (!hash && strlen(tests[i].hash) < 13)
			continue; /* expected failure */
		if (!strcmp(hash, tests[i].hash))
			continue; /* expected success */
		puts("FAILED");
		return 1;
	}

	puts("PASSED");

	return 0;
}
#endif
/*
 * The crypt_blowfish homepage is:
 *
 *	http://www.openwall.com/crypt/
 *
 * This code comes from John the Ripper password cracker, with reentrant
 * and crypt(3) interfaces added, but optimizations specific to password
 * cracking removed.
 *
 * Written by Solar Designer <solar at openwall.com> in 1998-2015.
 * No copyright is claimed, and the software is hereby placed in the public
 * domain. In case this attempt to disclaim copyright and place the software
 * in the public domain is deemed null and void, then the software is
 * Copyright (c) 1998-2015 Solar Designer and it is hereby released to the
 * general public under the following terms:
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted.
 *
 * There's ABSOLUTELY NO WARRANTY, express or implied.
 *
 * It is my intent that you should be able to use this on your system,
 * as part of a software package, or anywhere else to improve security,
 * ensure compatibility, or for any other purpose. I would appreciate
 * it if you give credit where it is due and keep your modifications in
 * the public domain as well, but I don't require that in order to let
 * you place this code and any modifications you make under a license
 * of your choice.
 *
 * This implementation is fully compatible with OpenBSD's bcrypt.c for prefix
 * "$2b$", originally by Niels Provos <provos at citi.umich.edu>, and it uses
 * some of his ideas. The password hashing algorithm was designed by David
 * Mazieres <dm at lcs.mit.edu>. For information on the level of
 * compatibility for bcrypt hash prefixes other than "$2b$", please refer to
 * the comments in BF_set_key() below and to the included crypt(3) man page.
 *
 * There's a paper on the algorithm that explains its design decisions:
 *
 *	http://www.usenix.org/events/usenix99/provos.html
 *
 * Some of the tricks in BF_ROUND might be inspired by Eric Young's
 * Blowfish library (I can't be sure if I would think of something if I
 * hadn't seen his code).
 */

#include <string.h>

#include <errno.h>
#ifndef __set_errno
#define __set_errno(val) errno = (val)
#endif

/* Just to make sure the prototypes match the actual definitions */

#if defined(__i386__) || defined(__x86_64__) || defined(__alpha__) || defined(__hppa__)
#define BF_SCALE			1
#else
#define BF_SCALE			0
#endif

typedef unsigned int BF_word;
typedef signed int BF_word_signed;

/* Number of Blowfish rounds, this is also hardcoded into a few places */
#define BF_N				16

typedef BF_word BF_key[BF_N + 2];

typedef struct {
	BF_word S[4][0x100];
	BF_key P;
} BF_ctx;

/*
 * Magic IV for 64 Blowfish encryptions that we do at the end.
 * The string is "OrpheanBeholderScryDoubt" on big-endian.
 */
static BF_word BF_magic_w[6] = {
	0x4F727068, 0x65616E42, 0x65686F6C,
	0x64657253, 0x63727944, 0x6F756274
};

/*
 * P-box and S-box tables initialized with digits of Pi.
 */
static BF_ctx BF_init_state = {
	{
		{
			0xd1310ba6, 0x98dfb5ac, 0x2ffd72db, 0xd01adfb7,
			0xb8e1afed, 0x6a267e96, 0xba7c9045, 0xf12c7f99,
			0x24a19947, 0xb3916cf7, 0x0801f2e2, 0x858efc16,
			0x636920d8, 0x71574e69, 0xa458fea3, 0xf4933d7e,
			0x0d95748f, 0x728eb658, 0x718bcd58, 0x82154aee,
			0x7b54a41d, 0xc25a59b5, 0x9c30d539, 0x2af26013,
			0xc5d1b023, 0x286085f0, 0xca417918, 0xb8db38ef,
			0x8e79dcb0, 0x603a180e, 0x6c9e0e8b, 0xb01e8a3e,
			0xd71577c1, 0xbd314b27, 0x78af2fda, 0x55605c60,
			0xe65525f3, 0xaa55ab94, 0x57489862, 0x63e81440,
			0x55ca396a, 0x2aab10b6, 0xb4cc5c34, 0x1141e8ce,
			0xa15486af, 0x7c72e993, 0xb3ee1411, 0x636fbc2a,
			0x2ba9c55d, 0x741831f6, 0xce5c3e16, 0x9b87931e,
			0xafd6ba33, 0x6c24cf5c, 0x7a325381, 0x28958677,
			0x3b8f4898, 0x6b4bb9af, 0xc4bfe81b, 0x66282193,
			0x61d809cc, 0xfb21a991, 0x487cac60, 0x5dec8032,
			0xef845d5d, 0xe98575b1, 0xdc262302, 0xeb651b88,
			0x23893e81, 0xd396acc5, 0x0f6d6ff3, 0x83f44239,
			0x2e0b4482, 0xa4842004, 0x69c8f04a, 0x9e1f9b5e,
			0x21c66842, 0xf6e96c9a, 0x670c9c61, 0xabd388f0,
			0x6a51a0d2, 0xd8542f68, 0x960fa728, 0xab5133a3,
			0x6eef0b6c, 0x137a3be4, 0xba3bf050, 0x7efb2a98,
			0xa1f1651d, 0x39af0176, 0x66ca593e, 0x82430e88,
			0x8cee8619, 0x456f9fb4, 0x7d84a5c3, 0x3b8b5ebe,
			0xe06f75d8, 0x85c12073, 0x401a449f, 0x56c16aa6,
			0x4ed3aa62, 0x363f7706, 0x1bfedf72, 0x429b023d,
			0x37d0d724, 0xd00a1248, 0xdb0fead3, 0x49f1c09b,
			0x075372c9, 0x80991b7b, 0x25d479d8, 0xf6e8def7,
			0xe3fe501a, 0xb6794c3b, 0x976ce0bd, 0x04c006ba,
			0xc1a94fb6, 0x409f60c4, 0x5e5c9ec2, 0x196a2463,
			0x68fb6faf, 0x3e6c53b5, 0x1339b2eb, 0x3b52ec6f,
			0x6dfc511f, 0x9b30952c, 0xcc814544, 0xaf5ebd09,
			0xbee3d004, 0xde334afd, 0x660f2807, 0x192e4bb3,
			0xc0cba857, 0x45c8740f, 0xd20b5f39, 0xb9d3fbdb,
			0x5579c0bd, 0x1a60320a, 0xd6a100c6, 0x402c7279,
			0x679f25fe, 0xfb1fa3cc, 0x8ea5e9f8, 0xdb3222f8,
			0x3c7516df, 0xfd616b15, 0x2f501ec8, 0xad0552ab,
			0x323db5fa, 0xfd238760, 0x53317b48, 0x3e00df82,
			0x9e5c57bb, 0xca6f8ca0, 0x1a87562e, 0xdf1769db,
			0xd542a8f6, 0x287effc3, 0xac6732c6, 0x8c4f5573,
			0x695b27b0, 0xbbca58c8, 0xe1ffa35d, 0xb8f011a0,
			0x10fa3d98, 0xfd2183b8, 0x4afcb56c, 0x2dd1d35b,
			0x9a53e479, 0xb6f84565, 0xd28e49bc, 0x4bfb9790,
			0xe1ddf2da, 0xa4cb7e33, 0x62fb1341, 0xcee4c6e8,
			0xef20cada, 0x36774c01, 0xd07e9efe, 0x2bf11fb4,
			0x95dbda4d, 0xae909198, 0xeaad8e71, 0x6b93d5a0,
			0xd08ed1d0, 0xafc725e0, 0x8e3c5b2f, 0x8e7594b7,
			0x8ff6e2fb, 0xf2122b64, 0x8888b812, 0x900df01c,
			0x4fad5ea0, 0x688fc31c, 0xd1cff191, 0xb3a8c1ad,
			0x2f2f2218, 0xbe0e1777, 0xea752dfe, 0x8b021fa1,
			0xe5a0cc0f, 0xb56f74e8, 0x18acf3d6, 0xce89e299,
			0xb4a84fe0, 0xfd13e0b7, 0x7cc43b81, 0xd2ada8d9,
			0x165fa266, 0x80957705, 0x93cc7314, 0x211a1477,
			0xe6ad2065, 0x77b5fa86, 0xc75442f5, 0xfb9d35cf,
			0xebcdaf0c, 0x7b3e89a0, 0xd6411bd3, 0xae1e7e49,
			0x00250e2d, 0x2071b35e, 0x226800bb, 0x57b8e0af,
			0x2464369b, 0xf009b91e, 0x5563911d, 0x59dfa6aa,
			0x78c14389, 0xd95a537f, 0x207d5ba2, 0x02e5b9c5,
			0x83260376, 0x6295cfa9, 0x11c81968, 0x4e734a41,
			0xb3472dca, 0x7b14a94a, 0x1b510052, 0x9a532915,
			0xd60f573f, 0xbc9bc6e4, 0x2b60a476, 0x81e67400,
			0x08ba6fb5, 0x571be91f, 0xf296ec6b, 0x2a0dd915,
			0xb6636521, 0xe7b9f9b6, 0xff34052e, 0xc5855664,
			0x53b02d5d, 0xa99f8fa1, 0x08ba4799, 0x6e85076a
		}, {
			0x4b7a70e9, 0xb5b32944, 0xdb75092e, 0xc4192623,
			0xad6ea6b0, 0x49a7df7d, 0x9cee60b8, 0x8fedb266,
			0xecaa8c71, 0x699a17ff, 0x5664526c, 0xc2b19ee1,
			0x193602a5, 0x75094c29, 0xa0591340, 0xe4183a3e,
			0x3f54989a, 0x5b429d65, 0x6b8fe4d6, 0x99f73fd6,
			0xa1d29c07, 0xefe830f5, 0x4d2d38e6, 0xf0255dc1,
			0x4cdd2086, 0x8470eb26, 0x6382e9c6, 0x021ecc5e,
			0x09686b3f, 0x3ebaefc9, 0x3c971814, 0x6b6a70a1,
			0x687f3584, 0x52a0e286, 0xb79c5305, 0xaa500737,
			0x3e07841c, 0x7fdeae5c, 0x8e7d44ec, 0x5716f2b8,
			0xb03ada37, 0xf0500c0d, 0xf01c1f04, 0x0200b3ff,
			0xae0cf51a, 0x3cb574b2, 0x25837a58, 0xdc0921bd,
			0xd19113f9, 0x7ca92ff6, 0x94324773, 0x22f54701,
			0x3ae5e581, 0x37c2dadc, 0xc8b57634, 0x9af3dda7,
			0xa9446146, 0x0fd0030e, 0xecc8c73e, 0xa4751e41,
			0xe238cd99, 0x3bea0e2f, 0x3280bba1, 0x183eb331,
			0x4e548b38, 0x4f6db908, 0x6f420d03, 0xf60a04bf,
			0x2cb81290, 0x24977c79, 0x5679b072, 0xbcaf89af,
			0xde9a771f, 0xd9930810, 0xb38bae12, 0xdccf3f2e,
			0x5512721f, 0x2e6b7124, 0x501adde6, 0x9f84cd87,
			0x7a584718, 0x7408da17, 0xbc9f9abc, 0xe94b7d8c,
			0xec7aec3a, 0xdb851dfa, 0x63094366, 0xc464c3d2,
			0xef1c1847, 0x3215d908, 0xdd433b37, 0x24c2ba16,
			0x12a14d43, 0x2a65c451, 0x50940002, 0x133ae4dd,
			0x71dff89e, 0x10314e55, 0x81ac77d6, 0x5f11199b,
			0x043556f1, 0xd7a3c76b, 0x3c11183b, 0x5924a509,
			0xf28fe6ed, 0x97f1fbfa, 0x9ebabf2c, 0x1e153c6e,
			0x86e34570, 0xeae96fb1, 0x860e5e0a, 0x5a3e2ab3,
			0x771fe71c, 0x4e3d06fa, 0x2965dcb9, 0x99e71d0f,
			0x803e89d6, 0x5266c825, 0x2e4cc978, 0x9c10b36a,
			0xc6150eba, 0x94e2ea78, 0xa5fc3c53, 0x1e0a2df4,
			0xf2f74ea7, 0x361d2b3d, 0x1939260f, 0x19c27960,
			0x5223a708, 0xf71312b6, 0xebadfe6e, 0xeac31f66,
			0xe3bc4595, 0xa67bc883, 0xb17f37d1, 0x018cff28,
			0xc332ddef, 0xbe6c5aa5, 0x65582185, 0x68ab9802,
			0xeecea50f, 0xdb2f953b, 0x2aef7dad, 0x5b6e2f84,
			0x1521b628, 0x29076170, 0xecdd4775, 0x619f1510,
			0x13cca830, 0xeb61bd96, 0x0334fe1e, 0xaa0363cf,
			0xb5735c90, 0x4c70a239, 0xd59e9e0b, 0xcbaade14,
			0xeecc86bc, 0x60622ca7, 0x9cab5cab, 0xb2f3846e,
			0x648b1eaf, 0x19bdf0ca, 0xa02369b9, 0x655abb50,
			0x40685a32, 0x3c2ab4b3, 0x319ee9d5, 0xc021b8f7,
			0x9b540b19, 0x875fa099, 0x95f7997e, 0x623d7da8,
			0xf837889a, 0x97e32d77, 0x11ed935f, 0x16681281,
			0x0e358829, 0xc7e61fd6, 0x96dedfa1, 0x7858ba99,
			0x57f584a5, 0x1b227263, 0x9b83c3ff, 0x1ac24696,
			0xcdb30aeb, 0x532e3054, 0x8fd948e4, 0x6dbc3128,
			0x58ebf2ef, 0x34c6ffea, 0xfe28ed61, 0xee7c3c73,
			0x5d4a14d9, 0xe864b7e3, 0x42105d14, 0x203e13e0,
			0x45eee2b6, 0xa3aaabea, 0xdb6c4f15, 0xfacb4fd0,
			0xc742f442, 0xef6abbb5, 0x654f3b1d, 0x41cd2105,
			0xd81e799e, 0x86854dc7, 0xe44b476a, 0x3d816250,
			0xcf62a1f2, 0x5b8d2646, 0xfc8883a0, 0xc1c7b6a3,
			0x7f1524c3, 0x69cb7492, 0x47848a0b, 0x5692b285,
			0x095bbf00, 0xad19489d, 0x1462b174, 0x23820e00,
			0x58428d2a, 0x0c55f5ea, 0x1dadf43e, 0x233f7061,
			0x3372f092, 0x8d937e41, 0xd65fecf1, 0x6c223bdb,
			0x7cde3759, 0xcbee7460, 0x4085f2a7, 0xce77326e,
			0xa6078084, 0x19f8509e, 0xe8efd855, 0x61d99735,
			0xa969a7aa, 0xc50c06c2, 0x5a04abfc, 0x800bcadc,
			0x9e447a2e, 0xc3453484, 0xfdd56705, 0x0e1e9ec9,
			0xdb73dbd3, 0x105588cd, 0x675fda79, 0xe3674340,
			0xc5c43465, 0x713e38d8, 0x3d28f89e, 0xf16dff20,
			0x153e21e7, 0x8fb03d4a, 0xe6e39f2b, 0xdb83adf7
		}, {
			0xe93d5a68, 0x948140f7, 0xf64c261c, 0x94692934,
			0x411520f7, 0x7602d4f7, 0xbcf46b2e, 0xd4a20068,
			0xd4082471, 0x3320f46a, 0x43b7d4b7, 0x500061af,
			0x1e39f62e, 0x97244546, 0x14214f74, 0xbf8b8840,
			0x4d95fc1d, 0x96b591af, 0x70f4ddd3, 0x66a02f45,
			0xbfbc09ec, 0x03bd9785, 0x7fac6dd0, 0x31cb8504,
			0x96eb27b3, 0x55fd3941, 0xda2547e6, 0xabca0a9a,
			0x28507825, 0x530429f4, 0x0a2c86da, 0xe9b66dfb,
			0x68dc1462, 0xd7486900, 0x680ec0a4, 0x27a18dee,
			0x4f3ffea2, 0xe887ad8c, 0xb58ce006, 0x7af4d6b6,
			0xaace1e7c, 0xd3375fec, 0xce78a399, 0x406b2a42,
			0x20fe9e35, 0xd9f385b9, 0xee39d7ab, 0x3b124e8b,
			0x1dc9faf7, 0x4b6d1856, 0x26a36631, 0xeae397b2,
			0x3a6efa74, 0xdd5b4332, 0x6841e7f7, 0xca7820fb,
			0xfb0af54e, 0xd8feb397, 0x454056ac, 0xba489527,
			0x55533a3a, 0x20838d87, 0xfe6ba9b7, 0xd096954b,
			0x55a867bc, 0xa1159a58, 0xcca92963, 0x99e1db33,
			0xa62a4a56, 0x3f3125f9, 0x5ef47e1c, 0x9029317c,
			0xfdf8e802, 0x04272f70, 0x80bb155c, 0x05282ce3,
			0x95c11548, 0xe4c66d22, 0x48c1133f, 0xc70f86dc,
			0x07f9c9ee, 0x41041f0f, 0x404779a4, 0x5d886e17,
			0x325f51eb, 0xd59bc0d1, 0xf2bcc18f, 0x41113564,
			0x257b7834, 0x602a9c60, 0xdff8e8a3, 0x1f636c1b,
			0x0e12b4c2, 0x02e1329e, 0xaf664fd1, 0xcad18115,
			0x6b2395e0, 0x333e92e1, 0x3b240b62, 0xeebeb922,
			0x85b2a20e, 0xe6ba0d99, 0xde720c8c, 0x2da2f728,
			0xd0127845, 0x95b794fd, 0x647d0862, 0xe7ccf5f0,
			0x5449a36f, 0x877d48fa, 0xc39dfd27, 0xf33e8d1e,
			0x0a476341, 0x992eff74, 0x3a6f6eab, 0xf4f8fd37,
			0xa812dc60, 0xa1ebddf8, 0x991be14c, 0xdb6e6b0d,
			0xc67b5510, 0x6d672c37, 0x2765d43b, 0xdcd0e804,
			0xf1290dc7, 0xcc00ffa3, 0xb5390f92, 0x690fed0b,
			0x667b9ffb, 0xcedb7d9c, 0xa091cf0b, 0xd9155ea3,
			0xbb132f88, 0x515bad24, 0x7b9479bf, 0x763bd6eb,
			0x37392eb3, 0xcc115979, 0x8026e297, 0xf42e312d,
			0x6842ada7, 0xc66a2b3b, 0x12754ccc, 0x782ef11c,
			0x6a124237, 0xb79251e7, 0x06a1bbe6, 0x4bfb6350,
			0x1a6b1018, 0x11caedfa, 0x3d25bdd8, 0xe2e1c3c9,
			0x44421659, 0x0a121386, 0xd90cec6e, 0xd5abea2a,
			0x64af674e, 0xda86a85f, 0xbebfe988, 0x64e4c3fe,
			0x9dbc8057, 0xf0f7c086, 0x60787bf8, 0x6003604d,
			0xd1fd8346, 0xf6381fb0, 0x7745ae04, 0xd736fccc,
			0x83426b33, 0xf01eab71, 0xb0804187, 0x3c005e5f,
			0x77a057be, 0xbde8ae24, 0x55464299, 0xbf582e61,
			0x4e58f48f, 0xf2ddfda2, 0xf474ef38, 0x8789bdc2,
			0x5366f9c3, 0xc8b38e74, 0xb475f255, 0x46fcd9b9,
			0x7aeb2661, 0x8b1ddf84, 0x846a0e79, 0x915f95e2,
			0x466e598e, 0x20b45770, 0x8cd55591, 0xc902de4c,
			0xb90bace1, 0xbb8205d0, 0x11a86248, 0x7574a99e,
			0xb77f19b6, 0xe0a9dc09, 0x662d09a1, 0xc4324633,
			0xe85a1f02, 0x09f0be8c, 0x4a99a025, 0x1d6efe10,
			0x1ab93d1d, 0x0ba5a4df, 0xa186f20f, 0x2868f169,
			0xdcb7da83, 0x573906fe, 0xa1e2ce9b, 0x4fcd7f52,
			0x50115e01, 0xa70683fa, 0xa002b5c4, 0x0de6d027,
			0x9af88c27, 0x773f8641, 0xc3604c06, 0x61a806b5,
			0xf0177a28, 0xc0f586e0, 0x006058aa, 0x30dc7d62,
			0x11e69ed7, 0x2338ea63, 0x53c2dd94, 0xc2c21634,
			0xbbcbee56, 0x90bcb6de, 0xebfc7da1, 0xce591d76,
			0x6f05e409, 0x4b7c0188, 0x39720a3d, 0x7c927c24,
			0x86e3725f, 0x724d9db9, 0x1ac15bb4, 0xd39eb8fc,
			0xed545578, 0x08fca5b5, 0xd83d7cd3, 0x4dad0fc4,
			0x1e50ef5e, 0xb161e6f8, 0xa28514d9, 0x6c51133c,
			0x6fd5c7e7, 0x56e14ec4, 0x362abfce, 0xddc6c837,
			0xd79a3234, 0x92638212, 0x670efa8e, 0x406000e0
		}, {
			0x3a39ce37, 0xd3faf5cf, 0xabc27737, 0x5ac52d1b,
			0x5cb0679e, 0x4fa33742, 0xd3822740, 0x99bc9bbe,
			0xd5118e9d, 0xbf0f7315, 0xd62d1c7e, 0xc700c47b,
			0xb78c1b6b, 0x21a19045, 0xb26eb1be, 0x6a366eb4,
			0x5748ab2f, 0xbc946e79, 0xc6a376d2, 0x6549c2c8,
			0x530ff8ee, 0x468dde7d, 0xd5730a1d, 0x4cd04dc6,
			0x2939bbdb, 0xa9ba4650, 0xac9526e8, 0xbe5ee304,
			0xa1fad5f0, 0x6a2d519a, 0x63ef8ce2, 0x9a86ee22,
			0xc089c2b8, 0x43242ef6, 0xa51e03aa, 0x9cf2d0a4,
			0x83c061ba, 0x9be96a4d, 0x8fe51550, 0xba645bd6,
			0x2826a2f9, 0xa73a3ae1, 0x4ba99586, 0xef5562e9,
			0xc72fefd3, 0xf752f7da, 0x3f046f69, 0x77fa0a59,
			0x80e4a915, 0x87b08601, 0x9b09e6ad, 0x3b3ee593,
			0xe990fd5a, 0x9e34d797, 0x2cf0b7d9, 0x022b8b51,
			0x96d5ac3a, 0x017da67d, 0xd1cf3ed6, 0x7c7d2d28,
			0x1f9f25cf, 0xadf2b89b, 0x5ad6b472, 0x5a88f54c,
			0xe029ac71, 0xe019a5e6, 0x47b0acfd, 0xed93fa9b,
			0xe8d3c48d, 0x283b57cc, 0xf8d56629, 0x79132e28,
			0x785f0191, 0xed756055, 0xf7960e44, 0xe3d35e8c,
			0x15056dd4, 0x88f46dba, 0x03a16125, 0x0564f0bd,
			0xc3eb9e15, 0x3c9057a2, 0x97271aec, 0xa93a072a,
			0x1b3f6d9b, 0x1e6321f5, 0xf59c66fb, 0x26dcf319,
			0x7533d928, 0xb155fdf5, 0x03563482, 0x8aba3cbb,
			0x28517711, 0xc20ad9f8, 0xabcc5167, 0xccad925f,
			0x4de81751, 0x3830dc8e, 0x379d5862, 0x9320f991,
			0xea7a90c2, 0xfb3e7bce, 0x5121ce64, 0x774fbe32,
			0xa8b6e37e, 0xc3293d46, 0x48de5369, 0x6413e680,
			0xa2ae0810, 0xdd6db224, 0x69852dfd, 0x09072166,
			0xb39a460a, 0x6445c0dd, 0x586cdecf, 0x1c20c8ae,
			0x5bbef7dd, 0x1b588d40, 0xccd2017f, 0x6bb4e3bb,
			0xdda26a7e, 0x3a59ff45, 0x3e350a44, 0xbcb4cdd5,
			0x72eacea8, 0xfa6484bb, 0x8d6612ae, 0xbf3c6f47,
			0xd29be463, 0x542f5d9e, 0xaec2771b, 0xf64e6370,
			0x740e0d8d, 0xe75b1357, 0xf8721671, 0xaf537d5d,
			0x4040cb08, 0x4eb4e2cc, 0x34d2466a, 0x0115af84,
			0xe1b00428, 0x95983a1d, 0x06b89fb4, 0xce6ea048,
			0x6f3f3b82, 0x3520ab82, 0x011a1d4b, 0x277227f8,
			0x611560b1, 0xe7933fdc, 0xbb3a792b, 0x344525bd,
			0xa08839e1, 0x51ce794b, 0x2f32c9b7, 0xa01fbac9,
			0xe01cc87e, 0xbcc7d1f6, 0xcf0111c3, 0xa1e8aac7,
			0x1a908749, 0xd44fbd9a, 0xd0dadecb, 0xd50ada38,
			0x0339c32a, 0xc6913667, 0x8df9317c, 0xe0b12b4f,
			0xf79e59b7, 0x43f5bb3a, 0xf2d519ff, 0x27d9459c,
			0xbf97222c, 0x15e6fc2a, 0x0f91fc71, 0x9b941525,
			0xfae59361, 0xceb69ceb, 0xc2a86459, 0x12baa8d1,
			0xb6c1075e, 0xe3056a0c, 0x10d25065, 0xcb03a442,
			0xe0ec6e0e, 0x1698db3b, 0x4c98a0be, 0x3278e964,
			0x9f1f9532, 0xe0d392df, 0xd3a0342b, 0x8971f21e,
			0x1b0a7441, 0x4ba3348c, 0xc5be7120, 0xc37632d8,
			0xdf359f8d, 0x9b992f2e, 0xe60b6f47, 0x0fe3f11d,
			0xe54cda54, 0x1edad891, 0xce6279cf, 0xcd3e7e6f,
			0x1618b166, 0xfd2c1d05, 0x848fd2c5, 0xf6fb2299,
			0xf523f357, 0xa6327623, 0x93a83531, 0x56cccd02,
			0xacf08162, 0x5a75ebb5, 0x6e163697, 0x88d273cc,
			0xde966292, 0x81b949d0, 0x4c50901b, 0x71c65614,
			0xe6c6c7bd, 0x327a140a, 0x45e1d006, 0xc3f27b9a,
			0xc9aa53fd, 0x62a80f00, 0xbb25bfe2, 0x35bdd2f6,
			0x71126905, 0xb2040222, 0xb6cbcf7c, 0xcd769c2b,
			0x53113ec0, 0x1640e3d3, 0x38abbd60, 0x2547adf0,
			0xba38209c, 0xf746ce76, 0x77afa1c5, 0x20756060,
			0x85cbfe4e, 0x8ae88dd8, 0x7aaaf9b0, 0x4cf9aa7e,
			0x1948c25c, 0x02fb8a8c, 0x01c36ae4, 0xd6ebe1f9,
			0x90d4f869, 0xa65cdea0, 0x3f09252d, 0xc208e69f,
			0xb74e6132, 0xce77e25b, 0x578fdfe3, 0x3ac372e6
		}
	}, {
		0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344,
		0xa4093822, 0x299f31d0, 0x082efa98, 0xec4e6c89,
		0x452821e6, 0x38d01377, 0xbe5466cf, 0x34e90c6c,
		0xc0ac29b7, 0xc97c50dd, 0x3f84d5b5, 0xb5470917,
		0x9216d5d9, 0x8979fb1b
	}
};

static const unsigned char BF_itoa64[64 + 1] =
	"./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

static const unsigned char BF_atoi64[0x60] = {
	64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 0, 1,
	54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 64, 64, 64, 64, 64,
	64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
	17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 64, 64, 64, 64, 64,
	64, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42,
	43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 64, 64, 64, 64, 64
};

#define BF_safe_atoi64(dst, src) \
{ \
	tmp = (unsigned char)(src); \
	if ((unsigned int)(tmp -= 0x20) >= 0x60) return -1; \
	tmp = BF_atoi64[tmp]; \
	if (tmp > 63) return -1; \
	(dst) = tmp; \
}

static int BF_decode(BF_word *dst, const char *src, int size)
{
	unsigned char *dptr = (unsigned char *)dst;
	unsigned char *end = dptr + size;
	const unsigned char *sptr = (const unsigned char *)src;
	unsigned int tmp, c1, c2, c3, c4;

	do {
		BF_safe_atoi64(c1, *sptr++);
		BF_safe_atoi64(c2, *sptr++);
		*dptr++ = (c1 << 2) | ((c2 & 0x30) >> 4);
		if (dptr >= end) break;

		BF_safe_atoi64(c3, *sptr++);
		*dptr++ = ((c2 & 0x0F) << 4) | ((c3 & 0x3C) >> 2);
		if (dptr >= end) break;

		BF_safe_atoi64(c4, *sptr++);
		*dptr++ = ((c3 & 0x03) << 6) | c4;
	} while (dptr < end);

	return 0;
}

static void BF_encode(char *dst, const BF_word *src, int size)
{
	const unsigned char *sptr = (const unsigned char *)src;
	const unsigned char *end = sptr + size;
	unsigned char *dptr = (unsigned char *)dst;
	unsigned int c1, c2;

	do {
		c1 = *sptr++;
		*dptr++ = BF_itoa64[c1 >> 2];
		c1 = (c1 & 0x03) << 4;
		if (sptr >= end) {
			*dptr++ = BF_itoa64[c1];
			break;
		}

		c2 = *sptr++;
		c1 |= c2 >> 4;
		*dptr++ = BF_itoa64[c1];
		c1 = (c2 & 0x0f) << 2;
		if (sptr >= end) {
			*dptr++ = BF_itoa64[c1];
			break;
		}

		c2 = *sptr++;
		c1 |= c2 >> 6;
		*dptr++ = BF_itoa64[c1];
		*dptr++ = BF_itoa64[c2 & 0x3f];
	} while (sptr < end);
}

static void BF_swap(BF_word *x, int count)
{
	static int endianness_check = 1;
	char *is_little_endian = (char *)&endianness_check;
	BF_word tmp;

	if (*is_little_endian)
	do {
		tmp = *x;
		tmp = (tmp << 16) | (tmp >> 16);
		*x++ = ((tmp & 0x00FF00FF) << 8) | ((tmp >> 8) & 0x00FF00FF);
	} while (--count);
}

#if BF_SCALE
/* Architectures which can shift addresses left by 2 bits with no extra cost */
#define BF_ROUND(L, R, N) \
	tmp1 = L & 0xFF; \
	tmp2 = L >> 8; \
	tmp2 &= 0xFF; \
	tmp3 = L >> 16; \
	tmp3 &= 0xFF; \
	tmp4 = L >> 24; \
	tmp1 = data.ctx.S[3][tmp1]; \
	tmp2 = data.ctx.S[2][tmp2]; \
	tmp3 = data.ctx.S[1][tmp3]; \
	tmp3 += data.ctx.S[0][tmp4]; \
	tmp3 ^= tmp2; \
	R ^= data.ctx.P[N + 1]; \
	tmp3 += tmp1; \
	R ^= tmp3;
#else
/* Architectures with no complicated addressing modes supported */
#define BF_INDEX(S, i) \
	(*((BF_word *)(((unsigned char *)S) + (i))))
#define BF_ROUND(L, R, N) \
	tmp1 = L & 0xFF; \
	tmp1 <<= 2; \
	tmp2 = L >> 6; \
	tmp2 &= 0x3FC; \
	tmp3 = L >> 14; \
	tmp3 &= 0x3FC; \
	tmp4 = L >> 22; \
	tmp4 &= 0x3FC; \
	tmp1 = BF_INDEX(data.ctx.S[3], tmp1); \
	tmp2 = BF_INDEX(data.ctx.S[2], tmp2); \
	tmp3 = BF_INDEX(data.ctx.S[1], tmp3); \
	tmp3 += BF_INDEX(data.ctx.S[0], tmp4); \
	tmp3 ^= tmp2; \
	R ^= data.ctx.P[N + 1]; \
	tmp3 += tmp1; \
	R ^= tmp3;
#endif

/*
 * Encrypt one block, BF_N is hardcoded here.
 */
#define BF_ENCRYPT \
	L ^= data.ctx.P[0]; \
	BF_ROUND(L, R, 0); \
	BF_ROUND(R, L, 1); \
	BF_ROUND(L, R, 2); \
	BF_ROUND(R, L, 3); \
	BF_ROUND(L, R, 4); \
	BF_ROUND(R, L, 5); \
	BF_ROUND(L, R, 6); \
	BF_ROUND(R, L, 7); \
	BF_ROUND(L, R, 8); \
	BF_ROUND(R, L, 9); \
	BF_ROUND(L, R, 10); \
	BF_ROUND(R, L, 11); \
	BF_ROUND(L, R, 12); \
	BF_ROUND(R, L, 13); \
	BF_ROUND(L, R, 14); \
	BF_ROUND(R, L, 15); \
	tmp4 = R; \
	R = L; \
	L = tmp4 ^ data.ctx.P[BF_N + 1];

#define BF_body() \
	L = R = 0; \
	ptr = data.ctx.P; \
	do { \
		ptr += 2; \
		BF_ENCRYPT; \
		*(ptr - 2) = L; \
		*(ptr - 1) = R; \
	} while (ptr < &data.ctx.P[BF_N + 2]); \
\
	ptr = data.ctx.S[0]; \
	do { \
		ptr += 2; \
		BF_ENCRYPT; \
		*(ptr - 2) = L; \
		*(ptr - 1) = R; \
	} while (ptr < &data.ctx.S[3][0xFF]);

static void BF_set_key(const char *key, BF_key expanded, BF_key initial,
    unsigned char flags)
{
	const char *ptr = key;
	unsigned int bug, i, j;
	BF_word safety, sign, diff, tmp[2];

/*
 * There was a sign extension bug in older revisions of this function. While
 * we would have liked to simply fix the bug and move on, we have to provide
 * a backwards compatibility feature (essentially the bug) for some systems and
 * a safety measure for some others. The latter is needed because for certain
 * multiple inputs to the buggy algorithm there exist easily found inputs to
 * the correct algorithm that produce the same hash. Thus, we optionally
 * deviate from the correct algorithm just enough to avoid such collisions.
 * While the bug itself affected the majority of passwords containing
 * characters with the 8th bit set (although only a percentage of those in a
 * collision-producing way), the anti-collision safety measure affects
 * only a subset of passwords containing the '\xff' character (not even all of
 * those passwords, just some of them). This character is not found in valid
 * UTF-8 sequences and is rarely used in popular 8-bit character encodings.
 * Thus, the safety measure is unlikely to cause much annoyance, and is a
 * reasonable tradeoff to use when authenticating against existing hashes that
 * are not reliably known to have been computed with the correct algorithm.
 *
 * We use an approach that tries to minimize side-channel leaks of password
 * information - that is, we mostly use fixed-cost bitwise operations instead
 * of branches or table lookups. (One conditional branch based on password
 * length remains. It is not part of the bug aftermath, though, and is
 * difficult and possibly unreasonable to avoid given the use of C strings by
 * the caller, which results in similar timing leaks anyway.)
 *
 * For actual implementation, we set an array index in the variable "bug"
 * (0 means no bug, 1 means sign extension bug emulation) and a flag in the
 * variable "safety" (bit 16 is set when the safety measure is requested).
 * Valid combinations of settings are:
 *
 * Prefix "$2a$": bug = 0, safety = 0x10000
 * Prefix "$2b$": bug = 0, safety = 0
 * Prefix "$2x$": bug = 1, safety = 0
 * Prefix "$2y$": bug = 0, safety = 0
 */
	bug = (unsigned int)flags & 1;
	safety = ((BF_word)flags & 2) << 15;

	sign = diff = 0;

	for (i = 0; i < BF_N + 2; i++) {
		tmp[0] = tmp[1] = 0;
		for (j = 0; j < 4; j++) {
			tmp[0] <<= 8;
			tmp[0] |= (unsigned char)*ptr; /* correct */
			tmp[1] <<= 8;
			tmp[1] |= (BF_word_signed)(signed char)*ptr; /* bug */
/*
 * Sign extension in the first char has no effect - nothing to overwrite yet,
 * and those extra 24 bits will be fully shifted out of the 32-bit word. For
 * chars 2, 3, 4 in each four-char block, we set bit 7 of "sign" if sign
 * extension in tmp[1] occurs. Once this flag is set, it remains set.
 */
			if (j)
				sign |= tmp[1] & 0x80;
			if (!*ptr)
				ptr = key;
			else
				ptr++;
		}
		diff |= tmp[0] ^ tmp[1]; /* Non-zero on any differences */

		expanded[i] = tmp[bug];
		initial[i] = BF_init_state.P[i] ^ tmp[bug];
	}

/*
 * At this point, "diff" is zero iff the correct and buggy algorithms produced
 * exactly the same result. If so and if "sign" is non-zero, which indicates
 * that there was a non-benign sign extension, this means that we have a
 * collision between the correctly computed hash for this password and a set of
 * passwords that could be supplied to the buggy algorithm. Our safety measure
 * is meant to protect from such many-buggy to one-correct collisions, by
 * deviating from the correct algorithm in such cases. Let's check for this.
 */
	diff |= diff >> 16; /* still zero iff exact match */
	diff &= 0xffff; /* ditto */
	diff += 0xffff; /* bit 16 set iff "diff" was non-zero (on non-match) */
	sign <<= 9; /* move the non-benign sign extension flag to bit 16 */
	sign &= ~diff & safety; /* action needed? */

/*
 * If we have determined that we need to deviate from the correct algorithm,
 * flip bit 16 in initial expanded key. (The choice of 16 is arbitrary, but
 * let's stick to it now. It came out of the approach we used above, and it's
 * not any worse than any other choice we could make.)
 *
 * It is crucial that we don't do the same to the expanded key used in the main
 * Eksblowfish loop. By doing it to only one of these two, we deviate from a
 * state that could be directly specified by a password to the buggy algorithm
 * (and to the fully correct one as well, but that's a side-effect).
 */
	initial[0] ^= sign;
}

static const unsigned char flags_by_subtype[26] =
	{2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 4, 0};

static char *BF_crypt(const char *key, const char *setting,
	char *output, int size,
	BF_word min)
{
	struct {
		BF_ctx ctx;
		BF_key expanded_key;
		union {
			BF_word salt[4];
			BF_word output[6];
		} binary;
	} data;
	BF_word L, R;
	BF_word tmp1, tmp2, tmp3, tmp4;
	BF_word *ptr;
	BF_word count;
	int i;

	if (size < 7 + 22 + 31 + 1) {
		__set_errno(ERANGE);
		return NULL;
	}

	if (setting[0] != '$' ||
	    setting[1] != '2' ||
	    setting[2] < 'a' || setting[2] > 'z' ||
	    !flags_by_subtype[(unsigned int)(unsigned char)setting[2] - 'a'] ||
	    setting[3] != '$' ||
	    setting[4] < '0' || setting[4] > '3' ||
	    setting[5] < '0' || setting[5] > '9' ||
	    (setting[4] == '3' && setting[5] > '1') ||
	    setting[6] != '$') {
		__set_errno(EINVAL);
		return NULL;
	}

	count = (BF_word)1 << ((setting[4] - '0') * 10 + (setting[5] - '0'));
	if (count < min || BF_decode(data.binary.salt, &setting[7], 16)) {
		__set_errno(EINVAL);
		return NULL;
	}
	BF_swap(data.binary.salt, 4);

	BF_set_key(key, data.expanded_key, data.ctx.P,
	    flags_by_subtype[(unsigned int)(unsigned char)setting[2] - 'a']);

	memcpy(data.ctx.S, BF_init_state.S, sizeof(data.ctx.S));

	L = R = 0;
	for (i = 0; i < BF_N + 2; i += 2) {
		L ^= data.binary.salt[i & 2];
		R ^= data.binary.salt[(i & 2) + 1];
		BF_ENCRYPT;
		data.ctx.P[i] = L;
		data.ctx.P[i + 1] = R;
	}

	ptr = data.ctx.S[0];
	do {
		ptr += 4;
		L ^= data.binary.salt[(BF_N + 2) & 3];
		R ^= data.binary.salt[(BF_N + 3) & 3];
		BF_ENCRYPT;
		*(ptr - 4) = L;
		*(ptr - 3) = R;

		L ^= data.binary.salt[(BF_N + 4) & 3];
		R ^= data.binary.salt[(BF_N + 5) & 3];
		BF_ENCRYPT;
		*(ptr - 2) = L;
		*(ptr - 1) = R;
	} while (ptr < &data.ctx.S[3][0xFF]);

	do {
		int done;

		for (i = 0; i < BF_N + 2; i += 2) {
			data.ctx.P[i] ^= data.expanded_key[i];
			data.ctx.P[i + 1] ^= data.expanded_key[i + 1];
		}

		done = 0;
		do {
			BF_body();
			if (done)
				break;
			done = 1;

			tmp1 = data.binary.salt[0];
			tmp2 = data.binary.salt[1];
			tmp3 = data.binary.salt[2];
			tmp4 = data.binary.salt[3];
			for (i = 0; i < BF_N; i += 4) {
				data.ctx.P[i] ^= tmp1;
				data.ctx.P[i + 1] ^= tmp2;
				data.ctx.P[i + 2] ^= tmp3;
				data.ctx.P[i + 3] ^= tmp4;
			}
			data.ctx.P[16] ^= tmp1;
			data.ctx.P[17] ^= tmp2;
		} while (1);
	} while (--count);

	for (i = 0; i < 6; i += 2) {
		L = BF_magic_w[i];
		R = BF_magic_w[i + 1];

		count = 64;
		do {
			BF_ENCRYPT;
		} while (--count);

		data.binary.output[i] = L;
		data.binary.output[i + 1] = R;
	}

	memcpy(output, setting, 7 + 22 - 1);
	output[7 + 22 - 1] = BF_itoa64[(int)
		BF_atoi64[(int)setting[7 + 22 - 1] - 0x20] & 0x30];

/* This has to be bug-compatible with the original implementation, so
 * only encode 23 of the 24 bytes. :-) */
	BF_swap(data.binary.output, 6);
	BF_encode(&output[7 + 22], data.binary.output, 23);
	output[7 + 22 + 31] = '\0';

	return output;
}

static int _crypt_output_magic(const char *setting, char *output, int size)
{
	if (size < 3)
		return -1;

	output[0] = '*';
	output[1] = '0';
	output[2] = '\0';

	if (setting[0] == '*' && setting[1] == '0')
		output[1] = '1';

	return 0;
}

/*
 * Please preserve the runtime self-test. It serves two purposes at once:
 *
 * 1. We really can't afford the risk of producing incompatible hashes e.g.
 * when there's something like gcc bug 26587 again, whereas an application or
 * library integrating this code might not also integrate our external tests or
 * it might not run them after every build. Even if it does, the miscompile
 * might only occur on the production build, but not on a testing build (such
 * as because of different optimization settings). It is painful to recover
 * from incorrectly-computed hashes - merely fixing whatever broke is not
 * enough. Thus, a proactive measure like this self-test is needed.
 *
 * 2. We don't want to leave sensitive data from our actual password hash
 * computation on the stack or in registers. Previous revisions of the code
 * would do explicit cleanups, but simply running the self-test after hash
 * computation is more reliable.
 *
 * The performance cost of this quick self-test is around 0.6% at the "$2a$08"
 * setting.
 */
char *php_crypt_blowfish_rn(const char *key, const char *setting,
	char *output, int size)
{
	const char *test_key = "8b \xd0\xc1\xd2\xcf\xcc\xd8";
	const char *test_setting = "$2a$00$abcdefghijklmnopqrstuu";
	static const char * const test_hashes[2] =
		{"i1D709vfamulimlGcq0qq3UvuUasvEa\0\x55", /* 'a', 'b', 'y' */
		"VUrPmXD6q/nVSSp7pNDhCR9071IfIRe\0\x55"}; /* 'x' */
	const char *test_hash = test_hashes[0];
	char *retval;
	const char *p;
	int save_errno, ok;
	struct {
		char s[7 + 22 + 1];
		char o[7 + 22 + 31 + 1 + 1 + 1];
	} buf;

/* Hash the supplied password */
	_crypt_output_magic(setting, output, size);
	retval = BF_crypt(key, setting, output, size, 16);
	save_errno = errno;

/*
 * Do a quick self-test. It is important that we make both calls to BF_crypt()
 * from the same scope such that they likely use the same stack locations,
 * which makes the second call overwrite the first call's sensitive data on the
 * stack and makes it more likely that any alignment related issues would be
 * detected by the self-test.
 */
	memcpy(buf.s, test_setting, sizeof(buf.s));
	if (retval) {
		unsigned int flags = flags_by_subtype[
		    (unsigned int)(unsigned char)setting[2] - 'a'];
		test_hash = test_hashes[flags & 1];
		buf.s[2] = setting[2];
	}
	memset(buf.o, 0x55, sizeof(buf.o));
	buf.o[sizeof(buf.o) - 1] = 0;
	p = BF_crypt(test_key, buf.s, buf.o, sizeof(buf.o) - (1 + 1), 1);

	ok = (p == buf.o &&
	    !memcmp(p, buf.s, 7 + 22) &&
	    !memcmp(p + (7 + 22), test_hash, 31 + 1 + 1 + 1));

	{
		const char *k = "\xff\xa3" "34" "\xff\xff\xff\xa3" "345";
		BF_key ae, ai, ye, yi;
		BF_set_key(k, ae, ai, 2); /* $2a$ */
		BF_set_key(k, ye, yi, 4); /* $2y$ */
		ai[0] ^= 0x10000; /* undo the safety (for comparison) */
		ok = ok && ai[0] == 0xdb9c59bc && ye[17] == 0x33343500 &&
		    !memcmp(ae, ye, sizeof(ae)) &&
		    !memcmp(ai, yi, sizeof(ai));
	}

	__set_errno(save_errno);
	if (ok)
		return retval;

/* Should not happen */
	_crypt_output_magic(setting, output, size);
	__set_errno(EINVAL); /* pretend we don't support this hash type */
	return NULL;
}

#define fillbuf ptn_sha256_fillbuf
#define b64t ptn_sha256_b64t
#define K ptn_sha256_K
/* SHA256-based Unix crypt implementation.
   Released into the Public Domain by Ulrich Drepper <drepper@redhat.com>.  */
/* Windows VC++ port by Pierre Joye <pierre@php.net> */


#include <errno.h>
#include <limits.h>

#ifdef PHP_WIN32
# define __alignof__ __alignof
#else
# ifndef HAVE_ALIGNOF
#  include <stddef.h>
#  define __alignof__(type) offsetof (struct { char c; type member;}, member)
# endif
#endif

#include <stdio.h>
#include <stdlib.h>

#ifdef PHP_WIN32
# include <string.h>
#else
# include <sys/param.h>
# include <sys/types.h>
# include <string.h>
#endif

char * __php_stpncpy(char *dst, const char *src, size_t len)
{
	size_t n = strlen(src);
	if (n > len) {
		n = len;
	}
	return strncpy(dst, src, len) + n;
}

#ifndef MIN
# define MIN(a, b) (((a) < (b)) ? (a) : (b))
#endif
#ifndef MAX
# define MAX(a, b) (((a) > (b)) ? (a) : (b))
#endif

/* Structure to save state of computation between the single steps.  */
struct sha256_ctx {
	uint32_t H[8];

	uint32_t total[2];
	uint32_t buflen;
	char buffer[128]; /* NB: always correctly aligned for uint32_t.  */
};

#if defined(PHP_WIN32) || (!defined(WORDS_BIGENDIAN))
# define SWAP(n) \
    (((n) << 24) | (((n) & 0xff00) << 8) | (((n) >> 8) & 0xff00) | ((n) >> 24))
#else
# define SWAP(n) (n)
#endif

/* This array contains the bytes used to pad the buffer to the next
   64-byte boundary.  (FIPS 180-2:5.1.1)  */
static const unsigned char fillbuf[64] = { 0x80, 0 /* , 0, 0, ...  */ };


/* Constants for SHA256 from FIPS 180-2:4.2.2.  */
static const uint32_t K[64] = {
	0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
	0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
	0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
	0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
	0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
	0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
	0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
	0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
	0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
	0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
	0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
	0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
	0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
	0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
	0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
	0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
};


/* Process LEN bytes of BUFFER, accumulating context into CTX.
   It is assumed that LEN % 64 == 0.  */
static void sha256_process_block (const void *buffer, size_t len, struct sha256_ctx *ctx) {
	const uint32_t *words = buffer;
	size_t nwords = len / sizeof (uint32_t);
	unsigned int t;

	uint32_t a = ctx->H[0];
	uint32_t b = ctx->H[1];
	uint32_t c = ctx->H[2];
	uint32_t d = ctx->H[3];
	uint32_t e = ctx->H[4];
	uint32_t f = ctx->H[5];
	uint32_t g = ctx->H[6];
	uint32_t h = ctx->H[7];

	/* First increment the byte count.  FIPS 180-2 specifies the possible
	 length of the file up to 2^64 bits.  Here we only compute the
	 number of bytes.  Do a double word increment.  */
	ctx->total[0] += (uint32_t)len;
	if (ctx->total[0] < len) {
		++ctx->total[1];
	}

	/* Process all bytes in the buffer with 64 bytes in each round of
	 the loop.  */
	while (nwords > 0) {
		uint32_t W[64];
		uint32_t a_save = a;
		uint32_t b_save = b;
		uint32_t c_save = c;
		uint32_t d_save = d;
		uint32_t e_save = e;
		uint32_t f_save = f;
		uint32_t g_save = g;
		uint32_t h_save = h;

	/* Operators defined in FIPS 180-2:4.1.2.  */
#define Ch(x, y, z) ((x & y) ^ (~x & z))
#define Maj(x, y, z) ((x & y) ^ (x & z) ^ (y & z))
#define S0(x) (CYCLIC (x, 2) ^ CYCLIC (x, 13) ^ CYCLIC (x, 22))
#define S1(x) (CYCLIC (x, 6) ^ CYCLIC (x, 11) ^ CYCLIC (x, 25))
#define R0(x) (CYCLIC (x, 7) ^ CYCLIC (x, 18) ^ (x >> 3))
#define R1(x) (CYCLIC (x, 17) ^ CYCLIC (x, 19) ^ (x >> 10))

	/* It is unfortunate that C does not provide an operator for
	cyclic rotation.  Hope the C compiler is smart enough.  */
#define CYCLIC(w, s) ((w >> s) | (w << (32 - s)))

		/* Compute the message schedule according to FIPS 180-2:6.2.2 step 2.  */
		for (t = 0; t < 16; ++t) {
			W[t] = SWAP (*words);
			++words;
		}
		for (t = 16; t < 64; ++t)
			W[t] = R1 (W[t - 2]) + W[t - 7] + R0 (W[t - 15]) + W[t - 16];

		/* The actual computation according to FIPS 180-2:6.2.2 step 3.  */
		for (t = 0; t < 64; ++t) {
			uint32_t T1 = h + S1 (e) + Ch (e, f, g) + K[t] + W[t];
			uint32_t T2 = S0 (a) + Maj (a, b, c);
			h = g;
			g = f;
			f = e;
			e = d + T1;
			d = c;
			c = b;
			b = a;
			a = T1 + T2;
		}

		/* Add the starting values of the context according to FIPS 180-2:6.2.2
		step 4.  */
		a += a_save;
		b += b_save;
		c += c_save;
		d += d_save;
		e += e_save;
		f += f_save;
		g += g_save;
		h += h_save;

		/* Prepare for the next round.  */
		nwords -= 16;
	}

	/* Put checksum in context given as argument.  */
	ctx->H[0] = a;
	ctx->H[1] = b;
	ctx->H[2] = c;
	ctx->H[3] = d;
	ctx->H[4] = e;
	ctx->H[5] = f;
	ctx->H[6] = g;
	ctx->H[7] = h;
}


/* Initialize structure containing state of computation.
   (FIPS 180-2:5.3.2)  */
static void sha256_init_ctx(struct sha256_ctx *ctx) {
	ctx->H[0] = 0x6a09e667;
	ctx->H[1] = 0xbb67ae85;
	ctx->H[2] = 0x3c6ef372;
	ctx->H[3] = 0xa54ff53a;
	ctx->H[4] = 0x510e527f;
	ctx->H[5] = 0x9b05688c;
	ctx->H[6] = 0x1f83d9ab;
	ctx->H[7] = 0x5be0cd19;

	ctx->total[0] = ctx->total[1] = 0;
	ctx->buflen = 0;
}


/* Process the remaining bytes in the internal buffer and the usual
   prolog according to the standard and write the result to RESBUF.

   IMPORTANT: On some systems it is required that RESBUF is correctly
   aligned for a 32 bits value.  */
static void * sha256_finish_ctx(struct sha256_ctx *ctx, void *resbuf) {
	/* Take yet unprocessed bytes into account.  */
	uint32_t bytes = ctx->buflen;
	size_t pad;
	unsigned int i;

	/* Now count remaining bytes.  */
	ctx->total[0] += bytes;
	if (ctx->total[0] < bytes) {
		++ctx->total[1];
	}

	pad = bytes >= 56 ? 64 + 56 - bytes : 56 - bytes;
	memcpy(&ctx->buffer[bytes], fillbuf, pad);

	/* Put the 64-bit file length in *bits* at the end of the buffer.  */
	*(uint32_t *) &ctx->buffer[bytes + pad + 4] = SWAP (ctx->total[0] << 3);
	*(uint32_t *) &ctx->buffer[bytes + pad] = SWAP ((ctx->total[1] << 3) |
						  (ctx->total[0] >> 29));

	/* Process last bytes.  */
	sha256_process_block(ctx->buffer, bytes + pad + 8, ctx);

	/* Put result from CTX in first 32 bytes following RESBUF.  */
	for (i = 0; i < 8; ++i) {
		((uint32_t *) resbuf)[i] = SWAP(ctx->H[i]);
	}

	return resbuf;
}


static void sha256_process_bytes(const void *buffer, size_t len, struct sha256_ctx *ctx) {
	/* When we already have some bits in our internal buffer concatenate
	 both inputs first.  */
	if (ctx->buflen != 0) {
		size_t left_over = ctx->buflen;
		size_t add = 128 - left_over > len ? len : 128 - left_over;

		  memcpy(&ctx->buffer[left_over], buffer, add);
		  ctx->buflen += (uint32_t)add;

		if (ctx->buflen > 64) {
			sha256_process_block(ctx->buffer, ctx->buflen & ~63, ctx);
			ctx->buflen &= 63;
			/* The regions in the following copy operation cannot overlap.  */
			memcpy(ctx->buffer, &ctx->buffer[(left_over + add) & ~63], ctx->buflen);
		}

		buffer = (const char *) buffer + add;
		len -= add;
	}

	/* Process available complete blocks.  */
	if (len >= 64) {
/* To check alignment gcc has an appropriate operator.  Other
compilers don't.  */
#if __GNUC__ >= 2
# define UNALIGNED_P(p) (((uintptr_t) p) % __alignof__ (uint32_t) != 0)
#else
# define UNALIGNED_P(p) (((uintptr_t) p) % sizeof (uint32_t) != 0)
#endif
		if (UNALIGNED_P (buffer))
			while (len > 64) {
				sha256_process_block(memcpy(ctx->buffer, buffer, 64), 64, ctx);
				buffer = (const char *) buffer + 64;
				len -= 64;
			} else {
				sha256_process_block(buffer, len & ~63, ctx);
				buffer = (const char *) buffer + (len & ~63);
				len &= 63;
			}
	}

	/* Move remaining bytes into internal buffer.  */
	if (len > 0) {
		size_t left_over = ctx->buflen;

		memcpy(&ctx->buffer[left_over], buffer, len);
		left_over += len;
		if (left_over >= 64) {
			sha256_process_block(ctx->buffer, 64, ctx);
			left_over -= 64;
			memcpy(ctx->buffer, &ctx->buffer[64], left_over);
		}
		ctx->buflen = (uint32_t)left_over;
	}
}


/* Define our magic string to mark salt for SHA256 "encryption"
   replacement.  */
static const char sha256_salt_prefix[] = "$5$";

/* Prefix for optional rounds specification.  */
static const char sha256_rounds_prefix[] = "rounds=";

/* Maximum salt string length.  */
#define SALT_LEN_MAX 16
/* Default number of rounds if not explicitly specified.  */
#define ROUNDS_DEFAULT 5000
/* Minimum number of rounds.  */
#define ROUNDS_MIN 1000
/* Maximum number of rounds.  */
#define ROUNDS_MAX 999999999

/* Table with characters for base64 transformation.  */
static const char b64t[64] ZEND_NONSTRING =
"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

char * php_sha256_crypt_r(const char *key, const char *salt, char *buffer, int buflen)
{
#ifdef PHP_WIN32
	ZEND_SET_ALIGNED(32, unsigned char alt_result[32]);
	ZEND_SET_ALIGNED(32, unsigned char temp_result[32]);
#else
	ZEND_SET_ALIGNED(__alignof__ (uint32_t), unsigned char alt_result[32]);
	ZEND_SET_ALIGNED(__alignof__ (uint32_t), unsigned char temp_result[32]);
#endif

	struct sha256_ctx ctx;
	struct sha256_ctx alt_ctx;
	size_t salt_len;
	size_t key_len;
	size_t cnt;
	char *cp;
	char *copied_key = NULL;
	char *copied_salt = NULL;
	char *p_bytes;
	char *s_bytes;
	/* Default number of rounds.  */
	size_t rounds = ROUNDS_DEFAULT;
	bool rounds_custom = false;

	/* Find beginning of salt string.  The prefix should normally always
	be present.  Just in case it is not.  */
	if (strncmp(sha256_salt_prefix, salt, sizeof(sha256_salt_prefix) - 1) == 0) {
		/* Skip salt prefix.  */
		salt += sizeof(sha256_salt_prefix) - 1;
	}

	if (strncmp(salt, sha256_rounds_prefix, sizeof(sha256_rounds_prefix) - 1) == 0) {
		const char *num = salt + sizeof(sha256_rounds_prefix) - 1;
		char *endp;
		zend_ulong srounds = ZEND_STRTOUL(num, &endp, 10);
		if (*endp == '$') {
			salt = endp + 1;
			if (srounds < ROUNDS_MIN || srounds > ROUNDS_MAX) {
				return NULL;
			}

			rounds = srounds;
			rounds_custom = true;
		}
	}

	salt_len = MIN(strcspn(salt, "$"), SALT_LEN_MAX);
	key_len = strlen(key);
	char *tmp_key = NULL;
	ALLOCA_FLAG(use_heap_key);
	char *tmp_salt = NULL;
	ALLOCA_FLAG(use_heap_salt);

	SET_ALLOCA_FLAG(use_heap_key);
	SET_ALLOCA_FLAG(use_heap_salt);

	if ((uintptr_t)key % __alignof__ (uint32_t) != 0) {
		tmp_key = (char *) do_alloca(key_len + __alignof__(uint32_t), use_heap_key);
		key = copied_key = memcpy(tmp_key + __alignof__(uint32_t) - (uintptr_t)tmp_key % __alignof__(uint32_t), key, key_len);
	}

	if ((uintptr_t)salt % __alignof__(uint32_t) != 0) {
		tmp_salt = (char *) do_alloca(salt_len + 1 + __alignof__(uint32_t), use_heap_salt);
		salt = copied_salt =
		memcpy(tmp_salt + __alignof__(uint32_t) - (uintptr_t)tmp_salt % __alignof__ (uint32_t), salt, salt_len);
		copied_salt[salt_len] = 0;
	}

	/* Prepare for the real work.  */
	sha256_init_ctx(&ctx);

	/* Add the key string.  */
	sha256_process_bytes(key, key_len, &ctx);

	/* The last part is the salt string.  This must be at most 16
	 characters and it ends at the first `$' character (for
	 compatibility with existing implementations).  */
	sha256_process_bytes(salt, salt_len, &ctx);


	/* Compute alternate SHA256 sum with input KEY, SALT, and KEY.  The
	 final result will be added to the first context.  */
	sha256_init_ctx(&alt_ctx);

	/* Add key.  */
	sha256_process_bytes(key, key_len, &alt_ctx);

	/* Add salt.  */
	sha256_process_bytes(salt, salt_len, &alt_ctx);

	/* Add key again.  */
	sha256_process_bytes(key, key_len, &alt_ctx);

	/* Now get result of this (32 bytes) and add it to the other
	 context.  */
	sha256_finish_ctx(&alt_ctx, alt_result);

	/* Add for any character in the key one byte of the alternate sum.  */
	for (cnt = key_len; cnt > 32; cnt -= 32) {
		sha256_process_bytes(alt_result, 32, &ctx);
	}
	sha256_process_bytes(alt_result, cnt, &ctx);

	/* Take the binary representation of the length of the key and for every
	1 add the alternate sum, for every 0 the key.  */
	for (cnt = key_len; cnt > 0; cnt >>= 1) {
		if ((cnt & 1) != 0) {
			sha256_process_bytes(alt_result, 32, &ctx);
		} else {
			sha256_process_bytes(key, key_len, &ctx);
		}
	}

	/* Create intermediate result.  */
	sha256_finish_ctx(&ctx, alt_result);

	/* Start computation of P byte sequence.  */
	sha256_init_ctx(&alt_ctx);

	/* For every character in the password add the entire password.  */
	for (cnt = 0; cnt < key_len; ++cnt) {
		sha256_process_bytes(key, key_len, &alt_ctx);
	}

	/* Finish the digest.  */
	sha256_finish_ctx(&alt_ctx, temp_result);

	/* Create byte sequence P.  */
	ALLOCA_FLAG(use_heap_p_bytes);
	cp = p_bytes = do_alloca(key_len, use_heap_p_bytes);
	for (cnt = key_len; cnt >= 32; cnt -= 32) {
		cp = zend_mempcpy((void *)cp, (const void *)temp_result, 32);
	}
	memcpy(cp, temp_result, cnt);

	/* Start computation of S byte sequence.  */
	sha256_init_ctx(&alt_ctx);

	/* For every character in the password add the entire password.  */
	for (cnt = 0; cnt < (size_t) (16 + alt_result[0]); ++cnt) {
		sha256_process_bytes(salt, salt_len, &alt_ctx);
	}

	/* Finish the digest.  */
	sha256_finish_ctx(&alt_ctx, temp_result);

	/* Create byte sequence S.  */
	ALLOCA_FLAG(use_heap_s_bytes);
	cp = s_bytes = do_alloca(salt_len, use_heap_s_bytes);
	for (cnt = salt_len; cnt >= 32; cnt -= 32) {
		cp = zend_mempcpy(cp, temp_result, 32);
	}
	memcpy(cp, temp_result, cnt);

	/* Repeatedly run the collected hash value through SHA256 to burn
	CPU cycles.  */
	for (cnt = 0; cnt < rounds; ++cnt) {
		/* New context.  */
		sha256_init_ctx(&ctx);

		/* Add key or last result.  */
		if ((cnt & 1) != 0) {
			sha256_process_bytes(p_bytes, key_len, &ctx);
		} else {
			sha256_process_bytes(alt_result, 32, &ctx);
		}

		/* Add salt for numbers not divisible by 3.  */
		if (cnt % 3 != 0) {
			sha256_process_bytes(s_bytes, salt_len, &ctx);
		}

		/* Add key for numbers not divisible by 7.  */
		if (cnt % 7 != 0) {
			sha256_process_bytes(p_bytes, key_len, &ctx);
		}

		/* Add key or last result.  */
		if ((cnt & 1) != 0) {
			sha256_process_bytes(alt_result, 32, &ctx);
		} else {
			sha256_process_bytes(p_bytes, key_len, &ctx);
		}

		/* Create intermediate result.  */
		sha256_finish_ctx(&ctx, alt_result);
	}

	/* Now we can construct the result string.  It consists of three
	parts.  */
	cp = __php_stpncpy(buffer, sha256_salt_prefix, MAX(0, buflen));
	buflen -= sizeof(sha256_salt_prefix) - 1;

	if (rounds_custom) {
#ifdef PHP_WIN32
		int n = _snprintf(cp, MAX(0, buflen), "%s" ZEND_ULONG_FMT "$", sha256_rounds_prefix, rounds);
#else
		int n = snprintf(cp, MAX(0, buflen), "%s%zu$", sha256_rounds_prefix, rounds);
#endif
		cp += n;
		buflen -= n;
	}

	cp = __php_stpncpy(cp, salt, MIN ((size_t) MAX (0, buflen), salt_len));
	buflen -= MIN(MAX (0, buflen), (int)salt_len);

	if (buflen > 0) {
		*cp++ = '$';
		--buflen;
	}

#define b64_from_24bit(B2, B1, B0, N)					      \
  do {									      \
    unsigned int w = ((B2) << 16) | ((B1) << 8) | (B0);			      \
    int n = (N);							      \
    while (n-- > 0 && buflen > 0)					      \
      {									      \
	*cp++ = b64t[w & 0x3f];						      \
	--buflen;							      \
	w >>= 6;							      \
      }									      \
  } while (0)

	b64_from_24bit(alt_result[0], alt_result[10], alt_result[20], 4);
	b64_from_24bit(alt_result[21], alt_result[1], alt_result[11], 4);
	b64_from_24bit(alt_result[12], alt_result[22], alt_result[2], 4);
	b64_from_24bit(alt_result[3], alt_result[13], alt_result[23], 4);
	b64_from_24bit(alt_result[24], alt_result[4], alt_result[14], 4);
	b64_from_24bit(alt_result[15], alt_result[25], alt_result[5], 4);
	b64_from_24bit(alt_result[6], alt_result[16], alt_result[26], 4);
	b64_from_24bit(alt_result[27], alt_result[7], alt_result[17], 4);
	b64_from_24bit(alt_result[18], alt_result[28], alt_result[8], 4);
	b64_from_24bit(alt_result[9], alt_result[19], alt_result[29], 4);
	b64_from_24bit(0, alt_result[31], alt_result[30], 3);
	if (buflen <= 0) {
		errno = ERANGE;
		buffer = NULL;
	} else
		*cp = '\0';		/* Terminate the string.  */

	/* Clear the buffer for the intermediate result so that people
     attaching to processes or reading core dumps cannot get any
     information.  We do it in this way to clear correct_words[]
     inside the SHA256 implementation as well.  */
	sha256_init_ctx(&ctx);
	sha256_finish_ctx(&ctx, alt_result);
	ZEND_SECURE_ZERO(temp_result, sizeof(temp_result));
	ZEND_SECURE_ZERO(p_bytes, key_len);
	ZEND_SECURE_ZERO(s_bytes, salt_len);
	ZEND_SECURE_ZERO(&ctx, sizeof(ctx));
	ZEND_SECURE_ZERO(&alt_ctx, sizeof(alt_ctx));

	if (copied_key != NULL) {
		ZEND_SECURE_ZERO(copied_key, key_len);
	}
	if (copied_salt != NULL) {
		ZEND_SECURE_ZERO(copied_salt, salt_len);
	}
	if (tmp_key != NULL) {
		free_alloca(tmp_key, use_heap_key);
	}
	if (tmp_salt != NULL) {
		free_alloca(tmp_salt, use_heap_salt);
	}
	free_alloca(p_bytes, use_heap_p_bytes);
	free_alloca(s_bytes, use_heap_s_bytes);

	return buffer;
}


/* This entry point is equivalent to the `crypt' function in Unix
   libcs.  */
char * php_sha256_crypt(const char *key, const char *salt)
{
	/* We don't want to have an arbitrary limit in the size of the
	password.  We can compute an upper bound for the size of the
	result in advance and so we can prepare the buffer we pass to
	`sha256_crypt_r'.  */
	ZEND_TLS char *buffer;
	ZEND_TLS int buflen = 0;
	int needed = (sizeof(sha256_salt_prefix) - 1
			+ sizeof(sha256_rounds_prefix) + 9 + 1
			+ (int)strlen(salt) + 1 + 43 + 1);

	if (buflen < needed) {
		char *new_buffer = (char *) realloc(buffer, needed);
		if (new_buffer == NULL) {
			return NULL;
		}

		buffer = new_buffer;
		buflen = needed;
	}

	return php_sha256_crypt_r(key, salt, buffer, buflen);
}


#ifdef TEST
static const struct
{
	const char *input;
	const char result[32];
} tests[] =
	{
	/* Test vectors from FIPS 180-2: appendix B.1.  */
	{ "abc",
	"\xba\x78\x16\xbf\x8f\x01\xcf\xea\x41\x41\x40\xde\x5d\xae\x22\x23"
	"\xb0\x03\x61\xa3\x96\x17\x7a\x9c\xb4\x10\xff\x61\xf2\x00\x15\xad" },
	/* Test vectors from FIPS 180-2: appendix B.2.  */
	{ "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
	"\x24\x8d\x6a\x61\xd2\x06\x38\xb8\xe5\xc0\x26\x93\x0c\x3e\x60\x39"
	"\xa3\x3c\xe4\x59\x64\xff\x21\x67\xf6\xec\xed\xd4\x19\xdb\x06\xc1" },
	/* Test vectors from the NESSIE project.  */
	{ "",
	"\xe3\xb0\xc4\x42\x98\xfc\x1c\x14\x9a\xfb\xf4\xc8\x99\x6f\xb9\x24"
	"\x27\xae\x41\xe4\x64\x9b\x93\x4c\xa4\x95\x99\x1b\x78\x52\xb8\x55" },
	{ "a",
	"\xca\x97\x81\x12\xca\x1b\xbd\xca\xfa\xc2\x31\xb3\x9a\x23\xdc\x4d"
	"\xa7\x86\xef\xf8\x14\x7c\x4e\x72\xb9\x80\x77\x85\xaf\xee\x48\xbb" },
	{ "message digest",
	"\xf7\x84\x6f\x55\xcf\x23\xe1\x4e\xeb\xea\xb5\xb4\xe1\x55\x0c\xad"
	"\x5b\x50\x9e\x33\x48\xfb\xc4\xef\xa3\xa1\x41\x3d\x39\x3c\xb6\x50" },
	{ "abcdefghijklmnopqrstuvwxyz",
	"\x71\xc4\x80\xdf\x93\xd6\xae\x2f\x1e\xfa\xd1\x44\x7c\x66\xc9\x52"
	"\x5e\x31\x62\x18\xcf\x51\xfc\x8d\x9e\xd8\x32\xf2\xda\xf1\x8b\x73" },
	{ "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
	"\x24\x8d\x6a\x61\xd2\x06\x38\xb8\xe5\xc0\x26\x93\x0c\x3e\x60\x39"
	"\xa3\x3c\xe4\x59\x64\xff\x21\x67\xf6\xec\xed\xd4\x19\xdb\x06\xc1" },
	{ "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
	"\xdb\x4b\xfc\xbd\x4d\xa0\xcd\x85\xa6\x0c\x3c\x37\xd3\xfb\xd8\x80"
	"\x5c\x77\xf1\x5f\xc6\xb1\xfd\xfe\x61\x4e\xe0\xa7\xc8\xfd\xb4\xc0" },
	{ "123456789012345678901234567890123456789012345678901234567890"
	"12345678901234567890",
	"\xf3\x71\xbc\x4a\x31\x1f\x2b\x00\x9e\xef\x95\x2d\xd8\x3c\xa8\x0e"
	"\x2b\x60\x02\x6c\x8e\x93\x55\x92\xd0\xf9\xc3\x08\x45\x3c\x81\x3e" }
  };
#define ntests (sizeof (tests) / sizeof (tests[0]))


static const struct
{
	const char *salt;
	const char *input;
	const char *expected;
} tests2[] =
{
	{ "$5$saltstring", "Hello world!",
	"$5$saltstring$5B8vYYiY.CVt1RlTTf8KbXBH3hsxY/GNooZaBBGWEc5" },
	{ "$5$rounds=10000$saltstringsaltstring", "Hello world!",
	"$5$rounds=10000$saltstringsaltst$3xv.VbSHBb41AL9AvLeujZkZRBAwqFMz2."
	"opqey6IcA" },
	{ "$5$rounds=5000$toolongsaltstring", "This is just a test",
	"$5$rounds=5000$toolongsaltstrin$Un/5jzAHMgOGZ5.mWJpuVolil07guHPvOW8"
	"mGRcvxa5" },
	{ "$5$rounds=1400$anotherlongsaltstring",
	"a very much longer text to encrypt.  This one even stretches over more"
	"than one line.",
	"$5$rounds=1400$anotherlongsalts$Rx.j8H.h8HjEDGomFU8bDkXm3XIUnzyxf12"
	"oP84Bnq1" },
	{ "$5$rounds=77777$short",
	"we have a short salt string but not a short password",
	"$5$rounds=77777$short$JiO1O3ZpDAxGJeaDIuqCoEFysAe1mZNJRs3pw0KQRd/" },
	{ "$5$rounds=123456$asaltof16chars..", "a short string",
	"$5$rounds=123456$asaltof16chars..$gP3VQ/6X7UUEW3HkBn2w1/Ptq2jxPyzV/"
	"cZKmF/wJvD" },
	{ "$5$rounds=10$roundstoolow", "the minimum number is still observed",
	"$5$rounds=1000$roundstoolow$yfvwcWrQ8l/K0DAWyuPMDNHpIVlTQebY9l/gL97"
	"2bIC" },
};
#define ntests2 (sizeof (tests2) / sizeof (tests2[0]))


int main(void) {
	struct sha256_ctx ctx;
	char sum[32];
	int result = 0;
	int cnt, i;
	char buf[1000];
	static const char expected[32] =
	"\xcd\xc7\x6e\x5c\x99\x14\xfb\x92\x81\xa1\xc7\xe2\x84\xd7\x3e\x67"
	"\xf1\x80\x9a\x48\xa4\x97\x20\x0e\x04\x6d\x39\xcc\xc7\x11\x2c\xd0";

	for (cnt = 0; cnt < (int) ntests; ++cnt) {
		sha256_init_ctx(&ctx);
		sha256_process_bytes(tests[cnt].input, strlen(tests[cnt].input), &ctx);
		sha256_finish_ctx(&ctx, sum);
		if (memcmp(tests[cnt].result, sum, 32) != 0) {
			printf("test %d run %d failed\n", cnt, 1);
			result = 1;
		}

		sha256_init_ctx(&ctx);
		for (i = 0; tests[cnt].input[i] != '\0'; ++i) {
			sha256_process_bytes(&tests[cnt].input[i], 1, &ctx);
		}
		sha256_finish_ctx(&ctx, sum);
		if (memcmp(tests[cnt].result, sum, 32) != 0) {
			printf("test %d run %d failed\n", cnt, 2);
			result = 1;
		}
	}

	/* Test vector from FIPS 180-2: appendix B.3.  */

	memset(buf, 'a', sizeof(buf));
	sha256_init_ctx(&ctx);
	for (i = 0; i < 1000; ++i) {
		sha256_process_bytes (buf, sizeof (buf), &ctx);
	}

	sha256_finish_ctx(&ctx, sum);

	if (memcmp(expected, sum, 32) != 0) {
		printf("test %d failed\n", cnt);
		result = 1;
	}

	for (cnt = 0; cnt < ntests2; ++cnt) {
		char *cp = php_sha256_crypt(tests2[cnt].input, tests2[cnt].salt);
		if (strcmp(cp, tests2[cnt].expected) != 0) {
			printf("test %d: expected \"%s\", got \"%s\"\n", cnt, tests2[cnt].expected, cp);
			result = 1;
		}
	}

	if (result == 0)
	puts("all tests OK");

	return result;
}
#endif

#undef fillbuf
#undef b64t
#undef K
#undef SWAP
#undef S0
#undef S1
#undef R0
#undef R1
#undef CYCLIC
#undef UNALIGNED_P

#define fillbuf ptn_sha512_fillbuf
#define b64t ptn_sha512_b64t
#define K ptn_sha512_K
/* SHA512-based Unix crypt implementation.
   Released into the Public Domain by Ulrich Drepper <drepper@redhat.com>.  */
/* Windows VC++ port by Pierre Joye <pierre@php.net> */


#include <errno.h>
#include <limits.h>
#ifdef PHP_WIN32
# define __alignof__ __alignof
#else
# ifndef HAVE_ALIGNOF
#  include <stddef.h>
#  define __alignof__(type) offsetof (struct { char c; type member;}, member)
# endif
#endif

#include <stdio.h>
#include <stdlib.h>

#ifdef PHP_WIN32
# include <string.h>
#else
# include <sys/param.h>
# include <sys/types.h>
# include <string.h>
#endif


#ifndef MIN
# define MIN(a, b) (((a) < (b)) ? (a) : (b))
#endif
#ifndef MAX
# define MAX(a, b) (((a) > (b)) ? (a) : (b))
#endif

/* See #51582 */
#ifndef UINT64_C
# define UINT64_C(value) __CONCAT(value, ULL)
#endif

/* Structure to save state of computation between the single steps.  */
struct sha512_ctx
{
	uint64_t H[8];

	uint64_t total[2];
	uint64_t buflen;
	char buffer[256];	/* NB: always correctly aligned for uint64_t.  */
};


#if defined(PHP_WIN32) || (!defined(WORDS_BIGENDIAN))
# define SWAP(n) \
  (((n) << 56)					\
   | (((n) & 0xff00) << 40)			\
   | (((n) & 0xff0000) << 24)			\
   | (((n) & 0xff000000) << 8)			\
   | (((n) >> 8) & 0xff000000)			\
   | (((n) >> 24) & 0xff0000)			\
   | (((n) >> 40) & 0xff00)			\
   | ((n) >> 56))
#else
# define SWAP(n) (n)
#endif

/* This array contains the bytes used to pad the buffer to the next
   64-byte boundary.  (FIPS 180-2:5.1.2)  */
static const unsigned char fillbuf[128] = { 0x80, 0 /* , 0, 0, ...  */ };

/* Constants for SHA512 from FIPS 180-2:4.2.3.  */
static const uint64_t K[80] = {
	UINT64_C (0x428a2f98d728ae22), UINT64_C (0x7137449123ef65cd),
	UINT64_C (0xb5c0fbcfec4d3b2f), UINT64_C (0xe9b5dba58189dbbc),
	UINT64_C (0x3956c25bf348b538), UINT64_C (0x59f111f1b605d019),
	UINT64_C (0x923f82a4af194f9b), UINT64_C (0xab1c5ed5da6d8118),
	UINT64_C (0xd807aa98a3030242), UINT64_C (0x12835b0145706fbe),
	UINT64_C (0x243185be4ee4b28c), UINT64_C (0x550c7dc3d5ffb4e2),
	UINT64_C (0x72be5d74f27b896f), UINT64_C (0x80deb1fe3b1696b1),
	UINT64_C (0x9bdc06a725c71235), UINT64_C (0xc19bf174cf692694),
	UINT64_C (0xe49b69c19ef14ad2), UINT64_C (0xefbe4786384f25e3),
	UINT64_C (0x0fc19dc68b8cd5b5), UINT64_C (0x240ca1cc77ac9c65),
	UINT64_C (0x2de92c6f592b0275), UINT64_C (0x4a7484aa6ea6e483),
	UINT64_C (0x5cb0a9dcbd41fbd4), UINT64_C (0x76f988da831153b5),
	UINT64_C (0x983e5152ee66dfab), UINT64_C (0xa831c66d2db43210),
	UINT64_C (0xb00327c898fb213f), UINT64_C (0xbf597fc7beef0ee4),
	UINT64_C (0xc6e00bf33da88fc2), UINT64_C (0xd5a79147930aa725),
	UINT64_C (0x06ca6351e003826f), UINT64_C (0x142929670a0e6e70),
	UINT64_C (0x27b70a8546d22ffc), UINT64_C (0x2e1b21385c26c926),
	UINT64_C (0x4d2c6dfc5ac42aed), UINT64_C (0x53380d139d95b3df),
	UINT64_C (0x650a73548baf63de), UINT64_C (0x766a0abb3c77b2a8),
	UINT64_C (0x81c2c92e47edaee6), UINT64_C (0x92722c851482353b),
	UINT64_C (0xa2bfe8a14cf10364), UINT64_C (0xa81a664bbc423001),
	UINT64_C (0xc24b8b70d0f89791), UINT64_C (0xc76c51a30654be30),
	UINT64_C (0xd192e819d6ef5218), UINT64_C (0xd69906245565a910),
	UINT64_C (0xf40e35855771202a), UINT64_C (0x106aa07032bbd1b8),
	UINT64_C (0x19a4c116b8d2d0c8), UINT64_C (0x1e376c085141ab53),
	UINT64_C (0x2748774cdf8eeb99), UINT64_C (0x34b0bcb5e19b48a8),
	UINT64_C (0x391c0cb3c5c95a63), UINT64_C (0x4ed8aa4ae3418acb),
	UINT64_C (0x5b9cca4f7763e373), UINT64_C (0x682e6ff3d6b2b8a3),
	UINT64_C (0x748f82ee5defb2fc), UINT64_C (0x78a5636f43172f60),
	UINT64_C (0x84c87814a1f0ab72), UINT64_C (0x8cc702081a6439ec),
	UINT64_C (0x90befffa23631e28), UINT64_C (0xa4506cebde82bde9),
	UINT64_C (0xbef9a3f7b2c67915), UINT64_C (0xc67178f2e372532b),
	UINT64_C (0xca273eceea26619c), UINT64_C (0xd186b8c721c0c207),
	UINT64_C (0xeada7dd6cde0eb1e), UINT64_C (0xf57d4f7fee6ed178),
	UINT64_C (0x06f067aa72176fba), UINT64_C (0x0a637dc5a2c898a6),
	UINT64_C (0x113f9804bef90dae), UINT64_C (0x1b710b35131c471b),
	UINT64_C (0x28db77f523047d84), UINT64_C (0x32caab7b40c72493),
	UINT64_C (0x3c9ebe0a15c9bebc), UINT64_C (0x431d67c49c100d4c),
	UINT64_C (0x4cc5d4becb3e42b6), UINT64_C (0x597f299cfc657e2a),
	UINT64_C (0x5fcb6fab3ad6faec), UINT64_C (0x6c44198c4a475817)
  };


/* Process LEN bytes of BUFFER, accumulating context into CTX.
   It is assumed that LEN % 128 == 0.  */
static void
sha512_process_block(const void *buffer, size_t len, struct sha512_ctx *ctx) {
	const uint64_t *words = buffer;
	size_t nwords = len / sizeof(uint64_t);
	uint64_t a = ctx->H[0];
	uint64_t b = ctx->H[1];
	uint64_t c = ctx->H[2];
	uint64_t d = ctx->H[3];
	uint64_t e = ctx->H[4];
	uint64_t f = ctx->H[5];
	uint64_t g = ctx->H[6];
	uint64_t h = ctx->H[7];

  /* First increment the byte count.  FIPS 180-2 specifies the possible
	 length of the file up to 2^128 bits.  Here we only compute the
	 number of bytes.  Do a double word increment.  */
	ctx->total[0] += len;
	if (ctx->total[0] < len) {
		++ctx->total[1];
	}

	/* Process all bytes in the buffer with 128 bytes in each round of
	 the loop.  */
	while (nwords > 0) {
		uint64_t W[80];
		uint64_t a_save = a;
		uint64_t b_save = b;
		uint64_t c_save = c;
		uint64_t d_save = d;
		uint64_t e_save = e;
		uint64_t f_save = f;
		uint64_t g_save = g;
		uint64_t h_save = h;
		unsigned int t;

/* Operators defined in FIPS 180-2:4.1.2.  */
#define Ch(x, y, z) ((x & y) ^ (~x & z))
#define Maj(x, y, z) ((x & y) ^ (x & z) ^ (y & z))
#define S0(x) (CYCLIC (x, 28) ^ CYCLIC (x, 34) ^ CYCLIC (x, 39))
#define S1(x) (CYCLIC (x, 14) ^ CYCLIC (x, 18) ^ CYCLIC (x, 41))
#define R0(x) (CYCLIC (x, 1) ^ CYCLIC (x, 8) ^ (x >> 7))
#define R1(x) (CYCLIC (x, 19) ^ CYCLIC (x, 61) ^ (x >> 6))

		/* It is unfortunate that C does not provide an operator for
		   cyclic rotation.  Hope the C compiler is smart enough.  */
#define CYCLIC(w, s) ((w >> s) | (w << (64 - s)))

		/* Compute the message schedule according to FIPS 180-2:6.3.2 step 2.  */
		for (t = 0; t < 16; ++t) {
			W[t] = SWAP (*words);
			++words;
		}

		for (t = 16; t < 80; ++t) {
			W[t] = R1 (W[t - 2]) + W[t - 7] + R0 (W[t - 15]) + W[t - 16];
		}

		/* The actual computation according to FIPS 180-2:6.3.2 step 3.  */
		for (t = 0; t < 80; ++t) {
			uint64_t T1 = h + S1 (e) + Ch (e, f, g) + K[t] + W[t];
			uint64_t T2 = S0 (a) + Maj (a, b, c);
			h = g;
			g = f;
			f = e;
			e = d + T1;
			d = c;
			c = b;
			b = a;
			a = T1 + T2;
		}

		/* Add the starting values of the context according to FIPS 180-2:6.3.2
		step 4.  */
		a += a_save;
		b += b_save;
		c += c_save;
		d += d_save;
		e += e_save;
		f += f_save;
		g += g_save;
		h += h_save;

		/* Prepare for the next round.  */
		nwords -= 16;
	}

	/* Put checksum in context given as argument.  */
	ctx->H[0] = a;
	ctx->H[1] = b;
	ctx->H[2] = c;
	ctx->H[3] = d;
	ctx->H[4] = e;
	ctx->H[5] = f;
	ctx->H[6] = g;
	ctx->H[7] = h;
}


/* Initialize structure containing state of computation.
   (FIPS 180-2:5.3.3)  */
static void sha512_init_ctx (struct sha512_ctx *ctx) {
	ctx->H[0] = UINT64_C (0x6a09e667f3bcc908);
	ctx->H[1] = UINT64_C (0xbb67ae8584caa73b);
	ctx->H[2] = UINT64_C (0x3c6ef372fe94f82b);
	ctx->H[3] = UINT64_C (0xa54ff53a5f1d36f1);
	ctx->H[4] = UINT64_C (0x510e527fade682d1);
	ctx->H[5] = UINT64_C (0x9b05688c2b3e6c1f);
	ctx->H[6] = UINT64_C (0x1f83d9abfb41bd6b);
	ctx->H[7] = UINT64_C (0x5be0cd19137e2179);

	ctx->total[0] = ctx->total[1] = 0;
	ctx->buflen = 0;
}


/* Process the remaining bytes in the internal buffer and the usual
	prolog according to the standard and write the result to RESBUF.

	IMPORTANT: On some systems it is required that RESBUF is correctly
	aligned for a 32 bits value. */
static void * sha512_finish_ctx (struct sha512_ctx *ctx, void *resbuf) {
	/* Take yet unprocessed bytes into account.  */
	uint64_t bytes = ctx->buflen;
	size_t pad;
	unsigned int i;

	/* Now count remaining bytes.  */
	ctx->total[0] += bytes;
	if (ctx->total[0] < bytes) {
		++ctx->total[1];
	}

	pad = bytes >= 112 ? 128 + 112 - (size_t)bytes : 112 - (size_t)bytes;
	memcpy(&ctx->buffer[bytes], fillbuf, pad);

	/* Put the 128-bit file length in *bits* at the end of the buffer.  */
	*(uint64_t *) &ctx->buffer[bytes + pad + 8] = SWAP(ctx->total[0] << 3);
	*(uint64_t *) &ctx->buffer[bytes + pad] = SWAP((ctx->total[1] << 3) |
						(ctx->total[0] >> 61));

	/* Process last bytes.  */
	sha512_process_block(ctx->buffer, (size_t)(bytes + pad + 16), ctx);

	/* Put result from CTX in first 64 bytes following RESBUF.  */
	for (i = 0; i < 8; ++i) {
		((uint64_t *) resbuf)[i] = SWAP(ctx->H[i]);
	}

	return resbuf;
}

static void
sha512_process_bytes(const void *buffer, size_t len, struct sha512_ctx *ctx) {
	/* When we already have some bits in our internal buffer concatenate
	 both inputs first.  */
	if (ctx->buflen != 0) {
		size_t left_over = (size_t)ctx->buflen;
		size_t add = (size_t)(256 - left_over > len ? len : 256 - left_over);

		memcpy(&ctx->buffer[left_over], buffer, add);
		ctx->buflen += add;

		if (ctx->buflen > 128) {
			sha512_process_block(ctx->buffer, ctx->buflen & ~127, ctx);

			ctx->buflen &= 127;
			/* The regions in the following copy operation cannot overlap.  */
			memcpy(ctx->buffer, &ctx->buffer[(left_over + add) & ~127],
					(size_t)ctx->buflen);
		}

		buffer = (const char *) buffer + add;
		len -= add;
	}

	/* Process available complete blocks.  */
	if (len >= 128) {
#ifndef _STRING_ARCH_unaligned
/* To check alignment gcc has an appropriate operator.  Other
   compilers don't.  */
# if __GNUC__ >= 2
#  define UNALIGNED_P(p) (((uintptr_t) p) % __alignof__ (uint64_t) != 0)
# else
#  define UNALIGNED_P(p) (((uintptr_t) p) % sizeof(uint64_t) != 0)
# endif
		if (UNALIGNED_P(buffer))
			while (len > 128) {
				sha512_process_block(memcpy(ctx->buffer, buffer, 128), 128, ctx);
				buffer = (const char *) buffer + 128;
				len -= 128;
			}
		else
#endif
		{
		  sha512_process_block(buffer, len & ~127, ctx);
		  buffer = (const char *) buffer + (len & ~127);
		  len &= 127;
		}
	}

  /* Move remaining bytes into internal buffer.  */
	if (len > 0) {
		size_t left_over = (size_t)ctx->buflen;

		memcpy(&ctx->buffer[left_over], buffer, len);
		left_over += len;
		if (left_over >= 128) {
			sha512_process_block(ctx->buffer, 128, ctx);
			left_over -= 128;
			memcpy(ctx->buffer, &ctx->buffer[128], left_over);
		}
		ctx->buflen = left_over;
	}
}


/* Define our magic string to mark salt for SHA512 "encryption"
   replacement.  */
static const char sha512_salt_prefix[] = "$6$";

/* Prefix for optional rounds specification.  */
static const char sha512_rounds_prefix[] = "rounds=";

/* Maximum salt string length.  */
#define SALT_LEN_MAX 16
/* Default number of rounds if not explicitly specified.  */
#define ROUNDS_DEFAULT 5000
/* Minimum number of rounds.  */
#define ROUNDS_MIN 1000
/* Maximum number of rounds.  */
#define ROUNDS_MAX 999999999

/* Table with characters for base64 transformation.  */
static const char b64t[64] ZEND_NONSTRING =
"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";


char *
php_sha512_crypt_r(const char *key, const char *salt, char *buffer, int buflen) {
#ifdef PHP_WIN32
	ZEND_SET_ALIGNED(64, unsigned char alt_result[64]);
	ZEND_SET_ALIGNED(64, unsigned char temp_result[64]);
#else
	ZEND_SET_ALIGNED(__alignof__ (uint64_t), unsigned char alt_result[64]);
	ZEND_SET_ALIGNED(__alignof__ (uint64_t), unsigned char temp_result[64]);
#endif
	struct sha512_ctx ctx;
	struct sha512_ctx alt_ctx;
	size_t salt_len;
	size_t key_len;
	size_t cnt;
	char *cp;
	char *copied_key = NULL;
	char *copied_salt = NULL;
	char *p_bytes;
	char *s_bytes;
	/* Default number of rounds.  */
	size_t rounds = ROUNDS_DEFAULT;
	bool rounds_custom = false;

	/* Find beginning of salt string.  The prefix should normally always
	 be present.  Just in case it is not.  */
	if (strncmp(sha512_salt_prefix, salt, sizeof(sha512_salt_prefix) - 1) == 0) {
		/* Skip salt prefix.  */
		salt += sizeof(sha512_salt_prefix) - 1;
	}

	if (strncmp(salt, sha512_rounds_prefix, sizeof(sha512_rounds_prefix) - 1) == 0) {
		const char *num = salt + sizeof(sha512_rounds_prefix) - 1;
		char *endp;
		zend_ulong srounds = ZEND_STRTOUL(num, &endp, 10);

		if (*endp == '$') {
			salt = endp + 1;
			if (srounds < ROUNDS_MIN || srounds > ROUNDS_MAX) {
				return NULL;
			}

			rounds = srounds;
			rounds_custom = true;
		}
	}

	salt_len = MIN(strcspn(salt, "$"), SALT_LEN_MAX);
	key_len = strlen(key);
	char *tmp_key = NULL;
	ALLOCA_FLAG(use_heap_key);
	char *tmp_salt = NULL;
	ALLOCA_FLAG(use_heap_salt);

	SET_ALLOCA_FLAG(use_heap_key);
	SET_ALLOCA_FLAG(use_heap_salt);

	if ((uintptr_t)key % __alignof__ (uint64_t) != 0) {
		tmp_key = (char *) do_alloca(key_len + __alignof__ (uint64_t), use_heap_key);
		key = copied_key =
		memcpy(tmp_key + __alignof__(uint64_t) - (uintptr_t)tmp_key % __alignof__(uint64_t), key, key_len);
	}

	if ((uintptr_t)salt % __alignof__ (uint64_t) != 0) {
		tmp_salt = (char *) do_alloca(salt_len + 1 + __alignof__(uint64_t), use_heap_salt);
		salt = copied_salt = memcpy(tmp_salt + __alignof__(uint64_t) - (uintptr_t)tmp_salt % __alignof__(uint64_t), salt, salt_len);
		copied_salt[salt_len] = 0;
	}

	/* Prepare for the real work.  */
	sha512_init_ctx(&ctx);

	/* Add the key string.  */
	sha512_process_bytes(key, key_len, &ctx);

	/* The last part is the salt string.  This must be at most 16
	 characters and it ends at the first `$' character (for
	 compatibility with existing implementations).  */
	sha512_process_bytes(salt, salt_len, &ctx);


	/* Compute alternate SHA512 sum with input KEY, SALT, and KEY.  The
	 final result will be added to the first context.  */
	sha512_init_ctx(&alt_ctx);

	/* Add key.  */
	sha512_process_bytes(key, key_len, &alt_ctx);

	/* Add salt.  */
	sha512_process_bytes(salt, salt_len, &alt_ctx);

	/* Add key again.  */
	sha512_process_bytes(key, key_len, &alt_ctx);

	/* Now get result of this (64 bytes) and add it to the other
	 context.  */
	sha512_finish_ctx(&alt_ctx, alt_result);

	/* Add for any character in the key one byte of the alternate sum.  */
	for (cnt = key_len; cnt > 64; cnt -= 64) {
		sha512_process_bytes(alt_result, 64, &ctx);
	}
	sha512_process_bytes(alt_result, cnt, &ctx);

	/* Take the binary representation of the length of the key and for every
	 1 add the alternate sum, for every 0 the key.  */
	for (cnt = key_len; cnt > 0; cnt >>= 1) {
		if ((cnt & 1) != 0) {
			sha512_process_bytes(alt_result, 64, &ctx);
		} else {
			sha512_process_bytes(key, key_len, &ctx);
		}
	}

	/* Create intermediate result.  */
	sha512_finish_ctx(&ctx, alt_result);

	/* Start computation of P byte sequence.  */
	sha512_init_ctx(&alt_ctx);

	/* For every character in the password add the entire password.  */
	for (cnt = 0; cnt < key_len; ++cnt) {
		sha512_process_bytes(key, key_len, &alt_ctx);
	}

	/* Finish the digest.  */
	sha512_finish_ctx(&alt_ctx, temp_result);

	/* Create byte sequence P.  */
	ALLOCA_FLAG(use_heap_p_bytes);
	cp = p_bytes = do_alloca(key_len, use_heap_p_bytes);
	for (cnt = key_len; cnt >= 64; cnt -= 64) {
		cp = zend_mempcpy((void *) cp, (const void *)temp_result, 64);
	}

	memcpy(cp, temp_result, cnt);

	/* Start computation of S byte sequence.  */
	sha512_init_ctx(&alt_ctx);

	/* For every character in the password add the entire password.  */
	for (cnt = 0; cnt < (size_t) (16 + alt_result[0]); ++cnt) {
		sha512_process_bytes(salt, salt_len, &alt_ctx);
	}

	/* Finish the digest.  */
	sha512_finish_ctx(&alt_ctx, temp_result);

	/* Create byte sequence S.  */
	ALLOCA_FLAG(use_heap_s_bytes);
	cp = s_bytes = do_alloca(salt_len, use_heap_s_bytes);
	for (cnt = salt_len; cnt >= 64; cnt -= 64) {
		cp = zend_mempcpy(cp, temp_result, 64);
	}
	memcpy(cp, temp_result, cnt);

	/* Repeatedly run the collected hash value through SHA512 to burn
	 CPU cycles.  */
	for (cnt = 0; cnt < rounds; ++cnt) {
		/* New context.  */
		sha512_init_ctx(&ctx);

		/* Add key or last result.  */
		if ((cnt & 1) != 0) {
			sha512_process_bytes(p_bytes, key_len, &ctx);
		} else {
			sha512_process_bytes(alt_result, 64, &ctx);
		}

		/* Add salt for numbers not divisible by 3.  */
		if (cnt % 3 != 0) {
			sha512_process_bytes(s_bytes, salt_len, &ctx);
		}

		/* Add key for numbers not divisible by 7.  */
		if (cnt % 7 != 0) {
			sha512_process_bytes(p_bytes, key_len, &ctx);
		}

		/* Add key or last result.  */
		if ((cnt & 1) != 0) {
			sha512_process_bytes(alt_result, 64, &ctx);
		} else {
			sha512_process_bytes(p_bytes, key_len, &ctx);
		}

		/* Create intermediate result.  */
		sha512_finish_ctx(&ctx, alt_result);
	}

	/* Now we can construct the result string.  It consists of three
	 parts.  */
	cp = __php_stpncpy(buffer, sha512_salt_prefix, MAX(0, buflen));
	buflen -= sizeof(sha512_salt_prefix) - 1;

	if (rounds_custom) {
#ifdef PHP_WIN32
	  int n = _snprintf(cp, MAX(0, buflen), "%s" ZEND_ULONG_FMT "$", sha512_rounds_prefix, rounds);
#else
	  int n = snprintf(cp, MAX(0, buflen), "%s%zu$", sha512_rounds_prefix, rounds);
#endif
	  cp += n;
	  buflen -= n;
	}

	cp = __php_stpncpy(cp, salt, MIN((size_t) MAX(0, buflen), salt_len));
	buflen -= (int) MIN((size_t) MAX(0, buflen), salt_len);

	if (buflen > 0) {
		*cp++ = '$';
		--buflen;
	}

#define b64_from_24bit(B2, B1, B0, N)                    \
  do {									                 \
	unsigned int w = ((B2) << 16) | ((B1) << 8) | (B0);	 \
	int n = (N);							             \
	while (n-- > 0 && buflen > 0)					     \
	  {									                 \
	*cp++ = b64t[w & 0x3f];						         \
	--buflen;							                 \
	w >>= 6;							                 \
	  }									                 \
  } while (0)

	b64_from_24bit(alt_result[0], alt_result[21], alt_result[42], 4);
	b64_from_24bit(alt_result[22], alt_result[43], alt_result[1], 4);
	b64_from_24bit(alt_result[44], alt_result[2], alt_result[23], 4);
	b64_from_24bit(alt_result[3], alt_result[24], alt_result[45], 4);
	b64_from_24bit(alt_result[25], alt_result[46], alt_result[4], 4);
	b64_from_24bit(alt_result[47], alt_result[5], alt_result[26], 4);
	b64_from_24bit(alt_result[6], alt_result[27], alt_result[48], 4);
	b64_from_24bit(alt_result[28], alt_result[49], alt_result[7], 4);
	b64_from_24bit(alt_result[50], alt_result[8], alt_result[29], 4);
	b64_from_24bit(alt_result[9], alt_result[30], alt_result[51], 4);
	b64_from_24bit(alt_result[31], alt_result[52], alt_result[10], 4);
	b64_from_24bit(alt_result[53], alt_result[11], alt_result[32], 4);
	b64_from_24bit(alt_result[12], alt_result[33], alt_result[54], 4);
	b64_from_24bit(alt_result[34], alt_result[55], alt_result[13], 4);
	b64_from_24bit(alt_result[56], alt_result[14], alt_result[35], 4);
	b64_from_24bit(alt_result[15], alt_result[36], alt_result[57], 4);
	b64_from_24bit(alt_result[37], alt_result[58], alt_result[16], 4);
	b64_from_24bit(alt_result[59], alt_result[17], alt_result[38], 4);
	b64_from_24bit(alt_result[18], alt_result[39], alt_result[60], 4);
	b64_from_24bit(alt_result[40], alt_result[61], alt_result[19], 4);
	b64_from_24bit(alt_result[62], alt_result[20], alt_result[41], 4);
	b64_from_24bit(0, 0, alt_result[63], 2);

	if (buflen <= 0) {
		errno = ERANGE;
		buffer = NULL;
	} else {
		*cp = '\0';		/* Terminate the string.  */
	}

	/* Clear the buffer for the intermediate result so that people
	 attaching to processes or reading core dumps cannot get any
	 information.  We do it in this way to clear correct_words[]
	 inside the SHA512 implementation as well.  */
	sha512_init_ctx(&ctx);
	sha512_finish_ctx(&ctx, alt_result);
	ZEND_SECURE_ZERO(temp_result, sizeof(temp_result));
	ZEND_SECURE_ZERO(p_bytes, key_len);
	ZEND_SECURE_ZERO(s_bytes, salt_len);
	ZEND_SECURE_ZERO(&ctx, sizeof(ctx));
	ZEND_SECURE_ZERO(&alt_ctx, sizeof(alt_ctx));
	if (copied_key != NULL) {
		ZEND_SECURE_ZERO(copied_key, key_len);
	}
	if (copied_salt != NULL) {
		ZEND_SECURE_ZERO(copied_salt, salt_len);
	}
	if (tmp_key != NULL) {
		free_alloca(tmp_key, use_heap_key);
	}
	if (tmp_salt != NULL) {
		free_alloca(tmp_salt, use_heap_salt);
	}
	free_alloca(p_bytes, use_heap_p_bytes);
	free_alloca(s_bytes, use_heap_s_bytes);

	return buffer;
}


/* This entry point is equivalent to the `crypt' function in Unix
   libcs.  */
char *
php_sha512_crypt(const char *key, const char *salt) {
	/* We don't want to have an arbitrary limit in the size of the
	 password.  We can compute an upper bound for the size of the
	 result in advance and so we can prepare the buffer we pass to
	 `sha512_crypt_r'.  */
	ZEND_TLS char *buffer;
	ZEND_TLS int buflen = 0;
	int needed = (int)(sizeof(sha512_salt_prefix) - 1
		+ sizeof(sha512_rounds_prefix) + 9 + 1
		+ strlen(salt) + 1 + 86 + 1);

	if (buflen < needed) {
		char *new_buffer = (char *) realloc(buffer, needed);
		if (new_buffer == NULL) {
			return NULL;
		}

		buffer = new_buffer;
		buflen = needed;
	}

	return php_sha512_crypt_r (key, salt, buffer, buflen);
}

#ifdef TEST
static const struct {
	const char *input;
	const char result[64];
} tests[] =
	{
	/* Test vectors from FIPS 180-2: appendix C.1.  */
	{ "abc",
	  "\xdd\xaf\x35\xa1\x93\x61\x7a\xba\xcc\x41\x73\x49\xae\x20\x41\x31"
	  "\x12\xe6\xfa\x4e\x89\xa9\x7e\xa2\x0a\x9e\xee\xe6\x4b\x55\xd3\x9a"
	  "\x21\x92\x99\x2a\x27\x4f\xc1\xa8\x36\xba\x3c\x23\xa3\xfe\xeb\xbd"
	  "\x45\x4d\x44\x23\x64\x3c\xe8\x0e\x2a\x9a\xc9\x4f\xa5\x4c\xa4\x9f" },
	/* Test vectors from FIPS 180-2: appendix C.2.  */
	{ "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn"
	  "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu",
	  "\x8e\x95\x9b\x75\xda\xe3\x13\xda\x8c\xf4\xf7\x28\x14\xfc\x14\x3f"
	  "\x8f\x77\x79\xc6\xeb\x9f\x7f\xa1\x72\x99\xae\xad\xb6\x88\x90\x18"
	  "\x50\x1d\x28\x9e\x49\x00\xf7\xe4\x33\x1b\x99\xde\xc4\xb5\x43\x3a"
	  "\xc7\xd3\x29\xee\xb6\xdd\x26\x54\x5e\x96\xe5\x5b\x87\x4b\xe9\x09" },
	/* Test vectors from the NESSIE project.  */
	{ "",
	  "\xcf\x83\xe1\x35\x7e\xef\xb8\xbd\xf1\x54\x28\x50\xd6\x6d\x80\x07"
	  "\xd6\x20\xe4\x05\x0b\x57\x15\xdc\x83\xf4\xa9\x21\xd3\x6c\xe9\xce"
	  "\x47\xd0\xd1\x3c\x5d\x85\xf2\xb0\xff\x83\x18\xd2\x87\x7e\xec\x2f"
	  "\x63\xb9\x31\xbd\x47\x41\x7a\x81\xa5\x38\x32\x7a\xf9\x27\xda\x3e" },
	{ "a",
	  "\x1f\x40\xfc\x92\xda\x24\x16\x94\x75\x09\x79\xee\x6c\xf5\x82\xf2"
	  "\xd5\xd7\xd2\x8e\x18\x33\x5d\xe0\x5a\xbc\x54\xd0\x56\x0e\x0f\x53"
	  "\x02\x86\x0c\x65\x2b\xf0\x8d\x56\x02\x52\xaa\x5e\x74\x21\x05\x46"
	  "\xf3\x69\xfb\xbb\xce\x8c\x12\xcf\xc7\x95\x7b\x26\x52\xfe\x9a\x75" },
	{ "message digest",
	  "\x10\x7d\xbf\x38\x9d\x9e\x9f\x71\xa3\xa9\x5f\x6c\x05\x5b\x92\x51"
	  "\xbc\x52\x68\xc2\xbe\x16\xd6\xc1\x34\x92\xea\x45\xb0\x19\x9f\x33"
	  "\x09\xe1\x64\x55\xab\x1e\x96\x11\x8e\x8a\x90\x5d\x55\x97\xb7\x20"
	  "\x38\xdd\xb3\x72\xa8\x98\x26\x04\x6d\xe6\x66\x87\xbb\x42\x0e\x7c" },
	{ "abcdefghijklmnopqrstuvwxyz",
	  "\x4d\xbf\xf8\x6c\xc2\xca\x1b\xae\x1e\x16\x46\x8a\x05\xcb\x98\x81"
	  "\xc9\x7f\x17\x53\xbc\xe3\x61\x90\x34\x89\x8f\xaa\x1a\xab\xe4\x29"
	  "\x95\x5a\x1b\xf8\xec\x48\x3d\x74\x21\xfe\x3c\x16\x46\x61\x3a\x59"
	  "\xed\x54\x41\xfb\x0f\x32\x13\x89\xf7\x7f\x48\xa8\x79\xc7\xb1\xf1" },
	{ "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
	  "\x20\x4a\x8f\xc6\xdd\xa8\x2f\x0a\x0c\xed\x7b\xeb\x8e\x08\xa4\x16"
	  "\x57\xc1\x6e\xf4\x68\xb2\x28\xa8\x27\x9b\xe3\x31\xa7\x03\xc3\x35"
	  "\x96\xfd\x15\xc1\x3b\x1b\x07\xf9\xaa\x1d\x3b\xea\x57\x78\x9c\xa0"
	  "\x31\xad\x85\xc7\xa7\x1d\xd7\x03\x54\xec\x63\x12\x38\xca\x34\x45" },
	{ "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
	  "\x1e\x07\xbe\x23\xc2\x6a\x86\xea\x37\xea\x81\x0c\x8e\xc7\x80\x93"
	  "\x52\x51\x5a\x97\x0e\x92\x53\xc2\x6f\x53\x6c\xfc\x7a\x99\x96\xc4"
	  "\x5c\x83\x70\x58\x3e\x0a\x78\xfa\x4a\x90\x04\x1d\x71\xa4\xce\xab"
	  "\x74\x23\xf1\x9c\x71\xb9\xd5\xa3\xe0\x12\x49\xf0\xbe\xbd\x58\x94" },
	{ "123456789012345678901234567890123456789012345678901234567890"
	  "12345678901234567890",
	  "\x72\xec\x1e\xf1\x12\x4a\x45\xb0\x47\xe8\xb7\xc7\x5a\x93\x21\x95"
	  "\x13\x5b\xb6\x1d\xe2\x4e\xc0\xd1\x91\x40\x42\x24\x6e\x0a\xec\x3a"
	  "\x23\x54\xe0\x93\xd7\x6f\x30\x48\xb4\x56\x76\x43\x46\x90\x0c\xb1"
	  "\x30\xd2\xa4\xfd\x5d\xd1\x6a\xbb\x5e\x30\xbc\xb8\x50\xde\xe8\x43" }
  };
#define ntests (sizeof (tests) / sizeof (tests[0]))


static const struct
{
	const char *salt;
	const char *input;
	const char *expected;
} tests2[] = {
	{ "$6$saltstring", "Hello world!",
	"$6$saltstring$svn8UoSVapNtMuq1ukKS4tPQd8iKwSMHWjl/O817G3uBnIFNjnQJu"
	"esI68u4OTLiBFdcbYEdFCoEOfaS35inz1"},
	{ "$6$rounds=10000$saltstringsaltstring", "Hello world!",
	"$6$rounds=10000$saltstringsaltst$OW1/O6BYHV6BcXZu8QVeXbDWra3Oeqh0sb"
	"HbbMCVNSnCM/UrjmM0Dp8vOuZeHBy/YTBmSK6H9qs/y3RnOaw5v." },
	{ "$6$rounds=5000$toolongsaltstring", "This is just a test",
	"$6$rounds=5000$toolongsaltstrin$lQ8jolhgVRVhY4b5pZKaysCLi0QBxGoNeKQ"
	"zQ3glMhwllF7oGDZxUhx1yxdYcz/e1JSbq3y6JMxxl8audkUEm0" },
	{ "$6$rounds=1400$anotherlongsaltstring",
	"a very much longer text to encrypt.  This one even stretches over more"
	"than one line.",
	"$6$rounds=1400$anotherlongsalts$POfYwTEok97VWcjxIiSOjiykti.o/pQs.wP"
	"vMxQ6Fm7I6IoYN3CmLs66x9t0oSwbtEW7o7UmJEiDwGqd8p4ur1" },
	{ "$6$rounds=77777$short",
	"we have a short salt string but not a short password",
	"$6$rounds=77777$short$WuQyW2YR.hBNpjjRhpYD/ifIw05xdfeEyQoMxIXbkvr0g"
	"ge1a1x3yRULJ5CCaUeOxFmtlcGZelFl5CxtgfiAc0" },
	{ "$6$rounds=123456$asaltof16chars..", "a short string",
	"$6$rounds=123456$asaltof16chars..$BtCwjqMJGx5hrJhZywWvt0RLE8uZ4oPwc"
	"elCjmw2kSYu.Ec6ycULevoBK25fs2xXgMNrCzIMVcgEJAstJeonj1" },
	{ "$6$rounds=10$roundstoolow", "the minimum number is still observed",
	"$6$rounds=1000$roundstoolow$kUMsbe306n21p9R.FRkW3IGn.S9NPN0x50YhH1x"
	"hLsPuWGsUSklZt58jaTfF4ZEQpyUNGc0dqbpBYYBaHHrsX." },
};
#define ntests2 (sizeof (tests2) / sizeof (tests2[0]))


int main (void) {
	struct sha512_ctx ctx;
	char sum[64];
	int result = 0;
	int cnt;
	int i;
	char buf[1000];
	static const char expected[64] =
		"\xe7\x18\x48\x3d\x0c\xe7\x69\x64\x4e\x2e\x42\xc7\xbc\x15\xb4\x63"
		"\x8e\x1f\x98\xb1\x3b\x20\x44\x28\x56\x32\xa8\x03\xaf\xa9\x73\xeb"
		"\xde\x0f\xf2\x44\x87\x7e\xa6\x0a\x4c\xb0\x43\x2c\xe5\x77\xc3\x1b"
		"\xeb\x00\x9c\x5c\x2c\x49\xaa\x2e\x4e\xad\xb2\x17\xad\x8c\xc0\x9b";

	for (cnt = 0; cnt < (int) ntests; ++cnt) {
		sha512_init_ctx (&ctx);
		sha512_process_bytes (tests[cnt].input, strlen (tests[cnt].input), &ctx);
		sha512_finish_ctx (&ctx, sum);
		if (memcmp (tests[cnt].result, sum, 64) != 0) {
			printf ("test %d run %d failed\n", cnt, 1);
			result = 1;
		}

		sha512_init_ctx (&ctx);
		for (i = 0; tests[cnt].input[i] != '\0'; ++i) {
			sha512_process_bytes (&tests[cnt].input[i], 1, &ctx);
		}
		sha512_finish_ctx (&ctx, sum);
		if (memcmp (tests[cnt].result, sum, 64) != 0) {
			printf ("test %d run %d failed\n", cnt, 2);
			result = 1;
		}
	}

	/* Test vector from FIPS 180-2: appendix C.3.  */

	memset (buf, 'a', sizeof (buf));
	sha512_init_ctx (&ctx);
	for (i = 0; i < 1000; ++i) {
		sha512_process_bytes (buf, sizeof (buf), &ctx);
	}

	sha512_finish_ctx (&ctx, sum);
	if (memcmp (expected, sum, 64) != 0) {
		printf ("test %d failed\n", cnt);
		result = 1;
	}

	for (cnt = 0; cnt < ntests2; ++cnt) {
		char *cp = php_sha512_crypt(tests2[cnt].input, tests2[cnt].salt);

		if (strcmp (cp, tests2[cnt].expected) != 0) {
			printf ("test %d: expected \"%s\", got \"%s\"\n",
					cnt, tests2[cnt].expected, cp);
			result = 1;
		}
	}

	if (result == 0) {
		puts ("all tests OK");
	}

	return result;
}
#endif

#undef fillbuf
#undef b64t
#undef K
#undef SWAP
#undef S0
#undef S1
#undef R0
#undef R1
#undef CYCLIC
#undef UNALIGNED_P
/* MD5 crypt implementation using the windows CryptoApi */
#define MD5_MAGIC "$1$"
#define MD5_MAGIC_LEN 3

static const unsigned char itoa64[] =		/* 0 ... 63 => ascii - 64 */
	"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/* Convert a 16/32 bit integer to Base64 string representation */
static void to64(char *s, int32_t v, int n)
{
	while (--n >= 0) {
		*s++ = itoa64[v & 0x3f];
		v >>= 6;
	}
}

/*
 * MD5 password encryption.
 */
char * php_md5_crypt_r(const char *pw, const char *salt, char *out)
{
	(void)out;
	ZEND_TLS char passwd[MD5_HASH_MAX_LEN], *p;
	const char *sp, *ep;
	unsigned char final[16];
	unsigned int i, sl, pwl;
	PHP_MD5_CTX	ctx, ctx1;
	uint32_t l;
	int pl;

	pwl = strlen(pw);

	/* Refine the salt first */
	sp = salt;

	/* If it starts with the magic string, then skip that */
	if (strncmp(sp, MD5_MAGIC, MD5_MAGIC_LEN) == 0)
		sp += MD5_MAGIC_LEN;

	/* It stops at the first '$', max 8 chars */
	for (ep = sp; *ep != '\0' && *ep != '$' && ep < (sp + 8); ep++);

	/* get the length of the true salt */
	sl = ep - sp;

	PHP_MD5Init(&ctx);

	/* The password first, since that is what is most unknown */
	PHP_MD5Update(&ctx, (const unsigned char *)pw, pwl);

	/* Then our magic string */
	PHP_MD5Update(&ctx, (const unsigned char *)MD5_MAGIC, MD5_MAGIC_LEN);

	/* Then the raw salt */
	PHP_MD5Update(&ctx, (const unsigned char *)sp, sl);

	/* Then just as many characters of the MD5(pw,salt,pw) */
	PHP_MD5Init(&ctx1);
	PHP_MD5Update(&ctx1, (const unsigned char *)pw, pwl);
	PHP_MD5Update(&ctx1, (const unsigned char *)sp, sl);
	PHP_MD5Update(&ctx1, (const unsigned char *)pw, pwl);
	PHP_MD5Final(final, &ctx1);

	for (pl = pwl; pl > 0; pl -= 16)
		PHP_MD5Update(&ctx, final, (unsigned int)(pl > 16 ? 16 : pl));

	/* Don't leave anything around in vm they could use. */
	ZEND_SECURE_ZERO(final, sizeof(final));

	/* Then something really weird... */
	for (i = pwl; i != 0; i >>= 1)
		if ((i & 1) != 0)
		    PHP_MD5Update(&ctx, final, 1);
		else
		    PHP_MD5Update(&ctx, (const unsigned char *)pw, 1);

	/* Now make the output string */
	memcpy(passwd, MD5_MAGIC, MD5_MAGIC_LEN);
	strlcpy(passwd + MD5_MAGIC_LEN, sp, sl + 1);
	strcat(passwd, "$");

	PHP_MD5Final(final, &ctx);

	/*
	 * And now, just to make sure things don't run too fast. On a 60 MHz
	 * Pentium this takes 34 msec, so you would need 30 seconds to build
	 * a 1000 entry dictionary...
	 */
	for (i = 0; i < 1000; i++) {
		PHP_MD5Init(&ctx1);

		if ((i & 1) != 0)
			PHP_MD5Update(&ctx1, (const unsigned char *)pw, pwl);
		else
			PHP_MD5Update(&ctx1, final, 16);

		if ((i % 3) != 0)
			PHP_MD5Update(&ctx1, (const unsigned char *)sp, sl);

		if ((i % 7) != 0)
			PHP_MD5Update(&ctx1, (const unsigned char *)pw, pwl);

		if ((i & 1) != 0)
			PHP_MD5Update(&ctx1, final, 16);
		else
			PHP_MD5Update(&ctx1, (const unsigned char *)pw, pwl);

		PHP_MD5Final(final, &ctx1);
	}

	p = passwd + sl + MD5_MAGIC_LEN + 1;

	l = (final[ 0]<<16) | (final[ 6]<<8) | final[12]; to64(p,l,4); p += 4;
	l = (final[ 1]<<16) | (final[ 7]<<8) | final[13]; to64(p,l,4); p += 4;
	l = (final[ 2]<<16) | (final[ 8]<<8) | final[14]; to64(p,l,4); p += 4;
	l = (final[ 3]<<16) | (final[ 9]<<8) | final[15]; to64(p,l,4); p += 4;
	l = (final[ 4]<<16) | (final[10]<<8) | final[ 5]; to64(p,l,4); p += 4;
	l =		       final[11]		; to64(p,l,2); p += 2;
	*p = '\0';

	/* Don't leave anything around in vm they could use. */
	ZEND_SECURE_ZERO(final, sizeof(final));
	return (passwd);
}

#define PTN_CRYPT_MAX_SALT_LEN 123
#define PTN_CRYPT_OUTPUT_SIZE 256
#define PTN_IS_VALID_SALT_CHARACTER(c) (((c) >= '.' && (c) <= '9') || ((c) >= 'A' && (c) <= 'Z') || ((c) >= 'a' && (c) <= 'z'))

static char *ptn_php_crypt_port(const char *password, const char *salt, char *result, size_t result_size) {
    if (salt[0] == '*' && (salt[1] == '0' || salt[1] == '1')) {
        return NULL;
    }

    char *crypt_res = NULL;
    if (salt[0] == '$' && salt[1] == '1' && salt[2] == '$') {
        char output[MD5_HASH_MAX_LEN];
        crypt_res = php_md5_crypt_r(password, salt, output);
        if (crypt_res == NULL) {
            return NULL;
        }
        snprintf(result, result_size, "%s", crypt_res);
        return result;
    }

    if (salt[0] == '$' && salt[1] == '6' && salt[2] == '$') {
        crypt_res = php_sha512_crypt_r(password, salt, result, (int)result_size);
        return crypt_res == NULL ? NULL : result;
    }

    if (salt[0] == '$' && salt[1] == '5' && salt[2] == '$') {
        crypt_res = php_sha256_crypt_r(password, salt, result, (int)result_size);
        return crypt_res == NULL ? NULL : result;
    }

    if (salt[0] == '$' && salt[1] == '2' && salt[2] != 0 && salt[3] == '$') {
        crypt_res = php_crypt_blowfish_rn(password, salt, result, (int)result_size);
        return crypt_res == NULL ? NULL : result;
    }

    if (salt[0] == '_' || (PTN_IS_VALID_SALT_CHARACTER((unsigned char)salt[0]) && PTN_IS_VALID_SALT_CHARACTER((unsigned char)salt[1]))) {
        struct php_crypt_extended_data buffer;
        memset(&buffer, 0, sizeof(buffer));
        _crypt_extended_init();
        crypt_res = _crypt_extended_r((const unsigned char *)password, salt, &buffer);
        if (crypt_res == NULL || (salt[0] == '*' && salt[1] == '0')) {
            return NULL;
        }
        snprintf(result, result_size, "%s", crypt_res);
        return result;
    }

    return NULL;
}

#undef PHPAPI
#undef ZEND_TLS
#undef ZEND_NONSTRING
#undef ZEND_ATTRIBUTE_UNUSED
#undef ZEND_SECURE_ZERO
#undef ZEND_SET_ALIGNED
#undef ZEND_STRTOUL
#undef ALLOCA_FLAG
#undef SET_ALLOCA_FLAG
#undef do_alloca
#undef free_alloca
#undef strlcpy
#undef MD5_HASH_MAX_LEN
#undef MD5_MAGIC
#undef MD5_MAGIC_LEN
#undef PTN_IS_VALID_SALT_CHARACTER
#undef PHP_MD5Init
#undef _PASSWORD_EFMT1
#undef crypt
#undef __set_errno
#undef BF_SCALE
#undef BF_N
#undef BF_safe_atoi64
#undef BF_ROUND
#undef BF_INDEX
#undef BF_ENCRYPT
#undef BF_body
#undef Ch
#undef Maj
#undef SALT_LEN_MAX
#undef ROUNDS_DEFAULT
#undef ROUNDS_MIN
#undef ROUNDS_MAX
#undef b64_from_24bit
