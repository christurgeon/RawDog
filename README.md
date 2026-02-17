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

## macOS Finder Integration

You can add a right-click Quick Action so you can select ARW files in Finder and convert them without opening a terminal.

1. Open **Automator** and create a new **Quick Action**
2. Set "Workflow receives current" to **files or folders** in **Finder**
3. Add a **Run Shell Script** action with "Pass input" set to **as arguments**
4. Paste this script:

```bash
export PATH="/usr/local/bin:$HOME/.cargo/bin:$PATH"
rawdog "$@"
```

5. Save as "Convert with RawDog"

Now right-click any ARW files in Finder → **Quick Actions** → **Convert with RawDog**. It handles multiple files at once.

To prompt for an output folder instead, use this script:

```bash
export PATH="/usr/local/bin:$HOME/.cargo/bin:$PATH"

OUTPUT_DIR=$(osascript -e 'set theFolder to choose folder with prompt "Choose output folder"' -e 'POSIX path of theFolder' 2>/dev/null)

if [ -z "$OUTPUT_DIR" ]; then
    exit 0
fi

rawdog "$@" -o "$OUTPUT_DIR"
```

If the action doesn't appear, check **System Settings → Privacy & Security → Extensions → Finder**.

## License

MIT
