pub struct KeyboardLayout {
    keys: Vec<Key>,
    layout: String,
}

#[derive(Clone, Copy)]
pub struct Key {
    pub pos: usize,
    pub row: usize,
    pub col: usize,
    pub len: usize,
}

impl KeyboardLayout {
    pub fn from_str(layout: &str) -> Self {
        let layout = String::from(layout.trim());
        let mut prev_ch = '└';
        let mut len = 0;
        let mut col = 0;
        let mut row = 0;
        let mut pos = 0;
        let mut keys_in_row = 0;
        let mut body = String::new();
        let mut keys: Vec<Key> = Vec::new();

        let is_key = |c: char| c.is_ascii();

        for (i, ch) in layout.chars().enumerate() {
            if ch == '\n' {
                if keys_in_row > 0 {
                    row += 1;
                    col = 0;
                }
                keys_in_row = 0;
                prev_ch = ch;
                continue;
            }
            if is_key(ch) {
                if is_key(prev_ch) {
                } else {
                    pos = i;
                }
                len += 1;
                body.push(ch);
            } else if is_key(prev_ch) {
                if body.trim().len() > 0 {
                    keys.push(Key { pos, row, col, len });
                    col += 1;
                    keys_in_row += 1;
                }
                len = 0;
                body.clear();
            }
            prev_ch = ch;
        }

        Self { layout, keys }
    }

    pub fn key(&self, id: usize) -> &Key {
        &self.keys[id]
    }

    pub fn len(&self) -> usize {
        self.keys.len()
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
        assert_eq!(data.key(3).pos, 13);
    }

    #[test]
    fn test_split_row() {
        let row = "│ESC│F1 │        │2│    F11   │F12│";
        let data = KeyboardLayout::from_str(row);
        assert_eq!(data.len(), 5);
        assert_eq!(data.key(2).len, 1);
        assert_eq!(data.key(3).len, 10);
        assert_eq!(data.key(3).pos, 20);
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
        assert_eq!(data.key(0).pos, 72);
    }
}
