# To install:
#   brew tap cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.8.3'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "b549e9d89002b17d73f83951f3e10ae4b26fa95c876f68c1d2d766c953093e32"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "2feb0eb924a996832825168e323574351e8839846f2d54e72aed2efd80ba3617"
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
