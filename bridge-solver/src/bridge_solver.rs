//! Main solver implementation
//!
//! Uses alpha-beta search with MTD(f) driver

use super::cards::*;
use super::hands::Hands;
use super::search;
use super::types::*;

/// Ordered cards container for move ordering
pub struct OrderedCards {
    cards: [u8; TOTAL_TRICKS],
    count: usize,
}

impl OrderedCards {
    #[inline]
    pub fn new() -> Self {
        OrderedCards {
            cards: [0; TOTAL_TRICKS],
            count: 0,
        }
    }

    #[inline]
    fn add(&mut self, card: usize) {
        self.cards[self.count] = card as u8;
        self.count += 1;
    }

    /// Add cards in natural order (high to low)
    #[inline]
    fn add_cards(&mut self, cards: Cards) {
        for card in cards.iter() {
            self.add(card);
        }
    }

    /// Add cards in reversed order (low to high)
    #[inline]
    fn add_reversed(&mut self, cards: Cards) {
        // Iterate in reverse by collecting to bottom first
        let mut remaining = cards;
        while !remaining.is_empty() {
            let card = remaining.bottom();
            self.add(card);
            remaining.remove(card);
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = usize> + '_ {
        self.cards[..self.count].iter().map(|&c| c as usize)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.count
    }

    #[inline]
    pub fn card(&self, i: usize) -> usize {
        self.cards[i] as usize
    }
}

/// Format a card as a string (e.g., "SA" for spade ace)
fn card_name(card: usize) -> String {
    const SUITS: [char; 4] = ['S', 'H', 'D', 'C'];
    const RANKS: [char; 13] = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2'];
    let suit = suit_of(card);
    let rank = rank_of(card);
    format!("{}{}", SUITS[suit], RANKS[12 - rank])
}

/// Check if a card is equivalent to one of the already-tried cards.
/// Two cards are equivalent if all cards between them (in the remaining deck)
/// belong to the current player's hand.
///
/// This is a key optimization: if we've tried the King and all cards between
/// King and Jack belong to us (i.e., Queen is ours or already played), then
/// trying the Jack will give the same result as the King.
#[inline]
fn is_equivalent(card: usize, tried_cards: Cards, all_cards: Cards, my_hand: Cards) -> bool {
    let mut result = false;

    if !tried_cards.is_empty() {
        // Get the suit of this card
        let suit = suit_of(card);
        let tried_suit = tried_cards.suit(suit);

        if !tried_suit.is_empty() {
            let all_suit = all_cards.suit(suit);
            let my_suit = my_hand.suit(suit);

            // Check if equivalent to a higher-ranked tried card
            // "above" = tried cards with higher rank (lower index) than this card
            let above = tried_suit.slice(0, card);
            if !above.is_empty() {
                // Get the closest tried card above (lowest rank among higher cards)
                let closest_above = above.bottom();
                // Cards between closest_above and card (exclusive on both ends)
                let between = all_suit.slice(closest_above + 1, card);
                let my_between = my_suit.slice(closest_above + 1, card);
                // If all remaining cards between them are mine, they're equivalent
                if between == my_between {
                    result = true;
                }
            }

            // Check if equivalent to a lower-ranked tried card
            if !result {
                // "below" = tried cards with lower rank (higher index) than this card
                let below = tried_suit.slice(card + 1, NUM_SUITS * NUM_RANKS);
                if !below.is_empty() {
                    // Get the closest tried card below (highest rank among lower cards)
                    let closest_below = below.top();
                    // Cards between card and closest_below (exclusive on both ends)
                    let between = all_suit.slice(card + 1, closest_below);
                    let my_between = my_suit.slice(card + 1, closest_below);
                    // If all remaining cards between them are mine, they're equivalent
                    if between == my_between {
                        result = true;
                    }
                }
            }
        }
    }

    // Log if xray tracing is enabled and under limit
    if xray_should_log() {
        let suit = suit_of(card);
        let tried_suit = tried_cards.suit(suit);
        let my_suit = my_hand.suit(suit);
        let all_suit = all_cards.suit(suit);
        eprintln!(
            "EQUIV: card={} tried=0x{:x} hand=0x{:x} all=0x{:x} -> {}",
            card_name(card),
            tried_suit.value(),
            my_suit.value(),
            all_suit.value(),
            if result { "true" } else { "false" }
        );
    }

    result
}

/// Order lead cards (when starting a trick)
/// Priority: ruff_leads > good_leads > high_leads > normal_leads > bad_leads > trump_leads
pub fn order_leads(
    playable: Cards,
    hands: &Hands,
    seat: Seat,
    trump: usize,
    all_cards: Cards,
) -> OrderedCards {
    let mut ordered = OrderedCards::new();
    let mut remaining = playable;

    let pd_hand = hands[partner(seat)];
    let lho_hand = hands[left_hand_opp(seat)];
    let rho_hand = hands[right_hand_opp(seat)];
    let partnership_cards = hands[seat].union(pd_hand);

    let mut good_leads = Cards::new();
    let mut high_leads = Cards::new();
    let mut normal_leads = Cards::new();
    let mut bad_leads = Cards::new();
    let mut trump_leads = Cards::new();
    let mut ruff_leads = Cards::new();

    let is_suit_contract = trump < NOTRUMP;

    for suit in 0..NUM_SUITS {
        let my_suit = playable.suit(suit);
        if my_suit.is_empty() {
            continue;
        }

        // Handle trump suit specially in suit contracts
        if is_suit_contract && suit == trump {
            trump_leads.add(my_suit.top());
            if my_suit.size() > 1 {
                trump_leads.add(my_suit.bottom());
            }
            continue;
        }

        // Skip suits where opponents can ruff
        if is_suit_contract {
            if lho_hand.suit(trump).size() > 0 && lho_hand.suit(suit).is_empty() {
                continue;
            }
            if rho_hand.suit(trump).size() > 0 && rho_hand.suit(suit).is_empty() {
                continue;
            }
        }

        let pd_suit = pd_hand.suit(suit);
        let lho_suit = lho_hand.suit(suit);
        let rho_suit = rho_hand.suit(suit);
        let all_suit = all_cards.suit(suit);

        // Get relative ranks (A, K, Q, J, T) in this suit
        let a = if !all_suit.is_empty() { all_suit.top() } else { continue };
        let all_minus_a = all_suit.different(Cards::from_bits(1u64 << a));
        let k = if !all_minus_a.is_empty() { all_minus_a.top() } else { a };
        let all_minus_ak = all_minus_a.different(Cards::from_bits(1u64 << k));
        let q = if !all_minus_ak.is_empty() { all_minus_ak.top() } else { k };
        let all_minus_akq = all_minus_ak.different(Cards::from_bits(1u64 << q));
        let j = if !all_minus_akq.is_empty() { all_minus_akq.top() } else { q };
        let all_minus_akqj = all_minus_akq.different(Cards::from_bits(1u64 << j));
        let t = if !all_minus_akqj.is_empty() { all_minus_akqj.top() } else { j };

        let our_suits = my_suit.union(pd_suit);

        // Check for good leads (finesse positions)
        // Partner has K and LHO has A, etc.
        if pd_suit.size() >= 2 && lho_suit.size() >= 2 {
            let mut qj = Cards::new();
            qj.add(q);
            qj.add(j);
            let mut jt = Cards::new();
            jt.add(j);
            jt.add(t);

            if (pd_suit.have(k) && lho_suit.have(a))
                || (pd_suit.have(a) && lho_suit.have(k) && (pd_suit.have(q) || our_suits.include(qj)))
                || (pd_suit.have(k) && lho_suit.have(q) && (pd_suit.have(j) || our_suits.include(jt)))
            {
                good_leads.add(my_suit.top());
                if my_suit.size() > 1 {
                    good_leads.add(my_suit.bottom());
                }
                continue;
            }
        }

        // Check for bad leads (high card in front of RHO's higher card)
        if my_suit.size() >= 2 && rho_suit.size() >= 2 {
            if (my_suit.have(a) && rho_suit.have(k))
                || (my_suit.have(k) && rho_suit.have(a) && !partnership_cards.have(q))
            {
                if is_suit_contract {
                    bad_leads.add(my_suit.top());
                    if my_suit.size() > 1 {
                        bad_leads.add(my_suit.bottom());
                    }
                }
                continue;
            }
        }

        // Check for high leads (both sides have A/K/Q)
        let mut akq = Cards::new();
        akq.add(a);
        akq.add(k);
        akq.add(q);
        if !lho_suit.is_empty() && !rho_suit.is_empty() && partnership_cards.intersect(akq).size() >= 2 {
            high_leads.add(my_suit.top());
            if my_suit.size() > 1 {
                high_leads.add(my_suit.bottom());
            }
            continue;
        }

        // Check for ruff leads (partner can ruff)
        if is_suit_contract && pd_suit.is_empty() && !lho_suit.is_empty() && !rho_suit.is_empty()
            && pd_hand.suit(trump).size() > 0
            && pd_hand.suit(trump).size() <= playable.suit(trump).size()
            && my_suit.bottom() != a
        {
            ruff_leads.add(my_suit.bottom());
            continue;
        }

        // Normal leads (top and bottom)
        normal_leads.add(my_suit.top());
        if my_suit.size() > 1 {
            normal_leads.add(my_suit.bottom());
        }
    }

    // Add in priority order
    if is_suit_contract {
        ordered.add_cards(ruff_leads);
        remaining.remove_cards(ruff_leads);
    }
    ordered.add_cards(good_leads);
    remaining.remove_cards(good_leads);
    ordered.add_cards(high_leads);
    remaining.remove_cards(high_leads);
    ordered.add_cards(normal_leads);
    remaining.remove_cards(normal_leads);
    if is_suit_contract {
        ordered.add_cards(bad_leads);
        remaining.remove_cards(bad_leads);
        ordered.add_cards(trump_leads);
        remaining.remove_cards(trump_leads);
    }
    // Add any remaining cards
    ordered.add_cards(remaining);

    ordered
}

/// Order follow cards (when following suit or discarding)
/// Matches the C++ OrderCards logic for better move ordering
pub fn order_follows(
    playable: Cards,
    hands: &Hands,
    seat: Seat,
    trump: usize,
    lead_suit: Suit,
    winning_seat: Seat,
    winning_card: usize,
    card_in_trick: usize,
    wins_over: impl Fn(usize, usize) -> bool,
) -> OrderedCards {
    let mut ordered = OrderedCards::new();

    let pd_suit = hands[partner(seat)].suit(lead_suit);
    let lho_suit = hands[left_hand_opp(seat)].suit(lead_suit);

    let trick_ending = card_in_trick == 3;
    let second_seat = card_in_trick == 1;

    // Helper to check if card1 is higher rank than card2 (lower index = higher rank)
    let higher_rank = |c1: usize, c2: usize| c1 < c2;

    // Following suit?
    let my_suit = playable.suit(lead_suit);
    if !my_suit.is_empty() {
        // Can't beat current winner - play low first
        if !wins_over(my_suit.top(), winning_card) {
            ordered.add_reversed(playable);
            return ordered;
        }

        // Partner is winning - check if we should play low
        if winning_seat == partner(seat) {
            // Play low if:
            // - Trick is ending (partner wins)
            // - LHO has no cards in suit
            // - Partner's winning card beats LHO's best
            // - LHO's options above partner's card equals LHO's options above our best
            //   (meaning we can't improve the situation by playing high)
            if trick_ending
                || lho_suit.is_empty()
                || higher_rank(winning_card, lho_suit.top())
                || lho_suit.slice(0, winning_card) == lho_suit.slice(0, my_suit.top())
            {
                ordered.add_reversed(playable);
                return ordered;
            }
        }

        // Second seat analysis - should we duck for partner?
        if second_seat && !pd_suit.is_empty() && higher_rank(pd_suit.top(), winning_card) {
            let combined = pd_suit.union(my_suit);
            // If LHO has a higher card than our combined best, and their options
            // above partner's card equals options above our best, play low
            if !lho_suit.is_empty()
                && higher_rank(lho_suit.top(), combined.top())
                && lho_suit.slice(0, pd_suit.top()) == lho_suit.slice(0, my_suit.top())
            {
                ordered.add_reversed(playable);
                return ordered;
            }
            // If LHO can't beat partner, play low
            if lho_suit.is_empty() || higher_rank(pd_suit.top(), lho_suit.top()) {
                ordered.add_reversed(playable);
                return ordered;
            }
        }

        // Split cards into those that beat the winner and those that don't
        let higher_cards = my_suit.slice(0, winning_card);
        let lower_cards = my_suit.different(higher_cards);

        // Order higher cards based on whether we need to beat LHO
        if trick_ending
            || lho_suit.is_empty()
            || higher_rank(higher_cards.bottom(), lho_suit.top())
        {
            // We can safely play low among our winning cards
            ordered.add_reversed(higher_cards);
        } else {
            // Try high cards first (might need to beat LHO)
            ordered.add_cards(higher_cards);
        }
        // Add lower cards (low first)
        ordered.add_reversed(lower_cards);
        return ordered;
    }

    // Not following suit - ruff or discard
    let is_suit_contract = trump < NOTRUMP;
    let my_trumps = if is_suit_contract { playable.suit(trump) } else { Cards::new() };

    if !my_trumps.is_empty() {
        // Can ruff
        let lho_has_trumps = !hands[left_hand_opp(seat)].suit(trump).is_empty();

        // Check if partner is winning and can hold the trick
        let partner_winning = winning_seat == partner(seat);
        if partner_winning && (trick_ending || (!lho_suit.is_empty() && wins_over(winning_card, lho_suit.top()))) {
            // Partner can win - don't ruff, discard instead
        } else if suit_of(winning_card) == trump {
            // Someone already trumped - try to overruff if possible
            if winning_seat != partner(seat) && wins_over(my_trumps.top(), winning_card) {
                // We can overruff - try higher trumps first
                let higher_trumps = my_trumps.slice(my_trumps.top(), winning_card);
                ordered.add_reversed(higher_trumps);
                // Then add the rest of playable cards
                let remaining = playable.different(higher_trumps);
                add_discards(&mut ordered, remaining, trump);
                return ordered;
            }
        } else if trick_ending || !lho_suit.is_empty() || !lho_has_trumps {
            // The lowest trump is guaranteed to win
            ordered.add(my_trumps.bottom());
            let remaining = playable.different(Cards::from_bits(1u64 << my_trumps.bottom()));
            add_discards(&mut ordered, remaining, trump);
            return ordered;
        } else {
            // LHO might overruff - try trumps high to low
            ordered.add_reversed(my_trumps);
            let remaining = playable.different(my_trumps);
            add_discards(&mut ordered, remaining, trump);
            return ordered;
        }
    }

    // Discard - try bottom card from each suit first
    add_discards(&mut ordered, playable, trump);
    ordered
}

/// Add discards matching C++ logic:
/// 1. For each non-trump suit, add the bottom (lowest) card
/// 2. Sort those discards by suit length (longer suits first)
/// 3. Add remaining cards
fn add_discards(ordered: &mut OrderedCards, mut playable: Cards, trump: usize) {
    // Collect bottom card from each non-trump suit, tracking suit lengths
    let mut discards: [(usize, usize); 4] = [(0, 0); 4]; // (card, suit_length)
    let mut num_discards = 0;

    for suit in 0..4 {
        if suit == trump {
            continue;
        }
        let suit_cards = playable.suit(suit);
        if !suit_cards.is_empty() {
            let bottom = suit_cards.bottom();
            // Count how many cards remain in this suit after removing bottom
            let remaining_in_suit = playable.suit(suit).size();
            discards[num_discards] = (bottom, remaining_in_suit);
            num_discards += 1;
            playable.remove(bottom);
        }
    }

    // Sort discards by suit length (longer suits first) - stable sort to preserve suit order for ties
    discards[..num_discards].sort_by(|a, b| b.1.cmp(&a.1));

    // Add sorted discards
    for i in 0..num_discards {
        ordered.add(discards[i].0);
    }

    // Add remaining cards
    ordered.add_cards(playable);
}

/// Double-dummy solver
pub struct Solver {
    hands: Hands,
    trump: usize,
    initial_leader: Seat,
    num_tricks: usize,
}

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
pub(crate) static NODE_COUNT: AtomicU64 = AtomicU64::new(0);
pub(crate) static XRAY_COUNT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static XRAY_LIMIT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static NO_PRUNING: AtomicBool = AtomicBool::new(false);
pub(crate) static NO_TT: AtomicBool = AtomicBool::new(false);
pub(crate) static NO_RANK_SKIP: AtomicBool = AtomicBool::new(false);
pub(crate) static SHOW_PERF: AtomicBool = AtomicBool::new(false);

/// Get the node count from the last solve (for profiling)
pub fn get_node_count() -> u64 {
    NODE_COUNT.load(Ordering::Relaxed)
}

/// Set xray tracing limit (0 = disabled)
pub fn set_xray_limit(limit: usize) {
    XRAY_LIMIT.store(limit, Ordering::Relaxed);
    XRAY_COUNT.store(0, Ordering::Relaxed);
}

/// Check if xray tracing is enabled
pub fn xray_enabled() -> bool {
    XRAY_LIMIT.load(Ordering::Relaxed) > 0
}

/// Check if xray logging should occur (enabled and under limit)
pub(crate) fn xray_should_log() -> bool {
    let limit = XRAY_LIMIT.load(Ordering::Relaxed);
    limit > 0 && XRAY_COUNT.load(Ordering::Relaxed) <= limit
}

/// Set no-pruning mode (disables fast/slow tricks pruning for debugging)
pub fn set_no_pruning(enabled: bool) {
    NO_PRUNING.store(enabled, Ordering::Relaxed);
}

/// Set no-TT mode (disables transposition table for debugging)
pub fn set_no_tt(enabled: bool) {
    NO_TT.store(enabled, Ordering::Relaxed);
}

/// Set no-rank-skip mode (disables min_relevant_ranks optimization for debugging)
pub fn set_no_rank_skip(enabled: bool) {
    NO_RANK_SKIP.store(enabled, Ordering::Relaxed);
}

/// Set show-perf mode (outputs [PERF] lines to stderr after each solve)
pub fn set_show_perf(enabled: bool) {
    SHOW_PERF.store(enabled, Ordering::Relaxed);
}

impl Solver {
    /// Create a new solver
    pub fn new(hands: Hands, trump: usize, initial_leader: Seat) -> Self {
        let num_tricks = hands.num_tricks();
        Solver {
            hands,
            trump,
            initial_leader,
            num_tricks,
        }
    }

