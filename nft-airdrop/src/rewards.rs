elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use elrond_wasm::elrond_codec::NestedDecodeInput;

pub type ManagedHash<M> = ManagedByteArray<M, 32>;

#[derive(TypeAbi, TopEncode)]
pub struct RewardsCheckpoint<M: ManagedTypeApi> {
    pub total_nft_supply: BigUint<M>,
    pub reward_token: TokenIdentifier<M>,
    pub reward_supply: BigUint<M>,
    pub reward_nonce: u64,
}

#[elrond_wasm::module]
pub trait RewardsModule {
    
    #[payable("*")]
    #[endpoint(addRewardsCheckpoint)]
    fn add_rewards_checkpoint(
        &self,
        root_hash: ManagedHash<Self::Api>,
        total_nft_supply: BigUint
    ) {
        require!(
            self.rewards_checkpoints(&root_hash).is_empty(),
            "Checkpoint already exists"
        );
        let caller = self.blockchain().get_caller();
        require!(self.whitelisted(caller).get(), "Not allowed to deposit!");

        let (reward_token, reward_nonce, reward_supply) = self.call_value().payment_as_tuple();

        require!(reward_supply > 0, "Amount must be higher than 0");

        let checkpoint = RewardsCheckpoint {
            total_nft_supply,
            reward_token,
            reward_supply,
            reward_nonce,
        };
        self.rewards_checkpoints(&root_hash).set(&checkpoint);
    }

    fn calculate_reward_amount(
        &self,
        rewards_supply: BigUint,
        user_nft_amount: u32,
        total_nft_supply: BigUint,
    ) -> BigUint {
        (rewards_supply * BigUint::from(user_nft_amount.clone())) / total_nft_supply
    }

    // Storage
    #[view(rewardsCheckpoints)]
    #[storage_mapper("rewardsCheckpoints")]
    fn rewards_checkpoints(
        &self,
        root_hash: &ManagedHash<Self::Api>,
    ) -> SingleValueMapper<RewardsCheckpoint<Self::Api>>;

    #[storage_mapper("rewardsClaimed")]
    fn rewards_claimed(
        &self,
        user: &ManagedAddress<Self::Api>,
        root_hash: &ManagedHash<Self::Api>,
    ) -> SingleValueMapper<bool>;

    #[storage_mapper("whitelisted")]
    fn whitelisted(&self, address: ManagedAddress) -> SingleValueMapper<bool>;
}

impl<M: ManagedTypeApi> TopDecode for RewardsCheckpoint<M> {
    fn top_decode<I>(input: I) -> Result<Self, DecodeError>
    where
        I: elrond_codec::TopDecodeInput,
    {
        let mut input = input.into_nested_buffer();

        let total_nft_supply = BigUint::dep_decode(&mut input)?;
        let reward_token = TokenIdentifier::dep_decode(&mut input)?;
        let reward_supply = BigUint::dep_decode(&mut input)?;

        let reward_nonce = if input.is_depleted() {
            0u64
        } else {
            u64::dep_decode(&mut input)?
        };

        Result::Ok(RewardsCheckpoint {
            total_nft_supply,
            reward_token,
            reward_supply,
            reward_nonce,
        })
    }
}
