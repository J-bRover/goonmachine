use base64::{engine::general_purpose, Engine as _};
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: {} <encode|decode> <input>", args[0]);
        std::process::exit(1);
    }

    let input = &args[2];

    match args[1].as_str() {
        "encode" => encode(input),
        "decode" => decode(input),
        _ => std::process::exit(1),
    };
}

fn encode(input: &str) {
    let bytes = fs::read(input).unwrap_or_else(|_| panic!("Could not find {}", input));
    let encoded = general_purpose::STANDARD.encode(&bytes);
    print!("{}", encoded);
}

fn decode(input: &str) {
    let decoded = general_purpose::STANDARD.decode(input).expect("Could not decode");
    fs::write("main.r32", decoded).expect("Could not write to main.r32");
}
