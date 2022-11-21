use crate::*;

#[near_bindgen]
impl Contract {
    /// reciever_id is expected to be anonymous_name.carbonite_contract_id
    /// will mint an nft after creating the sub a/c
    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
        title: String,
        description: Option<String>,
        public_key: PublicKey,
    ) -> PromiseOrValue<()> {
        let initial_storage = env::storage_usage();

        assert_valid_carbonite_user_account_pattern(&receiver_id);

        let token_metadata = TokenMetadata::new_default(title, description);

        self.internal_add_token_to_owner(&receiver_id, &token_metadata);

        create_sub_account(receiver_id.clone(), public_key);

        NftMintLog::emit(vec![NftMintLog {
            owner_id: receiver_id.to_string(),
            token_ids: vec![receiver_id.to_string()],
            memo: Some(format!(
                "{} has successfully joined Carbonite.Club",
                receiver_id
            )),
        }]);

        let storage_used = env::storage_usage() - initial_storage;

        refund_excess_deposit(storage_used)

        // while onboarding users, for a fixed size of title and description appropriate amount of allowance will be given to their funciton access key
        // and appropriate amount of near to cover storage costs
        // for standarisation purpose later a mint_event will be emitted
        // Add a gas check to ensure sub account creation and the full execution if account creation does not revert on panic
        // todo!();
    }

    /// accept invite from a company for a particular task, panic if can't accept
    pub fn accept_invite(&mut self, task_id: TaskId) {
        self.ping_task(&task_id);

        let user_id = env::predecessor_account_id();

        let mut task = self.task_metadata_by_id.get(&task_id).unwrap();

        match task.task_state {
            TaskState::Open => {
                if let TaskType::InviteOnly {
                    invited_accounts, ..
                } = &task.task_details.task_type
                {
                    require!(
                        invited_accounts.contains(&user_id),
                        "you are not invited for this task"
                    );

                    task.person_assigned = Some(user_id);

                    task.task_state = TaskState::Pending;
                }
            }
            _ => env::panic_str("Task State is not Open to Accept Invitations"),
        }

        self.task_metadata_by_id.insert(&task_id, &task);
    }
}

/// asserts that passed account ID is exactly of form valid_username.carbonite.near
pub(crate) fn assert_valid_carbonite_user_account_pattern(account_id: &AccountId) {
    let account_id = account_id.as_str();

    let (username, carbonite_contract_id) = account_id
        .split_once(".")
        .unwrap_or_else(|| env::panic_str("Invalid account ID passed"));

    require!(
        username
            .bytes()
            .into_iter()
            .all(|c| matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')),
        "Invalid username passed"
    );

    require!(
        carbonite_contract_id == env::current_account_id().as_str(),
        "Invalid account ID passed"
    );
}
