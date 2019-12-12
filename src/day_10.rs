use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Sub};
use thiserror::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Rational {
    p: i64,
    q: u64,
}

impl Rational {
    fn is_negative(&self) -> bool {
        self.p < 0
    }

    fn new(p: i64, q: i64) -> Self {
        if q < 0 {
            Self::new(-p, -q)
        } else {
            let gcd = gcd(p, q);
            let q = (q / gcd) as u64;
            let p = p / gcd;
            Rational { p, q }
        }
    }
}

impl Add<i64> for Rational {
    type Output = Self;
    fn add(self, other: i64) -> Self {
        Rational::new(self.p + self.q as i64 * other, self.q as i64)
    }
}

impl Sub<Rational> for i64 {
    type Output = Rational;
    fn sub(self, other: Rational) -> Rational {
        Rational::new(self * other.q as i64 - other.p, other.q as i64)
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.p * other.q as i64).cmp(&(other.p * self.q as i64)))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.p * other.q as i64).cmp(&(other.p * self.q as i64))
    }
}

/// This type uniquely defines angles. Multiples of the same vector yield the same angle.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct AngleHash {
    x: i64,
    y: i64,
}

impl From<(i64, i64)> for AngleHash {
    fn from((x, y): (i64, i64)) -> Self {
        let gcd = gcd(x, y);
        AngleHash {
            x: x / gcd,
            y: y / gcd,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Vector {
    x: i64,
    y: i64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn vector_to(self, other: Point) -> Vector {
        Vector {
            x: other.x as i64 - self.x as i64,
            y: other.y as i64 - self.y as i64,
        }
    }

    fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

#[test]
fn test_pseudo_angle() {
    assert_eq!(Vector { x: 0, y: -1 }.pseudo_angle(), Rational::new(0, 1));
    assert_eq!(Vector { x: 1, y: 0 }.pseudo_angle(), Rational::new(1, 1));
    assert_eq!(Vector { x: 0, y: 1 }.pseudo_angle(), Rational::new(2, 1));
    assert_eq!(Vector { x: -1, y: 0 }.pseudo_angle(), Rational::new(3, 1));
}

impl Vector {
    /// A number between [0..4] that increases monotonely with the clockwise angle to the y axis.
    fn pseudo_angle(&self) -> Rational {
        let p = Rational::new(self.y, self.x.abs() + self.y.abs());
        if self.x >= 0 {
            p + 1
        } else {
            3 - p
        }
    }

    /// A number that increases monotonely with the radius of the vector
    fn pseudo_radius(&self) -> i64 {
        self.x * self.x + self.y * self.y
    }

    fn angle_hash(&self) -> AngleHash {
        AngleHash::from((self.x, self.y))
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if a < 0 {
        gcd(-a, b)
    } else if b < 0 {
        gcd(a, -b)
    } else if b > a {
        gcd(b, a)
    } else if b == 0 {
        a
    } else {
        // a >= b > 0
        gcd(b, a % b)
    }
}

#[test]
fn test_gcd() {
    assert_eq!(gcd(0, 4), 4);
    assert_eq!(gcd(2, 6), 2);
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum AstroidFieldTile {
    Empty,
    Astroid,
}

#[derive(Debug, Clone)]
pub struct AstroidField {
    width: usize,
    tiles: Vec<AstroidFieldTile>,
}

impl AstroidField {
    fn new(width: usize, height: usize) -> AstroidField {
        AstroidField {
            width,
            tiles: (0..width * height)
                .map(|_| AstroidFieldTile::Empty)
                .collect(),
        }
    }

    fn set(&mut self, p: Point, val: AstroidFieldTile) {
        self.tiles[p.x + p.y * self.width] = val;
    }

    fn get(&self, p: Point) -> AstroidFieldTile {
        self.tiles[p.x + p.y * self.width]
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.tiles.len() / self.width
    }

    fn coordinates(&self) -> impl Iterator<Item = Point> + 'static {
        let width = self.width();
        let height = self.height();
        (0..width).flat_map(move |x| (0..height).map(move |y| Point { x, y }))
    }

    fn astroids<'a>(&'a self) -> impl Iterator<Item = Point> + 'a {
        self.coordinates()
            .filter(move |p| self.get(*p) == AstroidFieldTile::Astroid)
    }
}

#[derive(Error, Debug)]
pub enum SolutionError {
    #[error("could not parse field. Unknown character: {0}")]
    UnknownFieldCharacter(char),
}

pub fn parse_input(s: &str) -> Result<AstroidField, SolutionError> {
    let width = s.lines().next().unwrap().len();
    let tiles: Vec<AstroidFieldTile> = s
        .trim()
        .lines()
        .flat_map(|line| {
            line.chars().map(|c| match c {
                '.' => Ok(AstroidFieldTile::Empty),
                '#' => Ok(AstroidFieldTile::Astroid),
                c => Err(SolutionError::UnknownFieldCharacter(c)),
            })
        })
        .collect::<Result<Vec<AstroidFieldTile>, SolutionError>>()?;
    Ok(AstroidField { width, tiles })
}

pub fn part_1(field: &AstroidField) -> usize {
    best_location(field).1
}

fn best_location(field: &AstroidField) -> (Point, usize) {
    field
        .astroids()
        .map(|p1| {
            (
                p1,
                field
                    .astroids()
                    .filter(|&p2| p1 != p2)
                    .map(|p2| p1.vector_to(p2).angle_hash())
                    .collect::<HashSet<_>>()
                    .len(),
            )
        })
        .max_by_key(|(_, astroids_visible)| *astroids_visible)
        .unwrap()
}

pub fn part_2(field: &AstroidField) -> usize {
    let location = best_location(field).0;
    // Astroids, grouped by angle
    let mut grouped: HashMap<Rational, Vec<(Vector, Point)>> = HashMap::new();
    for asteroid in field.astroids().filter(|&p2| location != p2) {
        let vector = location.vector_to(asteroid);
        grouped
            .entry(vector.pseudo_angle())
            .or_insert_with(|| Vec::new())
            .push((vector, asteroid));
    }
    let mut sorted: Vec<(Rational, Vec<(Vector, Point)>)> = grouped.into_iter().collect();
    sorted.sort_unstable_by_key(|i| i.0);
    for (_, vec) in &mut sorted {
        vec.sort_unstable_by_key(|(p, _)| -p.pseudo_radius());
    }

    let mut i = 0;
    loop {
        for (_, vectors) in sorted.iter_mut() {
            if let Some((_, point)) = vectors.pop() {
                i += 1;
                if i == 200 {
                    return (100 * point.x + point.y) as usize;
                }
            }
        }
    }
}

#[test]
fn test_part_1() -> anyhow::Result<()> {
    assert_eq!(
        best_location(&parse_input(
            ".#..#
.....
#####
....#
...##
"
        )?),
        (Point::new(3, 4), 8)
    );
    assert_eq!(
        best_location(&parse_input(
            "......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####
"
        )?),
        (Point::new(5, 8), 33)
    );
    assert_eq!(
        best_location(&parse_input(
            "#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###.            
"
        )?),
        (Point::new(1, 2), 35)
    );
    assert_eq!(
        best_location(&parse_input(
            ".#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..                  
"
        )?),
        (Point::new(6, 3), 41)
    );
    assert_eq!(
        best_location(&parse_input(
            ".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##                            
"
        )?),
        (Point::new(11, 13), 210)
    );
    Ok(())
}
