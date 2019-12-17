use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::combinator::all_consuming;
use nom::combinator::map_res;
use nom::multi::separated_nonempty_list;
use nom::sequence::separated_pair;
use nom::IResult;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub struct ParseError;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Reaction<'a> {
    input: Vec<(u32, &'a str)>,
    output: (u32, &'a str),
}

pub fn parse_element(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_uppercase())(input)
}

pub fn parse_number(input: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), u32::from_str)(input)
}

pub fn parse_ingredient(input: &str) -> IResult<&str, (u32, &str)> {
    separated_pair(parse_number, tag(" "), parse_element)(input)
}

pub fn parse_reaction<'a>(input: &'a str) -> Result<Reaction<'a>, ParseError> {
    all_consuming(separated_pair(
        separated_nonempty_list(tag(", "), parse_ingredient),
        tag(" => "),
        parse_ingredient,
    ))(input)
    .map(|(_, (input, output))| Reaction { input, output })
    .map_err(|_| ParseError {})
}

pub fn parse_input<'a>(input: &'a str) -> Result<Vec<Reaction<'a>>, ParseError> {
    input.trim().lines().map(parse_reaction).collect()
}

#[test]
fn test_parse() {
    assert_eq!(
        parse_reaction("2 VZBJ, 3 SWJZ => 3 QZLC"),
        Ok(Reaction {
            input: vec![(2, "VZBJ"), (3, "SWJZ")],
            output: (3, "QZLC")
        })
    );
}
