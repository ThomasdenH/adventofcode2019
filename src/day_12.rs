use scan_fmt::*;
use std::fmt;
use std::iter;
use std::iter::Sum;
use std::ops::AddAssign;
use std::ops::{Add, Sub};
use thiserror::*;
use std::collections::HashSet;

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct Vector {
    x: i64,
    y: i64,
    z: i64,
}

impl fmt::Debug for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<x={}, y={}, z={}>", self.x, self.y, self.z)
    }
}

impl Vector {
    /// The sum of the absolute values of each coordinate
    fn one_norm(self) -> u64 {
        (self.x.abs() + self.y.abs() + self.z.abs()) as u64
    }
}

impl Sum for Vector {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Vector>,
    {
        iter.fold(Vector::default(), |acc, x| acc + x)
    }
}

impl Vector {
    fn signum(self) -> Self {
        Vector {
            x: self.x.signum(),
            y: self.y.signum(),
            z: self.z.signum(),
        }
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;
    fn add(mut self, other: Vector) -> Vector {
        self += other;
        self
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;
    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Moon {
    pos: Vector,
    vel: Vector,
}

impl fmt::Debug for Moon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pos={:?} vel={:?}", self.pos, self.vel)
    }
}

impl Moon {
    fn from_pos(pos: Vector) -> Moon {
        Moon {
            pos,
            vel: Vector::default(),
        }
    }

    fn force_from(self, other: Moon) -> Vector {
        (other.pos - self.pos).signum()
    }

    fn force(self, moons: &[Moon]) -> Vector {
        moons.iter().map(|other| self.force_from(*other)).sum()
    }

    fn time_step(&mut self, force: Vector) {
        self.vel += force;
        self.pos += self.vel;
    }

    fn potential_energy(self) -> u64 {
        self.pos.one_norm()
    }

    fn kinetic_energy(self) -> u64 {
        self.vel.one_norm()
    }

    fn total_energy(self) -> u64 {
        dbg!((self.pos, self.vel));
        dbg!(self.potential_energy()) * dbg!(self.kinetic_energy())
    }
}

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("could not parse the input")]
    ParseError,
}

pub fn parse_state(input: &str) -> Result<Vec<Moon>, SolutionError> {
    Ok(input
        .trim()
        .lines()
        .map(|line| {
            scan_fmt!(
                line,
                "pos=<x={d}, y={d}, z={d}>, vel=<x={d}, y={d}, z={d}>",
                i64,
                i64,
                i64,
                i64,
                i64,
                i64
            )
            .map(|(x, y, z, vx, vy, vz)| Moon {
                pos: Vector { x, y, z },
                vel: Vector {
                    x: vx,
                    y: vy,
                    z: vz,
                },
            })
            .unwrap()
        })
        .collect())
}

pub fn parse_input(input: &str) -> Result<Vec<Moon>, SolutionError> {
    Ok(input
        .trim()
        .lines()
        .map(|line| {
            scan_fmt!(line, "<x={d}, y={d}, z={d}>", i64, i64, i64)
                .map(|(x, y, z)| Moon::from_pos(Vector { x, y, z }))
                .unwrap()
        })
        .collect())
}

pub fn simulate<'a>(moons: &'a [Moon]) -> impl Iterator<Item = Vec<Moon>> {
    iter::successors(Some(moons.to_vec()), |moons| {
        let forces = moons
            .iter()
            .map(|moon| moon.force(&moons))
            .collect::<Vec<_>>();
        let mut moons = moons.to_vec();
        for (moon, force) in moons.iter_mut().zip(forces.into_iter()) {
            moon.time_step(force);
        }
        Some(moons)
    })
}

pub fn part_1(moons: Vec<Moon>) -> u64 {
    simulate(&moons)
        .nth(1000)
        .unwrap()
        .iter()
        .map(|moon| moon.total_energy())
        .sum()
}

pub fn part_2(moons: Vec<Moon>) -> usize {
    let x = one_dimensional_loop([
        (moons[0].pos.x, moons[0].vel.x),
        (moons[1].pos.x, moons[1].vel.x),
        (moons[2].pos.x, moons[2].vel.x),
        (moons[3].pos.x, moons[3].vel.x),
    ]);
    let y = one_dimensional_loop([
        (moons[0].pos.y, moons[0].vel.y),
        (moons[1].pos.y, moons[1].vel.y),
        (moons[2].pos.y, moons[2].vel.y),
        (moons[3].pos.y, moons[3].vel.y),
    ]);
    let z = one_dimensional_loop([
        (moons[0].pos.z, moons[0].vel.z),
        (moons[1].pos.z, moons[1].vel.z),
        (moons[2].pos.z, moons[2].vel.z),
        (moons[3].pos.z, moons[3].vel.z),
    ]);
    lcm(lcm(x, y), z)
}

fn gcd(a: usize, b: usize) -> usize {
    if b > a {
        gcd(b, a)
    } else if b == 0 {
        a
    } else {
        // a >= b > 0
        gcd(b, a % b)
    }
}

fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}

pub fn one_dimensional_loop(mut moons: [(i64, i64); 4]) -> usize {
    let mut set: HashSet<[(i64, i64); 4]> = HashSet::new();
    let mut force = [0i64, 0, 0, 0];
    for i in 0.. {
        if set.contains(&moons) {
            return i;
        }
        set.insert(moons);
        for (force, moon) in force.iter_mut().zip(moons.iter()) {
            *force = moons.iter().map(|other| (other.0 - moon.0).signum()).sum();
        }
        for (moon, force) in moons.iter_mut().zip(force.iter()) {
            moon.1 += force;
            moon.0 += moon.1;
        }
    }
    unreachable!();
}

