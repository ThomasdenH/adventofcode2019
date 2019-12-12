use adventofcode::day_04::*;

#[test]
fn test_part_1() {
    assert_eq!(part_1(parse_input("235741-706948")), 1178);
}

#[test]
fn test_part_2() {
    assert_eq!(part_2(parse_input("235741-706948")), 763);
}
