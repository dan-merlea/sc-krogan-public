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
