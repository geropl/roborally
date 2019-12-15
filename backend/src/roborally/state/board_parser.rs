#![allow(clippy::trivially_copy_pass_by_ref)]
use failure::Fail;

use std::path::{ Path, PathBuf };
use std::fs;
use std::str::Chars;

use super::{ Board, ETileType, Tile, Position, EDirection };

#[derive(Debug, Fail)]
pub enum ParserError {
    #[fail(display = "Error while reading from file: {}", msg)]
    FileError {
        msg: String,
    },
    #[fail(display = "Expected wall, found: '{}'", msg)]
    WallNotFound {
        msg: String,
    },
    #[fail(display = "Expected ETileType, found: '{}'", msg)]
    UnknownTileType {
        msg: String,
    },
    #[fail(display = "Unexpected end of row: {:?}", position)]
    EndOfRow {
        position: Position,
    },
    #[fail(display = "Unexpected end of row")]
    TileEndOfRow {
    },
    #[fail(display = "Unexpected row length: before ({}), after ({})", before, after)]
    UnexpectedRowLength {
        before: usize,
        after: usize,
    },
    #[fail(display = "Missing tile for hwall at: {:?}", position)]
    MissingTileForHWall {
        position: Position,
    },
}

pub fn load_board_by_name(name: &str) -> Result<Board, ParserError> {
    let base_path = PathBuf::from(format!("./backend/data/boards/{}.brd", name));
    if !base_path.exists() {
        return Err(ParserError::FileError{ msg: format!("File not found: {}", base_path.display()) });
    }
    load_board_by_name_from_file(&base_path)
}

pub fn load_board_by_name_from_file(path: &Path) -> Result<Board, ParserError> {
    let content = match fs::read_to_string(path) {
        Ok(c) => Ok(c),
        Err(e) => Err(ParserError::FileError{ msg: format!("{}", e) }),
    }?;

    parse_board(content)
}

fn parse_board(content: String) -> Result<Board, ParserError> {
    let mut tiles: Vec<Tile> = vec![];
    let mut hwalls: Vec<HWall> = vec![];
    let mut y_raw = 0;
    let mut y = 0;
    let mut x = 0;
    for row_str in content.split('\n') {
        if row_str.is_empty() {
            continue
        }

        if y_raw % 2 == 0 {
            let row_walls = parse_horizontal_wall_row(row_str, y)?;
            let row_length = row_walls.len();
            if x == 0 {
                x = row_length;
            } else if x != row_length {
                return Err(ParserError::UnexpectedRowLength{ before: x, after: row_length })
            }
            hwalls.extend(row_walls);
        } else {
            let row_tiles = parse_tile_row(row_str, y)?;
            if x != row_tiles.len() {
                return Err(ParserError::UnexpectedRowLength{ before: x, after: row_tiles.len() })
            }
            tiles.extend(row_tiles);
            y += 1;
        }
        y_raw += 1;
    }

    let x = x as i32;
    for hwall in hwalls {
        if hwall.wall {
            let mut position = hwall.position;
            let mut direction = EDirection::NORTH;
            if hwall.position.y == y {
                // Adjust position.y and Edirection for last HWall which affects the row before, not after
                position = Position {
                    y: hwall.position.y - 1,
                    ..hwall.position
                };
                direction = EDirection::SOUTH;
            };
            let index = index(&position, x);
            let tile = match tiles.get_mut(index) {
                Some(t) => t,
                None => return Err(ParserError::MissingTileForHWall{ position }),
            };
            tile.walls.push(direction);
        }
    }

    Ok(Board {
        tiles,
        size_x: x,
        size_y: y,
    })
}

fn index(pos: &Position, x_size: i32) -> usize {
    (pos.y * x_size + pos.x) as usize
}

struct HWall {
    position: Position,
    wall: bool,
}
fn parse_horizontal_wall_row(row_str: &str, y: i32) -> Result<Vec<HWall>, ParserError> {
    let mut walls: Vec<HWall> = vec![];
    let mut x = 0;
    let mut chars = row_str.chars();
    loop {
        let wall = match &[chars.next(), chars.next()] {
            [_, Some('-')] => Ok(true),
            [_, Some(' ')] => Ok(false),
            [_, None] => return Ok(walls),
            _ => Err(ParserError::EndOfRow{ position: Position{ x, y }}),
        }?;
        walls.push(HWall{
            position: Position{ x, y },
            wall,
        });
        x += 1;
    }
}

