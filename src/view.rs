// this file and enumeration.rs and metadata.rs combined has all the view calls of contract

use crate::*;

#[near_bindgen]
impl Contract {
    // return account_id of owner of the contract
    pub fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }

    // returns all recognised skills list -> Paginated (Not really needed but might be required)
    pub fn get_recognised_skills_list(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<Skills> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.recognised_skills
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .collect()
    }

    // get details of a company given company_id
    pub fn get_company_details(&self, account_id: AccountId) -> Option<JsonCompany> {
        if let Some(details) = self.whitelisted_companies.get(&account_id) {
            Some(JsonCompany {
                account_id,
                details,
            })
        } else {
            None
        }
    }

    // check if a particular accountId is in pending company verification list
    // change the fn name to be more approriate
    pub fn has_company_requested_verification(&self, account_id: AccountId) -> bool{
        self.pending_verification_requests.get(&account_id).is_some()
    }

    // get all the companies listed and verified on carbonite
    pub fn get_pending_companies_list(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<CompanyRegDetails> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.pending_verification_requests
            .keys()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|company_id| self.pending_verification_requests.get(&company_id).unwrap())
            .collect()
    }

    // get all the companies listed and verified on carbonite
    pub fn get_whitelisted_companies_list(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonCompany> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.whitelisted_companies
            .keys()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|company_id| self.get_company_details(company_id).unwrap())
            .collect()
    }

    // get the list of all ft tokens supported on carbonite
    pub fn get_approved_ft_tokens_list(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<AccountId> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.approved_ft_tokens
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .collect()
    }

    // get complete task details given a task_id
    pub fn get_task_details(&self, task_id: TaskId) -> Option<JsonTask> {
        if let Some(task) = self.task_metadata_by_id.get(&task_id) {
            Some(JsonTask { task_id, task })
        } else {
            None
        }
    }

    // get all the tasks listed on carbonite
    pub fn get_all_tasks_list(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonTask> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.task_metadata_by_id
            .keys()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|task_id| self.get_task_details(task_id).unwrap())
            .collect()
    }

    // get all the tasks that user is / was invited to
    pub fn get_invited_tasks_for_user_list(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<TaskId> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        let Some(invited_tasks_per_user) = self.task_invitations_per_user.get(&account_id) else{
            return vec![]
        };

        invited_tasks_per_user
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|task_id| task_id)
            .collect()
    }

    // get all the tasks listed by a company
    pub fn get_tasks_from_company_list(
        &self,
        company_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonTask> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        let Some(tasks_by_company) = self.tasks_by_company.get(&company_id) else{
            return vec![]
        };

        tasks_by_company
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|task_id| self.get_task_details(task_id).unwrap())
            .collect()
    }

    // get submission details given task_id and account_id from which submission was made
    pub fn get_submission_details(
        &self,
        task_id: TaskId,
        account_id: AccountId,
    ) -> Option<JsonSubmission> {
        let Some(submissions_map) = self.submissions_per_task.get(&task_id) else{
            return None
        };

        if let Some(submission) = submissions_map.get(&account_id) {
            Some(JsonSubmission {
                task_id,
                account_id,
                submission,
            })
        } else {
            None
        }
    }

    // get all submissions for a task
    pub fn get_submissions_for_task_list(
        &self,
        task_id: TaskId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonSubmission> {
        let Some(submissions_map) = self.submissions_per_task.get(&task_id) else{
            return vec![]
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));

        submissions_map
            .keys()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|account_id| JsonSubmission {
                task_id: task_id.clone(),
                submission: submissions_map.get(&account_id).unwrap(),
                account_id,
            })
            .collect()
    }
}
