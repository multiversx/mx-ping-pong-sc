#![no_std]

elrond_wasm::imports!();

mod user_status;

use user_status::UserStatus;

/// Derived empirically.
const PONG_ALL_LOW_GAS_LIMIT: u64 = 3_000_000;

/// A contract that allows anyone to send a fixed sum, locks it for a while and then allows users to take it back.
/// Sending funds to the contract is called "ping".
/// Taking the same funds back is called "pong".
///
/// Restrictions:
/// - `ping` can be called only after the contract is activated. By default the contract is activated on deploy.
/// - Users can only `ping` once, ever.
/// - Only the set amount can be `ping`-ed, no more, no less.
/// - The contract can optionally have a maximum cap. No more users can `ping` after the cap has been reached.
/// - The `ping` endpoint optionally accepts
/// - `pong` can only be called after the contract expired (a certain duration has passed since activation).
/// - `pongAll` can be used to send to all users to `ping`-ed. If it runs low on gas, it will interrupt itself.
/// It can be continued anytime.
#[elrond_wasm::contract]
pub trait PingPong {
    /// Necessary configuration when deploying:
    /// `token_id` - the Token Identifier of the token that is going to be used. Pass "EGLD" if you want to use EGLD instead of an ESDT.
    /// `ping_amount` - the exact amount that needs to be sent when `ping`-ing.
    /// `duration_in_seconds` - how much time (in seconds) until contract expires.
    /// `opt_activation_timestamp` - optionally specify the contract to only actvivate at a later date.
    /// `max_funds` - optional funding cap, no more funds than this can be added to the contract.
    #[init]
    fn init(
        &self,
        token_id: TokenIdentifier,
        ping_amount: Self::BigUint,
        duration_in_seconds: u64,
        #[var_args] opt_activation_timestamp: OptionalArg<u64>,
        #[var_args] max_funds: OptionalArg<Self::BigUint>,
    ) -> SCResult<()> {
        require!(
            token_id.is_egld() || token_id.is_valid_esdt_identifier(),
            "Invalid token provided"
        );

        self.ping_amount().set(&ping_amount);

        let activation_timestamp = opt_activation_timestamp
            .into_option()
            .unwrap_or_else(|| self.blockchain().get_block_timestamp());
        let deadline = activation_timestamp + duration_in_seconds;

        self.activation_timestamp().set(&activation_timestamp);
        self.deadline().set(&deadline);
        self.max_funds().set(&max_funds.into_option());

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
            "invalid payment token"
        );
        require!(
            payment_amount == self.ping_amount().get(),
            "the payment must match the fixed sum"
        );

        let activation_timestamp = self.activation_timestamp().get();
        let current_block_timestamp = self.blockchain().get_block_timestamp();
        require!(
            activation_timestamp <= current_block_timestamp,
            "smart contract not active yet"
        );

        require!(
            current_block_timestamp < self.deadline().get(),
            "deadline has passed"
        );

        if let Some(max_funds) = self.max_funds().get() {
            let sc_balance_after_transfer = self.blockchain().get_sc_balance(&payment_token, 0);
            require!(
                sc_balance_after_transfer <= max_funds,
                "smart contract full"
            );
        }

        let caller = self.blockchain().get_caller();
        let user_id = self.user_mapper().get_or_create_user(&caller);
        let user_status = self.user_status(user_id).get();

