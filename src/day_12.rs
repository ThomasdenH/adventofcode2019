/* use thiserror::*;
use scan_fmt::*;
use std::iter;

#[derive(Clone, Copy)]
pub struct Vector {
    x: i64,
    y: i64,
    z: i64
}

#[derive(Clone, Copy)]
pub struct Moon {
    pos: Vector,
    vel: Vector
}

impl Moon {
    fn force_from(self, other: Moon) -> Vector {
        Vector {
            x: (other.x - self.x).signum(),
            y: (other.y - self.y).signum(),
            z: (other.z - self.z).signum()
        }
    }

    fn time_step(self, force: Vector) -> Moon {
        self.
    }
}

#[derive(Error, Debug)]
enum SolutionError {
    #[error("could not parse the input")]
    ParseError
}

pub fn parse_input(input: &str) -> Result<Vec<Moon>, SolutionError> {
    input.trim()
        .lines()
        .map(|line| scan_fmt!("<x={d}, y={d}, z={d}>").map_err(|_| SolutionError::ParseError))
        .collect()
}

pub compute_force(moon: Moon, moons: &[Moons]) -> Vector {
    moons.iter(|other| moon.force_from(other)).sum()
}

pub fn simulate(moons: &mut [Moon]>) -> impl Iterator<Item = &[Moon]> {
    iter::successors(moons, |moons| {
        
    });
}
*/