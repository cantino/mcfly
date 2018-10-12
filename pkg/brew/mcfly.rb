# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.1.2'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
      url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "1ea16a88a146f010db22100bf7b1cf32e894c203b9f3089773840347d6dd6ba3"
  elsif OS.linux?
      url "https://github.com/cantino/mcfly/releases/download/#{version}/YYYY-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "721b3d2d283cc91faf5ab94e09b62a3888e298f0d801f89fb99e1ca1c7183cba"
  end

  def install
    prefix.install "mcfly-bash.sh"
    bin.install "mcfly"
  end
end
