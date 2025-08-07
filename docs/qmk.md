# QMK Integration

This document explains how to make maximum of QMK-powered keyboard from
Charon perspective. It all assumes that [Raw HID](https://docs.qmk.fm/features/rawhid)
is enabled on the keyboard, so Charon can extract additional information from it.

## Protocol

Charon proposes a trivial *protocol* used for exchanging data with QMK.
As per specification each data packet is 32-bytes long. We reserve byte 0
as function identifier. The 256 functions limit should be more than enough
for Charon purposes. Here is the description of currently used protocol.

| Byte 0  | Data bytes | Name         | Description                                                |
|---------|------------|--------------|------------------------------------------------------------|
| 0x00    | -          | (reserved)   | No action                                                  |
| 0x01    | [1-31]     | ping/echo    | Returns same value; testing mechanissm                     |
| 0x02    | [1-2]      | layer change | byte 1: layer id, byte 2: `1` if default layer, `0` otherwise |
| 0x03    | [1-3]      | key event    | [1-2]: key id, [3] state (`1`: pressed, `0`: released)       |
