# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.2.2'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "66c17640f457e10e42ae8ba86526e0f95e173241cefdd1a18a478dfa44ac0ea2"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "1a203c4fb3d3267e825b60bb5711c3e4ec102465ed8641ad1edff7d002ed99e1"
  end

  def install
    prefix.install "mcfly.bash"
    bin.install "mcfly"
    ohai "ONE MORE STEP! Edit ~/.bashrc and add the following:"
    puts
    puts "if [ -f $(brew --prefix)/opt/mcfly/mcfly.bash ]; then"
    puts "  . $(brew --prefix)/opt/mcfly/mcfly.bash"
    puts "fi"
  end
end
