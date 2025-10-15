# Image2slippytiles

## About

This tools converts images and slide scans to slippy tiles quickly and efficiently.

Demo: https://files.coldfrontlabs.ca/tiling-demo/

### Supported input images types

Input images are processed using OpenSlide or Image depending.

`.svs` and `.dcm` images are processed with OpenSlide, and other files are attempted to be loaded with https://github.com/image-rs/image

Support Image types are list here: https://docs.rs/image/latest/image/codecs/index.html#supported-formats

## Usage

### Dependencies

To run on Rocky 9, install openslide libraries:
```
dnf install dnf-plugins-core
dnf copr enable @openslide/openslide
dnf install -y openslide
```

### Quick-start

Basic generating tiles from an image or slide:
```
./image2slippytiles giant_image.png
```

### Integration with Leaflet

After generating tiles, metadata about the tileset will be outputed, similar to:

```
TileMetadata {
    min_zoom: 0,
    max_zoom: 3,
    bounds: [
        0.0,
        0.0,
        -106.625,
        160.0,
    ],
    peak_memory: 114.44011,
    duration: 0.28572604,
    image_type: "png",
    image_metadata: ImageMetadata {
        width: 1280,
        height: 853,
    },
    slide_metadata: None,
}
```

The min and max zoom levels and the bounds can be used directly to setup the tileset with leaflet.

### Options

```
Usage: image2slippytiles [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>  The input image

Options:
  -v, --verbose                        Verbose output
  -d, --debug                          Debug output
  -o, --output <OUTPUT>                Output directory ('tiles' in the current directory if no option provided) [default: ./tiles]
  -m, --memory                         Output memory usage
  -z, --zoom <ZOOM>                    Starting zoom level [default: 0]
  -j, --json                           Output JSON metadata
  -t, --format <FORMAT>                Output type (png, jpg, webp) [default: png]
  -c, --colour <COLOUR>                Background color hex code: formatted as #RRGGBBAA [default: #00000000]
      --timeout <SECONDS>              Time in seconds that the process may run for [default: 10800]
  -T, --thumbnail                      Generate thumbnail
  -s, --thumbnailsize <THUMBNAILSIZE>  Thumbnail size [default: 512]
      --thumbnailfromtiles             Generate a thumbnail from an existing tileset
      --thumbnailfromzoomifytiles      Generate a thumbnail from an existing zoomify tileset
  -h, --help                           Print help
  -V, --version                        Print version
```

## Building

To build on Rocky 9, the Openslide library must be installed.

First, install EPEL, then run:

```
dnf install dnf-plugins-core
dnf copr enable @openslide/openslide
dnf install openslide-devel libwebp-devel jbigkit-devel
```

### AppImage

The appimage version can be built with cargo-appimage - https://crates.io/crates/cargo-appimage

By running: `cargo appimage`

## Tests

`jq` must be installed for testing.

There are acceptance tests for 6 sample images in various formats and sizes. The acceptance test:

- Compiles release version of the program
- Tiles the images
- Compare the hash of the generated files to the expected results

Tests can be run with:

```
./tests/acceptance-test.sh
```