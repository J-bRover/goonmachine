use crate::{
    engine::sprite::{Tools, Utils, BUTTON_WIDTH},
    render::{
        bitmap::{BITMAP, BITMAP4X4, BITMAP4X6},
        colors::Colors,
    },
};

pub fn set_pix(pixels: &mut [Vec<Colors>], y: i32, x: i32, col: Colors) {
    //If the new pixel has 0 alpha, just keep the old guy
    //We don't wanna implement full alpha stuff cause pixel art
    //This much is fine for images with empty bgs
    if col == Colors::Blank {
        return;
    }

    if y < 0 || x < 0 || y >= pixels.len() as i32 || x >= pixels[0].len() as i32 {
        return;
    }

    pixels[y as usize][x as usize] = col;
}

//Generics so we can pass in both vecs and arrays as whats necessary
pub fn draw<R>(pixels: &mut [Vec<Colors>], x: i32, y: i32, img: &[R])
where
    R: AsRef<[Colors]>,
{
    for (j, row) in img.as_ref().iter().enumerate() {
        for (i, col) in row.as_ref().iter().enumerate() {
            set_pix(pixels, y + j as i32, x + i as i32, *col);
        }
    }
}

/* Loop over every character and use the 8x8 bitmap
 * Use bitmasking to check which pixels to be set
 */
pub fn print_scr(pixels: &mut [Vec<Colors>], x: i32, y: i32, col: Colors, msg: String) {
    for i in 0..msg.len() {
        let c = msg.as_bytes().get(i).unwrap();
        let mut idx: usize = (*c).into();
        idx -= 32;
        if idx >= BITMAP.len() {
            idx = 0;
        }

        for dx in 0..8 {
            for (dy, row) in BITMAP[idx].iter().enumerate().take(8) {
                if (row >> (7 - dx)) & 1 == 1 {
                    set_pix(pixels, y + dy as i32, x + dx + i as i32 * 8, col);
                }
            }
        }
    }
}

pub fn print_scr_mini(pixels: &mut [Vec<Colors>], x: i32, y: i32, col: Colors, msg: String) {
    for i in 0..msg.len() {
        let c = msg.as_bytes().get(i).unwrap();
        let mut idx: usize = (*c).into();
        let orig_idx: usize = (*c).into();
        idx -= 32;
        idx /= 2;
        idx *= 4;

        if idx >= BITMAP4X4.len() {
            idx = 0;
        }

        for dx in 0..4 {
            for dy in 0..4 {
                if ((BITMAP4X4[idx + dx] >> (3 - dy)) >> ((orig_idx & 1) * 4)) & 1 == 1 {
                    set_pix(pixels, (y - dy) + 3, x + dx as i32 + i as i32 * 5, col);
                }
            }
        }
    }
}

pub fn print_scr_mid(pixels: &mut [Vec<Colors>], x: i32, y: i32, col: Colors, msg: String) {
    for i in 0..msg.len() {
        let c = msg.as_bytes().get(i).unwrap();
        let mut idx: usize = (*c).into();

        idx -= 32;
        if idx >= BITMAP4X6.len() {
            idx = 0;
        }
        idx *= 6;

        for dy in 0..6 {
            let row = BITMAP4X6[idx + dy];
            for dx in 0..4 {
                if (row >> (7 - dx)) & 1 == 1 {
                    set_pix(pixels, y + dy as i32, x + dx + i as i32 * 4, col);
                }
            }
        }
    }
}

pub fn rect_fill(pixels: &mut [Vec<Colors>], x: i32, y: i32, w: i32, h: i32, col: Colors) {
    for j in x..x + w {
        for i in y..y + h {
            set_pix(pixels, i, j, col);
        }
    }
}

pub fn rect(pixels: &mut [Vec<Colors>], x: i32, y: i32, w: i32, h: i32, col: Colors) {
    for i in x..x + w {
        set_pix(pixels, y, i, col);
        set_pix(pixels, y + h, i, col);
    }

    for i in y..y + h {
        set_pix(pixels, i, x, col);
        set_pix(pixels, i, x + w, col);
    }

    set_pix(pixels, y + h, x + w, col);
}

//This kinda gives weird results we might wanna switch out the algorithm at some point
pub fn circle(pixels: &mut [Vec<Colors>], cx: i32, cy: i32, r: i32, col: Colors) {
    let r2 = r * r;
    for x in cx - r..=cx + r {
        for y in cy - r..=cy + r {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r2 {
                set_pix(pixels, y, x, col);
            }
        }
    }
}

pub fn clear(pixels: &mut [Vec<Colors>], col: Colors) {
    let height = pixels.len() as i32;
    let width = pixels[0].len() as i32;
    for y in 0..height {
        for x in 0..width {
            set_pix(pixels, y, x, col);
        }
    }
}

