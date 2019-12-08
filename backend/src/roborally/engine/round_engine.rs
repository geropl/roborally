use failure::Fail;

use crate::roborally::state::{
    State,
    PlayerID,
    EPoweredDown,
    StateError,
    RoundPhase,
};
use super::execution_engine::{ ExecutionEngine };
use super::player_input::{ PlayerInput };

#[derive(Debug, Fail)]
pub enum EngineError {
    #[fail(display = "Invalid player id: {}", player_id)]
    Invalid {
        player_id: PlayerID,
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

impl From<StateError> for EngineError {
    fn from(err: StateError) -> EngineError {
        // TODO this is dump. Maybe move all errors into one enum? Or is there a better way to chain errors?
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
        let mut state = state;
        assert_phase(&state, RoundPhase::PREPARATION)?;

        // 0. Prepare
        //  - powered down robot:
        //    - discard all damage tokens
        for i in 0..(state.players.len() - 1) {
            let player = &state.players[i];
            if player.robot.powered_down == EPoweredDown::Yes ||
                player.robot.powered_down == EPoweredDown::NextRound {
                let mut new_robot = player.robot.set_damage(0);
                
                //  - advance power down state
                if new_robot.powered_down == EPoweredDown::NextRound {
                    new_robot = new_robot.set_powered_down(EPoweredDown::Yes);
                }
                state = Box::from(state.update_robot(new_robot)?);
            }
        }

        // 1. Deal Program Cards:
        //  - draw 9 cards randomly (- damage tokens) cards
        for i in 0..(state.players.len() - 1) {
            let player = &state.players[i];
            let cards_to_draw = 9 - player.robot.damage;
            let (deck, cards) = state.deck.take_random_cards(cards_to_draw);
            let new_player = player.set_program_card_deck(cards);
            state = Box::from(state.update_player(new_player)?);
            state = Box::from(state.set_deck(deck));
        }

        Ok(state)
    }

    pub fn set_player_input(&self, state: Box<State>, input: &PlayerInput) -> Result<Box<State>, EngineError> {
        assert_phase(&state, RoundPhase::PROGRAM)?;

        // 2. Program registers + 3. Announce Power Down
        //  - input:
        //    - registers
        //    - if powered_down:
        //       - leave powered down?
        //      else
        //       - player with damaged robots may announce power down _for next turn_

        if all_players_provided_input(&state) {
            Ok(state.set_phase(RoundPhase::EXECUTE))
        } else {
            Ok(state)
        }
    }

    pub fn run_execute(&self, state: Box<State>) -> Result<Box<State>, EngineError> {
        let mut state = state;
        assert_phase(&state, RoundPhase::EXECUTE)?;

        // 4. Register execution phase
        //state = self.exec_engine.run_register_phase(state)?;
        state = state.set_phase(RoundPhase::CLEANUP);

        // 5. Cleanup
        //  - repairs and upgrades:
        //    - single-wrench: -1 damage token
        //    - crossed-wrench: -1 damage token + option card
        //  - discard all program cards from registers that aren't locked
        // TODO When to check for death?

        Ok(state.set_phase(RoundPhase::PREPARATION))
    }
}

fn all_players_provided_input(state: &Box<State>) -> bool {
    false   // TODO implement
}

fn assert_phase(state: &Box<State>, expected: RoundPhase) -> Result<(), EngineError> {
    if state.phase != expected {
        return Err(EngineError::InvalidRoundPhase{
            expected,
            actual: state.phase
        });
    }
    Ok(())
}