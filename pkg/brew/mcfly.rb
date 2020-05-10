# To install:
#   brew tap cantino/mcfly https://github.com/cantino/mcfly
#   brew install mcfly
#
# To remove:
#   brew uninstall mcfly
#   brew untap cantino/mcfly

class Mcfly < Formula
  version 'v0.3.6'
  desc "McFly"
  homepage "https://github.com/cantino/mcfly"

  if OS.mac?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "5ef778a66deb713f17f8128857d214cbbde287701b5902b06b42e215681e322d"
  elsif OS.linux?
    url "https://github.com/cantino/mcfly/releases/download/#{version}/mcfly-#{version}-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "29290213ecdce3f194d758500f17d0f932cdd74fb8762cd01c32bc4435de6bde"
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
