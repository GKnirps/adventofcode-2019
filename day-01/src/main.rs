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

    let module_masses: Vec<i32> = parse_lines(&lines)?;

    let payload_fuel: i32 = module_masses.iter().map(|mass| fuel_by_mass(*mass)).sum();
    println!("Basic payload fuel: {}", payload_fuel);

    let fuel: i32 = module_masses
        .iter()
        .map(|mass| total_fuel(fuel_by_mass(*mass)))
        .sum();
    println!("Total fuel: {}", fuel);

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;
    Ok(result)
}

fn parse_lines(lines: &[&str]) -> Result<Vec<i32>, String> {
    lines
        .iter()
        .filter(|line| !line.is_empty())
        .map(|line| line.parse::<i32>().map_err(|e| e.to_string()))
        .collect()
}

fn fuel_by_mass(mass: i32) -> i32 {
    mass / 3 - 2
}

fn total_fuel(payload_fuel: i32) -> i32 {
    let mut total: i32 = 0;
    let mut current_fuel: i32 = payload_fuel;
    while current_fuel > 0 {
        total += current_fuel;
        current_fuel = fuel_by_mass(current_fuel);
    }
    total
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

    #[test]
    fn test_total_fuel() {
        assert_eq!(total_fuel(654), 966);
        assert_eq!(total_fuel(33583), 50346);
    }
}
