use super::*;
use crate::mock::{new_test_ext, Test, Utxo};
use frame_support::{assert_noop, assert_ok};
use sp_core::{
    sr25519::{Public, Signature},
    testing::SR25519,
    H256, H512,
};
use sp_runtime::traits::BlakeTwo256;

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

#[test]
fn test_simple_transaction() {
    new_test_ext().execute_with(|| {
        // Create a genesis UTXO
        let genesis_utxo = TransactionOutput {
            value: 100,
            pubkey: H256::random(),
        };
        let genesis_hash = BlakeTwo256::hash_of(&genesis_utxo);
        UtxoStore::<Test>::insert(genesis_hash, genesis_utxo.clone());

        // Create a transaction spending the genesis UTXO
        let new_pubkey = H256::random();
        let transaction = create_test_transaction(
            vec![(genesis_hash, H512::zero())],
            vec![(50, new_pubkey.clone())],
        );

        // Validate transaction
        let result = Utxo::validate_transaction(&transaction);
        assert!(result.is_ok());

        // Check storage updates
        assert_ok!(Utxo::update_storage(&transaction, 50));
        assert!(UtxoStore::<Test>::get(genesis_hash).is_none());

        // Verify new UTXO exists
        let new_hash = BlakeTwo256::hash_of(&(&transaction.encode(), 0u64));
        let new_utxo = UtxoStore::<Test>::get(new_hash).unwrap();
        assert_eq!(new_utxo.value, 50);
        assert_eq!(new_utxo.pubkey, new_pubkey);
    });
}

#[test]
fn test_invalid_transaction() {
    new_test_ext().execute_with(|| {
        // Try to spend non-existent UTXO
        let transaction = create_test_transaction(
            vec![(H256::random(), H512::zero())],
            vec![(50, H256::random())],
        );

        assert_noop!(
            Utxo::validate_transaction(&transaction),
            Error::<Test>::MissingInputUtxo
        );
    });
}

#[test]
fn test_duplicate_input() {
    new_test_ext().execute_with(|| {
        let input_hash = H256::random();
        let transaction = create_test_transaction(
            vec![(input_hash.clone(), H512::zero()), (input_hash, H512::zero())],
            vec![(50, H256::random())],
        );

        assert_noop!(
            Utxo::validate_transaction(&transaction),
            Error::<Test>::DuplicateInput
        );
    });
}

#[test]
fn test_output_exceeds_input() {
    new_test_ext().execute_with(|| {
        let genesis_utxo = TransactionOutput {
            value: 100,
            pubkey: H256::random(),
        };
        let genesis_hash = BlakeTwo256::hash_of(&genesis_utxo);
        UtxoStore::<Test>::insert(genesis_hash, genesis_utxo.clone());

        let transaction = create_test_transaction(
            vec![(genesis_hash, H512::zero())],
            vec![(150, H256::random())],
        );

        assert_noop!(
            Utxo::validate_transaction(&transaction),
            Error::<Test>::OutputExceedsInput
        );
    });
}

#[test]
fn test_zero_value_output() {
    new_test_ext().execute_with(|| {
        let transaction = create_test_transaction(
            vec![(H256::random(), H512::zero())],
            vec![(0, H256::random())],
        );

        assert_noop!(
            Utxo::validate_transaction(&transaction),
            Error::<Test>::ZeroValueOutput
        );
    });
}

#[test]
fn test_reward_dispersion() {
    new_test_ext().execute_with(|| {
        // Set initial reward
        RewardTotal::<Test>::put(100);

        // Create mock author
        let author = Public::from_raw([0; 32]);
        
        // Disperse rewards
        Utxo::disperse_reward(&author);

        // Verify reward total is cleared
        assert_eq!(RewardTotal::<Test>::get(), 0);

        // Verify new UTXO is created for author
        let utxo_hash = BlakeTwo256::hash_of(&(&TransactionOutput {
            value: 200, // 100 from reward + 100 from issuance
            pubkey: H256::from_slice(author.as_slice()),
        }, 0u64));

        let author_utxo = UtxoStore::<Test>::get(utxo_hash).unwrap();
        assert_eq!(author_utxo.value, 200);
        assert_eq!(author_utxo.pubkey, H256::from_slice(author.as_slice()));
    });
}