    /// Solve and return NS tricks
    pub fn solve(&self) -> u8 {
        NODE_COUNT.store(0, Ordering::Relaxed);
        XRAY_COUNT.store(0, Ordering::Relaxed);
        let start = std::time::Instant::now();
        let num_tricks = self.num_tricks;
        let guess = self.guess_tricks();
        let result = self.mtdf_search(num_tricks, guess);
        if SHOW_PERF.load(Ordering::Relaxed) {
            let elapsed = start.elapsed();
            let iterations = NODE_COUNT.load(Ordering::Relaxed);
            let ns_per_iter = if iterations > 0 {
                elapsed.as_nanos() as f64 / iterations as f64
            } else {
                0.0
            };
            eprintln!("[PERF] iterations={}, time={:.3}s, ns/iter={:.1}",
                      iterations, elapsed.as_secs_f64(), ns_per_iter);
        }
        result
    }

    /// MTD(f) search driver
    fn mtdf_search(&self, num_tricks: usize, guess: usize) -> u8 {
        let mut cutoff_cache = search::CutoffCache::new(16);
        let mut pattern_cache = super::pattern::PatternCache::new(16);
        let mut hands = self.hands;

        let mut lower = 0i8;
        let mut upper = num_tricks as i8;
        let mut ns_tricks = guess as i8;

        while lower < upper {
            let beta = if ns_tricks == lower {
                ns_tricks + 1
            } else {
                ns_tricks
            };

            let mut searcher = search::Search::new(
                &mut hands,
                self.trump,
                self.initial_leader,
                &mut cutoff_cache,
                &mut pattern_cache,
            );
            ns_tricks = searcher.search(beta) as i8;

            if ns_tricks < beta {
                upper = ns_tricks;
            } else {
                lower = ns_tricks;
            }
        }

        lower as u8
    }

