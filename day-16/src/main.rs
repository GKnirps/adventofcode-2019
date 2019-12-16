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

    let after_100_phases = fft(input.clone(), 100);
    print!("First eight digits after 100 phases: ");
    for d in after_100_phases.iter().take(8) {
        print!("{}", d);
    }
    println!();

    let large_result = fft_large_input(&input, 10000)
        .ok_or_else(|| "Sorry, but this would take years".to_owned())?;
    for d in large_result {
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
                .skip(output_i)
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

fn fft_phase_upper_half(upper: &[i32]) -> Vec<i32> {
    // idea: the result of the upper half only depends on the upper half.
    // the first output of the upper half is a sum of the input at that point plus all inputs
    // after that
    // the second output of the upper half is the sum of the input at that point plus all
    // inputs after that and so on => we can just use a cumulative sum.
    let mut result: Vec<i32> = Vec::with_capacity(upper.len());
    let mut prev = 0;
    for v in upper.iter().rev() {
        prev = (prev + v) % 10;
        result.push(prev);
    }
    result.reverse();
    result
}

fn get_output_offset(input: &[i32]) -> usize {
    input.iter().take(7).fold(0, |n, d| n * 10 + *d as usize)
}

// im not calling this ffft
fn fft_large_input(input: &[i32], n_repeat: usize) -> Option<Vec<i32>> {
    // It is not feasible to calculate the fft for large inputs.
    // However, we may have some advantages:
    // - the 2nd half is cheap to calculate
    // - the 2nd half is independent of the first half
    // => idea: if the result we are looking for lies in the second half, we win!
    let offset = (input.len() * n_repeat) / 2;
    let output_offset = get_output_offset(input);

    if output_offset < offset {
        // I know of no efficient way to do this.
        return None;
    }

    let mut current_output: Vec<i32> = input
        .iter()
        .cycle()
        .skip(offset)
        .take(input.len() * n_repeat - offset)
        .cloned()
        .collect();

    for _ in 0..100 {
        current_output = fft_phase_upper_half(&current_output);
    }
    current_output
        .get((output_offset - offset)..(output_offset - offset + 8))
        .map(|o| o.to_vec())
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

    #[test]
    fn test_puzzle2_example() {
        assert_eq!(
            fft_large_input(&parse_input("03036732577212944063491565474664"), 10000),
            Some(vec![8, 4, 4, 6, 2, 0, 2, 6])
        );
    }
}
