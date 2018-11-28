# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.2.1'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "ace681e5bf129f31a3242061fa43d53c9506aea19658e32dd8ee14e57247f214"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "b2049ac3337189fe2e0c1abae71e917bb617fdefe5818f2297a47f13bbe5d653"
  end

  def install
    prefix.install "mcfly.bash"
    bin.install "mcfly"
    ohai "NOTE> Now you need to edit ~/.bashrc and add the following:"
    puts
    puts "if [[ -f \"$(brew --prefix)/opt/mcfly/mcfly.bash\" ]]; then"
    puts "  source \"$(brew --prefix)/opt/mcfly/mcfly.bash\""
    puts "fi"
  end
end
