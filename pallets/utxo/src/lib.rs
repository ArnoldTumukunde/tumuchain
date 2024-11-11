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

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub type Value = u128;

/// Maximum number of inputs or outputs in a transaction
pub const MAX_TRANSACTION_PARTS: u32 = 100;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {
	// Import various useful types required by all FRAME pallets.
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	// The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
	// (`Call`s) in this pallet.
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The pallet's configuration trait.
	///
	/// All our types and constants a pallet depends on must be declared here.
	/// These types are defined generically and made concrete when the pallet is declared in the
	/// `runtime/src/lib.rs` file of your chain.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A source to determine the block author
        type BlockAuthor: BlockAuthor;

        /// A source to determine the issuance portion of the block reward
        type Issuance: Issuance<<Self as frame_system::Config>::BlockNumber, Value>;

        #[pallet::constant]
        type MaxTransactionSize: Get<u32>;
	}

	/// Single transaction to be dispatched
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	#[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct Transaction {
		/// UTXOs to be used as inputs for current transaction
		pub inputs: BoundedVec<TransactionInput, ConstU32<MAX_TRANSACTION_PARTS>>,
		/// UTXOs to be created as a result of current transaction dispatch
		pub outputs: BoundedVec<TransactionOutput, ConstU32<MAX_TRANSACTION_PARTS>>,
	}

    /// Single transaction input that refers to one UTXO
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct TransactionInput {
        /// Reference to an UTXO to be spent
        pub outpoint: H256,
        /// Proof that transaction owner is authorized to spend referred UTXO &
        /// that the entire transaction is untampered
        pub sigscript: H512,
    }

    /// Single transaction output to create upon transaction dispatch
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct TransactionOutput {
        /// Value associated with this output
        pub value: Value,
        /// Public key associated with this output
        pub pubkey: H256,
    }

	/// storage items.
    #[pallet::storage]
    pub type UtxoStore<T: Config> = StorageMap<
        _,
        Identity,
        H256,
        TransactionOutput,
        OptionQuery
    >;

    #[pallet::storage]
    #[pallet::getter(fn reward_total)]
    pub type RewardTotal<T: Config> = StorageValue<_, Value, ValueQuery>;

	#[pallet::genesis_config]
    pub struct GenesisConfig {
        pub genesis_utxos: Vec<TransactionOutput>,
    }

    #[cfg(feature = "std")]
    impl Default for GenesisConfig {
        fn default() -> Self {
            Self {
                genesis_utxos: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {
            for utxo in &self.genesis_utxos {
                let hash = BlakeTwo256::hash_of(utxo);
                <UtxoStore<T>>::insert(hash, utxo);
            }
        }
    }

	/// Events that functions in this pallet can emit.
	///
	/// Events are a simple means of indicating to the outside world (such as dApps, chain explorers
	/// or other users) that some notable update in the runtime has occurred. In a FRAME pallet, the
	/// documentation for each event field and its parameters is added to a node's metadata so it
	/// can be used by external interfaces or tools.
	///
	///	The `generate_deposit` macro generates a function on `Pallet` called `deposit_event` which
	/// will convert the event type of your pallet into `RuntimeEvent` (declared in the pallet's
	/// [`Config`] trait) and deposit it using [`frame_system::Pallet::deposit_event`].
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        /// Transaction was executed successfully
        TransactionSuccess { transaction: Transaction },
        /// Rewards were issued
        RewardsIssued { amount: Value, utxo_hash: H256 },
        /// Rewards were wasted
        RewardsWasted,
	}

	/// Errors that can be returned by this pallet.
	///
	/// Errors tell users that something went wrong so it's important that their naming is
	/// informative. Similar to events, error documentation is added to a node's metadata so it's
	/// equally important that they have helpful documentation associated with them.
	///
	/// This type of runtime error can be up to 4 bytes in size should you want to return additional
	/// information.
	#[pallet::error]
	pub enum Error<T> {
        /// No inputs provided
        NoInputs,
        /// No outputs provided
        NoOutputs,
        /// Input used multiple times
        DuplicateInput,
        /// Output defined multiple times
        DuplicateOutput,
        /// Value overflow
        ValueOverflow,
        /// Missing input UTXO
        MissingInputUtxo,
        /// Invalid signature
        InvalidSignature,
        /// Zero value output
        ZeroValueOutput,
        /// Output already exists
        OutputAlreadyExists,
        /// Reward calculation error
        RewardError,
        /// Output total exceeds input total
        OutputExceedsInput,
        /// Output index overflow
        OutputIndexOverflow,
	}

	/// The pallet's dispatchable functions ([`Call`]s).
	///
	/// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	/// These functions materialize as "extrinsics", which are often compared to transactions.
	/// They must always return a `DispatchResult` and be annotated with a weight and call index.
	///
	/// The [`call_index`] macro is used to explicitly
	/// define an index for calls in the [`Call`] enum. This is useful for pallets that may
	/// introduce new dispatchables over time. If the order of a dispatchable changes, its index
	/// will also change which will break backwards compatibility.
	///
	/// The [`weight`] macro is used to assign a weight to each call.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a single u32 value as a parameter, writes the value
		/// to storage and emits an event.
		///
		/// It checks that the _origin_ for this call is _Signed_ and returns a dispatch
		/// error if it isn't. Learn more about origins here: <https://docs.substrate.io/build/origins/>
        #[pallet::call_index(0)]
        #[pallet::weight({
            let transaction_size = transaction.inputs.len().saturating_add(transaction.outputs.len());
            (10_000 as Weight)
                .saturating_mul(transaction_size as Weight)
                .saturating_add(10_000 as Weight)
        })]
        pub fn spend(
            origin: OriginFor<T>,
            transaction: Transaction,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let transaction_validity = Self::validate_transaction(&transaction)?;
            ensure!(
                transaction_validity.requires.is_empty(),
                Error::<T>::MissingInputUtxo
            );

            Self::update_storage(&transaction, transaction_validity.priority as Value)?;

            Self::deposit_event(Event::TransactionSuccess { transaction });
            Ok(())
        }
	}

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: BlockNumberFor<T>) {
            match T::BlockAuthor::block_author() {
                None => Self::deposit_event(Event::RewardsWasted),
                Some(author) => Self::disperse_reward(&author),
            }
        }
    }

	impl<T: Config> Pallet<T> {
		/// Validate transaction for validity, errors, & race conditions
		pub fn validate_transaction(transaction: &Transaction) -> Result<ValidTransaction, DispatchError> {
			// Check basic requirements
			ensure!(!transaction.inputs.is_empty(), Error::<T>::NoInputs);
			ensure!(!transaction.outputs.is_empty(), Error::<T>::NoOutputs);
	
			// Check for duplicate inputs
			let input_set: BTreeMap<_, ()> = transaction.inputs
				.iter()
				.map(|input| (input, ()))
				.collect();
			ensure!(
				input_set.len() == transaction.inputs.len(),
				Error::<T>::DuplicateInput
			);
	
			// Check for duplicate outputs
			let output_set: BTreeMap<_, ()> = transaction.outputs
				.iter()
				.map(|output| (output, ()))
				.collect();
			ensure!(
				output_set.len() == transaction.outputs.len(),
				Error::<T>::DuplicateOutput
			);
	
			let mut total_input: Value = 0;
			let mut total_output: Value = 0;
			let mut output_index: u64 = 0;
			let simple_transaction = Self::get_simple_transaction(transaction);
	
			// Variables for transaction pool
			let mut missing_utxos = Vec::new();
			let mut new_utxos = Vec::new();
			let mut reward = 0;
	
			// Validate inputs
			for input in transaction.inputs.iter() {
				if let Some(input_utxo) = <UtxoStore<T>>::get(&input.outpoint) {
					ensure!(
						sp_io::crypto::sr25519_verify(
							&Signature::from_raw(*input.sigscript.as_fixed_bytes()),
							&simple_transaction,
							&Public::from_h256(input_utxo.pubkey)
						),
						Error::<T>::InvalidSignature
					);
					total_input = total_input.checked_add(input_utxo.value)
						.ok_or(Error::<T>::ValueOverflow)?;
				} else {
					missing_utxos.push(input.outpoint.as_fixed_bytes().to_vec());
				}
			}
	
			// Validate outputs
			for output in transaction.outputs.iter() {
				ensure!(output.value > 0, Error::<T>::ZeroValueOutput);
				
				let hash = BlakeTwo256::hash_of(&(&transaction.encode(), output_index));
				output_index = output_index.checked_add(1)
					.ok_or(Error::<T>::OutputIndexOverflow)?;
				
				ensure!(
					!<UtxoStore<T>>::contains_key(hash),
					Error::<T>::OutputAlreadyExists
				);
				
				total_output = total_output.checked_add(output.value)
					.ok_or(Error::<T>::ValueOverflow)?;
				
				new_utxos.push(hash.as_fixed_bytes().to_vec());
			}
	
			// Verify input/output value relationship
			if missing_utxos.is_empty() {
				ensure!(
					total_input >= total_output,
					Error::<T>::OutputExceedsInput
				);
				reward = total_input.checked_sub(total_output)
					.ok_or(Error::<T>::RewardError)?;
			}
	
			Ok(ValidTransaction {
				requires: missing_utxos,
				provides: new_utxos,
				priority: reward as u64,
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		}
	
		/// Update storage to reflect changes made by transaction
		fn update_storage(transaction: &Transaction, reward: Value) -> DispatchResult {
			// Calculate new reward total
			let new_total = <RewardTotal<T>>::get()
				.checked_add(reward)
				.ok_or(Error::<T>::RewardError)?;
			<RewardTotal<T>>::put(new_total);
	
			// Remove spent UTXOs
			for input in transaction.inputs.iter() {
				<UtxoStore<T>>::remove(input.outpoint);
			}
	
			// Add new UTXOs
			let mut index: u64 = 0;
			for output in transaction.outputs.iter() {
				let hash = BlakeTwo256::hash_of(&(&transaction.encode(), index));
				index = index.checked_add(1)
					.ok_or(Error::<T>::OutputIndexOverflow)?;
				<UtxoStore<T>>::insert(hash, output);
			}
	
			Ok(())
		}
	
		/// Redistribute combined reward value to block author
		fn disperse_reward(author: &Public) {
			let reward = RewardTotal::<T>::take() + 
				T::Issuance::issuance(frame_system::Pallet::<T>::block_number());
	
			let utxo = TransactionOutput {
				value: reward,
				pubkey: H256::from_slice(author.as_slice()),
			};
	
			let hash = BlakeTwo256::hash_of(&(&utxo,
				<frame_system::Pallet<T>>::block_number().saturated_into::<u64>()));
	
			<UtxoStore<T>>::insert(hash, utxo);
			Self::deposit_event(Event::RewardsIssued { amount: reward, utxo_hash: hash });
		}
	
		/// Strips a transaction of its signature fields
		pub fn get_simple_transaction(transaction: &Transaction) -> Vec<u8> {
			let mut trx = transaction.clone();
			for input in trx.inputs.iter_mut() {
				input.sigscript = H512::zero();
			}
			trx.encode()
		}
	
		/// Helper for checking missing UTXOs
		pub fn get_missing_utxos(transaction: &Transaction) -> Vec<&H256> {
			let mut missing_utxos = Vec::new();
			for input in transaction.inputs.iter() {
				if <UtxoStore<T>>::get(&input.outpoint).is_none() {
					missing_utxos.push(&input.outpoint);
				}
			}
			missing_utxos
		}
	}
}
