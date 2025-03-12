# env-libvpx-sys

[![Crates.io](https://img.shields.io/crates/v/env-libvpx-sys.svg)](https://crates.io/crates/env-libvpx-sys)
![build](https://github.com/astraw/env-libvpx-sys/workflows/Build%20and%20Run/badge.svg)
[![Documentation](https://docs.rs/env-libvpx-sys/badge.svg)](https://docs.rs/env-libvpx-sys/)

⚠⚠ This repository is now archived. I no longer use it and cannot maintain it.
Please fork it and use it yourself. If your are interested in taking ownership
of the project please contact me ([@astraw](https://github.com/astraw)). ⚠⚠

Rust bindings to libvpx.

## Features and characteristics

The `env-libvpx-sys` crate offers the following:

* It provides only the `-sys` layer. VPX header files are wrapped with bindgen
  and the native library is linked. However, no higher-level Rust interface is
  provided. (See the [vpx-encode crate](https://crates.io/crates/vpx-encode) for
  a simple higher-level interface).
* It adds [Continuous Integration tests for Windows, Linux and
  Mac](https://github.com/astraw/env-libvpx-sys/actions).
* It includes bundled bindgen-generated FFI wrapper for a few versions of
  libvpx. You can also enable `generate` feature of this crate to generate FFI
  on the fly for a custom version of libvpx.
* It originally started as a fork of
  [libvpx-native-sys](https://crates.io/crates/libvpx-native-sys) (see
  [history](#History) below).

## How libvpx version is selected

At compilation time, `build.rs` determines how to link libvpx, including what
version to use.

### Option 1: let `pkg-config` find it

This scenario is the default and is used when the environment variable
`VPX_LIB_DIR` is not set. In this case,
[`pkg-config`](https://crates.io/crates/pkg-config) will attempt to
automatically discover libvpx.

If `VPX_VERSION` is set, `build.rs` will ensure that `pkg-config` returns the
same version. If `VPX_VERSION` is not set, the version returned by `pkg-config`
will be used.

Note that `pkg-config` will check the `VPX_STATIC` environment variable, and if
it is set, will attempt static linking.

### Option 2: specify libvpx location manually

In this scenario, set the following environment variables: `VPX_LIB_DIR`,
`VPX_INCLUDE_DIR`, and `VPX_VERSION` appropriately. Caution: if `VPX_VERSION`
does not match the linked library

Additionally, `VPX_STATIC` may be set to `1` to force static linking.

### Discussion about theoretical alternative of using cargo features to specify library version

At one point, cargo features were considered as a means to select the library
version used. However, this meant
the final application binary would need to specify the library version used.
This would place a requirement on all crates in the dependency chain from the
final application binary to `env-libvpx-sys` that they must explicitly depend on
`env-libvpx-sys` (even if, as is very likely beyond a vpx wrapper crate such as
[`vpx-encode`](https://crates.io/crates/vpx-encode)) the intermediate or final
crate does not directly call into `env-libvpx-sys`.

As an additional problem, because cargo features are additive, the possibility
for conflicting build requests with two sets of features would be possible in
this scenario. The present alternative, namely setting the environment variable
`VPX_VERSION`, naturally enforces the selection of only a single version.

## (Re)generating the bindings with bindgen

If the bindings for your version are not pre-generated in the `generated/`
directory, you may let [`bindgen`](https://crates.io/crates/bindgen)
automatically generate them during the build process by using the `generate`
cargo feature.

To save your (re)generated bindings and commit them to this repository, build
using with the `generate` cargo feature. The easiest way to do this is to use
the script `regen-ffi.sh` (or `regen-ffi.bat` on Windows). Then, copy the
generated file in `target/debug/build/env-libvpx-sys-<hash>/out/ffi.rs` to
`generated/vpx-ffi-<version>.rs`. Finally, add this file to version control.

## History and thanks

This began as a fork of
[libvpx-native-sys](https://crates.io/crates/libvpx-native-sys) with a [fix to
simplify working with Windows](https://github.com/kornelski/rust-vpx/pull/1).
Thanks to those authors!
