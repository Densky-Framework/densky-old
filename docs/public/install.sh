#!/bin/sh
set -e

if [ "$OS" = Windows_NT ]; then
    echo 'error: Use the .ps1 file for Windows Powershell.'
    exit 1
fi

# Reset
Color_Off=''

# Regular Colors
Red=''
Green=''
Dim='' # White

# Bold
Bold_White=''
Bold_Green=''

if [ -t 1 ]; then
    # Reset
    Color_Off='\033[0m' # Text Reset

    # Regular Colors
    Red='\033[0;31m'   # Red
    Green='\033[0;32m' # Green
    Dim='\033[0;2m'    # White

    # Bold
    Bold_Green='\033[1;32m' # Bold Green
    Bold_White='\033[1m'    # Bold White
fi

error() {
    echo "${Red}error${Color_Off}:" "$@" >&2
    exit 1
}

info() {
    echo "${Dim}$@ ${Color_Off}"
}

info_bold() {
    echo "${Bold_White}$@ ${Color_Off}"
}

success() {
    echo "${Green}$@ ${Color_Off}"
}

command -v unzip >/dev/null ||
    error 'unzip is required to install Densky (see: https://github.com/denoland/deno#unzip-is-required)'

case $(uname -ms) in
'Darwin x86_64')
    target=darwin-x64
    ;;
'Darwin arm64')
    target=darwin-aarch64
    ;;
'Linux aarch64' | 'Linux arm64')
    # target=linux-aarch64
    error "Official Densky builds for Unix aarch64 are not available."
    exit 1
    ;;
*)
    target=linux-x64
    ;;
esac

if [ $target = darwin-x64 ]; then
    # Is this process running in Rosetta?
    # redirect stderr to devnull to avoid error message when not running in Rosetta
    if [[ $(sysctl -n sysctl.proc_translated 2>/dev/null) = 1 ]]; then
        target=darwin-aarch64
        info "Your shell is running in Rosetta 2. Downloading bun for $target instead"
    fi
fi

exe_name=densky

github_repo="https://github.com/Densky-Framework/densky"
if [ $# = 0 ]; then
    download_uri=$github_repo/releases/latest/download/$exe_name-$target.zip
else
    download_uri=$github_repo/releases/download/$1/$exe_name-$target.zip
fi

install_env=DENSKY_INSTALL
install_dir=${DENSKY_INSTALL:-$HOME/.densky}
bin_dir=$install_dir/bin
exe=$bin_dir/densky

if [ ! -d $bin_dir ]; then
    mkdir -p "$bin_dir" ||
        error "Failed to create install directory \"$bin_dir\""
fi

curl --fail --location --progress-bar --output "$exe.zip" "$download_uri" ||
    error "Failed to download Densky from \"$download_uri\""

unzip -oqd "$bin_dir" "$exe.zip" ||
    error 'Failed to extract Densky'

chmod +x "$exe" ||
    error 'Failed to set permissions on Densky executable'

rm -r "$exe.zip"

success "Densky was installed successfully to $Bold_Green$exe"

shell_config=''
commands=""

case $(basename "$SHELL") in
fish)
    commands="set -xU $install_env $install_dir
  fish_add_path \$$install_env/bin"

    shell_config=$HOME/.config/fish/config.fish
    ;;
zsh)
    commands="export $install_env=$install_dir
  export PATH=\"\$$install_env/bin:\$PATH\""

    shell_config=$HOME/.zshrc
    ;;
bash)
    commands="export $install_env=$install_dir
  export PATH=\$$install_env:\$PATH"

    shell_config="$HOME/.bashrc"
    ;;
*)
    shell_config="~/.bashrc"
    commands="export $install_env=$install_dir
  export PATH=\"\$$install_env/bin:\$PATH\""
    ;;
esac

echo "Manually add the directory to $shell_config (or similar):"
info_bold "  $commands"

echo
info "To get started, run:"
echo

info_bold "  densky --help"


