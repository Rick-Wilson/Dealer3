//! Test SlowTrumpTricks against C++ reference implementation
//! Run: cargo test --example test_slow_trump_tricks
//!
//! NOTE: The C++ solver_xray.cc has a BUG at lines 1618-1619 where Have(Cards)
//! silently converts to Have(1) due to operator bool(). This means the
//! "KQ against A" pattern check is effectively disabled in the C++ code.
//!
//! For iteration lockstep, we match the BUGGY C++ behavior.
//! Only "Kx behind A" and "Qxx behind AK" patterns are actually working.

use dealer_dds::solver2::{
    Hands, WEST, NORTH, EAST, SOUTH, SPADE, HEART, DIAMOND, CLUB,
    slow_trump_tricks_opponent,
};

// ============================================================================
// Test 1: Kx behind A (pd has Kx, lho has A) - THIS PATTERN WORKS IN C++
// N=K2, W=A, E=Q, S=J in spades
// ============================================================================

#[test]
fn test_kx_behind_a_pd_has_kx() {
    let hands = Hands::from_solver_format(
        "K2 - - -",  // North has K2
        "A - - -",   // West has A
        "Q - - -",   // East has Q
        "J - - -"    // South has J
    ).unwrap();

    // From West's perspective (opponents are N-S):
    //   my = North, pd = South, lho = East, rho = West
    // Pattern: pd(S) has K? No, N has K. So check from East's perspective.

    // From East's perspective (opponents are N-S):
    //   my = South, pd = North, lho = West, rho = East
    // Pattern: pd(N) has K2 (Kx), lho(W) has A -> matches!
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, EAST, 13, false), 1,
        "East: pd(N) has K2, lho(W) has A -> should return 1");

    // From West's perspective:
    //   my = North, pd = South, lho = East, rho = West
    // Pattern: pd(S) has J (not K), my(N) has K2, rho(W) has A -> matches second case!
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, WEST, 13, false), 1,
        "West: my(N) has K2, rho(W) has A -> should return 1");

    // From North/South perspective - they have the K, opponents have A
    // No finesse FOR opponents
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, NORTH, 13, false), 0);
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, SOUTH, 13, false), 0);
}

// ============================================================================
// Test 2: K singleton - should NOT trigger Kx behind A (need Kx, not just K)
// ============================================================================

#[test]
fn test_k_singleton_no_match() {
    let hands = Hands::from_solver_format(
        "K - - -",   // North has just K (singleton)
        "A - - -",   // West has A
        "Q2 - - -",  // East has Q2
        "J - - -"    // South has J
    ).unwrap();

    // K singleton should NOT match "Kx behind A" pattern
    for seat in [WEST, NORTH, EAST, SOUTH] {
        assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, seat, 13, false), 0,
            "K singleton should not trigger finesse for seat {:?}", seat);
    }
}

// ============================================================================
// Test 3: KQ against A - THIS PATTERN IS BROKEN IN C++ (always returns 0)
// N=A, W=K, E=Q, S=J in spades
// ============================================================================

#[test]
fn test_kq_against_a_buggy_behavior() {
    let hands = Hands::from_solver_format(
        "A - - -",   // North has A
        "K - - -",   // West has K
        "Q - - -",   // East has Q
        "J - - -"    // South has J
    ).unwrap();

    // Due to C++ bug, KQ against A pattern never triggers
    // All seats should return 0
    for seat in [WEST, NORTH, EAST, SOUTH] {
        assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, seat, 13, false), 0,
            "KQ against A is buggy in C++, should return 0 for seat {:?}", seat);
    }
}

// ============================================================================
// Test 4: Qxx behind AK - THIS PATTERN WORKS IN C++
// N=Q32, W=AK, E=J, S=T in spades (need 5+ trumps)
// ============================================================================

#[test]
fn test_qxx_behind_ak() {
    let hands = Hands::from_solver_format(
        "Q32 - - -",  // North has Qxx
        "AK - - -",   // West has AK
        "J - - -",    // East has J
        "T - - -"     // South has T
    ).unwrap();

    // From East's perspective (opponents are N-S):
    //   my = South, pd = North, lho = West, rho = East
    // Pattern: pd(N) has Q with 3+ cards, lho(W) has AK -> matches!
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, EAST, 13, false), 1,
        "East: pd(N) has Q32, lho(W) has AK -> should return 1");

    // From West's perspective:
    //   my = North, pd = South, lho = East, rho = West
    // Pattern: my(N) has Q32, rho(W) has AK -> matches second case!
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, WEST, 13, false), 1,
        "West: my(N) has Q32, rho(W) has AK -> should return 1");

    // From North/South - no finesse for opponents
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, NORTH, 13, false), 0);
    assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, SOUTH, 13, false), 0);
}