    /// Count fast tricks for a suit, properly handling entries and blocking.
    ///
    /// This follows the C++ SuitFastTricks logic:
    /// - If partner has no winners: return my winners
    /// - If I have no winners: return partner's winners only if I have cards in suit
    /// - If suit is blocked by partner (my top < partner's bottom): return partner's winners
    /// - If suit is blocked by me (my bottom > partner's top): return my winners
    /// - Otherwise: return min(my cards, my winners + partner winners - 1)
    ///   The -1 is because if partner has no small cards, one winner acts as transport
    fn suit_fast_tricks(
        my_suit: Cards,
        my_winners: usize,
        pd_suit: Cards,
        pd_winners: usize,
        pd_entry: &mut bool,
    ) -> usize {
        // Entry from partner if my top winner can cover partner's bottom card.
        if !pd_suit.is_empty() && my_winners > 0 && higher_rank(my_suit.top(), pd_suit.bottom()) {
            *pd_entry = true;
        }
        // Partner has no winners.
        if pd_winners == 0 {
            return my_winners;
        }
        // Cash all my winners, then partner's - but only if I have cards to lead
        if my_winners == 0 {
            return if !my_suit.is_empty() { pd_winners } else { 0 };
        }
        // Suit blocked by partner (my top is lower than partner's bottom)
        if !pd_suit.is_empty() && lower_rank(my_suit.top(), pd_suit.bottom()) {
            return pd_winners;
        }
        // Suit blocked by me (my bottom is higher than partner's top)
        if !pd_suit.is_empty() && higher_rank(my_suit.bottom(), pd_suit.top()) {
            return my_winners;
        }
        // If partner has no small cards, treat one winner as a small card
        let adjusted_pd_winners = if pd_winners == pd_suit.size() && pd_winners > 0 {
            pd_winners - 1
        } else {
            pd_winners
        };
        my_suit.size().min(my_winners + adjusted_pd_winners)
    }

