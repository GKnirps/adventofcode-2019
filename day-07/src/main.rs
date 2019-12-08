use intcode::{parse, run_program, ReturnStatus, State};
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

    let max_signal = find_max_signal(program.clone())?;
    println!("Maximal thruster signal is {}", max_signal);

    let max_feedback_signal = find_max_signal_loop(program)?;
    println!(
        "Maximal thruster signal with feedback loop is {}",
        max_feedback_signal
    );

    Ok(())
}

fn read_file(path: &Path) -> std::io::Result<String> {
    let ifile = File::open(path)?;
    let mut bufr = BufReader::new(ifile);
    let mut result = String::with_capacity(2048);
    bufr.read_to_string(&mut result)?;

    Ok(result)
}

fn run_with_feedback_loop(prog: Vec<isize>, phase_settings: &[isize]) -> Result<isize, String> {
    let mut processes: Vec<State> = Vec::with_capacity(phase_settings.len());

    let mut output: Vec<isize> = vec![0];
    let mut input: Vec<isize> = Vec::with_capacity(10);
    let mut last_status = ReturnStatus::Wait;

    // start processes with phase settings
    for phase in phase_settings {
        mem::swap(&mut input, &mut output);
        input.insert(0, *phase);
        output.clear();

        let (state, status, o) = run_program(State::new(prog.clone()), &input)?;
        processes.push(state);
        last_status = status;
        output = o;
    }

    while last_status != ReturnStatus::Halt {
        for process in &mut processes {
            mem::swap(&mut input, &mut output);
            output.clear();

            // There is probably a better way to do this so  processes[i].clone() is not
            // necessary, but it is 1:15 am and I am tired.
            let (state, status, o) = run_program(process.clone(), &input)?;
            *process = state;
            if last_status == ReturnStatus::Halt && status != ReturnStatus::Halt {
                return Err(
                    "Process is waiting, but input process has halted -> deadlock".to_owned(),
                );
            }
            last_status = status;
            output = o;
        }
    }

    output
        .get(0)
        .cloned()
        .ok_or_else(|| "No output found".to_owned())
}

fn generate_possible_settings_chain() -> Vec<[isize; 5]> {
    let mut base = [0, 1, 2, 3, 4];
    all_permutations(base.len(), &mut base)
}

fn generate_possible_settings_loop() -> Vec<[isize; 5]> {
    let mut base = [5, 6, 7, 8, 9];
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
    let all_settings = generate_possible_settings_chain();

    for settings in all_settings {
        let signal = run_with_feedback_loop(prog.clone(), &settings)?;
        max_signal = cmp::max(max_signal, signal);
    }
    Ok(max_signal)
}

fn find_max_signal_loop(prog: Vec<isize>) -> Result<isize, String> {
    let mut max_signal = isize::min_value();
    let all_settings = generate_possible_settings_loop();

    for settings in all_settings {
        let signal = run_with_feedback_loop(prog.clone(), &settings)?;
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

    #[test]
    fn test_feedback_loop_example_1() {
        // given
        let program = vec![
            3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1,
            28, 1005, 28, 6, 99, 0, 0, 5,
        ];

        // when
        let signal = find_max_signal_loop(program).expect("Expected valid program execution");

        // then
        assert_eq!(signal, 139629729);
    }

    #[test]
    fn test_feedback_loop_example_2() {
        // given
        let program = vec![
            3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54,
            -5, 54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4,
            53, 1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
        ];

        // when
        let signal = find_max_signal_loop(program).expect("Expected valid program execution");

        // then
        assert_eq!(signal, 18216);
    }
}
