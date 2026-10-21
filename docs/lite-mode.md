# Lite mode: 8 GB RAM, no GPU

FilmCraft runs on a machine with 8 GB of RAM and only integrated graphics (or none). Lite mode
turns on **by itself** when the machine has 8 GB of RAM or less (Windows and Linux). You can also
switch it on or off yourself:

```sh
filmcraft --lite          # always lite
filmcraft --no-lite       # never lite
FILMCRAFT_LITE=1 filmcraft
```

The log file says `lite mode: on` or `lite mode: off` near the top.

## What it changes

| | normal | lite |
|---|---|---|
| Decoded frames held by all clips | 1 GiB | 256 MiB |
| Monitor / thumbnail frame cache (Settings ▸ Memory) | 768 MB | at most 256 MB |
| Decoders kept alive when idle / recently used | 4 / 8 | 2 / 3 |
| Threads per video decoder (H.264, HEVC, VP9, AV1) | all cores, up to 16 | up to 4 |
| Render and export threads | all cores | up to 4 |
| Preview compositing | GPU | CPU (no GPU memory used) |
| GPU export renderer | yes | no (exports render on the CPU) |
| Playback / paused resolution of the monitors | 1/2 / Full | 1/4 / 1/2 |

Fewer decoder threads matter most: a frame-threaded decoder holds `threads + 2` pictures in flight,
so a 4K clip on a 16-thread decoder can take half a gigabyte by itself.

You can still raise the monitors' resolution from the monitor's settings menu at any time.

## No GPU at all

The window itself still needs a graphics adapter. If there is no usable GPU driver, FilmCraft falls
back by itself after a failed start; to go straight to software drawing:

```sh
filmcraft --lite --graphics software   # Windows (WARP) and Linux (llvmpipe)
```

## Building on an 8 GB machine

Compiling the workspace in parallel can use more than 8 GB. Build only the app, in release mode,
with fewer jobs:

```sh
cargo build --release -p filmcraft -j 2
```

The app is then `target/release/filmcraft` (`filmcraft.exe` on Windows).

## Tips for smooth editing

- Use proxies for 4K or HEVC footage (Clip ▸ Proxy ▸ Create Proxies…, preset *H.264 1/4*).
- Keep Settings ▸ Playback ▸ *Draft decoding* on.
- Close other heavy apps (browsers) while exporting.
