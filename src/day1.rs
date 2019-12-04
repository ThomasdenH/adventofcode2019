use crate::Result;

pub fn parse_input(s: &str) -> Result<Vec<u32>> {
    s.lines()
        .map(|s| s.trim().parse::<u32>().map_err(|e| e.into()))
        .collect()
}

fn fuel_requirement(mass: u32) -> u32 {
    if mass < 6 {
        0
    } else {
        (mass / 3) - 2
    }
}

pub fn part_1(input: impl Iterator<Item = u32>) -> u32 {
    input.map(fuel_requirement).sum()
}

pub fn part_2(input: impl Iterator<Item = u32>) -> u32 {
    input
        .map(|mass| {
            let mut total = 0;
            let mut current = fuel_requirement(mass);
            while current > 0 {
                total += current;
                current = fuel_requirement(current);
            }
            total
        })
        .sum()
}
