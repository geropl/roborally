use failure::Fail;

use crate::roborally::state::{
    State,
    PlayerID,
    EPoweredDown,
    StateError,
    // RobotID,
    // EDirection,
    // EConnection,
    // Position,
};

#[derive(Debug, Fail)]
pub enum EngineError {
    #[fail(display = "Invalid player id: {}", player_id)]
    Invalid {
        player_id: PlayerID,
    },
    #[fail(display = "Engine error: {}", msg)]
    GenericAlgorithmError {
        msg: String,
    }
}

impl From<StateError> for EngineError {
    fn from(err: StateError) -> EngineError {
        // TODO this is dump. Maybe move all errors into one enum? Or is there a better way to chain errors?
        EngineError::GenericAlgorithmError{ msg: format!{"{}", err} }
    }
}

pub struct Engine {
}

impl Engine {
    pub fn run_round_initialization(&self, state: Box<State>) -> Result<Box<State>, EngineError> {
        let mut state = state;

        // 0. Prepare
        //  - powered down robot:
        //    - discard all damage tokens
        for i in 0..(state.players.len() - 1) {
            let player = &state.players[i];
            if player.robot.powered_down == EPoweredDown::Yes ||
                player.robot.powered_down == EPoweredDown::NextRound {
                let mut new_robot = player.robot.set_damage(0);
                
                //  - advance power down state
                if player.robot.powered_down == EPoweredDown::NextRound {
                    new_robot = new_robot.set_powered_down(EPoweredDown::Yes);
                }
                state = Box::from(state.update_robot(new_robot)?);
            }
        }

        // 1. Deal Program Cards:
        //  - shuffle
        //  - draw 9 (- damage tokens, - locked registers) cards

        // 2. Program registers
        //  - registers
        //  - 
        //  - input: leave powered down?

        // 3, Announce Power Down
        //  - player with damaged robots may announce power down _for next turn_
        //  - 

        // 4. Complete Registers (register phase)

        // 5. Cleanup
        //  - repairs and upgrades:
        //    - single-wrench: -1 damage token
        //    - crossed-wrench: -1 damage token + option card
        //  - discard all program cards from registers that aren't locked
        Err(EngineError::Invalid{ player_id: 0 })
    }
}