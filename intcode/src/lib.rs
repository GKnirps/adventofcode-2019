pub fn parse(input: &str) -> Result<Vec<isize>, String> {
    input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<isize>().map_err(|e| e.to_string()))
        .collect()
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct State {
    pub mem: Vec<isize>,
    pub ip: usize,
}

impl State {
    pub fn new(mem: Vec<isize>) -> State {
        State { mem, ip: 0 }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum ReturnStatus {
    Halt,
    Wait,
}

// Return values: current state, return status and output
pub fn run_program(
    state: State,
    input: &[isize],
) -> Result<(State, ReturnStatus, Vec<isize>), String> {
    let State { mut mem, mut ip } = state;
    let mut output: Vec<isize> = Vec::new();
    let mut in_iter = input.iter();
    if mem.len() <= ip {
        return Err("Invalid instruction pointer".to_owned());
    }
    while mem[ip] != 99 {
        match mem[ip] % 100 {
            // addition
            1 => {
                let (v1, v2, dest) = get_binary_op_operands(ip, &mem)?;
                mem[dest] = v1 + v2;
                ip += 4;
            }
            // multiplication
            2 => {
                let (v1, v2, dest) = get_binary_op_operands(ip, &mem)?;
                mem[dest] = v1 * v2;
                ip += 4;
            }
            // read
            3 => {
                let dest = get_input_dest(ip, &mem)?;
                if let Some(value) = in_iter.next() {
                    mem[dest] = *value;
                    ip += 2;
                } else {
                    // No input to read. Return the control flow to the caller
                    return Ok((State { mem, ip }, ReturnStatus::Wait, output));
                }
            }
            // write
            4 => {
                output.push(get_output_operand(ip, &mem)?);
                ip += 2;
            }
            // jump not zero
            5 => {
                let (condition, dest) = get_two_operands(ip, &mem)?;
                ip = if condition != 0 {
                    get_valid_address(dest, mem.len())?
                } else {
                    ip + 3
                }
            }
            // jump zero
            6 => {
                let (condition, dest) = get_two_operands(ip, &mem)?;
                ip = if condition == 0 {
                    get_valid_address(dest, mem.len())?
                } else {
                    ip + 3
                }
            }
            // less than
            7 => {
                let (v1, v2, dest) = get_binary_op_operands(ip, &mem)?;
                mem[dest] = if v1 < v2 { 1 } else { 0 };
                ip += 4;
            }
            8 => {
                let (v1, v2, dest) = get_binary_op_operands(ip, &mem)?;
                mem[dest] = if v1 == v2 { 1 } else { 0 };
                ip += 4;
            }
            _ => {
                return Err(format!("Unknown opcode {}", mem[ip]));
            }
        };
        if ip >= mem.len() {
            return Err("Program did not halt".to_owned());
        }
    }
    Ok((State { mem, ip }, ReturnStatus::Halt, output))
}

fn get_input_dest(ip: usize, mem: &[isize]) -> Result<usize, String> {
    if ip + 1 >= mem.len() {
        return Err(format!(
            "Not enough operands for ip {} and mem.len() {}",
            ip,
            mem.len()
        ));
    }
    get_valid_address(mem[ip + 1], mem.len())
}

fn get_output_operand(ip: usize, mem: &[isize]) -> Result<isize, String> {
    if ip + 1 >= mem.len() {
        return Err(format!(
            "Not enough operands for ip {} and mem.len() {}",
            ip,
            mem.len()
        ));
    }
    get_value(mem[ip + 1], (mem[ip] / 100) % 10, mem)
}

fn get_two_operands(ip: usize, mem: &[isize]) -> Result<(isize, isize), String> {
    if ip + 2 >= mem.len() {
        return Err(format!(
            "Not enough operands for pc {} and mem.len() {}",
            ip,
            mem.len()
        ));
    }
    let v1 = get_value(mem[ip + 1], (mem[ip] / 100) % 10, mem)?;
    let v2 = get_value(mem[ip + 2], (mem[ip] / 1000) % 10, mem)?;

    Ok((v1, v2))
}

fn get_binary_op_operands(ip: usize, mem: &[isize]) -> Result<(isize, isize, usize), String> {
    if ip + 3 >= mem.len() {
        return Err(format!(
            "Not enough operands for pc {} and mem.len() {}",
            ip,
            mem.len()
        ));
    }
    let v1 = get_value(mem[ip + 1], (mem[ip] / 100) % 10, mem)?;
    let v2 = get_value(mem[ip + 2], (mem[ip] / 1000) % 10, mem)?;
    let dest = get_valid_address(mem[ip + 3], mem.len())?;
    Ok((v1, v2, dest))
}

fn get_valid_address(raw_address: isize, memsize: usize) -> Result<usize, String> {
    if raw_address < 0 || raw_address as usize >= memsize {
        Err(format!(
            "memory index {} is out of bounds (memsize: {})",
            raw_address, memsize
        ))
    } else {
        Ok(raw_address as usize)
    }
}

fn get_value(raw_value: isize, mode: isize, mem: &[isize]) -> Result<isize, String> {
    if mode == 1 {
        Ok(raw_value)
    } else if mode == 0 {
        get_value_at(raw_value, mem)
    } else {
        Err(format!("Unknown mode: {}", mode))
    }
}

fn get_value_at(raw_address: isize, mem: &[isize]) -> Result<isize, String> {
    Ok(mem[get_valid_address(raw_address, mem.len())?])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_valid_address() {
        assert_eq!(get_valid_address(42, 43), Ok(42));
        assert_eq!(get_valid_address(0, 43), Ok(0));

        assert!(get_valid_address(43, 43).is_err());
        assert!(get_valid_address(-1, 43).is_err());
    }

    #[test]
    fn get_value_at_works_for_valid_adress() {
        // given
        let mem = &[1, 2, 3];
        let address = 2;

        // when
        let result = get_value_at(address, mem);

        // then
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn get_value_at_fails_for_invalid_address() {
        // given
        let mem = &[1, 2, 3];
        let address = 3;

        // when
        let result = get_value_at(address, mem);

        // then
        assert!(result.is_err());
    }

    #[test]
    fn get_value_works_for_position_mode() {
        // given
        let mem = &[10, 20, 30];
        let raw_value = 2;
        let mode = 0;

        // when
        let result = get_value(raw_value, mode, mem);

        // then
        assert_eq!(result, Ok(30));
    }

    #[test]
    fn get_value_works_for_immediate_mode() {
        // given
        let mem = &[10, 20, 30];
        let raw_value = 2;
        let mode = 1;

        // when
        let result = get_value(raw_value, mode, mem);

        // then
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn test_day2_example_1() {
        // given
        let input = &[1];

        // when
        let (state, status, output) = run_program(
            State::new(vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50]),
            input,
        )
        .expect("Expected successful run");

        // then
        assert_eq!(state.mem.get(0), Some(&3500));
        assert_eq!(status, ReturnStatus::Halt);
        assert!(output.is_empty());
    }

    #[test]
    fn test_day2_example_2() {
        // given
        let input = &[1];

        // when
        let (state, status, output) =
            run_program(State::new(vec![1, 0, 0, 0, 99]), input).expect("Expected successful run");

        // then
        assert_eq!(state.mem.get(0), Some(&2));
        assert_eq!(status, ReturnStatus::Halt);
        assert!(output.is_empty());
    }

    fn test_program(prog: Vec<isize>, input: &[isize], expected_output: &[isize]) {
        let (_, status, output) =
            run_program(State::new(prog), input).expect("Expected program to halt gracefully");

        // then
        assert_eq!(status, ReturnStatus::Halt);
        assert_eq!(output, expected_output);
    }

    #[test]
    fn test_day5_examples() {
        // position mode: return 1 if input = 8
        test_program(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], &[8], &[1]);
        test_program(vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], &[80], &[0]);

        // position mode: return 1 if input < 8
        test_program(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], &[7], &[1]);
        test_program(vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], &[8], &[0]);

        // immediate mode: return 1 if input = 8
        test_program(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99], &[8], &[1]);
        test_program(vec![3, 3, 1108, -1, 8, 3, 4, 3, 99], &[9], &[0]);

        // immediate mode: return 1 if input < 8
        test_program(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99], &[7], &[1]);
        test_program(vec![3, 3, 1107, -1, 8, 3, 4, 3, 99], &[8], &[0]);

        // jump test: return 0 for zero input
        test_program(
            vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
            &[0],
            &[0],
        );
        test_program(
            vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9],
            &[-42],
            &[1],
        );
    }
}
