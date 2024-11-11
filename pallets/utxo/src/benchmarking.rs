#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_core::{sr25519::Public, H256};
use sp_runtime::traits::BlakeTwo256;


const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: Event<T>) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn create_funded_utxo<T: Config>(value: Value, pubkey: H256) -> H256 {
    let utxo = TransactionOutput { value, pubkey };
    let hash = BlakeTwo256::hash_of(&utxo);
    UtxoStore::<T>::insert(hash, utxo);
    hash
}

benchmarks! {
    spend {
        let i in 1 .. MAX_TRANSACTION_PARTS as u32;
        let o in 1 .. MAX_TRANSACTION_PARTS as u32;
        
        let caller: T::AccountId = whitelisted_caller();
        let pub_key = H256::random();
        
        // Create input UTXOs
        let mut inputs = Vec::new();
        let value_per_utxo = 100;
        for _ in 0..i {
            let hash = create_funded_utxo::<T>(value_per_utxo, pub_key);
            inputs.push((hash, H512::zero()));
        }
        
        // Create output definitions
        let mut outputs = Vec::new();
        let value_per_output = (i as u128 * value_per_utxo) / (o as u128);
        for _ in 0..o {
            outputs.push((value_per_output, H256::random()));
        }
        
        let transaction = create_test_transaction(inputs, outputs);

    }: _(RawOrigin::Signed(caller), transaction.clone())
    verify {
        assert_last_event::<T>(Event::TransactionSuccess { transaction }.into());
    }

    impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

fn create_test_transaction(inputs: Vec<(H256, H512)>, outputs: Vec<(Value, H256)>) -> Transaction {
    Transaction {
        inputs: BoundedVec::try_from(
            inputs
                .into_iter()
                .map(|(outpoint, sigscript)| TransactionInput {
                    outpoint,
                    sigscript,
                })
                .collect::<Vec<_>>(),
        )
        .unwrap(),
        outputs: BoundedVec::try_from(
            outputs
                .into_iter()
                .map(|(value, pubkey)| TransactionOutput { value, pubkey })
                .collect::<Vec<_>>(),
        )
        .unwrap(),
    }
}