/*
 * Additional hash algorithms used by ext/hash. The MD4, FNV, and RIPEMD
 * constants and round structure follow php-src's BSD-3-Clause ext/hash
 * implementations, trimmed here to PTN's one-shot digest path.
 */

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static uint32_t ptn_hash_extra_rotl32(uint32_t value, uint32_t amount) {
    return (value << amount) | (value >> (32U - amount));
}

static void ptn_hash_extra_store_le32(unsigned char *out, const uint32_t *input, size_t words) {
    for (size_t i = 0; i < words; i++) {
        out[i * 4] = (unsigned char)(input[i] & 0xff);
        out[i * 4 + 1] = (unsigned char)((input[i] >> 8) & 0xff);
        out[i * 4 + 2] = (unsigned char)((input[i] >> 16) & 0xff);
        out[i * 4 + 3] = (unsigned char)((input[i] >> 24) & 0xff);
    }
}

static uint32_t ptn_hash_extra_load_le32(const unsigned char *input) {
    return (uint32_t)input[0]
        | ((uint32_t)input[1] << 8)
        | ((uint32_t)input[2] << 16)
        | ((uint32_t)input[3] << 24);
}

static void ptn_hash_extra_md4_transform(uint32_t state[4], const unsigned char block[64]) {
    uint32_t x[16];
    for (size_t i = 0; i < 16; i++) {
        x[i] = ptn_hash_extra_load_le32(block + i * 4);
    }

    uint32_t a = state[0];
    uint32_t b = state[1];
    uint32_t c = state[2];
    uint32_t d = state[3];

#define PTN_MD4_F(x, y, z) ((z) ^ ((x) & ((y) ^ (z))))
#define PTN_MD4_G(x, y, z) (((x) & ((y) | (z))) | ((y) & (z)))
#define PTN_MD4_H(x, y, z) ((x) ^ (y) ^ (z))
#define PTN_MD4_R1(a, b, c, d, k, s) (a) = ptn_hash_extra_rotl32((a) + PTN_MD4_F((b), (c), (d)) + x[(k)], (s))
#define PTN_MD4_R2(a, b, c, d, k, s) (a) = ptn_hash_extra_rotl32((a) + PTN_MD4_G((b), (c), (d)) + x[(k)] + UINT32_C(0x5a827999), (s))
#define PTN_MD4_R3(a, b, c, d, k, s) (a) = ptn_hash_extra_rotl32((a) + PTN_MD4_H((b), (c), (d)) + x[(k)] + UINT32_C(0x6ed9eba1), (s))

    PTN_MD4_R1(a, b, c, d, 0, 3);
    PTN_MD4_R1(d, a, b, c, 1, 7);
    PTN_MD4_R1(c, d, a, b, 2, 11);
    PTN_MD4_R1(b, c, d, a, 3, 19);
    PTN_MD4_R1(a, b, c, d, 4, 3);
    PTN_MD4_R1(d, a, b, c, 5, 7);
    PTN_MD4_R1(c, d, a, b, 6, 11);
    PTN_MD4_R1(b, c, d, a, 7, 19);
    PTN_MD4_R1(a, b, c, d, 8, 3);
    PTN_MD4_R1(d, a, b, c, 9, 7);
    PTN_MD4_R1(c, d, a, b, 10, 11);
    PTN_MD4_R1(b, c, d, a, 11, 19);
    PTN_MD4_R1(a, b, c, d, 12, 3);
    PTN_MD4_R1(d, a, b, c, 13, 7);
    PTN_MD4_R1(c, d, a, b, 14, 11);
    PTN_MD4_R1(b, c, d, a, 15, 19);

    PTN_MD4_R2(a, b, c, d, 0, 3);
    PTN_MD4_R2(d, a, b, c, 4, 5);
    PTN_MD4_R2(c, d, a, b, 8, 9);
    PTN_MD4_R2(b, c, d, a, 12, 13);
    PTN_MD4_R2(a, b, c, d, 1, 3);
    PTN_MD4_R2(d, a, b, c, 5, 5);
    PTN_MD4_R2(c, d, a, b, 9, 9);
    PTN_MD4_R2(b, c, d, a, 13, 13);
    PTN_MD4_R2(a, b, c, d, 2, 3);
    PTN_MD4_R2(d, a, b, c, 6, 5);
    PTN_MD4_R2(c, d, a, b, 10, 9);
    PTN_MD4_R2(b, c, d, a, 14, 13);
    PTN_MD4_R2(a, b, c, d, 3, 3);
    PTN_MD4_R2(d, a, b, c, 7, 5);
    PTN_MD4_R2(c, d, a, b, 11, 9);
    PTN_MD4_R2(b, c, d, a, 15, 13);

    PTN_MD4_R3(a, b, c, d, 0, 3);
    PTN_MD4_R3(d, a, b, c, 8, 9);
    PTN_MD4_R3(c, d, a, b, 4, 11);
    PTN_MD4_R3(b, c, d, a, 12, 15);
    PTN_MD4_R3(a, b, c, d, 2, 3);
    PTN_MD4_R3(d, a, b, c, 10, 9);
    PTN_MD4_R3(c, d, a, b, 6, 11);
    PTN_MD4_R3(b, c, d, a, 14, 15);
    PTN_MD4_R3(a, b, c, d, 1, 3);
    PTN_MD4_R3(d, a, b, c, 9, 9);
    PTN_MD4_R3(c, d, a, b, 5, 11);
    PTN_MD4_R3(b, c, d, a, 13, 15);
    PTN_MD4_R3(a, b, c, d, 3, 3);
    PTN_MD4_R3(d, a, b, c, 11, 9);
    PTN_MD4_R3(c, d, a, b, 7, 11);
    PTN_MD4_R3(b, c, d, a, 15, 15);

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;

#undef PTN_MD4_R3
#undef PTN_MD4_R2
#undef PTN_MD4_R1
#undef PTN_MD4_H
#undef PTN_MD4_G
#undef PTN_MD4_F
}

