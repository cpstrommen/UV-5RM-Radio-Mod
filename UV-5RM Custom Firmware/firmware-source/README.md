# bf5rh-fw

> [!WARNING]
> Experimental, and under development.

Experimental from-scratch Rust firmware for the Baofeng UV-5RH/5RM.

Scaffolding copied and adapted from [bf-k61-firmware-aura](https://github.com/sophiel-meow/bf-k61-firmware-aura) — same
structure, different MCU family (KD32F328CB/Cortex-M0 there vs.
AT32F421/Cortex-M4 here), so the PAC crate, `memory.x`, and register
access are all new.

## Hardware and build profiles

The default profile targets the original YP07/FD6818B hardware. This repository
also carries feature-gated BK4819 support for radios whose hold-8 startup screen
reads `5RHBK V0.14 BF`, plus an AIRBAND profile built with both features:

| Cargo features | Intended target |
|---|---|
| default | Original YP07 firmware profile |
| `bk4819` | BK4819 base profile |
| `bk4819,airband` | BK4819 profile with AIR1 storage and AIRBAND Band/SCAN route |

Do not interchange images between hardware identifiers. See the
[repository README](../../README.md) for the hardware matrix and the
[AIRBAND guide](../airband/README.md) for exact hashes, installation and current
hardware validation.

## Build

```sh
nix develop --command cargo build --release
nix develop --command cargo objcopy --release -- -O binary bf5rh-fw.bin
```

The AIRBAND build is close to the 60 KiB application limit and uses a dedicated
Cargo configuration enabling LLVM machine outlining:

```sh
cargo build --release --locked \
  --manifest-path Cargo.toml \
  --config ../source-changes/airband/firmware.toml \
  --features bk4819,airband \
  --target-dir /tmp/uv5rm-airband-fw
```

Run the repository-level AIRBAND test orchestration from `UV-5RM Custom
Firmware` before wrapping or flashing:

```sh
python3 source-changes/airband/test.py
```

That command covers the database tools, real app logic against a mocked radio,
all four AIRBAND overlay segments, firmware menu routing and legacy checks. Host
tests and simulator renders do not replace physical RF/display validation.

## Flash (stock bootloader entry only)

1. Wrap+encrypt the built binary(https://github.com/amoxu/Baofeng-UV-5RM-5RH-RE):
   ```sh
   ./uv5rm-wrap-tool/uv5rm-wrap-tool \
     -m wrap -i bf5rh-fw.bin -o bf5rh-fw.BF -f
   ```
   (region 2 is omitted here — an empty/absent region 2 is the
   untested path `tools/flash.py`'s docstring warns about.)
2. Hold the radio's side key while powering on so it enters flash mode
3. `nix develop --command python tools/flash.py bf5rh-fw.BF /dev/ttyUSB0`