        match user_status {
            UserStatus::New => {
                self.user_status(user_id).set(&UserStatus::Registered);
                Ok(())
            }
            UserStatus::Registered => {
                sc_error!("can only ping once")
            }
            UserStatus::Withdrawn => {
                sc_error!("already withdrawn")
            }
        }
    }

    fn pong_by_user_id(&self, user_id: usize) -> SCResult<()> {
        let user_status = self.user_status(user_id).get();
        match user_status {
            UserStatus::New => {
                sc_error!("can't pong, never pinged")
            }
            UserStatus::Registered => {
                self.user_status(user_id).set(&UserStatus::Withdrawn);

                if let Some(user_address) = self.user_mapper().get_user_address(user_id) {
                    let token_id = self.accepted_payment_token_id().get();
                    let amount = self.ping_amount().get();

                    self.send()
                        .direct(&user_address, &token_id, 0, &amount, b"pong");
                        
                    Ok(())
                } else {
                    sc_error!("unknown user")
                }
            }
            UserStatus::Withdrawn => {
                sc_error!("already withdrawn")
            }
        }
    }

    /// User can take back funds from the contract.
    /// Can only be called after expiration.
    #[endpoint]
    fn pong(&self) -> SCResult<()> {
        require!(
            self.blockchain().get_block_timestamp() >= self.deadline().get(),
            "can't withdraw before deadline"
        );

        let caller = self.blockchain().get_caller();
        let user_id = self.user_mapper().get_user_id(&caller);
        self.pong_by_user_id(user_id)
    }

    /// Send back funds to all users who pinged.
    /// Returns
    /// - `completed` if everything finished
    /// - `interrupted` if run out of gas midway.
    /// Can only be called after expiration.
    #[endpoint(pongAll)]
    fn pong_all(&self) -> SCResult<OperationCompletionStatus> {
        require!(
            self.blockchain().get_block_timestamp() >= self.deadline().get(),
            "can't withdraw before deadline"
        );

        let num_users = self.user_mapper().get_user_count();
        let mut pong_all_last_user = self.pong_all_last_user().get();
        loop {
            if pong_all_last_user >= num_users {
                // clear field and reset to 0
                pong_all_last_user = 0;
                self.pong_all_last_user().set(&pong_all_last_user);
                return Ok(OperationCompletionStatus::Completed);
            }

            if self.blockchain().get_gas_left() < PONG_ALL_LOW_GAS_LIMIT {
                self.pong_all_last_user().set(&pong_all_last_user);
                return Ok(OperationCompletionStatus::InterruptedBeforeOutOfGas);
            }

            pong_all_last_user += 1;
            let _ = self.pong_by_user_id(pong_all_last_user);
        }
    }

    /// Lists the addresses of all users that have `ping`-ed,
    /// in the order they have `ping`-ed
    #[view(getUserAddresses)]
    fn get_user_addresses(&self) -> MultiResultVec<Address> {
        self.user_mapper().get_all_addresses().into()
    }

    // storage

    #[view(getAcceptedPaymentToken)]
    #[storage_mapper("accepted_payment_token_id")]
    fn accepted_payment_token_id(&self) -> SingleValueMapper<Self::Storage, TokenIdentifier>;

    #[view(getPingAmount)]
    #[storage_mapper("ping_amount")]
    fn ping_amount(&self) -> SingleValueMapper<Self::Storage, Self::BigUint>;

    #[view(getDeadline)]
    #[storage_mapper("deadline")]
    fn deadline(&self) -> SingleValueMapper<Self::Storage, u64>;

    /// Block timestamp of the block where the contract got activated.
    /// If not specified in the constructor it is the the deploy block timestamp.
    #[view(getActivationTimestamp)]
    #[storage_mapper("activation_timestamp")]
    fn activation_timestamp(&self) -> SingleValueMapper<Self::Storage, u64>;

    /// Optional funding cap.
    #[view(getMaxFunds)]
    #[storage_mapper("max_funds")]
    fn max_funds(&self) -> SingleValueMapper<Self::Storage, Option<Self::BigUint>>;

    #[storage_mapper("user")]
    fn user_mapper(&self) -> UserMapper<Self::Storage>;

    /// State of user funds.
    /// 0 - user unknown, never `ping`-ed
    /// 1 - `ping`-ed
    /// 2 - `pong`-ed
    #[view(getUserStatus)]
    #[storage_mapper("user_status")]
    fn user_status(&self, user_id: usize) -> SingleValueMapper<Self::Storage, UserStatus>;

    /// Part of the `pongAll` status, the last user to be processed.
    /// 0 if never called `pongAll` or `pongAll` completed..
    #[view(pongAllLastUser)]
    #[storage_mapper("pong_all_last_user")]
    fn pong_all_last_user(&self) -> SingleValueMapper<Self::Storage, usize>;
}