    /// Count guaranteed fast tricks from a given seat's perspective.
    ///
    /// This properly handles entries and blocking between the two hands.
    fn fast_tricks_from_seat(&self, hands: &Hands, seat: Seat, all_cards: Cards) -> usize {
        let my_hand = hands[seat];
        let pd_hand = hands[partner(seat)];

        let mut my_tricks = 0;
        let mut pd_tricks = 0;
        let mut my_entry = false;  // Can I overtake partner's cards (get the lead from partner)?
        let mut pd_entry = false;  // Can partner overtake my cards (take the lead from me)?

        for suit in 0..NUM_SUITS {
            // Skip trump suit in trump contracts (handled separately)
            if self.trump < NOTRUMP && suit == self.trump {
                continue;
            }

            let my_suit = my_hand.suit(suit);
            let pd_suit = pd_hand.suit(suit);
            let all_suit = all_cards.suit(suit);

            if my_suit.is_empty() && pd_suit.is_empty() {
                continue;
            }

            // Count winners for each hand
            let mut my_winners = 0;
            let mut pd_winners = 0;
            for card in all_suit.iter() {
                if my_suit.have(card) {
                    my_winners += 1;
                } else if pd_suit.have(card) {
                    pd_winners += 1;
                } else {
                    break; // First card not in our partnership ends the winners
                }
            }

            // From my perspective: sets my_entry if I can overtake partner
            my_tricks += Self::suit_fast_tricks(my_suit, my_winners, pd_suit, pd_winners, &mut my_entry);

            // From partner's perspective: sets pd_entry if partner can overtake me
            pd_tricks += Self::suit_fast_tricks(pd_suit, pd_winners, my_suit, my_winners, &mut pd_entry);
        }

        // If partner can overtake me (pd_entry), partner can take the lead.
        // In that case, we can cash whichever line gives more tricks.
        if pd_entry {
            my_tricks.max(pd_tricks)
        } else {
            my_tricks
        }
    }

