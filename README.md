# Image2slippytiles

## About

### Description

This tools converts images and slide scans to slippy tiles quickly and efficiently.

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