use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::rng;
use crate::database::models::hand_card::{CardType, HandCard};
use crate::database::models::hand_card::CardType::{Other, Primary, Secondary};
use crate::database::models::skill::Skill;
use crate::mechanics::spell_cards::special_decks::{get_basics, get_curses, InnerCards};

const HAND_LIMIT: u8 = 7;

pub fn draw_cards(
    cards: Vec<Skill>,
    hand: Vec<HandCard>,
    amount: isize,
) -> Vec<InnerCards> {
    let cards: Vec<InnerCards> = cards.iter().map(|c| c.into()).collect();
    let curses: Vec<InnerCards> = get_curses();
    let basic: Vec<InnerCards> = get_basics();

    //                              Primary,          Secondary,           Other
    let mut weight: [f32;3] = [     42.5,               15.0,              42.5];
    let weighted_cards: [(CardType, &[InnerCards]); 3] = [
        (Primary, cards.as_slice()),
        (Secondary, curses.as_slice()),
        (Other, basic.as_slice())
    ];

    let hand_size = hand.len() as isize;
    for card in hand {

        match card.card_type {
            Primary => weight[0] = (weight[0] - 3.0f32).max(0.1f32),
            Secondary => weight[1] = (weight[1] - 3.0f32).max(0.1f32),
            Other => weight[2] = (weight[2] - 3.0f32).max(0.1f32)
        }
    }
    let mut rng = rng();
    let total_amount = amount.min((HAND_LIMIT as isize) - hand_size);

    if cards.is_empty() {
        weight[0] = 0.0;
    }

    if total_amount <= 0 {
        return vec![];
    }

    let mut new_cards = Vec::with_capacity(total_amount as usize);
    for _ in 0..total_amount {
        let dist = WeightedIndex::new(&weight).unwrap();

        let index = dist.sample(&mut rng);
        let selected_collection = weighted_cards[index].clone();

        let mut card_weight = vec![];
        for card in selected_collection.1 {
            card_weight.push(card.cost)
        }

        let dist = WeightedIndex::new(&card_weight).unwrap();
        let index = dist.sample(&mut rng);

        let a = selected_collection.1[index].clone();
        new_cards.push(a);

        match selected_collection.0 {
            Primary => weight[0] = (weight[0] - 3.0f32).max(0.1f32),
            Secondary => weight[1] = (weight[1] - 3.0f32).max(0.1f32),
            Other => weight[2] = (weight[2] - 3.0f32).max(0.1f32)
        }

    }

    new_cards
}