//! Fast parallel deal generation using stateless independent deals.
//!
//! This module provides a simplified parallel architecture where:
//! - Supervisor generates seeds (trivially fast)
//! - Workers generate deals from seeds (fully independent, no state sharing)
//!
//! This is much more efficient than the legacy parallel module because:
//! 1. No shuffle state dependency between deals
//! 2. Seeds are just u64 values (8 bytes vs ~300 bytes work state)
//! 3. No shared configuration needed (each worker can generate independently)

use dealer_core::{
    generate_deal_from_seed, generate_deal_from_seed_no_predeal, Deal, FastDealConfig,
    FastDealGenerator,
};
use rayon::prelude::*;
use std::sync::Arc;

/// Work unit for fast parallel generation - just a seed and serial number.
#[derive(Clone, Copy)]
pub struct FastWorkUnit {
    /// Serial number for ordering results
    pub serial_number: u64,
    /// Seed for generating this deal
    pub seed: u64,
}

/// Completed work from a fast worker.
pub struct FastCompletedWork {
    /// Serial number for ordering
    pub serial_number: u64,
    /// The generated deal
    pub deal: Deal,
    /// Whether the filter passed
    pub passed: bool,
}

/// Configuration for fast parallel execution.
#[derive(Clone, Default)]
pub struct FastParallelConfig {
    /// Number of worker threads (0 = auto-detect)
    pub num_threads: usize,
}

/// Fast supervisor for parallel deal generation.
///
/// This supervisor is much simpler than the legacy one - it just generates seeds
/// and dispatches them to workers. No complex state management needed.
pub struct FastSupervisor {
    /// The seed generator
    generator: FastDealGenerator,
    /// Predeal configuration (shared via Arc if non-empty)
    predeal_config: Option<Arc<FastDealConfig>>,
    /// Next serial number to assign
    next_serial: u64,
}

impl FastSupervisor {
    /// Create a new fast supervisor.
    pub fn new(seed: u64, parallel_config: FastParallelConfig) -> Self {
        // Configure rayon thread pool if custom thread count specified
        if parallel_config.num_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(parallel_config.num_threads)
                .build_global()
                .ok(); // Ignore error if pool already initialized
        }

        Self {
            generator: FastDealGenerator::new(seed),
            predeal_config: None,
            next_serial: 0,
        }
    }

    /// Create a new fast supervisor with predeal configuration.
    pub fn with_predeal(
        seed: u64,
        predeal_config: FastDealConfig,
        parallel_config: FastParallelConfig,
    ) -> Self {
        if parallel_config.num_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(parallel_config.num_threads)
                .build_global()
                .ok();
        }

        Self {
            generator: FastDealGenerator::with_config(seed, FastDealConfig::new()),
            predeal_config: Some(Arc::new(predeal_config)),
            next_serial: 0,
        }
    }

    /// Generate a batch of work units (just seeds).
    fn generate_batch(&mut self, count: usize) -> Vec<FastWorkUnit> {
        let mut units = Vec::with_capacity(count);

        for _ in 0..count {
            units.push(FastWorkUnit {
                serial_number: self.next_serial,
                seed: self.generator.next_seed(),
            });
            self.next_serial += 1;
        }

        units
    }

    /// Process a batch of work units in parallel.
    ///
    /// Returns results sorted by serial number.
    pub fn process_batch<F>(&mut self, count: usize, filter: F) -> Vec<FastCompletedWork>
    where
        F: Fn(&Deal) -> bool + Sync,
    {
        let units = self.generate_batch(count);

        let mut results: Vec<FastCompletedWork> = if let Some(ref config) = self.predeal_config {
            // With predeal - need to share config
            let config = Arc::clone(config);
            units
                .into_par_iter()
                .map(|unit| {
                    let deal = generate_deal_from_seed(unit.seed, &config);
                    let passed = filter(&deal);
                    FastCompletedWork {
                        serial_number: unit.serial_number,
                        deal,
                        passed,
                    }
                })
                .collect()
        } else {
            // No predeal - fully independent generation
            units
                .into_par_iter()
                .map(|unit| {
                    let deal = generate_deal_from_seed_no_predeal(unit.seed);
                    let passed = filter(&deal);
                    FastCompletedWork {
                        serial_number: unit.serial_number,
                        deal,
                        passed,
                    }
                })
                .collect()
        };

        // Sort by serial number for deterministic output
        results.sort_by_key(|w| w.serial_number);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_supervisor_batch() {
        let config = FastParallelConfig { num_threads: 1 };
        let mut supervisor = FastSupervisor::new(42, config);

        let results = supervisor.process_batch(10, |_| true);

        assert_eq!(results.len(), 10);

        // Check serial numbers are in order
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.serial_number, i as u64);
            assert!(result.passed);
        }
    }

    #[test]
    fn test_fast_supervisor_deterministic() {
        let config = FastParallelConfig { num_threads: 4 };

        let mut sup1 = FastSupervisor::new(123, config.clone());
        let mut sup2 = FastSupervisor::new(123, config);

        let results1 = sup1.process_batch(50, |_| true);
        let results2 = sup2.process_batch(50, |_| true);

        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.serial_number, r2.serial_number);
            assert_eq!(r1.deal, r2.deal);
        }
    }

    #[test]
    fn test_fast_supervisor_filter() {
        let config = FastParallelConfig { num_threads: 2 };
        let mut supervisor = FastSupervisor::new(42, config);

        // Filter: North has >= 15 HCP
        let results = supervisor.process_batch(100, |deal| deal.north.hcp() >= 15);

        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.iter().filter(|r| !r.passed).count();

        assert!(passed > 0, "Expected some deals to pass");
        assert!(failed > 0, "Expected some deals to fail");
        assert_eq!(passed + failed, 100);
    }

    #[test]
    fn test_fast_supervisor_matches_sequential() {
        let seed = 999u64;

        // Sequential generation
        let mut seq_gen = FastDealGenerator::new(seed);
        let sequential_deals: Vec<Deal> = (0..20).map(|_| seq_gen.next_deal()).collect();

        // Parallel generation
        let config = FastParallelConfig { num_threads: 4 };
        let mut supervisor = FastSupervisor::new(seed, config);
        let parallel_results = supervisor.process_batch(20, |_| true);

        // Should match exactly
        for (i, (seq_deal, par_result)) in sequential_deals
            .iter()
            .zip(parallel_results.iter())
            .enumerate()
        {
            assert_eq!(
                seq_deal, &par_result.deal,
                "Deal {} differs between sequential and parallel",
                i
            );
        }
    }

    #[test]
    fn test_fast_work_unit_size() {
        // Work units should be tiny - just 16 bytes (serial + seed)
        let unit_size = std::mem::size_of::<FastWorkUnit>();
        assert_eq!(unit_size, 16, "FastWorkUnit should be exactly 16 bytes");
    }

    #[test]
    fn test_fast_supervisor_with_predeal() {
        use dealer_core::{Card, Position, Rank, Suit};

        let mut predeal = FastDealConfig::new();
        predeal
            .predeal(Position::North, &[Card::new(Suit::Spades, Rank::Ace)])
            .unwrap();

        let config = FastParallelConfig { num_threads: 2 };
        let mut supervisor = FastSupervisor::with_predeal(42, predeal, config);

        let results = supervisor.process_batch(20, |_| true);

        // All deals should have AS in North
        for result in &results {
            assert!(result
                .deal
                .hand(Position::North)
                .cards()
                .contains(&Card::new(Suit::Spades, Rank::Ace)));
        }
    }
}
