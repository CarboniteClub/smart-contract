use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::Base64VecU8;

use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, BorshStorageKey, CryptoHash, PanicOnDefault,
    Promise, PublicKey, Timestamp,
};

use std::collections::{HashMap, HashSet};

mod company;
mod internal;
mod metadata;
mod task;
mod user;
mod utils;

pub use crate::company::*;
pub use crate::internal::*;
pub use crate::metadata::*;
pub use crate::task::*;
pub use crate::user::*;
pub use crate::utils::*;

const DEFAULT_RECOGNISED_SKILLS_SET: [&str; 2] = ["UI Designing", "UX Designing"];

const DEFAULT_MEDIA_REFERENCE: &str = "ipfs://dummy_default_media_link";
const DEFAULT_NFT_REFERENCE: &str = "ipfs://dummy_default_nft_link";

const BASE_STORAGE_COST: Balance = 10_000_000_000_000_000_000_000; // this is equal to 0.01 NEAR

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    TokensByAccountId,
    TasksCompletedPerAccount,
    TasksCompletedPerAccountInner { account_id_hash: CryptoHash },
    TasksByCompany,
    TasksByCompanyInner { task_id_hash: CryptoHash },
    TaskMetadataById,
    RecognisedSkills,
    WhitelistedCompanies,
    ApprovedFTTokens,
    NFTContractMetadata,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // owner of the contract
    pub owner_id: AccountId,

    // keeps track of metadata of carbonite NFT for a given account
    pub tokens_by_account_id: UnorderedMap<AccountId, TokenMetadata>,

    // keeps track of tasks that are completed and verified for a given account
    pub tasks_completed_per_account: LookupMap<AccountId, UnorderedSet<TaskId>>,

    // keeps track of all tasks that are given for a given company
    pub tasks_by_company: LookupMap<AccountId, UnorderedSet<TaskId>>,

    // keeps track of task metadata for a given task ID
    pub task_metadata_by_id: UnorderedMap<TaskId, Task>,

    // keeps track of all the skills that are recognised by carbonite community
    pub recognised_skills: UnorderedSet<Skills>,

    // keeps track of whitelisted companies that are verified to be genuine
    pub whitelisted_companies: UnorderedMap<AccountId, Company>,

    // keeps track of approved tokens that the companies can use for paying
    pub approved_ft_tokens: UnorderedSet<AccountId>,

    // keeps track of metadata of the contract
    pub metadata: LazyOption<NFTContractMetadata>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        let carbonite_account_id = env::current_account_id();

        let mut this = Self {
            owner_id,
            tokens_by_account_id: UnorderedMap::new(StorageKey::TokensByAccountId),
            tasks_completed_per_account: LookupMap::new(StorageKey::TasksCompletedPerAccount),
            tasks_by_company: LookupMap::new(StorageKey::TasksByCompany),
            task_metadata_by_id: UnorderedMap::new(StorageKey::TaskMetadataById),
            recognised_skills: UnorderedSet::new(StorageKey::RecognisedSkills),
            whitelisted_companies: UnorderedMap::new(StorageKey::WhitelistedCompanies),
            approved_ft_tokens: UnorderedSet::new(StorageKey::ApprovedFTTokens),
            metadata: LazyOption::new(StorageKey::NFTContractMetadata, Some(&metadata)),
        };

        for skill in DEFAULT_RECOGNISED_SKILLS_SET.into_iter() {
            this.recognised_skills.insert(&skill.into());
        }

        let carbonite = Company {
            name: "Carbonite".to_string(),
            icon: "CARBONITE".to_string(),
            industries: "Blockchain".to_string(),
            description: "Build your Proof of Skills".to_string(),
            location: None,
            reference: "ipfs://dummylink".to_string(),
        };

        this.whitelisted_companies
            .insert(&carbonite_account_id, &carbonite);

        let near_contract_id = AccountId::new_unchecked("near".to_string());

        this.approved_ft_tokens.insert(&near_contract_id);

        this
    }

    /// owner only method to approve multiple ft_token support
    #[payable]
    pub fn approve_ft_tokens(&mut self, ft_tokens_contract_ids: Vec<AccountId>) {
        self.assert_owner();

        let initial_storage = env::storage_usage();

        for ft_contract_id in ft_tokens_contract_ids {
            self.approved_ft_tokens.insert(&ft_contract_id);
        }

        let storage_used = env::storage_usage() - initial_storage;

        refund_deposit(storage_used);
    }
}
