use crate::intcode::{Computer, ComputerError, Value};
use permutohedron::Heap;
use std::fmt::Debug;
use thiserror::*;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("could not parse the input ints")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("a computer error occurred")]
    ComputerError(#[from] ComputerError),
}

pub struct Solution {}

async fn output(memory: Vec<Value>, input: &[Value]) -> Result<Value, ComputerError> {
    let mut output_a: &mut Value = &mut 0;
    Computer::load(memory)
        .run(&mut input.iter().copied(), Some(&mut output_a))
        .await?;
    Ok(*output_a)
}

/// Parse the input to a common format between both parts.
pub fn parse_input(input: &str) -> Result<Vec<Value>, SolutionError> {
    input
        .trim()
        .split(',')
        .map(|s| s.parse::<Value>().map_err(SolutionError::from))
        .collect()
}

/// Solve the first part for the parsed input.
pub async fn part_1(parsed_input: &[Value]) -> Result<Value, SolutionError> {
    let mut max = None;
    for perm in Heap::new(&mut [0, 1, 2, 3, 4]) {
        let mut previous = 0;
        for phase_setting in &perm {
            previous = output(parsed_input.to_vec(), &[*phase_setting, previous]).await?;
        }
        max = Some(max.map(|m: Value| m.max(previous)).unwrap_or(previous));
    }
    Ok(max.unwrap())
}

/// Solve the second part for the parsed input.
fn part_2(parsed_input: &[Value]) -> Result<Value, SolutionError> {
    Ok(0)
}
