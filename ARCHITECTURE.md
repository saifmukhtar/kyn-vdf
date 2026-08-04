# kyn-vdf Architecture

> A deep-dive into why this crate exists, what problem it solves, how it was built,
> and exactly how the math works under the hood.

---

## Table of Contents

1. [Why We Built This](#1-why-we-built-this)
2. [What Was Needed](#2-what-was-needed)
3. [How It Was Built](#3-how-it-was-built)
4. [How It Works — The Math](#4-how-it-works--the-math)
   - [Class Groups of Imaginary Quadratic Fields](#41-class-groups-of-imaginary-quadratic-fields)
   - [Binary Quadratic Forms](#42-binary-quadratic-forms)
   - [Discriminant Generation](#43-discriminant-generation)
   - [Shanks' NUCOMP and NUDUPL](#44-shanks-nucomp-and-nudupl)
   - [The Wesolowski Verification Equation](#45-the-wesolowski-verification-equation)
   - [BQFC Serialization](#46-bqfc-serialization)
5. [Codebase Map](#5-codebase-map)
6. [Design Decisions](#6-design-decisions)
7. [What This Is Not](#7-what-this-is-not)

---

## 1. Why We Built This

Verifiable Delay Functions (VDFs) are cryptographic primitives that require a prover
to spend a specific, unavoidable amount of sequential computation time, while allowing
any third party to verify the result in drastically less time.

They are used in:
- **Randomness beacons** (Chia Network, Ethereum consensus)
- **Leader election** in proof-of-stake blockchains
- **Time-locked encryption** (data that becomes decryptable after a delay)
- **Proof-of-sequential-work** for anti-spam and rate-limiting

The **Kinetic network** uses VDFs as a core primitive. Every light client — a browser
wallet, a mobile app, a validator watching the network — must be able to verify that the
prover spent the required sequential time.

### The Gap That Existed

Every open-source VDF implementation before `kyn-vdf` shared the same dependency chain:

```
chiavdf (Chia's reference) ──▶ libgmp (C, GNU bignum) ──▶ native binary
```

This means verification required:
- A C++ compiler
- `libgmp` installed on the OS
- A native OS binary (Linux/macOS/Windows)

**None of that works in:**
- WebAssembly (browser light clients)
- iOS / Android (no C++ NDK without complex cross-compilation)
- CosmWasm / Substrate smart contracts
- Pure Rust embedded environments

We needed verification to work *everywhere* Rust runs. That meant starting from scratch.

---

## 2. What Was Needed

A clean-room, pure Rust implementation that:

| Requirement | Why |
|---|---|
| Zero FFI / Zero C | Must compile to `wasm32-unknown-unknown` |
| Arbitrary-precision arithmetic | 1024-bit integers — no hardware support |
| Chia protocol compatibility | Must verify proofs from real Chia/Kinetic provers |
| `O(log T)` verification | Verification must be fast regardless of proof size |
| No panics in any code path | Safe for adversarial inputs and WASM runtimes |
| `Result`-based error handling | Callers must know *why* a proof failed |

The "no panics" constraint was particularly hard. Mathematical code that handles
arbitrary-precision numbers has countless edge cases — negative square roots, non-coprime
inputs, forms that don't reduce — and the temptation to `unwrap()` or `panic!()` is
ever-present. We eliminated every one.

---

## 3. How It Was Built

### The Build Order

```
discriminant generation  →  form arithmetic  →  BQFC serialization  →  verification equation
     (chia.rs)                 (math.rs)              (chia.rs)               (chia.rs)
```

**Step 1 — Discriminant generation (`create_discriminant`)**
Implemented Chia's `HashPrime` algorithm: iterate SHA-256 over a seed with a big-endian
counter, concatenate outputs, force specific bits, test with Miller-Rabin (25 rounds).
Produces a 1024-bit negative prime $p \equiv 7 \pmod 8$ deterministically from any seed.

**Step 2 — Binary quadratic form arithmetic (`Form`)**
Implemented the `Form` struct $(a, b, c)$ with:
- Gauss reduction (`reduce`)
- Shanks' NUDUPL (`square`) — sub-quadratic squaring via partial XGCD
- Shanks' NUCOMP (`compose`) — sub-quadratic composition via partial XGCD
- Fast exponentiation (`pow`) — binary exponentiation with opportunistic reduction

Each of these was validated against Python reference implementations derived from the
Chia chiavdf source before being committed.

**Step 3 — BQFC serialization (`serialize_form` / `deserialize_form`)**
Implemented Chia's Binary Quadratic Form Compression (BQFC) exactly — the same
100-byte wire format used by the C++ prover. Getting byte-exact compatibility required
careful study of `bqfc.c` and differential testing against Chia test vectors.

**Step 4 — Wesolowski verification (`verify_wesolowski`)**
Wired together: derive the Fiat-Shamir prime $B$, compute $r = 2^T \bmod B$, then
check $\pi^B \cdot x^r = y$ using the class group operations above.

**Step 5 — Hardening**
- Replaced all `panic!` / `assert!` / `unwrap` in library code with `Result<T, KynVdfError>`
- Added property-based tests (`proptest`) for group axioms
- Added differential fuzz testing (`cargo-fuzz`) for deserialization
- Validated against Chia's published test vectors byte-by-byte

### Dependencies (minimal by design)

| Crate | Purpose |
|---|---|
| `num-bigint` | 1024-bit arbitrary-precision integer arithmetic |
| `num-traits` | Numeric traits (`Zero`, `One`, `Signed`, …) |
| `num-integer` | `Integer::gcd`, `extended_gcd`, `mod_floor` |
| `sha2` | SHA-256 for `HashPrime` and the Fiat-Shamir challenge |
| `thiserror` | Ergonomic `#[derive(Error)]` for `KynVdfError` |

Zero unsafe code. Zero OS syscalls. Zero network access. Compiles to WASM with
`cargo build --target wasm32-unknown-unknown --release`.

---

## 4. How It Works — The Math

### 4.1 Class Groups of Imaginary Quadratic Fields

For a negative integer $D < 0$ (the *discriminant*), the **class group**
$\text{Cl}(\mathbb{Q}(\sqrt{D}))$ is a finite abelian group whose elements are
equivalence classes of ideals in the ring of integers $\mathcal{O}_K$.

For cryptographic purposes, the key properties are:

1. **Unknown group order**: Computing $|\text{Cl}(\mathbb{Q}(\sqrt{D}))|$ is as hard as
   factoring $D$. For a 1024-bit prime $D$, this is computationally infeasible.
2. **Efficient computation**: Group elements can be composed in $O(\log^2 D)$ time using
   Gauss reduction.
3. **No trapdoor**: Unlike RSA groups, nobody knows the group order — not even the
   prover. This makes the group order *verifiably unknown*, a requirement for VDFs.

### 4.2 Binary Quadratic Forms

Group elements are represented as **reduced binary quadratic forms**:

$$f = (a, b, c) \quad \text{where} \quad b^2 - 4ac = D$$

A form is *reduced* when $|b| \leq a \leq c$ (and $b \geq 0$ when $|b| = a$ or $a = c$).

In code, this is the [`Form`](src/math.rs) struct:

```rust
pub struct Form {
    pub a: BigInt,   // first coefficient
    pub b: BigInt,   // second coefficient (middle)
    pub c: BigInt,   // third coefficient (derived: c = (b² - D) / 4a)
}
```

The **identity element** is $(1, 1, (1 - D)/4)$ and the canonical **generator** is
$(2, 1, (1 - D)/8)$ — both constructible without knowing the group order.

### 4.3 Discriminant Generation

```
seed bytes  ──SHA256 loop──▶  candidate prime p  ──▶  D = -p
```

Chia's `HashPrime` algorithm (`hash_prime` in `chia.rs`):

1. Treat `seed` as a big-endian counter; increment it before each hash round.
2. Concatenate SHA-256 outputs until `length_bits / 8` bytes are collected.
3. Force bits at indices `[0, 1, 2, length_bits - 1]` to 1, ensuring:
   - The number is odd (`bit 0 = 1`)
   - $p \equiv 7 \pmod 8$ (bits 0, 1, 2 all set) — needed for the generator to exist
   - The number has exactly `length_bits` bits (`MSB = 1`)
4. Run Miller-Rabin primality test (25 rounds). If composite, loop.

The same seed always produces the same $D$ — it is a deterministic, pseudorandom prime.

### 4.4 Shanks' NUCOMP and NUDUPL

Naïve form composition (Gauss) runs in $O((\log D)^2)$. Shanks' algorithms
(`NUCOMP` for multiplication, `NUDUPL` for squaring) reduce this to
$O((\log D)^{3/2})$ by terminating the extended GCD early once the remainder
drops below $|D|^{1/4}$.

**NUDUPL (squaring) — key steps in `math.rs`:**

```
1.  Partial XGCD of (a, b) until remainder < |D|^(1/4)  →  (s, t, r0, r1)
2.  Compute the new 'a' coefficient:  a' = (a/s)² / gcd(...)
3.  Compute the new 'b' coefficient from the partial quotients
4.  Gauss-reduce (a', b', c') into canonical reduced form
```

**NUCOMP (composition of two distinct forms) — key steps:**

```
1.  gcd(a1, a2, (b1+b2)/2) with Bezout coefficients
2.  Partial XGCD on the normalized inputs
3.  Construct new (a, b) from the composition matrix
4.  Gauss-reduce into reduced form
```

Both algorithms are used by `Form::pow` (binary exponentiation) to evaluate
$x^e$ for any exponent $e$:

```rust
pub fn pow(&self, exp: &BigUint, d: &BigInt) -> Form {
    // Standard binary (double-and-add) exponentiation
    // Each bit of `exp` → one NUDUPL + optionally one NUCOMP
    // Result is always in reduced form
}
```

### 4.5 The Wesolowski Verification Equation

Given:
- $x$ — generator form (deterministic from the challenge seed)
- $y$ — claimed output: $y = x^{2^T}$ (computed by the prover)
- $\pi$ — the Wesolowski proof form
- $T$ — number of sequential squarings

**Verification checks:**

$$\pi^B \cdot x^r = y$$

where:
- $B = \text{HashPrime}(\text{ser}(x) \| \text{ser}(y),\; 264)$ — a 264-bit Fiat-Shamir prime, bound to the specific $(x, y)$ pair
- $r = 2^T \bmod B$ — the remainder term

**Why this works (Wesolowski soundness):**

The prover knows $B$ only after fixing $y$ (since $B$ depends on $y$). To cheat, the
prover would need to find a $\pi$ satisfying the equation without computing $x^{2^T}$
— which requires solving the discrete logarithm in the class group modulo a 264-bit
prime. The probability of success is $\leq 1/B \approx 2^{-264}$.

**Why verification is $O(\log T)$:**

Both `proof.pow(&b, d)` and `x.pow(&r, d)` are exponentiation with exponents bounded
by $B$ (264 bits) and $r < B$ (264 bits) respectively. The number of NUDUPL operations
is $O(\log B) = O(264)$ — completely independent of $T$.

```text
T = 100         verify ≈  128 ms  ← same
T = 100,000     verify ≈  128 ms  ← same
T = 5,000,000   verify ≈  ~130 ms ← same
```

### 4.6 BQFC Serialization

Chia's Binary Quadratic Form Compression (BQFC) encodes a 1024-bit form into exactly
**100 bytes** using partial XGCD decomposition of the $(a, b)$ coefficients.

**Wire format (100 bytes):**

```
Byte 0    : flags (b_sign, t_sign, is_identity, is_generator)
Byte 1    : g_size (byte-length of g coefficient minus 1)
Bytes 2…  : little-endian packed fields (a_part, t_part, g, b0)
```

Decompression reconstructs $b$ via:
1. Modular inverse of $t$ modulo $a$
2. Square root recovery of $t^2 \cdot D \bmod a$
3. Reconstruction of full $b$ from quotient terms

The `deserialize_form` function validates the result by checking:
- $b^2 - 4ac = D$ (discriminant identity)
- The form is in reduced normal form

---

## 5. Codebase Map

```
kyn-vdf/
├── src/
│   ├── lib.rs          — Public API: verify_chia_vdf, KynVdfVerifier
│   ├── error.rs        — KynVdfError enum (all failure modes)
│   └── math.rs         — Form struct, NUCOMP, NUDUPL, Gauss reduction, pow
│   └── chia.rs         — HashPrime, create_discriminant, BQFC, verify_wesolowski
│
├── tests/
│   ├── test_vectors.rs  — Chia compatibility: known challenge → known output
│   ├── edge_cases.rs    — Boundary conditions and error path coverage
│   └── proptest_math.rs — Property-based group axiom verification
│
├── fuzz/
│   └── fuzz_targets/fuzz_vdf.rs  — libFuzzer harness for deserialization
│
├── benches/
│   └── vdf_benchmark.rs — Criterion benchmarks (class group ops + verify)
│
├── examples/
│   ├── hardware_bench.rs  — Full end-to-end timing vs chiavdf
│   └── extended_bench.rs  — High-iteration (T ≥ 1M) verify timing
│
└── .github/workflows/
    └── release.yml     — CI: test → build WASM → publish GitHub Release
```

---

## 6. Design Decisions

### Why `num-bigint` over custom big-integer arithmetic?

Custom big-integer code is one of the highest-risk areas in cryptographic software.
`num-bigint` is a mature, well-audited crate used across the Rust ecosystem. The
performance cost over libgmp is ~5× for verification — acceptable given the WASM
compatibility it enables.

### Why verifier-only?

VDF *proving* requires $T$ sequential squarings — that is the delay. Running $T = 500{,}000$
squarings in pure Rust on `num-bigint` would take significantly longer than the chiavdf
C++ prover (itself already takes 3.58 seconds). Proving belongs on validator hardware
with native C++ and libgmp. Verification belongs everywhere else.

### Why 1024-bit discriminant?

Chia Network uses 1024-bit discriminants. Smaller sizes are faster but weaker;
larger sizes are slower. 1024 bits gives ~112 bits of security for the underlying
group discrete logarithm problem. `create_discriminant` is parameterized so any
multiple-of-8 bit size works.

### Why 264-bit Fiat-Shamir prime $B$?

Wesolowski soundness error is $1/B$. At 264 bits, this is $\approx 2^{-264}$,
providing ~128 bits of security for the soundness argument. The prime size is
fixed by the Chia protocol specification.

### Why `cdylib` + `rlib` crate types?

- `rlib` — standard Rust library linking (used when `kyn-vdf` is a dependency)
- `cdylib` — produces `kyn_vdf.wasm` when targeting `wasm32-unknown-unknown`

Both are needed simultaneously to support both use cases from one crate.

---

## 7. What This Is Not

| Capability | Status |
|---|---|
| VDF proof generation (proving) | ❌ Not implemented — use `chiavdf` |
| `wasm-bindgen` JS exports | ✅ Implemented via `wasm.rs` (`verifyChiaVdf`) |
| `no_std` support | ⚠️ Partial — `num-bigint` requires `alloc` |
| Non-Chia VDF schemes (Pietrzak, RSA) | ❌ Not implemented |
| Parallel/GPU proving acceleration | ❌ Out of scope |
| Formal verification | ❌ Not done — property tested only |

---

*Built by [Saif Mukhtar](https://github.com/saifmukhtar) for the Kinetic network.*