static void ptn_hash_extra_md4_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[16]) {
    size_t padded_len = input_len + 1;
    while ((padded_len % 64) != 56) {
        padded_len++;
    }
    if (padded_len < input_len || padded_len > SIZE_MAX - 8) {
        ptn_abort_out_of_memory();
    }

    unsigned char *message = calloc(padded_len + 8, 1);
    if (message == NULL) {
        ptn_abort_out_of_memory();
    }
    if (input_len != 0) {
        memcpy(message, input, input_len);
    }
    message[input_len] = 0x80;

    uint64_t bit_len = (uint64_t)input_len * 8;
    for (size_t i = 0; i < 8; i++) {
        message[padded_len + i] = (unsigned char)(bit_len >> (8 * i));
    }

    uint32_t state[4] = {
        UINT32_C(0x67452301),
        UINT32_C(0xefcdab89),
        UINT32_C(0x98badcfe),
        UINT32_C(0x10325476)
    };
    for (size_t offset = 0; offset < padded_len + 8; offset += 64) {
        ptn_hash_extra_md4_transform(state, message + offset);
    }

    free(message);
    ptn_hash_extra_store_le32(digest, state, 4);
}

static void ptn_hash_extra_fnv1a64_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[8]) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    for (size_t i = 0; i < input_len; i++) {
        hash ^= (uint64_t)input[i];
        hash *= UINT64_C(0x100000001b3);
    }
    for (size_t i = 0; i < 8; i++) {
        digest[i] = (unsigned char)(hash >> (56 - i * 8));
    }
}

