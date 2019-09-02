#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

pub static ALL_DIRECTIONS: [Direction; 8] = [
    Direction::North,
    Direction::NorthEast,
    Direction::East,
    Direction::SouthEast,
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
];

pub static ORTHOGONAL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

pub static DIAGONAL_DIRECTIONS: [Direction; 4] = [
    Direction::NorthEast,
    Direction::SouthEast,
    Direction::SouthWest,
    Direction::NorthWest,
];

impl Direction {
    pub fn reverse(self) -> Direction {
        Direction::from_index(self.to_index()+4)
    }

    pub fn rotate_clockwise(self) -> Direction {
        Direction::from_index(self.to_index()+1)
    }

    pub fn rotate_counterclockwise(self) -> Direction {
        Direction::from_index(self.to_index()-1)
    }

    pub fn to_index(self) -> i32 {
        match self {
            Direction::North => 0,
            Direction::NorthEast => 1,
            Direction::East => 2,
            Direction::SouthEast => 3,
            Direction::South => 4,
            Direction::SouthWest => 5,
            Direction::West => 6,
            Direction::NorthWest => 7,
        }
    }

    pub fn from_index(idx: i32) -> Self {
        match idx & 7 {
            0 => Direction::North,
            1 => Direction::NorthEast,
            2 => Direction::East,
            3 => Direction::SouthEast,
            4 => Direction::South,
            5 => Direction::SouthWest,
            6 => Direction::West,
            7 => Direction::NorthWest,
            _ => { unreachable!(); }
        }
    }

    pub fn is_orthogonal(&self) -> bool {
        self == &Direction::North
            || self == &Direction::East
            || self == &Direction::South
            || self == &Direction::West
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn step(self, d: Direction) -> Position {
        match d {
            Direction::North => Position { x: self.x, y: self.y - 1 },
            Direction::NorthEast => Position { x: self.x + 1, y: self.y - 1 },
            Direction::East => Position { x: self.x + 1, y: self.y },
            Direction::SouthEast => Position { x: self.x + 1, y: self.y + 1 },
            Direction::South => Position { x: self.x, y: self.y + 1 },
            Direction::SouthWest => Position { x: self.x - 1, y: self.y + 1 },
            Direction::West => Position { x: self.x - 1, y: self.y },
            Direction::NorthWest => Position { x: self.x - 1, y: self.y - 1 },
        }
    }

    pub fn adjacent_to(self, o: Position) -> bool {
        self != o && (self.x - o.x).abs() <= 1 && (self.y - o.y).abs() <= 1
    }

    pub fn chebyshev_distance(self, o: Position) -> i32 {
        i32::max((self.x - o.x).abs(), (self.y - o.y).abs())
    }
}
