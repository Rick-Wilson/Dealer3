//! Test follow ordering against C++ reference implementation
//! Run: cargo test --example test_follow_order
//!
//! Reference values come from the C++ gen_follow_order_refs.cc

use dealer_dds::solver2::{
    Hands, Cards, WEST, NORTH, EAST, SOUTH, NOTRUMP, SPADE, HEART, DIAMOND, CLUB,
    order_follows,
};
use dealer_dds::solver2::cards::{name_of, card_of, suit_of};
use dealer_dds::solver2::types::Suit;

/// Convert ordered cards to a string like "SA SQ HJ H6"
fn ordered_to_string(ordered: impl Iterator<Item = usize>) -> String {
    ordered.map(|c| name_of(c)).collect::<Vec<_>>().join(" ")
}

/// Check if card1 beats card2 given the trump suit
fn wins_over(c1: usize, c2: usize, trump: Suit) -> bool {
    let s1 = suit_of(c1);
    let s2 = suit_of(c2);
    if s1 == s2 {
        c1 < c2  // Lower index = higher rank
    } else {
        s1 == trump  // Only win if we trumped
    }
}

// Deal: quick_test_8
// North: AKQ J6 KJ 9
// West:  65 AK4 AQ T
// East:  J7 QT9 T AK
// South: 98 87 96 QJ

fn get_quick_test_8_hands() -> Hands {
    Hands::from_solver_format(
        "AKQ J6 KJ 9",   // North
        "65 AK4 AQ T",   // West
        "J7 QT9 T AK",   // East
        "98 87 96 QJ"    // South
    ).unwrap()
}

/// Get playable cards - must follow suit if possible
fn get_playable(hands: &Hands, seat: usize, lead_suit: Suit) -> Cards {
    let suit_cards = hands[seat].suit(lead_suit);
    if suit_cards.is_empty() {
        hands[seat]
    } else {
        suit_cards
    }
}

// ============================================================================
// Following suit tests
// ============================================================================

#[test]
fn test_follow_west_leads_s6_east_follows_2nd_nt() {
    // West leads S6, East follows (2nd seat) - East has J7, can beat 6
    let hands = get_quick_test_8_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(SPADE, 4),  // winning_card: S6
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SJ S7");
}

#[test]
fn test_follow_west_leads_ha_east_follows_2nd_nt() {
    // West leads HA, East follows (2nd seat) - can't beat A
    let hands = get_quick_test_8_hands();
    let lead_suit = HEART;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(HEART, 12),  // winning_card: HA
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "H9 HT HQ");
}

#[test]
fn test_follow_north_leads_sa_east_follows_2nd_nt() {
    // North leads SA, East follows (2nd seat) - can't beat A
    let hands = get_quick_test_8_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(SPADE, 12),  // winning_card: SA
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S7 SJ");
}

#[test]
fn test_follow_west_leads_da_south_follows_4th_nt() {
    // West leads DA, South follows (4th seat), North winning with K
    let hands = get_quick_test_8_hands();
    let lead_suit = DIAMOND;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(DIAMOND, 11),  // winning_card: DK
        3,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D6 D9");
}

#[test]
fn test_follow_west_leads_ct_east_follows_2nd_nt() {
    // West leads CT, East follows (2nd seat) - East has AK, can beat T
    let hands = get_quick_test_8_hands();
    let lead_suit = CLUB;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(CLUB, 8),  // winning_card: CT
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "CK CA");
}

#[test]
fn test_follow_north_leads_hj_east_follows_2nd_nt() {
    // North leads HJ, East follows (2nd seat) - East has QT9
    let hands = get_quick_test_8_hands();
    let lead_suit = HEART;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(HEART, 9),  // winning_card: HJ
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "H9 HT HQ");
}

#[test]
fn test_follow_west_leads_hk_north_follows_2nd_nt() {
    // West leads HK, North follows (2nd seat) - North has J6
    let hands = get_quick_test_8_hands();
    let lead_suit = HEART;
    let playable = get_playable(&hands, NORTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        NORTH,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(HEART, 11),  // winning_card: HK
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "H6 HJ");
}

