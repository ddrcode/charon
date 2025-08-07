# QMK Integration

This document explains how to make maximum of QMK-powered keyboard from
Charon perspective. It all assumes that [Raw HID](https://docs.qmk.fm/features/rawhid)
is enabled on the keyboard, so Charon can extract additional information from it.

## Protocol

Charon proposes a trivial *protocol* used for exchanging data with QMK.
As per specification each data packet is 32-bytes long. We reserve byte 0
as function identifier. The 256 functions limit should be more than enough
for Charon purposes. Here is the description of currently used protocol.

|byte[0]|data bytes|Name|Description|
-------------------------------------
|0x00|-|reserved|no action|
|0x01|[1-31]|ping/echo|sends back the same data|
|0x02|[1-2]|layer_change|byte 1 - current layer, byte 1: 1 if default layer, 0 otherwise|
|0x03|[1-2]|key_event|[1:2]: key code, [3]: release (0) or press (1)|
