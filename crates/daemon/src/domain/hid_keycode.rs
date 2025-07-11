use evdev::KeyCode;

use crate::error::KOSError;

/// Represents a USB HID Usage ID for a key.
/// See: https://www.usb.org/sites/default/files/hut1_3_0.pdf for details
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum HidKeyCode {
    KEY_A = 0x04,
    KEY_B = 0x05,
    KEY_C = 0x06,
    KEY_D = 0x07,
    KEY_E = 0x08,
    KEY_F = 0x09,
    KEY_G = 0x0A,
    KEY_H = 0x0B,
    KEY_I = 0x0C,
    KEY_J = 0x0D,
    KEY_K = 0x0E,
    KEY_L = 0x0F,
    KEY_M = 0x10,
    KEY_N = 0x11,
    KEY_O = 0x12,
    KEY_P = 0x13,
    KEY_Q = 0x14,
    KEY_R = 0x15,
    KEY_S = 0x16,
    KEY_T = 0x17,
    KEY_U = 0x18,
    KEY_V = 0x19,
    KEY_W = 0x1A,
    KEY_X = 0x1B,
    KEY_Y = 0x1C,
    KEY_Z = 0x1D,

    KEY_1 = 0x1E,
    KEY_2 = 0x1F,
    KEY_3 = 0x20,
    KEY_4 = 0x21,
    KEY_5 = 0x22,
    KEY_6 = 0x23,
    KEY_7 = 0x24,
    KEY_8 = 0x25,
    KEY_9 = 0x26,
    KEY_0 = 0x27,

    KEY_ENTER = 0x28,
    KEY_ESC = 0x29,
    KEY_BACKSPACE = 0x2A,
    KEY_TAB = 0x2B,
    KEY_SPACE = 0x2C,

    KEY_MINUS = 0x2D,
    KEY_EQUAL = 0x2E,
    KEY_LEFTBRACE = 0x2F,
    KEY_RIGHTBRACE = 0x30,
    KEY_BACKSLASH = 0x31,
    KEY_SEMICOLON = 0x33,
    KEY_APOSTROPHE = 0x34,
    KEY_GRAVE = 0x35,
    KEY_COMMA = 0x36,
    KEY_DOT = 0x37,
    KEY_SLASH = 0x38,

    KEY_CAPSLOCK = 0x39,

    KEY_F1 = 0x3A,
    KEY_F2 = 0x3B,
    KEY_F3 = 0x3C,
    KEY_F4 = 0x3D,
    KEY_F5 = 0x3E,
    KEY_F6 = 0x3F,
    KEY_F7 = 0x40,
    KEY_F8 = 0x41,
    KEY_F9 = 0x42,
    KEY_F10 = 0x43,
    KEY_F11 = 0x44,
    KEY_F12 = 0x45,

    KEY_INSERT = 0x49,
    KEY_DELETE = 0x4C,
    KEY_HOME = 0x4A,
    KEY_END = 0x4D,
    KEY_PAGEUP = 0x4B,
    KEY_PAGEDOWN = 0x4E,
    KEY_UP = 0x52,
    KEY_DOWN = 0x51,
    KEY_LEFT = 0x50,
    KEY_RIGHT = 0x4F,

    KEY_NUMLOCK = 0x53,
    KEY_SCROLLLOCK = 0x47,

    KEY_KP0 = 0x62,
    KEY_KP1 = 0x59,
    KEY_KP2 = 0x5A,
    KEY_KP3 = 0x5B,
    KEY_KP4 = 0x5C,
    KEY_KP5 = 0x5D,
    KEY_KP6 = 0x5E,
    KEY_KP7 = 0x5F,
    KEY_KP8 = 0x60,
    KEY_KP9 = 0x61,
    KEY_KPDOT = 0x63,
    KEY_KPENTER = 0x58,
    KEY_KPSLASH = 0x54,
    KEY_KPASTERISK = 0x55,
    KEY_KPMINUS = 0x56,
    KEY_KPPLUS = 0x57,

    KEY_MUTE = 0x7F,
    KEY_VOLUMEUP = 0x80,
    KEY_VOLUMEDOWN = 0x81,

    KEY_LEFTCTRL = 0xE0,
    KEY_LEFTSHIFT = 0xE1,
    KEY_LEFTALT = 0xE2,
    KEY_LEFTMETA = 0xE3,
    KEY_RIGHTCTRL = 0xE4,
    KEY_RIGHTSHIFT = 0xE5,
    KEY_RIGHTALT = 0xE6,
    KEY_RIGHTMETA = 0xE7,
}

impl HidKeyCode {
    pub fn code(&self) -> u8 {
        *self as u8
    }

    pub fn is_modifier(&self) -> bool {
        (0xE0..=0xE7).contains(&self.code())
    }

    pub fn modifier_mask(&self) -> u8 {
        if self.is_modifier() {
            return 1 << (self.code() - 0xE0);
        }
        0
    }

