use crate::{Card, Hand, Position};
use gnurandom::{GnuRandom, GnuRandomState};

/// Represents a complete bridge deal (4 hands of 13 cards each)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deal {
    pub north: Hand,
    pub east: Hand,
    pub south: Hand,
    pub west: Hand,
}

impl Deal {
    /// Create a new empty deal
    pub fn new() -> Self {
        Deal {
            north: Hand::new(),
            east: Hand::new(),
            south: Hand::new(),
            west: Hand::new(),
        }
    }

    /// Get a reference to a hand by position
    pub fn hand(&self, position: Position) -> &Hand {
        match position {
            Position::North => &self.north,
            Position::East => &self.east,
            Position::South => &self.south,
            Position::West => &self.west,
        }
    }

    /// Get a mutable reference to a hand by position
    pub fn hand_mut(&mut self, position: Position) -> &mut Hand {
        match position {
            Position::North => &mut self.north,
            Position::East => &mut self.east,
            Position::South => &mut self.south,
            Position::West => &mut self.west,
        }
    }

    /// Sort all hands in the deal
    pub fn sort_all_hands(&mut self) {
        self.north.sort();
        self.east.sort();
        self.south.sort();
        self.west.sort();
    }
}

impl Default for Deal {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared predeal configuration, immutable after setup.
/// Can be shared across threads via Arc for parallel generation.
#[derive(Clone)]
pub struct DealGeneratorConfig {
    /// Lookup table to avoid modulo operations (65KB)
    pub zero52: Box<[u8; 65536]>,
    /// Full pack with predealt cards marked as None
    pub fullpack: [Option<u8>; 52],
    /// Predealt cards (matches dealer.c's stacked_pack)
    pub stacked_pack: [Option<u8>; 52],
}

/// Per-deal state that changes with each generation.
/// This is the minimal state needed to reproduce a specific deal.
#[derive(Clone, Copy)]
pub struct DealWorkState {
    /// RNG state at this point
    pub rng_state: GnuRandomState,
    /// Current deal state (cards in slots)
    pub curdeal: [u8; 52],
}

/// Captured state of a DealGenerator, allowing exact reproduction of deals.
/// Used for parallel deal generation where each worker needs its own generator.
#[derive(Clone)]
pub struct DealGeneratorState {
    rng_state: GnuRandomState,
    zero52: Box<[u8; 65536]>, // Boxed to avoid stack overflow on clone
    curdeal: [u8; 52],
    fullpack: [Option<u8>; 52],
    stacked_pack: [Option<u8>; 52],
}

/// Generator for creating random bridge deals
pub struct DealGenerator {
    rng: GnuRandom,
    zero52: [u8; 65536],            // Lookup table to avoid modulo operations
    curdeal: [u8; 52],              // Current deal (slot-indexed, matches dealer.c)
    fullpack: [Option<u8>; 52],     // Full pack with predealt cards marked as None
    stacked_pack: [Option<u8>; 52], // Predealt cards (matches dealer.c's stacked_pack)
}

impl DealGenerator {
    /// Create a new deal generator with a given seed
    pub fn new(seed: u32) -> Self {
        let mut rng = GnuRandom::new();
        rng.srandom(seed);

        // Initialize fullpack in suit-rank order (matches newpack in dealer.c)
        // Clubs 2-A, Diamonds 2-A, Hearts 2-A, Spades 2-A
        let mut fullpack = [None; 52];
        for (i, card) in fullpack.iter_mut().enumerate() {
            *card = Some(i as u8);
        }

        // Initialize stacked_pack to None (no predeal)
        let stacked_pack = [None; 52];

        // curdeal starts empty, will be set up by setup_deal()
        let curdeal = [0u8; 52];

        // Initialize zero52 table (will be rebuilt if predeal is used)
        let mut gen = DealGenerator {
            rng,
            zero52: [0u8; 65536],
            curdeal,
            fullpack,
            stacked_pack,
        };
        gen.rebuild_zero52();
        // Set up the initial deal (matches dealer.c calling setup_deal once before loop)
        gen.setup_deal();
        gen
    }

