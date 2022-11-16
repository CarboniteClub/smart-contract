use crate::*;

pub type Skills = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: String,                // Data URL
    pub base_uri: String, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: String, // URL to a JSON file with more info
    pub reference_hash: Base64VecU8, // Base64-encoded sha256 hash of JSON from reference field
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Default, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct CarboniteMetdata {
    pub xp: u16,
    pub tasks_completed: Vec<TaskId>,
    pub total_tasks_completed: u16,
}

#[derive(Serialize, Deserialize, NearSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub token_id: String,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
}

pub trait NonFungibleTokenMetadata {
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadata for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

impl TokenMetadata {
    /// creates the default NFT token metadata with given title and description
    pub fn new_default(title: String, description: Option<String>) -> Self {
        let media_hash = Base64VecU8::from([5_u8; 32].to_vec());
        let reference_hash = Base64VecU8::from([5_u8; 32].to_vec());

        Self {
            title,
            description,
            media: DEFAULT_MEDIA_REFERENCE.to_string(),
            media_hash,
            issued_at: env::block_timestamp_ms(),
            updated_at: None,
            extra: None,
            carbonite_metadata: Default::default(),
            reference: DEFAULT_NFT_REFERENCE.to_string(),
            reference_hash,
        }
    }
}

impl Contract {
    /// updates given user's carbonite metadata corresponding to task_given, no checks except existence of user_id and task_id are performed
    pub(crate) fn update_user_carbonite_metadata_for_task(
        &mut self,
        task_id: &TaskId,
        user_id: &AccountId,
    ) {
        let mut token_metadata = self
            .tokens_by_account_id
            .get(user_id)
            .unwrap_or_else(|| env::panic_str("Invalid user"));

        self.task_metadata_by_id
            .get(&task_id)
            .unwrap_or_else(|| env::panic_str("invalid task"));

        // update carbonite metadata

        token_metadata.carbonite_metadata.total_tasks_completed += 1;

        token_metadata
            .carbonite_metadata
            .tasks_completed
            .push(task_id.clone());

        self.tokens_by_account_id.insert(user_id, &token_metadata);
    }
}
