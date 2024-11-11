//! # Template Pallet
//!
//! A pallet with minimal functionality to help developers understand the essential components of
//! writing a FRAME pallet. It is typically used in beginner tutorials or in Substrate template
//! nodes as a starting point for creating a new pallet and **not meant to be used in production**.
//!
//! ## Overview
//!
//! This template pallet contains basic examples of:
//! - declaring a storage item that stores a single `u32` value
//! - declaring and using events
//! - declaring and using errors
//! - a dispatchable function that allows a user to set a new value to storage and emits an event
//!   upon success
//! - another dispatchable function that causes a custom error to be thrown
//!
//! Each pallet section is annotated with an attribute using the `#[pallet::...]` procedural macro.
//! This macro generates the necessary code for a pallet to be aggregated into a FRAME runtime.
//!
//! Learn more about FRAME macros [here](https://docs.substrate.io/reference/frame-macros/).
//!
//! ### Pallet Sections
//!
//! The pallet sections in this template are:
//!
//! - A **configuration trait** that defines the types and parameters which the pallet depends on
//!   (denoted by the `#[pallet::config]` attribute). See: [`Config`].
//! - A **means to store pallet-specific data** (denoted by the `#[pallet::storage]` attribute).
//!   See: [`storage_types`].
//! - A **declaration of the events** this pallet emits (denoted by the `#[pallet::event]`
//!   attribute). See: [`Event`].
//! - A **declaration of the errors** that this pallet can throw (denoted by the `#[pallet::error]`
//!   attribute). See: [`Error`].
//! - A **set of dispatchable functions** that define the pallet's functionality (denoted by the
//!   `#[pallet::call]` attribute). See: [`dispatchables`].
//!
//! Run `cargo doc --package pallet-template --open` to view this pallet's documentation.

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use sp_core::U256;
use core::cmp::{min, max};
use sp_runtime::traits::{UniqueSaturatedInto, Time};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

const DIFFICULTY_ADJUST_WINDOW: u128 = 60;

fn damp(actual: u128, goal: u128, damp_factor: u128) -> u128 {
    (actual + (damp_factor - 1) * goal) / damp_factor
}

fn clamp(actual: u128, goal: u128, clamp_factor: u128) -> u128 {
    max(goal / clamp_factor, min(actual, goal * clamp_factor))
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
        type TimeProvider: Time;
        type TargetBlockTime: Get<u128>;
        type DampFactor: Get<u128>;
        type ClampFactor: Get<u128>;
        type MaxDifficulty: Get<u128>;
        type MinDifficulty: Get<u128>;
    }

    #[pallet::storage]
    #[pallet::getter(fn difficulty_and_timestamps)]
    pub type PastDifficultiesAndTimestamps<T: Config> = 
        StorageValue<_, BoundedVec<DifficultyAndTimestamp<T::Moment>, ConstU32<60>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn difficulty)]
    pub type CurrentDifficulty<T: Config> = StorageValue<_, Difficulty, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub initial_difficulty: Difficulty,
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <CurrentDifficulty<T>>::put(self.initial_difficulty);
        }
    }

    #[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Debug, Default)]
    pub struct DifficultyAndTimestamp<M> {
        pub difficulty: Difficulty,
        pub timestamp: M,
    }

    pub type Difficulty = U256;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        DifficultyUpdated {
            difficulty: Difficulty,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        FailedToUpdateDifficulty,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_finalize(_block_number: T::BlockNumber) {
            let mut data = Self::difficulty_and_timestamps();
            
            // If we haven't filled up the window yet, just add the new data point
            if data.len() < DIFFICULTY_ADJUST_WINDOW as usize {
                let _ = data.try_push(DifficultyAndTimestamp {
                    timestamp: T::TimeProvider::now(),
                    difficulty: Self::difficulty(),
                });
            } else {
                // Shift all elements left and add new data point at the end
                for i in 1..data.len() {
                    data[i - 1] = data[i];
                }
                data[data.len() - 1] = DifficultyAndTimestamp {
                    timestamp: T::TimeProvider::now(),
                    difficulty: Self::difficulty(),
                };
            }

            <PastDifficultiesAndTimestamps<T>>::put(data);
            Self::update_difficulty();
        }
    }

    impl<T: Config> Pallet<T> {
        fn update_difficulty() {
            let data = Self::difficulty_and_timestamps();
            
            // Calculate timestamp delta
            let mut ts_delta = 0;
            for i in 1..data.len() {
                let prev: u128 = data[i - 1].timestamp.unique_saturated_into();
                let cur: u128 = data[i].timestamp.unique_saturated_into();
                ts_delta += cur.saturating_sub(prev);
            }

            // Prevent division by zero
            if ts_delta == 0 {
                ts_delta = 1;
            }

            // Calculate difficulty sum
            let mut diff_sum = U256::zero();
            for item in data.iter() {
                diff_sum += item.difficulty;
            }

            // Enforce minimum difficulty
            if diff_sum < U256::from(T::MinDifficulty::get()) {
                diff_sum = U256::from(T::MinDifficulty::get());
            }

            // Calculate the average length of the adjustment window
            let adjustment_window = DIFFICULTY_ADJUST_WINDOW * T::TargetBlockTime::get();

            // Adjust time delta toward goal subject to dampening and clamping
            let adj_ts = clamp(
                damp(ts_delta, adjustment_window, T::DampFactor::get()),
                adjustment_window,
                T::ClampFactor::get(),
            );

            // Calculate new difficulty
            let difficulty = min(
                U256::from(T::MaxDifficulty::get()),
                max(
                    U256::from(T::MinDifficulty::get()),
                    diff_sum * U256::from(T::TargetBlockTime::get()) / U256::from(adj_ts)
                )
            );

            // Update storage and emit event
            <CurrentDifficulty<T>>::put(difficulty);
            Self::deposit_event(Event::DifficultyUpdated { difficulty });
        }
    }
}