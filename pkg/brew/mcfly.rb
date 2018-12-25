# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.2.5'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "7003950b55b14cbc6ad9e4f8eff8e6cc15642e924b994afcbced4468d8b144b9"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "58596d3e93de5e8437fddf757d3c048b2711f1447c099bf9cec2ff35266a0202"
  end

  def install
    prefix.install "mcfly.bash"
    bin.install "mcfly"
  end

  def caveats
    <<~EOS
      ONE MORE STEP! Edit ~/.bashrc and add the following:

      if [ -f $(brew --prefix)/opt/mcfly/mcfly.bash ]; then
        . $(brew --prefix)/opt/mcfly/mcfly.bash
      fi
    EOS
  end
end
