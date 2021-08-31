#![no_std]

elrond_wasm::imports!();

/// A contract that allows anyone to send a fixed sum, locks it for a while and then allows users to take it back.
/// Sending funds to the contract is called "ping".
/// Taking the same funds back is called "pong".
///
/// Restrictions:
/// - Only the set amount can be `ping`-ed, no more, no less.
/// - `pong` can only be called after a certain period after `ping`.
#[elrond_wasm::contract]
pub trait PingPong {
    /// Necessary configuration when deploying:
    /// `ping_amount` - the exact amount that needs to be sent when `ping`-ing.  
    /// `duration_in_seconds` - how much time (in seconds) until `pong` can be called after the initial `ping` call  
    /// `token_id` - Optional. The Token Identifier of the token that is going to be used. Default is "EGLD".
    #[init]
    fn init(
        &self,
        ping_amount: Self::BigUint,
        duration_in_seconds: u64,
        #[var_args] opt_token_id: OptionalArg<TokenIdentifier>,
    ) -> SCResult<()> {
        require!(ping_amount > 0, "Ping amount cannot be set to zero");
        self.ping_amount().set(&ping_amount);

        require!(
            duration_in_seconds > 0,
            "Duration in seconds cannot be set to zero"
        );
        self.duration_in_seconds().set(&duration_in_seconds);

        let token_id = opt_token_id
            .into_option()
            .unwrap_or_else(TokenIdentifier::egld);
        require!(
            token_id.is_egld() || token_id.is_valid_esdt_identifier(),
            "Invalid token provided"
        );

        Ok(())
    }

    /// User sends some tokens to be locked in the contract for a period of time.
    /// Optional `_data` argument is ignored.
    #[payable("*")]
    #[endpoint]
    fn ping(
        &self,
        #[payment_token] payment_token: TokenIdentifier,
        #[payment_amount] payment_amount: Self::BigUint,
    ) -> SCResult<()> {
        require!(
            payment_token == self.accepted_payment_token_id().get(),
            "Invalid payment token"
        );
        require!(
            payment_amount == self.ping_amount().get(),
            "The payment must match the fixed ping amount"
        );

        let caller = self.blockchain().get_caller();
        require!(!self.did_user_ping(&caller), "Already pinged");

        let current_block_timestamp = self.blockchain().get_block_timestamp();
        self.user_ping_timestamp(&caller)
            .set(&current_block_timestamp);

        Ok(())
    }

    /// User can take back funds from the contract.
    /// Can only be called after expiration.
    #[endpoint]
    fn pong(&self) -> SCResult<()> {
        let caller = self.blockchain().get_caller();
        require!(self.did_user_ping(&caller), "Must ping first");

        let user_ping_timestamp = self.user_ping_timestamp(&caller).get();
        let duration_in_seconds = self.duration_in_seconds().get();
        let pong_enable_timestamp = user_ping_timestamp + duration_in_seconds;

        let current_timestamp = self.blockchain().get_block_timestamp();
        require!(
            current_timestamp >= pong_enable_timestamp,
            "Cannot pong before deadline"
        );

        self.user_ping_timestamp(&caller).clear();

        let token_id = self.accepted_payment_token_id().get();
        let amount = self.ping_amount().get();

        self.send()
            .direct(&caller, &token_id, 0, &amount, b"pong successful");

        Ok(())
    }

    #[view(didUserPing)]
    fn did_user_ping(&self, address: &Address) -> bool {
        !self.user_ping_timestamp(address).is_empty()
    }

    // storage

    #[view(getAcceptedPaymentToken)]
    #[storage_mapper("acceptedPaymentTokenId")]
    fn accepted_payment_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(getPingAmount)]
    #[storage_mapper("pingAmount")]
    fn ping_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(getDurationTimestamp)]
    #[storage_mapper("durationInSeconds")]
    fn duration_in_seconds(&self) -> SingleValueMapper<Self::Storage, u64>;

    #[view(getUserPingTimestamp)]
    #[storage_mapper("userPingTimestamp")]
    fn user_ping_timestamp(&self, address: &Address) -> SingleValueMapper<Self::Storage, u64>;
}
