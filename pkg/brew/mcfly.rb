# To install:
#   brew tap cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.5.13'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "1a792ac11307549953c631da8cab99f96a3d06e1cee6abd5ff71cfefbed683e0"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "3051995d5e2ac6a2fcfea37e801caee01ce6c4b8fbda3bd4c61fc82e31c4a764"
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