    /// Count guaranteed fast tricks for the side to play.
    /// Returns the number of tricks that can be cashed immediately.
    fn fast_tricks(&self, hands: &Hands, seat_to_play: Seat, max_tricks: usize) -> usize {
        let all_cards = hands.all_cards();
        let tricks = self.fast_tricks_from_seat(hands, seat_to_play, all_cards);
        #[cfg(feature = "debug_mtdf")]
        if tricks > 0 && max_tricks >= 10 {
            eprintln!("  fast_tricks: seat={}, tricks={}, max={}", seat_to_play, tricks, max_tricks);
        }
        tricks.min(max_tricks)
    }

    /// Count slow tricks for EW when NS is leading (NT contracts)
    /// Slow tricks are guaranteed tricks that require giving up the lead first.
    /// For example, Kx behind RHO's Ace is a guaranteed trick via finesse.
    fn slow_tricks_ew_nt(&self, hands: &Hands, seat_to_play: Seat) -> usize {
        // EW's hands (opponents of current player)
        let lho_hand = hands[left_hand_opp(seat_to_play)];
        let rho_hand = hands[right_hand_opp(seat_to_play)];

        // NS's hands (the side currently leading)
        let my_hand = hands[seat_to_play];
        let pd_hand = hands[partner(seat_to_play)];
        let ns_cards = my_hand.union(pd_hand);

        let all_cards = hands.all_cards();
        let mut rank_winners = Cards::new();

        // For each suit where NS has cards
        for suit in 0..NUM_SUITS {
            if my_hand.suit(suit).is_empty() {
                continue;
            }
            let all_suit = all_cards.suit(suit);
            if all_suit.is_empty() {
                continue;
            }
            let top = all_suit.top();
            // If NS has the top card, no slow trick for EW in this suit
            if ns_cards.have(top) {
                return 0; // C++ returns {0, {}} immediately if any suit has NS top
            }
            rank_winners.add(top);
        }

        if rank_winners.is_empty() {
            return 0;
        }

        // Check if all rank winners are in one opponent's hand
        if lho_hand.include(rank_winners) || rho_hand.include(rank_winners) {
            // All winners in one hand - EW gets all of them
            rank_winners.size()
        } else {
            // Winners split - EW gets at least 1 (could be blocked)
            1
        }
    }

