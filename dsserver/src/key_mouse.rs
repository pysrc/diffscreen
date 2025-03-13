pub fn mouse_to_engin(key: u8) -> Option<enigo::Button> {
    match key {
        233 => Some(enigo::Button::Left),
        235 => Some(enigo::Button::Right),
        _ => None,
    }
}

pub fn key_to_enigo(key: u8) -> Option<enigo::Key> {
    match key {
        27 => Some(enigo::Key::Escape),
        190 => Some(enigo::Key::F1),
        191 => Some(enigo::Key::F2),
        192 => Some(enigo::Key::F3),
        193 => Some(enigo::Key::F4),
        194 => Some(enigo::Key::F5),
        195 => Some(enigo::Key::F6),
        196 => Some(enigo::Key::F7),
        197 => Some(enigo::Key::F8),
        198 => Some(enigo::Key::F9),
        199 => Some(enigo::Key::F10),
        200 => Some(enigo::Key::F11),
        201 => Some(enigo::Key::F12),
        // 19 => Some(enigo::Key::Pause), // Pause
        // 97 => Some(enigo::Key::Print), // Print
        255 => Some(enigo::Key::Delete),
        87 => Some(enigo::Key::End),
        96 => Some(enigo::Key::Unicode('`')),
        48 => Some(enigo::Key::Unicode('0')),
        49 => Some(enigo::Key::Unicode('1')),
        50 => Some(enigo::Key::Unicode('2')),
        51 => Some(enigo::Key::Unicode('3')),
        52 => Some(enigo::Key::Unicode('4')),
        53 => Some(enigo::Key::Unicode('5')),
        54 => Some(enigo::Key::Unicode('6')),
        55 => Some(enigo::Key::Unicode('7')),
        56 => Some(enigo::Key::Unicode('8')),
        57 => Some(enigo::Key::Unicode('9')),
        45 => Some(enigo::Key::Unicode('-')),
        61 => Some(enigo::Key::Unicode('=')),
        8 => Some(enigo::Key::Backspace),
        9 => Some(enigo::Key::Tab),
        91 => Some(enigo::Key::Unicode('[')),
        93 => Some(enigo::Key::Unicode(']')),
        92 => Some(enigo::Key::Unicode('\\')),
        229 => Some(enigo::Key::CapsLock),
        59 => Some(enigo::Key::Unicode(';')),
        39 => Some(enigo::Key::Unicode('\'')),
        13 => Some(enigo::Key::Return),
        225 => Some(enigo::Key::Shift), // ShiftL
        44 => Some(enigo::Key::Unicode(',')),
        46 => Some(enigo::Key::Unicode('.')),
        47 => Some(enigo::Key::Unicode('/')),
        226 => Some(enigo::Key::Shift), // ShiftR
        82 => Some(enigo::Key::UpArrow),
        227 => Some(enigo::Key::Control), // ControlL
        233 => Some(enigo::Key::Alt),     // AltL
        32 => Some(enigo::Key::Space),
        234 => Some(enigo::Key::Alt), // AltR
        // 103 => Some(enigo::Key::Menu),
        228 => Some(enigo::Key::Control), // ControlR
        81 => Some(enigo::Key::LeftArrow),
        84 => Some(enigo::Key::DownArrow),
        83 => Some(enigo::Key::RightArrow),
        // 99 => Some(enigo::Key::Raw(45)), // Insert
        86 => Some(enigo::Key::PageDown),
        80 => Some(enigo::Key::Home),
        85 => Some(enigo::Key::PageUp),
        a if a >= 97 && a <= 122 => Some(enigo::Key::Unicode((a - 97 + ('a' as u8)) as char)),
        _ => None,
    }
}