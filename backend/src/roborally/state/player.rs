#![allow(clippy::trivially_copy_pass_by_ref)]

use derive_builder::Builder;

use super::board::*;
use super::cards::*;
use super::StateError;

pub type PlayerID = u32;

pub const REGISTER_COUNT: usize = 5;
/// The maximum number of damage tokens that a robot can take and still function. Anything above destroxy the robot.
pub const MAX_DAMAGE_TOKENS: u32 = 9;
pub const DEFAULT_LIFE_TOKENS: u32 = 3;

#[derive(Debug, Clone)]
pub struct PlayerConfig {
    player_count: usize,
    // register_count: u32, TODO depends on damage tokens/max damage!
    life_tokens: u32,
}
impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            player_count: 2,
            life_tokens: DEFAULT_LIFE_TOKENS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerID,
    pub robot: Robot,
    pub registers: Vec<Register>,
    pub program_card_deck: Vec<MoveCard>,
}

impl Player {
    pub fn create_from(config: &PlayerConfig) -> Vec<Player> {
        let mut players = Vec::with_capacity(config.player_count);
        for id in 0..config.player_count {
            players.push(Player {
                id: id as u32,
                robot: Robot {
                    id: id as u32,
                    damage: 0,
                    life_tokens: config.life_tokens,
                    powered_down: EPoweredDown::No,
                    position: Position::new(0, 0),
                    direction: EDirection::SOUTH,
                },
                registers: (0..REGISTER_COUNT).map(|_| Register::default()).collect(),
                program_card_deck: vec![],
            });
        }
        players
    }

    /**
     * For test purposes
     */
    #[cfg(test)]
    pub fn new_with_move(id: PlayerID, robot: Robot, move_card: MoveCard) -> Player {
        Player {
            id,
            robot,
            registers: vec![Register{
                move_card: Some(move_card),
                locked: false
            }],
            program_card_deck: vec![],
        }
    }

    pub fn set_program_card_deck(&self, program_card_deck: Vec<MoveCard>) -> Player {
        Player {
            robot: self.robot.clone(),
            program_card_deck,
            registers: self.registers.clone(),
            ..*self
        }
    }

    pub fn choose_card(&self, register_index: usize, move_card_id: MoveCardID) -> Result<Player, StateError> {
        let mut new_player = self.clone();
        let register = &mut new_player.registers[register_index];
        if register.move_card.is_some() {
            return Err(StateError::DoublePlayerInput {
                player_id: self.id,
            });
        }
        let card_index = new_player.program_card_deck.iter().position(|c| c.id == move_card_id)
            .ok_or(StateError::InvalidProgramCardChoice{
                player_id: self.id,
                move_card_id,
            })?;
        
        register.move_card = Some(new_player.program_card_deck.remove(card_index));
        Ok(new_player)
    }

    pub fn take_program_cards_from_unlocked_registers(&self) -> (Vec<MoveCard>, Player) {
        let mut cards = vec![];
        let mut new_player = self.clone();
        for r in &mut new_player.registers {
            if r.locked {
                continue;
            }
            cards.push(r.move_card.take().unwrap());
        }
        (cards, new_player)
    }

    pub fn count_unlocked_registers(&self) -> usize {
        let mut c: usize = 0;
        for r in &self.registers {
            if !r.locked {
                c += 1;
            }
        }
        c
        // self.registers.iter()
        //     .filter(|r| !r.locked)
        //     .count()
    }

    pub fn is_active(&self) -> bool {
        self.robot.life_tokens > 0 && !self.robot.is_destroyed()
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
    pub powered_down: EPoweredDown,
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

    pub fn set_damage(&self, new_damage: u32) -> Robot {
        Robot {
            damage: new_damage,
            ..*self
        }
    }

    pub fn set_powered_down(&self, powered_down: EPoweredDown) -> Robot {
        Robot {
            powered_down,
            ..*self
        }
    }

    pub fn die(&self) -> Robot {
        Robot {
            damage: 0,
            ..*self
        }
    }

    pub fn is_destroyed(&self) -> bool {
        self.damage == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EPoweredDown {
    No,
    NextRound,
    Yes
}

impl Default for EPoweredDown {
    fn default() -> EPoweredDown {
        EPoweredDown::No
    }
}

#[derive(Debug, Clone, Default)]
pub struct Register {
    pub move_card: Option<MoveCard>,
    pub locked: bool,
}

pub struct PlayerIter {
    players: Vec<Player>,
}

impl PlayerIter {
    pub fn new(players: Vec<Player>) -> PlayerIter {
        PlayerIter {
            players,
        }
    }

    pub fn iter(&self) -> PlayerIterator {
        PlayerIterator::new(&self.players)
    }
}

pub struct PlayerIterator<'a> {
    players: &'a [Player],
    next_index: usize,
    _phantom: std::marker::PhantomData<&'a Player>,
}

impl PlayerIterator<'_> {
    pub fn new(players: &[Player]) -> PlayerIterator {
        PlayerIterator {
            players,
            next_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for PlayerIterator<'a> {
    type Item = &'a Player;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.players.get(self.next_index);
        self.next_index += 1;
        result
    }
}