fn parse_tile_row(row_str: &str, y: i32) -> Result<Vec<Tile>, ParserError> {
    let mut tiles: Vec<Tile> = vec![];
    let mut x = 0;
    let mut chars = row_str.chars();
    loop {
        let wall = match_vertical_wall(&mut chars)?;
        let tile_type = match match_tile_type(&mut chars) {
            Ok(t) => t,
            Err(_) => {
                if tiles.is_empty() {
                    return Err(ParserError::EndOfRow{ position: Position{ x, y }})
                }

                if wall {
                    let last_index = tiles.len() - 1;
                    tiles[last_index].walls.push(EDirection::EAST);
                }
                break;
            },
        };
        
        tiles.push(Tile {
            position: Position{ x, y },
            ttype: tile_type,
            walls: if wall { vec![EDirection::WEST] } else { vec![] }
        });
        
        x += 1;
    }
    Ok(tiles)
}

fn match_vertical_wall(chars: &mut Chars) -> Result<bool, ParserError> {
    match chars.next() {
        Some('|') => Ok(true),
        Some(' ') => Ok(false),
        Some(c) => Err(ParserError::WallNotFound{ msg: c.to_string() }),
        None => Err(ParserError::TileEndOfRow{}),
    }
}

fn match_tile_type(chars: &mut Chars) -> Result<ETileType, ParserError> {
    match chars.next() {
        Some('o') => Ok(ETileType::Regular),
        Some(' ') => Ok(ETileType::NoTile),
        Some(c) => Err(ParserError::UnknownTileType{ msg: c.to_string() }),
        None => Err(ParserError::TileEndOfRow{}),
    }
}

#[cfg(test)]
mod test {
    use super::super::{ Board, ETileType, Tile, Position, EDirection };
    use super::{ ParserError, parse_board, index };


    #[test]
    fn test_parse_board() -> Result<(), ParserError> {
        let content = "
 -     
 o o o 
   - - 
 o|o   
   -   
 o|o o 
   -   ";
        let actual_board = parse_board(String::from(content))?;
        let expected_board = Board {
            tiles: vec![
                Tile {
                    position: Position{ x: 0, y: 0 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::NORTH],
                },
                Tile {
                    position: Position{ x: 1, y: 0 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::SOUTH],
                },
                Tile {
                    position: Position{ x: 2, y: 0 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::SOUTH],
                },
                Tile {
                    position: Position{ x: 0, y: 1 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::EAST],
                },
                Tile {
                    position: Position{ x: 1, y: 1 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::SOUTH],
                },
                Tile {
                    position: Position{ x: 2, y: 1 },
                    ttype: ETileType::NoTile,
                    walls: vec![],
                },
                Tile {
                    position: Position{ x: 0, y: 2 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::EAST],
                },
                Tile {
                    position: Position{ x: 1, y: 2 },
                    ttype: ETileType::Regular,
                    walls: vec![EDirection::SOUTH],
                },
                Tile {
                    position: Position{ x: 2, y: 2 },
                    ttype: ETileType::Regular,
                    walls: vec![],
                },
            ],
            size_x: 3,
            size_y: 3,
        };

        compare_boards(&expected_board, &actual_board);
        Ok(())
    }

    fn compare_boards(exp_board: &Board, act_board: &Board) {
        assert_eq!(exp_board.size_x, act_board.size_x, "size_x");
        assert_eq!(exp_board.size_y, act_board.size_y, "size_y");
        assert_eq!(exp_board.tiles.len(), act_board.tiles.len(), "tiles.len");

        // Compare
        for exp_tile in &exp_board.tiles {
            let index = index(&exp_tile.position, exp_board.size_x);
            let act_tile = match act_board.tiles.get(index) {
                Some(t) => t,
                None => panic!("Expected tile at: {:?}", exp_tile.position),
            };
            assert_eq!(exp_tile.position, act_tile.position, "position");
            assert_eq!(exp_tile.ttype, act_tile.ttype, "tile type");

            for dir in &EDirection::DIRECTIONS {
                let exp_neighbor = exp_board.get_neighbor_in(&exp_tile.position, *dir);
                let act_neighbor = act_board.get_neighbor_in(&exp_tile.position, *dir);
                assert_eq!(exp_neighbor, act_neighbor, "neighbor");
            }
        }
    }
}