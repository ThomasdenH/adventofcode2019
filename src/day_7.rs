use crate::intcode::{Computer, ComputerError, Value};
use aoc_types::DaySolution;
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

fn output(memory: Vec<Value>, mut input: &[Value]) -> Result<Value, ComputerError> {
    let mut output_a: &mut Value = &mut 0;
    Computer::load(memory).run(&mut input.iter().copied(), Some(&mut output_a))?;
    Ok(*output_a)
}

impl<'a, 'b, 'c> DaySolution<'a, 'b, 'c, Vec<Value>, Value, Value, SolutionError> for Solution {
    /// Parse the input to a common format between both parts.
    fn parse_input(input: &'a str) -> Result<Vec<Value>, SolutionError> {
        input
            .trim()
            .split(',')
            .map(|s| s.parse::<Value>().map_err(SolutionError::from))
            .collect()
    }

    /// Solve the first part for the parsed input.
    fn part_1(parsed_input: &'b Vec<Value>) -> Result<Value, SolutionError> {
        Ok(Heap::new(&mut [0, 1, 2, 3, 4])
            .map(|perm| -> Result<Value, ComputerError> {
                perm.iter().fold(Ok(0), |previous, phase_setting| {
                    output(parsed_input.to_vec(), &[*phase_setting, previous?])
                })
            })
            .fold(Ok(None), |acc, x| -> Result<Option<Value>, SolutionError> {
                let x: Value = x?;
                Ok(if let Some(acc) = acc? {
                    Some(x.max(acc))
                } else {
                    Some(x)
                })
            })?
            .expect("at least one permutation should appear"))
    }

    /// Solve the second part for the parsed input.
    fn part_2(parsed_input: &'c Vec<Value>) -> Result<Value, SolutionError> {
        Ok(0)
    }
}
