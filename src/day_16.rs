use std::iter;
use thiserror::*;

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("not a digit")]
    NotADigit,
}

pub fn parse_input(s: &str) -> Result<Vec<i64>, SolutionError> {
    s.trim()
        .chars()
        .map(|c| {
            c.to_digit(10)
                .map(|d| d as i64)
                .ok_or(SolutionError::NotADigit)
        })
        .collect()
}

fn base_pattern() -> impl Iterator<Item = i64> + 'static {
    [0, 1, 0, -1].iter().cloned().cycle()
}

fn pattern(index: usize) -> impl Iterator<Item = i64> + 'static {
    base_pattern()
        .flat_map(move |i| iter::repeat(i).take(index + 1))
        .skip(1)
}

fn phase<'a>(buffer: &'a [i64]) -> impl Iterator<Item = i64> + 'a {
    let len = buffer.len();
    (0..len)
        .map(move |index| {
            buffer
                .iter()
                .copied()
                .zip(pattern(index))
                .map(|(a, b)| a * b)
                .sum::<i64>()
                .abs()
                % 10
        })
}

fn fft(mut buffer: Vec<i64>, phases: usize) -> Vec<i64> {
    let mut temp = Vec::with_capacity(buffer.len());
    for _ in 0..phases {
        temp.clear();
        temp.extend(phase(&buffer));
        std::mem::swap(&mut temp, &mut buffer);
    }
    buffer
}

pub fn part_1(input: Vec<i64>) -> String {
    fft(input, 100)
        .iter()
        .take(8)
        .map(i64::to_string)
        .collect()
}

#[test]
fn test_phase() {
    let before = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let after = phase(&before).collect::<Vec<_>>();
    assert_eq!(
        &after,
        &[4, 8, 2, 2, 6, 1, 5, 8]
    )
}

#[test]
fn test_fft() {
    assert_eq!(
        &fft(vec![1, 2, 3, 4, 5, 6, 7, 8], 4),
        &[0, 1, 0, 2, 9, 4, 9, 8]
    );
}
