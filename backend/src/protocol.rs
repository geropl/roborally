use crate::game::state;

tonic::include_proto!("protocol");

impl From<&Box<state::State>> for GameState {
    fn from(state: &Box<state::State>) -> GameState {
        use std::borrow::Borrow;

        let players: Vec<Player> = state.players.iter()
            .map(Player::from)
            .collect();
        GameState {
            board: Some(Board::from(state.board.borrow())),
            players,
        }
    }
}

impl From<&state::State> for GameState {
    fn from(state: &state::State) -> GameState {
        use std::borrow::Borrow;

        let players: Vec<Player> = state.players.iter()
            .map(Player::from)
            .collect();
        GameState {
            board: Some(Board::from(state.board.borrow())),
            players,
        }
    }
}

impl From<&state::Player> for Player {
    fn from(player: &state::Player) -> Player {
        Player {
            id: player.id,
            robot: Some(Robot::from(&player.robot)),
        }
    }
}

impl From<&state::Robot> for Robot {
    fn from(robot: &state::Robot) -> Robot {
        Robot {
            id: robot.id,
            position: Some((&robot.position).into()),
            direction: EDirection::from(robot.direction).into(),
            damage: robot.damage,
            life_tokens: robot.life_tokens,
        }
    }
}

impl From<&state::Board> for Board {
    fn from(board: &state::Board) -> Board {
        let tiles: Vec<Tile> = board.tiles.iter()
            .map(Tile::from)
            .collect();
        Board {
            tiles,
            size_x: board.size_x,
            size_y: board.size_y,
        }
    }
}

impl From<&state::Tile> for Tile {
    fn from(tile: &state::Tile) -> Tile {
        let ttype: ETileType = tile.ttype.into();
        let walls = tile.walls.iter()
            .map(|dir| EDirection::from(*dir).into())
            .collect();
        Tile {
            position: Some((&tile.position).into()),
            r#type: ttype.into(),
            walls,
        }
    }
}

impl From<state::EDirection> for EDirection {
    fn from(dir: state::EDirection) -> EDirection {
        match dir {
            state::EDirection::NORTH => EDirection::North,
            state::EDirection::EAST => EDirection::East,
            state::EDirection::SOUTH => EDirection::South,
            state::EDirection::WEST => EDirection::West,
        }
    }
}

impl From<state::ETileType> for ETileType {
    fn from(ttype: state::ETileType) -> ETileType {
        match ttype {
            state::ETileType::Free => ETileType::Free,
            state::ETileType::NoTile => ETileType::NoTile,
        }
    }
}

impl From<&state::Position> for Position {
    fn from(pos: &state::Position) -> Position {
        Position {
            x: pos.x,
            y: pos.y,
        }
    }
}