/* SPDX-License-Identifier: GPL-3.0-or-later */
/*
 * AureaOS — TurboJPEG legacy API compatibility probe
 * Development diagnostic only; not production decoder code.
 */

#include <turbojpeg.h>

#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_INPUT_BYTES ((size_t)64 * 1024 * 1024)
#define MAX_RGBA_BYTES ((size_t)256 * 1024 * 1024)
#define MAX_DIMENSION 16384

static int read_input(const char *path, unsigned char **data, size_t *length)
{
    FILE *file = fopen(path, "rb");
    if (file == NULL) {
        perror("fopen");
        return -1;
    }

    if (fseek(file, 0, SEEK_END) != 0) {
        perror("fseek");
        fclose(file);
        return -1;
    }

    long file_length = ftell(file);
    if (file_length <= 0 || (unsigned long)file_length > MAX_INPUT_BYTES) {
        fprintf(stderr, "Input is empty or exceeds the 64 MiB limit.\n");
        fclose(file);
        return -1;
    }

    if (fseek(file, 0, SEEK_SET) != 0) {
        perror("fseek");
        fclose(file);
        return -1;
    }

    unsigned char *buffer = malloc((size_t)file_length);
    if (buffer == NULL) {
        fprintf(stderr, "Unable to allocate input buffer.\n");
        fclose(file);
        return -1;
    }

    size_t bytes_read = fread(buffer, 1, (size_t)file_length, file);
    if (bytes_read != (size_t)file_length || ferror(file)) {
        fprintf(stderr, "Unable to read the complete JPEG input.\n");
        free(buffer);
        fclose(file);
        return -1;
    }

    fclose(file);
    *data = buffer;
    *length = bytes_read;
    return 0;
}

static int decode_and_check(const unsigned char *jpeg, size_t jpeg_length)
{
    if (jpeg_length > (size_t)ULONG_MAX) {
        fprintf(stderr, "JPEG input length does not fit TurboJPEG's API.\n");
        return -1;
    }

    tjhandle decoder = tjInitDecompress();
    if (decoder == NULL) {
        fprintf(stderr, "tjInitDecompress failed: %s\n", tjGetErrorStr());
        return -1;
    }

    int width = 0;
    int height = 0;
    int subsampling = TJSAMP_UNKNOWN;
    int colorspace = TJCS_UNKNOWN;

    if (tjDecompressHeader3(decoder, jpeg, (unsigned long)jpeg_length,
                            &width, &height, &subsampling, &colorspace) != 0) {
        fprintf(stderr, "tjDecompressHeader3 failed: %s\n",
                tjGetErrorStr2(decoder));
        tjDestroy(decoder);
        return -1;
    }

    if (width <= 0 || height <= 0 ||
        width > MAX_DIMENSION || height > MAX_DIMENSION) {
        fprintf(stderr, "JPEG dimensions are invalid or exceed %d pixels per axis.\n",
                MAX_DIMENSION);
        tjDestroy(decoder);
        return -1;
    }

    if (colorspace != TJCS_RGB &&
        colorspace != TJCS_YCbCr &&
        colorspace != TJCS_GRAY) {
        fprintf(stderr, "This initial RGBA probe does not accept CMYK/YCCK JPEG input.\n");
        tjDestroy(decoder);
        return -1;
    }

    size_t pixel_count = (size_t)width * (size_t)height;
    if (pixel_count > SIZE_MAX / 4) {
        fprintf(stderr, "RGBA8 output size overflows size_t.\n");
        tjDestroy(decoder);
        return -1;
    }

    size_t rgba_length = pixel_count * 4;
    if (rgba_length > MAX_RGBA_BYTES) {
        fprintf(stderr, "RGBA8 output exceeds the 256 MiB limit.\n");
        tjDestroy(decoder);
        return -1;
    }

    unsigned char *rgba = malloc(rgba_length);
    if (rgba == NULL) {
        fprintf(stderr, "Unable to allocate RGBA8 output buffer.\n");
        tjDestroy(decoder);
        return -1;
    }

    int pitch = width * 4;
    int flags = TJFLAG_STOPONWARNING | TJFLAG_LIMITSCANS;
    int result = tjDecompress2(decoder, jpeg, (unsigned long)jpeg_length,
                               rgba, width, pitch, height, TJPF_RGBA, flags);
    if (result != 0) {
        fprintf(stderr, "tjDecompress2 failed: %s\n", tjGetErrorStr2(decoder));
        free(rgba);
        tjDestroy(decoder);
        return -1;
    }

    for (size_t alpha = 3; alpha < rgba_length; alpha += 4) {
        if (rgba[alpha] != 255) {
            fprintf(stderr, "RGBA output contains a non-opaque alpha sample.\n");
            free(rgba);
            tjDestroy(decoder);
            return -1;
        }
    }

    printf("TurboJPEG legacy API: OK\n");
    printf("Dimensions: %d x %d\n", width, height);
    printf("Subsampling: %d; colorspace: %d\n", subsampling, colorspace);
    printf("RGBA8 bytes: %zu\n", rgba_length);
    printf("Opaque alpha channel: OK\n");

    free(rgba);
    tjDestroy(decoder);
    return 0;
}

int main(int argc, char **argv)
{
    if (argc != 2) {
        fprintf(stderr, "Usage: %s IMAGE.jpg\n", argv[0]);
        return EXIT_FAILURE;
    }

    unsigned char *jpeg = NULL;
    size_t jpeg_length = 0;
    if (read_input(argv[1], &jpeg, &jpeg_length) != 0) {
        return EXIT_FAILURE;
    }

    int result = decode_and_check(jpeg, jpeg_length);
    free(jpeg);
    return result == 0 ? EXIT_SUCCESS : EXIT_FAILURE;
}
