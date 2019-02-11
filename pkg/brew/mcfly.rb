# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.3.3'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "a392323a7f1798154df5fab333406362eab1570acea6576cba7f3672d7253488"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "d8a5247a1540fa05db24402efab934393148169790b083f7ed8e8fd67f5fc428"
  end

  def install
    prefix.install "mcfly.bash"
    bin.install "mcfly"
  end

  def caveats
    <<~EOS
      ONE MORE STEP! Edit ~/.bashrc and add the following at the end:

      if [ -r $(brew --prefix)/opt/mcfly/mcfly.bash ]; then
        . $(brew --prefix)/opt/mcfly/mcfly.bash
      fi
    EOS
  end
end
