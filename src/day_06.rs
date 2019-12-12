use aoc_types::DaySolution;
use std::collections::{HashMap, HashSet};
use std::iter;
use thiserror::*;

fn descendants_of<'a>(
    planet: &'a str,
    descendants: &mut HashMap<&'a str, u32>,
    orbits: &[(&'a str, &'a str)],
) -> u32 {
    if let Some(&des) = descendants.get(planet) {
        des
    } else {
        let count = orbits
            .iter()
            .filter_map(|(a, orbiter)| {
                if *a == planet {
                    Some(1 + descendants_of(orbiter, descendants, orbits))
                } else {
                    None
                }
            })
            .sum();
        descendants.insert(planet, count);
        count
    }
}

pub struct Solution {}
type ParsedInput<'a> = Vec<(&'a str, &'a str)>;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("no path was found")]
    NoPathFound,
    #[error("could not parse the input")]
    CouldNotParse,
}

impl<'a, 'b, 'c> DaySolution<'a, 'b, 'c, ParsedInput<'a>, u32, u32, SolutionError> for Solution {
    fn parse_input(input: &'a str) -> Result<ParsedInput<'a>, SolutionError> {
        input
            .trim()
            .lines()
            .map(|line| -> Result<(&str, &str), SolutionError> {
                let mut coords = line.split(')');
                Ok((
                    coords.next().ok_or(SolutionError::CouldNotParse)?,
                    coords.next().ok_or(SolutionError::CouldNotParse)?,
                ))
            })
            .collect()
    }

    fn part_1(orbits: &ParsedInput<'a>) -> Result<u32, SolutionError> {
        let planets: HashSet<&str> = orbits
            .iter()
            .flat_map(|(a, b)| iter::once(*a).chain(iter::once(*b)))
            .collect();

        let mut descendants: HashMap<&str, u32> = HashMap::new();

        let mut total = 0;
        for planet in planets {
            total += descendants_of(planet, &mut descendants, orbits);
        }
        Ok(total)
    }

    fn part_2(orbits: &ParsedInput<'a>) -> Result<u32, SolutionError> {
        let planets: HashSet<&str> = orbits
            .iter()
            .flat_map(|(a, b)| iter::once(*a).chain(iter::once(*b)))
            .collect();
        let parent: HashMap<&str, &str> = planets
            .iter()
            .filter_map(|planet| {
                orbits
                    .iter()
                    .find_map(|(a, b)| if b == planet { Some(*a) } else { None })
                    .map(|parent| (*planet, parent))
            })
            .collect();
        let mut distance: HashMap<&str, u32> = HashMap::new();

        let mut a = "SAN";
        let mut b = "YOU";

        let mut current_distance = 0;
        while let Some(parent_of_you) = parent.get(b) {
            b = parent_of_you;
            distance.insert(b, current_distance);
            current_distance += 1;
        }

        current_distance = 0;
        while let Some(parent_of_san) = parent.get(a) {
            a = parent_of_san;
            if let Some(d) = distance.get(a) {
                return Ok(d + current_distance);
            }
            current_distance += 1;
        }
        Err(SolutionError::NoPathFound)
    }
}
