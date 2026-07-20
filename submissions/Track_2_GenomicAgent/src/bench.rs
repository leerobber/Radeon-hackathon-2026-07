use std::time::Instant;
use crate::tools::{Tool, VcfAnalyzerTool, LdBlockTool, HaplotypeToolTool};
use crate::agent::GenomicAgent;
use crate::tools::ToolRegistry;

/// Real benchmarks: every number below is measured with Instant::now()/
/// elapsed() around actual execution, not printed as a literal. A
/// previous version of this file hardcoded every number here (including
/// a "GPU optimization potential: 3-4x speedup via vLLM" line with zero
/// vLLM or GPU code anywhere in the crate) and never read the one
/// Instant it did create. That's been removed rather than "corrected"
/// with different invented numbers -- this crate has no GPU/ROCm code,
/// so it makes no GPU performance claim at all.
pub fn run_benchmarks() -> anyhow::Result<()> {
    println!("\n======================================================================");
    println!("GENOMIC AGENT BENCHMARKS (all numbers measured, not estimated)");
    println!("======================================================================\n");

    let (vcf_ms, vcf_snps) = benchmark_vcf_analyzer()?;
    let (ld_ms, ld_pairs) = benchmark_ld_block()?;
    let (hap_ms, _) = benchmark_haplotype()?;
    let pipeline_ms = benchmark_pipeline()?;

    println!("\n======================================================================");
    println!("PERFORMANCE SUMMARY (measured on this machine, this run)");
    println!("======================================================================");
    println!(
        "VCF parsing + stats:   {:.3}ms for {} SNPs ({:.0} SNPs/sec)",
        vcf_ms, vcf_snps, vcf_snps as f64 / (vcf_ms / 1000.0).max(1e-9)
    );
    println!(
        "LD computation:        {:.3}ms for {} pairs tested ({:.0} pairs/sec)",
        ld_ms, ld_pairs, ld_pairs as f64 / (ld_ms / 1000.0).max(1e-9)
    );
    println!("Haplotype tallying:    {:.3}ms", hap_ms);
    println!("Full 3-query pipeline: {:.3}ms", pipeline_ms);
    println!(
        "\nNo GPU/ROCm acceleration is implemented in this crate (Cargo.toml has one \
         dependency: anyhow). Prior benchmark output claiming vLLM/GPU speedup did not \
         correspond to any code in this repository and has been removed rather than replaced \
         with a different unverified number.\n"
    );

    Ok(())
}

fn benchmark_vcf_analyzer() -> anyhow::Result<(f64, usize)> {
    println!("1. VcfAnalyzer");
    println!("   {}", "-".repeat(40));
    let start = Instant::now();
    let result = VcfAnalyzerTool.execute("bench")?;
    let elapsed = start.elapsed();
    println!("  {}", result.lines().next().unwrap_or(""));
    println!("  Wall time: {:.3}ms\n", elapsed.as_secs_f64() * 1000.0);
    // 400 SNPs is the fixed synthetic dataset size used by load_dataset() in tools.rs
    Ok((elapsed.as_secs_f64() * 1000.0, 400))
}

fn benchmark_ld_block() -> anyhow::Result<(f64, u64)> {
    println!("2. LdBlock");
    println!("   {}", "-".repeat(40));
    let start = Instant::now();
    let result = LdBlockTool.execute("bench")?;
    let elapsed = start.elapsed();
    println!("  {}", result.lines().next().unwrap_or(""));
    println!("  Wall time: {:.3}ms\n", elapsed.as_secs_f64() * 1000.0);
    // window=30 over 400 SNPs, matching LdBlockTool::execute's WINDOW constant
    let pairs: u64 = (0..400u64).map(|i| 30u64.min(400 - i - 1)).sum();
    Ok((elapsed.as_secs_f64() * 1000.0, pairs))
}

fn benchmark_haplotype() -> anyhow::Result<(f64, usize)> {
    println!("3. HaplotypeTool");
    println!("   {}", "-".repeat(40));
    let start = Instant::now();
    let result = HaplotypeToolTool.execute("bench")?;
    let elapsed = start.elapsed();
    println!("  {}", result.lines().next().unwrap_or(""));
    println!("  Wall time: {:.3}ms\n", elapsed.as_secs_f64() * 1000.0);
    Ok((elapsed.as_secs_f64() * 1000.0, 0))
}

fn benchmark_pipeline() -> anyhow::Result<f64> {
    println!("4. Full Agent Pipeline (3 queries, real routing + real tool execution)");
    println!("   {}", "-".repeat(40));

    let mut registry = ToolRegistry::new();
    registry.register(Box::new(VcfAnalyzerTool));
    registry.register(Box::new(LdBlockTool));
    registry.register(Box::new(HaplotypeToolTool));
    let mut agent = GenomicAgent::new(registry);

    let queries = [
        "Analyze the VCF file and tell me about SNP distribution",
        "What are the linkage disequilibrium blocks in this region?",
        "Find haplotype patterns for variants with MAF > 0.05",
    ];

    let start = Instant::now();
    for (i, q) in queries.iter().enumerate() {
        let q_start = Instant::now();
        agent.process_query(q)?;
        println!("  Query {}: {:.3}ms", i + 1, q_start.elapsed().as_secs_f64() * 1000.0);
    }
    let total = start.elapsed().as_secs_f64() * 1000.0;
    println!("  Total: {:.3}ms ({:.3}ms average)\n", total, total / queries.len() as f64);

    Ok(total)
}
