use std::collections::HashSet;

use lazy_static::lazy_static;
use rand::prelude::*;

use crate::api::types::{Cow, CowColor};

// This macro from the lazy_static crate allows the creation of lazily initialized
// read-only global static variables.
lazy_static! {
    pub(crate) static ref COW_NAMES: HashSet<String> = [
        "Arabella", "Bella", "Bessie", "Betty", "Bianca", "Blackjack", "Bossy",
        "Brownie", "Buttercup", "Butterscotch", "Cayenne", "Clarabelle", "Cookie",
        "Daisy", "Domino", "Dottie", "Flossie", "Gertie", "Ginger", "Goldie",
        "Guenevere", "Guinness", "Henrietta", "Maggie", "Marshmallow", "Millie",
        "Minnie", "Muffin", "Nellie", "Oreo", "Peaches", "Penelope", "Penny",
        "Phoebe", "Popcorn", "Princess", "Rosie", "Ruby", "Smokey", "Snowflake",
        "Speckles", "Sprinles", "Sugar", "Sweetie",
    ].iter().map(|name| name.to_string()).collect();

    // vec![...] is the Rust macro for making vectors.
    static ref COW_PHRASES: Vec<String> = vec![
        "Mooo! {} understands.".to_string(),
        "Mooo! {} offers kind words of encouragement.".to_string(),
        "Mooo! {} thinks it's all for the best.".to_string(),
        "Mooo! {} thinks you tried your best.".to_string(),
        "Mooo! {} can't really disagree.".to_string(),
        "Mooo! {} appreciates you making an effort.".to_string(),
    ];
}

pub(crate) fn make_cow(name: &str, id: u32) -> Cow {
    let mut random = thread_rng();
    // x..y is exclusive, x..=y is inclusive.
    // Number literals often need to be annotated for type.
    let color = match random.gen_range(0_u32..=3) {
        0 => CowColor::Black, 1 => CowColor::BlackWithWhitePatches,
        2 => CowColor::Brown, 3 => CowColor::Tan,
        // This match statement needs a catch-all arm because the compiler
        // can't prove that a match on 0..=3 only generates outcomes in that range.
        // Non-exhaustive match statements are invalid.
        _ => unimplemented!(),
    };
    let age = random.gen_range(5_u32..=30);
    let weight = random.gen_range(1300_u32..=1800);
    Cow::new(name, id, color, age, weight)
}

pub(crate) fn make_cow_phrase(name: &str) -> String {
    let mut random = thread_rng();
    let template = COW_PHRASES.choose(&mut random).unwrap();
    // I would normally use format!() here, but it only accepts string literals,
    // so it can evaluate them at compile time.
    template.replace("{}", name)
}