    pub fn seq_from_char(c: char) -> Result<Vec<Self>, KOSError> {
        use HidKeyCode::*;
        let mut seq = Vec::with_capacity(4);
        let mut c = c;
        let mut needs_shift = c.is_ascii_uppercase();

        if needs_shift {
            c.make_ascii_lowercase();
        } else {
            let new_c = match c {
                '!' => '1',
                '@' => '2',
                '#' => '3',
                '$' => '4',
                '%' => '5',
                '^' => '6',
                '&' => '7',
                '*' => '8',
                '(' => '9',
                ')' => '0',
                '_' => '-',
                '+' => '=',
                '{' => '[',
                '}' => ']',
                '|' => '\\',
                ':' => ';',
                '"' => '\'',
                '<' => ',',
                '>' => '.',
                '?' => '/',
                '~' => '`',
                _ => c,
            };
            needs_shift = new_c != c;
            c = new_c;
        }

        if needs_shift {
            seq.push(KEY_LEFTSHIFT);
        }

        let key_code = match c {
            'a'..='z' => HidKeyCode::try_from((c as u8) - b'a' + KEY_A.code())?,
            '1'..='9' => HidKeyCode::try_from((c as u8) - b'1' + KEY_1.code())?,
            '0' => KEY_0,
            ' ' => KEY_SPACE,
            '\n' => KEY_ENTER,
            '\t' => KEY_TAB,
            '`' => KEY_GRAVE,
            '-' => KEY_MINUS,
            '=' => KEY_EQUAL,
            '[' => KEY_LEFTBRACE,
            ']' => KEY_RIGHTBRACE,
            '\\' => KEY_BACKSLASH,
            ';' => KEY_SEMICOLON,
            '\'' => KEY_APOSTROPHE,
            ',' => KEY_COMMA,
            '.' => KEY_DOT,
            '/' => KEY_SLASH,
            _ => return Err(KOSError::UnsupportedCharacter(c)),
        };

        seq.push(key_code);
        Ok(seq)
    }
}

impl From<HidKeyCode> for u8 {
    fn from(key: HidKeyCode) -> Self {
        key.code()
    }
}

impl TryFrom<u8> for HidKeyCode {
    type Error = KOSError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use HidKeyCode::*;
        let val = match value {
            0x04 => KEY_A,
            0x05 => KEY_B,
            0x06 => KEY_C,
            0x07 => KEY_D,
            0x08 => KEY_E,
            0x09 => KEY_F,
            0x0A => KEY_G,
            0x0B => KEY_H,
            0x0C => KEY_I,
            0x0D => KEY_J,
            0x0E => KEY_K,
            0x0F => KEY_L,
            0x10 => KEY_M,
            0x11 => KEY_N,
            0x12 => KEY_O,
            0x13 => KEY_P,
            0x14 => KEY_Q,
            0x15 => KEY_R,
            0x16 => KEY_S,
            0x17 => KEY_T,
            0x18 => KEY_U,
            0x19 => KEY_V,
            0x1A => KEY_W,
            0x1B => KEY_X,
            0x1C => KEY_Y,
            0x1D => KEY_Z,

            0x1E => KEY_1,
            0x1F => KEY_2,
            0x20 => KEY_3,
            0x21 => KEY_4,
            0x22 => KEY_5,
            0x23 => KEY_6,
            0x24 => KEY_7,
            0x25 => KEY_8,
            0x26 => KEY_9,
            0x27 => KEY_0,

            0x28 => KEY_ENTER,
            0x29 => KEY_ESC,
            0x2A => KEY_BACKSPACE,
            0x2B => KEY_TAB,
            0x2C => KEY_SPACE,

            0x2D => KEY_MINUS,
            0x2E => KEY_EQUAL,
            0x2F => KEY_LEFTBRACE,
            0x30 => KEY_RIGHTBRACE,
            0x31 => KEY_BACKSLASH,
            0x33 => KEY_SEMICOLON,
            0x34 => KEY_APOSTROPHE,
            0x35 => KEY_GRAVE,
            0x36 => KEY_COMMA,
            0x37 => KEY_DOT,
            0x38 => KEY_SLASH,

            0x39 => KEY_CAPSLOCK,

            0x3A => KEY_F1,
            0x3B => KEY_F2,
            0x3C => KEY_F3,
            0x3D => KEY_F4,
            0x3E => KEY_F5,
            0x3F => KEY_F6,
            0x40 => KEY_F7,
            0x41 => KEY_F8,
            0x42 => KEY_F9,
            0x43 => KEY_F10,
            0x44 => KEY_F11,
            0x45 => KEY_F12,

            0x49 => KEY_INSERT,
            0x4C => KEY_DELETE,
            0x4A => KEY_HOME,
            0x4D => KEY_END,
            0x4B => KEY_PAGEUP,
            0x4E => KEY_PAGEDOWN,
            0x52 => KEY_UP,
            0x51 => KEY_DOWN,
            0x50 => KEY_LEFT,
            0x4F => KEY_RIGHT,

            0x53 => KEY_NUMLOCK,
            0x47 => KEY_SCROLLLOCK,

            0x62 => KEY_KP0,
            0x59 => KEY_KP1,
            0x5A => KEY_KP2,
            0x5B => KEY_KP3,
            0x5C => KEY_KP4,
            0x5D => KEY_KP5,
            0x5E => KEY_KP6,
            0x5F => KEY_KP7,
            0x60 => KEY_KP8,
            0x61 => KEY_KP9,
            0x63 => KEY_KPDOT,
            0x58 => KEY_KPENTER,
            0x54 => KEY_KPSLASH,
            0x55 => KEY_KPASTERISK,
            0x56 => KEY_KPMINUS,
            0x57 => KEY_KPPLUS,

            0x7F => KEY_MUTE,
            0x80 => KEY_VOLUMEUP,
            0x81 => KEY_VOLUMEDOWN,

            0xE0 => KEY_LEFTCTRL,
            0xE1 => KEY_LEFTSHIFT,
            0xE2 => KEY_LEFTALT,
            0xE3 => KEY_LEFTMETA,
            0xE4 => KEY_RIGHTCTRL,
            0xE5 => KEY_RIGHTSHIFT,
            0xE6 => KEY_RIGHTALT,
            0xE7 => KEY_RIGHTMETA,

            _ => return Err(KOSError::UnsupportedCharacter(value.into())),
        };
        Ok(val)
    }
}

