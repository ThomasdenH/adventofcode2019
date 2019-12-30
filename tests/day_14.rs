pub use adventofcode::day_14::{parse_input, part_1, part_2};
use anyhow::Result;

#[test]
fn test_part_1() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day14")?;
    file.read_to_string(&mut s)?;
    assert_eq!(part_1(parse_input(&s)?), 899155);
    Ok(())
}

#[test]
fn test_part_2() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day14")?;
    file.read_to_string(&mut s)?;
    assert_eq!(part_2(parse_input(&s)?), 2390226);
    Ok(())
}
