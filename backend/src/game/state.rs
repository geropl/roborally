#![allow(clippy::trivially_copy_pass_by_ref)]

use derive_builder::Builder;

use std::clone::Clone;
use std::sync::Arc;
use std::default::Default;

#[derive(Debug, Default, Clone)]
pub struct State {
    pub board: Arc<Board>,
    pub players: Vec<Player>,
}

impl State {
    pub fn new(board: Board, players: Vec<Player>) -> State {
        State {
            board: Arc::new(board),
            players: players.into_iter().collect(),
        }
    }

    pub fn get_robot_for(&self, player_id: PlayerID) -> Option<&Robot> {
        self.players.iter()
            .find(|p| p.id == player_id)
            .map(|p| &p.robot)
    }

    pub fn get_robot(&self, robot_id: RobotID) -> Option<&Robot> {
        self.players.iter()
            .find(|p| p.robot.id == robot_id)
            .map(|p| &p.robot)
    }

    pub fn update_robot(&self, new_robot: Robot) -> State {
        let old_player_index = self.players.iter()
            .position(|p| p.robot.id == new_robot.id)
            .unwrap();  // TODO logic error

        let mut new_players = self.players.clone();
        let old_player = new_players.remove(old_player_index);
        new_players.push(Player {
            robot: new_robot,
            ..old_player
        });

        State {
            players: new_players,
            board: self.board.clone(),
        }
    }

    pub fn find_robot_at(&self, pos: &Position) -> Option<&Robot> {
        self.players.iter()
            .find(|p| p.robot.position == *pos)
            .map(|p| &p.robot)
    }
}

pub type PlayerID = u32;

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerID,
    pub robot: Robot,
}

impl Player {
    pub fn new(id: PlayerID, robot: Robot) -> Player {
        Player {
            id,
            robot,
        }
    }
}

pub type RobotID = u32;

#[derive(Debug, Default, Clone, Builder)]
#[builder(default)]
pub struct Robot {
    pub id: RobotID,
    pub damage: u32,
    pub life_tokens: u32,
    pub position: Position,
    pub direction: EDirection,
}

impl Robot {
    pub fn set_direction(&self, new_direction: EDirection) -> Robot {
        Robot {
            direction: new_direction,
            ..*self
        }
    }

    pub fn set_position(&self, new_position: Position) -> Robot {
        Robot {
            position: new_position,
            ..*self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EConnection {
    Free(Position),
    Walled,
}

/**
 * Spans a rectangular board constisting of tiles.
 * Not every tile is playable, [0, 0] is the North-West/upper-left corner
 */
#[derive(Debug, Default)]
pub struct Board {
    pub tiles: Vec<Tile>,
    pub size_x: u32,
    pub size_y: u32,
}

impl Board {
    pub fn new_empty_board(size_x: u32, size_y: u32) -> Board {
        let mut board = Board {
            tiles: Vec::with_capacity((size_x * size_y) as usize),
            size_x,
            size_y
        };
        board.fill_with_tiles(ETileType::Free);
        board
    }

    pub fn fill_with_tiles(&mut self, ttype: ETileType) {
        for x in 0..self.size_x {
            for y in 0..self.size_y {
                let position = Position::new(x, y);
                let tile = TileBuilder::default()
                    .position(position)
                    .ttype(ttype)
                    .walls(vec![])
                    .build().unwrap();
                self.tiles.push(tile);
            }
        }
    }

    pub fn get_neighbor_in(&self, pos: &Position, direction: EDirection) -> Option<EConnection> {
        let new_pos = match direction {
            EDirection::NORTH =>
                Board::ensure_on_board(pos.y.overflowing_sub(1), self.size_y)
                    .map(|n| pos.set_y(n)),
            EDirection::SOUTH => 
                Board::ensure_on_board(pos.y.overflowing_add(1), self.size_y)
                    .map(|n| pos.set_y(n)),
            EDirection::WEST => 
                Board::ensure_on_board(pos.x.overflowing_sub(1), self.size_x)
                    .map(|n| pos.set_x(n)),
            EDirection::EAST => 
                Board::ensure_on_board(pos.x.overflowing_add(1), self.size_x)
                    .map(|n| pos.set_x(n)),
        }?;

        let old_tile = &self.tiles[self.tile_index(pos)];
        let new_tile = &self.tiles[self.tile_index(&new_pos)];
        
        if old_tile.walls.contains(&direction) || new_tile.walls.contains(&direction.opposite()) {
            return Some(EConnection::Walled);
        }
        Some(EConnection::Free(new_pos))
    }

    fn ensure_on_board((new_val, overflow): (u32, bool), max: u32) -> Option<u32> {
        if overflow {
            return None;
        }
        if new_val >= max {
            return None;
        }
        Some(new_val)
    }

    fn tile_index(&self, pos: &Position) -> usize {
        (pos.x + (pos.y * self.size_x)) as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EDirection {
    NORTH,
    EAST,
    SOUTH,
    WEST,
}

impl EDirection {

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
        static DIRECTIONS: [EDirection;  4] = [EDirection::NORTH, EDirection::EAST, EDirection::SOUTH, EDirection::WEST];
        
        let index = DIRECTIONS.iter().position(|d| *d == self).unwrap();
        let max = DIRECTIONS.len() as i8;
        let new_index = (index as i8 + offset + max) % max;
        DIRECTIONS[new_index as usize]
    }
}

impl Default for EDirection {
    fn default() -> EDirection {
        EDirection::NORTH
    }
}

#[derive(Debug, Builder)]
pub struct Tile {
    pub position: Position,
    pub ttype: ETileType,
    pub walls: Vec<EDirection>,
}

#[derive(Debug, Copy, Clone)]
pub enum ETileType {
    Free,
    NoTile,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Position {
        Position {
            x,
            y,
        }
    }

    pub fn set_x(&self, x: u32) -> Position {
        Position {
            x,
            y: self.y,
        }
    }

    pub fn set_y(&self, y: u32) -> Position {
        Position {
            x: self.x,
            y,
        }
    }
}