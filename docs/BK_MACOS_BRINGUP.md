# Experimental BK4819 bring-up on macOS

The bundled `UV-5RM_Custom_Firmware.BF` targets `5RHNRFB-YP07` / FD6818B.
Do not use that binary for a radio reporting `5RHBK V0.14 BF`.
The `bk4819` Cargo feature selects a separate experimental driver. This is
hardware-porting work; compiling or uploading it does not establish RF support.
The default build still selects the FD6818B driver.

A [review against PR #3](PR3_BK_REVIEW.md) produced a TX-capable candidate,
`UV-5RM_Custom_Firmware_BK4819.BF`, SHA-256
`0bfffa6596fb4408715393cf46106ccacfa2027f03626b8788209aae76d9c1bf`.
It is built with `--features bk4819` (without `rx-only`). The initial image
flashed in this guide remains a separate receive-only bring-up artifact.
The candidate's hardware validation is recorded separately below.

## Current evidence

A stock radio reporting `5RHBK V0.14 BF` answered the stock programming protocol
from macOS 27.2 on Apple Silicon at `/dev/cu.usbserial-10`, using Python 3.15.2
and pyserial 3.5.
The 33,664-byte configuration and the separate 768-byte calibration area were
read twice with matching results, and also matched earlier independent reads.
These are configuration/calibration backups, not MCU firmware backups.
The bootloader acknowledged the flashing protocol and all 60 packages of the
60,644-byte experimental receive-only image. Upload took 8.1 seconds and the
bootloader acknowledged completion. The custom firmware then answered its CPS
handshake, and its raw calibration bytes matched the original stock wire data.
The flashed package SHA-256 is
`b107a0d1e546248eb7524e1b1f81b59277952092394729ef0663aac95eceb30a`.
The user reported normal menu navigation before the CPS app installation.
The logo and all nine apps then passed full readback verification. A final raw
read confirmed that the 32,000-byte channel table matches the decoded stock
backup and calibration is unchanged after app installation. The user confirmed
strong NOAA reception with loud, clear speaker audio on 162.550 MHz after
unplugging the programming cable. The connected cable had silenced the internal
speaker; this did not require a firmware change. This verifies one VHF FM receive
test, not every band, receive mode, squelch condition or scan feature.
Transmission and TEXT operation remain unverified and the flashed image is
receive-only.

Initial bring-up validation: seven Python backup-tool tests and three host-side
driver checks passed. Both the initial BK receive-only (60,644 bytes) and initial
BK TX-capable (61,344 bytes) builds fit, using Rust 1.99.1. The receive-only image
was flashed first; the later reviewed candidate and its results are recorded
below. The default
YP07 firmware rebuilds byte-for-byte to the supplied BF image with SHA-256
`4340f205430f0741e15a8e0dd28057985efff81e8dec68392affae7611fb2985`.

The BK driver's chip register recipe comes from Sophiel's GPL-3.0
[FD6818/BK4819 driver at 901ac448](https://github.com/sophiel-meow/bf-k61-firmware-aura/blob/901ac448533739719f8ddb3b030890edb6ca5a59/src/drivers/fd6818.rs).
The MCU serial bus, RX/TX front-end switches, and power-off behavior retain this
repository's UV-5RH board implementation. The firmware checks for chip ID
`0x4819` before initializing the BK RF driver; a mismatch drops the power latch
before changing the RF configuration. The receive-only build emits a
static serial diagnostic at 115200 baud reporting the chip ID.

## Mac setup and stock backup

Run the commands below from `UV-5RM Custom Firmware`. The port name can change;
use `python -m serial.tools.list_ports -v` to identify the connected cable.

```sh
python3 -m venv backup/venv
. backup/venv/bin/activate
python -m pip install pyserial
python -m serial.tools.list_ports -v
mkdir -p backup/my-bk
python source-changes/radio_read.py /dev/cu.usbserial-10 backup/my-bk/stock.img --calibration --verify
```

The radio must be on at its normal screen. Power-cycle normally before the read
and again before another operation. The tool permits only identification and
64-byte read requests, validates response addresses, refuses existing output
files, and with `--verify` requires two identical reads before saving anything.
It saves a JSON manifest with SHA-256 hashes and each region's radio address and
file offset. Configuration data uses the stock XOR transform; calibration reads
are already plaintext, verified against custom-firmware direct SPI reads on the
BK radio, so that export must not be XOR-decoded. The calibration file covers `0xF000..0xF2FF`; this is not a complete
external-flash dump. Keep everything inside the git-ignored `backup` folder.
Do not publish a radio's settings or calibration with a PR.

## Build and package

Install a current stable Rust toolchain with target `thumbv7em-none-eabi`.
The first hardware test uses **both** features below. `rx-only` makes
`Radio::enter_tx` return false even if settings or an app requests transmission;
changing TXINH cannot enable TX in this build. TEXT communication cannot be
validated using this build.

```sh
cargo build --release --locked --manifest-path firmware-source/Cargo.toml --config firmware-source/.cargo/config.toml --features bk4819,rx-only --target-dir /tmp/uv5rm-bk-build
python source-changes/elf2bin.py /tmp/uv5rm-bk-build/thumbv7em-none-eabi/release/bf5rh-fw backup/my-bk/bk-rx-only.bin
```

The existing wrapper requires at least five stock reference images to cross-check
its transform. Clone the research repository into the ignored backup directory:

```sh
git clone https://github.com/amoxu/Baofeng-UV-5RM-5RH-RE backup/reference
python source-changes/wrap_bf.py backup/my-bk/bk-rx-only.bin backup/my-bk/bk-rx-only.BF backup/reference/factory_firmware
shasum -a 256 backup/my-bk/bk-rx-only.BF
```

The wrapper verifies against the stock images, checks the ARM vector table and
60 KiB limit, and creates an application-only package. Record the full hash of
your actual output; it can differ with compiler and source changes.

## Recovery candidate

The research repository supplies
[`5RH_BK_V0.14_231123.BF`](https://github.com/amoxu/Baofeng-UV-5RM-5RH-RE/blob/main/factory_firmware/BK_BK4819/5RH_BK_V0.14_231123.BF).
The copy inspected during this bring-up has SHA-256
`2b9aa883196cbc57c12e1ec5b62ccd062dc589253d5cfacb50eef6faec171441`.
It contains a 59,720-byte application and a 1,540-byte second region.
Its provenance and suitability for an individual radio are unverified.
Having the file is not proof of a working recovery path.

The simple `source-changes/flash_bf.py` deliberately accepts only one-region
packages. The local bring-up backup also contains an application-only recovery
candidate, created by preserving the stock encrypted application bytes and
replacing only the BF container header. It omits region 2 to preserve the
radio's existing system/config region. Its SHA-256 is
`559db6512eef44c1069eab2c1bf2c66484fe7780565e904ebb54518224a15319`.
Neither candidate has yet been demonstrated to restore this radio. Restoring
stock application code would also not automatically restore settings erased
or changed by the custom firmware.

## Flash and verify

Charge the battery first. Turn the radio off, hold **both side buttons** while
turning it on, and release them. The screen should stay black.

```sh
python source-changes/bootloader_probe.py /dev/cu.usbserial-10
```

A valid bootloader acknowledgement proves only the connection/protocol. After
reviewing the actual image and recorded checksum, flash the **BK receive-only**
package using its own SHA-256 (the third argument accepts the full hash):

```sh
python source-changes/flash_bf.py backup/my-bk/bk-rx-only.BF /dev/cu.usbserial-10 YOUR_ACTUAL_SHA256
```

The firmware should boot normally. Verify the custom CPS handshake and compare
the calibration area against the stock backup before installing apps. Then the
existing setup tool installs and reads back all apps and the logo:

```sh
python source-changes/setup_radio.py /dev/cu.usbserial-10 unused-logo-argument 64 apps
```

After the tool finishes and the radio restarts, **unplug the programming cable
from the radio before testing the internal speaker**. On the tested radio the
cable silenced the speaker even though RF reception was working.

Record display/keypad operation, reboot/power-off, receive audio, VHF/UHF tuning,
squelch, tones and scanning separately. TX, power, frequency accuracy and TEXT
require later testing with a TX-capable build and suitable RF test equipment.
Keep the README compatibility warning until those results justify changing it.

## Offline checks

From `UV-5RM Custom Firmware`:

```sh
python -m unittest discover -s source-changes -p test_radio_read.py
rustc --edition=2021 --test firmware-source/tools/check_bk_driver.rs -o /tmp/check-bk-driver
/tmp/check-bk-driver
```

The host driver checks simulate bus transactions to inspect initialization and
front-end selection. They cannot verify electrical timing or RF behavior.

## Final candidate after PR #3 review

See [the review](PR3_BK_REVIEW.md) for the stock-image findings, adopted fixes,
intentional differences, build checksum and validation limits. The application
is 61,404 bytes, leaving 36 bytes. To rebuild it, use the same build/package
commands above with `--features bk4819` and separate output filenames.

To install the already-built candidate from `UV-5RM Custom Firmware`:

```sh
python source-changes/bootloader_probe.py /dev/cu.usbserial-10
python source-changes/flash_bf.py UV-5RM_Custom_Firmware_BK4819.BF /dev/cu.usbserial-10 0bfffa6596fb4408715393cf46106ccacfa2027f03626b8788209aae76d9c1bf
```

The radio must be in flash mode first. Existing apps, channels and calibration
are stored separately; check them after the update. Confirm reception with
TXINH on. When ready for a controlled transmit test, first select an appropriate
transmit frequency, then change SETTINGS → RADIO → TXINH to off and press Menu
to save. NOAA weather reception is a receive-only check.

Additional clock checks (from `UV-5RM Custom Firmware`):

```sh
rustc --edition=2021 --cfg 'feature="bk4819"' --test firmware-source/tools/check_bk_clock.rs -o /tmp/check-bk-clock
/tmp/check-bk-clock
```

The reviewed candidate subsequently flashed in 8.1 seconds and answered the
custom CPS handshake. The logo and all nine apps passed full readback, channels
and calibration remained unchanged, and TXINH was confirmed on. Four additional
clock-fallback tests pass, bringing the offline checks to 14. The user confirmed
that NOAA reception on 162.550 MHz still works after this update, with the cable
unplugged. No TX test was performed.
