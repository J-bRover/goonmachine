use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{Read, Write},
    path::Path,
};

use base64::{engine::general_purpose, Engine as _};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

use crate::{
    engine::{rico::PixelsType, sprite::SPRITE_SIZE},
    render::colors::Colors,
};
use bincode::{config::standard, Decode, Encode};
use walkdir::WalkDir;

#[derive(Encode, Decode, Debug, Clone)]
pub struct Cartridge {
    pub sprite_sheet: Vec<PixelsType>,
    pub scripts: HashMap<String, String>,
}

pub const PATH: &str = "r32/";

const HELLO_WORLD: &str = 
"function start()
    rico:log(\"Welcome to RICO-32!\")
    rico:set_frame_rate(60)
end

function update(dt)
    rico:clear(\"BLACK\")
    rico:print_scr(10, 10, \"WHITE\", \"Hello, World!\")
    
    local mouse = rico:mouse()
    if mouse.pressed then
        rico:circle(mouse.x, mouse.y, 5, \"RED\")
    end
end";

fn encode(input: &Vec<u8>) -> String {
    general_purpose::STANDARD.encode(input)
}

fn decode(input: &Vec<u8>) -> Vec<u8> {
    general_purpose::STANDARD.decode(input).expect("Could not decode")
}

impl Default for Cartridge {
    fn default() -> Self {
        let mut scripts = HashMap::new();
        scripts.insert("main.lua".to_string(), HELLO_WORLD.to_string());
        Cartridge {
            sprite_sheet: vec![vec![vec![Colors::Blank; SPRITE_SIZE]; SPRITE_SIZE]; 60],
            scripts,
        }
    }
}

fn write_cart(bin_path: &str, cart: &Cartridge) -> Result<(), Box<dyn Error>> {
    let encoded = bincode::encode_to_vec(cart, bincode::config::standard())?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&encoded)?;
    let compressed_bytes = encoder.finish()?;
    if bin_path.ends_with(".r32.txt") {
        let base_64 = encode(&compressed_bytes);
        fs::write(bin_path, base_64)?;
    } else {
        fs::write(bin_path, compressed_bytes)?;
    }
    Ok(())
}

fn load_file(bin_path: &str) -> Result<Cartridge, Box<dyn Error>> {
    let mut bytes = fs::read(&bin_path)?;
    if bin_path.ends_with(".r32.txt") { bytes = decode(&bytes) };
    let mut decoder = GzDecoder::new(&bytes[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    let (cart, _) = bincode::decode_from_slice(&decompressed, standard())?;

    Ok(cart)
}

pub fn get_cart(bin_path: &str) -> Result<Cartridge, Box<dyn Error>> {
    match load_file(bin_path) {
        Ok(data) => Ok(data),
        Err(_) => {
            let cart = Cartridge::default();
            write_cart(bin_path, &cart)?;
            Ok(cart)
        }
    }
}

pub fn load_cartridge(bin_path: &str) -> Result<Cartridge, Box<dyn Error>> {
    let cart = get_cart(bin_path)?;

    if Path::new(PATH).exists() {
        fs::remove_dir_all(PATH)?;
    }
    for (file, content) in &cart.scripts {
        let f_path = PATH.to_owned() + file;
        if let Some(parent) = Path::new(&f_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(f_path, content)?;
    }

    Ok(cart)
}

pub fn update_sprites(bin_path: &str, sprite_sheet: &[PixelsType]) -> Result<(), Box<dyn Error>> {
    let mut cart = get_cart(bin_path)?;
    cart.sprite_sheet = sprite_sheet.to_vec();
    write_cart(bin_path, &cart)?;
    Ok(())
}

pub fn update_scripts(bin_path: &str) -> Result<(), Box<dyn Error>> {
    let mut cart = get_cart(bin_path)?;
    cart.scripts.clear();

    for entry in WalkDir::new(PATH)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && e.file_name().to_str().unwrap().ends_with(".lua"))
    {
        let path = entry.path();
        //That replace took 20 minutes to debug btw
        let rel = path.strip_prefix(PATH).unwrap().to_string_lossy().to_string().replace("\\", "/");
        let contents = fs::read_to_string(path)?;
        cart.scripts.insert(rel, contents);
    }

    write_cart(bin_path, &cart)?;

    Ok(())
}
