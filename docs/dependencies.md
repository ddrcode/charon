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
