use failure::{ Fail, Error };

use crate::roborally::state::{
    GameState,
    Round,
    State,
    PlayerID,
    Player,
    EPoweredDown,
    StateError,
    ERoundPhase,
    EGamePhase,
    EGameResult,
    MAX_DAMAGE_TOKENS,
};
use super::register_engine::{ RegisterEngine, RegisterEngineError };
use super::player_input::{ PlayerInput };

#[derive(Debug, Fail)]
pub enum EngineError {
    #[fail(display = "Invalid input for player {}: {}", player_id, msg)]
    InvalidPlayerInput {
        player_id: PlayerID,
        msg: String,
    },
    #[fail(display = "Engine error: {}", msg)]
    GenericAlgorithmError {
        msg: String,
    },
    #[fail(display = "Invalid round phase! Expected: {:?}, found: {:?}", expected, actual)]
    InvalidRoundPhase {
        expected: ERoundPhase,
        actual: ERoundPhase,
    },
    #[fail(display = "Invalid game phase! Expected: {:?}, found: {:?}", expected, actual)]
    InvalidGamePhase {
        expected: EGamePhase,
        actual: EGamePhase,
    }
}

// TODO this is dump. Maybe move all errors into one enum? Or is there a better way to chain errors?
impl From<StateError> for EngineError {
    fn from(err: StateError) -> EngineError {
        EngineError::GenericAlgorithmError{ msg: format!{"{}", err} }
    }
}

impl From<RegisterEngineError> for EngineError {
    fn from(err: RegisterEngineError) -> EngineError {
        EngineError::GenericAlgorithmError{ msg: format!{"{}", err} }
    }
}

pub struct GameEngine {
    pub game_engine: RoundEngine,
}

impl GameEngine {
    pub fn new() -> Self {
        GameEngine {
            game_engine: RoundEngine::new()
        }
    }

    pub fn initialize(&self, game_state: &mut GameState) -> Result<(), Error> {
        assert_game_phase(&game_state, EGamePhase::INITIAL)?;
        // TODO EGamePhase::PREPARATION: User input necessary: Choose start positions (order random, has to be stored for later choices)

        // Create and initialize first round
        let round = game_state.add_round();
        let round = self.game_engine.run_round_initialization(round)?;
        game_state.update_round(round)?;

        game_state.phase = EGamePhase::RUNNING;
        Ok(())
    }

    pub fn set_player_program_input(&self, game_state: &mut GameState, input: &PlayerInput) -> Result<(), Error> {
        assert_game_phase(&game_state, EGamePhase::RUNNING)?;
        let round = game_state.current_round()?;
        let mut round = self.game_engine.set_player_program_input(round, input)?;

        if round.phase == ERoundPhase::EXECUTION {
            let (rnd, game_result) = self.game_engine.run_execute(round)?;
            round = rnd;
            if game_result.is_some() {
                game_state.game_result = game_result;
            }
        }
        let current_phase = round.phase;
        game_state.update_round(round)?;

        // Has current round ended? Start new!
        if current_phase == ERoundPhase::DONE {
            let round = game_state.add_round();
            let round = self.game_engine.run_round_initialization(round)?;
            game_state.update_round(round)?;
        }
        Ok(())
    }
}

pub struct RoundEngine {
    pub register_engine: RegisterEngine,
}

impl RoundEngine {
    pub fn new() -> Self {
        RoundEngine {
            register_engine: RegisterEngine::new()
        }
    }
    
    fn run_round_initialization(&self, round: &Round) -> Result<Round, EngineError> {
        assert_round_phase(&round, ERoundPhase::INITIALIZATION)?;
        let mut state = round.state.clone();

        // 0. Prepare
        //  - powered down robot:
        //    - discard all damage tokens
        let player_it = state.active_players_cloned();
        for player in player_it.iter() {
            if player.robot.powered_down == EPoweredDown::Yes ||
                player.robot.powered_down == EPoweredDown::NextRound {
                let mut new_robot = player.robot.set_damage(0);
                
                //  - advance power down state
                if new_robot.powered_down == EPoweredDown::NextRound {
                    new_robot = new_robot.set_powered_down(EPoweredDown::Yes);
                }
                state = state.update_robot(new_robot)?;
            }
        }

        // 1. Deal Program Cards:
        //  - draw 9 cards randomly (- damage tokens) cards
        let player_it = state.active_players_cloned();
        for player in player_it.iter() {
            let cards_to_draw = MAX_DAMAGE_TOKENS - player.robot.damage;
            let (deck, cards) = state.deck.take_random_cards(cards_to_draw);
            let new_player = player.set_program_card_deck(cards);
            state = state.update_player(new_player)?;
            state = state.set_deck(deck);
        }

        Ok(round.advance(state, ERoundPhase::PROGRAMMING))
    }

