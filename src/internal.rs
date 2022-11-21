use crate::*;

impl Contract {
    /// adds given token_metadata and associated a/c ID to tokens_by_account_id, panics if given a/c ID was already present in the collection
    pub fn internal_add_token_to_owner(
        &mut self,
        owner_id: &AccountId,
        token_metadata: &TokenMetadata,
    ) {
        require!(
            self.tokens_by_account_id
                .insert(owner_id, token_metadata)
                .is_none(),
            "account ID already exists"
        );
    }

    /// adds given company and associated a/c ID to whitelisted_companies, panics if given a/c ID was already present in the collection
    pub fn internal_add_company_to_whitelisted_companies(
        &mut self,
        company_id: &AccountId,
        company: &Company,
    ) {
        require!(
            self.whitelisted_companies
                .insert(company_id, company)
                .is_none(),
            "company ID already exists"
        );
    }

    /// adds the task_id associated with company_id to collection and panics if task_id already exists
    pub fn internal_add_tasks_to_company(&mut self, company_id: &AccountId, task_id: &TaskId) {
        let mut task_set = self.tasks_by_company.get(&company_id).unwrap_or_else(|| {
            UnorderedSet::new(StorageKey::TasksByCompanyInner {
                company_id_hash: hash_id(company_id.as_str()),
            })
        });

        require!(
            task_set.insert(task_id),
            format!("{company_id} already has task {task_id}")
        );

        self.tasks_by_company.insert(company_id, &task_set);
    }

    /// adds the task_id to associated invited accounts
    pub fn internal_add_task_invitations_per_user(&mut self, task_id: &TaskId, task: &Task) {
        let mut task_set;

        if let TaskType::InviteOnly {
            invited_accounts, ..
        } = &task.task_details.task_type
        {
            for account_id in invited_accounts {
                task_set = self
                    .task_invitations_per_user
                    .get(&account_id)
                    .unwrap_or_else(|| {
                        UnorderedSet::new(StorageKey::TaskInvitationsPerUserInner {
                            account_id_hash: hash_id(account_id.as_str()),
                        })
                    });

                task_set.insert(&task_id);

                self.task_invitations_per_user
                    .insert(&account_id, &task_set);
            }
        }
    }

    /// removes task_invitation from users in case the task is removed due to any reason such as claim_refund
    pub fn internal_remove_task_invitations_per_user(&mut self, task_id: &TaskId, task: &Task) {
        let mut task_set;

        if let TaskType::InviteOnly {
            invited_accounts, ..
        } = &task.task_details.task_type
        {
            for account_id in invited_accounts {
                task_set = self.task_invitations_per_user.get(&account_id).unwrap(); // unwrap would always work bcz it is called for a particular

                task_set.remove(&task_id);

                if task_set.is_empty() {
                    self.task_invitations_per_user.remove(&account_id);
                } else {
                    self.task_invitations_per_user
                        .insert(&account_id, &task_set);
                }
            }
        }
    }

    /// add submission to the given task, panics if re submitting the task
    pub fn internal_add_submission_to_task(
        &mut self,
        task_id: &TaskId,
        user_id: &AccountId,
        submission: &Submission,
    ) {
        let mut submission_map = self.submissions_per_task.get(task_id).unwrap_or_else(|| {
            UnorderedMap::new(StorageKey::SubmissionsPerTaskInner {
                task_id_hash: hash_id(task_id.as_str()),
            })
        });

        require!(
            submission_map.insert(user_id, submission).is_none(),
            "can't resubmit the task"
        );

        self.submissions_per_task.insert(task_id, &submission_map);
    }

    /// removes the task_id and associated company_id from collection and panics if task_id doesn't existss
    pub fn internal_remove_tasks_from_company(&mut self, company_id: &AccountId, task_id: &TaskId) {
        let mut task_set = self
            .tasks_by_company
            .get(company_id)
            .unwrap_or_else(|| env::panic_str("no tasks for this company exists"));

        task_set.remove(task_id);

        if task_set.is_empty() {
            self.tasks_by_company.remove(company_id);
        } else {
            self.tasks_by_company.insert(company_id, &task_set);
        }
    }
}
