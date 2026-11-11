# UV-5RM Radio Mod

Custom firmware and add-on apps for the Baofeng UV-5RM, built on the open-source
[bf-5rh-firmware-open](https://github.com/sophiel-meow/bf-5rh-firmware-open). The main addition
to the original project is `TEXT`; this branch also contains a separately built
BK4819 profile and the receive-only `AIRBAND` directory application.

> [!WARNING]
> Flashing is experimental and may be irreversible. A proven stock-restoration
> procedure is not available for every hardware variant, a bad flash can brick
> the radio, and CHIRP does not communicate with this custom firmware. Identify
> the hardware before choosing an image, keep every backup, and do not interrupt
> a write. Use the included tools for channels and configuration.

## Documentation map

- [AIRBAND user, installation and developer guide](UV-5RM%20Custom%20Firmware/airband/README.md)
- [5RHBK/BK4819 macOS bring-up and recovery notes](docs/BK_MACOS_BRINGUP.md)
- [Cross-check against the existing BK4819 PR](docs/PR3_BK_REVIEW.md)
- [Future passive-location research proposal](docs/airband-location-future/README.md)
- [Firmware source notes](UV-5RM%20Custom%20Firmware/firmware-source/README.md)

## Current profiles and validation

These images are not interchangeable. Holding **8** while powering on displays
the hardware identifier used to select a profile.

| Profile | Hardware | Artifact | Current evidence |
|---|---|---|---|
| Original | `5RHNRFB-YP07` | `UV-5RM_Custom_Firmware.BF` | Original profile; rebuild and regression checks pass |
| BK4819 base | `5RHBK V0.14 BF` | `UV-5RM_Custom_Firmware_BK4819.BF` | Boots and receives on the tested radio; PR #3 independently reports RX/TX on another radio; output power and TEXT remain unmeasured here |
| AIRBAND/BK4819 | `5RHBK V0.14 BF` | `airband/UV-5RM_AIRBAND_BK4819.BF` | Installed from macOS and verified by full readback on one radio; protected data remained unchanged and TXINH remained on |

The final AIRBAND artifact has SHA-256
`6cf9ff04cf025ab7368396ca0d6d7892a9eaf37e44e0c51d1bf3b5c15eee1413`.
The app has SHA-256
`9a0b98c280816304db2f4a692d6911acf6ea23257463f329645a55792d56a6f1`.
The database has SHA-256
`16db31f1440a1428d290d5dc6d8fb4f94af343d201aae696aac69a55c2925208`.
Machine-readable sizes, hashes and validation state are recorded in
[artifacts.json](UV-5RM%20Custom%20Firmware/airband/artifacts.json).

“Installed and read back” is not the same as complete RF validation. On the
final build, the arrow editor, enlarged AIRBAND text and AM airband reception
still need physical confirmation. AIRBAND itself never requests transmission;
the surrounding firmware retains its normal transmit support, controlled by
TXINH and applicable law.

## What it adds

| Feature | What it does |
|---|---|
| **AIRBAND** | Offline US airport/VOR lookup, service selection, favorites, airport scanning and nearby lists; see the [AIRBAND profile](UV-5RM%20Custom%20Firmware/airband/README.md) |
| **TEXT** | Sends messages up to 50 characters, retries until confirmed, saves them, and plays a tone when one arrives |
| **Scan modes** | Press Band to switch between Weather, Police/Fire/EMS, Air, Repeaters, FRS, GMRS, MURS and more, or type a custom range |
| **BAND SWEEP** | Sweeps a whole band for activity |
| **RPT FINDER** | Works out a repeater's input frequency and tone |
| **FOX HUNT** | Signal meter and rough distance estimate for finding a transmitter |
| **SEARCH/QT SCAN** | Finds a nearby signal's frequency, or the tone on your channel; press 1 to switch |
| **Look and feel** | Plain black and red screens with a custom start-up tune |

Screenshots: see the `preview_*.png` files in `UV-5RM Custom Firmware`.

## Before you start: check your radio

The original bundled image targets YP07 hardware. The BK base and AIRBAND images
are separate builds. Radios that look identical can have a different RF chip.

The [BK4819/macOS guide](docs/BK_MACOS_BRINGUP.md) records the board-specific
driver work, backups and known recovery candidates. The
[PR #3 cross-check](docs/PR3_BK_REVIEW.md) explains which findings were adopted.

1. Turn the radio off, then hold **8** while turning it on.
2. Read the text on the screen, then turn the radio off without pressing anything else.

| Screen says | What to do |
|---|---|
| `5RHNRFB-YP07` | Use only the original YP07 profile |
| `5RHBK V0.14 BF` | Use the BK4819 base image or the dedicated AIRBAND/BK4819 profile |
| Anything else, such as `...NRF` | **Do not flash any image documented here** without a separate port and hardware validation |

## What you need

- A Windows PC, or macOS using the [tested experimental workflow](docs/BK_MACOS_BRINGUP.md). Hardware compatibility still depends on the firmware build.
- [Python](https://www.python.org/downloads/), plus the serial library: `py -m pip install pyserial`
- A Baofeng programming cable. The commands below assume `COM3`, so change it to match your port.

Run these from the `UV-5RM Custom Firmware` folder.

## Install the original non-AIRBAND profile

These generic steps are retained for the original app bundle. **Do not use this
section to install AIRBAND.** Its fixed slot, capability check, database layout,
plain-background setup and readback process are handled by the
[dedicated AIRBAND installer](UV-5RM%20Custom%20Firmware/airband/README.md#install-on-the-5rhbk-radio).

**1. Back up the radio.** Flash mode can't read it, so do this first, with the radio on and on
its main screen. This only reads. It may not answer until the radio has been power-cycled, and
the folder has to exist.

```
mkdir my-backup
py source-changes\radio_read.py COM3 my-backup\factory_memory.img
```

**2. Enter flash mode.** Turn the radio off, hold **both side buttons** and turn it on. The screen
stays black. Check that it answers (this doesn't write anything):

```
py source-changes\bootloader_probe.py COM3
```

**3. Flash.** This is the point of no return. It takes under a minute, so don't touch the radio or
the cable. The long number is a checksum, and the tool refuses a file that doesn't match it.

```
py source-changes\flash_bf.py UV-5RM_Custom_Firmware.BF COM3 4340f205430f0741
```

**4. Install the blank artwork and apps.** Wait for the radio to restart to its
main screen. This reads everything back to confirm it. The logo-path argument
is retained for command compatibility but the current tool installs blank data.

```
py source-changes\setup_radio.py COM3 unused-logo-argument 64 apps
```

**5. Turn on transmit when ready to test it.** Go to `SETTINGS` → `RADIO` → `TXINH`.
Press **Menu**, choose off, then press **Menu** again to save; **Exit** cancels.
The receive-only bring-up build cannot transmit even with TXINH off.

**6. Load channels (optional).** Open `UV-5RM_Channel_Uploader.html` in Chrome or Edge. The channel
list in this repo is for Clark County, Nevada, so check that it suits your area. Your radio should
keep whatever channels it already had.

To choose which channels get scanned, open `UV-5RM_Scan_Builder.html`, tick the ones you want, and
load the CSV it saves into the uploader.

Optional extras: `set_boot_tune.py` sets a Doom-style start-up riff, and `set_radio2_mario_tune.py`
sets a Super Mario one but only runs on one specific radio, so copy and edit it for yours.

## Using TEXT

Both radios need `TEXT` open, on the same simplex frequency. It uses the frequency the radio was on
when you opened it. Don't use a repeater.

| Key | Action |
|---|---|
| Number keys | Type, flip-phone style; the letters are shown at the top of the screen |
| **Menu** | Send |
| **Exit** | Delete a letter; stops retries while sending; closes the app when nothing is typed |
| Hold **Exit** | Close the app |
| **Up / Down** | Scroll saved messages |
| **\*** | Send a numbered test ping |
| **A/B** | Switch between low (default) and high power |
| Hold **#** twice | Erase all saved messages |

| Mark | Meaning |
|---|---|
| `>` | Sent, waiting for a reply |
| `+` | The other radio confirmed it |
| `!` | Never confirmed. It may still have arrived, since only the reply may have been lost |
| `<` | Received |

**Limits:**
- A radio only receives while `TEXT` is open.
- It's built for two radios; with more, the automatic replies would collide.
- Messages have **no encryption**, and they're stored unencrypted on the radio. Scrambling isn't
  security, so any radio with this app can read them.
- Range and long messages have barely been tested. Most testing was short pings a few feet apart.

## Legal

You're responsible for what you transmit.

- **`TEXT` transmits on its own.** With the app open, the radio sends a reply automatically to any
  message it receives, so stay on a frequency you're licensed for.
- **Ham bands:** you need a license and must identify with your callsign. `TEXT` doesn't send one,
  so you'd have to identify by voice. Encrypting messages isn't allowed.
- **FRS, GMRS and MURS:** this radio isn't type-accepted for them, so treat those channels as
  receive-only. Rules also vary by country.
- **Power:** output hasn't been measured, and the radio's actual power may differ from its label.

## Building from source

`firmware-source` is the complete source of the provided firmware and apps; the firmware and `TEXT`
were confirmed to rebuild byte for byte. It needs [Rust](https://rustup.rs) and the target in its
`rust-toolchain.toml`. Use a short `--target-dir`, because long paths break linking on Windows.

```
cargo build --release --manifest-path firmware-source/Cargo.toml --config firmware-source/.cargo/config.toml --target-dir C:/t/fw
py source-changes/elf2bin.py C:/t/fw/thumbv7em-none-eabi/release/bf5rh-fw fw.bin
```

`source-changes` holds the patches against upstream and the PC tools. `source-changes/simulator` runs
the apps against a simulated radio, with no hardware needed:

```
cargo build --release --manifest-path source-changes/simulator/Cargo.toml --target x86_64-pc-windows-msvc --target-dir C:/t/sim
C:/t/sim/x86_64-pc-windows-msvc/release/sim.exe out-folder any-160x128.rgb565
```

The output folder has to exist. The simulator only checks logic, so it can't prove how a real radio
behaves on the air.

## Credits and license

- Firmware base: [bf-5rh-firmware-open](https://github.com/sophiel-meow/bf-5rh-firmware-open)
- Modem settings researched from the UV-K5 community's
  [messenger firmware](https://github.com/kamilsss655/uv-k5-firmware-custom)
- Licensed under GPL-3.0, the same as the firmware it's built on. See `LICENSE`.

Provided as is, with no warranty.

## AIRBAND profile

The dedicated [AIRBAND app and install guide](UV-5RM%20Custom%20Firmware/airband/README.md)
include a nationwide FAA database and a separate BK4819 firmware profile. The app
is receive-only, occupies the former FOX HUNT slot, and opens from the first
item in the Band / SCAN frequency-mode list. Airport/VOR IDs use an arrow-driven
A–Z and 0–9 editor. Database updates are separate
from firmware updates. Nearby lists use an explicitly selected airport center.
The [future automatic location design](docs/airband-location-future/README.md)
is a proposal; signal-based automatic location is not implemented.

The final profile was installed from macOS on a `5RHBK V0.14 BF` radio and
verified by direct readback. The app image matched the release artifact, stored
artwork was blank, protected radio data was unchanged, and TXINH remained on.
The programming cable must be unplugged for normal internal-speaker audio.

AIRBAND does not calculate a VOR radial. That requires access to the relative
phase of the VOR navigation components, including the 9,960 Hz subcarrier path;
the current app API exposes RSSI and squelch but no received-baseband samples.
The future location proposal likewise does not claim triangulation or a
navigation-quality position.

## Recommended repository and contribution workflow

This checkout currently follows `jakegarrett13-afk/UV-5RM-Radio-Mod` and the
AIRBAND/BK work is much larger than a routine fix. Preserve it in your own fork
and feature branch first; then offer it upstream as a draft PR. Do not push the
experimental work directly to upstream `main`, and do not base it on PR #3's
branch because this tree already incorporates and documents the relevant PR #3
findings independently.

Recommended sequence:

1. Create a local branch from the current working tree, for example
   `airband-bk4819`. Switching to a new branch does not discard uncommitted work.
2. Review `git status` carefully. The ignored `backup/`, `my-backup/` and
   `stock-recovery/` directories contain radio-specific material and must never
   be committed. Inspect any unexpected binary before staging it.
3. Commit the source, tests, documentation and intentional release artifacts.
   Prefer coherent commits for BK4819/platform support, AIRBAND app/database
   support, and documentation/release files. If separating them safely would
   risk losing the tested state, make one clearly labeled checkpoint commit
   first and refine history only on a second review branch.
4. Create a GitHub fork, add it as a separate remote (normally `fork`), and push
   the feature branch there. Keep `origin` pointing at the upstream repository
   so future upstream changes are easy to inspect and merge.
5. Publish a tagged prerelease from the fork, such as `airband-bk4819-v0.1.0`,
   with the three SHA-256 hashes above, the supported boot identifier, the FAA
   cycle, known limitations, and a link to the detailed guide.
6. Open an upstream issue or draft PR that explains the dependency on BK4819
   support and links to the existing PR #3. Ask whether maintainers prefer one
   stacked series or separate PRs before spending time rewriting history.

For this checkout, the immediate local checkpoint can be made with:

```sh
git switch -c feature/airband-bk4819
git status --short
git diff --check
# Review every listed path before staging. The release .BF, .app and .db are intentional.
git add -A
git diff --cached --check
git commit -m "Add BK4819 AIRBAND profile"
```

After creating a fork through GitHub, keep this repository's current `origin` as
the upstream remote and add the fork separately:

```sh
git remote add fork https://github.com/YOUR-NAME/UV-5RM-Radio-Mod.git
git push -u fork feature/airband-bk4819
```

Open the first pull request as **Draft** from
`YOUR-NAME:feature/airband-bk4819` to `jakegarrett13-afk:main`. If PR #3 merges
first, fetch upstream and rebase or merge its new `main` into a separate review
branch, resolve the overlapping BK driver work there, rerun every test, and keep
the original hardware-tested branch/tag untouched as a recovery point.

A practical PR series, if the maintainer wants smaller reviews, is:

1. BK4819 board support, calibration/backup safeguards and driver tests.
2. AIRBAND firmware capability, flash allocation and fixed menu routing.
3. AIRBAND app, FAA database builder, installer, tests and documentation.
4. Plain-background UI changes, if the maintainer considers that a separate
   policy choice rather than part of AIRBAND.

Before publishing, rerun the complete AIRBAND test command, verify the recorded
hashes, and test the final physical UI and known AM station. Report failures as
hardware/model-specific observations, including the hold-8 identifier, compiler
version, artifact hash and exact reproduction steps.