    /// Rebuild the zero52 lookup table (called after setting up predeal)
    /// This matches initprogram() in dealer.c
    fn rebuild_zero52(&mut self) {
        let mut val = 0usize;
        let mut i_cycle = 0usize;

        for i in 0..65536 {
            // Skip predealt slots
            while val < 52 && self.stacked_pack[val].is_some() {
                val += 1;
                if val == 52 {
                    val = 0;
                    i_cycle = i;
                }
            }

            if val < 52 {
                self.zero52[i] = val as u8;
                val += 1;
                if val == 52 {
                    val = 0;
                    i_cycle = i + 1;
                }
            }
        }

        // Fill the last part with 0xFF (prevents over-representation)
        for i in (i_cycle..65536).rev() {
            self.zero52[i] = 0xFF;
        }
    }

    /// Set up the current deal from fullpack (matches setup_deal in dealer.c)
    /// This is called once after predeal, before the shuffle loop
    fn setup_deal(&mut self) {
        let mut j = 0usize;

        for i in 0..52 {
            if let Some(card) = self.stacked_pack[i] {
                // This slot has a predealt card
                self.curdeal[i] = card;
            } else {
                // Find next available card from fullpack
                while j < 52 && self.fullpack[j].is_none() {
                    j += 1;
                }
                if j < 52 {
                    self.curdeal[i] = self.fullpack[j].unwrap();
                    j += 1;
                }
            }
        }
    }

    /// Predeal cards to a specific position
    /// This matches predeal() in dealer.c
    /// Returns an error if more than 13 cards are dealt to one position or if a card is dealt twice
    pub fn predeal(&mut self, position: Position, cards: &[Card]) -> Result<(), String> {
        let pos_offset = (position as usize) * 13;

        for &card in cards {
            let card_idx = card.to_index() as usize;

            // Find this card in fullpack and mark it as predealt
            // dealer.c: if (fullpack[i] == onecard) { fullpack[i] = NO_CARD; ... }
            if self.fullpack[card_idx].is_none() {
                return Err(format!("Card {:?} predealt twice", card));
            }

            // Mark card as removed from fullpack
            self.fullpack[card_idx] = None;

            // Find first empty slot for this position
            let mut placed = false;
            for slot in pos_offset..(pos_offset + 13) {
                if self.stacked_pack[slot].is_none() {
                    self.stacked_pack[slot] = Some(card_idx as u8);
                    placed = true;
                    break;
                }
            }

            if !placed {
                return Err(format!(
                    "More than 13 cards for position {}",
                    position.to_char()
                ));
            }
        }

        // Rebuild zero52 table and setup_deal after predeal
        self.rebuild_zero52();
        self.setup_deal();
        Ok(())
    }

    /// Generate a random deal using Knuth's shuffle algorithm
    /// This exactly matches dealer.exe's shuffle implementation with predeal support
    /// NOTE: Each call reshuffles the SAME curdeal (not a fresh sorted deck)
    pub fn generate(&mut self) -> Deal {
        // Knuth's shuffle algorithm (forward iteration, as in dealer.c)
        // For each slot i, swap with a random slot j (0 <= j <= 51)
        // IMPORTANT: We shuffle the existing curdeal, not a fresh sorted one!
        // When predeal is active, skip predealt slots (matches dealer.c lines 859-877)
        for i in 0..52 {
            // If this slot is predealt, skip it (don't swap)
            if self.stacked_pack[i].is_none() {
                // Generate random index using zero52 lookup table
                // dealer.c uses: k = (RANDOM() >> (31 - RANDBITS));
                // with RANDBITS=16, this is: k = (RANDOM() >> 15);
                // The inner do-while loops retry until we find a non-predealt slot
                let j = loop {
                    let r = self.rng.next_u32();
                    let k = r >> 15; // Shift right by 15 bits
                    let j = self.zero52[(k & 0xFFFF) as usize]; // NRANDMASK = 0xFFFF for 16-bit RANDBITS

                    if j != 0xFF && self.stacked_pack[j as usize].is_none() {
                        break j as usize;
                    }
                    // Retry if j == 0xFF or if j is a predealt slot
                };

                // Swap curdeal[i] with curdeal[j]
                self.curdeal.swap(i, j);
            }
        }

        // Distribute cards to hands from curdeal
        // curdeal is slot-indexed: slots 0-12=North, 13-25=East, 26-38=South, 39-51=West
        let mut deal = Deal::new();

        for slot in 0..52 {
            let card_index = self.curdeal[slot];
            let card = Card::from_index(card_index).unwrap();
            let position = Position::from_index(slot / 13).unwrap();
            deal.hand_mut(position).add_card(card);
        }

        // Sort all hands for display
        deal.sort_all_hands();

        deal
    }

