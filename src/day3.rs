use auto_enums::auto_enum;
use nom::character::complete::anychar;
use nom::character::complete::digit1;
use nom::character::complete::line_ending;
use nom::combinator::map_res;
use nom::multi::separated_list;
use nom::sequence::separated_pair;
use nom::sequence::tuple;
use std::convert::TryFrom;
use std::iter;
use std::str::FromStr;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum BaseDirection {
    Down,
    Right,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }

    fn distance_from_center(self) -> u32 {
        self.distance(Point::new(0, 0))
    }

    fn move_by(&mut self, direction: Direction, amount: u32) {
        let amount = i64::from(amount);
        match direction {
            Direction::Down => self.y += amount,
            Direction::Up => self.y -= amount,
            Direction::Right => self.x += amount,
            Direction::Left => self.x -= amount,
        }
    }

    fn distance(self, other: Self) -> u32 {
        (self.x - other.x).abs() as u32 + (self.y - other.y).abs() as u32
    }
}

/// A line part, starting at a point with a certain length. The length doesn't
/// include the starting point, so a length of 1 ends at a point next to the
/// starting point.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct LinePart {
    start: Point,
    direction: BaseDirection,
    length: u32,
}

impl LinePart {
    fn new(start: Point, direction: Direction, length: u32) -> LinePart {
        match direction {
            Direction::Down => LinePart {
                start,
                direction: BaseDirection::Down,
                length,
            },
            Direction::Right => LinePart {
                start,
                direction: BaseDirection::Right,
                length,
            },
            Direction::Up => LinePart {
                start: Point::new(start.x, start.y - i64::from(length)),
                direction: BaseDirection::Down,
                length,
            },
            Direction::Left => LinePart {
                start: Point::new(start.x - i64::from(length), start.y),
                direction: BaseDirection::Right,
                length,
            },
        }
    }

    fn high_y(self) -> i64 {
        match self.direction {
            BaseDirection::Down => self.start.y + i64::from(self.length),
            BaseDirection::Right => self.start.y,
        }
    }

    fn high_x(self) -> i64 {
        match self.direction {
            BaseDirection::Down => self.start.x,
            BaseDirection::Right => self.start.x + i64::from(self.length),
        }
    }

    #[auto_enum(Iterator)]
    fn intersections(self, other: LinePart) -> impl Iterator<Item = Point> + 'static {
        #[nested]
        match (self.direction, other.direction) {
            (BaseDirection::Down, BaseDirection::Down) => {
                #[nested]
                let r = if self.start.x == other.start.x {
                    let intersection_y_start = self.start.y.max(other.start.y);
                    let intersection_y_end = self.high_y().min(other.high_y());
                    #[nested]
                    let b = if intersection_y_start <= intersection_y_end {
                        (intersection_y_start..=intersection_y_end)
                            .map(move |y| Point::new(self.start.x, y))
                    } else {
                        iter::empty()
                    };
                    b
                } else {
                    iter::empty()
                };
                r
            }
            (BaseDirection::Right, BaseDirection::Right) => {
                #[nested]
                let b = if self.start.y == other.start.y {
                    let intersection_x_start = self.start.x.max(other.start.x);
                    let intersection_x_end = self.high_x().min(other.high_x());
                    #[nested]
                    let a = if intersection_x_start <= intersection_x_end {
                        (intersection_x_start..=intersection_x_end)
                            .map(move |x| Point::new(x, self.start.y))
                    } else {
                        iter::empty()
                    };
                    a
                } else {
                    iter::empty()
                };
                b
            }
            _ => {
                let (downwards, rightwards) = if self.direction == BaseDirection::Down {
                    (self, other)
                } else {
                    (other, self)
                };
                #[nested]
                let b = if downwards.start.x >= rightwards.start.x
                    && downwards.start.x <= rightwards.high_x()
                    && rightwards.start.y >= downwards.start.y
                    && rightwards.start.y <= downwards.high_y()
                {
                    iter::once(Point::new(downwards.start.x, rightwards.start.y))
                } else {
                    iter::empty()
                };
                b
            }
        }
    }
}

#[test]
fn test_intersection() {
    let line_part_1 = LinePart::new(Point::new(0, 0), Direction::Right, 2);
    let line_part_2 = LinePart::new(Point::new(1, -1), Direction::Down, 2);
    assert_eq!(
        line_part_1
            .intersections(line_part_2)
            .collect::<Vec<Point>>(),
        &[Point::new(1, 0)]
    );
}

impl TryFrom<char> for Direction {
    type Error = ();
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'D' => Ok(Direction::Down),
            'U' => Ok(Direction::Up),
            'L' => Ok(Direction::Left),
            'R' => Ok(Direction::Right),
            _ => Err(()),
        }
    }
}

type Instruction = (Direction, u32);
type Instructions = Vec<Instruction>;

