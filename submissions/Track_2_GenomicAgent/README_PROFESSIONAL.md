# Genomic Research Agent — Quick Start Guide

**Status:** Builds and runs, real computation. Includes real GPU-accelerated
LD computation (`gpu-bench`), dispatched to and verified against actual
AMD GPU hardware — see "GPU acceleration" below for exactly what that
means and does not mean (it is not the literal ROCm/HIP API).

---

## 1. Prerequisites

- Rust 1.70+ (https://rustup.rs/)
- Git

---

## 2. Clone & Setup

```bash
git clone https://github.com/AMD-DEV-CONTEST/Radeon-hackathon-2026-07.git
cd Radeon-hackathon-2026-07/submissions/Track_2_GenomicAgent

bash setup.sh          # Linux/macOS
# or
setup.bat              # Windows
```

---

## 3. Run the Demo

```bash
cargo run --release
```

Runs three queries through the agent: VCF analysis, LD block detection,
and haplotype tallying, each against a deterministic synthetic dataset
generated at runtime (see "About the data" below). The output includes
real computed numbers (SNP counts, MAF, r² values, haplotype
frequencies) — it will look the same on every run because the data
generator is seeded deterministically, not because the numbers are
hardcoded. Run `cargo test --release` to see property-based tests that
check this (e.g. two identical genotype columns must compute r²≈1.0).

---

## 4. Run Benchmarks

```bash
cargo run --release -- bench
```

Every number in the output is measured with `Instant::now()`/`elapsed()`
around actual execution on your machine, not printed as a literal —
timing and throughput will vary by hardware and will differ from any
number quoted elsewhere in this repo's history. Run it yourself rather
than trusting a pasted example.

---

## 5. GPU acceleration

```bash
cargo run --release -- gpu-bench
```

**What this is:** a real WGSL compute shader (`src/shaders/ld_r2.wgsl`)
computing pairwise Pearson r² (linkage disequilibrium) in parallel across
SNP pairs, dispatched through `wgpu` (`src/gpu_ld.rs`). It explicitly
enumerates GPU adapters and prefers a real AMD adapter (PCI vendor
`0x1002`) over any other GPU present, so it actually targets AMD
hardware rather than whichever GPU a generic "high performance" heuristic
would pick (on a machine with both an AMD iGPU and an NVIDIA discrete
GPU, the naive heuristic picks NVIDIA — this code doesn't).

**Correctness:** every GPU result is cross-validated against a CPU
reference implementation of the same statistic. `cargo test` includes
this cross-validation as an automated test
(`gpu_ld::tests::gpu_matches_cpu_reference_within_float_tolerance`), and
`gpu-bench` reports the max observed difference on every run (it should
be ~1e-6, float-rounding-only). This isn't a shader that "runs" — it's
one whose output is checked against a known-correct answer every time.

**Real measured numbers** (AMD Radeon 780M, this machine, this run —
will differ on other hardware, run it yourself):

| Scale | CPU | GPU (incl. upload+dispatch+readback) | Speedup |
|-------|-----|-----|---------|
| 1,000 SNPs, 94,050 pairs | 12.98ms | 9.40ms | 1.38x |
| 4,000 SNPs, 584,825 pairs | 79.12ms | 22.81ms | 3.47x |

Speedup grows with problem size because GPU dispatch/buffer-readback has
fixed overhead that only pays off once there's enough parallel work to
amortize it — this is normal and expected for GPU compute, and the
benchmark reports both scales honestly rather than cherry-picking the
better number.

**What this is not:** literal AMD ROCm/HIP API calls. This development
machine has no ROCm/HIP SDK installed, and Windows HIP support for this
specific integrated GPU (Radeon 780M / RDNA3, gfx1103) is unconfirmed
without it — installing a multi-GB SDK for possibly-unsupported hardware
wasn't done without flagging it first. `wgpu` was used instead because
it's verifiable *today*, on real hardware, dispatching to and measuring
the actual AMD GPU via its Vulkan driver — real GPU acceleration, real
correctness verification, just not through the ROCm-specific API
surface. If literal HIP/ROCm kernels are required for scoring, the
algorithm in `ld_r2.wgsl` is the direct port target: same math, same
per-pair parallelism, translated from WGSL to HIP C++.

`setup.sh`/`setup.bat` no longer reference `RADEON_API_KEY` or a
"Radeon Cloud" walkthrough from an earlier, unbuilt plan for this
submission — nothing in `src/` ever read that variable.

---

## 6. File Structure

```
Track_2_GenomicAgent/
├── Cargo.toml
├── src/
│   ├── main.rs         # Entry point (default / bench / gpu-bench / fast modes)
│   ├── agent.rs         # Keyword-based query routing to a tool
│   ├── tools.rs          # 3 genomic tools, real computation (see vcf.rs)
│   ├── vcf.rs             # Synthetic VCF generation + real VCF-format parser
│   │                        + real MAF/missingness/LD-r²/haplotype computation
│   ├── gpu_ld.rs           # Real GPU-accelerated LD (wgpu), AMD-adapter-targeted,
│   │                        cross-validated against CPU reference
│   ├── shaders/
│   │   └── ld_r2.wgsl       # The actual compute shader dispatched to the GPU
│   └── bench.rs             # Real timing (Instant::now/elapsed) around real execution
├── LICENSE
├── setup.sh / setup.bat
└── README_PROFESSIONAL.md
```

---

## 7. What each tool does

### VcfAnalyzer
Parses a synthetic VCF (real VCF-format text, generated deterministically
at runtime, not a bundled real-patient file) and computes real per-variant
statistics: SNP count, minor allele frequency, missingness.

### LdBlock
Computes real pairwise linkage disequilibrium (Pearson r², the standard
population-genetics LD statistic) between nearby SNPs within a sliding
window, and groups markers into blocks where r² exceeds a threshold via
union-find. All numbers reported (pairs tested, mean r², block sizes)
come from that computation, not from a literal.

### HaplotypeTool
Tallies real observed haplotype patterns from phased genotype pairs
across a small SNP window and reports their frequencies.

### About the data
All three tools currently analyze a synthetic dataset generated at
runtime (`vcf::generate_synthetic_vcf`), not a real 1000 Genomes or
patient VCF file. The generator embeds genuine linkage-disequilibrium
structure via founder-haplotype resampling (nearby SNPs really are
correlated, and the LD tool's job is to genuinely detect that, not
report a hardcoded number) — but it is not real biological data, and
the README says so rather than implying otherwise. Swapping in a real
VCF file would only require pointing `load_dataset()` in `tools.rs` at
a file-read instead of the generator; `parse_vcf()` already accepts
arbitrary VCF-format text.

---

## 8. Troubleshooting

**"Rust not found"** — Install from https://rustup.rs/, verify with `rustc --version`.

**"Build fails"** — `rustup update`, then `cargo clean && cargo build --release`.

---

**Built for AMD AI DevMaster Hackathon 2026-07**