//I like doing constant images this way so we can modify them easily
pub fn image_from_tool(
    tool: Tools,
) -> [[Colors; BUTTON_WIDTH as usize - 2]; BUTTON_WIDTH as usize - 2] {
    let ye = Colors::Yellow;
    let br = Colors::Brown;
    let bl = Colors::Blank;
    let re = Colors::Red;
    let db = Colors::Blue;
    let si = Colors::Silver;
    let gr = Colors::Gray;
    let pi = Colors::Pink;
    match tool {
        Tools::Pencil => [
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, ye, br, br, bl],
            [bl, bl, bl, bl, bl, ye, ye, ye, br, bl],
            [bl, bl, bl, bl, ye, ye, ye, ye, ye, bl],
            [bl, bl, bl, ye, ye, ye, ye, ye, bl, bl],
            [bl, bl, ye, ye, ye, ye, ye, bl, bl, bl],
            [bl, bl, re, ye, ye, ye, bl, bl, bl, bl],
            [bl, re, re, re, ye, bl, bl, bl, bl, bl],
            [bl, re, re, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
        ],
        Tools::Fill => [
            [bl, bl, bl, bl, si, gr, gr, bl, bl, bl],
            [bl, bl, bl, si, si, gr, gr, bl, bl, bl],
            [bl, db, db, si, gr, gr, gr, si, bl, bl],
            [db, db, si, si, gr, gr, si, si, si, bl],
            [db, si, si, si, gr, gr, si, si, si, si],
            [db, si, si, gr, gr, gr, si, si, si, si],
            [db, si, si, si, si, si, si, si, si, si],
            [db, bl, si, si, si, si, si, si, si, si],
            [db, bl, bl, si, si, si, si, si, si, si],
            [bl, bl, bl, bl, bl, si, si, si, si, bl],
        ],
        Tools::Eraser => [
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, re, re, bl, bl, bl],
            [bl, bl, bl, bl, re, pi, re, re, bl, bl],
            [bl, bl, bl, re, re, re, pi, re, re, bl],
            [bl, bl, re, re, re, re, re, pi, re, bl],
            [bl, re, pi, re, re, re, re, re, bl, bl],
            [bl, re, re, pi, re, re, re, bl, bl, bl],
            [bl, bl, re, re, pi, re, bl, bl, bl, bl],
            [bl, bl, bl, re, re, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
        ],
        Tools::Select => [
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, si, si, bl, si, si, bl, si, si, bl],
            [bl, si, bl, bl, bl, bl, bl, bl, si, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, si, bl, bl, bl, bl, bl, bl, si, bl],
            [bl, si, bl, bl, bl, bl, bl, bl, si, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, si, bl, bl, bl, bl, bl, bl, si, bl],
            [bl, si, si, bl, si, si, bl, si, si, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
        ],
    }
}

pub fn image_from_util(
    util: Utils,
) -> [[Colors; BUTTON_WIDTH as usize - 2]; BUTTON_WIDTH as usize - 2] {
    let bl = Colors::Blank;
    let re = Colors::Red;
    let gr = Colors::Gray;
    let ge = Colors::Green;
    match util {
        Utils::FlipHor => [
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, gr, bl, bl, gr, bl, bl, bl],
            [bl, bl, gr, gr, bl, bl, gr, gr, bl, bl],
            [bl, gr, gr, gr, gr, gr, gr, gr, gr, bl],
            [bl, gr, gr, gr, gr, gr, gr, gr, gr, bl],
            [bl, bl, gr, gr, bl, bl, gr, gr, bl, bl],
            [bl, bl, bl, gr, bl, bl, gr, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
        ],
        Utils::FlipVert => [
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, gr, gr, bl, bl, bl, bl],
            [bl, bl, bl, gr, gr, gr, gr, bl, bl, bl],
            [bl, bl, gr, gr, gr, gr, gr, gr, bl, bl],
            [bl, bl, bl, bl, gr, gr, bl, bl, bl, bl],
            [bl, bl, bl, bl, gr, gr, bl, bl, bl, bl],
            [bl, bl, gr, gr, gr, gr, gr, gr, bl, bl],
            [bl, bl, bl, gr, gr, gr, gr, bl, bl, bl],
            [bl, bl, bl, bl, gr, gr, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
        ],
        Utils::Clear => [
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, re, re, bl, bl, bl, bl, re, re, bl],
            [bl, re, re, re, bl, bl, re, re, re, bl],
            [bl, bl, re, re, re, re, re, re, bl, bl],
            [bl, bl, bl, re, re, re, re, bl, bl, bl],
            [bl, bl, bl, re, re, re, re, bl, bl, bl],
            [bl, bl, re, re, re, re, re, re, bl, bl],
            [bl, re, re, re, bl, bl, re, re, re, bl],
            [bl, re, re, bl, bl, bl, bl, re, re, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
        ],
        Utils::Save => [
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, ge, bl],
            [bl, bl, bl, bl, bl, bl, bl, ge, ge, bl],
            [bl, bl, bl, bl, bl, bl, ge, ge, bl, bl],
            [bl, bl, bl, bl, bl, ge, ge, bl, bl, bl],
            [bl, bl, ge, bl, ge, ge, bl, bl, bl, bl],
            [bl, ge, ge, ge, ge, bl, bl, bl, bl, bl],
            [bl, bl, ge, ge, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
            [bl, bl, bl, bl, bl, bl, bl, bl, bl, bl],
        ],
    }
}
