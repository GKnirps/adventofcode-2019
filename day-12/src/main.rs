use std::cmp;

fn main() -> Result<(), String> {
    // hardcoded input again
    let moons: &[Body] = &[
        Body::new((17, -7, -11)),
        Body::new((1, 4, -1)),
        Body::new((6, -2, -6)),
        Body::new((19, 11, 9)),
    ];

    let after_1000 = n_steps(moons, 1000);
    println!(
        "System energy after 1000 steps: {}",
        system_energy(&after_1000)
    );

    let cycle_length = steps_for_cycle(moons);
    println!("{} steps until we see the first cycle", cycle_length);

    Ok(())
}

type Vec3 = (i32, i32, i32);

fn sum_vec3((x1, y1, z1): Vec3, (x2, y2, z2): Vec3) -> Vec3 {
    (x1 + x2, y1 + y2, z1 + z2)
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct Body {
    pos: Vec3,
    vel: Vec3,
}

impl Body {
    fn new(pos: Vec3) -> Body {
        Body {
            pos,
            vel: (0, 0, 0),
        }
    }
}

fn grav_accel(accelerated: &Body, other: &Body) -> Vec3 {
    let (ax, ay, az) = accelerated.pos;
    let (ox, oy, oz) = other.pos;
    ((ox - ax).signum(), (oy - ay).signum(), (oz - az).signum())
}

fn n_steps(initial_bodies: &[Body], n: usize) -> Vec<Body> {
    let mut bodies: Vec<Body> = initial_bodies.to_vec();
    for _ in 0..n {
        bodies = step(&bodies);
    }
    bodies
}

fn step(bodies: &[Body]) -> Vec<Body> {
    bodies
        .iter()
        .map(|body| {
            let vel = bodies
                .iter()
                .map(|other| grav_accel(body, other))
                .fold(body.vel, sum_vec3);
            Body {
                pos: sum_vec3(body.pos, vel),
                vel,
            }
        })
        .collect()
}

fn step_1d(dims: &[(i32, i32)]) -> Vec<(i32, i32)> {
    dims.iter()
        .map(|(pos, vel)| {
            let nextvel = vel
                + dims
                    .iter()
                    .map(|(o_pos, _)| (o_pos - pos).signum())
                    .sum::<i32>();
            (pos + nextvel, nextvel)
        })
        .collect()
}

fn steps_for_cycle_1d(dims: &[(i32, i32)]) -> usize {
    let mut previous = dims.to_vec();
    let mut count: usize = 0;

    loop {
        let next = step_1d(&previous);
        count += 1;
        if next == dims {
            return count;
        }
        previous = next;
    }
}

fn greatest_common_divisor(x: usize, y: usize) -> usize {
    for d in (1..=cmp::min(x, y)).rev() {
        if x % d == 0 && y % d == 0 {
            return d;
        }
    }
    1
}

fn steps_for_cycle(bodies: &[Body]) -> usize {
    let dim_x: Vec<(i32, i32)> = bodies.iter().map(|body| (body.pos.0, body.vel.0)).collect();
    let dim_y: Vec<(i32, i32)> = bodies.iter().map(|body| (body.pos.1, body.vel.1)).collect();
    let dim_z: Vec<(i32, i32)> = bodies.iter().map(|body| (body.pos.2, body.vel.2)).collect();

    let cycle_x = steps_for_cycle_1d(&dim_x);
    let cycle_y = steps_for_cycle_1d(&dim_y);
    let cycle_z = steps_for_cycle_1d(&dim_z);

    let gcd_xy = greatest_common_divisor(cycle_x, cycle_y);
    let lcm_xy = (cycle_x / gcd_xy) * cycle_y;

    let gcd_xyz = greatest_common_divisor(lcm_xy, cycle_z);
    (lcm_xy / gcd_xyz) * cycle_z
}

fn body_energy(body: &Body) -> i32 {
    let Body {
        pos: (px, py, pz),
        vel: (vx, vy, vz),
    } = body;
    (px.abs() + py.abs() + pz.abs()) * (vx.abs() + vy.abs() + vz.abs())
}

fn system_energy(bodies: &[Body]) -> i32 {
    bodies.iter().map(body_energy).sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_step() {
        // given
        let moons: &[Body] = &[
            Body::new((-1, 0, 2)),
            Body::new((2, -10, -7)),
            Body::new((4, -8, 8)),
            Body::new((3, 5, -1)),
        ];

        // when
        let result = step(moons);

        // then
        assert_eq!(
            &result,
            &[
                Body {
                    pos: (2, -1, 1),
                    vel: (3, -1, -1)
                },
                Body {
                    pos: (3, -7, -4),
                    vel: (1, 3, 3)
                },
                Body {
                    pos: (1, -7, 5),
                    vel: (-3, 1, -3)
                },
                Body {
                    pos: (2, 2, 0),
                    vel: (-1, -3, 1)
                },
            ]
        );
    }

    #[test]
    fn test_body_energy() {
        // given
        let body = Body {
            pos: (2, 1, -3),
            vel: (-3, -2, 1),
        };

        // when
        let result = body_energy(&body);

        // then
        assert_eq!(result, 36);
    }

    #[test]
    fn test_cycle_example_1() {
        // given
        let moons: &[Body] = &[
            Body::new((-1, 0, 2)),
            Body::new((2, -10, -7)),
            Body::new((4, -8, 8)),
            Body::new((3, 5, -1)),
        ];

        // when
        let result = steps_for_cycle(moons);

        // then
        assert_eq!(result, 2772);
    }
}
