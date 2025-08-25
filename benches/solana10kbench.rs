use std::hint::black_box;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::{SeedableRng, Rng};
use rand::rngs::StdRng;

// ---------- Struct ----------

#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod, BorshSerialize, BorshDeserialize)]
pub struct AmmPool {
    pub token_a_mint: [u8; 32],
    pub token_b_mint: [u8; 32],
    pub token_a_vault: [u8; 32],
    pub token_b_vault: [u8; 32],
    pub pool_mint: [u8; 32],
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_supply: u64,
    pub fee_rate: u16,
    pub padding: [u8; 6],
}

// ---------- Helper ----------

fn random_pubkey(rng: &mut StdRng) -> [u8; 32] {
    let mut arr = [0u8; 32];
    rng.fill(&mut arr);
    arr
}

fn generate_amm_pools(n: usize) -> Vec<AmmPool> {
    let mut rng = StdRng::seed_from_u64(43);
    (0..n).map(|_| AmmPool {
        token_a_mint: random_pubkey(&mut rng),
        token_b_mint: random_pubkey(&mut rng),
        token_a_vault: random_pubkey(&mut rng),
        token_b_vault: random_pubkey(&mut rng),
        pool_mint: random_pubkey(&mut rng),
        reserve_a: rng.random(),
        reserve_b: rng.random(),
        total_supply: rng.random(),
        fee_rate: rng.random(),
        padding: [0; 6],
    }).collect()
}

// ---------- Manual deserialization ----------

fn manual_amm_pool_total_supply(data: &[u8]) -> u64 {
    // total_supply offset = 32*5 + 8*2 = 160
    u64::from_le_bytes(data[160..168].try_into().unwrap())
}

// ---------- Benchmarks ----------

fn bench_amm_pool(c: &mut Criterion) {
    let pools = generate_amm_pools(10_000);

    let borsh_bytes: Vec<u8> = pools.iter()
        .flat_map(|p| borsh::to_vec(p).unwrap())
        .collect();

    let bytemuck_bytes: Vec<u8> = pools.iter()
        .flat_map(|p| bytemuck::bytes_of(p).iter().copied())
        .collect();

    // ----- Borsh -----
    c.bench_function("borsh_amm_pool", |b| {
        b.iter(|| {
            let mut cursor = &borsh_bytes[..];
            let mut acc = 0u64;
            for _ in 0..pools.len() {
                let p: AmmPool = BorshDeserialize::deserialize(&mut cursor).unwrap();
                acc = acc.wrapping_add(p.total_supply);
            }
            black_box(acc);
        })
    });

    // ----- Bytemuck -----
    c.bench_function("bytemuck_amm_pool", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            let pool_size = std::mem::size_of::<AmmPool>();
            for i in 0..pools.len() {
                let start = i * pool_size;
                let end = start + pool_size;
                let p: &AmmPool = bytemuck::from_bytes(&bytemuck_bytes[start..end]);
                acc = acc.wrapping_add(p.total_supply);
            }
            black_box(acc);
        })
    });

    // ----- Manual -----
    c.bench_function("manual_amm_pool", |b| {
        b.iter(|| {
            let mut acc = 0u64;
            let pool_size = std::mem::size_of::<AmmPool>();
            let mut offset = 0;
            for _ in 0..pools.len() {
                acc = acc.wrapping_add(manual_amm_pool_total_supply(&bytemuck_bytes[offset..]));
                offset += pool_size;
            }
            black_box(acc);
        })
    });
}

// ---------- Criterion Main ----------

criterion_group!(benches, bench_amm_pool);
criterion_main!(benches);
