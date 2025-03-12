# Download https://github.com/ShiftMediaProject/libvpx/releases/download/v1.10.0/libvpx_v1.10.0_msvc16.zip
# and unzip into %HomeDrive%%HomePath%\libvpx_v1.10.0_msvc16
$env:VPX_STATIC="1"
$env:VPX_VERSION="1.10.0"
$env:VPX_LIB_DIR="$env:HomeDrive$env:HomePath\libvpx_v1.10.0_msvc16\lib\x64"
$env:VPX_INCLUDE_DIR="$env:HomeDrive$env:HomePath\libvpx_v1.10.0_msvc16\include"

# Download llvm from https://releases.llvm.org/download.html
# https://github.com/llvm/llvm-project/releases/download/llvmorg-18.1.8/clang+llvm-18.1.8-x86_64-pc-windows-msvc.tar.xz
$env:LIBCLANG_PATH="$env:HomeDrive$env:HomePath\clang+llvm-18.1.8-x86_64-pc-windows-msvc\bin"
cargo build --release
