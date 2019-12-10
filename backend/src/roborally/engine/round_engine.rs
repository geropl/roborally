use failure::Fail;

use crate::roborally::state::{
    State,
    PlayerID,
    EPoweredDown,
    StateError,
    RoundPhase,
    REGISTER_COUNT,
};
use super::execution_engine::{ ExecutionEngine, ExecutionEngineError };
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
        expected: RoundPhase,
        actual: RoundPhase,
    }
}

// TODO this is dump. Maybe move all errors into one enum? Or is there a better way to chain errors?
impl From<StateError> for EngineError {
    fn from(err: StateError) -> EngineError {
        EngineError::GenericAlgorithmError{ msg: format!{"{}", err} }
    }
}

impl From<ExecutionEngineError> for EngineError {
    fn from(err: ExecutionEngineError) -> EngineError {
        EngineError::GenericAlgorithmError{ msg: format!{"{}", err} }
    }
}

pub struct RoundEngine {
    pub exec_engine: ExecutionEngine,
}

impl RoundEngine {
    pub fn new() -> Self {
        RoundEngine {
            exec_engine: ExecutionEngine::new()
        }
    }
    
    pub fn run_round_initialization(&self, state: Box<State>) -> Result<Box<State>, EngineError> {
        assert_phase(&state, RoundPhase::PREPARATION)?;
        let mut state = state;

        // 0. Prepare
        //  - powered down robot:
        //    - discard all damage tokens
        for i in 0..state.players.len() {
            let player = &state.players[i];
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
        for i in 0..state.players.len() {
            let player = &state.players[i];
            let cards_to_draw = 9 - player.robot.damage;
            let (deck, cards) = state.deck.take_random_cards(cards_to_draw);
            let new_player = player.set_program_card_deck(cards);
            state = state.update_player(new_player)?;
            state = state.set_deck(deck);
        }

        Ok(state)
    }

    pub fn set_player_input(&self, state: Box<State>, input: &PlayerInput) -> Result<Box<State>, EngineError> {
        assert_phase(&state, RoundPhase::PROGRAM)?;
        let mut state = state;

        // 2. Program registers + 3. Announce Power Down
        //  - input:
        //    - registers
        //    - if powered_down:
        //       - leave powered down?
        //      else
        //       - player with damaged robots may announce power down _for next turn_
        // TODO Power down
        let player = state.get_player_or_fail(input.player_id)?;
        let unlocked_registers_count = player.count_unlocked_registers();
        if unlocked_registers_count != input.move_cards.len() {
            return Err(EngineError::InvalidPlayerInput {
                player_id: input.player_id,
                msg: "Got more program cards than unlocked registers!".to_string(),
            });
        }

        let mut new_player = player.clone();
        for i in 0..input.move_cards.len() {
            new_player.registers[i].move_card = Some(input.move_cards[i].clone());
        }
        state = state.update_player(new_player)?;

        if all_players_provided_input(&state) {
            state = state.set_phase(RoundPhase::EXECUTE)
        }

        Ok(state)
    }

    pub fn run_execute(&self, state: Box<State>) -> Result<Box<State>, EngineError> {
        assert_phase(&state, RoundPhase::EXECUTE)?;
        let mut state = state;

        // 4. Register execution phase
        state = self.exec_engine.run_register_phase(state)?;
        state = state.set_phase(RoundPhase::CLEANUP);

        // 5. Cleanup
        //  - TODO repairs and upgrades:
        //    - single-wrench: -1 damage token
        //    - crossed-wrench: -1 damage token + option card

        //  - discard all program cards from registers that aren't locked
        for p in &state.players {
            let (cards, new_player) = p.take_program_cards_from_unlocked_registers();
            let new_deck = state.deck.add_cards(cards);
            state = state.set_deck(new_deck);
            state = state.update_player(new_player)?;
        }

        // TODO When to check for death?

        Ok(state.set_phase(RoundPhase::PREPARATION))
    }
}

fn all_players_provided_input(state: &State) -> bool {
    state.players.iter()
        .all(|p| p.registers.len() == REGISTER_COUNT)
}

fn assert_phase(state: &State, expected: RoundPhase) -> Result<(), EngineError> {
    if state.phase != expected {
        return Err(EngineError::InvalidRoundPhase {
            expected,
            actual: state.phase
        });
    }
    Ok(())
}