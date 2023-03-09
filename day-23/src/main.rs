use intcode::{parse, run_program, ReturnStatus, State};
use std::env;
use std::fs::read_to_string;
use std::path::Path;

fn main() -> Result<(), String> {
    let filename = env::args()
        .nth(1)
        .ok_or_else(|| "No file name given.".to_owned())?;
    let content = read_to_string(Path::new(&filename)).map_err(|e| e.to_string())?;
    let program = parse(&content)?;

    let (first_nat, last_nat) = run_servers(&program)?;

    if let Some(y255) = first_nat {
        println!("The Y value of the first package sent to address 255 is {y255}.");
    } else {
        println!("Nothing was ever sent to address 255.");
    }

    if let Some(nat) = last_nat {
        println!("The first Y value delivered by the NAT to the computer at address 0 twice in a row is {nat}");
    } else {
        println!("Nothing was sent from the NAT to a server twice in a row");
    }

    Ok(())
}

// There are several ways to do this, I chose one that does not require me to rewrite my intcode
// interpreter
fn run_servers(program: &[isize]) -> Result<(Option<isize>, Option<isize>), String> {
    let mut servers: Vec<(State, ReturnStatus, Vec<isize>)> = (0..50)
        .map(|i| (State::new(program.to_vec()), ReturnStatus::Wait, vec![i]))
        .collect();

    let mut all_halt = false;
    let mut first_nat: Option<isize> = None;
    let mut last_nat: Option<(isize, isize)> = None;
    let mut last_delivered_nat: Option<isize> = None;

    while !all_halt {
        all_halt = true;
        for server_i in 0..servers.len() {
            let (state, status, queue) = &mut servers[server_i];
            if *status == ReturnStatus::Halt {
                continue;
            }
            // if no input is provided, the input will always be a -1
            if queue.is_empty() {
                queue.push(-1);
            }
            // I hate to clone `state` here, but I don't want to rewrite the intcode interpreter
            let (new_state, new_status, output) = run_program(state.clone(), queue)?;
            all_halt = all_halt && new_status == ReturnStatus::Halt;
            queue.clear();
            *state = new_state;
            *status = new_status;

            // assume that sending packets is always done in one go (i.e. all three outputs are
            // written without reading anything in between). If this is not the case, we need to
            // rewrite this
            if output.len() % 3 != 0 {
                return Err(
                    "Incomplete send operation. I was hoping™ that this would not happen."
                        .to_owned(),
                );
            }

            for send_op in output.chunks_exact(3) {
                if send_op[0] == 255 {
                    if first_nat.is_none() {
                        first_nat = Some(send_op[2]);
                    }
                    last_nat = Some((send_op[1], send_op[2]));
                } else if let Some(dest) = TryInto::<usize>::try_into(send_op[0])
                    .ok()
                    .and_then(|i| servers.get_mut(i))
                {
                    dest.2.push(send_op[1]);
                    dest.2.push(send_op[2]);
                } else {
                    eprintln!(
                        "Sending packet to unknown address {}, discarding packet.",
                        send_op[0]
                    );
                }
            }
        }
        if servers.iter().all(|(_, _, queue)| queue.is_empty()) {
            if last_delivered_nat.is_some() && last_nat.map(|(_, y)| y) == last_delivered_nat {
                return Ok((first_nat, last_delivered_nat));
            }
            if let Some((x, y)) = last_nat {
                servers[0].2.push(x);
                servers[0].2.push(y);
                last_delivered_nat = Some(y);
            }
        }
    }

    Ok((first_nat, None))
}

#[cfg(test)]
mod test {
    use super::*;
}
