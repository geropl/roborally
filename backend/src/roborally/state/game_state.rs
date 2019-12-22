use super::{ ProgramCardDeckConfig, BoardConfig, PlayerConfig, State, StateError, PlayerID };

use failure::Error;

#[derive(Debug, Default)]
pub struct GameConfig {
    pub deck_config: ProgramCardDeckConfig,
    pub board_config: BoardConfig,
    pub player_config: PlayerConfig,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub phase: EGamePhase,
    initial_state: Box<State>,
    pub start_state: Box<State>,
    player_precedence: Vec<PlayerID>,   // TODO move this into state?
    rounds: Vec<Round>,
    pub game_result: EGameResult,
}

impl GameState {
    pub fn create_from(config: &GameConfig) -> Result<GameState, Error> {
        let initial_state = State::create_from(config)?;
        let player_precedence = GameState::random_player_precedence(&initial_state);
        Ok(GameState {
            phase: EGamePhase::INITIAL,
            start_state: initial_state.clone(),
            initial_state,
            player_precedence,
            rounds: vec![],
            game_result: EGameResult::None,
        })
    }

    pub fn update_round(&mut self, round: Round) -> Result<(), Error> {
        let i = self.rounds.iter().position(|r| r.id == round.id)
            .ok_or(StateError::RoundNotFound{ round_id: round.id })?;
        self.rounds[i] = round;
        Ok(())
    }

    pub fn add_round(&mut self) -> &Round {
        let state = match self.rounds.last() {
            Some(r) => &r.state,
            None => &self.initial_state,
        };
        let round = Round::new(self.rounds.len() as u32, state.clone());
        self.rounds.push(round);
        self.rounds.last().unwrap()
    }

    pub fn current_round(&self) -> Result<&Round, StateError> {
        match self.rounds.last() {
            None => Err(StateError::GameStateMissingRound{ round_id: None }),
            Some(r) => Ok(r),
        }
    }

    pub fn all_rounds(&self) -> impl Iterator<Item=&Round> {
        self.rounds.iter()
    }

    pub fn initial_state(&self) -> &State {
        &self.initial_state
    }

    pub fn first_player_id_by_precedence(&self) -> PlayerID {
        *self.player_precedence.first().unwrap()
    }

    pub fn next_player_id_by_precedence(&self, current_id: PlayerID) -> Option<PlayerID> {
        match self.player_precedence.iter().position(|pid| *pid == current_id) {
            None => None,   // TODO should be an error, really. Would be straight forward if this was in State...
            Some(current_position) => self.player_precedence.get(current_position + 1).cloned(),
        }
    }

    fn random_player_precedence(state: &State) -> Vec<PlayerID> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();

        let mut ids = state.active_player_ids();
        ids.shuffle(&mut rng);
        ids
    }
}

// This is only needed to initialize Arc<GameState> and not used anywhere in th game
impl Default for GameState {
    fn default() -> Self {
        let initial_state = Box::from(State::default());
        GameState {
            phase: EGamePhase::INITIAL,
            start_state: initial_state.clone(),
            initial_state,
            player_precedence: vec![],
            rounds: vec![],
            game_result: EGameResult::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EGamePhase {
    INITIAL,
    PREPARATION,
    RUNNING,
    ENDED,
}

pub type RoundID = u32;

#[derive(Debug, Clone)]
pub struct Round {
    pub id: RoundID,
    pub phase: ERoundPhase,
    pub state: Box<State>,
}

impl Round {
    pub fn new(id: RoundID, state: Box<State>) -> Round {
        Round {
            id,
            phase: ERoundPhase::INITIALIZATION,
            state,
        }
    }

    pub fn advance(&self, state: Box<State>, phase: ERoundPhase) -> Round {
        Round {
            id: self.id,
            state,
            phase
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ERoundPhase {
    /**
     * Cards are dealt + some maintenance stuff (power down)
     */
    INITIALIZATION,

    /**
     * The program phase: players fill their registers with their dealt cards and may announce power down
     */
    PROGRAMMING,

    /**
     * The robots are moved according to the users programs and all resulting effects are executed
     */
    EXECUTION,

    /**
     * Robots are repaired and option cards are drawn and executed
     */
    CLEANUP,

    /**
     * All activities this round are done. Either start with next round or end this game (if the conditions are met)
     */
    DONE
}

#[derive(Debug, Clone, PartialEq)]
pub enum EGameResult {
    Draw { player_ids: Vec<PlayerID>, },
    Win { player_id: PlayerID, },
    None,
}
impl EGameResult {
    pub fn is_some(&self) -> bool {
        *self != EGameResult::None
    }
}