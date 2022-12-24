elrond_wasm::imports!();
elrond_wasm::derive_imports!();

use crate::models::*;

#[elrond_wasm::module]
pub trait StorageModule {

    // SET STATE

    #[only_owner]
    #[endpoint(changeSigner)]
    fn change_signer(&self, new_signer: ManagedAddress) {
        self.signer().set(&new_signer);
    }

    #[only_owner]
    #[endpoint(whitelistAddress)]
    fn whitelist_address(&self, new_whitelisted: ManagedAddress) {
        self.whitelisted(new_whitelisted).set(true);
    }

    #[only_owner]
    #[endpoint(removeWhitelistAddress)]
    fn remove_whitelist_address(&self, whitelisted: ManagedAddress) {
        self.whitelisted(whitelisted).clear();
    }

    // STORAGE

    #[storage_mapper("signer")]
    fn signer(&self) -> SingleValueMapper<ManagedAddress>;

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


    #[storage_mapper("rewardsOwner")]
    fn rewards_owner(&self, hash: &ManagedHash<Self::Api>) -> SingleValueMapper<ManagedAddress>;

}