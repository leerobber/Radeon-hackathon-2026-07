# Radeon Cloud GPU Testing Guide

**Objective:** Run benchmarks on AMD Radeon GPU and verify 4-6x speedup

---

## Step 1: Create Radeon Cloud Account

1. Go to **https://radeon-global.anruicloud.com/**
2. Click **Sign up** (or **Login** if you have an account)
3. Use email: `leer4030@gmail.com`
4. Create password

---

## Step 2: Add Your SSH Public Key

**Your SSH Public Key:**
```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIIf/yAdplysgG2X1XrYyLSV8KWYsYhrQo7yz89dCTIbJ leer4030@gmail.com
```

**Steps:**
1. Log in to Radeon Cloud
2. Click your **avatar** (top-right) → **Profile**
3. Find **SSH Public Key** section
4. Paste the public key above
5. Click **Save Key**

✅ SSH key is now registered

---

## Step 3: Create a Radeon GPU Template

1. In Radeon Cloud, click **Profile** → **Add Template**
2. Fill in the form:
   - **Title:** `Genomic Agent GPU`
   - **Container Image:** `rocm/pytorch:latest` (or similar ROCm image)
   - **SSH Access:** Toggle **ON**
3. Click **Add Template**

---

## Step 4: Launch GPU Instance

1. Go back to **My Templates**
2. Find your template "Genomic Agent GPU"
3. Click **Launch**
4. Wait for status to show **"Your workspace is ready (100%)"**
5. Note the SSH connection info:
   - Host
   - Port
   - Username

Example:
```
SSH access
user@instance.anruicloud.com -p 22
```

---

## Step 5: SSH Into Instance

**In your local terminal (PowerShell/Bash):**

```bash
ssh user@host -p port
```

Replace `user`, `host`, and `port` with values from Step 4.

When prompted: "Are you sure you want to continue connecting?" → Type `yes`

---

## Step 6: Clone and Setup Project

**Inside the Radeon instance:**

```bash
# Clone your fork
git clone https://github.com/leerobber/Radeon-hackathon-2026-07.git
cd Radeon-hackathon-2026-07/submissions/Track_2_GenomicAgent

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Build project
bash setup.sh

# Run benchmarks
cargo run --release -- bench
```

---

## Step 7: Expected Results

**Benchmark Output (CPU baseline):**
```
Full Agent Pipeline: 140.4ms average
  - Tool execution: 2-5ms
  - LLM inference: 130-135ms
  - Response parsing: 5ms
```

**With Radeon GPU (vLLM):**
Expected to see **4-6x speedup** on LLM inference:
```
Full Agent Pipeline: 35-50ms average
  - Tool execution: 2-5ms (unchanged)
  - LLM inference: 30-40ms (4-6x faster)
  - Response parsing: 5ms (unchanged)
```

---

## Step 8 (Optional): Deploy vLLM for Custom Model

For even better optimization with Llama-7B:

**Create a second template:**
1. Profile → Add Template
2. **Deploy Type:** vLLM Model API
3. **Serve Command:**
   ```bash
   vllm serve meta-llama/Llama-2-7b-chat-hf \
     --host 0.0.0.0 --port 8000 \
     --quantization awq --dtype float16
   ```
4. Launch this instance
5. Update your agent to use this endpoint

---

## Troubleshooting

**"Connection refused"**
- Instance still starting (wait 2-3 minutes)
- Check SSH key is saved in Profile

**"SSH key permission denied"**
- Ensure you pasted the `.pub` file (not the private key)
- Check file permissions: `ls -la ~/.ssh/`

**"Rust not found"**
- Run setup script: `bash setup.sh`
- Or install manually: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

**"GPU not detected"**
- Run: `rocm-smi` to verify ROCm installation
- Benchmarks run on CPU if GPU unavailable (fallback)

---

## Cleanup

When done testing:

1. In Radeon Cloud, go to **Profile → Active Instance**
2. Click red **Destroy Instance** button
3. Confirm

This stops billing your free credits.

---

## Performance Expectations

| Component | CPU | GPU (Radeon) | Speedup |
|-----------|-----|--------------|---------|
| VCF Analysis | 13ms | 12ms | 1.1x |
| LD Computation | 15ms | 14ms | 1.1x |
| Haplotype | 16ms | 15ms | 1.1x |
| LLM Inference | 130ms | 30-40ms | **4-6x** |
| Full Pipeline | 140ms | 35-50ms | **3-4x** |

LLM inference is the bottleneck (95% of latency), so GPU optimization has maximum impact there.

---

## Contact Support

- **Radeon Cloud Help:** https://help.radeon.cloud
- **AMD Support:** ai_dev_contests@amd.com
- **Discord:** https://discord.gg/zt9caur5B3

---

**Ready to test? Launch your instance above!** 🚀
