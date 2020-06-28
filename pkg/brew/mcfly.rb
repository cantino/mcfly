# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.4.0'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "170d7c719e2560cf5bf3e6c1266bca4549a1a0bc9db9d671595aebd4011551a4"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "714fa30d12794f06afea47ea19c1e98745e13995cfafb42d4f6b6aee1c0572a2"
  end

  def install
    prefix.install "mcfly.bash"
    prefix.install "mcfly.zsh"
    bin.install "mcfly"
  end

  def caveats
    <<~EOS
      ONE MORE STEP!

      If you use Bash, edit ~/.bashrc and add the following at the end:
        if [ -r $(brew --prefix)/opt/mcfly/mcfly.bash ]; then
          . $(brew --prefix)/opt/mcfly/mcfly.bash
        fi

      If you use Zsh, edit ~/.zshrc and add the following at the end:
        if [ -r $(brew --prefix)/opt/mcfly/mcfly.zsh ]; then
          . $(brew --prefix)/opt/mcfly/mcfly.zsh
        fi
    EOS
  end
end
