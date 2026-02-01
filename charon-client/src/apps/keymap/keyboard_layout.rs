// SPDX-License-Identifier: GPL-3.0-or-later
use std::{fmt, mem};

const ARROWS: &'static str = "↑←↓→";

#[derive(Default)]
pub(crate) struct KeyboardLayout {
    keys: Vec<Key>,
    parts: Vec<LayoutPart>,
}

#[derive(Debug, Clone)]
pub(crate) struct Key {
    pub row: usize,
    pub col: usize,
    pub len: usize,
    pub lab: String,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:^width$}", self.lab, width = self.len)
    }
}

#[derive(Debug)]
enum LayoutPart {
    Key(usize),
    Decoration(String),
}

impl KeyboardLayout {
    pub fn from_str(layout: &str) -> Self {
        let layout = String::from(layout.trim());
        let mut prev_ch = '└';
        let mut len = 0;
        let mut col = 0;
        let mut row = 0;
        let mut keys_in_row = 0;
        let mut body = String::new();
        let mut decor = String::new();
        let mut keys: Vec<Key> = Vec::new();
        let mut parts: Vec<LayoutPart> = Vec::new();

        let is_key = |c: char| c.is_ascii() || ARROWS.contains(c);

        for ch in layout.chars() {
            if ch == '\n' {
                if keys_in_row > 0 {
                    row += 1;
                    col = 0;
                }
                keys_in_row = 0;
                prev_ch = ch;
                decor.push(ch);
                continue;
            }
            if is_key(ch) {
                if !is_key(prev_ch) {
                    parts.push(LayoutPart::Decoration(mem::take(&mut decor)));
                }
                len += 1;
                body.push(ch);
            } else if is_key(prev_ch) {
                let trimmed = body.trim();
                if trimmed.len() > 0 {
                    keys.push(Key {
                        row,
                        col,
                        len,
                        lab: trimmed.to_string(),
                    });
                    parts.push(LayoutPart::Key(keys.len() - 1));
                    col += 1;
                    keys_in_row += 1;
                    body.clear();
                } else {
                    parts.push(LayoutPart::Decoration(mem::take(&mut body)));
                }
                len = 0;
                decor.push(ch);
            } else {
                decor.push(ch);
            }
            prev_ch = ch;
        }
        if !decor.is_empty() {
            parts.push(LayoutPart::Decoration(decor));
        }
        Self { keys, parts }
    }

    pub fn key(&self, id: usize) -> &Key {
        &self.keys[id]
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn clear(&mut self) {
        self.keys.iter_mut().for_each(|key| {
            key.lab.clear();
        });
    }

    pub fn parts(&self) -> impl Iterator<Item = (String, bool)> {
        self.parts.iter().map(|part| match part {
            LayoutPart::Decoration(s) => (s.clone(), false),
            LayoutPart::Key(n) => (self.keys[*n].to_string(), true),
        })
    }

    pub fn set_label(&mut self, key_id: usize, label: &str) {
        self.keys
            .get_mut(key_id)
            .map(|key| key.lab = String::from(label));
    }
}

impl fmt::Display for KeyboardLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.parts().fold(String::new(), |a, b| a + &b.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_single_row() {
        let row = "│ESC│F1 │F2 │F3 │F4 │F5 │F6 │F7 │F8 │F9 │F10│F11│F12│";
        let data = KeyboardLayout::from_str(row);
        assert_eq!(data.len(), 13);
        assert_eq!(data.key(3).len, 3);
        assert_eq!(data.key(3).col, 3);
    }

    #[test]
    fn test_split_row() {
        let row = "│ESC│F1 │        │2│    F11   │F12│";
        let data = KeyboardLayout::from_str(row);
        assert_eq!(data.len(), 5);
        assert_eq!(data.key(2).len, 1);
        assert_eq!(data.key(3).len, 10);
    }

    #[test]
    fn test_multi_row() {
        let txt = r"
┌──┐┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐  ┌───┐ ┌───┐
│KN││ESC│F1 │F2 │F3 │F4 │F5 │F6 │F7 │F8 │F9 │F10│F11│F12│  │INS│ │PUP│
├──┤├───┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬─┴─┬─┘ ├───┤
│M1││ ~  │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │ 0 │ - │ = │BSP│   │DEL│
├──┤├────┴┬──┴┬──┴┬─ ┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬───┐  ├───┤
        ";
        let data = KeyboardLayout::from_str(txt.trim());
        assert_eq!(data.len(), 32);
        assert_eq!(data.key(16).col, 0);
        assert_eq!(data.key(16).row, 1);
        assert_eq!(data.key(16).len, 2);
    }

    #[test]
    fn test_to_string_simple() {
        let row = "│ESC│F1 │F2 │F3 │F4 │F5 │F6 │F7 │F8 │F9 │F10│F11│F12│";
        let data = KeyboardLayout::from_str(row);
        assert_eq!(data.to_string(), row);
    }

    #[test]
    fn test_to_string_full() {
        let txt = r"
┌──┐┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐  ┌───┐ ┌───┐
│KN││ESC│F1 │F2 │F3 │F4 │F5 │F6 │F7 │F8 │F9 │F10│F11│F12│  │INS│ │PUP│
├──┤├───┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬─┴─┬─┘ ├───┤
│M1││ ~  │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │ 0 │ - │ = │BSP│   │DEL│
├──┤├────┴┬──┴┬──┴┬─ ┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬──┴┬───┐  ├───┤
        "
        .trim();
        let data = KeyboardLayout::from_str(txt);
        assert_eq!(data.to_string(), txt);
    }
}
