# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.3.5'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "1bcd0024062eee14be03e2a6e096866adccfd31ee984571983a8af1b80766bd2"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "7369e080bf0d0c29754750f96ffee3df8f3a7d3ee593fcf93c7e77e2d12b24b8"
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
