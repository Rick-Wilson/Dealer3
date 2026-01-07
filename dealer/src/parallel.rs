//! Parallel deal generation with deterministic output ordering.
//!
//! This module implements a supervisor/worker architecture for parallel deal generation
//! that maintains deterministic output regardless of thread count.
//!
//! # Architecture
//!
//! - **Supervisor**: Owns the RNG, dispatches work in batches, collects and orders results
//! - **Workers**: Receive work state + shared config, produce one deal, evaluate filter, return result
//!
//! # Determinism
//!
//! Results are always output in serial number order, ensuring that the same seed
//! produces identical output regardless of how many threads are used.
//!
//! # Efficiency
//!
//! The 65KB predeal config (zero52 table) is shared via Arc, so each work unit only
//! carries the minimal per-deal state (~300 bytes: RNG state + curdeal array).
//!
//! # Note
//!
//! This module is kept for reference and testing. The main `--legacy` mode uses
//! single-threaded generation for dealer.exe compatibility. See `fast_parallel.rs`
//! for the production parallel implementation.

#![allow(dead_code)]

use dealer_core::{Deal, DealGenerator, DealGeneratorConfig, DealWorkState};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// A unit of work for a parallel worker.
/// Contains only the minimal per-deal state (RNG + curdeal = ~300 bytes).
#[derive(Clone, Copy)]
pub struct WorkUnit {
    /// Serial number for ordering results
    pub serial_number: u64,
    /// Per-deal work state (RNG state + curdeal)
    pub work_state: DealWorkState,
}

/// Result of evaluating a single deal.
pub enum WorkResult {
    /// Filter passed, includes the matching deal
    Pass(Deal),
    /// Filter failed
    Fail,
}

/// Completed work from a worker.
pub struct CompletedWork {
    /// Serial number for ordering
    pub serial_number: u64,
    /// The generated deal (always present, even if filter failed)
    pub deal: Deal,
    /// Whether the filter passed
    pub passed: bool,
}

/// Configuration for parallel execution.
#[derive(Clone, Default)]
pub struct ParallelConfig {
    /// Number of worker threads (0 = auto-detect)
    pub num_threads: usize,
    /// Work units per batch (0 = auto, typically 100 × num_threads)
    pub batch_size: usize,
}

impl ParallelConfig {
    /// Get the actual number of threads to use.
    pub fn actual_threads(&self) -> usize {
        if self.num_threads == 0 {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        } else {
            self.num_threads
        }
    }

    /// Get the actual batch size to use.
    pub fn actual_batch_size(&self) -> usize {
        if self.batch_size == 0 {
            100 * self.actual_threads()
        } else {
            self.batch_size
        }
    }
}

/// Supervisor for parallel deal generation.
///
/// The supervisor owns the main RNG and coordinates workers to generate
/// and evaluate deals in parallel while maintaining deterministic output.
pub struct Supervisor {
    /// The main deal generator (owns the RNG)
    generator: DealGenerator,
    /// Shared predeal configuration (65KB, shared via Arc)
    config_arc: Arc<DealGeneratorConfig>,
    /// Next serial number to assign
    next_serial: u64,
    /// Parallel execution configuration
    parallel_config: ParallelConfig,
}

impl Supervisor {
    /// Create a new supervisor with the given generator and configuration.
    pub fn new(generator: DealGenerator, parallel_config: ParallelConfig) -> Self {
        // Configure rayon thread pool if custom thread count specified
        if parallel_config.num_threads > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(parallel_config.num_threads)
                .build_global()
                .ok(); // Ignore error if pool already initialized
        }

        // Capture the shared config once (65KB)
        let config_arc = Arc::new(generator.capture_config());

