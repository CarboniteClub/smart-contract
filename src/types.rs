use ts_rs::TS;
use std::collections::HashSet;

pub type TaskId = String;
pub type AccountId = String;
pub type Timestamp = u64;
pub type Balance = u128;
pub type Skills = String;
pub type Base64VecU8 = String;
pub type PublicKey = String;

#[derive(TS)]
#[ts(export)]
pub struct Company {
    pub name: String,
    pub icon: String,             // Data URL of company logo
    pub industries: String, // various industries (comma seperated) in which company is working
    pub description: String, // short description about the company
    pub location: Option<String>, // None if company is remote or else represents headquarter location
    pub reference: String,        // website url of the company
}

#[derive(TS)]
#[ts(export)]
pub struct CompanyRegDetails {
    pub account_id: AccountId,
    pub company: Company,
    pub public_key: PublicKey,
}


#[derive(TS)]
#[ts(export)]
pub struct JsonCompany {
    pub account_id: AccountId,
    pub details: Company,
}

#[derive(TS)]
#[ts(export)]
pub struct NFTContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: String,                // Data URL
    pub base_uri: String, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: String, // URL to a JSON file with more info
    pub reference_hash: Base64VecU8, // Base64-encoded sha256 hash of JSON from reference field
}

#[derive(TS)]
#[ts(export)]
pub struct TokenMetadata {
    pub title: String,
    pub description: Option<String>, // free-form description, can be used as small about me section
    pub media: String, // URL to associated media stored on decentralised storage platform
    pub media_hash: Base64VecU8,
    pub issued_at: Timestamp, // When token was issued or minted, Unix epoch in milliseconds
    pub updated_at: Option<Timestamp>,
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON. for our purpose it can be achievement
    pub carbonite_metadata: CarboniteMetdata,
    pub reference: String, // URL to an off-chain JSON file with more info
    pub reference_hash: Base64VecU8, // Base64-encoded sha256 hash of JSON from reference field
}

#[derive(TS)]
#[ts(export)]
pub struct CarboniteMetdata {
    pub xp: u16,
    pub tasks_completed: Vec<TaskId>,
    pub total_tasks_completed: u16,
}

#[derive(TS)]
#[ts(export)]
pub struct JsonToken {
    pub token_id: String,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
}

#[derive(TS)]
#[ts(export)]
pub enum TaskType {
    InviteOnly {
        invited_accounts: HashSet<AccountId>, // should be ideally be 3
        valid_till: Timestamp,                // unix epoch in ms
    }, // keeps track of invited accounts if an invite only project and validity date till if which if no-one accepts then company can claim refund
    ForEveryone, // this task can be taken up by anyone the company has the choice to select the winner
}

#[derive(TS)]
#[ts(export)]
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

#[derive(TS)]
#[ts(export)]
pub struct TaskDetails {
    pub title: String,
    pub description: String,     // short description about the task
    pub required_skills: Skills, // required skills for the task in a comma seperated format
    pub task_type: TaskType,
    pub reference: String, // URL to an off-chain JSON file with more info, preferably a decentralised storage in encrypted format
    pub reference_hash: Base64VecU8, // Base64-encoded sha256 hash of Jencrypted JSON file itself from reference field
}

#[derive(TS)]
#[ts(export)]
pub struct Submission {
    pub submission_reference: String, // link to the decentralised submitted documents in encrypted format (preferrably)
    pub submission_reference_hash: Base64VecU8,
}

#[derive(TS)]
#[ts(export)]
pub struct JsonSubmission {
    pub task_id: TaskId,
    pub account_id: AccountId,
    pub submission: Submission,
}

#[derive(TS)]
#[ts(export)]
pub struct Task {
    pub task_details: TaskDetails,
    pub company_id: AccountId, // account ID of the company giving this task
    pub deadline: Timestamp, // if task is not completed till this (unix epoch in ms) then company can claim refund
    pub person_assigned: Option<AccountId>, // person assigned or person that accepted the invite for task in an invite only task
    pub task_state: TaskState,
    pub ft_contract_id: AccountId, // contract ID of approved token used to pay
    pub reward: Balance, // reward amount in smallest unit of tokens, Eg: for near it will be yoctoNEAR}
}

#[derive(TS)]
#[ts(export)]
pub struct JsonTask {
    pub task_id: TaskId,
    pub task: Task,
}