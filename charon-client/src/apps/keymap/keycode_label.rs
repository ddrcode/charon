/// Convert QMK keycode string to a display label.
/// The `max_len` parameter hints at available space for the label.
pub fn keycode_label(code: &str, max_len: usize) -> String {
    let label = match code {
        // Transparent / No key
        "KC_TRNS" | "KC_TRANSPARENT" => "▽",
        "KC_NO" => "✕",

        // Letters (KC_A -> A)
        s if s.starts_with("KC_") && s.len() == 4 => &s[3..],

        // Function keys (KC_F1 -> F1)
        s if s.starts_with("KC_F") && s.len() <= 6 => &s[3..],

        // Numbers (KC_1 -> 1)
        "KC_0" | "KC_1" | "KC_2" | "KC_3" | "KC_4" | "KC_5" | "KC_6" | "KC_7" | "KC_8" | "KC_9" => {
            &code[3..]
        }

        // Numpad
        "KC_P0" | "KC_P1" | "KC_P2" | "KC_P3" | "KC_P4" | "KC_P5" | "KC_P6" | "KC_P7" | "KC_P8"
        | "KC_P9" => &code[3..],
        "KC_PAST" => "*",
        "KC_PPLS" => "+",
        "KC_PMNS" => "-",
        "KC_PEQL" => "=",
        "KC_PERC" => "%",

        // Common keys
        "KC_SPC" | "KC_SPACE" => "SPC",
        "KC_ENT" | "KC_ENTER" => "ENT",
        "KC_ESC" | "KC_ESCAPE" => "ESC",
        "KC_BSPC" | "KC_BACKSPACE" => "BSP",
        "KC_TAB" => "TAB",
        "KC_CAPS" => "CAP",
        "KC_DEL" | "KC_DELETE" => "DEL",
        "KC_INS" | "KC_INSERT" => "INS",
        "KC_HOME" => "HOM",
        "KC_END" => "END",
        "KC_PGUP" => "PUP",
        "KC_PGDN" => "PDO",
        "KC_UP" => "↑",
        "KC_DOWN" => "↓",
        "KC_LEFT" => "←",
        "KC_RGHT" | "KC_RIGHT" => "→",
        "KC_PSCR" | "KC_PRINT_SCREEN" => "PSC",
        "KC_SCRL" | "KC_SCROLL_LOCK" => "SCR",
        "KC_PAUS" | "KC_PAUSE" => "PAU",
        "KC_NUM" | "KC_NUM_LOCK" => "NUM",
        "KC_MUTE" => "MUT",

        // Punctuation
        "KC_GRV" | "KC_GRAVE" => "`",
        "KC_MINS" | "KC_MINUS" => "-",
        "KC_EQL" | "KC_EQUAL" => "=",
        "KC_LBRC" | "KC_LEFT_BRACKET" => "[",
        "KC_RBRC" | "KC_RIGHT_BRACKET" => "]",
        "KC_BSLS" | "KC_BACKSLASH" => "\\",
        "KC_SCLN" | "KC_SEMICOLON" => ";",
        "KC_QUOT" | "KC_QUOTE" => "'",
        "KC_COMM" | "KC_COMMA" => ",",
        "KC_DOT" => ".",
        "KC_SLSH" | "KC_SLASH" => "/",

        // Modifiers
        "KC_LSFT" | "KC_LEFT_SHIFT" => "SFT",
        "KC_RSFT" | "KC_RIGHT_SHIFT" => "SFT",
        "KC_LCTL" | "KC_LEFT_CTRL" => "CTL",
        "KC_RCTL" | "KC_RIGHT_CTRL" => "CTL",
        "KC_LALT" | "KC_LEFT_ALT" => "⌥",
        "KC_RALT" | "KC_RIGHT_ALT" => "⌥",
        "KC_LGUI" | "KC_LEFT_GUI" => "⌘",
        "KC_RGUI" | "KC_RIGHT_GUI" => "⌘",

        // QMK special
        "QK_BOOT" | "RESET" => "RST",
        "RGB_MOD" => "RGB",

        // Layer functions: MO(n), DF(n), TO(n), TG(n), OSL(n)
        s if s.starts_with("MO(") => return format_layer_fn("MO", s, max_len),
        s if s.starts_with("DF(") => return format_layer_fn("DF", s, max_len),
        s if s.starts_with("TO(") => return format_layer_fn("TO", s, max_len),
        s if s.starts_with("TG(") => return format_layer_fn("TG", s, max_len),
        s if s.starts_with("OSL(") => return format_layer_fn("OSL", s, max_len),
        s if s.starts_with("LT(") => return format_layer_fn("LT", s, max_len),

        // One-shot modifiers: OSM(MOD_LSFT) -> ⇧
        s if s.starts_with("OSM(") => return format_osm(s, max_len),

        // Shifted keys: S(KC_1) -> !
        s if s.starts_with("S(") => return format_shifted(s, max_len),

        // ANY(0x...) - custom keycodes
        s if s.starts_with("ANY(") => "USR",

        // Fallback: try to make something readable
        s if s.starts_with("KC_") => &s[3..],

        // Unknown
        _ => "?",
    };

    truncate_label(label, max_len)
}

