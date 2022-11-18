elrond_wasm::imports!();

use crate::rewards::*;

#[elrond_wasm::module]
pub trait ViewsModule: crate::rewards::RewardsModule {
    
    #[view(getRewardsCheckpoint)]
    fn get_rewards_checkpoint(&self, root_hash: &ManagedHash<Self::Api>) -> MultiValue4<BigUint, TokenIdentifier, BigUint, u64> {
        let reward = self.rewards_checkpoints(root_hash).get();

        MultiValue4::from((reward.total_nft_supply, reward.reward_token, reward.reward_supply, reward.reward_nonce))
    }

    #[view(getRewardsClaimed)]
    fn get_rewards_claimed(&self, user: &ManagedAddress<Self::Api>, root_hash: &ManagedHash<Self::Api>) -> bool {
        self.rewards_claimed(user, root_hash).get()
    }

}