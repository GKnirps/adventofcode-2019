use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

static WIDTH: usize = 25;
static HEIGHT: usize = 6;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let pixels = parse_pixels(&content);

    match corruption_test(&pixels) {
        Some(checksum) => println!("Corruption checksum is {}", checksum),
        None => println!("Bad image, no corruption checksum"),
    }

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn parse_pixels(img: &str) -> Vec<u32> {
    img.trim().chars().filter_map(|c| c.to_digit(10)).collect()
}

fn corruption_test(pixels: &[u32]) -> Option<usize> {
    let min_0_layer = pixels
        .chunks_exact(WIDTH * HEIGHT)
        .min_by_key(|layer| layer.iter().filter(|p| **p == 0).count())?;

    Some(
        min_0_layer.iter().filter(|p| **p == 1).count()
            * min_0_layer.iter().filter(|p| **p == 2).count(),
    )
}
