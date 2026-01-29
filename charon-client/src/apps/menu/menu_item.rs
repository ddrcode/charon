pub struct MenuItem {
    pub name: String,
    pub icon: char,
    pub shortcut: String,
}

impl From<&(&'static str, char, &'static str)> for MenuItem {
    fn from(value: &(&'static str, char, &'static str)) -> Self {
        Self {
            name: value.0.into(),
            icon: value.1,
            shortcut: value.2.into(),
        }
    }
}