macro_rules! match_key {
    ($kc:expr, $( $name:ident ),*) => {
        match $kc {
            $(
                KeyCode::$name => HidKeyCode::$name,
            )*
            other => return Err(KOSError::UnsupportedKeyCode(other)),
        }
    };
}

impl TryFrom<&KeyCode> for HidKeyCode {
    type Error = KOSError;

    fn try_from(kc: &KeyCode) -> Result<Self, Self::Error> {
        Ok(match_key!(
            *kc,
            KEY_A,
            KEY_B,
            KEY_C,
            KEY_D,
            KEY_E,
            KEY_F,
            KEY_G,
            KEY_H,
            KEY_I,
            KEY_J,
            KEY_K,
            KEY_L,
            KEY_M,
            KEY_N,
            KEY_O,
            KEY_P,
            KEY_Q,
            KEY_R,
            KEY_S,
            KEY_T,
            KEY_U,
            KEY_V,
            KEY_W,
            KEY_X,
            KEY_Y,
            KEY_Z,
            KEY_1,
            KEY_2,
            KEY_3,
            KEY_4,
            KEY_5,
            KEY_6,
            KEY_7,
            KEY_8,
            KEY_9,
            KEY_0,
            KEY_ENTER,
            KEY_ESC,
            KEY_BACKSPACE,
            KEY_TAB,
            KEY_SPACE,
            KEY_MINUS,
            KEY_EQUAL,
            KEY_LEFTBRACE,
            KEY_RIGHTBRACE,
            KEY_BACKSLASH,
            KEY_SEMICOLON,
            KEY_APOSTROPHE,
            KEY_GRAVE,
            KEY_COMMA,
            KEY_DOT,
            KEY_SLASH,
            KEY_CAPSLOCK,
            KEY_F1,
            KEY_F2,
            KEY_F3,
            KEY_F4,
            KEY_F5,
            KEY_F6,
            KEY_F7,
            KEY_F8,
            KEY_F9,
            KEY_F10,
            KEY_F11,
            KEY_F12,
            KEY_LEFTCTRL,
            KEY_LEFTSHIFT,
            KEY_LEFTALT,
            KEY_LEFTMETA,
            KEY_RIGHTCTRL,
            KEY_RIGHTSHIFT,
            KEY_RIGHTALT,
            KEY_RIGHTMETA,
            KEY_INSERT,
            KEY_DELETE,
            KEY_HOME,
            KEY_END,
            KEY_PAGEUP,
            KEY_PAGEDOWN,
            KEY_UP,
            KEY_DOWN,
            KEY_LEFT,
            KEY_RIGHT,
            KEY_NUMLOCK,
            KEY_SCROLLLOCK,
            KEY_KP0,
            KEY_KP1,
            KEY_KP2,
            KEY_KP3,
            KEY_KP4,
            KEY_KP5,
            KEY_KP6,
            KEY_KP7,
            KEY_KP8,
            KEY_KP9,
            KEY_KPDOT,
            KEY_KPENTER,
            KEY_KPSLASH,
            KEY_KPASTERISK,
            KEY_KPMINUS,
            KEY_KPPLUS,
            KEY_MUTE,
            KEY_VOLUMEUP,
            KEY_VOLUMEDOWN
        ))
    }
}
