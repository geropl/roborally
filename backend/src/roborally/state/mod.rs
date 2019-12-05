use failure::Fail;

mod cards;
mod board;
mod player;

pub use board::*;
pub use cards::*;
pub use player::*;

#[derive(Debug, Fail)]
pub enum StateError {
    #[fail(display = "Robot with id {} not found", robot_id)]
    RobotNotFoundID {
        robot_id: RobotID,
    },
}