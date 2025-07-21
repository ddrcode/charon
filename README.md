# keyboard-os
The ghost between your keyboard and your machine.

## Is it stable?

Yes. So far it hasn’t crashed — despite the prototype being held together with `.unwrap()` and good intentions. That’s how solid the architecture is (and how solid Rust is). If the ghost drops your keystrokes into the Styx, we’ll fish them out and patch it.


## Planned features

- Keyboard-specific settings and statistics
- Multi-keyboard: use one keyboard for typing and the other one for macros
- Multiple layouts
- Sending Unicode characters (OS-specific)

## Screenshots

<img width="666" height="366" alt="splash screen" src="https://github.com/user-attachments/assets/67eacb3b-e3ac-41f4-a0fd-943c3f5d5318" />
<img width="665" height="365" alt="idle screen" src="https://github.com/user-attachments/assets/ca557aac-345c-43bc-b5db-2adb28691038" />
<img width="666" height="365" alt="typing fast screen" src="https://github.com/user-attachments/assets/0eb8a35e-34a2-425b-843e-4e85b91e7c0e" />
<img width="657" height="370" alt="Menu" src="https://github.com/user-attachments/assets/1dd3e38a-d8f0-4983-9e64-0a07a95b48c1" />



## Testing your devices

Use `evtest <input_file>`

## Credits

[rikka-chunibyo/HIDPi](https://github.com/rikka-chunibyo/HIDPi)
Without this project I would have never properly configured RP5 as HID gadget. Thank you!

