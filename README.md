# Deserialize-bench

A collection of Rust benchmarks comparing deserialization methods: **Borsh**, **Bytemuck**, and **manual parsing**. Designed to explore performance differences on both fixed-size and variable-length data structures, including Solana-style on-chain data.

---

## Benchmarks

- **Fixed-size struct (30M iterations)**: Compare deserialization overhead of simple structs.
- **10k users**: Deserialize realistic user data with Criterion.
- **Variable-size data**: Strings and vectors, showing real-world deserialization challenges.
- **Solana AMM pools**: Mimic on-chain accounts to benchmark practical usage.

---

## Dependencies

```toml
[dependencies]
borsh = { version = "1.5.7", features = ["derive", "std"] }
bytemuck = { version = "1.23.2", features = ["derive"] }
rand = "0.9.2"

[dev-dependencies]
criterion = "0.7.0"
```

## Running locally

```
cargo bench
```

Else if you want to run a single bench

```
cargo bench --bench <filename>
```

## Benchmark Results

| Benchmark | Method | Time (mean) | Notes |
|-----------|--------|------------|-------|
| Loop 30M iterations | Borsh 🔴 | 28.439 ms | Simple struct, repeated 30M times |
| Loop 30M iterations | Bytemuck 🟡 | 32.277 ms | Slightly slower than Borsh due to zero-copy overhead |
| Loop 30M iterations | Manual 🟢 | 28.490 ms | Direct slice parsing, almost same as Borsh |
|                      |        |          |                              |
| Deserialize 10k users | Borsh 🔴 | 7.5530 µs | Fixed-size struct, real data |
| Deserialize 10k users | Bytemuck 🟢 | 5.8817 µs | ~1.3× faster than Borsh |
| Deserialize 10k users | Manual 🟡 | 5.7893 µs | Slightly faster than Bytemuck |
|                      |        |          |                              |
| Deserialize 10k complex | Borsh 🟢 | 919.18 µs | Dynamic strings and vectors |
| Deserialize 10k complex | Manual 🔴 | 995.93 µs | Initial manual slower than Borsh |
| Deserialize 10k complex optimized | Manual 🟢 | 867.23 µs | Optimized manual faster than Borsh |
|                      |        |          |                              |
| Solana 10k pools | Borsh 🔴 | 401.81 µs | On-chain AMM-like struct |
| Solana 10k pools | Bytemuck 🟢 | 3.4398 µs | ~100× faster than Borsh |
| Solana 10k pools | Manual 🟡 | 4.0128 µs | Close to Bytemuck, can be further optimized |


