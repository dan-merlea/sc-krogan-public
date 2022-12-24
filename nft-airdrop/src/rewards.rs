elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::models::*;

#[elrond_wasm::module]
pub trait RewardsModule: crate::storage::StorageModule {
    
    #[payable("*")]
    #[endpoint(addRewardsCheckpoint)]
    fn add_rewards_checkpoint(
        &self,
        root_hash: ManagedHash<Self::Api>,
        total_nft_supply: BigUint
    ) {
        require!(self.rewards_checkpoints(&root_hash).is_empty(), "Checkpoint already exists");
        let caller = self.blockchain().get_caller();
        require!(self.whitelisted(caller.clone()).get(), "Not allowed to deposit!");

        let (reward_token, reward_nonce, reward_supply) = self.call_value().payment_as_tuple();

        require!(reward_supply > 0, "Amount must be higher than 0");

        let checkpoint = RewardsCheckpoint {
            total_nft_supply,
            reward_token,
            reward_supply,
            reward_nonce,
        };
        self.rewards_checkpoints(&root_hash).set(&checkpoint);
        self.rewards_owner(&root_hash).set(caller);
    }

    fn calculate_reward_amount(
        &self,
        rewards_supply: BigUint,
        user_nft_amount: u32,
        total_nft_supply: BigUint,
    ) -> BigUint {
        (rewards_supply * BigUint::from(user_nft_amount.clone())) / total_nft_supply
    }

}