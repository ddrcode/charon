# Charon setup files

This folder contains files that configure or create external services used by Charon.
They are all optional, although some of them, like `telemetry`, are related to important
part of Charon's logic.

## Folders

- `telemetry`: configuration of [Prometheus](https://prometheus.io/) and
  [Pushgateway](https://github.com/prometheus/pushgateway).
  This is crucial for collecting any statistics while typing - like WPM, key-presses
  count etc. The entire telemetry functionality can be disabled in the config file
  (`enbale_telemetry` option), but when enabled, Prometheus must be configured

- `qmk`: important only when Charon works with a QMK-powered programmable keyboard
  and only when [Raw HID](https://docs.qmk.fm/features/rawhid) is enabled on the keyboard
  side. When enabled, Charon can react to events like layer change and collect more
  telemetry, directly from the keyboard (bypassing OS layer).