fn parse_instruction<'a>(input: &'a str) -> nom::IResult<&'a str, Instruction> {
    tuple((
        map_res(anychar, Direction::try_from),
        map_res(digit1, u32::from_str),
    ))(input)
}

#[test]
fn test_parse_instruction() {
    assert_eq!(parse_instruction("R43"), Ok(("", (Direction::Right, 43))));
}

fn parse_line(input: &str) -> nom::IResult<&str, Instructions> {
    separated_list(nom::character::complete::char(','), parse_instruction)(input)
}

pub fn parse_input(input: &str) -> (Instructions, Instructions) {
    separated_pair(parse_line, line_ending, parse_line)(input.trim())
        .expect("could not parse input")
        .1
}

#[test]
fn test_parse_input() {
    use std::io::Read;
    let mut s = String::new();
    std::fs::File::open("./input/day3")
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    parse_input(&s);
}

struct LinePartIter<I: Iterator<Item = Instruction>> {
    point: Point,
    instructions: I,
}

impl<I: Iterator<Item = Instruction>> LinePartIter<I> {
    fn new(instructions: I) -> Self {
        LinePartIter {
            point: Point::new(0, 0),
            instructions,
        }
    }
}

impl<I: Iterator<Item = Instruction>> Iterator for LinePartIter<I> {
    type Item = LinePart;
    fn next(&mut self) -> Option<Self::Item> {
        self.instructions.next().map(|(dir, len)| {
            let line_part = LinePart::new(self.point, dir, len);
            self.point.move_by(dir, len);
            line_part
        })
    }
}

struct LinePartLengthIter<I: Iterator<Item = LinePart>> {
    len: u32,
    parts: I,
}

impl<I: Iterator<Item = LinePart>> From<I> for LinePartLengthIter<I> {
    fn from(parts: I) -> Self {
        LinePartLengthIter { len: 0, parts }
    }
}

impl<I: Iterator<Item = LinePart>> Iterator for LinePartLengthIter<I> {
    type Item = (u32, LinePart);
    fn next(&mut self) -> Option<(u32, LinePart)> {
        self.parts.next().map(|p| {
            let len = self.len;
            self.len += p.length;
            (len, p)
        })
    }
}

pub fn part_1((wire_1, wire_2): (Instructions, Instructions)) -> u32 {
    LinePartIter::new(wire_1.into_iter())
        .flat_map(|line_part_1| {
            LinePartIter::new(wire_2.clone().into_iter())
                .map(move |line_part_2| (line_part_1, line_part_2))
        })
        .flat_map(|(part_1, part_2)| part_1.intersections(part_2))
        .filter(|p| *p != Point::new(0, 0))
        .map(Point::distance_from_center)
        .min()
        .expect("no intersection points")
}

fn points_line_parts_and_length(
    i: impl Iterator<Item = Instruction>,
) -> impl Iterator<Item = (Point, LinePart, u32)> {
    let mut point = Point::new(0, 0);
    let mut length = 0;
    i.map(move |(dir, len)| {
        let this_point = point;
        let this_len = length;
        point.move_by(dir, len);
        length += len;
        (this_point, LinePart::new(this_point, dir, len), this_len)
    })
}

pub fn part_2((wire_1, wire_2): (Instructions, Instructions)) -> u32 {
    // Iterate over all combinations of parts
    points_line_parts_and_length(wire_1.into_iter())
        .flat_map(|plpal_1| {
            points_line_parts_and_length(wire_2.clone().into_iter())
                .map(move |plpal_2| (plpal_1, plpal_2))
        })
        .flat_map(|((p1, lp1, tl1), (p2, lp2, tl2))| {
            lp1.intersections(lp2).map(move |i| {
                (
                    // The total length
                    tl1 + tl2 + p1.distance(i) + p2.distance(i),
                    // The intersection
                    i,
                )
            })
        })
        .filter(|(_c, p)| *p != Point::new(0, 0))
        .map(|(c, _)| c)
        .min()
        .expect("no intersection points")
}

#[test]
fn test_part_1_examples() {
    let input = "R8,U5,L5,D3\nU7,R6,D4,L4";
    assert_eq!(part_1(parse_input(input)), 6);
    let input = "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83";
    assert_eq!(part_1(parse_input(input)), 159);
    let input = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7";
    assert_eq!(part_1(parse_input(input)), 135);
}

#[test]
fn test_part_2_examples() {
    let input = "R8,U5,L5,D3\nU7,R6,D4,L4";
    assert_eq!(part_2(parse_input(input)), 30);
    let input = "R75,D30,R83,U83,L12,D49,R71,U7,L72\nU62,R66,U55,R34,D71,R55,D58,R83";
    assert_eq!(part_2(parse_input(input)), 610);
    let input = "R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51\nU98,R91,D20,R16,D67,R40,U7,R15,U6,R7";
    assert_eq!(part_2(parse_input(input)), 410);
}