// ============================================================================
// Test 5: Only 2 trumps - need >= 3 for any finesse pattern
// ============================================================================

#[test]
fn test_only_two_trumps() {
    let hands = Hands::from_solver_format(
        "K - - -",   // North has K
        "A - - -",   // West has A
        "- - - -",   // East void
        "- - - -"    // South void
    ).unwrap();

    for seat in [WEST, NORTH, EAST, SOUTH] {
        assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, seat, 13, false), 0,
            "Only 2 trumps, need >= 3 for finesse");
    }
}

// ============================================================================
// Test 6: quick_test_4 depth=8 (Hearts: W=4, N=6, E=Q, S=8)
// Q is highest remaining - no A/K present
// Due to C++ bug, this should return 0 for all seats
// ============================================================================

#[test]
fn test_quick_test_4_depth_8() {
    let hands = Hands::from_solver_format(
        "- 6 - -",   // North has H6
        "- 4 - -",   // West has H4
        "- Q - -",   // East has HQ
        "- 8 - -"    // South has H8
    ).unwrap();

    // The "top 3" cards are Q, 8, 6 (calling them a, k, q in code)
    // "KQ against A" pattern would trigger if working, but it's buggy
    // So all should return 0
    for seat in [WEST, NORTH, EAST, SOUTH] {
        assert_eq!(slow_trump_tricks_opponent(&hands, HEART, seat, 4, false), 0,
            "No real A/K/Q finesse with Q-8-6-4, should return 0 for {:?}", seat);
    }
}

// ============================================================================
// Test 7: quick_test_8 full deal
// ============================================================================

#[test]
fn test_quick_test_8_spades() {
    let hands = Hands::from_solver_format(
        "AKQ J6 KJ 9",
        "65 AK4 AQ T",
        "J7 QT9 T AK",
        "98 87 96 QJ"
    ).unwrap();

    // Spades: N=AKQ, W=65, E=J7, S=98
    // No finesse patterns apply
    for seat in [WEST, NORTH, EAST, SOUTH] {
        assert_eq!(slow_trump_tricks_opponent(&hands, SPADE, seat, 8, false), 0);
    }
}

#[test]
fn test_quick_test_8_hearts() {
    let hands = Hands::from_solver_format(
        "AKQ J6 KJ 9",
        "65 AK4 AQ T",
        "J7 QT9 T AK",
        "98 87 96 QJ"
    ).unwrap();

    // Hearts: N=J6, W=AK4, E=QT9, S=87
    // No finesse patterns apply (due to card distribution and C++ bug)
    for seat in [WEST, NORTH, EAST, SOUTH] {
        assert_eq!(slow_trump_tricks_opponent(&hands, HEART, seat, 8, false), 0);
    }
}

#[test]
fn test_quick_test_8_diamonds() {
    let hands = Hands::from_solver_format(
        "AKQ J6 KJ 9",
        "65 AK4 AQ T",
        "J7 QT9 T AK",
        "98 87 96 QJ"
    ).unwrap();

    // Diamonds: N=KJ, W=AQ, E=T, S=96
    // Check for Kx behind A pattern
    // From West: my=N(KJ), pd=S(96), lho=E(T), rho=W(AQ)
    //   my(N) has K with x (KJ), rho(W) has A -> matches!
    assert_eq!(slow_trump_tricks_opponent(&hands, DIAMOND, WEST, 8, false), 1);
    assert_eq!(slow_trump_tricks_opponent(&hands, DIAMOND, EAST, 8, false), 1);
    assert_eq!(slow_trump_tricks_opponent(&hands, DIAMOND, NORTH, 8, false), 0);
    assert_eq!(slow_trump_tricks_opponent(&hands, DIAMOND, SOUTH, 8, false), 0);
}

#[test]
fn test_quick_test_8_clubs() {
    let hands = Hands::from_solver_format(
        "AKQ J6 KJ 9",
        "65 AK4 AQ T",
        "J7 QT9 T AK",
        "98 87 96 QJ"
    ).unwrap();

    // Clubs: N=9, W=T, E=AK, S=QJ
    // No finesse patterns
    for seat in [WEST, NORTH, EAST, SOUTH] {
        assert_eq!(slow_trump_tricks_opponent(&hands, CLUB, seat, 8, false), 0);
    }
}

fn main() {
    println!("Running SlowTrumpTricks tests...");
    println!("Use 'cargo test --example test_slow_trump_tricks' to run all tests");
}