fn format_layer_fn(prefix: &str, code: &str, max_len: usize) -> String {
    // Extract number from MO(3) -> 3
    if let Some(start) = code.find('(') {
        if let Some(end) = code.find(')') {
            let num = &code[start + 1..end];
            let label = format!("{}{}", prefix, num);
            return truncate_label(&label, max_len);
        }
    }
    truncate_label(prefix, max_len)
}

fn format_osm(code: &str, max_len: usize) -> String {
    let label = if code.contains("SFT") {
        "⇧"
    } else if code.contains("CTL") {
        "^"
    } else if code.contains("ALT") {
        "⌥"
    } else if code.contains("GUI") {
        "⌘"
    } else {
        "OSM"
    };
    truncate_label(label, max_len)
}

fn format_shifted(code: &str, max_len: usize) -> String {
    // S(KC_1) -> !
    let inner = code
        .strip_prefix("S(")
        .and_then(|s| s.strip_suffix(')'))
        .unwrap_or(code);

    let label = match inner {
        "KC_1" => "!",
        "KC_2" => "@",
        "KC_3" => "#",
        "KC_4" => "$",
        "KC_5" => "%",
        "KC_6" => "^",
        "KC_7" => "&",
        "KC_8" => "*",
        "KC_9" => "(",
        "KC_0" => ")",
        "KC_MINS" => "_",
        "KC_EQL" => "+",
        "KC_LBRC" => "{",
        "KC_RBRC" => "}",
        "KC_BSLS" => "|",
        "KC_SCLN" => ":",
        "KC_QUOT" => "\"",
        "KC_GRV" => "~",
        "KC_COMM" => "<",
        "KC_DOT" => ">",
        "KC_SLSH" => "?",
        _ => "S+",
    };
    truncate_label(label, max_len)
}

fn truncate_label(label: &str, max_len: usize) -> String {
    if label.chars().count() <= max_len {
        label.to_string()
    } else {
        label.chars().take(max_len).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_keys() {
        assert_eq!(keycode_label("KC_A", 3), "A");
        assert_eq!(keycode_label("KC_1", 3), "1");
        assert_eq!(keycode_label("KC_F1", 3), "F1");
        assert_eq!(keycode_label("KC_F12", 3), "F12");
    }

    #[test]
    fn test_special_keys() {
        assert_eq!(keycode_label("KC_TRNS", 3), "▽");
        assert_eq!(keycode_label("KC_NO", 3), "✕");
        assert_eq!(keycode_label("KC_SPC", 3), "SPC");
    }

    #[test]
    fn test_layer_functions() {
        assert_eq!(keycode_label("MO(3)", 3), "MO3");
        assert_eq!(keycode_label("DF(1)", 4), "DF1");
        assert_eq!(keycode_label("TO(4)", 3), "TO4");
    }

    #[test]
    fn test_shifted() {
        assert_eq!(keycode_label("S(KC_1)", 3), "!");
        assert_eq!(keycode_label("S(KC_LBRC)", 3), "{");
    }

    #[test]
    fn test_truncation() {
        assert_eq!(keycode_label("KC_BACKSPACE", 2), "BS");
    }
}