    /// Count slow tricks for NS when EW is leading (NT contracts)
    fn slow_tricks_ns_nt(&self, hands: &Hands, seat_to_play: Seat) -> usize {
        // NS's hands (opponents of current player)
        let lho_hand = hands[left_hand_opp(seat_to_play)];
        let rho_hand = hands[right_hand_opp(seat_to_play)];

        // EW's hands (the side currently leading)
        let my_hand = hands[seat_to_play];
        let pd_hand = hands[partner(seat_to_play)];
        let ew_cards = my_hand.union(pd_hand);

        let all_cards = hands.all_cards();
        let mut rank_winners = Cards::new();

        // For each suit where EW (current side) has cards
        for suit in 0..NUM_SUITS {
            if my_hand.suit(suit).is_empty() {
                continue;
            }
            let all_suit = all_cards.suit(suit);
            if all_suit.is_empty() {
                continue;
            }
            let top = all_suit.top();
            // If EW has the top card, no slow trick for NS in this suit
            if ew_cards.have(top) {
                return 0;
            }
            rank_winners.add(top);
        }

        if rank_winners.is_empty() {
            return 0;
        }

        // Check if all rank winners are in one defender's hand
        if lho_hand.include(rank_winners) || rho_hand.include(rank_winners) {
            rank_winners.size()
        } else {
            1
        }
    }

