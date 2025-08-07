# QMK Integration

This document explains how to make maximum of QMK-powered keyboard from
Charon perspective. It all assumes that [Raw HID](https://docs.qmk.fm/features/rawhid)
is enabled on the keyboard, so Charon can extract additional information from it.

## Development 

If you want to expand current features, either or QMK or Charon side, this section explain how
the integration is implemented. you are more than welcome to contribute your changes to Charon.

### Protocol

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


### Endianness

For consistency, assume that all numbers (like key id) are always being sent in little-endian format,
regardless the endianness of the QMK devices and the host. On Charon side always encode/decode numbers 
with [`to_le_bytes`](https://doc.rust-lang.org/std/primitive.f16.html#method.to_le_bytes) and
[`from_le_bytes`](https://doc.rust-lang.org/std/primitive.f16.html#method.from_le_bytes) 
respectively when using the protocol. 