// ============================================================================
// Partner winning tests
// ============================================================================

#[test]
fn test_follow_west_leads_s5_south_follows_4th_pd_wins_nt() {
    // West leads S5, South follows (4th seat), partner East won with J
    let hands = get_quick_test_8_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        NOTRUMP,
        lead_suit,
        EAST,  // winning_seat (partner)
        card_of(SPADE, 9),  // winning_card: SJ
        3,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S8 S9");
}

// ============================================================================
// Ruff/discard tests
// ============================================================================

#[test]
fn test_follow_west_leads_ct_north_ruffs_hearts() {
    // Hearts trump: West leads club, North has only C9
    let hands = get_quick_test_8_hands();
    let lead_suit = CLUB;
    let playable = get_playable(&hands, NORTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        NORTH,
        HEART,
        lead_suit,
        WEST,  // winning_seat
        card_of(CLUB, 8),  // winning_card: CT
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, HEART),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "C9");
}

#[test]
fn test_follow_west_leads_s6_south_follows_spades() {
    // Spades trump: West leads S6, South follows with trump (has S98)
    let hands = get_quick_test_8_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        SPADE,
        lead_suit,
        WEST,  // winning_seat
        card_of(SPADE, 4),  // winning_card: S6
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, SPADE),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S8 S9");
}

#[test]
fn test_follow_north_leads_dk_east_ruffs_hearts() {
    // Hearts trump: North leads DK, East has DT
    let hands = get_quick_test_8_hands();
    let lead_suit = DIAMOND;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        HEART,
        lead_suit,
        NORTH,  // winning_seat
        card_of(DIAMOND, 11),  // winning_card: DK
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, HEART),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "DT");
}

// ============================================================================
// 3rd seat tests
// ============================================================================

#[test]
fn test_follow_west_leads_s6_south_follows_3rd_nt() {
    // West leads S6, North played SA, South follows 3rd seat
    let hands = get_quick_test_8_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(SPADE, 12),  // winning_card: SA
        2,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S8 S9");
}

// ============================================================================
// Quick Test 6 tests (with voids)
// ============================================================================

// Deal: quick_test_6
// North: KQ J6 KJ - (void in clubs)
// West:  5 K4 AQ T
// East:  J7 QT9 - K (void in diamonds)
// South: 98 87 96 - (void in clubs)

fn get_quick_test_6_hands() -> Hands {
    Hands::from_solver_format(
        "KQ J6 KJ -",   // North (void in clubs)
        "5 K4 AQ T",    // West
        "J7 QT9 - K",   // East (void in diamonds)
        "98 87 96 -"    // South (void in clubs)
    ).unwrap()
}

// ============================================================================
// Following suit tests (quick_test_6)
// ============================================================================

#[test]
fn test_follow_6_west_leads_s5_east_follows_2nd_nt() {
    // West leads S5, East follows (2nd seat) - East has J7
    let hands = get_quick_test_6_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(SPADE, 3),  // winning_card: S5
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SJ S7");
}

#[test]
fn test_follow_6_west_leads_hk_north_follows_2nd_nt() {
    // West leads HK, North follows (2nd seat) - North has J6, can't beat K
    let hands = get_quick_test_6_hands();
    let lead_suit = HEART;
    let playable = get_playable(&hands, NORTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        NORTH,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(HEART, 11),  // winning_card: HK
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "H6 HJ");
}

#[test]
fn test_follow_6_north_leads_sk_east_follows_2nd_nt() {
    // North leads SK, East follows (2nd seat) - can't beat K
    let hands = get_quick_test_6_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(SPADE, 11),  // winning_card: SK
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S7 SJ");
}

#[test]
fn test_follow_6_west_leads_da_south_follows_4th_nt() {
    // West leads DA, South follows (4th seat), North winning with K
    let hands = get_quick_test_6_hands();
    let lead_suit = DIAMOND;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(DIAMOND, 11),  // winning_card: DK
        3,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D6 D9");
}

