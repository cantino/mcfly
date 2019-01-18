# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.3.1'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "c39e67f4d14edc0eb6539c699f223a3c69853dd39fece2ac45aa7d25ec1726ee"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "614a88956d3341acc1b6f339451b68f8c9eb30197aa2bc3d9dfd4e6e560ea93f"
  end

  def install
    prefix.install "mcfly.bash"
    bin.install "mcfly"
  end

  def caveats
    <<~EOS
      ONE MORE STEP! Edit ~/.bashrc and add the following:

      if [ -r $(brew --prefix)/opt/mcfly/mcfly.bash ]; then
        . $(brew --prefix)/opt/mcfly/mcfly.bash
      fi
    EOS
  end
end
