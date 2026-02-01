// SPDX-License-Identifier: GPL-3.0-or-later
use wake_on_lan::MagicPacket;

pub fn wake_host_on_lan(mac: &[u8; 6]) -> std::io::Result<()> {
    let packet = MagicPacket::new(mac);
    packet.send()
}
