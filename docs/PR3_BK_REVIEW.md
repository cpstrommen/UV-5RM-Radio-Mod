# BK build cross-check against PR #3

Reviewed [PR #3](https://github.com/jakegarrett13-afk/UV-5RM-Radio-Mod/pull/3)
at head `80b5939cbe81b63d0355af8459a4731cba8eaa78` from
`pkkshadow/UV-5RM-Radio-Mod`. This review does not merge or publish that PR.

The approaches agree on the important RF-chip distinction. The final local
candidate incorporates the missing BK board details identified by the PR while
retaining the dedicated BK driver and the configuration-backup fixes developed
in this workspace.

## Findings and decisions

| Area | Cross-check | Final local behavior |
|---|---|---|
| RF power / AGC / PLL init | Both use `1D0F` / `1F0F`, BK AGC values, `REG_1F=5454`, `REG_3E=A037`, and omit BK4829-only setup | Retained |
| MCU and front-end pins | PR agrees with the existing PC13/PA2/PA3 bus and GPIO3/4 TX, GPIO5/6 RX switches | Retained UV-5RH board wiring |
| BK init tail | Original local port omitted GPIO2 output-low and `REG_34=0200`; stock code confirms them | Added both |
| Low TX power | Original local port always used `D7`; PR identifies the stock `C3` exception at 155 MHz inclusive through 175 MHz exclusive | Added frequency-aware PA gain and boundary tests; calibrated APC upper byte preserved |
| MCU clock | Original local code could wait forever for HEXT | Adopted bounded wait and HICK fallback; both paths target 120 MHz |
| PLL reference range | Vendor library distinguishes the 8 MHz external-derived reference from 4 MHz HICK/2 | Explicitly select the corresponding reference range in the BK build |
| Microphone | PR retains `E920`; stock BK writes `E952` | Retained `E952`, then applies the radio's microphone calibration |
| Deviation | PR retains `3516`; stock BK init uses `04D2`, and its normal unscrambled path selects `04E0` while preserving the high nibble | Corrected init to `04D2`; retained stock-like normal `04E0` behavior |
| PA off | PR retains `0022`; stock BK uses `007F` | Retained stock BK `007F` |
| RX filters / AM AGC | PR retains the original driver's RX filter override and identical FM/AM AGC values | Retained the upstream BK-family driver recipe used in the successful NOAA test; these differences are not evidence of a defect |
| Chip guard | Local code verifies `4819` before RF initialization | Retained; compact mismatch path drops the power latch before RF initialization |
| Backup handling | PR does not address the stock calibration read encoding | Retained plaintext calibration export, read-header validation, duplicate-read verification, overwrite protection and address/hash manifest |

## Independent stock-image evidence

The inspected stock `5RH_BK_V0.14_231123.BF` came from
[amoxu's reverse-engineering repository](https://github.com/amoxu/Baofeng-UV-5RM-5RH-RE/blob/main/factory_firmware/BK_BK4819/5RH_BK_V0.14_231123.BF).
Its SHA-256 is
`2b9aa883196cbc57c12e1ec5b62ccd062dc589253d5cfacb50eef6faec171441`.
Addresses below refer to decrypted region 1 loaded at `0x08001000`:

- RF initialization: `0x08008BF0`; microphone write at `0x08008D5E`, initial
  deviation constant at `0x08008D34`, BK-only GPIO2 / REG_34 tail at
  `0x08008DA4..0x08008DB6`.
- PA gain selection: `0x08009EDC`; literal pool points to gain table
  `{FF, D7, FF}` at `0x0800E20C`. The low-power exception compares frequency
  in 10 Hz units against `15,500,000` and `17,500,000`, with unsigned lower
  inclusive / upper exclusive comparisons, then ORs in `C3`.
- Normal unscrambled deviation: `0x08009F68` ORs `04E0` into the preserved
  high nibble of REG_40.

The MCU vendor's
[clock driver](https://github.com/ArteryTek/AT32F421_Firmware_Library/blob/master/libraries/drivers/src/at32f421_crm.c)
and [clock definitions](https://github.com/ArteryTek/AT32F421_Firmware_Library/blob/master/libraries/drivers/inc/at32f421_crm.h)
confirm multiplier encodings 13 = x15 and 29 = x30, HICK/2 as the internal PLL
reference, and reference-range selections 2 for 8 MHz and 0 for 4 MHz.
The fallback path is tested in simulation; it has not been forced on the radio.

## Build and validation

- Rust 1.99.1; release, locked dependencies, `--features bk4819`.
- Application: **61,404 bytes**, leaving **36 bytes** within the 60 KiB limit.
- Final package: `UV-5RM Custom Firmware/UV-5RM_Custom_Firmware_BK4819.BF`.
- SHA-256: `0bfffa6596fb4408715393cf46106ccacfa2027f03626b8788209aae76d9c1bf`.
- Four real-clock-function tests with simulated MCU registers cover immediate,
  delayed, absent and excessively late HEXT startup. Both selected PLL paths
  calculate to 120 MHz and the absent-crystal path terminates.
- Three RF driver tests cover BK initialization and PA-off state, board band
  switches, and PA gain boundaries at 155 / 175 MHz for all power levels.
- Seven backup-tool tests pass.
- The default YP07 application still rebuilds byte-for-byte to the original
  61,352-byte application. Its wrapped BF hash remains
  `4340f205430f0741e15a8e0dd28057985efff81e8dec68392affae7611fb2985`.
- The final BF wrapper cross-checks its transform against seven stock images.

The earlier receive-only image has demonstrated menus, app installation and
clear NOAA reception on this user's BK radio. PR #3 separately reports RX/TX on
another radio; that is useful corroboration, not a TX test of this candidate.
This candidate restores TX capability behind the existing TXINH setting. Output
power, frequency accuracy, other bands/modes and TEXT between radios still need
hardware testing. Stock restoration also remains unverified.

## Installation on this radio

The reviewed candidate flashed in 8.1 seconds, and the bootloader acknowledged
completion. It booted and answered the custom CPS handshake. Full readback
verified the logo and all nine apps, and the channel table and calibration still
match the original backups. TXINH was confirmed on. The user confirmed that
NOAA reception on 162.550 MHz still works after unplugging the programming cable.
No transmission was initiated during installation.
