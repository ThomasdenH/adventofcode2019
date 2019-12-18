use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::multi::separated_nonempty_list;
use nom::sequence::separated_pair;
use nom::IResult;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::*;

fn div_ceil(a: u32, b: u32) -> u32 {
    let mut c = a / b;
    if c * b < a {
        c += 1;
    }
    c
}

#[derive(Debug, Eq, PartialEq, Error)]
#[error("could not parse input")]
pub struct ParseError;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Reaction<'a> {
    input: Vec<Ingredient<'a>>,
    output: Ingredient<'a>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ingredient<'a> {
    amount: u32,
    name: &'a str,
}

impl std::ops::Mul<u32> for Ingredient<'_> {
    type Output = Self;
    fn mul(mut self, other: u32) -> Self {
        self.amount *= other;
        self
    }
}

impl<'a> From<(u32, &'a str)> for Ingredient<'a> {
    fn from((amount, name): (u32, &'a str)) -> Self {
        Ingredient { amount, name }
    }
}

impl<'a> Reaction<'a> {
    pub fn requires_element(&self, s: &str) -> bool {
        self.input.iter().any(|other| s == other.name)
    }
}

pub fn parse_element(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_uppercase())(input)
}

pub fn parse_number(input: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), u32::from_str)(input)
}

pub fn parse_ingredient<'a>(input: &'a str) -> IResult<&str, Ingredient<'a>> {
    map(
        separated_pair(parse_number, tag(" "), parse_element),
        Ingredient::from,
    )(input)
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
            input: vec![(2, "VZBJ").into(), (3, "SWJZ").into()],
            output: (3, "QZLC").into()
        })
    );
}

pub struct ReactionGraph<'a> {
    reactions: Vec<Reaction<'a>>,
}

impl<'a> From<Vec<Reaction<'a>>> for ReactionGraph<'a> {
    fn from(reactions: Vec<Reaction<'a>>) -> Self {
        ReactionGraph { reactions }
    }
}

impl<'a> ReactionGraph<'a> {
    fn is_required_in_reaction(&self, ingredient_name: &str) -> bool {
        self.reactions.iter().any(|reaction| {
            reaction
                .input
                .iter()
                .any(|ingredient| ingredient.name == ingredient_name)
        })
    }

    fn remove_non_required_reaction(&mut self) -> Option<Reaction<'a>> {
        self.reactions
            .iter()
            .position(|reaction| !self.is_required_in_reaction(reaction.output.name))
            .map(|position| self.reactions.remove(position))
    }
}

#[derive(Default, Debug)]
pub struct IngredientList<'a> {
    ingredients: HashMap<&'a str, u32>,
}

impl<'a> IngredientList<'a> {
    fn remove_ingredient(&mut self, ingredient: Ingredient<'a>) -> u32 {
        if let Some(&current_amount) = self.ingredients.get(ingredient.name) {
            self.ingredients.remove(ingredient.name);
            div_ceil(current_amount, ingredient.amount)
        } else {
            0
        }
    }

    fn has_ingredient(&self, ingredient_name: &str) -> bool {
        self.ingredients.contains_key(ingredient_name)
    }

    fn add_ingredient(&mut self, ingredient: Ingredient<'a>) {
        if ingredient.amount > 0 {
            *self.ingredients.entry(ingredient.name).or_insert(0) += ingredient.amount;
        }
    }
}

pub fn part_1(input: Vec<Reaction<'_>>) -> u32 {
    let mut ingredients = IngredientList::default();
    ingredients.add_ingredient(Ingredient::from((1, "FUEL")));
    let mut reactions: ReactionGraph<'_> = input.into();
    while let Some(reaction) = dbg!(reactions.remove_non_required_reaction()) {
        // Now reverse this reaction and then remove it from the list
        let times = dbg!(&mut ingredients).remove_ingredient(reaction.output);
        for input in reaction.input {
            ingredients.add_ingredient(input * times);
        }
    }
    *dbg!(ingredients)
        .ingredients
        .get("ORE")
        .expect("could not create element")
}
