# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.5.6'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "de0f830ed5432b092c7d10b5c587055493373dff9f6f0ac2b0692693ed0ab58b"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "aed3f23d4a307bd21c2346dc63f48972b49acf96fb3afa9b9e63734081f21a76"
  end

  def install
    bin.install "mcfly"
  end

  def caveats
    <<~EOS
      ONE MORE STEP!

      Add the following to the end of your ~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish file.

      Bash:
        eval "$(mcfly init bash)"

      Zsh:
        eval "$(mcfly init zsh)"

      Fish:
        mcfly init fish | source
    EOS
  end
end
