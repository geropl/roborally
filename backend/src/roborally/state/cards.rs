use std::slice::Iter;

use std::fmt;

use crate::roborally::engine::execution_engine::{ ESimpleMove, TMove };

#[derive(Debug)]
pub struct ProgramCardDeckConfig {
    pub count_1_move: u32,
    pub count_2_move: u32,
    pub count_3_move: u32,
    pub count_back_up: u32,
    pub count_turn_right: u32,
    pub count_turn_left: u32,
    pub count_uturn: u32,
}

impl ProgramCardDeckConfig {
    pub fn card_count(&self) -> u32 {
        self.count_1_move + self.count_2_move + self.count_3_move + self.count_back_up + self.count_turn_left + self.count_turn_right + self.count_uturn
    }
}

impl Default for ProgramCardDeckConfig {
    fn default() -> ProgramCardDeckConfig {
        ProgramCardDeckConfig {
            count_1_move: 18,
            count_2_move: 12,
            count_3_move: 6,
            count_back_up: 6,
            count_turn_left: 18,
            count_turn_right: 18,
            count_uturn: 6,
        }
    }
}

pub struct ProgramCardDeckGenerator {
    rng: rand::rngs::ThreadRng,
}

impl ProgramCardDeckGenerator {
    pub fn new() -> ProgramCardDeckGenerator {
        ProgramCardDeckGenerator {
            rng: rand::thread_rng(),
        }
    }

    pub fn generate_program_deck(&self, config: ProgramCardDeckConfig) -> ProgramCardDeck {
        let mut cards = Vec::with_capacity(config.card_count() as usize);

        // TODO Unique priorities
        for _ in 1..config.count_1_move {
            cards.push(MoveCard::new_from_move(0, ESimpleMove::Forward));
        }
        ProgramCardDeck { cards }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProgramCardDeck {
    pub cards: Vec<MoveCard>,
}

impl ProgramCardDeck {
    // pub fn shuffle(&self) -> ProgramCardDeck {
    //     use rand::seq::SliceRandom;

    //     let mut new_cards = self.cards.clone();
    //     let mut rng = rand::thread_rng();
    //     new_cards.shuffle(&mut rng);

    //     ProgramCardDeck {
    //         cards: new_cards,
    //     }
    // }

    pub fn take_random_cards(&self, amount: u32) -> (ProgramCardDeck, Vec<MoveCard>) {
        use rand::seq::index;

        let amount = amount as usize;
        let mut cards = self.cards.clone();
        let mut rng = rand::thread_rng();
        let indeces = index::sample(&mut rng, cards.len(), amount).into_vec();

        let mut chosen_cards = Vec::with_capacity(amount);
        for i in indeces {
            chosen_cards.push(cards.remove(i));
        }

        (ProgramCardDeck { cards }, chosen_cards)
    }
}

#[derive(Debug, Clone)]
pub struct MoveCard {
    pub priority: u32,
    pub tmove: Box<dyn TMove + Send>,
}

impl fmt::Debug for (dyn TMove + Send + 'static) {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TMove")
    }
}
impl Clone for Box<dyn TMove + Send> {
    fn clone(&self) -> Box<dyn TMove + Send> {
        self.box_clone()
    }
}

impl MoveCard {
    pub fn new(priority: u32, tmove: Box<dyn TMove + Send>) -> MoveCard {
        MoveCard {
            priority,
            tmove,
        }
    }

    pub fn new_from_move(priority: u32, mmove: ESimpleMove) -> MoveCard {
        let tmove = SimpleMove::new(&[mmove]);
        MoveCard::new(priority, tmove)
    }

    pub fn new_from_moves(priority: u32, moves: &[ESimpleMove]) -> MoveCard {
        let tmove = SimpleMove::new(moves);
        MoveCard::new(priority, tmove)
    }
}

#[derive(Debug, Clone)]
pub struct SimpleMove {
    chain: Vec<ESimpleMove>,
}

impl SimpleMove {
    pub fn new(simple_moves: &[ESimpleMove]) -> Box<SimpleMove> {
        Box::from(SimpleMove {
            chain: simple_moves.to_vec(),
        })
    }
}

impl TMove for SimpleMove {
    fn iter(&self) -> Iter<ESimpleMove> {
        self.chain.iter()
    }
    fn box_clone(&self) -> Box<dyn TMove + Send> {
        Box::new((*self).clone())
    }
}