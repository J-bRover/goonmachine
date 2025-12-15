use std::collections::HashSet;

use winit::event::VirtualKeyCode;

#[derive(Default)]
pub struct Keyboard {
    pub keys_pressed: HashSet<VirtualKeyCode>,
    pub keys_just_pressed: HashSet<VirtualKeyCode>,
}

pub fn key_from_str(str: &str) -> Option<VirtualKeyCode> {
    //Only keys that we wanna support within the engine
    //Dont add others just to make sure lua doesn't have full access
    match str {
        "1" => Some(VirtualKeyCode::Key1),
        "2" => Some(VirtualKeyCode::Key2),
        "3" => Some(VirtualKeyCode::Key3),
        "4" => Some(VirtualKeyCode::Key4),
        "5" => Some(VirtualKeyCode::Key5),
        "6" => Some(VirtualKeyCode::Key6),
        "7" => Some(VirtualKeyCode::Key7),
        "8" => Some(VirtualKeyCode::Key8),
        "9" => Some(VirtualKeyCode::Key9),
        "0" => Some(VirtualKeyCode::Key0),
        "A" => Some(VirtualKeyCode::A),
        "B" => Some(VirtualKeyCode::B),
        "C" => Some(VirtualKeyCode::C),
        "D" => Some(VirtualKeyCode::D),
        "E" => Some(VirtualKeyCode::E),
        "F" => Some(VirtualKeyCode::F),
        "G" => Some(VirtualKeyCode::G),
        "H" => Some(VirtualKeyCode::H),
        "I" => Some(VirtualKeyCode::I),
        "J" => Some(VirtualKeyCode::J),
        "K" => Some(VirtualKeyCode::K),
        "L" => Some(VirtualKeyCode::L),
        "M" => Some(VirtualKeyCode::M),
        "N" => Some(VirtualKeyCode::N),
        "O" => Some(VirtualKeyCode::O),
        "P" => Some(VirtualKeyCode::P),
        "Q" => Some(VirtualKeyCode::Q),
        "R" => Some(VirtualKeyCode::R),
        "S" => Some(VirtualKeyCode::S),
        "T" => Some(VirtualKeyCode::T),
        "U" => Some(VirtualKeyCode::U),
        "V" => Some(VirtualKeyCode::V),
        "W" => Some(VirtualKeyCode::W),
        "X" => Some(VirtualKeyCode::X),
        "Y" => Some(VirtualKeyCode::Y),
        "Z" => Some(VirtualKeyCode::Z),
        "Left" => Some(VirtualKeyCode::Left),
        "Up" => Some(VirtualKeyCode::Up),
        "Right" => Some(VirtualKeyCode::Right),
        "Down" => Some(VirtualKeyCode::Down),
        "Back" => Some(VirtualKeyCode::Back),
        "Enter" => Some(VirtualKeyCode::Return),
        "Space" => Some(VirtualKeyCode::Space),
        "Period" => Some(VirtualKeyCode::Period),
        "LShift" => Some(VirtualKeyCode::LShift),
        "RShift" => Some(VirtualKeyCode::RShift),
        "Slash" => Some(VirtualKeyCode::Slash),
        "Backslash" => Some(VirtualKeyCode::Backslash),
        "Comma" => Some(VirtualKeyCode::Comma),
        "Colon" => Some(VirtualKeyCode::Colon),
        "Quote" => Some(VirtualKeyCode::Apostrophe),
        "LBracket" => Some(VirtualKeyCode::LBracket),
        "RBracket" => Some(VirtualKeyCode::RBracket),
        "Equals" => Some(VirtualKeyCode::Equals),
        "Minus" => Some(VirtualKeyCode::Minus),
        _ => None,
    }
}
