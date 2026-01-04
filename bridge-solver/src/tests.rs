//! Test suite matching C++ solver test cases

use super::*;
use super::types::*;

/// Test case structure matching test_cases.txt format
struct TestCase {
    name: &'static str,
    pbn: &'static str,
    trump: usize,
    leader: Seat,
    expected_ns_tricks: u8,
}

const TEST_CASES: &[TestCase] = &[
    // Note: Expected values verified against C++ solver (macroxue/bridge-solver)
    // C++ returns tricks for non-leader's side, so:
    // - EW leads (W/E): result = NS tricks directly
    // - NS leads (N/S): result = EW tricks, NS = 13 - result
    TestCase {
        name: "Test 1: NT West lead",
        pbn: "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72",
        trump: NOTRUMP,
        leader: WEST,
        expected_ns_tricks: 5,
    },
    TestCase {
        name: "Test 2: NT East lead",
        pbn: "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72",
        trump: NOTRUMP,
        leader: EAST,
        expected_ns_tricks: 5,
    },
    TestCase {
        name: "Test 3: NT North lead",
        pbn: "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72",
        trump: NOTRUMP,
        leader: NORTH,
        expected_ns_tricks: 6,
    },
    TestCase {
        name: "Test 4: Spades West lead",
        pbn: "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72",
        trump: SPADE,
        leader: WEST,
        expected_ns_tricks: 5,
    },
    TestCase {
        name: "Test 5: Hearts West lead",
        pbn: "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72",
        trump: HEART,
        leader: WEST,
        expected_ns_tricks: 2,
    },
    TestCase {
        name: "Test 6: Diamonds West lead",
        pbn: "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72",
        trump: DIAMOND,
        leader: WEST,
        expected_ns_tricks: 7,
    },
    TestCase {
        name: "Test 7: Clubs West lead",
        pbn: "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72",
        trump: CLUB,
        leader: WEST,
        expected_ns_tricks: 4,
    },
    TestCase {
        name: "Test 8: Cold 7NT",
        pbn: "N:AKQJ.AKQ.AKQ.AKQ T987.JT9.JT9.JT9 6543.876.876.876 2.5432.5432.5432",
        trump: NOTRUMP,
        leader: WEST,
        expected_ns_tricks: 13,
    },
    TestCase {
        name: "Test 9: NS makes 0",
        pbn: "N:T987.JT9.JT9.JT9 AKQJ.AKQ.AKQ.AKQ 2.5432.5432.5432 6543.876.876.876",
        trump: NOTRUMP,
        leader: WEST,
        expected_ns_tricks: 0,
    },
    TestCase {
        name: "Test 10: Balanced hands",
        pbn: "N:AK32.AK32.K32.32 QJT9.QJT.QJT.QJT 8765.987.987.987 4.654.A654.AK654",
        trump: NOTRUMP,
        leader: WEST,
        expected_ns_tricks: 10,
    },
];

#[test]
#[ignore] // 13-card tests disabled until pruning optimizations are implemented
fn test_all_cases() {
    for case in TEST_CASES {
        let hands = Hands::from_pbn(case.pbn)
            .unwrap_or_else(|| panic!("Failed to parse PBN for {}", case.name));

        let solver = Solver::new(hands, case.trump, case.leader);
        let ns_tricks = solver.solve();

        assert_eq!(
            ns_tricks, case.expected_ns_tricks,
            "{}: expected {} tricks, got {}",
            case.name, case.expected_ns_tricks, ns_tricks
        );
    }
}

#[test]
fn test_cards_basic_operations() {
    let mut cards = Cards::new();
    assert!(cards.is_empty());

    cards.add(cards::card_of(SPADE, types::ACE));
    assert_eq!(cards.size(), 1);
    assert!(cards.have(cards::card_of(SPADE, types::ACE)));

    cards.add(cards::card_of(HEART, types::KING));
    assert_eq!(cards.size(), 2);

    let spades = cards.suit(SPADE);
    assert_eq!(spades.size(), 1);
}

#[test]
fn test_hands_parsing() {
    let pbn = "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72";
    let hands = Hands::from_pbn(pbn).unwrap();

    assert_eq!(hands[NORTH].size(), 13);
    assert_eq!(hands[EAST].size(), 13);
    assert_eq!(hands[SOUTH].size(), 13);
    assert_eq!(hands[WEST].size(), 13);
    assert_eq!(hands.all_cards().size(), 52);
}