static void ptn_hash_extra_sha512_256_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[32]) {
    struct sha512_ctx ctx;
    ctx.H[0] = UINT64_C(0x22312194fc2bf72c);
    ctx.H[1] = UINT64_C(0x9f555fa3c84c64c2);
    ctx.H[2] = UINT64_C(0x2393b86b6f53b151);
    ctx.H[3] = UINT64_C(0x963877195940eabd);
    ctx.H[4] = UINT64_C(0x96283ee2a88effe3);
    ctx.H[5] = UINT64_C(0xbe5e1e2553863992);
    ctx.H[6] = UINT64_C(0x2b0199fc2c85b8aa);
    ctx.H[7] = UINT64_C(0x0eb72ddc81c52ca2);
    ctx.total[0] = 0;
    ctx.total[1] = 0;
    ctx.buflen = 0;
    if (input_len != 0) {
        sha512_process_bytes(input, input_len, &ctx);
    }
    unsigned char full_digest[64];
    sha512_finish_ctx(&ctx, full_digest);
    memcpy(digest, full_digest, 32);
}

typedef struct {
    uint32_t state[4];
    uint32_t count[2];
    unsigned char buffer[64];
} PtnHashExtraRipemd128Ctx;

typedef struct {
    uint32_t state[10];
    uint32_t count[2];
    unsigned char buffer[64];
} PtnHashExtraRipemd320Ctx;

#define PTN_RIPEMD_F0(x, y, z) ((x) ^ (y) ^ (z))
#define PTN_RIPEMD_F1(x, y, z) (((x) & (y)) | ((~(x)) & (z)))
#define PTN_RIPEMD_F2(x, y, z) (((x) | (~(y))) ^ (z))
#define PTN_RIPEMD_F3(x, y, z) (((x) & (z)) | ((y) & (~(z))))
#define PTN_RIPEMD_F4(x, y, z) ((x) ^ ((y) | (~(z))))
#define PTN_RIPEMD_ROL(n, x) (((x) << (n)) | ((x) >> (32 - (n))))

static const uint32_t PTN_RIPEMD_K_VALUES[5] = {
    UINT32_C(0x00000000),
    UINT32_C(0x5a827999),
    UINT32_C(0x6ed9eba1),
    UINT32_C(0x8f1bbcdc),
    UINT32_C(0xa953fd4e)
};
static const uint32_t PTN_RIPEMD_KK_VALUES[4] = {
    UINT32_C(0x50a28be6),
    UINT32_C(0x5c4dd124),
    UINT32_C(0x6d703ef3),
    UINT32_C(0x00000000)
};
static const uint32_t PTN_RIPEMD_KK160_VALUES[5] = {
    UINT32_C(0x50a28be6),
    UINT32_C(0x5c4dd124),
    UINT32_C(0x6d703ef3),
    UINT32_C(0x7a6d76e9),
    UINT32_C(0x00000000)
};
static const unsigned char PTN_RIPEMD_R[80] = {
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
     7,  4, 13,  1, 10,  6, 15,  3, 12,  0,  9,  5,  2, 14, 11,  8,
     3, 10, 14,  4,  9, 15,  8,  1,  2,  7,  0,  6, 13, 11,  5, 12,
     1,  9, 11, 10,  0,  8, 12,  4, 13,  3,  7, 15, 14,  5,  6,  2,
     4,  0,  5,  9,  7, 12,  2, 10, 14,  1,  3,  8, 11,  6, 15, 13
};
static const unsigned char PTN_RIPEMD_RR[80] = {
     5, 14,  7,  0,  9,  2, 11,  4, 13,  6, 15,  8,  1, 10,  3, 12,
     6, 11,  3,  7,  0, 13,  5, 10, 14, 15,  8, 12,  4,  9,  1,  2,
    15,  5,  1,  3,  7, 14,  6,  9, 11,  8, 12,  2, 10,  0,  4, 13,
     8,  6,  4,  1,  3, 11, 15,  0,  5, 12,  2, 13,  9,  7, 10, 14,
    12, 15, 10,  4,  1,  5,  8,  7,  6,  2, 13, 14,  0,  3,  9, 11
};
static const unsigned char PTN_RIPEMD_S[80] = {
    11, 14, 15, 12,  5,  8,  7,  9, 11, 13, 14, 15,  6,  7,  9,  8,
     7,  6,  8, 13, 11,  9,  7, 15,  7, 12, 15,  9, 11,  7, 13, 12,
    11, 13,  6,  7, 14,  9, 13, 15, 14,  8, 13,  6,  5, 12,  7,  5,
    11, 12, 14, 15, 14, 15,  9,  8,  9, 14,  5,  6,  8,  6,  5, 12,
     9, 15,  5, 11,  6,  8, 13, 12,  5, 12, 13, 14, 11,  8,  5,  6
};
static const unsigned char PTN_RIPEMD_SS[80] = {
     8,  9,  9, 11, 13, 15, 15,  5,  7,  7,  8, 11, 14, 14, 12,  6,
     9, 13, 15,  7, 12,  8,  9, 11,  7,  7, 12,  7,  6, 15, 13, 11,
     9,  7, 15, 11,  8,  6,  6, 14, 12, 13,  5, 14, 13, 13,  7,  5,
    15,  5,  8, 11, 14, 14,  6, 14,  6,  9, 12,  9, 12,  5, 15,  8,
     8,  5, 12,  9, 12,  5, 14,  6,  8, 13,  6,  5, 15, 13, 11, 11
};

