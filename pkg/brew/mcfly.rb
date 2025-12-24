# To install:
#   brew tap cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.9.4'
  deprecate! date: "2024-05-18", because: "is now in the core homebrew repository and you don't need this tap"
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "5ab584300dc6405a4730feb08f44ac4a7cd4a3308a1f9dc28813aa2d36782c7f"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "72d2c6fdaa111ac96c2cf725fc40e313e2856643482be58608911a09440313f1"
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
