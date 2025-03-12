# 5.1.3 - 2023-09-10

## Added

* Support for libvpx 1.13

# 5.1.1 - 2022-05-30

## Added

* Support for libvpx 1.11

# 5.1.0 - 2021-06-07

## Changed

* Simplified logic in `build.rs`.

## Added

* Support for libvpx 1.10

# 5.0.0 - 2020-10-23

## Changed

* [breaking] Remove implementations of `Default` for `vpx_codec_enc_cfg`,
  `vpx_codec_ctx`, `vpx_image_t`. The old code zero-initialized these, which is
  not valid and actually undefined behavior. This nevertheless worked for older
  compilers, but triggers a panic with newer versions of rust. The correct
  technique is to use `mem::MaybeUninit`.

## Added

* Support for libvpx 1.9

# 4.0.13 - 2020-03-27

## Changed

* Update github actions to perform better CI testing on Windows, Linux, Mac
* Recompile if environment variables change.

# 4.0.12 - 2019-12-02

## Added

* Use github actions to perform CI testing on Windows, Linux, Mac

## Changed

* The name of the windows static library is changed from `vpxmt` to `libvpx`.
  The new name is the name used by [Shift Media
  Project](https://github.com/ShiftMediaProject/libvpx) in their Windows builds.
