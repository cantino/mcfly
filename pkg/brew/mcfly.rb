# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.5.2'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "f5bed733f7fccb639a913a75c5007a303355216a9caf71a01eb9f7774f4ef861"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "fa4d26af3cb3e79293a42f55631e78c5784fbb54b88c828e3a53379cae086913"
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
