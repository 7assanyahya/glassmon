# Packaging & Homebrew Tap Setup

This directory contains distribution artifacts and packaging templates for `glassmon`.

---

## Setting Up a Homebrew Tap

1. **Create the Tap Repository on GitHub:**
   Create a public repository named `homebrew-tap` under your GitHub account (`7assanyahya/homebrew-tap`).

2. **Copy the Formula:**
   Copy `packaging/homebrew/glassmon.rb` to `Formula/glassmon.rb` in your `homebrew-tap` repository:
   ```bash
   mkdir -p Formula
   cp packaging/homebrew/glassmon.rb Formula/glassmon.rb
   ```

3. **Update Release Checksums:**
   After building a release tag `v0.1.0`, calculate the SHA-256 hashes of the release archives and replace the placeholders in `Formula/glassmon.rb`:
   ```bash
   shasum -a 256 glassmon-*.tar.gz
   ```

4. **Install via the Tap:**
   ```bash
   brew tap 7assanyahya/tap
   brew install glassmon
   ```
