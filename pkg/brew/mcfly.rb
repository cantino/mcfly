# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.1.1'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
      url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "cfff6282af88b1a667ff97925507f7a364b493471470579da86844d4b5696a91"
  elsif OS.linux?
      url "https://github.com/cantino/mcfly/releases/download/#{version}/YYYY-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "77d657731a2d69a6b65573eed9f5fba909830e00532b1b44f226562bfacd3aff"
  end

  def install
    prefix.install "mcfly-bash.sh"
    bin.install "mcfly"
  end
end
