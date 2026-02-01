# Charon
**The ghost between your keyboard and your machine.**

Or - if you don’t like poetry:
> A USB keyboard pass-through device built on Raspberry Pi, capable of intercepting input to run
> local apps (like a password manager or editor) and injecting the results as keystrokes.

Or - if you wear a tie and a blue shirt:
> A stealth personal assistant disguised as a keyboard.

Or - if this parapgraph is already too long for you:

<img width="578" alt="Charon" src="https://github.com/user-attachments/assets/63099e00-4ce6-4a08-a5d6-9f160f672aa9" />




## Current Features

- **Pass-through Mode** - Forwards keystrokes directly to the host with negligible latency.
- **In-App Mode** - Acts as an interceptor: brings up a menu on screen for user interaction.
- **Vim-everywhere** - Launch a local editor, type your text, and inject it as keystrokes to the host.
- **Charonsay** - Enjoy cryptic wisdom from the other side of the Styx as you type.
- **Telemetry** - Captures rich per-keystroke stats, because *every ESC press matters*.
- **Stats Screen / Charts** - Visualize metrics like average/max WPM over the past year.
- **Power Management** - Automatically dims the screen and lowers CPU usage when idle.
- **Password Manager** - Securely pick and type out passwords—no copy-paste involved.
- **Keymaps** - keystrokes writer supports multiple layouts/keymaps



##  "*But I already have a programmable keyboard (QMK, ZMK, etc)...*"

Perfect. Charon is *not* a replacement for QMK - it's an extension.

Think of it as giving your QMK keyboard:
- A brain,
- A screen, and
- A full Linux stack.

With Charon you can:
- Run any Linux app, interact with it, and send results to the host as keystrokes.
- Plug in multiple keyboards: type on one, trigger macros on the other.
- Get real telemetry (e.g., *how often did you press ESC last Tuesday?*).
- Do anything that needs more power, storage, display, or OS features than microcontroller-based solutions allow.



## "*But I don’t have a programmable keyboard...*"

Perfect - because you don’t need one.

Charon isn't about just remapping keys or adding layers. It's about bringing *apps* and a *screen* to *any* keyboard, programmable or not.



## "*Is It Stable?*"

Yes.
Despite the prototype being held together with `.unwrap()` and goodwill, it hasn’t crashed once.

> “If the ghost drops your keystrokes into the Styx, we’ll fish them out and patch it.”

More pragmatically:
- I now use Charon 100% of the time.
- The daemon reliably forwards keystrokes with **sub-millisecond latency**.
- The client is still limited to a few basic apps, but it’s growing fast.

It’s not "production-grade" or certified for aerospace use (yet), but it’s definitely *towel-grade*:
It doesn’t panic.



## Planned Features

- Per-keyboard settings and stats
- Multi-keyboard support (e.g. one for typing, another for macros)
- Unicode character injection (platform-specific)
- Mouse pass-through
- QMK Raw HID support
- Users / profiles



##  Known Limitations

- No wake-from-sleep: It can't currently wake the host from a deep sleep via keystrokes.
  **Workaround**: It *can* send a Wake-on-LAN magic packet (just press F8).



## Screenshots

<div>
Charon's welcome screen:<br/>
<img width="666" alt="splash screen" src="https://github.com/user-attachments/assets/67eacb3b-e3ac-41f4-a0fd-943c3f5d5318" />
</div>

<div>
When typing happens somewhere else... (the idle state)<br/>
<img width="666" alt="idle screen" src="https://github.com/user-attachments/assets/ca557aac-345c-43bc-b5db-2adb28691038" />
</div>

<div>
When you're typing faster than yourself, Cerberus comes:<br/>
<img width="666" alt="typing fast screen" src="https://github.com/user-attachments/assets/0eb8a35e-34a2-425b-843e-4e85b91e7c0e" />
</div>

<div>
Charts in the terminal? Why not...<br/>
<img width="666" alt="wpm stats" src="https://github.com/user-attachments/assets/e892e629-6749-40e9-91cc-38a809d136ac" />
</div>

<div>
Heatmap is fine too...<br/>
<img width="666" alt="key frequencies" src="https://github.com/user-attachments/assets/f48e23b5-3921-47ae-a568-040393adbb3d" />
</div>

<div>
Charon's menu - it pops up automatically on <i>magic key</i> (F7 by default):<br/>
<img width="666" alt="menu" src="https://github.com/user-attachments/assets/d9bea87b-6ed7-4a6d-bc6c-a8f14bf83a4b" />
</div>

## Want to Contribute?

Charon is still in its early stages, but it’s built to grow. If you:

- love tinkering with Linux input devices,
- enjoy Rust’s type system more than some people enjoy vacation,
- or have ideas about how to make keyboards even cooler...

You’re very welcome to get involved.

Open issues, suggest features, or just come say hi in Discussions.
No pressure, no CLA, just curiosity and a bit of ghostly magic.

## Testing your devices

Use `evtest <input_file>`

## Credits

[rikka-chunibyo/HIDPi](https://github.com/rikka-chunibyo/HIDPi)
Without this project I would have never properly configured RP5 as HID gadget. Thank you!

[passepartui](https://github.com/kardwen/passepartui)
The default password manager used by Charon (although it can be integrated with any other one tool).

[ASCII.co.uk](https://ascii.co.uk/)
The ASCII-art comes from the amazing repository on that website. A great ASCII-gem.


## License

Charon is licensed under the [GNU General Public License v3.0](LICENSE).


## Bonus: Pancakes from the Underworld

Charon doesn’t just ferry your keystrokes. Here’s what fuels him:

**Ghostly Pancakes**

Ingredients:
- 1 cup all-purpose flour
- 1 tbsp sugar
- 1 tsp baking powder
- 1/2 tsp baking soda
- 1 pinch of salt
- 1 cup buttermilk
- 1 egg
- 2 tbsp melted butter

Instructions:
1.	In a bowl, whisk together dry ingredients.
2.	In another bowl, beat the egg, then add buttermilk and melted butter.
3.	Combine wet and dry. Do not overmix — lumps are from the Styx.
4.	Cook on a hot greased griddle until bubbles form; flip and cook the other side.
5.	Serve with maple syrup. Or with Charon’s tears of joy.


