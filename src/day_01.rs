use anyhow::Result;

pub fn parse_input(s: &str) -> Result<Vec<u32>> {
    s.lines()
        .map(|s| s.trim().parse::<u32>().map_err(|e| e.into()))
        .collect()
}

fn base_fuel_requirement(mass: u32) -> u32 {
    if mass < 6 {
        0
    } else {
        (mass / 3) - 2
    }
}

fn cumulative_fuel_requirement(mass: u32) -> u32 {
    let mut total = 0;
    let mut current = base_fuel_requirement(mass);
    while current > 0 {
        total += current;
        current = base_fuel_requirement(current);
    }
    total
}

pub fn part_1(input: impl Iterator<Item = u32>) -> u32 {
    input.map(base_fuel_requirement).sum()
}

pub fn part_2(input: impl Iterator<Item = u32>) -> u32 {
    input.map(cumulative_fuel_requirement).sum()
}

#[test]
fn test_base_fuel_requirement() {
    assert_eq!(base_fuel_requirement(12), 2);
    assert_eq!(base_fuel_requirement(14), 2);
    assert_eq!(base_fuel_requirement(1969), 654);
    assert_eq!(base_fuel_requirement(100756), 33583);
}

#[test]
fn test_cumulative_fuel_requirement() {
    assert_eq!(cumulative_fuel_requirement(12), 2);
    assert_eq!(cumulative_fuel_requirement(14), 2);
    assert_eq!(cumulative_fuel_requirement(1969), 966);
    assert_eq!(cumulative_fuel_requirement(100756), 50346);
}
