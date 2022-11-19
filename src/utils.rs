use crate::*;

/// returns SHA-256 hash of the passed account ID
pub(crate) fn hash_id(id: &str) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(id.as_bytes()));
    hash
}

/// creates sub account provide with BASE_STORAGE_COST for a given account ID and public key
pub(crate) fn create_sub_account(account_id: AccountId, public_key: PublicKey) {
    Promise::new(account_id.clone())
        .create_account()
        .transfer(BASE_STORAGE_COST)
        .add_full_access_key(public_key);
}

/// refunds excess deposit attached to predecessor_account_id and panic if attached deposit is not enough to cover given storage_used in bytes
pub(crate) fn refund_excess_deposit(storage_used: u64) -> PromiseOrValue<()> {
    let storage_cost = env::storage_byte_cost() * (storage_used as u128);

    let refund_amount = env::attached_deposit()
        .checked_sub(storage_cost)
        .unwrap_or_else(|| env::panic_str("attached deposit was not enough"));

    if refund_amount > 0 {
        PromiseOrValue::Promise(Promise::new(env::predecessor_account_id()).transfer(refund_amount))
    } else {
        PromiseOrValue::Value(())
    }
}

impl Contract {
    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "Only Contract Owner can call this method"
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
pub mod test_utils {
    use crate::*;
    use near_sdk::Balance;
    use near_sdk::VMContext;

    // Helper functions

    pub fn carol() -> AccountId {
        "carol.near".parse().unwrap()
    }

    pub fn get_context(predecessor_account_id: AccountId, attached_deposit: Balance) -> VMContext {
        VMContext {
            current_account_id: "carbonite.near".parse().unwrap(),
            signer_account_id: predecessor_account_id.clone(),
            signer_account_pk: "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"
                .parse()
                .unwrap(),
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 1000 * 10u128.pow(24),
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit,
            prepaid_gas: near_sdk::Gas(10u64.pow(18)),
            random_seed: [8; 32],
            output_data_receivers: vec![],
            epoch_height: 0,
            view_config: None,
        }
    }
}
