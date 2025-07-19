pub fn unify_line_length(txt: &str) -> String {
    let maxlen = txt.lines().map(|line| line.len()).max().unwrap_or(0);
    txt.lines()
        .map(|line| format!("{:<width$}", line, width = maxlen))
        .collect::<Vec<_>>()
        .join("\n")
}