#[test]
fn test_example_one() {
    let mut moons = vec![
        Moon::from_pos(Vector { x: -1, y: 0, z: 2 }),
        Moon::from_pos(Vector {
            x: 2,
            y: -10,
            z: -7,
        }),
        Moon::from_pos(Vector { x: 4, y: -8, z: 8 }),
        Moon::from_pos(Vector { x: 3, y: 5, z: -1 }),
    ];
    let mut simulated = simulate(&moons);
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x=-1, y=  0, z= 2>, vel=<x= 0, y= 0, z= 0>
    pos=<x= 2, y=-10, z=-7>, vel=<x= 0, y= 0, z= 0>
    pos=<x= 4, y= -8, z= 8>, vel=<x= 0, y= 0, z= 0>
    pos=<x= 3, y=  5, z=-1>, vel=<x= 0, y= 0, z= 0>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 2, y=-1, z= 1>, vel=<x= 3, y=-1, z=-1>
    pos=<x= 3, y=-7, z=-4>, vel=<x= 1, y= 3, z= 3>
    pos=<x= 1, y=-7, z= 5>, vel=<x=-3, y= 1, z=-3>
    pos=<x= 2, y= 2, z= 0>, vel=<x=-1, y=-3, z= 1>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 5, y=-3, z=-1>, vel=<x= 3, y=-2, z=-2>
    pos=<x= 1, y=-2, z= 2>, vel=<x=-2, y= 5, z= 6>
    pos=<x= 1, y=-4, z=-1>, vel=<x= 0, y= 3, z=-6>
    pos=<x= 1, y=-4, z= 2>, vel=<x=-1, y=-6, z= 2>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 5, y=-6, z=-1>, vel=<x= 0, y=-3, z= 0>
    pos=<x= 0, y= 0, z= 6>, vel=<x=-1, y= 2, z= 4>
    pos=<x= 2, y= 1, z=-5>, vel=<x= 1, y= 5, z=-4>
    pos=<x= 1, y=-8, z= 2>, vel=<x= 0, y=-4, z= 0>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 2, y=-8, z= 0>, vel=<x=-3, y=-2, z= 1>
    pos=<x= 2, y= 1, z= 7>, vel=<x= 2, y= 1, z= 1>
    pos=<x= 2, y= 3, z=-6>, vel=<x= 0, y= 2, z=-1>
    pos=<x= 2, y=-9, z= 1>, vel=<x= 1, y=-1, z=-1>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x=-1, y=-9, z= 2>, vel=<x=-3, y=-1, z= 2>
    pos=<x= 4, y= 1, z= 5>, vel=<x= 2, y= 0, z=-2>
    pos=<x= 2, y= 2, z=-4>, vel=<x= 0, y=-1, z= 2>
    pos=<x= 3, y=-7, z=-1>, vel=<x= 1, y= 2, z=-2>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x=-1, y=-7, z= 3>, vel=<x= 0, y= 2, z= 1>
            pos=<x= 3, y= 0, z= 0>, vel=<x=-1, y=-1, z=-5>
            pos=<x= 3, y=-2, z= 1>, vel=<x= 1, y=-4, z= 5>
            pos=<x= 3, y=-4, z=-2>, vel=<x= 0, y= 3, z=-1>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 2, y=-2, z= 1>, vel=<x= 3, y= 5, z=-2>
            pos=<x= 1, y=-4, z=-4>, vel=<x=-2, y=-4, z=-4>
            pos=<x= 3, y=-7, z= 5>, vel=<x= 0, y=-5, z= 4>
            pos=<x= 2, y= 0, z= 0>, vel=<x=-1, y= 4, z= 2>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 5, y= 2, z=-2>, vel=<x= 3, y= 4, z=-3>
            pos=<x= 2, y=-7, z=-5>, vel=<x= 1, y=-3, z=-1>
            pos=<x= 0, y=-9, z= 6>, vel=<x=-3, y=-2, z= 1>
            pos=<x= 1, y= 1, z= 3>, vel=<x=-1, y= 1, z= 3>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 5, y= 3, z=-4>, vel=<x= 0, y= 1, z=-2>
            pos=<x= 2, y=-9, z=-3>, vel=<x= 0, y=-2, z= 2>
            pos=<x= 0, y=-8, z= 4>, vel=<x= 0, y= 1, z=-2>
            pos=<x= 1, y= 1, z= 5>, vel=<x= 0, y= 0, z= 2>"
        )
        .unwrap()
    );
    assert_eq!(
        simulated.next().unwrap(),
        parse_state(
            "pos=<x= 2, y= 1, z=-3>, vel=<x=-3, y=-2, z= 1>
            pos=<x= 1, y=-8, z= 0>, vel=<x=-1, y= 1, z= 3>
            pos=<x= 3, y=-6, z= 1>, vel=<x= 3, y= 2, z=-3>
            pos=<x= 2, y= 0, z= 4>, vel=<x= 1, y=-1, z=-1>"
        )
        .unwrap()
    );

    assert_eq!(parse_state(
        "pos=<x= 2, y= 1, z=-3>, vel=<x=-3, y=-2, z= 1>
        pos=<x= 1, y=-8, z= 0>, vel=<x=-1, y= 1, z= 3>
        pos=<x= 3, y=-6, z= 1>, vel=<x= 3, y= 2, z=-3>
        pos=<x= 2, y= 0, z= 4>, vel=<x= 1, y=-1, z=-1>"
    )
    .unwrap()
    .iter()
    .map(|moon| dbg!(moon.total_energy())).sum::<u64>(), 179);

    let energy_after_10_steps: u64 = simulate(&moons)
        .nth(10)
        .unwrap()
        .iter()
        .map(|moon| dbg!(moon.total_energy()))
        .sum();
    assert_eq!(energy_after_10_steps, 179u64);
}
