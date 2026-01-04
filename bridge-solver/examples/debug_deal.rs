//! Debug a specific deal by showing the play line
//!
//! Uses the 2nd deal from north_9tricks_nt.pbn

use dealer_core::{Card, Deal, Position, Rank, Suit};
use dealer_dds::{Denomination, DoubleDummySolver};

fn parse_holding(s: &str, suit: Suit) -> Vec<Card> {
    s.chars()
        .filter_map(|c| {
            let rank = match c {
                'A' => Some(Rank::Ace),
                'K' => Some(Rank::King),
                'Q' => Some(Rank::Queen),
                'J' => Some(Rank::Jack),
                'T' => Some(Rank::Ten),
                '9' => Some(Rank::Nine),
                '8' => Some(Rank::Eight),
                '7' => Some(Rank::Seven),
                '6' => Some(Rank::Six),
                '5' => Some(Rank::Five),
                '4' => Some(Rank::Four),
                '3' => Some(Rank::Three),
                '2' => Some(Rank::Two),
                _ => None,
            };
            rank.map(|r| Card::new(suit, r))
        })
        .collect()
}

fn parse_hand(s: &str) -> Vec<Card> {
    let suits: Vec<&str> = s.split('.').collect();
    if suits.len() != 4 {
        panic!("Invalid hand format: {}", s);
    }
    let mut cards = Vec::new();
    cards.extend(parse_holding(suits[0], Suit::Spades));
    cards.extend(parse_holding(suits[1], Suit::Hearts));
    cards.extend(parse_holding(suits[2], Suit::Diamonds));
    cards.extend(parse_holding(suits[3], Suit::Clubs));
    cards
}

