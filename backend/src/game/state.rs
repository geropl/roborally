use derive_builder::Builder;

use std::clone::Clone;
use std::rc::Rc;
use std::default::Default;

#[derive(Debug)]
pub struct State {
    pub board: Rc<Board>,
    players: Vec<Box<Player>>,
}

impl State {
    pub fn new(board: Board, players: Vec<Player>) -> State {
        State {
            board: Rc::new(board),
            players: players.into_iter().map(|p| Box::from(p)).collect(),
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

        let mut new_players = Vec::from(self.players.clone());
        let old_player = new_players.remove(old_player_index);
        new_players.push(Box::from(Player {
            robot: new_robot,
            ..*old_player
        }));

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
    pub damage: u16,
    pub life_tokens: u16,
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

/**
 * Spans a rectangular board constisting of tiles.
 * Not every tile is playable, [0, 0] is the North-West/upper-left corner
 */
#[derive(Debug)]
pub struct Board {
    tiles: Vec<Tile>,
    size_x: usize,
    size_y: usize,
}

impl Board {
    pub fn new_empty_board(size_x: usize, size_y: usize) -> Board {
        let mut board = Board {
            tiles: Vec::with_capacity((size_x * size_y) as usize),
            size_x,
            size_y
        };
        board.fill_with_tiles(ETileType::FREE);
        board
    }

    pub fn fill_with_tiles(&mut self, ttype: ETileType) {
        for x in 0..self.size_x {
            for y in 0..self.size_y {
                let position = Position::new(x, y);
                self.tiles.push(Tile::new(position, ttype));
            }
        }
    }

    pub fn get_neighbor_in(&self, pos: &Position, direction: &EDirection) -> Position {
        match direction {
            EDirection::NORTH => pos.set_y(pos.y - 1),
            EDirection::SOUTH => pos.set_y(pos.y + 1),
            EDirection::WEST => pos.set_x(pos.x - 1),
            EDirection::EAST => pos.set_x(pos.x + 1),
        }
    }

    pub fn is_wall_between(&self, a: &Position, b: &Position) -> bool {
        // TODO implement
        false
    }

    pub fn is_on_board(&self, pos: &Position) -> bool {
        // TODO implement
        true
    }

    fn tile_index(&self, x: usize, y: usize) -> usize {
        x + (y * self.size_x)
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
    const DIRECTIONS: [EDirection;  4] = [EDirection::NORTH, EDirection::EAST, EDirection::SOUTH, EDirection::WEST];

    pub fn turn_left(&self) -> EDirection {
        self.turn(-1)
    }

    pub fn turn_right(&self) -> EDirection {
        self.turn(1)
    }

    pub fn turn_around(&self) -> EDirection {
        self.turn(2)
    }

    fn turn(&self, offset: i8) -> EDirection {
        let index = EDirection::DIRECTIONS.iter().position(|d| d == self).unwrap();
        let max = EDirection::DIRECTIONS.len() as i8;
        let new_index = (index as i8 + offset + max) % max;
        EDirection::DIRECTIONS[new_index as usize]
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
}

impl Tile {
    pub fn new(position: Position, ttype: ETileType) -> Tile {
        Tile {
            position,
            ttype
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ETileType {
    FREE,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position {
            x,
            y,
        }
    }

    pub fn set_x(&self, x: usize) -> Position {
        Position {
            x,
            y: self.y,
        }
    }

    pub fn set_y(&self, y: usize) -> Position {
        Position {
            x: self.x,
            y,
        }
    }
}