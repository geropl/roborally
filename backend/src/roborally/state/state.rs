#![allow(clippy::trivially_copy_pass_by_ref)]

use failure::Error;

use std::sync::Arc;

use super::*;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub board: Arc<Board>,
    players: Vec<Player>,
    pub deck: ProgramCardDeck,
}

impl State {
    pub fn create_from(config: &GameConfig) -> Result<Box<State>, Error> {
        let board = Board::create_from(&config.board_config)?;
        let mut gen = ProgramCardDeckGenerator::new();
        Ok(Box::from(State {
            board: Arc::new(board),
            players: Player::create_from(&config.player_config),
            deck: gen.generate_program_deck(&config.deck_config),
        }))
    }

    #[cfg(test)]
    pub fn new_with_random_deck(board: Board, players: Vec<Player>) -> Box<State> {
        let config = ProgramCardDeckConfig::default();
        let mut gen = ProgramCardDeckGenerator::new();
        Box::from(State {
            board: Arc::new(board),
            players: players.into_iter().collect(),
            deck: gen.generate_program_deck(&config),
        })
    }

    pub fn get_robot_by_player_id_or_fail(&self, player_id: PlayerID) -> Result<&Robot, StateError> {
        self.players.iter()
            .find(|p| p.id == player_id)
            .map(|p| &p.robot)
            .ok_or(StateError::RobotNotFoundPlayerID{ player_id })
    }

    pub fn get_robot_by_id_or_fail(&self, robot_id: RobotID) -> Result<&Robot, StateError> {
        self.players.iter()
            .find(|p| p.robot.id == robot_id)
            .map(|p| &p.robot)
            .ok_or(StateError::RobotNotFoundID{ robot_id })
    }

    pub fn get_player_or_fail(&self, player_id: PlayerID) -> Result<&Player, StateError> {
        self.players.iter()
            .find(|p| p.id == player_id)
            .ok_or(StateError::PlayerNotFound{ player_id })
    }

    pub fn update_robot(&self, new_robot: Robot) -> Result<Box<State>, StateError> {
        let old_player_index = self.players.iter()
            .position(|p| p.robot.id == new_robot.id)
            .ok_or(StateError::RobotNotFoundID{ robot_id: new_robot.id })?;

        let mut new_players = self.players.clone();
        new_players[old_player_index] = Player {
            robot: new_robot,
            ..new_players[old_player_index].clone()
        };

        Ok(Box::from(State {
            players: new_players,
            board: self.board.clone(),
            deck: self.deck.clone(),
        }))
    }

    pub fn update_player(&self, new_player: Player) -> Result<Box<State>, StateError> {
        let old_player_index = self.players.iter()
            .position(|p| p.id == new_player.id)
            .ok_or(StateError::PlayerNotFound{ player_id: new_player.id })?;

        let mut new_players = self.players.clone();
        new_players[old_player_index] = new_player;

        Ok(Box::from(State {
            players: new_players,
            board: self.board.clone(),
            deck: self.deck.clone(),
        }))
    }

    pub fn set_deck(&self, new_deck: ProgramCardDeck) -> Box<State> {
        Box::from(State {
            deck: new_deck,
            players: self.players.clone(),
            board: self.board.clone(),
        })
    }

    pub fn find_robot_at(&self, pos: &Position) -> Option<&Robot> {
        self.players.iter()
            .find(|p| p.robot.position == *pos)
            .map(|p| &p.robot)
    }

    pub fn lock_registers_according_to_damage(&self) -> Box<State> {
        let mut state = Box::from(self.clone());
        for p in &mut state.players {
            for i in 0..REGISTER_COUNT {
                let mut r = p.registers.get_mut(i).unwrap();
                r.locked = i >= REGISTER_COUNT + 4 - (p.robot.damage as usize);  // 5 damage -> lock 5; damage -> lock 5,4;
            }
        }
        state
    }
    
    pub fn get_register_cards_sorted_by_priority(&self, register_index: usize) -> Result<Vec<(PlayerID, MoveCard)>, StateError> {
        let mut moves = Vec::with_capacity(self.players.len());
        for p in &self.players {
            let register = &p.registers[register_index];
            if register.move_card.is_none() {
                return Err(StateError::EmptyProgramRegister{ player_id: p.id });
            }
            moves.push((p.id, register.move_card.clone().unwrap()));
        }
        moves.sort_by(|a, b| a.1.priority.partial_cmp(&b.1.priority).unwrap());
        Ok(moves)
    }

    pub fn all_players(&self) -> impl Iterator<Item=&Player> {
        self.players.iter()
    }

    pub fn active_players(&self) -> impl Iterator<Item=&Player> {
        self.players.iter()
            .filter(|p| p.is_active())
    }

    // This is a work-around for the fact that we want to iterate over players while modifying state (which contains player)
    pub fn active_players_cloned(&self) -> PlayerIter {
        PlayerIter::new(self.players.iter()
            .filter(|p| p.is_active())
            .cloned()
            .collect())
    }

    pub fn register_count(&self) -> usize {
        self.players[0].registers.len()
    }
}