    /// Count slow tricks in trump contracts
    /// Checks for protected honor positions like Kx behind A
    fn slow_tricks_trump(&self, hands: &Hands, seat_to_play: Seat, leading: bool) -> usize {
        if self.trump >= NOTRUMP {
            return 0;
        }

        let my_trumps = hands[seat_to_play].suit(self.trump);
        let pd_trumps = hands[partner(seat_to_play)].suit(self.trump);
        let lho_trumps = hands[left_hand_opp(seat_to_play)].suit(self.trump);
        let rho_trumps = hands[right_hand_opp(seat_to_play)].suit(self.trump);

        let all_trumps = hands.all_cards().suit(self.trump);

        if all_trumps.size() < 3 {
            return 0;
        }

        // Get relative A, K, Q
        let a = all_trumps.top();
        let all_minus_a = all_trumps.different(Cards::from_bits(1u64 << a));
        let k = if !all_minus_a.is_empty() { all_minus_a.top() } else { return 0; };

        // Kx behind A (partner has Kx, LHO has A)
        // StrictlyInclude means has K and at least one other card
        if pd_trumps.have(k) && pd_trumps.size() >= 2 && lho_trumps.have(a) {
            return 1;
        }

        // Kx behind A (we have Kx, RHO has A) - need tempo to lead toward K
        if my_trumps.have(k) && my_trumps.size() >= 2 && rho_trumps.have(a) {
            // Only works if not leading or we have enough tricks to give up tempo
            if !leading || self.num_tricks >= 3 {
                return 1;
            }
        }

        // KQ against A
        let all_minus_ak = all_minus_a.different(Cards::from_bits(1u64 << k));
        let q = if !all_minus_ak.is_empty() { all_minus_ak.top() } else { return 0; };

        let opp_trumps = lho_trumps.union(rho_trumps);
        let our_trumps = my_trumps.union(pd_trumps);

        if opp_trumps.have(a) && (our_trumps.have(k) || our_trumps.have(q))
           && (my_trumps.size() >= 1 || pd_trumps.size() >= 1)
        {
            // We have KQ or K or Q, opponents have A - can force out A for 1 trick
            if our_trumps.have(k) && our_trumps.have(q) {
                return 1;
            }
        }

        // Qxx behind AK (needs 5+ trumps total)
        if all_trumps.size() >= 5 {
            // Partner has Qxx, LHO has AK
            if pd_trumps.have(q) && pd_trumps.size() >= 3
               && lho_trumps.have(a) && lho_trumps.have(k)
            {
                return 1;
            }
            // We have Qxx, RHO has AK
            if my_trumps.have(q) && my_trumps.size() >= 3
               && rho_trumps.have(a) && rho_trumps.have(k)
               && (!leading || self.num_tricks >= 4)
            {
                return 1;
            }
        }

        0
    }

