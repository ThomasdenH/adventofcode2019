use crate::intcode::{Computer, ComputerError, Value, Memory};
use futures::channel::mpsc::{Receiver, Sender, channel};
use futures::prelude::*;
use std::collections::HashMap;
use std::convert::TryFrom;
use thiserror::*;
use futures::future::try_select;
use futures::pin_mut;
use futures::future::Either;
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Direction {
    Up,
    Right,
    Left,
    Down,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum RelativeDirection {
    TurnLeft,
    TurnRight
}

impl TryFrom<Value> for RelativeDirection {
    type Error = SolutionError;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::TurnLeft),
            1 => Ok(Self::TurnRight),
            _ => Err(SolutionError::InvalidRotation)
        }
    }
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
    fn rotate(&mut self, relative_direction: RelativeDirection) {
        match relative_direction {
            RelativeDirection::TurnLeft => self.rotate_left(),
            RelativeDirection::TurnRight => self.rotate_right()
        }
    }

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
}

impl EmergencyHullPaintingRobot {
    async fn run(memory: Memory, field: &mut Field) -> Result<(), SolutionError> {
        let (mut to_robot_sender, mut to_robot_receiver) = channel(CHANNEL_BUFFER_SIZE);
        let (mut from_robot_sender, mut from_robot_receiver) = channel(CHANNEL_BUFFER_SIZE);
        let mut computer = Computer::load(memory);
        computer.set_input(Some(&mut to_robot_receiver));
        computer.set_output(Some(&mut from_robot_sender));  
        let computer_future = computer.run();
        let mut field_runner = FieldRunner::new(field);
        let field_runner_future = field_runner.run(to_robot_sender, from_robot_receiver);
        pin_mut!(computer_future, field_runner_future);
        match try_select(computer_future, field_runner_future).await {
            Ok(_) => Ok(()),
            Err(Either::Left((computer_err, _))) => Err(SolutionError::from(computer_err)),
            Err(Either::Right((solution_err, _))) => Err(solution_err)
        }?;
        Ok(())
    }
}

const CHANNEL_BUFFER_SIZE: usize = 2;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("an error occurred in the computer")]
    ComputerError(#[from] ComputerError),
    #[error("could not parse color")]
    ColorError,
    #[error("invalid rotation")]
    InvalidRotation,
    #[error("io error")]
    IoError
}

struct FieldRunner<'a> {
    field: &'a mut Field,
    position: Point,
    direction: Direction,
}

impl<'a> FieldRunner<'a> {
    fn new(field: &'a mut Field) -> Self {
        FieldRunner {
            field,
            position: Point::new(0, 0),
            direction: Direction::Up,
        }
    }

    fn do_move(&mut self, color: Color, relative_direction: RelativeDirection) {
        self.field.paint(self.position, color);
        self.direction.rotate(relative_direction);
        self.position.move_in_direction(self.direction);
    }

    async fn run(&mut self, mut camera: Sender<Value>, mut instructions: Receiver<Value>) -> Result<(), SolutionError> {
        loop {
            let currently_visible = self.field.view_color(self.position);
            camera.send(currently_visible.into()).await.map_err(|_| SolutionError::IoError)?;

            let color = Color::try_from(instructions.next().await.ok_or(SolutionError::IoError)?)?;
            let direction_to_move = RelativeDirection::try_from(instructions.next().await.ok_or(SolutionError::IoError)?)?;
            self.do_move(color, direction_to_move);
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

    fn view_color(&self, p: Point) -> Color {
        *self.colors.get(&p).unwrap_or(&Color::Black)
    }

    fn unique_tiles_painted(&self) -> usize {
        self.colors.len()
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min_x = self.colors.keys().map(|p| p.x).min().unwrap();
        let min_y = self.colors.keys().map(|p| p.y).min().unwrap();
        let max_x = self.colors.keys().map(|p| p.x).max().unwrap();
        let max_y = self.colors.keys().map(|p| p.y).max().unwrap();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                write!(f, "{}", match self.view_color(Point::new(x, y)) {
                    Color::Black => " ",
                    Color::White => "█"
                })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub use crate::intcode::parse_program as parse_input;

pub async fn part_1(memory: Memory) -> Result<usize, SolutionError> {
    let mut field = Field::new();
    EmergencyHullPaintingRobot::run(memory, &mut field).await?;
    Ok(field.unique_tiles_painted())
}

pub async fn part_2(memory: Memory) -> Result<Field, SolutionError> {
    let mut field = Field::new();
    field.paint(Point::new(0, 0), Color::White);
    EmergencyHullPaintingRobot::run(memory, &mut field).await?;
    Ok(field)
}

