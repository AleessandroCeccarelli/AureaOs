# TurboJPEG ABI probe

This diagnostic checks the legacy TurboJPEG API used by the proposed AureaOS JPEG adapter. It is deliberately separate from the Rust workspace and does not add a runtime or Cargo dependency.

## Fedora development host

Install the TurboJPEG development package recorded in the dependency register, then compile and run:

```sh
cc -std=c11 -Wall -Wextra -Werror -O2 \
  tools/dependencies/jpeg/turbojpeg-abi-probe.c \
  -o /tmp/aurea-turbojpeg-probe -lturbojpeg

/tmp/aurea-turbojpeg-probe \
  aurea-human-interfaces/resources/wallpapers/aureaos-midnight-flow.jpg
```

The expected output includes the JPEG dimensions, RGBA8 byte count, and successful opaque-alpha validation. The committed wallpaper should report 3840 x 2160 and 33,177,600 RGBA8 bytes.

## Debian 13 target

Install the Debian development package:

```sh
sudo apt install build-essential libturbojpeg0-dev
```

Run the same compile and probe commands from the repository root. A successful Fedora run is not evidence of Debian compatibility: the same source must compile and run against Debian Trixie's libturbojpeg0 2.1.5 package before the adapter is approved.

## Scope and safety

- Uses the legacy `tjInitDecompress`, `tjDecompressHeader3`, `tjDecompress2`, and `tjDestroy` API.
- Checks compressed input size, dimensions, RGBA8 arithmetic, and a 256 MiB decoded-output budget before allocation.
- Uses `TJPF_RGBA`, stops on decoder warnings, and enables the TurboJPEG progressive-scan limit.
- The initial probe accepts RGB, YCbCr, and grayscale JPEG color spaces; CMYK/YCCK are rejected explicitly.
- This is an ABI/decode smoke test, not a security audit, exhaustive JPEG conformance suite, or production decoder.
- Do not add this C source to the installed AureaOS image.

## Compatibility note

TurboJPEG 2.1.x does not define `TJCS_DEFAULT` or `TJCS_UNKNOWN` in its legacy colorspace enum. The probe therefore initializes the output-only `jpegColorspace` integer to `-1`; `tjDecompressHeader3()` writes the detected value on success. Do not replace this with a newer TurboJPEG 3.x enum constant when checking Debian 13 compatibility.

## Automated compatibility check

The GitHub Actions workflow `.github/workflows/turbojpeg-abi.yml` compiles this probe and runs it against the committed wallpaper in Debian 13 Trixie and Fedora containers. It also inspects the linked TurboJPEG runtime library. A successful workflow validates only this probe and image path; it does not approve a Rust FFI adapter or replace review of the headers, exported symbols, runtime package and decoder boundary.
