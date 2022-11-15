use crate::*;

#[near_bindgen]
impl Contract {
    //get the total supply of nft NFTs of this contract
    pub fn nft_total_supply(&self) -> U128 {
        U128(self.tokens_by_account_id.len() as u128)
    }

    //query for nft tokens on the contract regardless of the owner using pagination [Common Data of all passes]
    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonToken> {
        let start = u128::from(from_index.unwrap_or(U128(0)));

        self.tokens_by_account_id
            .keys()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|account_id| self.nft_token(&account_id).unwrap())
            .collect()
    }

    //get the total supply of NFTs for a given owner
    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        let tokens_for_owner_set = self.tokens_by_account_id.get(&account_id);

        if let Some(_) = tokens_for_owner_set {
            U128(1)
        } else {
            U128(0)
        }
    }

    //query for all the tokens for an owner
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<U64>,
    ) -> Vec<JsonToken> {
        if let Some(json_token) = self.nft_token(&account_id) {
            vec![json_token]
        } else {
            vec![]
        }
    }

    // return NFT Token related data for a given account_
    pub fn nft_token(&self, account_id: &AccountId) -> Option<JsonToken> {
        if let Some(token_metadata) = self.tokens_by_account_id.get(account_id) {
            let account = account_id.to_string();
            let (username, _) = account.split_once('.').unwrap();

            Some(JsonToken {
                token_id: username.to_owned(),
                owner_id: account_id.clone(),
                metadata: token_metadata,
            })
        } else {
            None
        }
    }
}
