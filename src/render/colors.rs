use bincode::{Decode, Encode};

use crate::engine::rico::PixelsType;
use std::fmt;
use std::str::FromStr;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum Colors {
    Blank = 0,
    Black,
    White,
    Gray,
    Silver,
    Red,
    Maroon,
    Orange,
    Yellow,
    Gold,
    Green,
    Olive,
    Brown,
    Blue,
    Teal,
    Purple,
    Pink,
}

//Theres so much duplicated implementation in this file but idk a better way to do this
impl Colors {
    pub fn pixels(width: usize, height: usize) -> PixelsType {
        vec![vec![Colors::Black; width]; height]
    }

    pub fn rgba(self) -> [u8; 4] {
        match self {
            Colors::Blank => [0, 0, 0, 0],
            Colors::Black => [0, 0, 0, 255],
            Colors::White => [255, 255, 255, 255],
            Colors::Gray => [128, 128, 128, 255],
            Colors::Silver => [192, 192, 192, 255],
            Colors::Red => [200, 40, 40, 255],
            Colors::Maroon => [128, 0, 0, 255],
            Colors::Orange => [255, 140, 0, 255],
            Colors::Yellow => [240, 230, 80, 255],
            Colors::Gold => [255, 215, 0, 255],
            Colors::Green => [0, 180, 0, 255],
            Colors::Olive => [128, 128, 0, 255],
            Colors::Brown => [139, 69, 19, 255],
            Colors::Blue => [65, 105, 225, 255],
            Colors::Teal => [0, 128, 128, 255],
            Colors::Purple => [138, 43, 226, 255],
            Colors::Pink => [255, 105, 180, 255],
        }
    }
}

//So that we can parse to strings for get_pix primarily
impl fmt::Display for Colors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Colors::Black => "BLACK",
            Colors::White => "WHITE",
            Colors::Gray => "GRAY",
            Colors::Silver => "SILVER",
            Colors::Red => "RED",
            Colors::Maroon => "MAROON",
            Colors::Orange => "ORANGE",
            Colors::Yellow => "YELLOW",
            Colors::Gold => "GOLD",
            Colors::Green => "GREEN",
            Colors::Olive => "OLIVE",
            Colors::Brown => "BROWN",
            Colors::Blue => "BLUE",
            Colors::Teal => "TEAL",
            Colors::Purple => "PURPLE",
            Colors::Pink => "PINK",
            Colors::Blank => "BLANK",
        };
        write!(f, "{s}")
    }
}

//So we can parse from strings for all lua calls
impl FromStr for Colors {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BLACK" => Ok(Colors::Black),
            "WHITE" => Ok(Colors::White),
            "GRAY" => Ok(Colors::Gray),
            "SILVER" => Ok(Colors::Silver),
            "RED" => Ok(Colors::Red),
            "MAROON" => Ok(Colors::Maroon),
            "ORANGE" => Ok(Colors::Orange),
            "YELLOW" => Ok(Colors::Yellow),
            "GOLD" => Ok(Colors::Gold),
            "GREEN" => Ok(Colors::Green),
            "OLIVE" => Ok(Colors::Olive),
            "BROWN" => Ok(Colors::Brown),
            "BLUE" => Ok(Colors::Blue),
            "TEAL" => Ok(Colors::Teal),
            "PURPLE" => Ok(Colors::Purple),
            "PINK" => Ok(Colors::Pink),
            "BLANK" => Ok(Colors::Blank),
            _ => Err(()),
        }
    }
}

pub const ALL_COLORS: [Colors; 17] = [
    Colors::Blank,
    Colors::Black,
    Colors::White,
    Colors::Gray,
    Colors::Silver,
    Colors::Red,
    Colors::Maroon,
    Colors::Orange,
    Colors::Yellow,
    Colors::Gold,
    Colors::Green,
    Colors::Olive,
    Colors::Brown,
    Colors::Blue,
    Colors::Teal,
    Colors::Purple,
    Colors::Pink,
];
