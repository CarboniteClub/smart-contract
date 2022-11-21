use crate::*;

/// TaskId = company_name.task_name      company account_id = company_name.carbonite.near
pub type TaskId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskType {
    InviteOnly {
        invited_accounts: HashSet<AccountId>, // should be ideally be 3
        valid_till: Timestamp,                // unix epoch in ms
    }, // keeps track of invited accounts if an invite only project and validity date till if which if no-one accepts then company can claim refund
    ForEveryone, // this task can be taken up by anyone the company has the choice to select the winner
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Copy, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskState {
    /// open is for invite only task that haven't been accepted
    Open,
    /// pending is for invite only task that haven't been completed yet but accepted or bounty tasks that haven't been completed
    Pending,
    /// bounty tasks that have been completed (atleast one submission)
    Completed,
    /// invite that didn't get accepted untils its validity
    Expired,
    /// bounty / invite only tasks that have not been completed but it's past deadline
    Overdue,
    /// when the payment is done, sometimes task might completed but not payed in bounty tasks because company has to verify and award the best one
    Payed,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct TaskDetails {
    pub title: String,
    pub description: String,     // short description about the task
    pub required_skills: Skills, // required skills for the task in a comma seperated format
    pub task_type: TaskType,
    pub reference: String, // URL to an off-chain JSON file with more info, preferably a decentralised storage in encrypted format
    pub reference_hash: Base64VecU8, // Base64-encoded sha256 hash of Jencrypted JSON file itself from reference field
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Submission {
    pub submission_reference: String, // link to the decentralised submitted documents in encrypted format (preferrably)
    pub submission_reference_hash: Base64VecU8,
}

#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonSubmission {
    pub task_id: TaskId,
    pub account_id: AccountId,
    pub submission: Submission,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Task {
    pub task_details: TaskDetails,
    pub company_id: AccountId, // account ID of the company giving this task
    pub deadline: Timestamp, // if task is not completed till this (unix epoch in ms) then company can claim refund
    pub person_assigned: Option<AccountId>, // person assigned or person that accepted the invite for task in an invite only task
    pub task_state: TaskState,
    pub ft_contract_id: AccountId, // contract ID of approved token used to pay
    pub reward: Balance, // reward amount in smallest unit of tokens, Eg: for near it will be yoctoNEAR}
}

#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonTask {
    pub task_id: TaskId,
    pub task: Task,
}

impl TaskDetails {
    /// assert that task_details are valid else panic
    pub fn assert_valid_ref_hash(&self) {
        require!(
            self.reference_hash.0.len() == 32,
            "hash should be 32 bytes long"
        )

        // todo!(); make it generic
    }
}

impl Submission {
    /// assert that submission details are valid else panic
    pub fn assert_valid_ref_hash(&self) {
        require!(
            self.submission_reference_hash.0.len() == 32,
            "hash should be 32 bytes long"
        )

        // todo!(); make it generic
    }
}

impl Task {
    /// creates task struct out of details given and also validates if task details are valid
    pub fn new(
        task_details: TaskDetails,
        company_id: AccountId,
        deadline: Timestamp,
        ft_contract_id: AccountId,
        reward: u128,
    ) -> Self {
        task_details.assert_valid_ref_hash();

        // asserting deadline is after current time is not necessary as even if it's wrong it won't casue any harm

        let task_state = match task_details.task_type {
            TaskType::ForEveryone => TaskState::Pending,
            TaskType::InviteOnly { .. } => TaskState::Open,
        };

        Self {
            task_details,
            company_id,
            deadline,
            person_assigned: None,
            task_state,
            ft_contract_id,
            reward,
        }
    }

    /// return if invite only or not
    pub fn is_invite_only(&self) -> bool {
        if let TaskType::InviteOnly { .. } = self.task_details.task_type {
            true
        } else {
            false
        }
    }

    /// returns if validity time is completed or not for invite only tasks, false for everyone type
    pub fn is_past_validity(&self) -> bool {
        match self.task_details.task_type {
            TaskType::InviteOnly { valid_till, .. } => env::block_timestamp_ms() >= valid_till,
            TaskType::ForEveryone => false,
        }
    }

    /// retrns if it is past task deadline or not
    pub fn is_past_deadline(&self) -> bool {
        env::block_timestamp_ms() >= self.deadline
    }
}

#[near_bindgen]
impl Contract {
    /// add tasks in near token, (Here company pays reward + storage_cost + storage_cost for person assigned for invite only task)
    #[payable]
    pub fn add_task_in_near_token(
        &mut self,
        task_id: TaskId,
        task_details: TaskDetails,
        deadline: Timestamp,
        reward: Balance,
    ) {
        let initial_storage = env::storage_usage();

        let company_id = env::predecessor_account_id();

        self.assert_whitelisted_company(&company_id);

        let near_contract_id = AccountId::try_from("near".to_string()).unwrap();

        let task = Task::new(
            task_details,
            company_id.clone(),
            deadline,
            near_contract_id,
            reward,
        );
        self.internal_add_tasks_to_company(&company_id, &task_id);

        self.task_metadata_by_id.insert(&task_id, &task);

        let mut storage_used = env::storage_usage() - initial_storage;

        // pay for person assigned if invite only task
        if task.is_invite_only() {
            self.internal_add_task_invitations_per_user(&task_id, &task);
            storage_used = env::storage_usage() - initial_storage;
            storage_used = storage_used + STORAGE_USED_PER_ACCOUNT;
        }

        let storage_cost = storage_used as u128 * env::storage_byte_cost();

        let refund_amount = env::attached_deposit() - (storage_cost + reward);

        if refund_amount > 0 {
            Promise::new(company_id).transfer(refund_amount);
        } else {
            env::panic_str("attached deposit was not enough");
        }
    }

    pub fn extend_deadline(&mut self, task_id: TaskId, new_deadline: Timestamp) -> bool {
        self.ping_task(&task_id);

        let mut task = self.task_metadata_by_id.get(&task_id).unwrap();

        match task.task_state {
            TaskState::Open | TaskState::Pending => {
                if task.deadline < new_deadline {
                    task.deadline = new_deadline;
                }
            }
            _ => return false,
        }

        self.task_metadata_by_id.insert(&task_id, &task);
        return true;
    }

    /// submits the task if the user is eligible to submit for the task
    #[payable]
    pub fn submit_task(&mut self, task_id: TaskId, submission: Submission) {
        let initial_storage = env::storage_usage();

        let user_id = env::predecessor_account_id();

        self.ping_task(&task_id);

        let mut task = self.task_metadata_by_id.get(&task_id).unwrap();

        submission.assert_valid_ref_hash();

        match task.task_state {
            TaskState::Pending => {
                if let Some(person_assigned) = task.person_assigned.clone() {
                    require!(
                        user_id == person_assigned,
                        "only person assigned can submit the task"
                    );
                }

                self.internal_add_submission_to_task(&task_id, &user_id, &submission);

                task.task_state = TaskState::Completed;

                if task.person_assigned.is_some() {
                    self.transfer_reward_to(&task_id, &user_id);

                    self.update_user_carbonite_metadata_for_task(&task_id, &user_id);
                    task.task_state = TaskState::Payed;

                    // make gas check for promise to go through
                    // todo!();
                }
            }
            TaskState::Completed => {
                self.internal_add_submission_to_task(&task_id, &user_id, &submission);
            }
            _ => {
                env::panic_str("can't submit the task now");
            }
        }

        self.task_metadata_by_id.insert(&task_id, &task);

        let storage_used = env::storage_usage() - initial_storage;
        refund_excess_deposit(storage_used);
    }
}
