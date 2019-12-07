use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_file(&Path::new(&filename)).map_err(|e| e.to_string())?;

    let initial_state = parse(&content)?;

    println!("Start process");
    match run_program(
        initial_state,
        &mut BufReader::new(io::stdin()),
        &mut io::stdout(),
    ) {
        Ok(_) => println!("Process halted"),
        Err(e) => println!("Process failed: {}", e),
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

fn parse(input: &str) -> Result<Vec<isize>, String> {
    input
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<isize>().map_err(|e| e.to_string()))
        .collect()
}

fn run_program(
    mem: Vec<isize>,
    source: &mut dyn BufRead,
    sink: &mut dyn Write,
) -> Result<isize, String> {
    let mut mem = mem;
    if mem.is_empty() {
        return Err("no program".to_owned());
    }
    let mut ip: usize = 0;
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
                let value = read(source)?;
                mem[dest] = value;
                ip += 2;
            }
            // write
            4 => {
                writeln!(sink, "{}", get_output_operand(ip, &mem)?).map_err(|e| e.to_string())?;
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
    Ok(mem[0])
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

fn read(source: &mut dyn BufRead) -> Result<isize, String> {
    let mut input = String::with_capacity(64);
    source.read_line(&mut input).map_err(|e| e.to_string())?;
    input.trim().parse::<isize>().map_err(|e| e.to_string())
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
        let input = b"1\n";
        let mut output = Vec::new();

        // when
        let result = run_program(
            vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50],
            &mut &input[..],
            &mut output,
        );

        // then
        assert_eq!(result, Ok(3500));
    }

    #[test]
    fn test_day2_example_2() {
        // given
        let input = b"1\n";
        let mut output = Vec::new();

        // when
        let result = run_program(vec![1, 0, 0, 0, 99], &mut &input[..], &mut output);

        // then
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn test_day5_position_equals_8_true() {
        // given
        let input = b"8\n";
        let mut output = Vec::new();

        // when
        run_program(
            vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8],
            &mut &input[..],
            &mut output,
        )
        .expect("Expected program to halt gracefully");

        // then
        assert_eq!(&output, b"1\n");
    }

    #[test]
    fn test_day5_position_equals_8_false() {
        // given
        let input = b"80\n";
        let mut output = Vec::new();

        // when
        run_program(
            vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8],
            &mut &input[..],
            &mut output,
        )
        .expect("Expected program to halt gracefully");

        // then
        assert_eq!(&output, b"0\n");
    }

    #[test]
    fn test_day5_position_lt_8_true() {
        // given
        let input = b"7\n";
        let mut output = Vec::new();

        // when
        run_program(
            vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8],
            &mut &input[..],
            &mut output,
        )
        .expect("Expected program to halt gracefully");

        // then
        assert_eq!(&output, b"1\n");
    }
}