    /// Estimate starting tricks for MTD(f)
    /// Ported from C++ GuessTricks() for consistent behavior
    fn guess_tricks(&self) -> usize {
        let num_tricks = self.num_tricks;
        let ns_points = self.hands[NORTH].points() + self.hands[SOUTH].points();
        let ew_points = self.hands[EAST].points() + self.hands[WEST].points();

        if self.trump >= NOTRUMP {
            // NT contract
            if ns_points * 2 < ew_points {
                return 0;
            }
            if ns_points < ew_points {
                return num_tricks / 2 + 1;
            }
        } else {
            // Suit contract - compare points AND trump length
            let n_trumps = self.hands[NORTH].suit(self.trump).size();
            let s_trumps = self.hands[SOUTH].suit(self.trump).size();
            let e_trumps = self.hands[EAST].suit(self.trump).size();
            let w_trumps = self.hands[WEST].suit(self.trump).size();

            let ns_max_trumps = n_trumps.max(s_trumps);
            let ew_max_trumps = e_trumps.max(w_trumps);

            if ns_points < ew_points
                && (ns_max_trumps < ew_max_trumps
                    || (ns_max_trumps == ew_max_trumps
                        && n_trumps + s_trumps < e_trumps + w_trumps))
            {
                return 0;
            }
        }

        num_tricks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_1_trick() {
        // Single trick - NS has ace, EW has king
        // N: SA  E: SK  S: S2  W: S3
        let hands = Hands::from_pbn(
            "N:A... K... 2... 3..."
        ).unwrap();

        // West leads - EW has the lead but NS has the ace
        let solver = Solver::new(hands, NOTRUMP, WEST);
        let ns_tricks = solver.solve();
        assert_eq!(ns_tricks, 1); // NS wins with the ace
    }

    #[test]
    fn test_solver_1_trick_ew_wins() {
        // Single trick - EW has ace
        // N: SK  E: SA  S: S2  W: S3
        let hands = Hands::from_pbn(
            "N:K... A... 2... 3..."
        ).unwrap();

        // West leads
        let solver = Solver::new(hands, NOTRUMP, WEST);
        let ns_tricks = solver.solve();
        assert_eq!(ns_tricks, 0); // EW wins with the ace
    }

    #[test]
    fn test_solver_2_tricks() {
        // Two tricks - NS has both aces
        // N: SA,HA  E: SK,HK  S: S2,H2  W: S3,H3
        let hands = Hands::from_pbn(
            "N:A.A.. K.K.. 2.2.. 3.3.."
        ).unwrap();

        // West leads
        let solver = Solver::new(hands, NOTRUMP, WEST);
        let ns_tricks = solver.solve();
        assert_eq!(ns_tricks, 2); // NS wins both tricks
    }

    #[test]
    fn test_solver_4_tricks() {
        // Four tricks - NS has all aces
        let hands = Hands::from_pbn(
            "N:A.A.A.A K.K.K.K 2.2.2.2 3.3.3.3"
        ).unwrap();

        // West leads
        let solver = Solver::new(hands, NOTRUMP, WEST);
        let ns_tricks = solver.solve();
        assert_eq!(ns_tricks, 4); // NS wins all 4 tricks
    }

    #[test]
    fn test_solver_8_tricks() {
        // 8 tricks - NS has AK in each suit
        let hands = Hands::from_pbn(
            "N:AK.AK.AK.AK QJ.QJ.QJ.QJ 32.32.32.32 T9.T9.T9.T9"
        ).unwrap();

        // West leads
        let start = std::time::Instant::now();
        let solver = Solver::new(hands, NOTRUMP, WEST);
        let ns_tricks = solver.solve();
        eprintln!("8-trick test took {:?}", start.elapsed());
        assert_eq!(ns_tricks, 8); // NS wins 8 tricks
    }

    #[test]
    #[ignore] // 13-card tests disabled until pruning optimizations are implemented
    fn test_solver_cold_13() {
        // NS has all top cards
        let hands = Hands::from_pbn(
            "N:AKQJ.AKQ.AKQ.AKQ T987.JT9.JT9.JT9 6543.876.876.876 2.5432.5432.5432"
        ).unwrap();

        eprintln!("Hands parsed, starting solve...");
        let solver = Solver::new(hands, NOTRUMP, WEST);
        let start = std::time::Instant::now();
        let ns_tricks = solver.solve();
        eprintln!("Solve took {:?}", start.elapsed());
        assert_eq!(ns_tricks, 13);
    }

    #[test]
    #[ignore] // 13-card tests disabled until pruning optimizations are implemented
    fn test_solver_cold_0() {
        // EW has all top cards
        let hands = Hands::from_pbn(
            "N:T987.JT9.JT9.JT9 AKQJ.AKQ.AKQ.AKQ 2.5432.5432.5432 6543.876.876.876"
        ).unwrap();

        let solver = Solver::new(hands, NOTRUMP, WEST);
        let ns_tricks = solver.solve();
        assert_eq!(ns_tricks, 0);
    }

    #[test]
    #[ignore] // 13-card tests disabled until pruning optimizations are implemented
    fn test_solver_9_tricks() {
        // From test case
        let hands = Hands::from_pbn(
            "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72"
        ).unwrap();

        let start = std::time::Instant::now();
        let solver = Solver::new(hands, NOTRUMP, WEST);
        let ns_tricks = solver.solve();
        eprintln!("9-trick test took {:?}", start.elapsed());
        assert_eq!(ns_tricks, 9);
    }

    #[test]
    #[ignore] // 13-card tests disabled until pruning optimizations are implemented
    fn test_solver_13card_north_only() {
        // Same 13-card deal, but only test North leading
        let hands = Hands::from_pbn(
            "N:AKQT3.J6.KJ42.95 652.AK42.AQ87.T4 J74.QT95.T.AK863 98.873.9653.QJ72"
        ).unwrap();

        let start = std::time::Instant::now();
        let solver = Solver::new(hands, NOTRUMP, NORTH);
        let ns_tricks = solver.solve();
        let nodes = get_node_count();
        eprintln!("13-card North lead test: {} tricks, {:?}, {} nodes", ns_tricks, start.elapsed(), nodes);
        // Note: Expected value needs verification with C++ solver
    }

}