    fn set_player_program_input(&self, round: &Round, input: &PlayerInput) -> Result<Round, EngineError> {
        assert_round_phase(&round, ERoundPhase::PROGRAMMING)?;
        let mut state = round.state.clone();

        // 2. Program registers + 3. Announce Power Down
        //  - input:
        //    - registers
        let new_player = self.set_registers(&state, input)?;
        state = state.update_player(new_player)?;

        //    - if powered_down:
        //       - leave powered down?
        //      else
        //       - player with damaged robots may announce power down _for next turn_
        // TODO Power down

        // Has this phase ended?
        let next_phase = if all_players_provided_input(&state) {
            ERoundPhase::EXECUTION
        } else {
            round.phase
        };

        Ok(round.advance(state, next_phase))
    }

    fn set_registers(&self, state: &State, input: &PlayerInput) -> Result<Player, EngineError> {
        let player = state.get_player_or_fail(input.player_id)?;
        let unlocked_registers_count = player.count_unlocked_registers();
        let input_register_count = input.register_cards_choices.len();
        if unlocked_registers_count != input_register_count {
            return Err(EngineError::InvalidPlayerInput {
                player_id: input.player_id,
                msg: format!("Got more program cards ({}) than unlocked registers ({})!", input_register_count, unlocked_registers_count),
            });
        }

        let mut new_player = player.clone();
        for i in 0..input.register_cards_choices.len() {
            let move_card_id = input.register_cards_choices[i];
            new_player = new_player.choose_card(i, move_card_id)?;
        }
        Ok(new_player)
    }

    fn run_execute(&self, round: Round) -> Result<(Round, EGameResult), EngineError> {
        assert_round_phase(&round, ERoundPhase::EXECUTION)?;
        let mut state = round.state.clone();

        // 4. Register execution phase
        let active_player_ids: Vec<u32> = state.active_player_ids();
        state = self.register_engine.execute_registers(state)?;

        // TODO Dead robots may reenter if life_tokens are left
        
        // Check for end game condition
        let active_player_ids_after: Vec<u32> = state.active_player_ids();
        let game_result = match active_player_ids_after.len() {
            0 => EGameResult::Draw { player_ids: active_player_ids },
            1 => EGameResult::Win { player_id: active_player_ids_after[0] },
            _ => EGameResult::None,
        };
        if game_result.is_some() {
            return Ok((round.advance(state, round.phase), game_result))
        }

        // 5. Cleanup
        let round = self.run_cleanup(round.advance(state, ERoundPhase::CLEANUP))?;
        Ok((round, EGameResult::None))
    }

    fn run_cleanup(&self, round: Round) -> Result<Round, EngineError> {
        assert_round_phase(&round, ERoundPhase::CLEANUP)?;
        let mut state = round.state.clone();

        // 5. Cleanup
        // TODO Reenter play on archive marker
        //  - TODO repairs and upgrades:
        //    - single-wrench: -1 damage token
        //    - crossed-wrench: -1 damage token + option card

        // adjust registers locks according to damage
        state = state.lock_registers_according_to_damage();

        //  - discard all program cards from registers that aren't locked
        let player_it = state.active_players_cloned();
        for player in player_it.iter() {
            let (cards, new_player) = player.take_program_cards_from_unlocked_registers();
            let new_deck = state.deck.add_cards(cards);
            state = state.set_deck(new_deck);
            state = state.update_player(new_player)?;
        }

        Ok(round.advance(state, ERoundPhase::DONE))
    }
}

fn all_players_provided_input(state: &State) -> bool {
    state.active_players()
        .all(|p| p.registers.iter()
            .all(|r| r.move_card.is_some()))
}

fn assert_game_phase(game_state: &GameState, expected: EGamePhase) -> Result<(), EngineError> {
    if game_state.phase != expected {
        return Err(EngineError::InvalidGamePhase {
            expected,
            actual: game_state.phase
        });
    }
    Ok(())
}

fn assert_round_phase(round: &Round, expected: ERoundPhase) -> Result<(), EngineError> {
    if round.phase != expected {
        return Err(EngineError::InvalidRoundPhase {
            expected,
            actual: round.phase
        });
    }
    Ok(())
}