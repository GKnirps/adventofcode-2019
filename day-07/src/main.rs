use intcode::{parse, run_program, State};
use std::cmp;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::mem;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;
    let program = parse(&content)?;

    let max_signal = find_max_signal(program)?;
    println!("Maximal thruster signal is {}", max_signal);

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn run_chained(prog: Vec<isize>, phase_settings: &[isize]) -> Result<isize, String> {
    let mut output: Vec<isize> = vec![0];
    let mut input: Vec<isize> = Vec::with_capacity(10);
    for phase in phase_settings {
        mem::swap(&mut input, &mut output);
        input.insert(0, *phase);
        output.clear();

        output = run_program(State::new(prog.clone()), &input)?.2;
    }
    output
        .get(0)
        .cloned()
        .ok_or_else(|| "No output found".to_owned())
}

fn generate_possible_settings() -> Vec<[isize; 5]> {
    let mut base = [0, 1, 2, 3, 4];
    all_permutations(base.len(), &mut base)
}

fn all_permutations(k: usize, array: &mut [isize; 5]) -> Vec<[isize; 5]> {
    if k == 1 {
        vec![*array]
    } else {
        let mut results = all_permutations(k - 1, array);

        for i in 0..(k - 1) {
            if k % 2 == 0 {
                array.swap(i, k - 1);
            } else {
                array.swap(0, k - 1);
            }
            results.extend(all_permutations(k - 1, array));
        }
        results
    }
}

fn find_max_signal(prog: Vec<isize>) -> Result<isize, String> {
    let mut max_signal = isize::min_value();
    let all_settings = generate_possible_settings();

    for settings in all_settings {
        let signal = run_chained(prog.clone(), &settings)?;
        max_signal = cmp::max(max_signal, signal);
    }
    Ok(max_signal)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_max_signal_example_1() {
        // given
        let program = vec![
            3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
        ];

        // when
        let signal = find_max_signal(program).expect("Expected valid program execution");

        // then
        assert_eq!(signal, 43210);
    }

    #[test]
    fn test_max_signal_example_2() {
        // given
        let program = vec![
            3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23,
            99, 0, 0,
        ];

        // when
        let signal = find_max_signal(program).expect("Expected valid program execution");

        // then
        assert_eq!(signal, 54321);
    }

    #[test]
    fn test_max_signal_example_3() {
        // given
        let program = vec![
            3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1,
            33, 31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
        ];

        // when
        let signal = find_max_signal(program).expect("Expected valid program execution");

        // then
        assert_eq!(signal, 65210);
    }
}
