use aoc_types::{year_2019::day_8, DaySolution};
use std::fmt;
use std::iter;
use thiserror::*;

#[derive(Debug)]
pub struct SpaceImageFormat {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

struct SpaceImageFormatLayer<'a> {
    width: usize,
    data: &'a [u8],
}

impl SpaceImageFormat {
    pub fn new_unchecked(width: usize, height: usize, data: Vec<u8>) -> SpaceImageFormat {
        SpaceImageFormat {
            width,
            height,
            data,
        }
    }

    fn layers(&self) -> impl Iterator<Item = SpaceImageFormatLayer<'_>> {
        let layer_size = self.width * self.height;
        let width = self.width;
        iter::successors(Some((None, self.data.as_slice())), move |(_, data)| {
            if data.is_empty() {
                None
            } else {
                let (a, b) = data.split_at(layer_size);
                Some((Some(a), b))
            }
        })
        .skip(1)
        .map(move |(data, _)| SpaceImageFormatLayer {
            width,
            data: data.unwrap(),
        })
    }
}

impl<'a> SpaceImageFormatLayer<'a> {
    fn count_digits(&self, d: u8) -> usize {
        self.data.iter().filter(|e| **e == d).count()
    }

    fn color_at(&self, x: usize, y: usize) -> u8 {
        self.data[y * self.width + x]
    }
}

impl fmt::Display for SpaceImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let color =
                    self.layers()
                        .map(|layer| layer.color_at(x, y))
                        .fold(
                            2,
                            |acc, layer_color| if acc == 2 { layer_color } else { acc },
                        );
                write!(f, "{}", if color == 0 { '_' } else { 'X' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct Solution {}

#[derive(Debug, Error)]
pub enum SolutionError {
    #[error("invalid digit in input")]
    InvalidDigit,
}

impl<'a, 'b, 'c>
    DaySolution<'a, 'b, 'c, SpaceImageFormat, usize, &'c SpaceImageFormat, SolutionError>
    for Solution
{
    fn parse_input(data: &str) -> Result<SpaceImageFormat, SolutionError> {
        Ok(SpaceImageFormat::new_unchecked(
            day_8::WIDTH,
            day_8::HEIGHT,
            data.trim()
                .chars()
                .map(|i| {
                    i.to_digit(3)
                        .map(|d| d as u8)
                        .ok_or(SolutionError::InvalidDigit)
                })
                .collect::<Result<Vec<u8>, SolutionError>>()?,
        ))
    }

    fn part_1(image: &SpaceImageFormat) -> Result<usize, SolutionError> {
        let layer = image
            .layers()
            .min_by_key(|layer| layer.count_digits(0))
            .unwrap();
        Ok(layer.count_digits(1) * layer.count_digits(2))
    }

    fn part_2(i: &'c SpaceImageFormat) -> Result<&'c SpaceImageFormat, SolutionError> {
        Ok(i)
    }
}
