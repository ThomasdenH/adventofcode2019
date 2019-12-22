use crate::intcode::{Computer, Value, ComputerError, Memory, parse_program};
use thiserror::*;
use std::convert::TryFrom;
use std::collections::{HashMap, VecDeque};
use std::iter;
use futures::prelude::*;
use futures::channel::mpsc::{channel, Sender, Receiver};
use std::marker::Unpin;
use futures::pin_mut;
use futures::future::{BoxFuture};

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("a computer error occurred")]
    ComputerError(#[from] ComputerError),
    #[error("unknown status code: {0}")]
    UnknownStatusCode(Value),
    #[error("could not find a path to the oxygen system")]
    CouldNotFindSystem
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum MovementCommand {
    North,
    South,
    West,
    East
}

impl MovementCommand {
    fn reversed(self) -> MovementCommand {
        match self {
            MovementCommand::North => MovementCommand::South,
            MovementCommand::South => MovementCommand::North,
            MovementCommand::West => MovementCommand::East,
            MovementCommand::East => MovementCommand::West
        }
    }
}

impl From<MovementCommand> for Value {
    fn from(cmd: MovementCommand) -> Self {
        match cmd {
            MovementCommand::North => 1,
            MovementCommand::South => 2,
            MovementCommand::West => 3,
            MovementCommand::East => 4
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Status {
    HitWall,
    Moved,
    MovedAndOnOxygen
}

impl TryFrom<Value> for Status {
    type Error = SolutionError;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Status::HitWall),
            1 => Ok(Status::Moved),
            2 => Ok(Status::MovedAndOnOxygen),
            s => Err(SolutionError::UnknownStatusCode(s))
        }
    }
}

pub fn parse_input(input: &str) -> Result<Memory, SolutionError> {
    parse_program(input).map_err(SolutionError::from)
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Point {
    x: i64,
    y: i64
}

impl Point {
    fn move_towards(&mut self, cmd: MovementCommand) {
        *self = match cmd {
            MovementCommand::North => Point { x: self.x, y: self.y - 1 },
            MovementCommand::South => Point { x: self.x, y: self.y + 1 },
            MovementCommand::West => Point { x: self.x - 1, y: self.y },
            MovementCommand::East => Point { x: self.x + 1, y: self.y }
        }
    }

    fn undo_move_towards(&mut self, cmd: MovementCommand) {
        self.move_towards(match cmd {
            MovementCommand::North => MovementCommand::South,
            MovementCommand::South => MovementCommand::North,
            MovementCommand::West => MovementCommand::East,
            MovementCommand::East => MovementCommand::West
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Tile {
    Wall,
    Empty,
    OxygenSystem
}

struct System<Input, Output>
    where Input: TryStream<Ok = Status, Error = SolutionError> + Unpin,
    Output: Sink<MovementCommand> + Unpin {
    map: HashMap<Point, Tile>,
    current_position: Point,
    input: Input,
    output: Output
}

impl<Input, Output> System<Input, Output>
    where Input: TryStream<Ok = Status, Error = SolutionError> + Unpin,
    Output: Sink<MovementCommand> + Unpin {
    fn new(input: Input, output: Output) -> Self {
        System {
            map: HashMap::new(),
            current_position: Point { x: 0, y: 0 },
            input,
            output
        }
    }

    /// Tries the moves out and then undoes them again.
    async fn try_moves(&mut self, moves: &[MovementCommand]) -> BoxFuture<'_, Result<Status, SolutionError>> {
        if let Some((first, remainder)) = moves.split_first() {
            self.output.send(*first).await;
            let result = match self.input.try_next().await.unwrap().unwrap() {
                Status::Moved => self.try_moves(remainder).await.boxed(),
                other => async move { Ok(other) }.boxed()
            };
            self.output.send(first.reversed()).await;
            result
        } else {
            // No final state
            async move { Ok(Status::Moved) }.boxed()
        }
    }
    
    async fn shortest_path_to_oxygen_system(&mut self) -> Result<usize, SolutionError> {
        use MovementCommand::*;
        let moves_to_explore: VecDeque<Vec<MovementCommand>> = VecDeque::new();
        for &cmd in &[East, West, South, North] {
            moves_to_explore.push_back(vec![cmd]);
        }
        while let Some(moves) = moves_to_explore.pop_front() {
            match self.try_moves(&moves).await? {
                Status::HitWall => continue,
                // We have found a path
                Status::MovedAndOnOxygen => return Ok(moves.len()),
                Status::Moved => {
                    for &next_move in &[
                        East,
                        South,
                        West,
                        North
                    ] {
                        let mut next_path = moves.clone();
                        next_path.push(next_move);
                        moves_to_explore.push_back(next_path);
                    }
                }
            }
        }
        Err(SolutionError::CouldNotFindSystem)
    }
}

const CHANNEL_SIZE: usize = 1;

pub fn part_1(input: Memory) -> Result<u64, SolutionError> {
    let mut robot = Computer::load(input);

    let (mut command_sender, mut command_receiver) = channel(CHANNEL_SIZE);
    let (mut status_sender, mut status_receiver) = channel(CHANNEL_SIZE);
    
    robot.set_input(Some(&mut command_receiver));
    robot.set_output(Some(&mut status_sender));

    let status_receiver = status_receiver.map(|s: Value| -> Result<Status, SolutionError> { Status::try_from(s) });
    let command_sender = command_sender.with(async move |v: MovementCommand| { Ok(Value::from(v)) });
    pin_mut!(command_sender);
    let mut system = System::new(
        status_receiver,
        command_sender
    );
    unimplemented!();
}
