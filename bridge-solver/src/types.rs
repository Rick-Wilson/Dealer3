//! Core type definitions matching the C++ solver

/// Suits: SPADE=0, HEART=1, DIAMOND=2, CLUB=3
pub type Suit = usize;
pub const SPADE: Suit = 0;
pub const HEART: Suit = 1;
pub const DIAMOND: Suit = 2;
pub const CLUB: Suit = 3;
pub const NUM_SUITS: usize = 4;
pub const NOTRUMP: usize = NUM_SUITS;

/// Ranks: TWO=0, ..., TEN=8, JACK=9, QUEEN=10, KING=11, ACE=12
pub type Rank = usize;
pub const TWO: Rank = 0;
pub const THREE: Rank = 1;
pub const FOUR: Rank = 2;
pub const FIVE: Rank = 3;
pub const SIX: Rank = 4;
pub const SEVEN: Rank = 5;
pub const EIGHT: Rank = 6;
pub const NINE: Rank = 7;
pub const TEN: Rank = 8;
pub const JACK: Rank = 9;
pub const QUEEN: Rank = 10;
pub const KING: Rank = 11;
pub const ACE: Rank = 12;
pub const NUM_RANKS: usize = 13;

/// Seats: WEST=0, NORTH=1, EAST=2, SOUTH=3
pub type Seat = usize;
pub const WEST: Seat = 0;
pub const NORTH: Seat = 1;
pub const EAST: Seat = 2;
pub const SOUTH: Seat = 3;
pub const NUM_SEATS: usize = 4;

pub const TOTAL_TRICKS: usize = NUM_RANKS;
pub const TOTAL_CARDS: usize = NUM_RANKS * NUM_SUITS;

/// Check if seat is NS (North or South)
#[inline]
pub fn is_ns(seat: Seat) -> bool {
    seat & 1 != 0
}

/// Get partner seat
#[inline]
pub fn partner(seat: Seat) -> Seat {
    (seat + 2) % NUM_SEATS
}

/// Get left-hand opponent
#[inline]
pub fn left_hand_opp(seat: Seat) -> Seat {
    (seat + 1) % NUM_SEATS
}

/// Get right-hand opponent
#[inline]
pub fn right_hand_opp(seat: Seat) -> Seat {
    (seat + 3) % NUM_SEATS
}

/// Get next seat (clockwise)
#[inline]
pub fn next_seat(seat: Seat) -> Seat {
    (seat + 1) % NUM_SEATS
}

/// Get seat name
pub fn seat_name(seat: Seat) -> &'static str {
    const NAMES: [&str; 4] = ["West", "North", "East", "South"];
    NAMES[seat]
}

/// Get seat letter
pub fn seat_letter(seat: Seat) -> char {
    seat_name(seat).chars().next().unwrap()
}

/// Get suit name
pub fn suit_name(suit: Suit) -> &'static str {
    const NAMES: [&str; 5] = ["Spade", "Heart", "Diamond", "Club", "NoTrump"];
    NAMES[suit]
}

/// Get rank name (character)
pub fn rank_name(rank: Rank) -> char {
    const NAMES: [char; 13] = [
        '2', '3', '4', '5', '6', '7', '8', '9', 'T', 'J', 'Q', 'K', 'A',
    ];
    NAMES[rank]
}

/// Convert character to suit
pub fn char_to_suit(c: char) -> Option<Suit> {
    match c.to_ascii_uppercase() {
        'S' => Some(SPADE),
        'H' => Some(HEART),
        'D' => Some(DIAMOND),
        'C' => Some(CLUB),
        'N' => Some(NOTRUMP),
        _ => None,
    }
}

/// Convert character to rank
pub fn char_to_rank(c: char) -> Option<Rank> {
    match c.to_ascii_uppercase() {
        '2' => Some(TWO),
        '3' => Some(THREE),
        '4' => Some(FOUR),
        '5' => Some(FIVE),
        '6' => Some(SIX),
        '7' => Some(SEVEN),
        '8' => Some(EIGHT),
        '9' => Some(NINE),
        'T' | '1' => Some(TEN),
        'J' => Some(JACK),
        'Q' => Some(QUEEN),
        'K' => Some(KING),
        'A' => Some(ACE),
        _ => None,
    }
}

/// Convert character to seat
pub fn char_to_seat(c: char) -> Option<Seat> {
    match c.to_ascii_uppercase() {
        'W' => Some(WEST),
        'N' => Some(NORTH),
        'E' => Some(EAST),
        'S' => Some(SOUTH),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ns() {
        assert!(!is_ns(WEST));
        assert!(is_ns(NORTH));
        assert!(!is_ns(EAST));
        assert!(is_ns(SOUTH));
    }

    #[test]
    fn test_partner() {
        assert_eq!(partner(WEST), EAST);
        assert_eq!(partner(NORTH), SOUTH);
        assert_eq!(partner(EAST), WEST);
        assert_eq!(partner(SOUTH), NORTH);
    }

    #[test]
    fn test_char_to_suit() {
        assert_eq!(char_to_suit('S'), Some(SPADE));
        assert_eq!(char_to_suit('h'), Some(HEART));
        assert_eq!(char_to_suit('N'), Some(NOTRUMP));
        assert_eq!(char_to_suit('X'), None);
    }

    #[test]
    fn test_char_to_rank() {
        assert_eq!(char_to_rank('A'), Some(ACE));
        assert_eq!(char_to_rank('T'), Some(TEN));
        assert_eq!(char_to_rank('2'), Some(TWO));
        assert_eq!(char_to_rank('X'), None);
    }
}