    /// Generate multiple deals
    pub fn generate_many(&mut self, count: usize) -> Vec<Deal> {
        (0..count).map(|_| self.generate()).collect()
    }

    /// Advance the RNG state as if generating one deal, but don't do the actual shuffle.
    /// This is an optimization for parallel batch generation where we only need to capture
    /// the RNG state and advance it, without doing the full shuffle work.
    ///
    /// IMPORTANT: This also updates curdeal with the swap results to maintain consistency
    /// with the original generator behavior.
    pub fn advance_one_deal(&mut self) {
        // Same RNG consumption pattern as generate(), but we still need to do the swaps
        // because curdeal state affects future deals
        for i in 0..52 {
            if self.stacked_pack[i].is_none() {
                let j = loop {
                    let r = self.rng.next_u32();
                    let k = r >> 15;
                    let j = self.zero52[(k & 0xFFFF) as usize];

                    if j != 0xFF && self.stacked_pack[j as usize].is_none() {
                        break j as usize;
                    }
                };

                self.curdeal.swap(i, j);
            }
        }
        // Skip deal distribution and sorting - that's the expensive part that workers will do
    }

    /// Capture the current generator state for later restoration.
    /// This allows parallel workers to reproduce the exact same deal sequence.
    pub fn capture_state(&self) -> DealGeneratorState {
        DealGeneratorState {
            rng_state: self.rng.capture_state(),
            zero52: Box::new(self.zero52),
            curdeal: self.curdeal,
            fullpack: self.fullpack,
            stacked_pack: self.stacked_pack,
        }
    }

    /// Create a new DealGenerator from a captured state.
    /// The new generator will produce the exact same deals as the original
    /// would have from the point the state was captured.
    pub fn from_state(state: DealGeneratorState) -> Self {
        Self {
            rng: GnuRandom::from_state(state.rng_state),
            zero52: *state.zero52,
            curdeal: state.curdeal,
            fullpack: state.fullpack,
            stacked_pack: state.stacked_pack,
        }
    }

    /// Generate exactly one deal and return both the deal and the number of
    /// RNG calls consumed. This is useful for parallel generation where the
    /// supervisor needs to know how much to advance the RNG.
    ///
    /// Returns (deal, rng_calls) where rng_calls is the number of times next_u32 was called.
    /// Note: With predeal, the number varies due to retries when hitting predealt slots.
    pub fn generate_one_with_rng_count(&mut self) -> (Deal, usize) {
        let mut rng_calls = 0;

        // Knuth's shuffle algorithm (forward iteration, as in dealer.c)
        for i in 0..52 {
            if self.stacked_pack[i].is_none() {
                let j = loop {
                    let r = self.rng.next_u32();
                    rng_calls += 1;
                    let k = r >> 15;
                    let j = self.zero52[(k & 0xFFFF) as usize];

                    if j != 0xFF && self.stacked_pack[j as usize].is_none() {
                        break j as usize;
                    }
                };

                self.curdeal.swap(i, j);
            }
        }

        // Distribute cards to hands from curdeal
        let mut deal = Deal::new();
        for slot in 0..52 {
            let card_index = self.curdeal[slot];
            let card = Card::from_index(card_index).unwrap();
            let position = Position::from_index(slot / 13).unwrap();
            deal.hand_mut(position).add_card(card);
        }
        deal.sort_all_hands();

        (deal, rng_calls)
    }