#define PTN_RIPEMD_K(n) PTN_RIPEMD_K_VALUES[(n) >> 4]
#define PTN_RIPEMD_KK(n) PTN_RIPEMD_KK_VALUES[(n) >> 4]
#define PTN_RIPEMD_KK160(n) PTN_RIPEMD_KK160_VALUES[(n) >> 4]
#define PTN_RIPEMD_ROLS(j, x) PTN_RIPEMD_ROL(PTN_RIPEMD_S[(j)], (x))
#define PTN_RIPEMD_ROLSS(j, x) PTN_RIPEMD_ROL(PTN_RIPEMD_SS[(j)], (x))

static void ptn_hash_extra_ripemd_decode(uint32_t *output, const unsigned char *input) {
    for (size_t i = 0; i < 16; i++) {
        output[i] = ptn_hash_extra_load_le32(input + i * 4);
    }
}

static void ptn_hash_extra_ripemd128_transform(uint32_t state[4], const unsigned char block[64]) {
    uint32_t a = state[0], b = state[1], c = state[2], d = state[3];
    uint32_t aa = state[0], bb = state[1], cc = state[2], dd = state[3];
    uint32_t tmp;
    uint32_t x[16];
    ptn_hash_extra_ripemd_decode(x, block);

    for (int j = 0; j < 16; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F0(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j));
        a = d; d = c; c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F3(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK(j));
        aa = dd; dd = cc; cc = bb; bb = tmp;
    }
    for (int j = 16; j < 32; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F1(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j));
        a = d; d = c; c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F2(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK(j));
        aa = dd; dd = cc; cc = bb; bb = tmp;
    }
    for (int j = 32; j < 48; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F2(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j));
        a = d; d = c; c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F1(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK(j));
        aa = dd; dd = cc; cc = bb; bb = tmp;
    }
    for (int j = 48; j < 64; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F3(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j));
        a = d; d = c; c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F0(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK(j));
        aa = dd; dd = cc; cc = bb; bb = tmp;
    }

    tmp = state[1] + c + dd;
    state[1] = state[2] + d + aa;
    state[2] = state[3] + a + bb;
    state[3] = state[0] + b + cc;
    state[0] = tmp;
    memset(x, 0, sizeof(x));
}

