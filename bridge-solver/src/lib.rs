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

pub mod cards;
mod hands;
pub mod types;
mod cache;
mod play;
mod bridge_solver;
mod search;
mod pattern;

pub use cards::Cards;
pub use hands::Hands;
pub use types::{Seat, Suit, NOTRUMP, NUM_SEATS, NUM_SUITS, NUM_RANKS, TOTAL_CARDS, TOTAL_TRICKS};
pub use types::{SPADE, HEART, DIAMOND, CLUB};
pub use types::{WEST, NORTH, EAST, SOUTH};
pub use bridge_solver::{Solver, get_node_count, set_xray_limit, set_no_pruning, set_no_tt, set_no_rank_skip, set_show_perf, order_leads, order_follows, OrderedCards};
pub use search::slow_trump_tricks_opponent;

#[cfg(test)]
mod tests;
