class Glassmon < Formula
  desc "Fast, good-looking terminal system monitor for macOS and Linux (Liquid Glass UI)"
  homepage "https://github.com/7assanyahya/glassmon"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/7assanyahya/glassmon/releases/download/v0.1.0/glassmon-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_AARCH64_APPLE_DARWIN_SHA256"
    else
      url "https://github.com/7assanyahya/glassmon/releases/download/v0.1.0/glassmon-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_X86_64_APPLE_DARWIN_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/7assanyahya/glassmon/releases/download/v0.1.0/glassmon-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_AARCH64_LINUX_SHA256"
    else
      url "https://github.com/7assanyahya/glassmon/releases/download/v0.1.0/glassmon-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_X86_64_LINUX_SHA256"
    end
  end

  def install
    bin.install "glassmon"
  end

  test do
    assert_match "glassmon", shell_output("#{bin}/glassmon --version")
  end
end
