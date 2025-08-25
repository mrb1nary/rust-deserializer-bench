use borsh::{BorshDeserialize, BorshSerialize};
use criterion::{Criterion, criterion_group, criterion_main};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;

// ---------- Struct ----------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AmmPoolDynamic {
    pub token_a_mint: [u8; 32],
    pub token_b_mint: [u8; 32],
    pub pool_mint: [u8; 32],
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_supply: u64,
    pub fee_rate: u16,
    pub positions: Vec<u64>, // dynamic per-user liquidity contributions
}

// ---------- Helpers ----------

fn random_pubkey(rng: &mut StdRng) -> [u8; 32] {
    let mut arr = [0u8; 32];
    rng.fill(&mut arr);
    arr
}

fn generate_dynamic_pools(n: usize) -> Vec<AmmPoolDynamic> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..n)
        .map(|_| {
            let positions_len = rng.random_range(1..10);
            let positions = (0..positions_len).map(|_| rng.random_range(1..1000)).collect();
            AmmPoolDynamic {
                token_a_mint: random_pubkey(&mut rng),
                token_b_mint: random_pubkey(&mut rng),
                pool_mint: random_pubkey(&mut rng),
                reserve_a: rng.random(),
                reserve_b: rng.random(),
                total_supply: rng.random(),
                fee_rate: rng.random(),
                positions,
            }
        })
        .collect()
}

// ---------- Benchmark: Borsh ----------

fn bench_borsh_dynamic(c: &mut Criterion) {
    let pools = generate_dynamic_pools(10_000);
    let bytes: Vec<u8> = pools.iter().flat_map(|p| borsh::to_vec(p).unwrap()).collect();

    c.bench_function("borsh_amm_pool_dynamic", |b| {
        b.iter(|| {
            let mut cursor = &bytes[..];
            let mut acc = 0u64;
            for _ in 0..pools.len() {
                let pool: AmmPoolDynamic = BorshDeserialize::deserialize(&mut cursor).unwrap();
                acc = acc.wrapping_add(pool.total_supply);
            }
            black_box(acc);
        });
    });
}

// ---------- Benchmark: Manual ----------

fn bench_manual_dynamic(c: &mut Criterion) {
    let pools = generate_dynamic_pools(10_000);
    let bytes: Vec<u8> = pools.iter().flat_map(|p| borsh::to_vec(p).unwrap()).collect();

    c.bench_function("manual_amm_pool_dynamic", |b| {
        b.iter(|| {
            let mut offset = 0;
            let mut acc = 0u64;

            for _ in 0..pools.len() {
                offset += 32 * 3; // token_a_mint, token_b_mint, pool_mint
                offset += 8 * 3; // reserve_a, reserve_b, total_supply
                offset += 2;     // fee_rate

                let vec_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
                offset += 4 + vec_len * 8; // skip positions

                // only accumulate total_supply (already skipped)
                acc = acc.wrapping_add(u64::from_le_bytes(
                    bytes[offset - 8 - 2 - 8 - 8..offset - 2 - 8 - 8].try_into().unwrap(),
                ));
            }

            black_box(acc);
        });
    });
}

// ---------- Criterion Main ----------

criterion_group!(benches, bench_borsh_dynamic, bench_manual_dynamic);
criterion_main!(benches);