        Self {
            generator,
            config_arc,
            next_serial: 0,
            parallel_config,
        }
    }

    /// Generate a batch of work units.
    ///
    /// Captures only the per-deal work state (RNG + curdeal) for each deal.
    fn generate_batch(&mut self, count: usize) -> Vec<WorkUnit> {
        let mut units = Vec::with_capacity(count);

        for _ in 0..count {
            let work_state = self.generator.capture_work_state();
            units.push(WorkUnit {
                serial_number: self.next_serial,
                work_state,
            });
            self.next_serial += 1;
            // Advance the generator RNG state (skips deal distribution and sorting)
            self.generator.advance_one_deal();
        }

        units
    }

    /// Process a batch of work units in parallel.
    ///
    /// Each worker generates a deal from its work state + shared config and evaluates the filter.
    /// Results are returned sorted by serial number.
    pub fn process_batch<F>(&mut self, count: usize, filter: F) -> Vec<CompletedWork>
    where
        F: Fn(&Deal) -> bool + Sync,
    {
        let units = self.generate_batch(count);
        let config = Arc::clone(&self.config_arc);

        // Process in parallel
        let mut results: Vec<CompletedWork> = units
            .into_par_iter()
            .map(|unit| {
                // Generate deal from work state using shared config
                let deal = DealGenerator::generate_from_work_state(&config, unit.work_state);
                let passed = filter(&deal);

                CompletedWork {
                    serial_number: unit.serial_number,
                    deal,
                    passed,
                }
            })
            .collect();

        // Sort by serial number to maintain deterministic order
        results.sort_by_key(|w| w.serial_number);

        results
    }

    /// Get the number of deals generated so far.
    pub fn generated_count(&self) -> u64 {
        self.next_serial
    }

    /// Get the current batch size.
    pub fn batch_size(&self) -> usize {
        self.parallel_config.actual_batch_size()
    }
}

/// Result collector that buffers out-of-order results and yields them in order.
///
/// Used for streaming output where we want to output results as soon as possible
/// while maintaining serial order.
pub struct OrderedResultCollector {
    /// Next serial number expected for output
    next_output_serial: u64,
    /// Buffered results waiting for earlier serials
    buffer: HashMap<u64, CompletedWork>,
}

impl OrderedResultCollector {
    /// Create a new collector.
    pub fn new() -> Self {
        Self {
            next_output_serial: 0,
            buffer: HashMap::new(),
        }
    }

    /// Add a completed work item.
    ///
    /// Returns an iterator of results that can now be output in order.
    pub fn add(&mut self, work: CompletedWork) -> impl Iterator<Item = CompletedWork> + '_ {
        self.buffer.insert(work.serial_number, work);

        // Yield all consecutive results starting from next_output_serial
        std::iter::from_fn(move || {
            if let Some(result) = self.buffer.remove(&self.next_output_serial) {
                self.next_output_serial += 1;
                Some(result)
            } else {
                None
            }
        })
    }

    /// Check if there are buffered results waiting.
    pub fn has_buffered(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Get the next expected serial number.
    pub fn next_expected(&self) -> u64 {
        self.next_output_serial
    }
}

