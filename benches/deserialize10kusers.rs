use std::hint::black_box;

use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use criterion::{Criterion, criterion_group, criterion_main};

// -------- Structs --------

// Borsh version
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserBorsh {
    pub balance: u64,
    pub nonce: u8,
    pub padding: [u8; 7], // keep alignment to 16 bytes
}

// Bytemuck version
#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, Debug)]
pub struct UserBytemuck {
    pub balance: u64,
    pub nonce: u8,
    pub padding: [u8; 7],
}

// -------- Benchmarks --------

fn bench_deserialize_10k(c: &mut Criterion) {
    // Generate 10k users and serialize with Borsh
    let users: Vec<UserBorsh> = (0..10_000)
        .map(|i| UserBorsh {
            balance: i as u64,
            nonce: (i % 256) as u8,
            padding: [0; 7],
        })
        .collect();
    let bytes: Vec<u8> = users
        .iter()
        .flat_map(|u| borsh::to_vec(u).unwrap())
        .collect();

    // ----- Borsh -----
    c.bench_function("borsh_deserialize_10k_users", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            let mut cursor = &bytes[..];
            for _ in 0..10_000 {
                let u: UserBorsh = BorshDeserialize::deserialize(&mut cursor).unwrap();
                acc = acc.wrapping_add(u.balance ^ (u.nonce as u64));
            }
            black_box(acc);
        })
    });

    // ----- Bytemuck -----
    c.bench_function("bytemuck_from_bytes_10k_users", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            for i in 0..10_000 {
                let start = i * std::mem::size_of::<UserBytemuck>();
                let end = start + std::mem::size_of::<UserBytemuck>();
                let u: &UserBytemuck = bytemuck::from_bytes(&bytes[start..end]);
                acc = acc.wrapping_add(u.balance ^ (u.nonce as u64));
            }
            black_box(acc);
        })
    });

    // ----- Manual -----
    c.bench_function("manual_from_slice_10k_users", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            let mut offset = 0;
            for _ in 0..10_000 {
                let balance = u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
                let nonce = bytes[offset + 8];
                offset += 16; // 8 balance + 1 nonce + 7 padding
                acc = acc.wrapping_add(balance ^ (nonce as u64));
            }
            black_box(acc);
        })
    });
}

criterion_group!(benches, bench_deserialize_10k);
criterion_main!(benches);
