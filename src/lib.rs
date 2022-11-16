use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};

use near_sdk::{
    env, near_bindgen, require, AccountId, Balance, BorshStorageKey, CryptoHash, NearSchema,
    PanicOnDefault, Promise, PublicKey, StorageUsage, Timestamp,
};

use std::collections::HashSet;

mod company;
mod enumeration;
mod index;
mod internal;
mod metadata;
mod payment;
mod task;
mod user;
mod utils;
mod view;
// mod types;

pub use crate::company::*;
pub use crate::index::*;
pub use crate::internal::*;
pub use crate::metadata::*;
pub use crate::payment::*;
pub use crate::task::*;
pub use crate::user::*;
pub use crate::utils::*;
pub use crate::view::*;

const DEFAULT_RECOGNISED_SKILLS_SET: [&str; 2] = ["UI Designing", "UX Designing"];

const DEFAULT_MEDIA_REFERENCE: &str = "ipfs://dummy_default_media_link";
const DEFAULT_NFT_REFERENCE: &str = "ipfs://dummy_default_nft_link";

const BASE_STORAGE_COST: Balance = 10_000_000_000_000_000_000_000; // this is equal to 0.01 NEAR

const STORAGE_USED_PER_ACCOUNT: StorageUsage = 64;

/// Helper structure to for keys of the persistent collections.
#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    TokensByAccountId,
    TasksCompletedPerAccountInner { account_id_hash: CryptoHash },
    TasksByCompany,
    TaskInvitationsPerUser,
    TaskInvitationsPerUserInner { account_id_hash: CryptoHash },
    TasksByCompanyInner { company_id_hash: CryptoHash },
    TaskMetadataById,
    SubmissionsPerTask,
    SubmissionsPerTaskInner { task_id_hash: CryptoHash },
    RecognisedSkills,
    PendingVerificationRequests,
    WhitelistedCompanies,
    ApprovedFTTokens,
    NFTContractMetadata,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// owner of the contract
    pub owner_id: AccountId,

    /// keeps track of metadata of carbonite NFT for a given account
    pub tokens_by_account_id: UnorderedMap<AccountId, TokenMetadata>,

    /// keeps tracks of submissons that are submitted for a particular task ID
    pub submissions_per_task: LookupMap<TaskId, UnorderedMap<AccountId, Submission>>, // keeps track of user_account and their submission

    /// keeps track of all tasks that are given for a given company
    pub tasks_by_company: LookupMap<AccountId, UnorderedSet<TaskId>>,

    /// keeps track of tasks that user was invited to
    pub task_invitations_per_user: LookupMap<AccountId, UnorderedSet<TaskId>>,

    /// keeps track of task metadata for a given task ID
    pub task_metadata_by_id: UnorderedMap<TaskId, Task>,

    /// keeps track of all the skills that are recognised by carbonite community
    pub recognised_skills: UnorderedSet<Skills>,

    /// keeps track of pending verification requests
    pub pending_verification_requests: UnorderedMap<AccountId, CompanyRegDetails>,

    /// keeps track of whitelisted companies that are verified to be genuine
    pub whitelisted_companies: UnorderedMap<AccountId, Company>,

    /// keeps track of approved tokens that the companies can use for paying
    pub approved_ft_tokens: UnorderedSet<AccountId>,

    /// keeps track of metadata of the contract
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
            submissions_per_task: LookupMap::new(StorageKey::SubmissionsPerTask),
            tasks_by_company: LookupMap::new(StorageKey::TasksByCompany),
            task_invitations_per_user: LookupMap::new(StorageKey::TaskInvitationsPerUser),
            task_metadata_by_id: UnorderedMap::new(StorageKey::TaskMetadataById),
            recognised_skills: UnorderedSet::new(StorageKey::RecognisedSkills),
            pending_verification_requests: UnorderedMap::new(
                StorageKey::PendingVerificationRequests,
            ),
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

        let near_contract_id = AccountId::try_from("near".to_string()).unwrap();

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

        refund_excess_deposit(storage_used);
    }

    /// request for verification of company
    #[payable]
    pub fn request_verification(&mut self, company_reg_details: CompanyRegDetails) {
        let initial_storage = env::storage_usage();

        let account_id = company_reg_details.account_id.clone();

        require!(
            self.pending_verification_requests
                .insert(&account_id, &company_reg_details)
                .is_none(),
            "verification has already been requested"
        );

        require!(
            self.whitelisted_companies.get(&account_id).is_none(),
            "Company has already been registered"
        );

        let storage_used = env::storage_usage() - initial_storage;
        refund_excess_deposit(storage_used);
    }

    /// owner only method to add new multiple whitelisted companies
    #[payable]
    pub fn whitelist_companies(&mut self, companies: Vec<AccountId>) {
        self.assert_owner();

        let initial_storage = env::storage_usage();

        require!(
            companies.len() <= 7,
            "can't whitelist more than 7 companies in a single call (hard gas limit)"
        );

        for company_id in companies {
            assert_valid_carbonite_company_account_pattern(company_id.as_str());

            if let Some(company_reg_details) =
                self.pending_verification_requests.remove(&company_id)
            {
                create_sub_account(company_id.clone(), company_reg_details.public_key);
                self.internal_add_company_to_whitelisted_companies(
                    &company_id,
                    &company_reg_details.company,
                );
            } else {
                env::panic_str(
                    format!("{company_id} is not in pending verification list").as_str(),
                );
            }
        }

        let storage_used = env::storage_usage() - initial_storage;
        refund_excess_deposit(storage_used);

        // Add a gas check to ensure sub account creation and the full execution if account creation does not revert on panic
        // Test and modify what should be max companies that can be whitelisted in a single call (restricted due to hard limit on gas on a function call)
        // todo!();
    }

    /// make appropriate changes to task_state of a given task
    pub fn ping_task(&mut self, task_id: &TaskId) {
        let mut task = self
            .task_metadata_by_id
            .get(task_id)
            .unwrap_or_else(|| env::panic_str("invalid task_id"));

        match task.task_state {
            TaskState::Open => {
                if task.is_past_validity() {
                    task.task_state = TaskState::Expired;
                }
            }
            TaskState::Pending => {
                if task.is_past_deadline() {
                    task.task_state = TaskState::Overdue;
                }
            }
            TaskState::Payed | TaskState::Completed | TaskState::Expired | TaskState::Overdue => {}
        };

        self.task_metadata_by_id.insert(&task_id, &task);
    }
}
