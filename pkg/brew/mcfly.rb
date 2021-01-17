# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.5.3'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "151b36e0b3a357718f06c65317545f9b186aa8b47e512dfef3757a5732dd6487"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "11a1112e73c66b559b37f7a8ce8f6465d56f2506af6c816386e22525aa54b883"
  end

  def install
    prefix.install "mcfly.bash"
    prefix.install "mcfly.zsh"
    prefix.install "mcfly.fish"
    bin.install "mcfly"
  end

  def caveats
    <<~EOS
      ONE MORE STEP!

      Add the following to the end of your ~/.bashrc, ~/.zshrc, or ~/.config/fish/config.fish file,
      as appropriate, changing /usr/local to your 'brew --prefix' if needed:

      Bash:
        if [ -r /usr/local/opt/mcfly/mcfly.bash ]; then
          . /usr/local/opt/mcfly/mcfly.bash
        fi

      Zsh:
        if [ -r /usr/local/opt/mcfly/mcfly.zsh ]; then
          . /usr/local/opt/mcfly/mcfly.zsh
        fi

      Fish:
        if test -r /usr/local/opt/mcfly/mcfly.fish
          source /usr/local/opt/mcfly/mcfly.fish
          mcfly_key_bindings
        end
    EOS
  end
end
