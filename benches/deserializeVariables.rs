use borsh::{BorshDeserialize, BorshSerialize};
use criterion::{Criterion, criterion_group, criterion_main};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::hint::black_box;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct UserBorsh {
    pub balance: u64,
    pub nonce: u8,
    pub padding: [u8; 7],
    pub name: String,
    pub transactions: Vec<u64>,
}

// Helper: generate random string
fn random_string(rng: &mut StdRng, len: usize) -> String {
    (0..len)
        .map(|_| rng.random_range(b'a'..=b'z') as char)
        .collect()
}

// Generate test data
fn prepare_data() -> Vec<u8> {
    let mut rng = StdRng::seed_from_u64(42);
    let users: Vec<UserBorsh> = (0..10_000)
        .map(|i| {
            let name_len = rng.random_range(5..20);
            let name = random_string(&mut rng, name_len);

            let tx_count = rng.random_range(1..10);
            let transactions: Vec<u64> = (0..tx_count).map(|_| rng.random_range(1..1000)).collect();

            UserBorsh {
                balance: i as u64,
                nonce: (i % 256) as u8,
                padding: [0; 7],
                name,
                transactions,
            }
        })
        .collect();

    // Serialize all users into one big Vec<u8>
    users
        .iter()
        .flat_map(|u| borsh::to_vec(u).unwrap())
        .collect()
}

// Benchmark Borsh deserialization
fn bench_borsh(c: &mut Criterion) {
    let bytes = prepare_data();
    c.bench_function("borsh_deserialize_10k_complex", |b| {
        b.iter(|| {
            let mut cursor = &bytes[..];
            let mut out = Vec::with_capacity(10_000);
            for _ in 0..10_000 {
                let user = UserBorsh::deserialize(&mut cursor).unwrap();
                black_box(out.push(user));
            }
        });
    });
}

// Manual deserialization (parse length + copy into owned types)
fn bench_manual(c: &mut Criterion) {
    let bytes = prepare_data();
    c.bench_function("manual_deserialize_10k_complex", |b| {
        b.iter(|| {
            let mut cursor = &bytes[..];
            let mut out = Vec::with_capacity(10_000);
            for _ in 0..10_000 {
                // balance
                let balance = u64::from_le_bytes(cursor[..8].try_into().unwrap());
                cursor = &cursor[8..];

                // nonce
                let nonce = cursor[0];
                cursor = &cursor[1..];

                // padding
                let mut padding = [0u8; 7];
                padding.copy_from_slice(&cursor[..7]);
                cursor = &cursor[7..];

                // name (Borsh encodes String as u32 length + UTF-8 bytes)
                let name_len = u32::from_le_bytes(cursor[..4].try_into().unwrap()) as usize;
                cursor = &cursor[4..];
                let name = String::from_utf8(cursor[..name_len].to_vec()).unwrap();
                cursor = &cursor[name_len..];

                // transactions (Vec<u64>)
                let tx_len = u32::from_le_bytes(cursor[..4].try_into().unwrap()) as usize;
                cursor = &cursor[4..];
                let mut transactions = Vec::with_capacity(tx_len);
                for _ in 0..tx_len {
                    let val = u64::from_le_bytes(cursor[..8].try_into().unwrap());
                    cursor = &cursor[8..];
                    transactions.push(val);
                }

                out.push(UserBorsh {
                    balance,
                    nonce,
                    padding,
                    name,
                    transactions,
                });
            }
            black_box(out);
        });
    });
}

// Optimized manual deserialization
fn bench_manual_optimized(c: &mut Criterion) {
    let bytes = prepare_data();
    c.bench_function("manual_deserialize_10k_optimized", |b| {
        b.iter(|| {
            let mut offset = 0;
            let mut out = Vec::with_capacity(10_000);

            for _ in 0..10_000 {
                // balance - direct unsafe read (if you want maximum speed)
                let balance =
                    u64::from_le_bytes(unsafe { *(bytes.as_ptr().add(offset) as *const [u8; 8]) });
                offset += 8;

                // nonce
                let nonce = bytes[offset];
                offset += 1;

                // Skip padding entirely (it's just zeros)
                offset += 7;

                // name - more efficient
                let name_len =
                    u32::from_le_bytes(unsafe { *(bytes.as_ptr().add(offset) as *const [u8; 4]) })
                        as usize;
                offset += 4;

                // Skip UTF-8 validation if you know data is valid
                let name = unsafe {
                    String::from_utf8_unchecked(bytes[offset..offset + name_len].to_vec())
                };
                offset += name_len;

                // transactions - batch read
                let tx_len =
                    u32::from_le_bytes(unsafe { *(bytes.as_ptr().add(offset) as *const [u8; 4]) })
                        as usize;
                offset += 4;

                let mut transactions = Vec::with_capacity(tx_len);
                for _ in 0..tx_len {
                    let val = u64::from_le_bytes(unsafe {
                        *(bytes.as_ptr().add(offset) as *const [u8; 8])
                    });
                    offset += 8;
                    transactions.push(val);
                }

                out.push(UserBorsh {
                    balance,
                    nonce,
                    padding: [0; 7], // Don't bother copying
                    name,
                    transactions,
                });
            }
            black_box(out);
        });
    });
}
criterion_group!(benches, bench_borsh, bench_manual, bench_manual_optimized);
criterion_main!(benches);
