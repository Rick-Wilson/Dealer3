//! Test lead ordering against C++ reference implementation
//! Run: cargo test --example test_lead_order
//!
//! Reference values come from the C++ gen_lead_order_refs.cc

use dealer_dds::solver2::{
    Hands, WEST, NORTH, EAST, SOUTH, NOTRUMP, SPADE, HEART, DIAMOND, CLUB,
    order_leads,
};
use dealer_dds::solver2::cards::name_of;

/// Convert ordered cards to a string like "SA SQ HJ H6"
fn ordered_to_string(ordered: impl Iterator<Item = usize>) -> String {
    ordered.map(|c| name_of(c)).collect::<Vec<_>>().join(" ")
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

// ============================================================================
// NT Tests
// ============================================================================

#[test]
fn test_nt_west_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HA H4 DA DQ CT S6 S5 HK",
        "NT West leads mismatch!\n  Got:      {}\n  Expected: HA H4 DA DQ CT S6 S5 HK", result);
}

#[test]
fn test_nt_north_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SA SQ HJ H6 C9 SK DK DJ",
        "NT North leads mismatch!\n  Got:      {}\n  Expected: SA SQ HJ H6 C9 SK DK DJ", result);
}

#[test]
fn test_nt_east_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 DT CA CK SJ S7 HT",
        "NT East leads mismatch!\n  Got:      {}\n  Expected: HQ H9 DT CA CK SJ S7 HT", result);
}

#[test]
fn test_nt_south_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D9 D6 S9 S8 H8 H7 CQ CJ",
        "NT South leads mismatch!\n  Got:      {}\n  Expected: D9 D6 S9 S8 H8 H7 CQ CJ", result);
}

// ============================================================================
// Spades Trump Tests
// ============================================================================

#[test]
fn test_spades_west_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HA H4 DA DQ CT S6 S5 HK",
        "Spades West leads mismatch!\n  Got:      {}\n  Expected: HA H4 DA DQ CT S6 S5 HK", result);
}

#[test]
fn test_spades_north_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HJ H6 C9 DK DJ SA SQ SK",
        "Spades North leads mismatch!\n  Got:      {}\n  Expected: HJ H6 C9 DK DJ SA SQ SK", result);
}

#[test]
fn test_spades_east_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 DT CA CK SJ S7 HT",
        "Spades East leads mismatch!\n  Got:      {}\n  Expected: HQ H9 DT CA CK SJ S7 HT", result);
}

#[test]
fn test_spades_south_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D9 D6 H8 H7 CQ CJ S9 S8",
        "Spades South leads mismatch!\n  Got:      {}\n  Expected: D9 D6 H8 H7 CQ CJ S9 S8", result);
}

// ============================================================================
// Hearts Trump Tests
// ============================================================================

#[test]
fn test_hearts_west_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "DA DQ CT S6 S5 HA H4 HK",
        "Hearts West leads mismatch!\n  Got:      {}\n  Expected: DA DQ CT S6 S5 HA H4 HK", result);
}

#[test]
fn test_hearts_north_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SA SQ C9 DK DJ HJ H6 SK",
        "Hearts North leads mismatch!\n  Got:      {}\n  Expected: SA SQ C9 DK DJ HJ H6 SK", result);
}

#[test]
fn test_hearts_east_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "DT CA CK SJ S7 HQ H9 HT",
        "Hearts East leads mismatch!\n  Got:      {}\n  Expected: DT CA CK SJ S7 HQ H9 HT", result);
}

#[test]
fn test_hearts_south_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D9 D6 S9 S8 CQ CJ H8 H7",
        "Hearts South leads mismatch!\n  Got:      {}\n  Expected: D9 D6 S9 S8 CQ CJ H8 H7", result);
}

// ============================================================================
// Diamonds Trump Tests
// ============================================================================

#[test]
fn test_diamonds_west_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HA H4 CT S6 S5 DA DQ HK",
        "Diamonds West leads mismatch!\n  Got:      {}\n  Expected: HA H4 CT S6 S5 DA DQ HK", result);
}

#[test]
fn test_diamonds_north_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SA SQ HJ H6 C9 DK DJ SK",
        "Diamonds North leads mismatch!\n  Got:      {}\n  Expected: SA SQ HJ H6 C9 DK DJ SK", result);
}

#[test]
fn test_diamonds_east_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 CA CK SJ S7 DT HT",
        "Diamonds East leads mismatch!\n  Got:      {}\n  Expected: HQ H9 CA CK SJ S7 DT HT", result);
}

#[test]
fn test_diamonds_south_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S9 S8 H8 H7 CQ CJ D9 D6",
        "Diamonds South leads mismatch!\n  Got:      {}\n  Expected: S9 S8 H8 H7 CQ CJ D9 D6", result);
}

