use crate::intcode::{parse_program, Computer, ComputerError, Value};
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::prelude::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use thiserror::*;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Direction {
    Up,
    Right,
    Left,
    Down,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }

    fn move_in_direction(&mut self, direction: Direction) {
        *self = match direction {
            Direction::Left => Point::new(self.x - 1, self.y),
            Direction::Right => Point::new(self.x + 1, self.y),
            Direction::Up => Point::new(self.x, self.y - 1),
            Direction::Down => Point::new(self.x, self.y + 1),
        };
    }
}

impl Direction {
    fn rotate_right(&mut self) {
        *self = match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    fn rotate_left(&mut self) {
        *self = match self {
            Direction::Right => Direction::Up,
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
        }
    }
}

struct EmergencyHullPaintingRobot {
    computer: Computer,
}

const CHANNEL_BUFFER_SIZE: usize = 2;

#[derive(Error, Debug)]
enum SolutionError {
    #[error("an error occurred in the computer")]
    ComputerError(#[from] ComputerError),
    #[error("could not parse color")]
    ColorError,
    #[error("invalid rotation")]
    InvalidRotation,
}

struct FieldRunner {
    field: Field,
    position: Point,
    direction: Direction,
}

impl FieldRunner {
    fn new(field: Field) -> Self {
        FieldRunner {
            field,
            position: Point::new(0, 0),
            direction: Direction::Up,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Color {
    Black,
    White,
}

impl TryFrom<Value> for Color {
    type Error = SolutionError;
    fn try_from(input: Value) -> Result<Self, Self::Error> {
        match input {
            0 => Ok(Color::Black),
            1 => Ok(Color::White),
            _ => Err(SolutionError::ColorError),
        }
    }
}

impl From<Color> for Value {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => 0,
            Color::White => 1,
        }
    }
}

pub struct Field {
    colors: HashMap<Point, Color>,
}

impl Field {
    fn new() -> Self {
        Field {
            colors: HashMap::new(),
        }
    }

    fn paint(&mut self, p: Point, c: Color) {
        self.colors.insert(p, c);
    }

    fn view_color(&mut self, p: Point) -> Color {
        *self.colors.get(&p).unwrap_or(&Color::Black)
    }

    fn unique_tiles_painted(&self) -> usize {
        self.colors.len()
    }
}

pub fn parse_input(input: &str) -> Vec<Value> {
    parse_program(input)
}
