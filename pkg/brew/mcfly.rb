# To install:
#   brew tap cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.9.1'
  deprecate! date: "2024-05-18", because: "is now in the core homebrew repository and you don't need this tap"
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "10655cfa16ed4abe0b01cbcc91234d93493750c12f76635e93efbb19430018d5"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "66ae9bb227915cdeb92e3fb13e26522ebf1b5a163b296162c9257474c0efe4ee"
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