    /// Capture the shared configuration (predeal settings) that doesn't change between deals.
    /// This can be shared across threads via Arc for efficient parallel generation.
    pub fn capture_config(&self) -> DealGeneratorConfig {
        DealGeneratorConfig {
            zero52: Box::new(self.zero52),
            fullpack: self.fullpack,
            stacked_pack: self.stacked_pack,
        }
    }

    /// Capture the per-deal work state (RNG and curdeal).
    /// This is the minimal state needed to reproduce a specific deal.
    pub fn capture_work_state(&self) -> DealWorkState {
        DealWorkState {
            rng_state: self.rng.capture_state(),
            curdeal: self.curdeal,
        }
    }

    /// Generate a deal from work state using shared config.
    /// This is the efficient parallel generation path - config is shared, only work_state is cloned.
    pub fn generate_from_work_state(
        config: &DealGeneratorConfig,
        work_state: DealWorkState,
    ) -> Deal {
        let mut rng = GnuRandom::from_state(work_state.rng_state);
        let mut curdeal = work_state.curdeal;

        // Knuth's shuffle algorithm
        for i in 0..52 {
            if config.stacked_pack[i].is_none() {
                let j = loop {
                    let r = rng.next_u32();
                    let k = r >> 15;
                    let j = config.zero52[(k & 0xFFFF) as usize];

                    if j != 0xFF && config.stacked_pack[j as usize].is_none() {
                        break j as usize;
                    }
                };

                curdeal.swap(i, j);
            }
        }

        // Distribute cards to hands
        let mut deal = Deal::new();
        for (slot, &card_index) in curdeal.iter().enumerate() {
            let card = Card::from_index(card_index).unwrap();
            let position = Position::from_index(slot / 13).unwrap();
            deal.hand_mut(position).add_card(card);
        }
        deal.sort_all_hands();

        deal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rank, Suit};

    #[test]
    fn test_deal_generation() {
        let mut gen = DealGenerator::new(1);
        let deal = gen.generate();

        // Each hand should have exactly 13 cards
        assert_eq!(deal.north.len(), 13);
        assert_eq!(deal.east.len(), 13);
        assert_eq!(deal.south.len(), 13);
        assert_eq!(deal.west.len(), 13);

        // Total HCP should be 40
        let total_hcp = deal.north.hcp() + deal.east.hcp() + deal.south.hcp() + deal.west.hcp();
        assert_eq!(total_hcp, 40);
    }

    #[test]
    fn test_deterministic_generation() {
        // Same seed should produce same deal
        let mut gen1 = DealGenerator::new(42);
        let mut gen2 = DealGenerator::new(42);

        let deal1 = gen1.generate();
        let deal2 = gen2.generate();

        assert_eq!(deal1, deal2);
    }

    #[test]
    fn test_different_seeds_different_deals() {
        let mut gen1 = DealGenerator::new(1);
        let mut gen2 = DealGenerator::new(2);

        let deal1 = gen1.generate();
        let deal2 = gen2.generate();

        // Different seeds should (almost certainly) produce different deals
        assert_ne!(deal1, deal2);
    }

    #[test]
    fn test_all_cards_distributed() {
        let mut gen = DealGenerator::new(123);
        let deal = gen.generate();

        // Collect all cards from all hands
        let mut all_cards = Vec::new();
        all_cards.extend_from_slice(deal.north.cards());
        all_cards.extend_from_slice(deal.east.cards());
        all_cards.extend_from_slice(deal.south.cards());
        all_cards.extend_from_slice(deal.west.cards());

        // Should have exactly 52 unique cards
        assert_eq!(all_cards.len(), 52);

        // Convert to indices and sort
        let mut indices: Vec<u8> = all_cards.iter().map(|c| c.to_index()).collect();
        indices.sort();

        // Should be exactly 0..52
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(i as u8, idx);
        }
    }

