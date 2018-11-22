# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.2.0'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "03f4a95b55e6b042ef34cc16703362dacfd15ef514c9edcdbf1b4bb9f2391610"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "4056bb6df54f75eaa16869faaa3d6b32c6fe540e7e5d489a9ea4538c7bedac19"
  end

  def install
    prefix.install "mcfly-bash.sh"
    bin.install "mcfly"
    ohai "To finish installing mcfly, add the following to your ~/.bash_profile (OS X) or ~/.bashrc (Linux) file:
if [ -f $(brew --prefix)/opt/mcfly/mcfly-bash.sh ]; then
  . $(brew --prefix)/opt/mcfly/mcfly-bash.sh
fi".strip
  end
end
