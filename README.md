# Diffscreen

Toy remote desktop implemented by rust.

The python implemented: https://github.com/pysrc/remote-desktop

## Build

`cargo build --release`


### For linux before build

`sudo apt install libxdo-dev libxcb-randr0-dev libxcb-shm0-dev`

For Debian-based GUI distributions, that means running:

`sudo apt-get install libx11-dev libxext-dev libxft-dev libxinerama-dev libxcursor-dev libxrender-dev libxfixes-dev libpango1.0-dev libgl1-mesa-dev libglu1-mesa-dev`

For RHEL-based GUI distributions, that means running:

`sudo yum groupinstall "X Software Development" && yum install pango-devel libXinerama-devel`

For Arch-based GUI distributions, that means running:

`sudo pacman -S libx11 libxext libxft libxinerama libxcursor libxrender libxfixes pango cairo libgl mesa --needed`

For Alpine linux:

`apk add pango-dev fontconfig-dev libxinerama-dev libxfixes-dev libxcursor-dev mesa-gl`

For NixOS (Linux distribution) this nix-shell environment can be used:

`nix-shell --packages rustc cmake git gcc xorg.libXext xorg.libXft xorg.libXinerama xorg.libXcursor xorg.libXrender xorg.libXfixes libcerf pango cairo libGL mesa pkg-config`



## dsserver

`./dsserver password port`

The default password is "diffscreen" and default port is 80.


## dsclient

Click run directly
