use crate::legacy_hash_tables::{SNEFRU_TABLES, TIGER_TABLES};
use gost94::{Digest as GostDigest, Gost94CryptoPro, Gost94Test};

pub(crate) fn gost_digest(bytes: &[u8]) -> Vec<u8> {
    Gost94Test::digest(bytes).to_vec()
}

pub(crate) fn gost_crypto_digest(bytes: &[u8]) -> Vec<u8> {
    Gost94CryptoPro::digest(bytes).to_vec()
}

pub(crate) fn tiger4_digest(bytes: &[u8]) -> Vec<u8> {
    let mut state = [
        0x0123_4567_89AB_CDEF,
        0xFEDC_BA98_7654_3210,
        0xF096_A5B4_C3B2_E187,
    ];

    for chunk in bytes.chunks_exact(64) {
        tiger4_compress(&mut state, chunk.try_into().expect("64-byte chunk"));
    }

    let remainder = bytes.chunks_exact(64).remainder();
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut block = [0u8; 64];
    block[..remainder.len()].copy_from_slice(remainder);
    block[remainder.len()] = 0x01;
    if remainder.len() >= 56 {
        tiger4_compress(&mut state, &block);
        block = [0u8; 64];
    }
    block[56..].copy_from_slice(&bit_len.to_le_bytes());
    tiger4_compress(&mut state, &block);

    let mut digest = Vec::with_capacity(24);
    for word in state {
        digest.extend_from_slice(&word.to_le_bytes());
    }
    digest
}

fn tiger4_compress(state: &mut [u64; 3], raw_block: &[u8; 64]) {
    let mut block = [0u64; 8];
    for (word, chunk) in block.iter_mut().zip(raw_block.chunks_exact(8)) {
        *word = u64::from_le_bytes(chunk.try_into().expect("8-byte chunk"));
    }

    let [mut a, mut b, mut c] = *state;
    tiger_pass(&mut a, &mut b, &mut c, &block, 5);
    tiger_key_schedule(&mut block);
    tiger_pass(&mut c, &mut a, &mut b, &block, 7);
    tiger_key_schedule(&mut block);
    tiger_pass(&mut b, &mut c, &mut a, &block, 9);
    tiger_key_schedule(&mut block);
    tiger_pass(&mut a, &mut b, &mut c, &block, 9);

    let tmp = a;
    a = c;
    c = b;
    b = tmp;

    state[0] ^= a;
    state[1] = b.wrapping_sub(state[1]);
    state[2] = c.wrapping_add(state[2]);
}

fn tiger_round(a: &mut u64, b: &mut u64, c: &mut u64, x: u64, mul: u8) {
    *c ^= x;
    let c_bytes = c.to_le_bytes();
    let a_mix = TIGER_TABLES[0][usize::from(c_bytes[0])]
        ^ TIGER_TABLES[1][usize::from(c_bytes[2])]
        ^ TIGER_TABLES[2][usize::from(c_bytes[4])]
        ^ TIGER_TABLES[3][usize::from(c_bytes[6])];
    let b_mix = TIGER_TABLES[3][usize::from(c_bytes[1])]
        ^ TIGER_TABLES[2][usize::from(c_bytes[3])]
        ^ TIGER_TABLES[1][usize::from(c_bytes[5])]
        ^ TIGER_TABLES[0][usize::from(c_bytes[7])];
    *a = a.wrapping_sub(a_mix);
    *b = b.wrapping_add(b_mix).wrapping_mul(u64::from(mul));
}

fn tiger_pass(a: &mut u64, b: &mut u64, c: &mut u64, block: &[u64; 8], mul: u8) {
    tiger_round(a, b, c, block[0], mul);
    tiger_round(b, c, a, block[1], mul);
    tiger_round(c, a, b, block[2], mul);
    tiger_round(a, b, c, block[3], mul);
    tiger_round(b, c, a, block[4], mul);
    tiger_round(c, a, b, block[5], mul);
    tiger_round(a, b, c, block[6], mul);
    tiger_round(b, c, a, block[7], mul);
}

