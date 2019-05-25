# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.3.4'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "4790fb4534c3a7b50c107f4f255a716aa9f8b1954f4563fcabf4575e2c62f921"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "4858b4ac47198485c3c276dc3434625c94867dd48d6cbc6ae402266c67559e05"
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