static void ptn_hash_extra_ripemd320_transform(uint32_t state[10], const unsigned char block[64]) {
    uint32_t a = state[0], b = state[1], c = state[2], d = state[3], e = state[4];
    uint32_t aa = state[5], bb = state[6], cc = state[7], dd = state[8], ee = state[9];
    uint32_t tmp;
    uint32_t x[16];
    ptn_hash_extra_ripemd_decode(x, block);

    for (int j = 0; j < 16; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F0(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j)) + e;
        a = e; e = d; d = PTN_RIPEMD_ROL(10, c); c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F4(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK160(j)) + ee;
        aa = ee; ee = dd; dd = PTN_RIPEMD_ROL(10, cc); cc = bb; bb = tmp;
    }
    tmp = b; b = bb; bb = tmp;
    for (int j = 16; j < 32; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F1(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j)) + e;
        a = e; e = d; d = PTN_RIPEMD_ROL(10, c); c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F3(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK160(j)) + ee;
        aa = ee; ee = dd; dd = PTN_RIPEMD_ROL(10, cc); cc = bb; bb = tmp;
    }
    tmp = d; d = dd; dd = tmp;
    for (int j = 32; j < 48; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F2(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j)) + e;
        a = e; e = d; d = PTN_RIPEMD_ROL(10, c); c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F2(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK160(j)) + ee;
        aa = ee; ee = dd; dd = PTN_RIPEMD_ROL(10, cc); cc = bb; bb = tmp;
    }
    tmp = a; a = aa; aa = tmp;
    for (int j = 48; j < 64; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F3(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j)) + e;
        a = e; e = d; d = PTN_RIPEMD_ROL(10, c); c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F1(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK160(j)) + ee;
        aa = ee; ee = dd; dd = PTN_RIPEMD_ROL(10, cc); cc = bb; bb = tmp;
    }
    tmp = c; c = cc; cc = tmp;
    for (int j = 64; j < 80; j++) {
        tmp = PTN_RIPEMD_ROLS(j, a + PTN_RIPEMD_F4(b, c, d) + x[PTN_RIPEMD_R[j]] + PTN_RIPEMD_K(j)) + e;
        a = e; e = d; d = PTN_RIPEMD_ROL(10, c); c = b; b = tmp;
        tmp = PTN_RIPEMD_ROLSS(j, aa + PTN_RIPEMD_F0(bb, cc, dd) + x[PTN_RIPEMD_RR[j]] + PTN_RIPEMD_KK160(j)) + ee;
        aa = ee; ee = dd; dd = PTN_RIPEMD_ROL(10, cc); cc = bb; bb = tmp;
    }
    tmp = e; e = ee; ee = tmp;

    state[0] += a;
    state[1] += b;
    state[2] += c;
    state[3] += d;
    state[4] += e;
    state[5] += aa;
    state[6] += bb;
    state[7] += cc;
    state[8] += dd;
    state[9] += ee;
    memset(x, 0, sizeof(x));
}

static void ptn_hash_extra_ripemd128_update(PtnHashExtraRipemd128Ctx *ctx, const unsigned char *input, size_t input_len) {
    unsigned int index = (unsigned int)((ctx->count[0] >> 3) & 0x3f);
    ctx->count[0] += (uint32_t)(input_len << 3);
    if (ctx->count[0] < ((uint32_t)input_len << 3)) {
        ctx->count[1]++;
    }
    ctx->count[1] += (uint32_t)(input_len >> 29);

    unsigned int part_len = 64 - index;
    size_t i = 0;
    if (input_len >= part_len) {
        memcpy(&ctx->buffer[index], input, part_len);
        ptn_hash_extra_ripemd128_transform(ctx->state, ctx->buffer);
        for (i = part_len; i + 63 < input_len; i += 64) {
            ptn_hash_extra_ripemd128_transform(ctx->state, &input[i]);
        }
        index = 0;
    }
    memcpy(&ctx->buffer[index], &input[i], input_len - i);
}

static void ptn_hash_extra_ripemd320_update(PtnHashExtraRipemd320Ctx *ctx, const unsigned char *input, size_t input_len) {
    unsigned int index = (unsigned int)((ctx->count[0] >> 3) & 0x3f);
    ctx->count[0] += (uint32_t)(input_len << 3);
    if (ctx->count[0] < ((uint32_t)input_len << 3)) {
        ctx->count[1]++;
    }
    ctx->count[1] += (uint32_t)(input_len >> 29);

    unsigned int part_len = 64 - index;
    size_t i = 0;
    if (input_len >= part_len) {
        memcpy(&ctx->buffer[index], input, part_len);
        ptn_hash_extra_ripemd320_transform(ctx->state, ctx->buffer);
        for (i = part_len; i + 63 < input_len; i += 64) {
            ptn_hash_extra_ripemd320_transform(ctx->state, &input[i]);
        }
        index = 0;
    }
    memcpy(&ctx->buffer[index], &input[i], input_len - i);
}

