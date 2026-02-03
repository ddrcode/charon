#!/bin/bash

# Copy new binaries and restart the charond service
# Assumes that the charond service is already set up and running
# This file should be copied to ~/.local/charon.service/restart-charon.sh
# TUI's config should be pointing to this file, to make upgrade option to work
# (`upgrade_script` option).

systemctl --user stop charond.service

cp ~/.local/bin/charond ~/.local/charon.service/charond
cp ~/.local/bin/charon-tui ~/.local/charon.service/charon-tui

systemctl --user start charond.service

exec ~/.local/charon.service/charon-tui --config ~/.config/charon/tui.toml