    #[test]
    fn test_partner_positions() {
        assert_eq!(Position::North.partner(), Position::South);
        assert_eq!(Position::South.partner(), Position::North);
        assert_eq!(Position::East.partner(), Position::West);
        assert_eq!(Position::West.partner(), Position::East);
    }

    #[test]
    fn test_predeal_basic() {
        let mut gen = DealGenerator::new(42);

        // Predeal AS, KS, QS to North
        let cards = vec![
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Spades, Rank::Queen),
        ];

        gen.predeal(Position::North, &cards).unwrap();

        // Generate a deal
        let deal = gen.generate();

        // Verify North has the predealt cards
        let north = deal.hand(Position::North);
        assert!(north.cards().contains(&Card::new(Suit::Spades, Rank::Ace)));
        assert!(north.cards().contains(&Card::new(Suit::Spades, Rank::King)));
        assert!(north
            .cards()
            .contains(&Card::new(Suit::Spades, Rank::Queen)));

        // Verify North has exactly 13 cards
        assert_eq!(north.len(), 13);
    }

    #[test]
    fn test_predeal_deterministic() {
        // Same seed and predeal should produce same deals
        let cards = vec![
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Hearts, Rank::King),
        ];

        let mut gen1 = DealGenerator::new(123);
        gen1.predeal(Position::North, &cards).unwrap();
        let deal1 = gen1.generate();

        let mut gen2 = DealGenerator::new(123);
        gen2.predeal(Position::North, &cards).unwrap();
        let deal2 = gen2.generate();

        assert_eq!(deal1, deal2);
    }

    #[test]
    fn test_predeal_multiple_positions() {
        let mut gen = DealGenerator::new(42);

        // Predeal to North
        gen.predeal(Position::North, &[Card::new(Suit::Spades, Rank::Ace)])
            .unwrap();

        // Predeal to South
        gen.predeal(Position::South, &[Card::new(Suit::Hearts, Rank::Ace)])
            .unwrap();

        let deal = gen.generate();

        assert!(deal
            .hand(Position::North)
            .cards()
            .contains(&Card::new(Suit::Spades, Rank::Ace)));
        assert!(deal
            .hand(Position::South)
            .cards()
            .contains(&Card::new(Suit::Hearts, Rank::Ace)));
    }

    #[test]
    fn test_predeal_all_13_cards() {
        let mut gen = DealGenerator::new(42);

        // Predeal all 13 spades to North
        let all_spades: Vec<Card> = (0..13)
            .map(|i| Card::from_index(39 + i).unwrap()) // Spades are indices 39-51
            .collect();

        gen.predeal(Position::North, &all_spades).unwrap();

        let deal = gen.generate();
        let north = deal.hand(Position::North);

        // Verify North has all 13 spades
        assert_eq!(north.suit_length(Suit::Spades), 13);
        assert_eq!(north.len(), 13);
    }

    #[test]
    fn test_predeal_duplicate_card_error() {
        let mut gen = DealGenerator::new(42);

        let ace_spades = Card::new(Suit::Spades, Rank::Ace);
        gen.predeal(Position::North, &[ace_spades]).unwrap();

        // Try to predeal same card again
        let result = gen.predeal(Position::South, &[ace_spades]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("predealt twice"));
    }

    #[test]
    fn test_predeal_too_many_cards_error() {
        let mut gen = DealGenerator::new(42);

        // Try to predeal 14 cards to one position
        let cards: Vec<Card> = (0..14).map(|i| Card::from_index(i).unwrap()).collect();

        let result = gen.predeal(Position::North, &cards);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("More than 13 cards"));
    }

    #[test]
    fn test_predeal_no_duplicate_cards_in_deal() {
        let mut gen = DealGenerator::new(42);

        gen.predeal(
            Position::North,
            &[
                Card::new(Suit::Spades, Rank::Ace),
                Card::new(Suit::Hearts, Rank::Ace),
            ],
        )
        .unwrap();

        let deal = gen.generate();

        // Collect all cards from all hands
        let mut all_cards = Vec::new();
        all_cards.extend_from_slice(deal.north.cards());
        all_cards.extend_from_slice(deal.east.cards());
        all_cards.extend_from_slice(deal.south.cards());
        all_cards.extend_from_slice(deal.west.cards());

        // Should have exactly 52 unique cards
        assert_eq!(all_cards.len(), 52);

        // Convert to indices and sort
        let mut indices: Vec<u8> = all_cards.iter().map(|c| c.to_index()).collect();
        indices.sort();

        // Should be exactly 0..52
        for (i, &idx) in indices.iter().enumerate() {
            assert_eq!(i as u8, idx);
        }
    }

    #[test]
    fn test_generator_state_capture_and_restore() {
        let mut gen1 = DealGenerator::new(42);

        // Generate a few deals to advance state
        for _ in 0..5 {
            gen1.generate();
        }

        // Capture state
        let state = gen1.capture_state();

        // Get next 3 deals from original
        let expected: Vec<Deal> = (0..3).map(|_| gen1.generate()).collect();

        // Create new generator from captured state
        let mut gen2 = DealGenerator::from_state(state);

        // Should produce identical deals
        for (i, expected_deal) in expected.iter().enumerate() {
            let actual_deal = gen2.generate();
            assert_eq!(
                &actual_deal, expected_deal,
                "Deal {} mismatch after state restore",
                i
            );
        }
    }

    #[test]
    fn test_generator_state_with_predeal() {
        let mut gen1 = DealGenerator::new(42);

        // Set up predeal
        gen1.predeal(Position::North, &[Card::new(Suit::Spades, Rank::Ace)])
            .unwrap();

        // Generate a few deals
        for _ in 0..3 {
            gen1.generate();
        }

        // Capture state
        let state = gen1.capture_state();

        // Get next deal from original
        let expected = gen1.generate();

        // Create new generator from captured state
        let mut gen2 = DealGenerator::from_state(state);

        // Should produce identical deal
        let actual = gen2.generate();
        assert_eq!(actual, expected);

        // Both should have the predealt card in North
        assert!(actual
            .hand(Position::North)
            .cards()
            .contains(&Card::new(Suit::Spades, Rank::Ace)));
    }

    #[test]
    fn test_multiple_state_captures_for_parallel() {
        // Simulate supervisor capturing states for multiple workers
        let mut supervisor = DealGenerator::new(1);

        // Capture states at different points
        let state1 = supervisor.capture_state();
        supervisor.generate(); // Advance by one deal
        let state2 = supervisor.capture_state();
        supervisor.generate();
        let state3 = supervisor.capture_state();

        // Workers restore states and generate deals
        let mut worker1 = DealGenerator::from_state(state1);
        let mut worker2 = DealGenerator::from_state(state2);
        let mut worker3 = DealGenerator::from_state(state3);

        let deal1 = worker1.generate();
        let deal2 = worker2.generate();
        let deal3 = worker3.generate();

        // Each should produce different deals
        assert_ne!(
            deal1, deal2,
            "Workers 1 and 2 should produce different deals"
        );
        assert_ne!(
            deal2, deal3,
            "Workers 2 and 3 should produce different deals"
        );
        assert_ne!(
            deal1, deal3,
            "Workers 1 and 3 should produce different deals"
        );

        // Verify determinism: fresh generator should produce same sequence
        let mut verify = DealGenerator::new(1);
        assert_eq!(
            verify.generate(),
            deal1,
            "Deal 1 should match fresh generator"
        );
        assert_eq!(
            verify.generate(),
            deal2,
            "Deal 2 should match fresh generator"
        );
        assert_eq!(
            verify.generate(),
            deal3,
            "Deal 3 should match fresh generator"
        );
    }
}
