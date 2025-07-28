# Charon
The ghost between your keyboard and your machine.

## Current features
- Pass-through mode - sends your key events directly to the host
- In-App mode – brings menu to the screen and allows interaction
- Vim-everywhere - type in the editor of your choice and send text as keystrokes to the host
- Charonsay – see wisdoms from the Styx-side while typing
- Telemetry – "proper" telemetry (Prometheus-based) holding stats about every keystroke
- Stats screen / charts – see the stats, i.e. avg/max WPM in the last year
- Power management - disables the screen and reduces CPU when inactive

## Limitations
- It can't wake up the host from sleep by sending key presses. Current workaround - it can send WoL magic packet.

## Is it stable?

Yes. So far it hasn’t crashed — despite the prototype being held together with `.unwrap()` and good intentions. That’s how solid the architecture is (and how solid Rust is). If the ghost drops your keystrokes into the Styx, we’ll fish them out and patch it.


## Planned features

- Keyboard-specific settings and statistics
- Multi-keyboard: i.e. use one keyboard for typing and the other one for macros
- Multiple layouts
- Sending Unicode characters (OS-specific)
- Mouse Pass-through

## Screenshots

<img width="666" alt="splash screen" src="https://github.com/user-attachments/assets/67eacb3b-e3ac-41f4-a0fd-943c3f5d5318" />
<img width="666" alt="idle screen" src="https://github.com/user-attachments/assets/ca557aac-345c-43bc-b5db-2adb28691038" />
<img width="666" alt="typing fast screen" src="https://github.com/user-attachments/assets/0eb8a35e-34a2-425b-843e-4e85b91e7c0e" />
<img width="666" alt="wpm stats" src="https://github.com/user-attachments/assets/e892e629-6749-40e9-91cc-38a809d136ac" />
<img width="666" alt="key frequencies" src="https://github.com/user-attachments/assets/f48e23b5-3921-47ae-a568-040393adbb3d" />
<img width="666" alt="menu" src="https://github.com/user-attachments/assets/d9bea87b-6ed7-4a6d-bc6c-a8f14bf83a4b" />



## Testing your devices

Use `evtest <input_file>`

## Credits

[rikka-chunibyo/HIDPi](https://github.com/rikka-chunibyo/HIDPi)
Without this project I would have never properly configured RP5 as HID gadget. Thank you!