fn parse_pbn_deal(deal_str: &str) -> Deal {
    // Format: "N:hand_n hand_e hand_s hand_w"
    let parts: Vec<&str> = deal_str.split(':').collect();
    let first_seat = parts[0].chars().next().unwrap();
    let hands_str: Vec<&str> = parts[1].split_whitespace().collect();

    let positions = match first_seat {
        'N' => [Position::North, Position::East, Position::South, Position::West],
        'E' => [Position::East, Position::South, Position::West, Position::North],
        'S' => [Position::South, Position::West, Position::North, Position::East],
        'W' => [Position::West, Position::North, Position::East, Position::South],
        _ => panic!("Invalid first seat: {}", first_seat),
    };

    let mut hand_cards: [Vec<Card>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    for (i, hand_str) in hands_str.iter().enumerate() {
        let pos = positions[i];
        hand_cards[pos as usize] = parse_hand(hand_str);
    }

    let mut deal = Deal::new();
    for pos in Position::ALL {
        for card in &hand_cards[pos as usize] {
            deal.hand_mut(pos).add_card(*card);
        }
    }
    deal
}

fn card_str(card: Card) -> String {
    let suit = match card.suit {
        Suit::Spades => 'S',
        Suit::Hearts => 'H',
        Suit::Diamonds => 'D',
        Suit::Clubs => 'C',
    };
    let rank = match card.rank {
        Rank::Ace => 'A',
        Rank::King => 'K',
        Rank::Queen => 'Q',
        Rank::Jack => 'J',
        Rank::Ten => 'T',
        Rank::Nine => '9',
        Rank::Eight => '8',
        Rank::Seven => '7',
        Rank::Six => '6',
        Rank::Five => '5',
        Rank::Four => '4',
        Rank::Three => '3',
        Rank::Two => '2',
    };
    format!("{}{}", rank, suit)
}

fn pos_char(pos: Position) -> char {
    match pos {
        Position::North => 'N',
        Position::East => 'E',
        Position::South => 'S',
        Position::West => 'W',
    }
}

fn print_deal(deal: &Deal) {
    println!("         North");
    print!("         ");
    for c in deal.hand(Position::North).cards() {
        print!("{} ", card_str(*c));
    }
    println!();
    println!();

    print!("West              East\n");
    let west_str: String = deal
        .hand(Position::West)
        .cards()
        .iter()
        .map(|c| card_str(*c))
        .collect::<Vec<_>>()
        .join(" ");
    let east_str: String = deal
        .hand(Position::East)
        .cards()
        .iter()
        .map(|c| card_str(*c))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{:<42}{}", west_str, east_str);
    println!();

    println!("         South");
    print!("         ");
    for c in deal.hand(Position::South).cards() {
        print!("{} ", card_str(*c));
    }
    println!();
}

fn main() {
    // Deal 2 from north_9tricks_nt.pbn (too slow for now)
    // [Deal "N:AKT52.97.965.J84 9.A864.AT743.T97 J874.KJ3.Q.AKQ32 Q63.QT52.KJ82.65"]
    // let deal_str = "N:AKT52.97.965.J84 9.A864.AT743.T97 J874.KJ3.Q.AKQ32 Q63.QT52.KJ82.65";

    // 6-card endgame for testing play history
    // N: SA HA DA CA TS TH - E: KS KH KD KC 9S 9H - S: QS QH QD QC 9D 9C - W: JS JH JD JC TD TC
    let deal_str = "N:AT.AT.A.A K9.K9.K.K Q.Q.Q9.Q9 J.J.JT.JT";

    println!("Debug Deal - Showing Play Line");
    println!("===============================\n");

    let deal = parse_pbn_deal(deal_str);
    print_deal(&deal);
    println!();

    let solver = DoubleDummySolver::new(deal.clone());

    // Test North declaring in NT
    let declarer = Position::North;
    let denomination = Denomination::NoTrump;

    println!(
        "Solving for {} declaring in {}...",
        pos_char(declarer),
        match denomination {
            Denomination::NoTrump => "NT",
            Denomination::Spades => "Spades",
            Denomination::Hearts => "Hearts",
            Denomination::Diamonds => "Diamonds",
            Denomination::Clubs => "Clubs",
        }
    );
    println!();

    let result = solver.solve_with_line(denomination, declarer);

    println!("Result: {} tricks for N/S\n", result.tricks);
    println!("Play line that achieves {} tricks:", result.tricks);
    println!("=====================================");

    // Display trick by trick
    let opening_leader = match declarer {
        Position::North => Position::East,
        Position::East => Position::South,
        Position::South => Position::West,
        Position::West => Position::North,
    };

    let mut trick_cards: Vec<(Position, Card)> = Vec::new();
    let mut trick_num = 1;
    let mut leader = opening_leader;
    let mut ns_tricks = 0;

    for (pos, card) in &result.play_line {
        trick_cards.push((*pos, *card));

        if trick_cards.len() == 4 {
            // Determine winner (simplified - NT only)
            let suit_led = trick_cards[0].1.suit;
            let mut winner = trick_cards[0].0;
            let mut winning_card = trick_cards[0].1;

            for &(p, c) in &trick_cards[1..] {
                if c.suit == suit_led && c.rank > winning_card.rank {
                    winner = p;
                    winning_card = c;
                }
            }

            let won_by_ns = winner == Position::North || winner == Position::South;
            if won_by_ns {
                ns_tricks += 1;
            }

            println!(
                "Trick {:2}: {} leads {} - {} plays {} - {} plays {} - {} plays {} => {} wins {} (N/S: {})",
                trick_num,
                pos_char(leader),
                card_str(trick_cards[0].1),
                pos_char(trick_cards[1].0),
                card_str(trick_cards[1].1),
                pos_char(trick_cards[2].0),
                card_str(trick_cards[2].1),
                pos_char(trick_cards[3].0),
                card_str(trick_cards[3].1),
                pos_char(winner),
                if won_by_ns { "N/S" } else { "E/W" },
                ns_tricks
            );

            leader = winner;
            trick_num += 1;
            trick_cards.clear();
        }
    }

    println!("\nFinal: N/S made {} tricks", ns_tricks);

    if ns_tricks != result.tricks {
        println!("\n*** ERROR: Play line gives {} tricks but solver reported {} ***",
            ns_tricks, result.tricks);
    }
}
