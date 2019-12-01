use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.split('\n').collect();

    let module_masses: Vec<u32> = parse_lines(&lines)?;

    let total_fuel: u32 = module_masses.iter().map(|mass| fuel_by_mass(*mass)).sum();

    println!("Total fuel required: {}", total_fuel);

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;
    Ok(result)
}

fn parse_lines(lines: &[&str]) -> Result<Vec<u32>, String> {
    lines
        .iter()
        .filter(|line| !line.is_empty())
        .map(|line| line.parse::<u32>().map_err(|e| e.to_string()))
        .collect()
}

fn fuel_by_mass(mass: u32) -> u32 {
    mass / 3 - 2
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fuel_by_mass() {
        assert_eq!(fuel_by_mass(12), 2);
        assert_eq!(fuel_by_mass(14), 2);
        assert_eq!(fuel_by_mass(1969), 654);
        assert_eq!(fuel_by_mass(100756), 33583);
    }
}
