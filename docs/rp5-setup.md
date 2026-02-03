# Raspberry Pi 5 Setup Guide

This guide walks you through setting up a Raspberry Pi 5 as a USB HID gadget for Charon.

---

## Prerequisites

### Hardware

- Raspberry Pi 5
- USB-C cable (for connecting RP5 to the host computer)
- MicroSD card (16GB+ recommended)
- USB keyboard (the one you want to pass through)
- Optional: small display for the TUI client

### Software

- Raspberry Pi OS Lite (64-bit) - Bookworm or later
- Rust toolchain (installed during setup)

---

## Step 1: Flash Raspberry Pi OS

1. Download [Raspberry Pi Imager](https://www.raspberrypi.com/software/)
2. Flash **Raspberry Pi OS Lite (64-bit)** to your SD card
3. In the imager settings, enable SSH and configure WiFi if needed
4. Boot the Pi and SSH in

---

## Step 2: Configure USB Gadget Mode

The RP5 needs to act as a USB HID device (keyboard) to the host computer. This requires enabling the `dwc2` USB driver in gadget mode.

> **Alternative:** For a simpler setup, check out [HIDPi](https://github.com/rikka-chunibyo/HIDPi) which automates most of the gadget configuration. The manual steps below are provided for reference and customization.

### Enable dwc2 overlay

Edit `/boot/firmware/config.txt`:

```bash
sudo nano /boot/firmware/config.txt
```

Add at the end:

```ini
dtoverlay=dwc2
```

### Load dwc2 module at boot

Edit `/etc/modules`:

```bash
sudo nano /etc/modules
```

Add:

```
dwc2
libcomposite
```

### Reboot

```bash
sudo reboot
```

---

## Step 3: Create HID Gadget Configuration

Create a script to configure the USB gadget. This sets up a composite device that presents itself as a keyboard to the host.

Create `/usr/local/bin/charon-gadget`:

```bash
sudo nano /usr/local/bin/charon-gadget
```

```bash
#!/bin/bash

# Charon USB HID Gadget Setup
# Based on https://github.com/rikka-chunibyo/HIDPi

GADGET_DIR="/sys/kernel/config/usb_gadget/charon"

# Exit if already configured
if [ -d "$GADGET_DIR" ]; then
    echo "Gadget already configured"
    exit 0
fi

# Load libcomposite if not loaded
modprobe libcomposite

# Create gadget directory
mkdir -p "$GADGET_DIR"
cd "$GADGET_DIR"

# USB IDs (use your own or keep these generic ones)
echo 0x1d6b > idVendor  # Linux Foundation
echo 0x0104 > idProduct # Multifunction Composite Gadget
echo 0x0100 > bcdDevice # v1.0.0
echo 0x0200 > bcdUSB    # USB 2.0

# Device strings
mkdir -p strings/0x409
echo "fedcba9876543210" > strings/0x409/serialnumber
echo "Charon" > strings/0x409/manufacturer
echo "Charon Keyboard" > strings/0x409/product

# Create HID function (keyboard)
mkdir -p functions/hid.usb0
echo 1 > functions/hid.usb0/protocol      # Keyboard
echo 1 > functions/hid.usb0/subclass      # Boot interface subclass
echo 8 > functions/hid.usb0/report_length # 8-byte reports

# HID report descriptor for a standard keyboard
echo -ne '\x05\x01\x09\x06\xa1\x01\x05\x07\x19\xe0\x29\xe7\x15\x00\x25\x01\x75\x01\x95\x08\x81\x02\x95\x01\x75\x08\x81\x03\x95\x05\x75\x01\x05\x08\x19\x01\x29\x05\x91\x02\x95\x01\x75\x03\x91\x03\x95\x06\x75\x08\x15\x00\x25\x65\x05\x07\x19\x00\x29\x65\x81\x00\xc0' > functions/hid.usb0/report_desc

# Create configuration
mkdir -p configs/c.1/strings/0x409
echo "Config 1: HID Keyboard" > configs/c.1/strings/0x409/configuration
echo 250 > configs/c.1/MaxPower

# Link function to configuration
ln -s functions/hid.usb0 configs/c.1/

# Enable gadget (bind to UDC)
ls /sys/class/udc > UDC

echo "Charon HID gadget configured"
```

Make it executable:

```bash
sudo chmod +x /usr/local/bin/charon-gadget
```

### Run at boot

Create a systemd service `/etc/systemd/system/charon-gadget.service`:

```bash
sudo nano /etc/systemd/system/charon-gadget.service
```

```ini
[Unit]
Description=Charon USB HID Gadget
After=sysinit.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/charon-gadget
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
```

Enable it:

```bash
sudo systemctl daemon-reload
sudo systemctl enable charon-gadget.service
```

---

## Step 4: Set Up Input Device Permissions

Charon needs access to evdev input devices. Create a udev rule:

```bash
sudo nano /etc/udev/rules.d/99-charon-input.rules
```

```
# Allow input group to access input devices
SUBSYSTEM=="input", GROUP="input", MODE="0660"

# Allow access to HID gadget
KERNEL=="hidg*", GROUP="input", MODE="0660"
```

Add your user to the input group:

```bash
sudo usermod -a -G input $USER
```

Reload udev and log out/in for group changes to take effect:

```bash
sudo udevadm control --reload
sudo udevadm trigger
```

---

## Step 5: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

---

## Step 6: Build Charon

Clone and build:

```bash
git clone https://github.com/your-repo/charon.git
cd charon
cargo build --release
```

The binaries will be at:
- `target/release/charond` - the daemon
- `target/release/charon-tui` - the TUI client

---

## Step 7: Install Charon (User Space)

Charon runs as a user-space service, making updates and debugging easier. Create the service directory and install binaries:

```bash
mkdir -p ~/.local/charon.service
mkdir -p ~/.local/bin
mkdir -p ~/.config/charon

cp target/release/charond ~/.local/charon.service/
cp target/release/charon-tui ~/.local/charon.service/

# Also keep copies in ~/.local/bin for easy access
cp target/release/charond ~/.local/bin/
cp target/release/charon-tui ~/.local/bin/
```

Copy the helper scripts from `setup/charon/`:

```bash
cp setup/charon/run-charon-tui.sh ~/.local/charon.service/
cp setup/charon/restart-charon.sh ~/.local/charon.service/
chmod +x ~/.local/charon.service/*.sh
```

---

## Step 8: Configure Charon

Create the daemon configuration:

```bash
nano ~/.config/charon/config.toml
```

Minimal configuration:

```toml
[hid]
device = "/dev/hidg0"

[input]
# Find your keyboard with: ls -la /dev/input/by-id/
# Look for entries ending in -event-kbd
devices = [
    "/dev/input/by-id/usb-YOUR_KEYBOARD-event-kbd"
]

[keymap]
layout = "en_us"

[telemetry]
enabled = false
```

Create the TUI client configuration:

```bash
nano ~/.config/charon/tui.toml
```

```toml
# Point to the restart script for in-app upgrades
upgrade_script = "~/.local/charon.service/restart-charon.sh"
```

To find your keyboard device:

```bash
ls -la /dev/input/by-id/ | grep kbd
```

---

## Step 9: Create User Systemd Service

Copy the service file:

```bash
mkdir -p ~/.config/systemd/user
cp setup/charon/charond.service ~/.config/systemd/user/
```

Or create it manually at `~/.config/systemd/user/charond.service`:

```ini
[Unit]
Description=Charon Daemon
Wants=network-online.target
After=network-online.target

[Service]
Type=simple
ExecStartPre=/usr/bin/sleep 1
ExecStart=%h/.local/charon.service/charond
Restart=on-failure
RestartSec=3

StandardOutput=append:%h/.local/charon.service/charond-stdout.log
StandardError=append:%h/.local/charon.service/charond-error.log

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user daemon-reload
systemctl --user enable charond.service
systemctl --user start charond.service
```

Check status:

```bash
systemctl --user status charond.service
tail -f ~/.local/charon.service/charond-stdout.log
```

---

## Step 10: Auto-start TUI Client (Optional)

If you have a display attached and want the TUI to start automatically at login, copy the desktop entry:

```bash
mkdir -p ~/.config/autostart
cp setup/charon/charon-client.desktop ~/.config/autostart/
```

This uses Kitty terminal in fullscreen mode. Edit the file if you prefer a different terminal emulator.

---

## Step 11: Connect and Test

1. Connect a USB keyboard to one of the RP5's USB-A ports
2. Connect the RP5's USB-C port to your host computer
3. The host should detect a new keyboard device
4. Type on your keyboard - keystrokes should pass through to the host

---

## Troubleshooting

### Gadget not appearing on host

Check if gadget is configured:

```bash
ls /sys/kernel/config/usb_gadget/charon/
```

Check UDC binding:

```bash
cat /sys/kernel/config/usb_gadget/charon/UDC
```

Verify dwc2 is loaded:

```bash
lsmod | grep dwc2
```

### Permission denied errors

Ensure your user has correct group memberships:

```bash
groups $USER
```

You should see `input` in the list. If not, run `sudo usermod -a -G input $USER` and log out/in.

Check device permissions:

```bash
ls -la /dev/input/event*
ls -la /dev/hidg0
```

### Keystrokes not passing through

Check charond logs:

```bash
tail -f ~/.local/charon.service/charond-stdout.log
tail -f ~/.local/charon.service/charond-error.log
# or
journalctl --user -u charond.service -f
```

Test input device manually:

```bash
sudo evtest /dev/input/by-id/usb-YOUR_KEYBOARD-event-kbd
```

### High latency

Ensure you're using USB 2.0 or higher cable. The RP5 USB-C port supports USB 2.0 in gadget mode.

---

## Next Steps

- [Configure QMK integration](qmk.md) for programmable keyboards
- [Set up telemetry](../setup/README.md) for typing statistics
- Configure the TUI client for your display
