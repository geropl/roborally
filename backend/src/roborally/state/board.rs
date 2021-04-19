#![allow(clippy::trivially_copy_pass_by_ref)]

use derive_builder::Builder;

use super::{ ParserError, StateError };

#[derive(Debug)]
pub struct BoardConfig {
    factory_floor: String,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            factory_floor: String::from("test-full-1"),
        }
    }
}

/**
 * Spans a rectangular board constisting of tiles.
 * Not every tile is playable, [0, 0] is the North-West/upper-left corner
 */
#[derive(Debug, Default)]
pub struct Board {
    pub tiles: Vec<Tile>,
    pub size_x: i32,
    pub size_y: i32,
}

impl Board {
    pub fn create_from(config: &BoardConfig) -> Result<Board, ParserError> {
        super::load_board_by_name(&config.factory_floor)
    }

    #[cfg(test)]
    pub fn load_board_by_name(name: &str) -> Result<Board, ParserError> {
        super::load_board_by_name(name)
    }

    pub fn get_neighbor_in(&self, pos: &Position, direction: EDirection) -> Result<EConnection, StateError> {
        if self.is_off_board(pos) {
            return Err(StateError::PositionOffBoard{ position: *pos });
        }

        let new_pos = match direction {
            EDirection::NORTH => pos.set_y(pos.y - 1),
            EDirection::SOUTH => pos.set_y(pos.y + 1),
            EDirection::WEST => pos.set_x(pos.x - 1),
            EDirection::EAST => pos.set_x(pos.x + 1),
        };

        let old_tile = &self.tiles[self.tile_index(pos)];
        let new_tile = if self.is_off_board(&new_pos) {
            None
        } else {
            Some(&self.tiles[self.tile_index(&new_pos)])
        };
        
        if old_tile.walls.contains(&direction)
            // Can only check both sides if the other exists at all...
            || (new_tile.is_some() && new_tile.unwrap().walls.contains(&direction.opposite())) {
            return Ok(EConnection::Walled);
        } else if self.is_off_board(&new_pos)
            || (new_tile.is_some() && new_tile.unwrap().ttype == ETileType::NoTile) {
            return Ok(EConnection::OffPlatform(new_pos));
        }
        Ok(EConnection::Free(new_pos))
    }

    pub fn get_tile_type_at(&self, pos: &Position) -> Result<ETileType, StateError> {
        let index = self.tile_index(pos);
        let tile = self.tiles.get(index)
            .ok_or_else(|| StateError::PositionOffBoard{ position: *pos })?;
        Ok(tile.ttype)
    }

    pub fn get_start_position_or_fail(&self, start_position_id: StartPositionID) -> Result<Position, StateError> {
        for tile in &self.tiles {
            if let Some(id) = tile.start_position_id {
                if id == start_position_id {
                    return Ok(tile.position)
                }
            }
        }
        Err(StateError::StartPositionNotFoundID{ start_position_id })
    }

    fn is_off_board(&self, position: &Position) -> bool {
        position.x < 0 || position.x >= self.size_x
            || position.y < 0 || position.y >= self.size_y
    }

    fn tile_index(&self, pos: &Position) -> usize {
        (pos.x + (pos.y * self.size_x)) as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EConnection {
    Free(Position),
    Walled,
    OffPlatform(Position),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

impl EDirection {
    pub const DIRECTIONS: [EDirection;  4] = [EDirection::NORTH, EDirection::EAST, EDirection::SOUTH, EDirection::WEST];

    pub fn turn_left(self) -> EDirection {
        self.turn(-1)
    }

    pub fn turn_right(self) -> EDirection {
        self.turn(1)
    }

    pub fn turn_around(self) -> EDirection {
        self.turn(2)
    }

    pub fn opposite(self) -> EDirection {
        self.turn_around()
    }

    fn turn(self, offset: i8) -> EDirection {
        let index = EDirection::DIRECTIONS.iter().position(|d| *d == self).unwrap();
        let max = EDirection::DIRECTIONS.len() as i8;
        let new_index = (index as i8 + offset + max) % max;
        EDirection::DIRECTIONS[new_index as usize]
    }

    pub fn rotate(&self, rotation_dir: &ERotationDirection) -> EDirection {
        match rotation_dir {
            ERotationDirection::Left => self.turn_left(),
            ERotationDirection::Right => self.turn_right(),
        }
    }

    /// try_rotate_towards tries to find a ERotationDirection (90° turn) to get from self (inbound direction) towards target direction
    pub fn try_rotate_towards(&self, target: &EDirection) -> Option<ERotationDirection> {
        if self.turn_around() == *target || self == target {
            return None
        }
        if self.turn_left() == *target {
            return Some(ERotationDirection::Left)
        }
        Some(ERotationDirection::Right)
    }
}

impl Default for EDirection {
    fn default() -> EDirection {
        EDirection::NORTH
    }
}

pub type StartPositionID = u32;

#[derive(Debug, Builder)]
pub struct Tile {
    pub position: Position,
    pub ttype: ETileType,
    pub walls: Vec<EDirection>,
    pub start_position_id: Option<StartPositionID>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ETileType {
    Regular,
    NoTile,
    Conveyor2 {
        out: EDirection,
        input: EDirection,
        speed: bool,
    },
    Conveyor3 {
        out: EDirection,
        inputs: [EDirection; 2],
        speed: bool,
    },
    Rotator {
        dir: ERotationDirection,
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ERotationDirection {
    Left,
    Right,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position {
            x,
            y,
        }
    }

    pub fn set_x(&self, x: i32) -> Position {
        Position {
            x,
            y: self.y,
        }
    }

    pub fn set_y(&self, y: i32) -> Position {
        Position {
            x: self.x,
            y,
        }
    }
}