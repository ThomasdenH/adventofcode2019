use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::combinator::map_res;
use nom::multi::separated_nonempty_list;
use nom::sequence::separated_pair;
use nom::IResult;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::*;

fn div_ceil(a: u128, b: u128) -> u128 {
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
    amount: u128,
    name: &'a str,
}

impl std::ops::Mul<u128> for Ingredient<'_> {
    type Output = Self;
    fn mul(mut self, other: u128) -> Self {
        self.amount *= other;
        self
    }
}

impl<'a> From<(u128, &'a str)> for Ingredient<'a> {
    fn from((amount, name): (u128, &'a str)) -> Self {
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

pub fn parse_number(input: &str) -> IResult<&str, u128> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), u128::from_str)(input)
}

pub fn parse_ingredient<'a>(input: &'a str) -> IResult<&str, Ingredient<'a>> {
    map(
        separated_pair(parse_number, tag(" "), parse_element),
        Ingredient::from,
    )(input)
}

pub fn parse_reaction(input: &str) -> Result<Reaction<'_>, ParseError> {
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

#[derive(Clone)]
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
    ingredients: HashMap<&'a str, u128>,
}

impl<'a> IngredientList<'a> {
    fn remove_ingredient(&mut self, ingredient: Ingredient<'a>) -> u128 {
        if let Some(&current_amount) = self.ingredients.get(ingredient.name) {
            self.ingredients.remove(ingredient.name);
            div_ceil(current_amount, ingredient.amount)
        } else {
            0
        }
    }

    fn add_ingredient(&mut self, ingredient: Ingredient<'a>) {
        if ingredient.amount > 0 {
            *self.ingredients.entry(ingredient.name).or_insert(0) += ingredient.amount;
        }
    }
}

fn ore_required_for_fuel(mut reactions: ReactionGraph<'_>, fuel: u128) -> u128 {
    let mut ingredients = IngredientList::default();
    ingredients.add_ingredient(Ingredient::from((fuel, "FUEL")));
    while let Some(reaction) = reactions.remove_non_required_reaction() {
        // Now reverse this reaction and then remove it from the list
        let times = ingredients.remove_ingredient(reaction.output);
        for input in reaction.input {
            ingredients.add_ingredient(input * times);
        }
    }
    *ingredients
        .ingredients
        .get("ORE")
        .expect("could not create element")
}

pub fn part_1(input: Vec<Reaction<'_>>) -> u128 {
    let reactions = ReactionGraph::from(input);
    ore_required_for_fuel(reactions, 1)
}

pub fn part_2(input: Vec<Reaction<'_>>) -> u128 {
    let input_ore: u128 = 1_000_000_000_000;
    let reactions = ReactionGraph::from(input);
    // Do a binary search
    let (mut a, mut b) = {
        // At the minimum, you can repeatedly create one fuel and throw away the end product.
        // This requires more ore, so it underestimates the created fuel.
        let ore_for_one = ore_required_for_fuel(reactions.clone(), 1);
        let mut max: u128 = input_ore / ore_for_one;
        // Now find a maximum
        loop {
            match ore_required_for_fuel(reactions.clone(), max).cmp(&input_ore) {
                // The requirement is equal, we're done!
                Ordering::Equal => return max,
                // We have enough fuel, so the correct number is between max / 2 and max
                Ordering::Greater => break (max / 2, max),
                // We don't have enough fuel, try to add more ore
                Ordering::Less => max *= 2,
            }
        }
    };

    while b - a > 1 {
        let middle = (a + b) / 2;
        let ore_required = ore_required_for_fuel(reactions.clone(), middle);
        match ore_required.cmp(&input_ore) {
            Ordering::Equal => return middle,
            Ordering::Greater => {
                b = middle;
            }
            Ordering::Less => {
                a = middle;
            }
        }
    }
    a
}