// ============================================================================
// Clubs Trump Tests
// ============================================================================

#[test]
fn test_clubs_west_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HA H4 DA DQ S6 S5 CT HK",
        "Clubs West leads mismatch!\n  Got:      {}\n  Expected: HA H4 DA DQ S6 S5 CT HK", result);
}

#[test]
fn test_clubs_north_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SA SQ HJ H6 DK DJ C9 SK",
        "Clubs North leads mismatch!\n  Got:      {}\n  Expected: SA SQ HJ H6 DK DJ C9 SK", result);
}

#[test]
fn test_clubs_east_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 DT SJ S7 CA CK HT",
        "Clubs East leads mismatch!\n  Got:      {}\n  Expected: HQ H9 DT SJ S7 CA CK HT", result);
}

#[test]
fn test_clubs_south_leads() {
    let hands = get_quick_test_8_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D9 D6 S9 S8 H8 H7 CQ CJ",
        "Clubs South leads mismatch!\n  Got:      {}\n  Expected: D9 D6 S9 S8 H8 H7 CQ CJ", result);
}

// ============================================================================
// quick_test_6 (6 tricks, with voids)
// North: KQ J6 KJ (void in clubs)
// West:  5 K4 AQ T
// East:  J7 QT9  K (void in diamonds)
// South: 98 87 96 (void in clubs)
// ============================================================================

fn get_quick_test_6_hands() -> Hands {
    Hands::from_solver_format(
        "KQ J6 KJ -",   // North (void in clubs)
        "5 K4 AQ T",    // West
        "J7 QT9 - K",   // East (void in diamonds)
        "98 87 96 -"    // South (void in clubs)
    ).unwrap()
}

// quick_test_6 NT Tests

#[test]
fn test_quick_test_6_nt_west_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HK H4 DA DQ S5 CT");
}

#[test]
fn test_quick_test_6_nt_north_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SK SQ HJ H6 DK DJ");
}

#[test]
fn test_quick_test_6_nt_east_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 SJ S7 CK HT");
}

#[test]
fn test_quick_test_6_nt_south_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, NOTRUMP, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "D9 D6 S9 S8 H8 H7");
}

// quick_test_6 Spades Tests

#[test]
fn test_quick_test_6_spades_west_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HK H4 DA DQ S5 CT");
}

#[test]
fn test_quick_test_6_spades_north_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HJ H6 SK SQ DK DJ");
}

#[test]
fn test_quick_test_6_spades_east_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 SJ S7 HT CK");
}

#[test]
fn test_quick_test_6_spades_south_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, SPADE, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "H8 H7 S9 S8 D9 D6");
}

// quick_test_6 Hearts Tests

#[test]
fn test_quick_test_6_hearts_west_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "DA DQ S5 HK H4 CT");
}

#[test]
fn test_quick_test_6_hearts_north_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SK SQ HJ H6 DK DJ");
}

#[test]
fn test_quick_test_6_hearts_east_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SJ S7 HQ H9 HT CK");
}

#[test]
fn test_quick_test_6_hearts_south_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, HEART, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S9 S8 H8 H7 D9 D6");
}

// quick_test_6 Diamonds Tests

#[test]
fn test_quick_test_6_diamonds_west_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HK H4 S5 DA DQ CT");
}

#[test]
fn test_quick_test_6_diamonds_north_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SK SQ HJ H6 DK DJ");
}

#[test]
fn test_quick_test_6_diamonds_east_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 SJ S7 HT CK");
}

#[test]
fn test_quick_test_6_diamonds_south_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, DIAMOND, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S9 S8 H8 H7 D9 D6");
}

// quick_test_6 Clubs Tests

#[test]
fn test_quick_test_6_clubs_west_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[WEST], &hands, WEST, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HK H4 DA DQ S5 CT");
}

#[test]
fn test_quick_test_6_clubs_north_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[NORTH], &hands, NORTH, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "SK SQ HJ H6 DK DJ");
}

#[test]
fn test_quick_test_6_clubs_east_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[EAST], &hands, EAST, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "HQ H9 SJ S7 CK HT");
}

#[test]
fn test_quick_test_6_clubs_south_leads() {
    let hands = get_quick_test_6_hands();
    let all_cards = hands.all_cards();
    let ordered = order_leads(hands[SOUTH], &hands, SOUTH, CLUB, all_cards);
    let result = ordered_to_string(ordered.iter());
    assert_eq!(result, "S9 S8 H8 H7 D9 D6");
}

fn main() {
    println!("Running lead order tests...\n");
    println!("Use 'cargo test --example test_lead_order' to run all tests");
}
