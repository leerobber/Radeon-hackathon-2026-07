use std::collections::HashMap;
use crate::vcf::{self, VcfData};

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, query: &str) -> anyhow::Result<String>;
}

/// Shared synthetic dataset generation. Same seed/size across tools so
/// they're analyzing the same (clearly synthetic) cohort. Previously
/// each tool ignored its input and returned a hardcoded canned string;
/// this generates real VCF text and parses it with the real parser in
/// `vcf.rs` every time a tool executes.
fn load_dataset() -> anyhow::Result<VcfData> {
    let text = vcf::generate_synthetic_vcf(400, 40, 20260720);
    vcf::parse_vcf(&text)
}

pub struct VcfAnalyzerTool;

impl Tool for VcfAnalyzerTool {
    fn name(&self) -> &str {
        "VcfAnalyzer"
    }

    fn description(&self) -> &str {
        "VcfAnalyzer: Parse VCF files and compute SNP statistics (count, MAF, missingness). Use for understanding variant distributions."
    }

    fn execute(&self, _query: &str) -> anyhow::Result<String> {
        let start = std::time::Instant::now();

        let data = load_dataset()?;
        let stats: Vec<_> = data.variants.iter().map(vcf::compute_variant_stats).collect();

        let total_snps = stats.len();
        let common_snps = stats.iter().filter(|s| s.maf > 0.05).count();
        let rare_snps = total_snps - common_snps;
        let avg_maf = stats.iter().map(|s| s.maf).sum::<f64>() / total_snps.max(1) as f64;
        let avg_missingness = stats.iter().map(|s| s.missingness).sum::<f64>() / total_snps.max(1) as f64;

        let elapsed = start.elapsed();

        let result = format!(
            "VCF Analysis Summary (synthetic dataset, {} samples):\n\
             - Total SNPs: {}\n\
             - Common SNPs (MAF > 0.05): {}\n\
             - Rare SNPs (MAF <= 0.05): {}\n\
             - Mean MAF: {:.3}\n\
             - Missing data: {:.2}%\n\
             - Processing time: {:.3}ms (measured)",
            data.sample_names.len(),
            total_snps,
            common_snps,
            rare_snps,
            avg_maf,
            avg_missingness * 100.0,
            elapsed.as_secs_f64() * 1000.0,
        );

        Ok(result)
    }
}

pub struct LdBlockTool;

impl Tool for LdBlockTool {
    fn name(&self) -> &str {
        "LdBlock"
    }

    fn description(&self) -> &str {
        "LdBlock: Identify linkage disequilibrium blocks and tag SNPs via real pairwise r² computation. Use for understanding genetic structure and variant independence."
    }

    fn execute(&self, _query: &str) -> anyhow::Result<String> {
        let start = std::time::Instant::now();

        let data = load_dataset()?;
        const R2_THRESHOLD: f64 = 0.8;
        const WINDOW: usize = 30; // only test nearby SNPs -- O(n*window), not O(n^2)

        // Real pairwise LD: union-find blocks of SNPs connected by r² > threshold.
        let n = data.variants.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(parent: &mut [usize], x: usize) -> usize {
            if parent[x] != x {
                parent[x] = find(parent, parent[x]);
            }
            parent[x]
        }

        let mut pairs_tested = 0u64;
        let mut r2_values_in_window = Vec::new();

        for i in 0..n {
            for j in (i + 1)..(i + WINDOW).min(n) {
                pairs_tested += 1;
                if let Some(r2) = vcf::compute_r_squared(&data.variants[i], &data.variants[j]) {
                    r2_values_in_window.push(r2);
                    if r2 > R2_THRESHOLD {
                        let ri = find(&mut parent, i);
                        let rj = find(&mut parent, j);
                        if ri != rj {
                            parent[ri] = rj;
                        }
                    }
                }
            }
        }

        let mut block_members: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            block_members.entry(root).or_default().push(i);
        }
        let mut blocks: Vec<&Vec<usize>> = block_members.values().filter(|m| m.len() > 1).collect();
        blocks.sort_by_key(|m| std::cmp::Reverse(m.len()));

