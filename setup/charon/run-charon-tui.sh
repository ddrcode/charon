#!/bin/bash

# This script runs the charon TUI with the specified config file.
# It should be copied to ~/.local/charon.service/run-charon-tui.sh
# The file used by the auto-start desktop entry.

exec ~/.local/charon.service/charon-tui --config ~/.config/charon/tui.toml
