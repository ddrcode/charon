# Dependencies

## Clipboard cache

Install `wl-clipboard`:

```bash
sudo apt install wl-clipboard
```

And configure it as a local service

```
[Unit]
Description=Charon clipboard cache (via wl-paste)
After=graphical-session.target

[Service]
ExecStart=/bin/sh -c 'mkdir -p %t/charon && /usr/bin/wl-paste --no-newline --watch tee %t/charon/clipboard-cache'
Restart=on-failure

[Install]
WantedBy=default.target
```

and run

```bash
systemctl --user daemon-reexec
systemctl --user enable --now charon-clipboard-cache.service
```

## Raw HID Device

1. Find your device by inspecting `/dev/hidraw*/` files, i.e.:
   ```
   udevadm info -a -n /dev/hidraw3
   ```
   and note `idVendor` and `idProduct`

1. Edit `/etc/udev/rules.d/99-qmk.rules`:

    ```
    SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3434", ATTRS{idProduct}=="01a1", MODE="0660", GROUP="charon"
    ```

2. Reload udev rules:
   ```
   sudo udevadm control --reload
   sudo udevadm trigger
   ```
