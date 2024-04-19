# `gpdwn`: download your entire GoPro Plus library

`gpdwn` is a small CLI utility that returns authenticated download links to one's GoPro Plus cloud video library.

GoPro allows downloading through their web UI, but I find it so bad I preferred reverse engineering how it works -- good job GoPro, you scared away a paying user!

As of the date of the last commit on this repository, it works -- _no guarantees_ on whether or not this tool will keep working in the future.

## Install

This program is written in Rust, so you need a Rust compiler to make it work.

See [rustup.rs](https://rustup.rs/) for more info.

```bash
cargo build --release

./target/release/gpdwn -- help
```

## Usage

```
Usage: ./target/release/gpdwn [OPTIONS]

Optional arguments:
  -v, --video-only  only download media IDs of "video" kind
  -e, --everything  download everything off GoPro servers ,even if they don't appear as media on their API
  -a, --auth-token AUTH-TOKEN
                    authentication token from https://plus.gopro.com, found  in your browser's "gp_access_token" cookie
  -c, --chunk-size CHUNK-SIZE
                    how many media IDs to download for each download URL generated, setting this to 0 will generate a single link with all media IDs (default: 25)
  -h, --help        print usage
```

Each link produced by this tool downloads a `.zip` file, which you can handle however you prefer.

Since `gpdwn` relies on an authentication cookie to obtain the download links, I highly recommed you feed them to `aria2c` for concurrent downloading, minimizing the chance of "not authenticated" errors:

```bash
./target/release/gpdwn --auth-token [TOKEN] > links
aria2c -i links
```
