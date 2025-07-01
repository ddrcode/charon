#!/bin/bash

set -e

echo "--------------------------------------"
echo "        Welcome to KeyboardOS         "
echo "--------------------------------------"
echo "The ghost between your keyboard and your machine."

# 1) Check for sudo
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (sudo $0)"
  exit 1
fi

# 2) Confirm user
echo "This script will install KeyboardOS pass-through service."
echo "Your physical keyboard will be connected via /dev/input/eventX"
echo "Your gadget endpoint will be /dev/hidg0"
read -p "Proceed? (y/N) " confirm
[ "$confirm" == "y" ] || exit 0

# 3) Ensure required packages
echo "Installing dependencies..."
apt update
apt install -y python3 python3-evdev neovim git

# 4) Add udev rule for /dev/hidg0
echo "Setting up udev rule for gadget permissions..."
cat <<EOF > /etc/udev/rules.d/99-hidg.rules
KERNEL=="hidg[0-9]*", MODE="0660", GROUP="plugdev"
EOF
udevadm control --reload-rules
udevadm trigger

# 5) Add user to input and plugdev groups
echo "Adding $SUDO_USER to input and plugdev groups..."
usermod -aG input $SUDO_USER
usermod -aG plugdev $SUDO_USER

# 6) Install systemd service
echo "Installing systemd user service..."
mkdir -p /home/$SUDO_USER/.config/systemd/user
cat <<EOF > /home/$SUDO_USER/.config/systemd/user/keyboardos.service
[Unit]
Description=KeyboardOS Pass-through Daemon
After=network.target

[Service]
ExecStart=/home/$SUDO_USER/keyboardos/pass_through.py
Restart=always

[Install]
WantedBy=default.target
EOF

chown $SUDO_USER:$SUDO_USER /home/$SUDO_USER/.config/systemd/user/keyboardos.service

# 7) Clone your actual code (in real life!)
echo "Cloning KeyboardOS repo..."
mkdir -p /home/$SUDO_USER/keyboardos
chown $SUDO_USER:$SUDO_USER /home/$SUDO_USER/keyboardos
echo "(Pretend we git clone here... your Rust or Python code will live there)"

# 8) Enable & start
echo "Enabling and starting KeyboardOS service..."
runuser -l $SUDO_USER -c "systemctl --user daemon-reload"
runuser -l $SUDO_USER -c "systemctl --user enable keyboardos.service"
runuser -l $SUDO_USER -c "systemctl --user start keyboardos.service"

echo "--------------------------------------"
echo " All done! Reboot your Pi & enjoy...  "
echo "   KeyboardOS is now your ghost 👻    "
echo "--------------------------------------"