fn tiger_key_schedule(block: &mut [u64; 8]) {
    block[0] = block[0].wrapping_sub(block[7] ^ 0xA5A5_A5A5_A5A5_A5A5);
    block[1] ^= block[0];
    block[2] = block[2].wrapping_add(block[1]);
    block[3] = block[3].wrapping_sub(block[2] ^ ((!block[1]) << 19));
    block[4] ^= block[3];
    block[5] = block[5].wrapping_add(block[4]);
    block[6] = block[6].wrapping_sub(block[5] ^ ((!block[4]) >> 23));
    block[7] ^= block[6];
    block[0] = block[0].wrapping_add(block[7]);
    block[1] = block[1].wrapping_sub(block[0] ^ ((!block[7]) << 19));
    block[2] ^= block[1];
    block[3] = block[3].wrapping_add(block[2]);
    block[4] = block[4].wrapping_sub(block[3] ^ ((!block[2]) >> 23));
    block[5] ^= block[4];
    block[6] = block[6].wrapping_add(block[5]);
    block[7] = block[7].wrapping_sub(block[6] ^ 0x0123_4567_89AB_CDEF);
}

pub(crate) fn snefru256_digest(bytes: &[u8]) -> Vec<u8> {
    let mut state = [0u32; 16];
    for chunk in bytes.chunks_exact(32) {
        snefru_transform(&mut state, chunk);
    }
    let remainder = bytes.chunks_exact(32).remainder();
    if !remainder.is_empty() {
        let mut block = [0u8; 32];
        block[..remainder.len()].copy_from_slice(remainder);
        snefru_transform(&mut state, &block);
    }

    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    state[14] = (bit_len >> 32) as u32;
    state[15] = bit_len as u32;
    snefru_rounds(&mut state);

    let mut digest = Vec::with_capacity(32);
    for word in state.iter().take(8) {
        digest.extend_from_slice(&word.to_be_bytes());
    }
    digest
}

fn snefru_transform(state: &mut [u32; 16], input: &[u8]) {
    debug_assert_eq!(input.len(), 32);
    for (index, chunk) in input.chunks_exact(4).enumerate() {
        state[8 + index] = u32::from_be_bytes(chunk.try_into().expect("4-byte chunk"));
    }
    snefru_rounds(state);
    state[8..].fill(0);
}

fn snefru_rounds(input: &mut [u32; 16]) {
    const SHIFTS: [u32; 4] = [16, 8, 16, 24];

    let mut words = *input;
    for index in 0..8 {
        let t0 = &SNEFRU_TABLES[2 * index];
        let t1 = &SNEFRU_TABLES[2 * index + 1];
        for &shift in &SHIFTS {
            snefru_round_pair(&mut words, 15, 0, 1, t0);
            snefru_round_pair(&mut words, 0, 1, 2, t0);
            snefru_round_pair(&mut words, 1, 2, 3, t1);
            snefru_round_pair(&mut words, 2, 3, 4, t1);
            snefru_round_pair(&mut words, 3, 4, 5, t0);
            snefru_round_pair(&mut words, 4, 5, 6, t0);
            snefru_round_pair(&mut words, 5, 6, 7, t1);
            snefru_round_pair(&mut words, 6, 7, 8, t1);
            snefru_round_pair(&mut words, 7, 8, 9, t0);
            snefru_round_pair(&mut words, 8, 9, 10, t0);
            snefru_round_pair(&mut words, 9, 10, 11, t1);
            snefru_round_pair(&mut words, 10, 11, 12, t1);
            snefru_round_pair(&mut words, 11, 12, 13, t0);
            snefru_round_pair(&mut words, 12, 13, 14, t0);
            snefru_round_pair(&mut words, 13, 14, 15, t1);
            snefru_round_pair(&mut words, 14, 15, 0, t1);

            for word in &mut words {
                *word = word.rotate_right(shift);
            }
        }
    }

    for (index, target) in input.iter_mut().take(8).enumerate() {
        *target ^= words[15 - index];
    }
}

fn snefru_round_pair(
    words: &mut [u32; 16],
    left: usize,
    center: usize,
    next: usize,
    table: &[u32; 256],
) {
    let sbe = table[(words[center] & 0xff) as usize];
    words[left] ^= sbe;
    words[next] ^= sbe;
}

