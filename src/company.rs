use crate::*;

// company account ID will be suffixed with -co whereas users can't have _ in their name
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct Company {
    pub name: String,
    pub icon: String,             // Data URL of company logo
    pub industries: String, // various industries (comma seperated) in which company is working
    pub description: String, // short description about the company
    pub location: Option<String>, // None if company is remote or else represents headquarter location
    pub reference: String,        // website url of the company
}
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct CompanyRegDetails {
    pub account_id: AccountId,
    pub company: Company,
    pub public_key: PublicKey,
}

#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonCompany {
    pub account_id: AccountId,
    pub details: Company,
}

#[near_bindgen]
impl Contract {
    /// company only method to edit company details
    #[payable]
    pub fn edit_company_details(&mut self, new_company_details: Company) {
        let initial_storage = env::storage_usage();

        let company_id = env::predecessor_account_id();

        self.assert_whitelisted_company(&company_id);

        self.whitelisted_companies
            .insert(&company_id, &new_company_details);

        let final_storage = env::storage_usage();

        let storage_used = final_storage.abs_diff(initial_storage);

        if final_storage > initial_storage {
            refund_excess_deposit(storage_used);
        } else if final_storage < initial_storage {
            let refund_amount = storage_used as u128 * env::storage_byte_cost();
            Promise::new(company_id).transfer(refund_amount);
        }

        // add a gas check for the promise to go through and place constraints on editing
        // todo!()
    }

    /// select a submission for a bounty task that is to be awarded
    #[payable]
    pub fn select_task(&mut self, task_id: TaskId, user_id: AccountId) {
        let initial_storage = env::storage_usage();

        self.ping_task(&task_id);

        let mut task = self.task_metadata_by_id.get(&task_id).unwrap();

        if let TaskState::Completed = task.task_state {
            if task.is_past_deadline() {
                let company_id = env::predecessor_account_id();

                require!(company_id == task.company_id, "invalid company");

                let submission_set = self.submissions_per_task.get(&task_id).unwrap();

                require!(
                    submission_set.get(&user_id).is_some(),
                    "given user has no submissions for this task"
                );

                task.task_state = TaskState::Payed;

                self.task_metadata_by_id.insert(&task_id, &task);

                self.transfer_reward_to(&task_id, &user_id);

                self.update_user_carbonite_metadata_for_task(&task_id, &user_id);

                let storage_used = env::storage_usage() - initial_storage;

                refund_excess_deposit(storage_used);
                // make gas checks for promise to go through
                // todo!()
            } else {
                env::panic_str("can't select tasks until deadline has reached");
            }
        } else {
            env::panic_str("can't select task now");
        }
    }

    /// claim refunds of expired or overdue task
    pub fn claim_refund(&mut self, task_id: TaskId) {
        let initial_storage = env::storage_usage();

        self.ping_task(&task_id);

        let task = self
            .task_metadata_by_id
            .get(&task_id)
            .unwrap_or_else(|| env::panic_str("invalid task"));

        match task.task_state {
            TaskState::Expired | TaskState::Overdue => {
                let company_id = env::predecessor_account_id();

                self.internal_remove_tasks_from_company(&company_id, &task_id);
                self.task_metadata_by_id.remove(&task_id);

                self.internal_remove_task_invitations_per_user(&task_id, &task);

                self.transfer_reward_to(&task_id, &company_id);

                let storage_used = initial_storage - env::storage_usage();
                let storage_cost = env::storage_byte_cost() * (storage_used as u128);

                Promise::new(company_id).transfer(storage_cost);

                // gas checks for promise to go through
                // todo!();
            }
            _ => env::panic_str(
                "can't claim refunds for tasks that are pending / open / completed / payed",
            ),
        }
    }
}

impl Contract {
    /// asserts that passed company is one of the whitelisted companies else panic
    pub(crate) fn assert_whitelisted_company(&self, company_id: &AccountId) {
        require!(
            self.whitelisted_companies.get(company_id).is_some(),
            "Invalid company"
        );
    }
}

/// asserts that passed account ID is exactly of form company_name-Co.carbonite.near
pub(crate) fn assert_valid_carbonite_company_account_pattern(account_id: &AccountId) {

    let account_id = account_id.as_str();

    let (mut company_name, carbonite_contract_id) = account_id
        .split_once(".")
        .unwrap_or_else(|| env::panic_str("Invalid account ID passed"));

    require!(company_name.ends_with("-co"), "Invalid company name passed");

    (company_name, _) = company_name.split_once("-").unwrap();

    require!(
        company_name
            .bytes()
            .into_iter()
            .all(|c| matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')),
        "Invalid company name passed"
    );

    require!(
        carbonite_contract_id == env::current_account_id().as_str(),
        "Invalid account ID passed"
    );
}
