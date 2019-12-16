use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let input = parse_input(&content);

    let after_100_phases = fft(input, 100);
    print!("First eight digits after 100 phases: ");
    for d in after_100_phases.iter().take(8) {
        print!("{}", d);
    }
    println!();

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn parse_input(input: &str) -> Vec<i32> {
    input
        .chars()
        .filter_map(|c| c.to_digit(10))
        .map(|d| d as i32)
        .collect()
}

// I could probably do this with fewer memory allocations, but that sounds like
// premature optimization, so why bother?
fn fft(input: Vec<i32>, n_phases: usize) -> Vec<i32> {
    let mut current_output = input;
    for _ in 0..n_phases {
        current_output = fft_phase(&current_output);
    }
    current_output
}

fn fft_phase(input: &[i32]) -> Vec<i32> {
    (0..input.len())
        .map(|output_i| {
            input
                .iter()
                .enumerate()
                .map(|(input_i, input_v)| input_v * pattern_for(output_i, input_i))
                .sum::<i32>()
                .abs()
                % 10
        })
        .collect()
}

// This would be nice as iterator, but I don't really need it as iterator.
static BASE_PATTERN: [i32; 4] = [0, 1, 0, -1];
fn pattern_for(output_i: usize, input_i: usize) -> i32 {
    BASE_PATTERN[((input_i + 1) / (output_i + 1)) % BASE_PATTERN.len()]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pattern_for() {
        assert_eq!(pattern_for(0, 0), 1);
        assert_eq!(pattern_for(0, 1), 0);
        assert_eq!(pattern_for(0, 2), -1);
        assert_eq!(pattern_for(0, 3), 0);
        assert_eq!(pattern_for(0, 4), 1);

        assert_eq!(pattern_for(1, 0), 0);
        assert_eq!(pattern_for(1, 1), 1);
        assert_eq!(pattern_for(1, 2), 1);
        assert_eq!(pattern_for(1, 3), 0);
        assert_eq!(pattern_for(1, 4), 0);
        assert_eq!(pattern_for(1, 5), -1);
        assert_eq!(pattern_for(1, 6), -1);
        assert_eq!(pattern_for(1, 7), 0);
        assert_eq!(pattern_for(1, 8), 0);
        assert_eq!(pattern_for(1, 9), 1);
    }

    #[test]
    fn test_fft_phase() {
        // given
        let input = &[1, 2, 3, 4, 5, 6, 7, 8];

        // when
        let result = fft_phase(input);

        // then
        assert_eq!(result, vec!(4, 8, 2, 2, 6, 1, 5, 8));
    }

    #[test]
    fn test_puzzle1_examples() {
        assert_eq!(
            fft(parse_input("80871224585914546619083218645595"), 100)[0..8],
            [2, 4, 1, 7, 6, 1, 7, 6]
        );
        assert_eq!(
            fft(parse_input("19617804207202209144916044189917"), 100)[0..8],
            [7, 3, 7, 4, 5, 4, 1, 8]
        );
        assert_eq!(
            fft(parse_input("69317163492948606335995924319873"), 100)[0..8],
            [5, 2, 4, 3, 2, 1, 3, 3]
        );
    }
}
