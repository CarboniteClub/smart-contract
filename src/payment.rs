use crate::*;

impl Contract {
    /// pay's out the reward in token specified in task, no other checks are performed
    pub(crate) fn transfer_reward_to(&mut self, task_id: &TaskId, user_id: &AccountId) {
        let task = self
            .task_metadata_by_id
            .get(&task_id)
            .unwrap_or_else(|| env::panic_str("invalid task"));

        if task.ft_contract_id.as_str() == "near" {
            Promise::new(user_id.clone()).transfer(task.reward);
        } else {
            // handle fungible token transfer
            // todo!();
        }
    }
}
