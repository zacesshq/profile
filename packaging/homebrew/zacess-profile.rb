# Homebrew formula for Zacess Profile 0.2.0, for Formula/zacess-profile.rb in the zacesshq/homebrew-tap
# repository. Each SHA256_ placeholder is the sum from the matching .sha256 file attached to the
# v0.2.0 release; packaging/fill-checksums.sh fills them in. The next release changes the version
# in the four URLs and the four sums.
class ZacessProfile < Formula
  desc "One accessibility profile for every app, checked for contrast and colour vision"
  homepage "https://zacess.com"
  version "0.2.0"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/zacesshq/profile/releases/download/v0.2.0/zacess-profile-v0.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_AARCH64_APPLE_DARWIN"
    end
    on_intel do
      url "https://github.com/zacesshq/profile/releases/download/v0.2.0/zacess-profile-v0.2.0-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_X86_64_APPLE_DARWIN"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/zacesshq/profile/releases/download/v0.2.0/zacess-profile-v0.2.0-aarch64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_AARCH64_UNKNOWN_LINUX_MUSL"
    end
    on_intel do
      url "https://github.com/zacesshq/profile/releases/download/v0.2.0/zacess-profile-v0.2.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "SHA256_X86_64_UNKNOWN_LINUX_MUSL"
    end
  end

  def install
    bin.install "zprf"
    pkgshare.install "profiles", "spec"
    doc.install "README.md", "CHANGELOG.md", "ACCESSIBILITY.md", "ROADMAP.md", "NOTICE"
  end

  def caveats
    <<~EOS
      The eight starter profiles are in:
        #{opt_pkgshare}/profiles
    EOS
  end

  test do
    assert_match "zprf #{version}", shell_output("#{bin}/zprf --version")
    assert_match "pass", shell_output("#{bin}/zprf check #{pkgshare}/profiles/zacess-dark.toml")
    system bin/"zprf", "render", pkgshare/"profiles/zacess-dark.toml", "--target", "all", "--out", testpath/"out"
    assert_path_exists testpath/"out/kitty/zacess-dark.conf"
  end
end