impl Default for OrderedResultCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config_defaults() {
        let config = ParallelConfig::default();
        assert!(config.actual_threads() >= 1);
        assert!(config.actual_batch_size() >= 100);
    }

    #[test]
    fn test_parallel_config_explicit() {
        let config = ParallelConfig {
            num_threads: 4,
            batch_size: 200,
        };
        assert_eq!(config.actual_threads(), 4);
        assert_eq!(config.actual_batch_size(), 200);
    }

    #[test]
    fn test_supervisor_batch_generation() {
        let gen = DealGenerator::new(42);
        let config = ParallelConfig {
            num_threads: 1,
            batch_size: 10,
        };
        let mut supervisor = Supervisor::new(gen, config);

        // Generate a batch
        let results = supervisor.process_batch(10, |_| true);

        assert_eq!(results.len(), 10);

        // Check serial numbers are in order
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.serial_number, i as u64);
            assert!(result.passed);
        }
    }

    #[test]
    fn test_supervisor_deterministic_output() {
        // Two supervisors with same seed should produce same deals
        let gen1 = DealGenerator::new(123);
        let gen2 = DealGenerator::new(123);

        let config = ParallelConfig {
            num_threads: 4, // Use multiple threads
            batch_size: 50,
        };

        let mut supervisor1 = Supervisor::new(gen1, config.clone());
        let mut supervisor2 = Supervisor::new(gen2, config);

        let results1 = supervisor1.process_batch(50, |_| true);
        let results2 = supervisor2.process_batch(50, |_| true);

        // Results should be identical
        assert_eq!(results1.len(), results2.len());
        for (r1, r2) in results1.iter().zip(results2.iter()) {
            assert_eq!(r1.serial_number, r2.serial_number);
            assert_eq!(r1.deal, r2.deal);
        }
    }

    #[test]
    fn test_supervisor_filter() {
        let gen = DealGenerator::new(42);
        let config = ParallelConfig {
            num_threads: 2,
            batch_size: 100,
        };
        let mut supervisor = Supervisor::new(gen, config);

        // Filter: North has >= 15 HCP
        let results = supervisor.process_batch(100, |deal| deal.north.hcp() >= 15);

        // Some should pass, some should fail
        let passed_count = results.iter().filter(|r| r.passed).count();
        let failed_count = results.iter().filter(|r| !r.passed).count();

        assert!(passed_count > 0, "Expected some deals to pass");
        assert!(failed_count > 0, "Expected some deals to fail");
        assert_eq!(passed_count + failed_count, 100);
    }

    #[test]
    fn test_ordered_result_collector() {
        let mut collector = OrderedResultCollector::new();

        // Add results out of order
        let work2 = CompletedWork {
            serial_number: 2,
            deal: Deal::new(),
            passed: true,
        };
        let work0 = CompletedWork {
            serial_number: 0,
            deal: Deal::new(),
            passed: true,
        };
        let work1 = CompletedWork {
            serial_number: 1,
            deal: Deal::new(),
            passed: true,
        };

        // Add work2 first - nothing should be yielded
        let yielded: Vec<_> = collector.add(work2).collect();
        assert!(yielded.is_empty());
        assert!(collector.has_buffered());

        // Add work0 - should yield work0
        let yielded: Vec<_> = collector.add(work0).collect();
        assert_eq!(yielded.len(), 1);
        assert_eq!(yielded[0].serial_number, 0);

        // Add work1 - should yield work1 and work2
        let yielded: Vec<_> = collector.add(work1).collect();
        assert_eq!(yielded.len(), 2);
        assert_eq!(yielded[0].serial_number, 1);
        assert_eq!(yielded[1].serial_number, 2);

        assert!(!collector.has_buffered());
    }

    #[test]
    fn test_parallel_matches_sequential() {
        // Verify that parallel execution produces same deals as sequential
        let seed = 999;

        // Sequential generation
        let mut seq_gen = DealGenerator::new(seed);
        let sequential_deals: Vec<Deal> = (0..20).map(|_| seq_gen.generate()).collect();

        // Parallel generation
        let par_gen = DealGenerator::new(seed);
        let config = ParallelConfig {
            num_threads: 4,
            batch_size: 20,
        };
        let mut supervisor = Supervisor::new(par_gen, config);
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
    fn test_work_unit_size() {
        // Verify that work units are small (should be ~300 bytes, not 65KB+)
        let work_state_size = std::mem::size_of::<DealWorkState>();
        let work_unit_size = std::mem::size_of::<WorkUnit>();

        // DealWorkState should be around 300 bytes (248 bytes RNG + 52 bytes curdeal)
        assert!(
            work_state_size < 400,
            "DealWorkState is too large: {} bytes",
            work_state_size
        );

        // WorkUnit adds serial number (8 bytes)
        assert!(
            work_unit_size < 410,
            "WorkUnit is too large: {} bytes",
            work_unit_size
        );
    }
}
