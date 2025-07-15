# keyboard-os
The ghost between your keyboard and your machine.

## Is it stable?

Yes. So far it hasn’t crashed — despite the prototype being held together with `.unwrap()` and good intentions. That’s how solid the architecture is (and how solid Rust is). If the ghost drops your keystrokes into the Styx, we’ll fish them out and patch it.


## Planned features

- Keyboard-specific settings and statistics
- Multi-keyboard: use one keyboard for typing and the other one for macros
- Multiple layouts
- Sending Unicode characters (OS-specific)

## Testing your devices

Use `evtest <input_file>`

## Credits

[rikka-chunibyo/HIDPi](https://github.com/rikka-chunibyo/HIDPi)
Without this project I would have never properly configured RP5 as HID gadget. Thank you!

