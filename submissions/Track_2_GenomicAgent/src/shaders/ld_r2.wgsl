// Pairwise Pearson r^2 (linkage disequilibrium) compute shader.
//
// One GPU thread per SNP pair. Each thread walks the sample dimension to
// compute covariance, using precomputed per-SNP means/stds (computed once
// on CPU, O(n_snps * n_samples), not worth parallelizing). This is the
// same statistic as vcf::compute_r_squared (the CPU reference) --
// gpu_ld.rs cross-validates GPU output against that function before this
// kernel is trusted for anything.

struct Params {
    num_samples: u32,
    num_pairs: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> dosages: array<f32>;   // [snp_idx * num_samples + sample_idx]
@group(0) @binding(2) var<storage, read> means: array<f32>;     // [snp_idx]
@group(0) @binding(3) var<storage, read> stds: array<f32>;      // [snp_idx], population std (sqrt of sum of squared deviations)
@group(0) @binding(4) var<storage, read> pair_i: array<u32>;    // [pair_idx]
@group(0) @binding(5) var<storage, read> pair_j: array<u32>;    // [pair_idx]
@group(0) @binding(6) var<storage, read_write> out_r2: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let k = gid.x;
    if (k >= params.num_pairs) {
        return;
    }

    let i = pair_i[k];
    let j = pair_j[k];
    let ns = params.num_samples;
    let mean_i = means[i];
    let mean_j = means[j];
    let base_i = i * ns;
    let base_j = j * ns;

    var cov: f32 = 0.0;
    for (var s: u32 = 0u; s < ns; s = s + 1u) {
        let xi = dosages[base_i + s] - mean_i;
        let xj = dosages[base_j + s] - mean_j;
        cov = cov + xi * xj;
    }

    let denom = stds[i] * stds[j];
    if (denom <= 0.0) {
        out_r2[k] = 0.0;
    } else {
        let r = cov / denom;
        out_r2[k] = r * r;
    }
}
