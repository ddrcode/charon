#!/usr/bin/env python3
import evdev
import os
import struct
import subprocess
import time

def nvim():
    print("F7 pressed → Entering Vim mode!")

    # Pause pass-through by ignoring real keyboard for now
    # (your loop naturally pauses here)

    # 1) Launch nvim to edit a temp file
    subprocess.run(['nvim', '/tmp/interposer.txt'])

    # 2) Read the file contents
    with open('/tmp/interposer.txt', 'r') as f:
        text = f.read()

    # 3) Pipe each character as HID keycodes
    for char in text:
        # Example: simple ASCII mapping (add full map for real usage!)
        if 'a' <= char <= 'z':
            mod = 0
            code = ord(char) - ord('a') + 4
        elif 'A' <= char <= 'Z':
            mod = 0x02  # Left Shift
            code = ord(char) - ord('A') + 4
        elif char == '\n':
            mod = 0
            code = 0x28  # Enter
        else:
            # TODO: add more mappings (space, symbols, etc)
            continue

        report = bytearray(8)
        report[0] = mod  # Modifier
        report[1] = 0    # Reserved
        report[2] = code  # Keycode 1
        # Keycode slots 3-8 remain zero

        os.write(hidg, report)
        time.sleep(0.005)  # mimic natural typing
        # Send key release
        report[0] = 0
        report[2] = 0
        os.write(hidg, report)

    print("Done! Vim output sent. Resuming pass-through.")
    # Loop automatically resumes pass-through!

# ---- CONFIG ----

# Replace with your physical keyboard's event file.
# Run `ls /dev/input/by-id/` to find something like: usb-YourKeyboard-event-kbd
KEYBOARD_EVENT = "/dev/input/event5"

# Your gadget output device (created by configfs)
HIDG_DEVICE = "/dev/hidg0"

# ---- OPEN DEVICES ----

# Open the real keyboard as an input device
keyboard = evdev.InputDevice(KEYBOARD_EVENT)
print(f"Listening to: {keyboard.name} ({keyboard.path})")

# Open the gadget HID endpoint for raw writes
hidg = os.open(HIDG_DEVICE, os.O_WRONLY | os.O_SYNC)

# ---- REPORT DESCRIPTOR (same as you used) ----
# Modifier byte + reserved + 6 keycodes
# Format: [1B mods][1B reserved][6B keycodes]
report = bytearray(8)

# Keep track of pressed keys for rollover
pressed_keys = set()

# Map evdev key codes to HID usage codes
# Example for standard keys; you may need to expand this!
EVDEV2HID = {
    evdev.ecodes.KEY_A: 4,
    evdev.ecodes.KEY_B: 5,
    evdev.ecodes.KEY_C: 6,
    evdev.ecodes.KEY_D: 7,
    evdev.ecodes.KEY_E: 8,
    evdev.ecodes.KEY_F: 9,
    evdev.ecodes.KEY_G: 10,
    evdev.ecodes.KEY_LEFTSHIFT: 0x02,  # Bit 1 for Left Shift
    evdev.ecodes.KEY_RIGHTSHIFT: 0x20, # Bit 5 for Right Shift
    # ... add more as needed
}

NVIM_TRIGGER = evdev.ecodes.KEY_F7

print("Starting pass-through loop...")

try:
    for event in keyboard.read_loop():
        if event.type != evdev.ecodes.EV_KEY:
            continue

        key_event = evdev.categorize(event)
        code = key_event.scancode
        value = key_event.keystate

        if code == NVIM_TRIGGER and value == 1:
            nvim()
            continue

        if code not in EVDEV2HID:
            continue

        # Handle modifiers vs normal keys
        if code in [evdev.ecodes.KEY_LEFTSHIFT, evdev.ecodes.KEY_RIGHTSHIFT]:
            if value:
                report[0] |= EVDEV2HID[code]
            else:
                report[0] &= ~EVDEV2HID[code]
        else:
            hid_code = EVDEV2HID[code]
            if value:  # Key down
                pressed_keys.add(hid_code)
            else:      # Key up
                pressed_keys.discard(hid_code)

        # Fill 6 slots with current pressed keys
        keycodes = list(pressed_keys)[:6]
        while len(keycodes) < 6:
            keycodes.append(0)

        report[2:] = keycodes

        # Write report
        os.write(hidg, report)

        # Debug print
        print(f"Report: {list(report)}")

except KeyboardInterrupt:
    print("\nExiting...")
    os.close(hidg)


