# To install:
#   brew tap cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.8.6'
  deprecate! date: "2024-05-18", because: "mcfly is now in the core homebrew repository and you don't need this tap"
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "2863068a9800d8b71965c0ddadeefef88aad1e5c5cec6757ade741251fec348b"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "b8241970861b080807807b546bc7ac4905a5b64ae6cb56b8b69657e2e26c2c6f"
  end

  def install
    bin.install "mcfly"
  end

  def caveats
    <<~EOS
      DEPRECATED! mcfly is now in the core homebrew repository and you don't need this tap.
      Please run:

      brew uninstall mcfly
      brew untap cantino/mcfly
      brew install mcfly
    EOS
  end
end