#[test]
fn test_follow_6_north_leads_hj_east_follows_2nd_nt() {
    // North leads HJ, East follows (2nd seat) - East has QT9
    let hands = get_quick_test_6_hands();
    let lead_suit = HEART;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(HEART, 9),  // winning_card: HJ
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "H9 HT HQ");
}

// ============================================================================
// Void/Discard tests (quick_test_6)
// ============================================================================

#[test]
fn test_follow_6_west_leads_da_east_discards_nt() {
    // NT: West leads diamond, East is void - must discard (East has J7 QT9 K)
    let hands = get_quick_test_6_hands();
    let lead_suit = DIAMOND;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(DIAMOND, 12),  // winning_card: DA
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "CK H9 HT HQ S7 SJ");
}

#[test]
fn test_follow_6_west_leads_ct_north_discards_nt() {
    // NT: West leads club, North is void - must discard (North has KQ J6 KJ)
    let hands = get_quick_test_6_hands();
    let lead_suit = CLUB;
    let playable = get_playable(&hands, NORTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        NORTH,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(CLUB, 8),  // winning_card: CT
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "DJ DK H6 HJ SQ SK");
}

#[test]
fn test_follow_6_west_leads_ct_south_discards_nt() {
    // NT: West leads club, South is void - must discard (South has 98 87 96)
    let hands = get_quick_test_6_hands();
    let lead_suit = CLUB;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        NOTRUMP,
        lead_suit,
        WEST,  // winning_seat
        card_of(CLUB, 8),  // winning_card: CT
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D6 D9 H7 H8 S8 S9");
}

// ============================================================================
// Ruff tests with voids (quick_test_6)
// ============================================================================

#[test]
fn test_follow_6_west_leads_da_east_ruffs_hearts() {
    // Hearts trump: West leads diamond, East void - ruff with hearts
    let hands = get_quick_test_6_hands();
    let lead_suit = DIAMOND;
    let playable = get_playable(&hands, EAST, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        EAST,
        HEART,
        lead_suit,
        WEST,  // winning_seat
        card_of(DIAMOND, 12),  // winning_card: DA
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, HEART),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "CK H9 HT HQ S7 SJ");
}

#[test]
fn test_follow_6_west_leads_ct_north_ruffs_spades() {
    // Spades trump: West leads club, North void - can ruff with spades (North has KQ in spades)
    let hands = get_quick_test_6_hands();
    let lead_suit = CLUB;
    let playable = get_playable(&hands, NORTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        NORTH,
        SPADE,
        lead_suit,
        WEST,  // winning_seat
        card_of(CLUB, 8),  // winning_card: CT
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, SPADE),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SQ DJ DK H6 HJ SK");
}

#[test]
fn test_follow_6_west_leads_ct_south_ruffs_diamonds() {
    // Diamonds trump: West leads club, South void - can ruff with diamonds (South has 96)
    let hands = get_quick_test_6_hands();
    let lead_suit = CLUB;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        DIAMOND,
        lead_suit,
        WEST,  // winning_seat
        card_of(CLUB, 8),  // winning_card: CT
        1,  // card_in_trick
        |c1, c2| wins_over(c1, c2, DIAMOND),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D6 D9 H7 H8 S8 S9");
}

// ============================================================================
// 3rd seat tests (quick_test_6)
// ============================================================================

#[test]
fn test_follow_6_west_leads_s5_south_follows_3rd_nt() {
    // West leads S5, North played K, South follows 3rd seat
    let hands = get_quick_test_6_hands();
    let lead_suit = SPADE;
    let playable = get_playable(&hands, SOUTH, lead_suit);
    let ordered = order_follows(
        playable,
        &hands,
        SOUTH,
        NOTRUMP,
        lead_suit,
        NORTH,  // winning_seat
        card_of(SPADE, 11),  // winning_card: SK
        2,  // card_in_trick
        |c1, c2| wins_over(c1, c2, NOTRUMP),
    );
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S8 S9");
}

fn main() {
    println!("Running follow order tests...\n");
    println!("Use 'cargo test --example test_follow_order' to run all tests");
}
