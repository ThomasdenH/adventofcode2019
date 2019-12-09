use adventofcode::day_8::Solution;
use anyhow::Result;
use aoc_types::DaySolution;

#[test]
fn test_part_1() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day8")?;
    file.read_to_string(&mut s)?;
    assert_eq!(Solution::part_1(&Solution::parse_input(&s)?)?, 2210);
    Ok(())
}

#[test]
fn test_part_2() -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    let mut s = String::new();
    let mut file = File::open("input/day8")?;
    file.read_to_string(&mut s)?;
    assert_eq!(
        Solution::part_2(&Solution::parse_input(&s)?)?.to_string(),
        "\
_XX___XX__XXXX__XX__XXXX_
X__X_X__X_X____X__X_X____
X____X____XXX__X____XXX__
X____X_XX_X____X_XX_X____
X__X_X__X_X____X__X_X____
_XX___XXX_XXXX__XXX_XXXX_
"
    );
    Ok(())
}
