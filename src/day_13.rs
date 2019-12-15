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
            4 => Ok(TileId::HorizontalPaddle),
            _ => Err(SolutionError::UnknownTileId)
        }
    }
}

pub use crate::intcode::parse_program as parse_input;

#[derive(Default, Clone, Debug)]
struct Screen {
    tiles: HashMap<ScreenPosition, TileId>
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

struct GameState {
    screen: Arc<Mutex<Screen>>,
    score: u64
}

impl GameState {
    fn new(screen: Arc<Mutex<Screen>>) -> GameState {
        GameState {
            screen,
            score: 0
        }
    } 

    async fn run(&mut self, mut receiver: Receiver<Value>) -> Result<(), SolutionError> {
        loop {
            let x = receiver.next().await.ok_or(SolutionError::ProtocolError)?;
            let y = receiver.next().await.ok_or(SolutionError::ProtocolError)?;
            if x == -1 && y == 0 {
                self.score = receiver.next()
                    .await
                    .ok_or(SolutionError::ProtocolError)?
                    .try_into()
                    .map_err(|_| SolutionError::ProtocolError)?;
            } else {                
                let position = ScreenPosition::try_from((x, y))?;
                let tile_id: TileId = receiver.next().await.ok_or(SolutionError::ProtocolError)?.try_into()?;
                self.screen.lock().unwrap().set_at(position, tile_id)
            }
        }
    }

    fn score(&self) -> u64 {
        self.score
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
