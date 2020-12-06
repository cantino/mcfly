# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.5.0'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "c52348b89d68b1053f0fd1e3ff7ca0b91b38a1a281c941fd5bc30b0821eff193"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "d1cac8b7d7fc07e32ab6f33f10525bfd3d7dcd255bed469e73ef21a6c6b1a08e"
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
