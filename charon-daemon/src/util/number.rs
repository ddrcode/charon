// SPDX-License-Identifier: GPL-3.0-or-later
pub fn integer_digit_count(n: f64) -> u32 {
    if n == 0.0 {
        1
    } else {
        n.abs().floor().log10().floor() as u32 + 1
    }
}
