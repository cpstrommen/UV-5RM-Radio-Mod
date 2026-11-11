"""Read stock configuration using CHIRP's BF5RM download sequence.

Only the stock handshake and 64-byte 'R' requests can leave this tool.
This does not back up MCU firmware. --calibration additionally saves the
0xF000..0xF2FF stock calibration area to a separate file, with an address map.
--verify reads each selected region twice and requires identical results.
Power-cycle normally before running; power-cycle again before other tools.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import time

import serial

IDENT = b"PROGRAMBFNORMALU"
MAGICS = [(b"\x46", 16), (b"\x4d", 15),
          (b"\x53\x45\x4E\x44\x21\x05\x0D\x01\x01\x01\x04\x11\x08\x05"
           b"\x0D\x0D\x01\x11\x0F\x09\x12\x09\x10\x04\x00", 1)]
REGIONS = [(0x0000, 0x8040), (0x9000, 0x0040),
           (0xA000, 0x02C0), (0xD000, 0x0040)]
CALIBRATION = [(0xF000, 0x0300)]
BLOCK = 0x40
KEY = b"CO 7"
HANDSHAKE = {IDENT} | {m for m, _ in MAGICS}


def crypt(buf):
    out = bytearray()
    for i, b in enumerate(buf):
        k = KEY[i % 4]
        if k != 32 and b not in (0, 255) and b != k and b != k ^ 255:
            out.append(b ^ k)
        else:
            out.append(b)
    return bytes(out)


def send(port, frame):
    is_read = (len(frame) == 4 and frame[0] == 0x52
               and frame[3] == BLOCK
               and int.from_bytes(frame[1:3], "big") % BLOCK == 0)
    if frame not in HANDSHAKE and not is_read:
        raise ValueError(f"REFUSED to send {frame[:8].hex()}")
    port.write(frame)


def exact(port, count):
    data = bytearray()
    deadline = time.monotonic() + 2.0
    while len(data) < count and time.monotonic() < deadline:
        chunk = port.read(count - len(data))
        if not chunk:
            break
        data.extend(chunk)
    if len(data) != count:
        raise RuntimeError(f"Short response: expected {count}, got {len(data)} bytes")
    return bytes(data)


def identify(port):
    port.reset_input_buffer()
    send(port, IDENT)
    if exact(port, 1) != b"\x06":
        raise RuntimeError("Radio did not acknowledge the stock handshake")
    responses = []
    for magic, count in MAGICS:
        send(port, magic)
        reply = exact(port, count)
        if count == 1 and reply != b"\x06":
            raise RuntimeError("Radio refused the stock read session")
        responses.append(reply.hex())
    return responses


def read_regions(port, regions, decrypt=True):
    data = bytearray()
    for start, size in regions:
        if start < 0 or size <= 0 or start % BLOCK or size % BLOCK or start + size > 0x10000:
            raise ValueError("Read regions must be aligned and within the 16-bit address space")
        for addr in range(start, start + size, BLOCK):
            request = struct.pack(">BHB", 0x52, addr, BLOCK)
            send(port, request)
            reply = exact(port, BLOCK + 4)
            if reply[:4] != request:
                raise RuntimeError(f"Unexpected read header at {addr:#06x}: {reply[:4].hex()}")
            data.extend(crypt(reply[4:]) if decrypt else reply[4:])
    return bytes(data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("port")
    parser.add_argument("output", type=Path)
    parser.add_argument("--calibration", action="store_true")
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    paths = [args.output, args.output.with_suffix(".json")]
    if args.calibration:
        paths.append(args.output.with_suffix(".calibration.bin"))
    for path in paths:
        if path.exists():
            parser.error(f"Refusing to overwrite existing backup: {path}")
    with serial.Serial(args.port, 115200, timeout=0.5,
                       rtscts=False, xonxoff=False) as port:
        identity = identify(port)
        config = read_regions(port, REGIONS)
        # Stock calibration reads are plaintext, confirmed against the
        # custom firmware's direct SPI-flash reads on the BK v0.14 radio.
        calibration = read_regions(port, CALIBRATION, decrypt=False) if args.calibration else None
        if args.verify:
            if read_regions(port, REGIONS) != config:
                raise RuntimeError("Configuration differed on the second read; no backup saved")
            if calibration is not None and read_regions(port, CALIBRATION, decrypt=False) != calibration:
                raise RuntimeError("Calibration differed on the second read; no backup saved")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    manifest = {"port": args.port, "identity_hex": identity,
                "verified_second_read": args.verify, "contains_mcu_firmware": False,
                "files": {}}
    for path, data, regions in [(args.output, config, REGIONS),
                                (args.output.with_suffix(".calibration.bin"), calibration, CALIBRATION)]:
        if data is None:
            continue
        with path.open("xb") as f:
            f.write(data)
        offset = 0
        mapping = []
        for start, size in regions:
            mapping.append({"radio_address": start, "file_offset": offset, "bytes": size})
            offset += size
        manifest["files"][path.name] = {"bytes": len(data), "regions": mapping,
                                       "wire_encoding": "plaintext" if regions == CALIBRATION else "CO 7 XOR",
                                       "sha256": hashlib.sha256(data).hexdigest()}
        print(f"saved {len(data)} bytes -> {path}")
    with args.output.with_suffix(".json").open("x") as f:
        json.dump(manifest, f, indent=2)
        f.write("\n")
    print("Power-cycle the radio before the next operation.")


if __name__ == "__main__":
    main()
