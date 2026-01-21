//! Bridge Double-Dummy Solver - Port of macroxue/bridge-solver
//!
//! This is a faithful Rust port of the C++ solver from:
//! https://github.com/macroxue/bridge-solver
//!
//! The algorithm uses:
//! - Alpha-beta search with MTD(f) driver
//! - Pattern-based transposition table with hierarchical bounds caching
//! - Move ordering heuristics for efficient pruning
//! - Fast trick estimation for early cutoffs

mod bridge_solver;
mod cache;
pub mod cards;
mod hands;
mod pattern;
mod play;
mod search;
pub mod types;

pub use bridge_solver::{
    get_node_count, order_follows, order_leads, set_no_pruning, set_no_rank_skip, set_no_tt,
    set_show_perf, set_xray_limit, OrderedCards, PartialTrick, PlayedCard, Solver,
};
pub use cards::Cards;
pub use hands::Hands;
pub use pattern::PatternCache;
pub use search::{slow_trump_tricks_opponent, CutoffCache};
pub use types::{Seat, Suit, NOTRUMP, NUM_RANKS, NUM_SEATS, NUM_SUITS, TOTAL_CARDS, TOTAL_TRICKS};
pub use types::{CLUB, DIAMOND, HEART, SPADE};
pub use types::{EAST, NORTH, SOUTH, WEST};

#[cfg(test)]
mod tests;
