# Genomic Research Agent — Quick Start Guide

**Status:** Builds and runs, real computation, CPU-only. No GPU/ROCm code
is implemented in this crate (see "GPU/ROCm status" below before
assuming otherwise).

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

## 5. GPU/ROCm status

**This crate has no GPU or ROCm code.** `Cargo.toml` has one dependency
(`anyhow`). `setup.sh` checks for `rocm-smi` and mentions a
`RADEON_API_KEY` environment variable, but nothing in `src/` reads that
variable or calls any GPU API — those are leftover references from an
earlier, more ambitious plan for this submission that wasn't built. An
earlier version of this file and the PR description both described a
"Radeon Cloud" walkthrough and claimed points for GPU/ROCm optimization;
running the code on a GPU cloud instance would produce identical
CPU-only output to running it locally, because there's no code path
that would engage a GPU either way. That claim has been removed rather
than left in place.

If GPU acceleration gets added later, the natural target is
`LdBlockTool`'s pairwise r² computation — it's embarrassingly parallel
(independent SNP pairs) and would be a reasonable fit for a compute
shader or ROCm/HIP kernel. Not implemented as of this writing.

---

## 6. File Structure

```
Track_2_GenomicAgent/
├── Cargo.toml
├── src/
│   ├── main.rs      # Entry point (default / bench / fast modes)
│   ├── agent.rs      # Keyword-based query routing to a tool
│   ├── tools.rs       # 3 genomic tools, real computation (see vcf.rs)
│   ├── vcf.rs          # Synthetic VCF generation + real VCF-format parser
│   │                     + real MAF/missingness/LD-r²/haplotype computation
│   └── bench.rs         # Real timing (Instant::now/elapsed) around real execution
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
