//! Test DD solver against Bridge Composer results
//!
//! Tests multiple deals from north_9tricks_nt-bridge_composer.pbn

use dealer_core::{Card, Deal, Hand, Position, Rank, Suit};
use dealer_dds::{Denomination, DoubleDummySolver};

fn parse_hand(s: &str) -> Hand {
    let suits_str: Vec<&str> = s.split('.').collect();
    let suits = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];
    let mut hand = Hand::new();

    for (suit_idx, &suit_str) in suits_str.iter().enumerate() {
        let suit = suits[suit_idx];
        for c in suit_str.chars() {
            let rank = match c {
                'A' => Rank::Ace,
                'K' => Rank::King,
                'Q' => Rank::Queen,
                'J' => Rank::Jack,
                'T' => Rank::Ten,
                '9' => Rank::Nine,
                '8' => Rank::Eight,
                '7' => Rank::Seven,
                '6' => Rank::Six,
                '5' => Rank::Five,
                '4' => Rank::Four,
                '3' => Rank::Three,
                '2' => Rank::Two,
                _ => panic!("Invalid rank: {}", c),
            };
            hand.add_card(Card::new(suit, rank));
        }
    }
    hand
}

fn parse_deal(n: &str, e: &str, s: &str, w: &str) -> Deal {
    let mut deal = Deal::new();
    *deal.hand_mut(Position::North) = parse_hand(n);
    *deal.hand_mut(Position::East) = parse_hand(e);
    *deal.hand_mut(Position::South) = parse_hand(s);
    *deal.hand_mut(Position::West) = parse_hand(w);
    deal
}

struct TestCase {
    name: &'static str,
    deal: Deal,
    // Expected: [N, E, S, W] for each denomination [NT, S, H, D, C]
    expected: [[u8; 4]; 5],
}

fn test_deal(tc: &TestCase) -> bool {
    let solver = DoubleDummySolver::new(tc.deal.clone());

    println!("\n{}", tc.name);
    println!("{}", "=".repeat(tc.name.len()));
    println!("\nOur Results vs Expected (Bridge Composer):");
    println!("       NT        S        H        D        C");

    let mut all_match = true;
    let denoms = [
        Denomination::NoTrump,
        Denomination::Spades,
        Denomination::Hearts,
        Denomination::Diamonds,
        Denomination::Clubs,
    ];

    for (pos_idx, pos) in [Position::North, Position::East, Position::South, Position::West]
        .iter()
        .enumerate()
    {
        let pos_char = match pos {
            Position::North => 'N',
            Position::East => 'E',
            Position::South => 'S',
            Position::West => 'W',
        };

        print!("  {}  ", pos_char);
        for (denom_idx, denom) in denoms.iter().enumerate() {
            let actual = solver.solve(*denom, *pos);
            let expected = tc.expected[denom_idx][pos_idx];
            let matches = actual == expected;
            if !matches {
                all_match = false;
            }
            let marker = if matches { " " } else { "!" };
            print!("{:2}/{:2}{} ", actual, expected, marker);
        }
        println!();
    }

    if all_match {
        println!("PASS - All results match!");
    } else {
        println!("FAIL - Some results differ (marked with !)");
    }

    all_match
}

fn main() {
    let test_cases = vec![
        // Deal 2
        TestCase {
            name: "Deal 2",
            deal: parse_deal(
                "AKT52.97.965.J84",
                "9.A864.AT743.T97",
                "J874.KJ3.Q.AKQ32",
                "Q63.QT52.KJ82.65",
            ),
            // [NT, S, H, D, C] x [N, E, S, W]
            expected: [
                [7, 3, 7, 3],   // NT
                [11, 2, 11, 2], // S
                [4, 8, 4, 8],   // H
                [4, 9, 4, 9],   // D
                [11, 2, 11, 2], // C
            ],
        },
        // Deal 3
        TestCase {
            name: "Deal 3",
            deal: parse_deal(
                "AKQT654.7.754.Q2",
                "J732.KQJ4.QT9.93",
                ".AT862.32.KT8654",
                "98.953.AKJ86.AJ7",
            ),
            expected: [
                [5, 8, 5, 8],  // NT
                [7, 5, 7, 5],  // S
                [6, 6, 6, 6],  // H
                [5, 8, 5, 8],  // D
                [8, 4, 8, 4],  // C
            ],
        },
        // Deal 4
        TestCase {
            name: "Deal 4",
            deal: parse_deal(
                "AQ96.AKQ3.AQ6.75",
                "KT42.T876.53.T43",
                "J87.J954.KJ42.Q6",
                "53.2.T987.AKJ982",
            ),
            expected: [
                [11, 2, 11, 2],  // NT
                [11, 2, 11, 2],  // S
                [11, 1, 11, 1],  // H
                [9, 4, 9, 4],    // D
                [10, 3, 10, 3],  // C
            ],
        },
        // Deal 6 (North makes only 3 tricks in NT per BC)
        TestCase {
            name: "Deal 6",
            deal: parse_deal(
                "J9.Q964.K42.A543",
                "AKT42.T853.Q76.7",
                "Q653.J7.JT985.K6",
                "87.AK2.A3.QJT982",
            ),
            expected: [
                [3, 9, 3, 9],  // NT
                [6, 7, 6, 7],  // S
                [3, 9, 3, 9],  // H
                [3, 9, 3, 9],  // D
                [6, 7, 6, 7],  // C
            ],
        },
    ];

    println!("Testing DD Solver against Bridge Composer results");
    println!("==================================================");

    let mut passed = 0;
    let mut failed = 0;

    for tc in &test_cases {
        if test_deal(tc) {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    println!("\n==================================================");
    println!("Summary: {} passed, {} failed", passed, failed);
}
