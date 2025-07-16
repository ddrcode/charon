#!/bin/sh

echo 1 | sudo tee /sys/class/backlight/11-0045/bl_power
sudo cpufreq-set -g powersave

