/* SPDX-License-Identifier: GPL-3.0-or-later */
/*
 * AureaOS — narrow TurboJPEG C ABI shim.
 * Codec declarations, constants and handles stay on this side of the ABI.
 */

#include <turbojpeg.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>

#define SHIM_INVALID_INPUT 1
#define SHIM_UNSUPPORTED_COLOR_SPACE 2
#define SHIM_DECODER_FAILURE 3
#define SHIM_OUTPUT_MISMATCH 4

static int read_header(const unsigned char *input, unsigned long input_len, int *width, int *height)
{
    if (input == NULL || input_len == 0 || width == NULL || height == NULL) return SHIM_INVALID_INPUT;
    tjhandle decoder = tjInitDecompress();
    if (decoder == NULL) return SHIM_DECODER_FAILURE;

    int subsampling = -1;
    int colorspace = -1;
    int result = tjDecompressHeader3(decoder, input, input_len, width, height, &subsampling, &colorspace);
    tjDestroy(decoder);

    if (result != 0 || *width <= 0 || *height <= 0) return SHIM_INVALID_INPUT;
    if (colorspace != TJCS_RGB && colorspace != TJCS_YCbCr && colorspace != TJCS_GRAY) return SHIM_UNSUPPORTED_COLOR_SPACE;
    return 0;
}

int aurea_turbojpeg_header(const unsigned char *input, unsigned long input_len, int *width, int *height)
{
    return read_header(input, input_len, width, height);
}

int aurea_turbojpeg_decode(const unsigned char *input, unsigned long input_len, int width, int height, unsigned char *output, size_t output_len)
{
    if (input == NULL || output == NULL || width <= 0 || height <= 0) return SHIM_INVALID_INPUT;
    if ((size_t)width > SIZE_MAX / (size_t)height) return SHIM_OUTPUT_MISMATCH;
    size_t pixels = (size_t)width * (size_t)height;
    if (pixels > SIZE_MAX / 4) return SHIM_OUTPUT_MISMATCH;
    size_t required = pixels * 4;
    if (output_len != required || width > INT_MAX / 4) return SHIM_OUTPUT_MISMATCH;

    int header_width = 0;
    int header_height = 0;
    int status = read_header(input, input_len, &header_width, &header_height);
    if (status != 0) return status;
    if (header_width != width || header_height != height) return SHIM_OUTPUT_MISMATCH;

    tjhandle decoder = tjInitDecompress();
    if (decoder == NULL) return SHIM_DECODER_FAILURE;
    int result = tjDecompress2(decoder, input, input_len, output, width, width * 4, height,
                               TJPF_RGBA, TJFLAG_STOPONWARNING | TJFLAG_LIMITSCANS);
    tjDestroy(decoder);
    return result == 0 ? 0 : SHIM_INVALID_INPUT;
}
