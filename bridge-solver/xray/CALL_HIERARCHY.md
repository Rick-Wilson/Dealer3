# Solver Call Hierarchy Comparison

## Overview

This document compares the high-level call structure between the C++ reference solver and the Rust solver2 implementation.

## C++ Solver (solver_xray.cc)

```
MTDf()                              // Line ~1175 - MTD(f) driver
│
└── SearchWithCache()               // Line ~1028 - Called at TRICK BOUNDARIES
    │
    │   // At trick start (TrickStarting() == true):
    │   //   - Sets trick->all_cards = hands.all_cards()  [Line 1053]
    │   //   - Does XRAY logging
    │   //   - Checks transposition table
    │
    └── SearchAtTrickStart()        // Line ~1101 - Fast/slow tricks pruning
        │
        │   // Pruning checks:
        │   //   - Fast tricks estimation
        │   //   - Slow tricks estimation (NT only)
        │
        └── EvaluatePlayableCards() // Line ~1138 - Main card evaluation loop
            │
            │   // For each playable card:
            │   //   - IsEquivalent() check using trick->all_cards
            │   //   - PlayCard() to make the move
            │   //   - Recursive call to SearchWithCache()
            │
            └── IsEquivalent()      // Line ~917 - Equivalence check
                │
                │   // Uses trick->all_cards (computed at trick start)
                │   // Logs EQUIV: when xray_iterations > 0
                │
                └── (returns bool)
```

### Key C++ Data Flow

```
SearchWithCache (depth=0, trick 0 start)
    ├── trick->all_cards = hands.all_cards()  // Computed ONCE here
    ├── SearchAtTrickStart
    │   └── EvaluatePlayableCards
    │       ├── IsEquivalent(card, tried, trick->all_cards, hand)  // depth 0
    │       ├── PlayCard(card)  // removes card from hands
    │       └── SearchWithCache (depth=1, mid-trick)
    │           ├── (trick->all_cards UNCHANGED - still from depth 0)
    │           └── EvaluatePlayableCards
    │               ├── IsEquivalent(card, tried, trick->all_cards, hand)  // depth 1, same all_cards
    │               ├── PlayCard(card)
    │               └── SearchWithCache (depth=2, mid-trick)
    │                   └── ... (trick->all_cards still from depth 0)
```

---

## Rust Solver (solver2/solver.rs)

```
mtdf_search()                       // Line ~634 - MTD(f) driver
│
└── alpha_beta_search()             // Line ~676 - Sets up state arrays
    │
    │   // Initializes:
    │   //   - cards_played, seats, lead_suits, winning_card_idx
    │   //   - trick_all_cards[TOTAL_TRICKS]  // NEW: stores all_cards per trick
    │
    └── search_recursive()          // Line ~710 - Main recursive search
        │
        │   // At trick start (card_in_trick == 0):
        │   //   - Stores trick_all_cards[trick_idx] = hands.all_cards()
        │   //   - Does XRAY logging
        │   //   - Checks transposition table
        │
        │   // At ALL positions:
        │   //   - Uses all_cards = trick_all_cards[trick_idx]
        │   //   - Fast/slow tricks pruning (at trick start)
        │
        ├── order_leads()           // Line ~267 - Lead ordering (at trick start)
        │   └── (uses all_cards for ordering heuristics)
        │
        ├── order_follows()         // Line ~426 - Follow ordering (mid-trick)
        │
        ├── is_equivalent()         // Line ~200 - Equivalence check
        │   │
        │   │   // Uses all_cards = trick_all_cards[trick_idx]
        │   │   // Logs EQUIV: when XRAY_LIMIT > 0
        │   │
        │   └── (returns bool)
        │
        └── play_card_and_continue() // Line ~1019 - Play and recurse
            │
            │   // Removes card from hand
            │   // Updates trick state
            │   // Calls search_recursive(depth + 1, ...)
            │
            └── search_recursive()  // Recursive call
```

### Key Rust Data Flow

```
search_recursive (depth=0, trick 0 start)
    ├── trick_all_cards[0] = hands.all_cards()  // Computed ONCE here
    ├── all_cards = trick_all_cards[0]
    ├── is_equivalent(card, tried, all_cards, hand)  // depth 0
    ├── play_card_and_continue(card)  // removes card from hands
    │   └── search_recursive (depth=1, mid-trick)
    │       ├── all_cards = trick_all_cards[0]  // REUSES value from depth 0
    │       ├── is_equivalent(card, tried, all_cards, hand)  // depth 1, same all_cards
    │       ├── play_card_and_continue(card)
    │       │   └── search_recursive (depth=2, mid-trick)
    │       │       ├── all_cards = trick_all_cards[0]  // Still from depth 0
    │       │       └── ...
```

---

## Structural Comparison

| Aspect | C++ | Rust |
|--------|-----|------|
| **Layers** | 3 layers: SearchWithCache → SearchAtTrickStart → EvaluatePlayableCards | 1 layer: search_recursive (flat) |
| **all_cards storage** | `trick->all_cards` member variable | `trick_all_cards[trick_idx]` array |
| **When all_cards computed** | At trick start in SearchWithCache | At trick start in search_recursive |
| **all_cards reuse** | Automatic via trick struct | Via array lookup |
| **XRAY logging location** | SearchWithCache | search_recursive |
| **TT check location** | SearchWithCache | search_recursive |
| **Pruning location** | SearchAtTrickStart | search_recursive |
| **Card loop location** | EvaluatePlayableCards | search_recursive |

---

## Key Files and Line Numbers

### C++ (solver_xray.cc)
- `IsEquivalent()`: Line 917
- `SearchWithCache()`: Line 1028
- `trick->all_cards = hands.all_cards()`: Line 1053
- `SearchAtTrickStart()`: Line 1101
- `EvaluatePlayableCards()`: Line 1138
- `MTDf()`: Line 1175

### Rust (solver2/solver.rs)
- `is_equivalent()`: Line 200
- `order_leads()`: Line 267
- `order_follows()`: Line 426
- `mtdf_search()`: Line 634
- `alpha_beta_search()`: Line 676
- `search_recursive()`: Line 710
- `trick_all_cards[trick_idx] = hands.all_cards()`: Line 863
- `play_card_and_continue()`: Line 1019

---

## Current Status

✅ **Aligned**: `all_cards` is now computed once per trick in both implementations

❌ **Still diverging**: Search paths differ, causing different iteration counts
- C++ finds cutoffs faster
- Likely due to move ordering differences or other algorithm details
