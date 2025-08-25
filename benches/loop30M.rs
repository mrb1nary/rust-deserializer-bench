use std::hint::black_box;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use criterion::{criterion_group, criterion_main, Criterion};

// ---------- Structs ----------
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct UserBorsh {
    pub balance: u64,
    pub nonce: u8,
    pub padding: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub struct UserBytemuck {
    pub balance: u64,
    pub nonce: u8,
    pub padding: [u8; 7],
}

// ---------- Manual Deserialize ----------
fn manual_deserialize(data: &[u8]) -> (u64, u8) {
    let balance = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let nonce = data[8];
    (balance, nonce)
}

// Max iterations (30 million)
const MAX_ITERS: usize = 30_000_000;

// ---------- Bench: Borsh ----------
fn bench_borsh(c: &mut Criterion) {
    let user = UserBorsh {
        balance: 1234567890123456789,
        nonce: 42,
        padding: [0; 7],
    };
    let bytes = borsh::to_vec(&user).unwrap();

    c.bench_function("borsh_deserialize", |b| {
        b.iter(|| {
            let mut acc: u64 = 0;
            for _ in 0..MAX_ITERS {
                let u = UserBorsh::try_from_slice(black_box(&bytes)).unwrap();
                acc = acc.wrapping_add(u.balance ^ (u.nonce as u64));
            }
            black_box(acc);
        })
    });
}

// ---------- Bench: Bytemuck ----------
fn bench_bytemuck(c: &mut Criterion) {
    let user = UserBytemuck {
        balance: 1234567890123456789,
        nonce: 42,
        padding: [0; 7],
    };
    let bytes = bytemuck::bytes_of(&user).to_vec();

    c.bench_function("bytemuck_from_bytes", |b| {
        b.iter(|| {
            let mut acc: u64 = 0;
            for _ in 0..MAX_ITERS {
                let u: &UserBytemuck = bytemuck::from_bytes(black_box(&bytes));
                acc = acc.wrapping_add(u.balance ^ (u.nonce as u64));
            }
            black_box(acc);
        })
    });
}

// ---------- Bench: Manual ----------
fn bench_manual(c: &mut Criterion) {
    let user = UserBytemuck {
        balance: 1234567890123456789,
        nonce: 42,
        padding: [0; 7],
    };
    let bytes = bytemuck::bytes_of(&user).to_vec();

    c.bench_function("manual_from_slice", |b| {
        b.iter(|| {
            let mut acc: u64 = 0;
            for _ in 0..MAX_ITERS {
                let (bal, non) = manual_deserialize(black_box(&bytes));
                acc = acc.wrapping_add(bal ^ (non as u64));
            }
            black_box(acc);
        })
    });
}

// ---------- Criterion Main ----------
criterion_group!(benches, bench_borsh, bench_bytemuck, bench_manual);
criterion_main!(benches);