        let mean_r2 = if r2_values_in_window.is_empty() {
            0.0
        } else {
            r2_values_in_window.iter().sum::<f64>() / r2_values_in_window.len() as f64
        };

        let elapsed = start.elapsed();

        let mut result = format!(
            "Linkage Disequilibrium Analysis (synthetic dataset, real pairwise r², window={}):\n\n",
            WINDOW
        );
        for (idx, members) in blocks.iter().take(5).enumerate() {
            let first = &data.variants[*members.iter().min().unwrap()];
            let last = &data.variants[*members.iter().max().unwrap()];
            result.push_str(&format!(
                "{}. {}:{}-{}\n   SNPs in block: {}\n",
                idx + 1,
                first.chrom,
                first.pos,
                last.pos,
                members.len(),
            ));
        }
        result.push_str(&format!(
            "\nTotal LD blocks found (size > 1, r² > {}): {}\n\
             Pairs tested: {}\n\
             Mean r² across tested pairs: {:.3}\n\
             Processing time: {:.3}ms (measured)",
            R2_THRESHOLD,
            blocks.len(),
            pairs_tested,
            mean_r2,
            elapsed.as_secs_f64() * 1000.0,
        ));

        Ok(result)
    }
}

pub struct HaplotypeToolTool;

impl Tool for HaplotypeToolTool {
    fn name(&self) -> &str {
        "HaplotypeTool"
    }

    fn description(&self) -> &str {
        "HaplotypeTool: Tally observed haplotype patterns and frequencies from phased genotypes. Use for ancestry inference and population genetics."
    }

    fn execute(&self, _query: &str) -> anyhow::Result<String> {
        let start = std::time::Instant::now();

        let data = load_dataset()?;
        const WINDOW: usize = 4; // small window of adjacent SNPs to tally as a haplotype

        let window_end = WINDOW.min(data.variants.len());
        let mut counts: HashMap<String, u64> = HashMap::new();
        let mut total_haps = 0u64;

        for sample_idx in 0..data.sample_names.len() {
            for hap_arm in 0..2 {
                let mut alleles = String::with_capacity(window_end);
                let mut complete = true;
                for variant in &data.variants[0..window_end] {
                    match variant.genotypes.get(sample_idx).and_then(|g| *g) {
                        Some((a0, a1)) => {
                            let allele = if hap_arm == 0 { a0 } else { a1 };
                            alleles.push(if allele == 0 { '0' } else { '1' });
                        }
                        None => {
                            complete = false;
                            break;
                        }
                    }
                }
                if complete {
                    *counts.entry(alleles).or_insert(0) += 1;
                    total_haps += 1;
                }
            }
        }

        let mut ranked: Vec<(String, u64)> = counts.into_iter().collect();
        ranked.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

        let elapsed = start.elapsed();

        let mut result = format!(
            "Haplotype Patterns (synthetic dataset, {}-SNP window, {} phased haplotype observations):\n\n",
            window_end, total_haps
        );
        for (i, (pattern, count)) in ranked.iter().take(6).enumerate() {
            let freq = *count as f64 / total_haps.max(1) as f64;
            result.push_str(&format!(
                "{}. {} | Freq: {:.1}% | n={}\n",
                i + 1,
                pattern,
                freq * 100.0,
                count,
            ));
        }
        result.push_str(&format!(
            "\nDistinct haplotypes observed: {}\n\
             Processing time: {:.3}ms (measured)",
            ranked.len(),
            elapsed.as_secs_f64() * 1000.0,
        ));

        Ok(result)
    }
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn execute(&self, tool_name: &str, query: &str) -> anyhow::Result<String> {
        if let Some(tool) = self.tools.get(tool_name) {
            tool.execute(query)
        } else {
            Err(anyhow::anyhow!("Tool {} not found", tool_name))
        }
    }

    pub fn get_descriptions(&self) -> Vec<String> {
        self.tools
            .values()
            .map(|t| format!("{}: {}", t.name(), t.description()))
            .collect()
    }
}
