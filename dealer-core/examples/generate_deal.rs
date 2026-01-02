use dealer_core::{DealGenerator, Position};

fn main() {
    let mut generator = DealGenerator::new(1);
    let deal = generator.generate();

    println!("Bridge Deal (Seed: 1)");
    println!("=====================\n");

    for position in Position::ALL {
        let hand = deal.hand(position);
        println!(
            "{}: {} cards, {} HCP, Shape: {}",
            position.to_char(),
            hand.len(),
            hand.hcp(),
            hand.shape()
        );

        // Print cards by suit
        for suit in dealer_core::Suit::ALL.iter().rev() {
            let cards_in_suit = hand.cards_in_suit(*suit);
            if !cards_in_suit.is_empty() {
                print!("  {} ", suit.symbol());
                for card in cards_in_suit {
                    print!("{}", card.rank.to_char());
                }
                println!();
            }
        }
        println!();
    }

    // Print statistics
    println!("Statistics:");
    println!("-----------");
    println!(
        "North: {} HCP, {} controls, balanced: {}",
        deal.north.hcp(),
        deal.north.controls(),
        deal.north.is_balanced()
    );
    println!(
        "South: {} HCP, {} controls, balanced: {}",
        deal.south.hcp(),
        deal.south.controls(),
        deal.south.is_balanced()
    );
}
