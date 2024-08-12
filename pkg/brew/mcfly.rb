# To install:
#   brew tap cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.9.2'
  deprecate! date: "2024-05-18", because: "is now in the core homebrew repository and you don't need this tap"
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "638a0406caf5f6607b31acbccad20831cc8af2f2dd4a2e614a3b218df2996180"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "f907b88861464b804860dcacf990c87cdbd4e7af08f6e854a86c35bb89b92c55"
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
