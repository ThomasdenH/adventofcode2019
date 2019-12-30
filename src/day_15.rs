use crate::intcode::{parse_program, Computer, ComputerError, Memory, Value};
use futures::channel::mpsc::{channel, SendError};
use futures::pin_mut;
use futures::prelude::*;
use futures::select;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::marker::Unpin;
use thiserror::*;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("a computer error occurred")]
    ComputerError(#[from] ComputerError),
    #[error("unknown status code: {0}")]
    UnknownStatusCode(Value),
    #[error("could not find a path to the oxygen system")]
    CouldNotFindSystem,
    #[error("could not send data")]
    SendError(#[from] SendError),
    #[error("unknown character: {0}")]
    UnknownCharacter(char)
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum MovementCommand {
    North,
    South,
    West,
    East,
}

impl MovementCommand {
    fn reversed(self) -> MovementCommand {
        match self {
            MovementCommand::North => MovementCommand::South,
            MovementCommand::South => MovementCommand::North,
            MovementCommand::West => MovementCommand::East,
            MovementCommand::East => MovementCommand::West,
        }
    }
}

impl From<MovementCommand> for Value {
    fn from(cmd: MovementCommand) -> Self {
        match cmd {
            MovementCommand::North => 1,
            MovementCommand::South => 2,
            MovementCommand::West => 3,
            MovementCommand::East => 4,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Status {
    HitWall,
    Moved,
    MovedAndOnOxygen,
}

impl TryFrom<Value> for Status {
    type Error = SolutionError;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Status::HitWall),
            1 => Ok(Status::Moved),
            2 => Ok(Status::MovedAndOnOxygen),
            s => Err(SolutionError::UnknownStatusCode(s)),
        }
    }
}

pub fn parse_input(input: &str) -> Result<Memory, SolutionError> {
    parse_program(input).map_err(SolutionError::from)
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn move_towards(&mut self, cmd: MovementCommand) {
        *self = self.moved_towards(cmd);
    }

    fn moved_towards(self, cmd: MovementCommand) -> Self {
        match cmd {
            MovementCommand::North => Point {
                x: self.x,
                y: self.y - 1,
            },
            MovementCommand::South => Point {
                x: self.x,
                y: self.y + 1,
            },
            MovementCommand::West => Point {
                x: self.x - 1,
                y: self.y,
            },
            MovementCommand::East => Point {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Tile {
    Wall,
    Empty,
    OxygenSystem,
}

struct System<Input, Output>
where
    Input: TryStream<Ok = Status, Error = SolutionError> + Unpin,
    Output: Sink<MovementCommand, Error = SolutionError> + Unpin,
{
    input: Input,
    output: Output,
    current_position: Point,
}

impl<Input, Output> System<Input, Output>
where
    Input: TryStream<Ok = Status, Error = SolutionError> + Unpin,
    Output: Sink<MovementCommand, Error = SolutionError> + Unpin,
{
    fn new(input: Input, output: Output) -> Self {
        System {
            input,
            output,
            current_position: Point::default(),
        }
    }

    async fn do_move(&mut self, cmd: MovementCommand) -> Result<Status, SolutionError> {
        self.output.send(cmd).await?;
        let result = self.input.try_next().await?.unwrap();
        if result != Status::HitWall {
            self.current_position.move_towards(cmd);
        }
        Ok(result)
    }

    async fn shortest_path_to_oxygen_system(&mut self) -> Result<usize, SolutionError> {
        use MovementCommand::*;
        let mut moves_to_explore: VecDeque<Vec<MovementCommand>> = VecDeque::new();
        for &cmd in &[East, West, South, North] {
            moves_to_explore.push_back(vec![cmd]);
        }
        while let Some(moves) = moves_to_explore.pop_front() {
            assert_eq!(self.current_position, Point::default());
            let mut last_performed = None;
            let mut last_status = Status::Moved;
            let mut already_visited = HashSet::new();
            // Perform moves and find the status
            for (i, &cmd) in moves.iter().enumerate() {
                last_status = self.do_move(cmd).await?;
                if last_status == Status::Moved || last_status == Status::MovedAndOnOxygen {
                    last_performed = Some(i);
                }
                if already_visited.contains(&self.current_position) {
                    last_status = Status::HitWall;
                    break;
                }
                if last_status != Status::Moved {
                    break;
                }
                already_visited.insert(self.current_position);
            }
            // Undo the made moves
            if let Some(last_performed) = last_performed {
                for cmd in moves[0..=last_performed].iter().rev() {
                    self.do_move(cmd.reversed()).await?;
                }
            }

            match last_status {
                // We hit a wall, discard this path
                Status::HitWall => {}
                // We found a path
                Status::MovedAndOnOxygen => return Ok(moves.len()),
                // We did not yet find a path, so append all moves and try again later
                Status::Moved => {
                    for &new_cmd in &[East, West, South, North] {
                        let mut new_moves = moves.clone();
                        new_moves.push(new_cmd);
                        moves_to_explore.push_back(new_moves);
                    }
                }
            }
            assert_eq!(self.current_position, Point::default());
        }
        Err(SolutionError::CouldNotFindSystem)
    }
}

struct Map {
    tiles: HashMap<Point, Tile>,
}

impl TryFrom<&str> for Map {
    type Error = SolutionError;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut tiles = HashMap::new();
        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                let p = Point { x: x as i64, y: y as i64 };
                match c {
                    '#' => {tiles.insert(p, Tile::Wall);},
                    '.' => {tiles.insert(p, Tile::Empty);},
                    'O' => {tiles.insert(p, Tile::OxygenSystem);},
                    ' ' => continue,
                    c => return Err(SolutionError::UnknownCharacter(c))
                }
            }
        }
        Ok(Map{ tiles})
    }
}

impl Map {
    async fn do_move<Input, Output>(
        output: &mut Output,
        input: &mut Input,
        cmd: MovementCommand
    ) -> Result<Status, SolutionError>
    where
        Input: TryStream<Ok = Status, Error = SolutionError> + Unpin,
        Output: Sink<MovementCommand, Error = SolutionError> + Unpin,
    {
        output.send(cmd).await?;
        let result = input.try_next().await?.unwrap();
        Ok(result)
    }

    fn position_of_oxygen(&self) -> Result<Point, SolutionError> {
        self
            .tiles
            .iter()
            .find(|(_key, value)| **value == Tile::OxygenSystem)
            .ok_or(SolutionError::CouldNotFindSystem)
            .map(|(key, _value)| *key)
    }

    async fn build_from<Input, Output>(
        mut input: Input,
        mut output: Output,
    ) -> Result<Self, SolutionError>
    where
        Input: TryStream<Ok = Status, Error = SolutionError> + Unpin,
        Output: Sink<MovementCommand, Error = SolutionError> + Unpin,
    {
        use MovementCommand::*;
        let mut tiles = HashMap::new();
        let mut stack = vec![vec![East], vec![West], vec![North], vec![South]];

        while let Some(moves) = stack.pop() {
            let mut new_paths_from_last_instruction = false;
            let mut current_position = Point::default();
            // Perform moves and find the status
            let mut reverse = Vec::new();
            for &cmd in moves.iter() {
                match Map::do_move(&mut output, &mut input, cmd).await? {
                    Status::HitWall => {
                        tiles.insert(current_position.moved_towards(cmd), Tile::Wall);
                        new_paths_from_last_instruction = false;
                    }
                    Status::Moved => {
                        reverse.push(cmd);
                        current_position.move_towards(cmd);
                        new_paths_from_last_instruction = if tiles.contains_key(&current_position) {
                            false
                        } else {
                            tiles.insert(current_position, Tile::Empty);
                            true
                        }
                    }
                    Status::MovedAndOnOxygen => {
                        reverse.push(cmd);
                        current_position.move_towards(cmd);
                        new_paths_from_last_instruction = if tiles.contains_key(&current_position) {
                            false
                        } else {
                            tiles.insert(current_position, Tile::OxygenSystem);
                            true
                        }
                    }
                }
            }
            // Undo the made moves
            for cmd in reverse.iter().rev() {
                Map::do_move(&mut output, &mut input, cmd.reversed()).await?;
            }

            if new_paths_from_last_instruction {
                for &next_cmd in &[East, West, North, South] {
                    let mut new_moves = moves.clone();
                    new_moves.push(next_cmd);
                    stack.push(new_moves);
                }
            }
        }

        Ok(Self { tiles })
    }

    fn flood_fill(&self) -> Result<u32, SolutionError> {
        let mut visited = HashSet::new();
        let mut to_visit = VecDeque::new();
        to_visit.push_back((self.position_of_oxygen()?, 0u32));
        let mut highest = 0;
        while let Some((point, mut counter)) = to_visit.pop_front() {
            debug_assert!([Tile::OxygenSystem, Tile::Empty].contains(self.tiles.get(&point).unwrap()));
            // Visit
            visited.insert(point);
            // Increase counter
            counter += 1;
            highest = counter;
            // Add neighbouring
            use MovementCommand::*;
            for &dir in &[East, West, South, North] {
                let new_point = point.moved_towards(dir);
                if Some(&Tile::Empty) == self.tiles.get(&new_point) && !visited.contains(&new_point)
                {
                    to_visit.push_back((new_point, counter));
                }
            }
        }
        Ok(highest - 1)
    }
}

const CHANNEL_SIZE: usize = 1;

pub async fn part_1(input: Memory) -> Result<usize, SolutionError> {
    let mut robot = Computer::load(input);

    let (command_sender, mut command_receiver) = channel(CHANNEL_SIZE);
    let (mut status_sender, status_receiver) = channel(CHANNEL_SIZE);

    robot.set_input(Some(&mut command_receiver));
    robot.set_output(Some(&mut status_sender));

    let status_receiver =
        status_receiver.map(|s: Value| -> Result<Status, SolutionError> { Status::try_from(s) });
    let command_sender = command_sender.with(
        async move |v: MovementCommand| -> Result<Value, SolutionError> { Ok(Value::from(v)) },
    );
    pin_mut!(command_sender);
    let mut system = System::new(status_receiver, command_sender);

    select!(
        system_res = system.shortest_path_to_oxygen_system().fuse() => system_res,
        _ = robot.run().fuse() => unreachable!()
    )
}

pub async fn part_2(input: Memory) -> Result<u32, SolutionError> {
    let mut robot = Computer::load(input);

    let (command_sender, mut command_receiver) = channel(CHANNEL_SIZE);
    let (mut status_sender, status_receiver) = channel(CHANNEL_SIZE);

    robot.set_input(Some(&mut command_receiver));
    robot.set_output(Some(&mut status_sender));

    let mut status_receiver =
        status_receiver.map(|s: Value| -> Result<Status, SolutionError> { Status::try_from(s) });
    let command_sender = command_sender.with(
        async move |v: MovementCommand| -> Result<Value, SolutionError> { Ok(Value::from(v)) },
    );
    pin_mut!(command_sender);

    let map = select!(
        map_res = Map::build_from(&mut status_receiver, &mut command_sender).fuse() => map_res,
        _ = robot.run().fuse() => unreachable!()
    )?;

    map.flood_fill()
}

#[test]
fn test_flood_fill() -> anyhow::Result<()> {
    let map = Map::try_from(
" ##   
#..## 
#.#..#
#.O.# 
 ###  ")?;
 assert_eq!(map.flood_fill()?, 4);
 Ok(())
}
