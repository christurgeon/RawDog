# RawDog

Raw files in, images out. No Lightroom, no fuss.

A no-nonsense Sony ARW converter supporting JPEG, TIFF, and PNG output.

## Installation

```sh
cargo install --path .
```

## Usage

Convert a single file:

```sh
rawdog photo.ARW
```

Convert all ARW files in a directory:

```sh
rawdog ./photos/
```

Specify an output directory:

```sh
rawdog ./photos/ -o ./jpegs/
```

Resize the long edge to 2048px:

```sh
rawdog photo.ARW -r 2048
```

Set JPEG quality to 85:

```sh
rawdog photo.ARW -q 85
```

Output as 16-bit TIFF:

```sh
rawdog photo.ARW -f tiff
```

Output as 16-bit PNG with resize:

```sh
rawdog ./photos/ -f png -r 2048
```

Combine options:

```sh
rawdog ./photos/ -o ./jpegs/ -q 95 -r 3000 --overwrite
```

## License

MIT
