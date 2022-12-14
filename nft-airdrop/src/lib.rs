#![no_std]

elrond_wasm::imports!();

mod storage;
mod rewards;
mod views;
pub mod models;

use models::*;

const SIGNATURE_LEN: usize = 64;
const MAX_DATA_LEN: usize = 120;

pub type Signature<M> = ManagedByteArray<M, SIGNATURE_LEN>;

#[elrond_wasm::contract]
pub trait NftAirdrop: 
    rewards::RewardsModule + 
    views::ViewsModule +
    storage::StorageModule
{

    #[init]
    fn init(&self, signer: ManagedAddress) {
        self.signer().set_if_empty(&signer);
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(
        &self,
        #[var_args] data: MultiValueEncoded<MultiValue3<ManagedHash<Self::Api>, u32, Signature<Self::Api>>>,
    ) {
        let caller = self.blockchain().get_caller();

        let mut egld_payment_amount = BigUint::zero();
        let mut output_payments = ManagedVec::new();
        let mut last_payment = EsdtTokenPayment::no_payment();

        for user_data in data.into_iter() {
            let (hash, 
                amount, 
                signature) = user_data.into_tuple();

            // Check if owner is present & whitelisted
            if self.rewards_owner(&hash).is_empty() == false {
                let owner = self.rewards_owner(&hash).get();
                require!(self.whitelisted(owner).get(), "Not allowed to claim rewards from this project!");
            }

            self.verify_signature(&caller, &hash, &amount, &signature);
            require!(!self.rewards_claimed(&caller, &hash).get(), "Already claimed rewards for this week");

            let checkpoint_mapper = self.rewards_checkpoints(&hash);
            require!(!checkpoint_mapper.is_empty(), "Invalid root hash");
            let checkpoint: RewardsCheckpoint<Self::Api> = checkpoint_mapper.get();

            let reward_amount = self.calculate_reward_amount(
                checkpoint.reward_supply,
                amount,
                checkpoint.total_nft_supply,
            );

            self.rewards_claimed(&caller, &hash).set(&true);

            if checkpoint.reward_token == TokenIdentifier::egld() {
                egld_payment_amount += reward_amount;
            } else {
                if checkpoint.reward_nonce != 0 {
                    last_payment = EsdtTokenPayment::new(checkpoint.reward_token, checkpoint.reward_nonce, reward_amount);
                    output_payments.push(last_payment.clone());
                    last_payment = EsdtTokenPayment::no_payment();
                } else {
                    if checkpoint.reward_token == last_payment.token_identifier && last_payment.token_type != EsdtTokenType::Invalid {
                        last_payment.amount += reward_amount;
                    } else {
                        if last_payment.token_type != EsdtTokenType::Invalid {
                            output_payments.push(last_payment.clone());
                        }
                        last_payment = EsdtTokenPayment::new(checkpoint.reward_token, checkpoint.reward_nonce, reward_amount);
                    }
                }
            }
        }
        if last_payment.token_type != EsdtTokenType::Invalid {
            output_payments.push(last_payment.clone());
        }
        if egld_payment_amount > 0 {
            self.send().direct_egld(&caller, &egld_payment_amount, &[]);
        }
        if !output_payments.is_empty() {
            self.send().direct_multi(&caller, &output_payments, &[]);
        }

    }

    fn verify_signature(
        &self,
        address: &ManagedAddress<Self::Api>,
        root_hash: &ManagedHash<Self::Api>,
        user_nft_amount: &u32,
        signature: &Signature<Self::Api>,
    ) {
        let data = self.create_hash(&address, root_hash, user_nft_amount);

        let signer: ManagedAddress = self.signer().get();
        let valid_signature = self.crypto().verify_ed25519_managed::<MAX_DATA_LEN>(
            &signer.as_managed_byte_array(),
            &data,
            &signature
        );
        require!(valid_signature, "Invalid signature");
    }

    fn create_hash(&self, address: &ManagedAddress, hash: &ManagedHash<Self::Api>, amount: &u32) -> ManagedBuffer {

        let mut buffer = ManagedBuffer::new();
        buffer.append(address.as_managed_buffer());
        buffer.append(hash.as_managed_buffer());
        buffer.append(&ManagedBuffer::from(b"_"));
        buffer.append(&self.decimal_to_ascii(amount.clone()));

        buffer
    }

    #[only_owner]
    #[endpoint(withdrawAll)]
    fn withdraw_all(
        &self,
        #[var_args] opt_token_identifier: OptionalValue<TokenIdentifier>,
        #[var_args] opt_token_nonce: OptionalValue<u64>
    ) {
        let token_identifier = opt_token_identifier
            .into_option()
            .unwrap_or_else(|| TokenIdentifier::egld());

        let token_nonce = if token_identifier.is_egld() {
            0
        } else {
            opt_token_nonce
                .into_option()
                .unwrap_or_default()
        };

        let owner = self.blockchain().get_owner_address();
        let balance = self.blockchain().get_sc_balance(&token_identifier, token_nonce);
        require!(balance > 0, "Nothing to withdraw.");
        self.send().direct(&owner, &token_identifier, token_nonce, &balance, b"Emergency withdraw");
    }

    fn decimal_to_ascii(&self, mut number: u32) -> ManagedBuffer {
        const MAX_NUMBER_CHARACTERS: usize = 10;
        const ZERO_ASCII: u8 = b'0';

        let mut as_ascii = [0u8; MAX_NUMBER_CHARACTERS];
        let mut nr_chars = 0;

        loop {
            unsafe {
                let reminder: u8 = (number % 10).try_into().unwrap_unchecked();
                number /= 10;

                as_ascii[nr_chars] = ZERO_ASCII + reminder;
                nr_chars += 1;
            }

            if number == 0 {
                break;
            }
        }

        let slice = &mut as_ascii[..nr_chars];
        slice.reverse();

        ManagedBuffer::new_from_bytes(slice)
    }

    
}