pub(crate) fn haval_digest(bytes: &[u8], passes: u8, output_bits: u16) -> Vec<u8> {
    debug_assert!(matches!(passes, 3 | 4 | 5));
    debug_assert!(matches!(output_bits, 128 | 160 | 192 | 224 | 256));

    let mut state = HAVAL_D0;
    for chunk in bytes.chunks_exact(128) {
        haval_transform(&mut state, chunk, passes);
    }

    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let remainder = bytes.chunks_exact(128).remainder();
    let pad_len = if remainder.len() < 118 {
        118 - remainder.len()
    } else {
        246 - remainder.len()
    };
    let mut final_bytes = Vec::with_capacity(remainder.len() + pad_len + 10);
    final_bytes.extend_from_slice(remainder);
    final_bytes.push(1);
    final_bytes.resize(remainder.len() + pad_len, 0);
    final_bytes.push(
        (HAVAL_VERSION & 0x07) | ((passes & 0x07) << 3) | (((output_bits as u8) & 0x03) << 6),
    );
    final_bytes.push((output_bits >> 2) as u8);
    final_bytes.extend_from_slice(&(bit_len as u32).to_le_bytes());
    final_bytes.extend_from_slice(&((bit_len >> 32) as u32).to_le_bytes());

    for chunk in final_bytes.chunks_exact(128) {
        haval_transform(&mut state, chunk, passes);
    }

    haval_fold_output(&mut state, output_bits);
    let mut digest = Vec::with_capacity(output_bits as usize / 8);
    for word in state.iter().take(output_bits as usize / 32) {
        digest.extend_from_slice(&word.to_le_bytes());
    }
    digest
}

const HAVAL_VERSION: u8 = 0x01;

const HAVAL_D0: [u32; 8] = [
    0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344, 0xa4093822, 0x299f31d0, 0x082efa98, 0xec4e6c89,
];

const HAVAL_K2: [u32; 32] = [
    0x452821e6, 0x38d01377, 0xbe5466cf, 0x34e90c6c, 0xc0ac29b7, 0xc97c50dd, 0x3f84d5b5, 0xb5470917,
    0x9216d5d9, 0x8979fb1b, 0xd1310ba6, 0x98dfb5ac, 0x2ffd72db, 0xd01adfb7, 0xb8e1afed, 0x6a267e96,
    0xba7c9045, 0xf12c7f99, 0x24a19947, 0xb3916cf7, 0x0801f2e2, 0x858efc16, 0x636920d8, 0x71574e69,
    0xa458fea3, 0xf4933d7e, 0x0d95748f, 0x728eb658, 0x718bcd58, 0x82154aee, 0x7b54a41d, 0xc25a59b5,
];

const HAVAL_K3: [u32; 32] = [
    0x9c30d539, 0x2af26013, 0xc5d1b023, 0x286085f0, 0xca417918, 0xb8db38ef, 0x8e79dcb0, 0x603a180e,
    0x6c9e0e8b, 0xb01e8a3e, 0xd71577c1, 0xbd314b27, 0x78af2fda, 0x55605c60, 0xe65525f3, 0xaa55ab94,
    0x57489862, 0x63e81440, 0x55ca396a, 0x2aab10b6, 0xb4cc5c34, 0x1141e8ce, 0xa15486af, 0x7c72e993,
    0xb3ee1411, 0x636fbc2a, 0x2ba9c55d, 0x741831f6, 0xce5c3e16, 0x9b87931e, 0xafd6ba33, 0x6c24cf5c,
];

const HAVAL_K4: [u32; 32] = [
    0x7a325381, 0x28958677, 0x3b8f4898, 0x6b4bb9af, 0xc4bfe81b, 0x66282193, 0x61d809cc, 0xfb21a991,
    0x487cac60, 0x5dec8032, 0xef845d5d, 0xe98575b1, 0xdc262302, 0xeb651b88, 0x23893e81, 0xd396acc5,
    0x0f6d6ff3, 0x83f44239, 0x2e0b4482, 0xa4842004, 0x69c8f04a, 0x9e1f9b5e, 0x21c66842, 0xf6e96c9a,
    0x670c9c61, 0xabd388f0, 0x6a51a0d2, 0xd8542f68, 0x960fa728, 0xab5133a3, 0x6eef0b6c, 0x137a3be4,
];