static void ptn_hash_extra_ripemd_padding(unsigned char padding[64]) {
    memset(padding, 0, 64);
    padding[0] = 0x80;
}

static void ptn_hash_extra_ripemd128_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[16]) {
    PtnHashExtraRipemd128Ctx ctx;
    ctx.count[0] = ctx.count[1] = 0;
    ctx.state[0] = UINT32_C(0x67452301);
    ctx.state[1] = UINT32_C(0xefcdab89);
    ctx.state[2] = UINT32_C(0x98badcfe);
    ctx.state[3] = UINT32_C(0x10325476);
    if (input_len != 0) {
        ptn_hash_extra_ripemd128_update(&ctx, input, input_len);
    }

    unsigned char bits[8];
    ptn_hash_extra_store_le32(bits, ctx.count, 2);
    unsigned char padding[64];
    ptn_hash_extra_ripemd_padding(padding);
    unsigned int index = (unsigned int)((ctx.count[0] >> 3) & 0x3f);
    unsigned int pad_len = index < 56 ? 56 - index : 120 - index;
    ptn_hash_extra_ripemd128_update(&ctx, padding, pad_len);
    ptn_hash_extra_ripemd128_update(&ctx, bits, 8);
    ptn_hash_extra_store_le32(digest, ctx.state, 4);
    memset(&ctx, 0, sizeof(ctx));
}

static void ptn_hash_extra_ripemd320_digest_bytes(const unsigned char *input, size_t input_len, unsigned char digest[40]) {
    PtnHashExtraRipemd320Ctx ctx;
    ctx.count[0] = ctx.count[1] = 0;
    ctx.state[0] = UINT32_C(0x67452301);
    ctx.state[1] = UINT32_C(0xefcdab89);
    ctx.state[2] = UINT32_C(0x98badcfe);
    ctx.state[3] = UINT32_C(0x10325476);
    ctx.state[4] = UINT32_C(0xc3d2e1f0);
    ctx.state[5] = UINT32_C(0x76543210);
    ctx.state[6] = UINT32_C(0xfedcba98);
    ctx.state[7] = UINT32_C(0x89abcdef);
    ctx.state[8] = UINT32_C(0x01234567);
    ctx.state[9] = UINT32_C(0x3c2d1e0f);
    if (input_len != 0) {
        ptn_hash_extra_ripemd320_update(&ctx, input, input_len);
    }

    unsigned char bits[8];
    ptn_hash_extra_store_le32(bits, ctx.count, 2);
    unsigned char padding[64];
    ptn_hash_extra_ripemd_padding(padding);
    unsigned int index = (unsigned int)((ctx.count[0] >> 3) & 0x3f);
    unsigned int pad_len = index < 56 ? 56 - index : 120 - index;
    ptn_hash_extra_ripemd320_update(&ctx, padding, pad_len);
    ptn_hash_extra_ripemd320_update(&ctx, bits, 8);
    ptn_hash_extra_store_le32(digest, ctx.state, 10);
    memset(&ctx, 0, sizeof(ctx));
}

#undef PTN_RIPEMD_ROLSS
#undef PTN_RIPEMD_ROLS
#undef PTN_RIPEMD_KK160
#undef PTN_RIPEMD_KK
#undef PTN_RIPEMD_K
#undef PTN_RIPEMD_ROL
#undef PTN_RIPEMD_F4
#undef PTN_RIPEMD_F3
#undef PTN_RIPEMD_F2
#undef PTN_RIPEMD_F1
#undef PTN_RIPEMD_F0
