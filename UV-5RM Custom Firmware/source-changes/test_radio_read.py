"""Offline checks for stock backup corruption and the read-only send guard."""
import unittest
from unittest.mock import patch
import radio_read as r


class Port:
    def __init__(self, reply=b'', split=1000):
        self.reply, self.split, self.sent = reply, split, []

    def write(self, data):
        self.sent.append(data)

    def read(self, count):
        count = min(count, self.split)
        data, self.reply = self.reply[:count], self.reply[count:]
        return data


class BackupTests(unittest.TestCase):
    def test_rejects_writes_before_touching_port(self):
        port = Port()
        for frame in [b'W\xf0\x00\x40', b'R\xf0\x00\x20', b'R\xf0\x01\x40', b'BOOTLOADER']:
            with self.assertRaises(ValueError):
                r.send(port, frame)
        self.assertEqual(port.sent, [])

    def test_partial_serial_chunks_and_decryption(self):
        payload = bytes(range(64))
        frame = b'R\xf0\x00\x40' + r.crypt(payload)
        port = Port(frame, split=3)
        self.assertEqual(r.read_regions(port, [(0xf000, 64)]), payload)
        self.assertEqual(port.sent, [b'R\xf0\x00\x40'])

    def test_wrong_address_is_not_a_backup(self):
        with self.assertRaisesRegex(RuntimeError, 'Unexpected read header'):
            r.read_regions(Port(b'R\xf0\x40\x40' + bytes(64)), [(0xf000, 64)])

    def test_calibration_is_plaintext(self):
        payload = bytes(range(64))
        port = Port(b'R\xf0\x00\x40' + payload)
        self.assertEqual(r.read_regions(port, [(0xf000, 64)], decrypt=False), payload)

    def test_truncated_reply_is_not_a_backup(self):
        with self.assertRaisesRegex(RuntimeError, 'Short response'):
            r.read_regions(Port(b'R\xf0\x00\x40' + bytes(63)), [(0xf000, 64)])

    def test_existing_backup_rejected_before_opening_port(self):
        from tempfile import TemporaryDirectory
        from pathlib import Path
        with TemporaryDirectory() as folder:
            out = Path(folder) / 'stock.img'
            out.write_bytes(b'original')
            with patch('sys.argv', ['radio_read.py', 'fake', str(out)]), patch.object(r.serial, 'Serial') as serial:
                with self.assertRaises(SystemExit):
                    r.main()
                serial.assert_not_called()
            self.assertEqual(out.read_bytes(), b'original')

    def test_changed_second_read_saves_nothing(self):
        from tempfile import TemporaryDirectory
        from pathlib import Path
        from contextlib import nullcontext
        with TemporaryDirectory() as folder:
            out = Path(folder) / 'stock.img'
            with patch('sys.argv', ['radio_read.py', 'fake', str(out), '--verify']), patch.object(r.serial, 'Serial', return_value=nullcontext(Port())), patch.object(r, 'identify', return_value=[]), patch.object(r, 'read_regions', side_effect=[b'a', b'b']):
                with self.assertRaisesRegex(RuntimeError, 'differed'):
                    r.main()
            self.assertFalse(out.exists())


if __name__ == '__main__':
    unittest.main()