const HAVAL_K5: [u32; 32] = [
    0xba3bf050, 0x7efb2a98, 0xa1f1651d, 0x39af0176, 0x66ca593e, 0x82430e88, 0x8cee8619, 0x456f9fb4,
    0x7d84a5c3, 0x3b8b5ebe, 0xe06f75d8, 0x85c12073, 0x401a449f, 0x56c16aa6, 0x4ed3aa62, 0x363f7706,
    0x1bfedf72, 0x429b023d, 0x37d0d724, 0xd00a1248, 0xdb0fead3, 0x49f1c09b, 0x075372c9, 0x80991b7b,
    0x25d479d8, 0xf6e8def7, 0xe3fe501a, 0xb6794c3b, 0x976ce0bd, 0x04c006ba, 0xc1a94fb6, 0x409f60c4,
];

const HAVAL_I2: [usize; 32] = [
    5, 14, 26, 18, 11, 28, 7, 16, 0, 23, 20, 22, 1, 10, 4, 8, 30, 3, 21, 9, 17, 24, 29, 6, 19, 12,
    15, 13, 2, 25, 31, 27,
];

const HAVAL_I3: [usize; 32] = [
    19, 9, 4, 20, 28, 17, 8, 22, 29, 14, 25, 12, 24, 30, 16, 26, 31, 15, 7, 3, 1, 0, 18, 27, 13, 6,
    21, 10, 23, 11, 5, 2,
];

const HAVAL_I4: [usize; 32] = [
    24, 4, 0, 14, 2, 7, 28, 23, 26, 6, 30, 20, 18, 25, 19, 3, 22, 11, 31, 21, 8, 27, 12, 9, 1, 29,
    5, 15, 17, 10, 16, 13,
];

const HAVAL_I5: [usize; 32] = [
    27, 3, 21, 26, 17, 11, 20, 29, 19, 0, 12, 7, 13, 8, 31, 10, 5, 9, 14, 30, 18, 6, 28, 24, 2, 23,
    16, 22, 4, 1, 25, 15,
];

const HAVAL_M: [[usize; 32]; 8] = [
    [
        0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3,
        2, 1,
    ],
    [
        1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4,
        3, 2,
    ],
    [
        2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5,
        4, 3,
    ],
    [
        3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6,
        5, 4,
    ],
    [
        4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7,
        6, 5,
    ],
    [
        5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0,
        7, 6,
    ],
    [
        6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1,
        0, 7,
    ],
    [
        7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2, 1, 0, 7, 6, 5, 4, 3, 2,
        1, 0,
    ],
];

fn haval_transform(state: &mut [u32; 8], block: &[u8], passes: u8) {
    debug_assert_eq!(block.len(), 128);
    let mut x = [0u32; 32];
    for (word, chunk) in x.iter_mut().zip(block.chunks_exact(4)) {
        *word = u32::from_le_bytes(chunk.try_into().expect("4-byte chunk"));
    }

    let mut e = *state;
    match passes {
        3 => transform3(&mut e, &x),
        4 => transform4(&mut e, &x),
        5 => transform5(&mut e, &x),
        _ => unreachable!("validated HAVAL pass count"),
    }
    for (target, value) in state.iter_mut().zip(e) {
        *target = (*target).wrapping_add(value);
    }
}

fn transform3(e: &mut [u32; 8], x: &[u32; 32]) {
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f1(
                hm(e, 1, i),
                hm(e, 0, i),
                hm(e, 3, i),
                hm(e, 5, i),
                hm(e, 6, i),
                hm(e, 2, i),
                hm(e, 4, i),
            ),
            hm(e, 7, i),
            x[i],
            0,
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f2(
                hm(e, 4, i),
                hm(e, 2, i),
                hm(e, 1, i),
                hm(e, 0, i),
                hm(e, 5, i),
                hm(e, 3, i),
                hm(e, 6, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I2[i]],
            HAVAL_K2[i],
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f3(
                hm(e, 6, i),
                hm(e, 1, i),
                hm(e, 2, i),
                hm(e, 3, i),
                hm(e, 4, i),
                hm(e, 5, i),
                hm(e, 0, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I3[i]],
            HAVAL_K3[i],
        );
    }
}

