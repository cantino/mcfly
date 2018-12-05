# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.2.4'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "72c87370d6f389d25be4b16390b54068a104615a07fe0732cc5a73242551dfae"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "7ef36796d995ac73ec92a8b6cf11634149a2577619dc8ebdc28613cfdf9b3ae3"
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
