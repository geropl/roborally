use std::slice::Iter;
use std::fmt;
use std::collections::HashSet;

use crate::roborally::engine::register_engine::{ ESimpleMove, TMove };

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

    pub fn generate_program_deck(&mut self, config: &ProgramCardDeckConfig) -> ProgramCardDeck {
        let mut cards = Vec::with_capacity(config.card_count() as usize);

        let mut priorities: HashSet<u32> = HashSet::new();
        for _ in 0..config.card_count() {
            loop {
                use rand::Rng;
                let prio = self.rng.gen_range(1, 1001);
                if priorities.insert(prio) {
                    break;
                }
            }
        }
        let mut it = priorities.into_iter();

        let add_move_card = |cards: &mut Vec<MoveCard>, it: &mut std::collections::hash_set::IntoIter<u32>, moves: &[ESimpleMove]| {
            cards.push(MoveCard::new_from_moves(cards.len() as u32, it.next().unwrap(), moves));
        };

        for _ in 0..config.count_1_move {
            add_move_card(&mut cards, &mut it, &[ESimpleMove::Forward]);
        }
        for _ in 0..config.count_2_move {
            add_move_card(&mut cards, &mut it, &[ESimpleMove::Forward, ESimpleMove::Forward]);
        }
        for _ in 0..config.count_3_move {
            add_move_card(&mut cards, &mut it, &[ESimpleMove::Forward, ESimpleMove::Forward, ESimpleMove::Forward]);
        }
        for _ in 0..config.count_back_up {
            add_move_card(&mut cards, &mut it, &[ESimpleMove::Backward]);
        }
        for _ in 0..config.count_turn_left {
            add_move_card(&mut cards, &mut it, &[ESimpleMove::TurnLeft]);
        }
        for _ in 0..config.count_turn_right {
            add_move_card(&mut cards, &mut it, &[ESimpleMove::TurnRight]);
        }
        for _ in 0..config.count_uturn {
            add_move_card(&mut cards, &mut it, &[ESimpleMove::UTurn]);
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
    
    pub fn add_cards(&self, new_cards: Vec<MoveCard>) -> ProgramCardDeck {
        let mut cards = self.cards.clone();
        cards.extend(new_cards);
        ProgramCardDeck {
            cards,
        }
    }

    pub fn take_random_cards(&self, amount: u32) -> (ProgramCardDeck, Vec<MoveCard>) {
        use rand::seq::index;

        let amount = amount as usize;
        let mut cards = self.cards.clone();
        let mut rng = rand::thread_rng();
        let mut indeces = index::sample(&mut rng, cards.len(), amount).into_vec();

        let mut chosen_cards = Vec::with_capacity(amount);
        indeces.sort();
        indeces.reverse();  // important because we want to remove items, which changes all following indexes
        for i in indeces {
            chosen_cards.push(cards.remove(i));
        }

        (ProgramCardDeck { cards }, chosen_cards)
    }
}

pub type MoveCardID = u32;

#[derive(Debug, Clone)]
pub struct MoveCard {
    pub id: MoveCardID,
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
    pub fn new(id: MoveCardID, priority: u32, tmove: Box<dyn TMove + Send>) -> MoveCard {
        MoveCard {
            id,
            priority,
            tmove,
        }
    }

    pub fn new_from_moves(id: MoveCardID, priority: u32, moves: &[ESimpleMove]) -> MoveCard {
        let tmove = SimpleMove::new(moves);
        MoveCard::new(id, priority, tmove)
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