fn transform4(e: &mut [u32; 8], x: &[u32; 32]) {
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f1(
                hm(e, 2, i),
                hm(e, 6, i),
                hm(e, 1, i),
                hm(e, 4, i),
                hm(e, 5, i),
                hm(e, 3, i),
                hm(e, 0, i),
            ),
            hm(e, 7, i),
            x[i],
            0,
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f2(
                hm(e, 3, i),
                hm(e, 5, i),
                hm(e, 2, i),
                hm(e, 0, i),
                hm(e, 1, i),
                hm(e, 6, i),
                hm(e, 4, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I2[i]],
            HAVAL_K2[i],
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f3(
                hm(e, 1, i),
                hm(e, 4, i),
                hm(e, 3, i),
                hm(e, 6, i),
                hm(e, 0, i),
                hm(e, 2, i),
                hm(e, 5, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I3[i]],
            HAVAL_K3[i],
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f4(
                hm(e, 6, i),
                hm(e, 4, i),
                hm(e, 0, i),
                hm(e, 5, i),
                hm(e, 2, i),
                hm(e, 1, i),
                hm(e, 3, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I4[i]],
            HAVAL_K4[i],
        );
    }
}

fn transform5(e: &mut [u32; 8], x: &[u32; 32]) {
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f1(
                hm(e, 3, i),
                hm(e, 4, i),
                hm(e, 1, i),
                hm(e, 0, i),
                hm(e, 5, i),
                hm(e, 2, i),
                hm(e, 6, i),
            ),
            hm(e, 7, i),
            x[i],
            0,
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f2(
                hm(e, 6, i),
                hm(e, 2, i),
                hm(e, 1, i),
                hm(e, 0, i),
                hm(e, 3, i),
                hm(e, 4, i),
                hm(e, 5, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I2[i]],
            HAVAL_K2[i],
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f3(
                hm(e, 2, i),
                hm(e, 6, i),
                hm(e, 0, i),
                hm(e, 4, i),
                hm(e, 3, i),
                hm(e, 1, i),
                hm(e, 5, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I3[i]],
            HAVAL_K3[i],
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f4(
                hm(e, 1, i),
                hm(e, 5, i),
                hm(e, 3, i),
                hm(e, 2, i),
                hm(e, 0, i),
                hm(e, 4, i),
                hm(e, 6, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I4[i]],
            HAVAL_K4[i],
        );
    }
    for i in 0..32 {
        e[7 - (i % 8)] = haval_step(
            f5(
                hm(e, 2, i),
                hm(e, 5, i),
                hm(e, 0, i),
                hm(e, 6, i),
                hm(e, 4, i),
                hm(e, 3, i),
                hm(e, 1, i),
            ),
            hm(e, 7, i),
            x[HAVAL_I5[i]],
            HAVAL_K5[i],
        );
    }
}

fn hm(e: &[u32; 8], register: usize, step: usize) -> u32 {
    e[HAVAL_M[register][step]]
}

fn haval_step(f: u32, e7: u32, x: u32, k: u32) -> u32 {
    f.rotate_right(7)
        .wrapping_add(e7.rotate_right(11))
        .wrapping_add(x)
        .wrapping_add(k)
}

fn f1(x6: u32, x5: u32, x4: u32, x3: u32, x2: u32, x1: u32, x0: u32) -> u32 {
    (x1 & x4) ^ (x2 & x5) ^ (x3 & x6) ^ (x0 & x1) ^ x0
}

fn f2(x6: u32, x5: u32, x4: u32, x3: u32, x2: u32, x1: u32, x0: u32) -> u32 {
    (x1 & x2 & x3)
        ^ (x2 & x4 & x5)
        ^ (x1 & x2)
        ^ (x1 & x4)
        ^ (x2 & x6)
        ^ (x3 & x5)
        ^ (x4 & x5)
        ^ (x0 & x2)
        ^ x0
}

fn f3(x6: u32, x5: u32, x4: u32, x3: u32, x2: u32, x1: u32, x0: u32) -> u32 {
    (x1 & x2 & x3) ^ (x1 & x4) ^ (x2 & x5) ^ (x3 & x6) ^ (x0 & x3) ^ x0
}

fn f4(x6: u32, x5: u32, x4: u32, x3: u32, x2: u32, x1: u32, x0: u32) -> u32 {
    (x1 & x2 & x3)
        ^ (x2 & x4 & x5)
        ^ (x3 & x4 & x6)
        ^ (x1 & x4)
        ^ (x2 & x6)
        ^ (x3 & x4)
        ^ (x3 & x5)
        ^ (x3 & x6)
        ^ (x4 & x5)
        ^ (x4 & x6)
        ^ (x0 & x4)
        ^ x0
}

fn f5(x6: u32, x5: u32, x4: u32, x3: u32, x2: u32, x1: u32, x0: u32) -> u32 {
    (x1 & x4) ^ (x2 & x5) ^ (x3 & x6) ^ (x0 & x1 & x2 & x3) ^ (x0 & x5) ^ x0
}

fn haval_fold_output(state: &mut [u32; 8], output_bits: u16) {
    match output_bits {
        128 => fold_haval128(state),
        160 => fold_haval160(state),
        192 => fold_haval192(state),
        224 => fold_haval224(state),
        256 => {}
        _ => unreachable!("validated HAVAL output width"),
    }
}

fn fold_haval128(state: &mut [u32; 8]) {
    state[3] = state[3].wrapping_add(
        (state[7] & 0xff000000)
            | (state[6] & 0x00ff0000)
            | (state[5] & 0x0000ff00)
            | (state[4] & 0x000000ff),
    );
    state[2] = state[2].wrapping_add(
        (((state[7] & 0x00ff0000) | (state[6] & 0x0000ff00) | (state[5] & 0x000000ff)) << 8)
            | ((state[4] & 0xff000000) >> 24),
    );
    state[1] = state[1].wrapping_add(
        (((state[7] & 0x0000ff00) | (state[6] & 0x000000ff)) << 16)
            | (((state[5] & 0xff000000) | (state[4] & 0x00ff0000)) >> 16),
    );
    state[0] = state[0].wrapping_add(
        ((state[7] & 0x000000ff) << 24)
            | (((state[6] & 0xff000000) | (state[5] & 0x00ff0000) | (state[4] & 0x0000ff00)) >> 8),
    );
}

fn fold_haval160(state: &mut [u32; 8]) {
    state[4] = state[4].wrapping_add(
        ((state[7] & 0xfe000000) | (state[6] & 0x01f80000) | (state[5] & 0x0007f000)) >> 12,
    );
    state[3] = state[3].wrapping_add(
        ((state[7] & 0x01f80000) | (state[6] & 0x0007f000) | (state[5] & 0x00000fc0)) >> 6,
    );
    state[2] = state[2]
        .wrapping_add((state[7] & 0x0007f000) | (state[6] & 0x00000fc0) | (state[5] & 0x0000003f));
    state[1] = state[1].wrapping_add(
        ((state[7] & 0x00000fc0) | (state[6] & 0x0000003f) | (state[5] & 0xfe000000))
            .rotate_right(25),
    );
    state[0] = state[0].wrapping_add(
        ((state[7] & 0x0000003f) | (state[6] & 0xfe000000) | (state[5] & 0x01f80000))
            .rotate_right(19),
    );
}

fn fold_haval192(state: &mut [u32; 8]) {
    state[5] = state[5].wrapping_add(((state[7] & 0xfc000000) | (state[6] & 0x03e00000)) >> 21);
    state[4] = state[4].wrapping_add(((state[7] & 0x03e00000) | (state[6] & 0x001f0000)) >> 16);
    state[3] = state[3].wrapping_add(((state[7] & 0x001f0000) | (state[6] & 0x0000fc00)) >> 10);
    state[2] = state[2].wrapping_add(((state[7] & 0x0000fc00) | (state[6] & 0x000003e0)) >> 5);
    state[1] = state[1].wrapping_add((state[7] & 0x000003e0) | (state[6] & 0x0000001f));
    state[0] =
        state[0].wrapping_add(((state[7] & 0x0000001f) | (state[6] & 0xfc000000)).rotate_right(26));
}

fn fold_haval224(state: &mut [u32; 8]) {
    state[6] = state[6].wrapping_add(state[7] & 0x0000000f);
    state[5] = state[5].wrapping_add((state[7] >> 4) & 0x0000001f);
    state[4] = state[4].wrapping_add((state[7] >> 9) & 0x0000000f);
    state[3] = state[3].wrapping_add((state[7] >> 13) & 0x0000001f);
    state[2] = state[2].wrapping_add((state[7] >> 18) & 0x0000000f);
    state[1] = state[1].wrapping_add((state[7] >> 22) & 0x0000001f);
    state[0] = state[0].wrapping_add((state[7] >> 27) & 0x0000001f);
}
