# Image2slippytiles

This tools converts images and slide scans to slippy tiles quickly and efficiently.

## Using

## Building

To build on Rocky 9, the Openslide library must be installed.

First, install EPEL, then run:

```
dnf install dnf-plugins-core
dnf copr enable @openslide/openslide
dnf install openslide-devel libwebp-devel jbigkit-devel
```