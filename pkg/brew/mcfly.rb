# To install:
#   brew tap cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.7.1'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "2d62398984b908ca06403f6ed9198702e8e193799fef57c9b7d38e69b7b4a33d"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "76c54d1b320aaf39f2f65e628615e3eaa2a56f94dce8547e542b7ec94a3c9346"
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

      You will need to restart your terminal when first installing and on some updates. If you receive a McFly error when running commands, try restarting your terminal.
    EOS
  end
end
