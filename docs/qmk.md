# QMK Integration

This document explains how to make maximum of QMK-powered keyboard from
Charon perspective. It all assumes that [Raw HID](https://docs.qmk.fm/features/rawhid)
is enabled on the keyboard, so Charon can extract additional information from it.

## Charon configuration

To make Charon recognize your keyboard, you must identify its `vendor id` and `product id`,
as Charon will be looking for this specific device.
There are various ways of finding the IDs:

- Call `lsusb` and identify your keyboard. Left to keyboard name you should see two hex
  numbers separated by colon. The first bit is vendor id, and the second - product id.
- In QMK project find `keyboard.json` of your keyboard and search for `vid` and `pid` values.

Once you know the IDs, edit `keyboards` section of Charon's config file. It may look like that:

```toml

[keyboard]
Use = "Keychron_Q10"


[keyboards]

[keyboards.Keychron_Q10]
vendor_id = 0x3434
product_id = 0x01A1
raw_hid_enabled = true
devices = [
    { name = "usb-Keychron_Keychron_Q10-event-kbd", alias="KeychronQ10" },
    { name = "usb-Keychron_Keychron_Q10-event-if02", alias="KeychronQ10-knob", optional = true }
]

[keyboards.Keychron_Q3]
devices = [
    { name = "usb-Keychron_Q3_keychron_q3_ansi-event-kbd", alias="KeychronQ10" },
]
```

Pay attention to `alias` as this is how the keyboard will be visible in all the statistics.
If you change the alias, all telemetry for new name will start from scratch.

The `raw_hid_enabled` flag indicates that Raw HID is enabled on QMK side.

Charon will start QMK Actor only if `vendor_id` and `product_id` are provided and
`raw_hid_enabled` flag is set to `True`.

If, out of madness, you connect more than one Raw HID-enabled QMK keyboards to Charon,
it will handle them with no problem. The daemon will start multiple QMK actors, each listening to
a different device, and each event will have keyboard signature (the alias). How cool is that!?


## QMK Configuration

There are two steps that need to be done on your keyboard side
1. Raw HID must be enabled.
2. Raw HID must be used a specific way, so Charon will understand messages coming from
   the keyboard.

For the first point - see [QMK documentation](https://docs.qmk.fm/features/rawhid).

In terms of Charon-specific Raw HID code: we have a
[library](../setup/qmk/charon.c) you could use.
Follow [QMK setup section](../setup/qmk/) for details.

## Development

If you want to expand current features, either on QMK or Charon side, this section explains how
the integration is implemented. You are more than welcome to contribute your changes to Charon.

### Protocol

Charon uses a trivial *protocol* for exchanging data with QMK.
As per specification each data packet is 32-bytes long. We reserve byte 0
as function identifier. The 256 functions limit should be more than enough
for Charon purposes. Here is the description of currently used protocol.

| Byte 0  |  Name          | Description                                                | Status |
|---------|--------------|------------------------------------------------------------|--------|
| 0x00    |(reserved)    | No action                                                  ||
| 0x01    |ping/echo     | Echoes same value; testing mechanism                       ||
| 0x02    |layer change  | informs about layer change                                 |✅|
| 0x03    |key event     | [1-2]: key id, [3] state (`1`: pressed, `0`: released)       |✅|
| 0x03    |keyboard info | [1]: num of cold, [2]: now of rows       ||
| 0x04    |change Charon mode|Make Charon to switch to a specific mode||
| 0x05    |Toggle Charon mode|Make Charon to toggle mode||
| 0x10    |layer chunk   | given layer keymap (sent in chunks) ||



### Endianness

For consistency, assume that all numbers (like key id) are always being sent in little-endian format,
regardless the endianness of the QMK devices and the host. On Charon side always encode/decode numbers
with [`to_le_bytes`](https://doc.rust-lang.org/std/primitive.f16.html#method.to_le_bytes) and
[`from_le_bytes`](https://doc.rust-lang.org/std/primitive.f16.html#method.from_le_bytes)
respectively when using the protocol.

### QMK functions

This section describes in detail functions described in Protocol table

#### Change Charon mode (`0x04`) and Toggle Charon mode (`0x05`)

```c
void charon_change_charon_mode(uint8_t mode);
void charon_toggle_charon_mode();
```
Protocol (change function only):

| Byte | arg name | Description | 
|------|----------|-------------|
| 1    | mode     | `0`: pass-through mode, `1`: in-app mode |

Requests Charon to switch to a specific mode:
0. `pass-through` mode: keys being send to the host computer
1. `in-app` mode: Charon takes control; the client app shows menu

##### Usage 

One potential use of this function is to control Charon's state with custom keycode, without a need
for a dedicated keyboard shortcut.
In such case Charon will be controlled only via Raw HID, so every non-QMK/RAW keyboard connected
to Charon won't be capable of controlling modes. To achieve this, define a custom
keycode and catch in `process_record_user` function, as shown in the example below:

```c
// In your keymap.c or <profile>.c file
#include "charon.h"

enum CustomKeycodes {
    CHARON_TOGGLE_MODE = SAFE_RANGE // first available keycode
};

bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    if (record->event.pressed) {
        switch (keycode) {
            case CHARON_TOGGLE_MODE:
                charon_toggle_charon_mode();
                return false;  // stop further processing
            // handle other cases
        }
    }
    return true;
}
```

