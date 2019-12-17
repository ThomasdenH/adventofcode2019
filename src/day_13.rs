use futures::future::try_select;
use std::convert::{TryFrom, TryInto};
use crate::intcode::{Value, Computer, Memory, ComputerError, io};
use thiserror::*;
use futures::prelude::*;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::select;
use std::collections::HashMap;
use futures::stream::Stream;
use std::pin::Pin;
use futures::task::{Context, Poll};
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use futures::pin_mut;
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct ScreenPosition {
    x: usize,
    y: usize
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum TileId {
    Empty,
    Wall,
    Block,
    HorizontalPaddle,
    Ball
}

impl TryFrom<(Value, Value)> for ScreenPosition {
    type Error = SolutionError;
    fn try_from((x, y): (Value, Value)) -> Result<Self, Self::Error> {
        Ok(ScreenPosition {
            x: usize::try_from(x).map_err(|_| SolutionError::InvalidScreenPosition)?,
            y: usize::try_from(y).map_err(|_| SolutionError::InvalidScreenPosition)?
        })
    }
}

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("the screen position is invalid")]
    InvalidScreenPosition,
    #[error("unknown tile id")]
    UnknownTileId,
    #[error("an computer error occurred")]
    ComputerError(#[from] ComputerError),
    #[error("a protocol error occurred")]
    ProtocolError,
    #[error("could not find the tile id")]
    CouldNotFindTileId
}

impl TryFrom<Value> for TileId {
    type Error = SolutionError;
    fn try_from(input: Value) -> Result<Self, Self::Error> {
        match input {
            0 => Ok(TileId::Empty),
            1 => Ok(TileId::Wall),
            2 => Ok(TileId::Block),
            3 => Ok(TileId::HorizontalPaddle),
            4 => Ok(TileId::Ball),
            _ => Err(SolutionError::UnknownTileId)
        }
    }
}

pub use crate::intcode::parse_program as parse_input;

#[derive(Default, Clone)]
struct Screen {
    tiles: HashMap<ScreenPosition, TileId>
}

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let max_x = self.tiles.keys().map(|sp| sp.x).max().unwrap();
        let max_y = self.tiles.keys().map(|sp| sp.y).max().unwrap();
        for y in 0..=max_y {
            for x in 0..=max_x {
                write!(f, "{}", match self.tiles.get(&&ScreenPosition::try_from((x as Value, y as Value)).unwrap()) {
                    Some(TileId::Ball) => 'B',
                    Some(TileId::Block) => 'X',
                    Some(TileId::Empty) => ' ',
                    Some(TileId::HorizontalPaddle) => '_',
                    Some(TileId::Wall) => 'W',
                    _ => ' '
                })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Screen {
    fn set_at(&mut self, pos: ScreenPosition, tile_id: TileId) {
        self.tiles.insert(pos, tile_id);
    }

    async fn read_instructions(&mut self, mut receiver: Receiver<Value>) -> Result<(), SolutionError> {
        loop {
            let x = receiver.next().await.ok_or(SolutionError::ProtocolError)?;
            let y = receiver.next().await.ok_or(SolutionError::ProtocolError)?;
            let position = ScreenPosition::try_from((x, y))?;
            let tile_id: TileId = receiver.next().await.ok_or(SolutionError::ProtocolError)?.try_into()?;
            self.set_at(position, tile_id)
        }
    }
    
    fn block_tile_count(&self) -> usize {
        self.tiles.values()
            .filter(|val| **val == TileId::Block)
            .count()
    }

    fn ball_position(&self) -> Result<ScreenPosition, SolutionError> {
        self.find_tile_id(TileId::Ball)
    }

    fn paddle_position(&self) -> Result<ScreenPosition, SolutionError> {
        self.find_tile_id(TileId::HorizontalPaddle)
    }

    fn find_tile_id(&self, tile_id: TileId) -> Result<ScreenPosition, SolutionError> {
        Ok(*self.tiles.iter()
            .find(|(_key, value)| **value == tile_id)
            .ok_or(SolutionError::CouldNotFindTileId)?
            .0)
    }
}

const CHANNEL_BUFFER_SIZE: usize = 1;

#[derive(Clone, Default)]
struct GameState {
    screen: Screen,
    score: u64,
    input: [Value; 3],
    input_index: usize
}

impl GameState {
    fn handle_input(&mut self) -> Result<(), SolutionError> {
        let [x, y, z] = self.input;
        if x == -1 && y == 0 {
            self.score = z.try_into().map_err(|_| SolutionError::ProtocolError)?;
        } else {                
            let position = ScreenPosition::try_from((x, y))?;
            let tile_id: TileId = z.try_into()?;
            self.screen.set_at(position, tile_id);
        }
        Ok(())
    }
    
    fn score(&self) -> u64 {
        self.score
    }
}

enum JoystickPosition {
    Neutral,
    Left,
    Right
}

impl From<JoystickPosition> for Value {
    fn from(pos: JoystickPosition) -> Value {
        match pos {
            JoystickPosition::Neutral => 0,
            JoystickPosition::Left => -1,
            JoystickPosition::Right => 1
        }
    }
}

#[async_trait]
impl io::Read for Arc<Mutex<GameState>> {
    async fn read(&mut self) -> Option<Value> {
        let screen = &self.lock().unwrap().screen;
        let paddle_pos = screen.paddle_position().unwrap().x;
        let ball_pos = screen.ball_position().unwrap().x;
        Some(if paddle_pos < ball_pos {
            JoystickPosition::Right
        } else if paddle_pos > ball_pos {
            JoystickPosition::Left
        } else {
            JoystickPosition::Neutral
        }.into())
    }
}

#[async_trait]
impl io::Write for Arc<Mutex<GameState>> {
    async fn write(&mut self, value: Value) {
        let mut state = self.lock().unwrap();
        let input_index = state.input_index;
        state.input[input_index] = value;
        state.input_index += 1;
        if state.input_index > 2 {
            state.input_index = 0;
            state.handle_input().unwrap();
        }
    }
}

pub async fn part_1(memory: Memory) -> Result<usize, SolutionError> {
    let mut computer = Computer::load(memory);
    let (mut output_sender, mut output_receiver) = channel(CHANNEL_BUFFER_SIZE);
    computer.set_output(Some(&mut output_sender));

    let mut screen = Screen::default();
    select!(
        computer_res = computer.run().fuse() => computer_res?,
        _ = screen.read_instructions(output_receiver).fuse() => unreachable!()
    );
    Ok(screen.block_tile_count())
}

pub async fn part_2(mut memory: Memory) -> Result<u64, SolutionError> {
    let mut game_state = Arc::new(Mutex::new(GameState::default()));
    let mut cloned_game_state = game_state.clone();
    *memory.get_mut(0) = 2;
    let mut computer = Computer::load(memory);
    computer.set_output(Some(&mut cloned_game_state));
    computer.set_input(Some(&mut game_state));
    computer.run().await?;
    let score = game_state.lock().unwrap().score();
    Ok(score)
}
