mod interact_cli;
mod interact_config;
mod interact_state;
mod ping_pong_proxy;

use clap::Parser;
pub use interact_config::Config;
use interact_state::State;
use multiversx_sc_snippets::imports::*;

const PING_PONG_CODE: MxscPath = MxscPath::new("output/ping-pong.mxsc.json");
pub const EGLD: &str = "EGLD";

pub async fn ping_pong_cli() {
    env_logger::init();

    let config = Config::load_config();

    let mut interact = PingPongInteract::new(config).await;
    let cli = interact_cli::InteractCli::parse();

    match &cli.command {
        Some(interact_cli::InteractCliCommand::Deploy(args)) => {
            interact
                .deploy(
                    args.ping_amount.clone(),
                    args.duration_in_seconds,
                    args.token_id.clone(),
                )
                .await;
        }
        Some(interact_cli::InteractCliCommand::Upgrade(args)) => {
            interact
                .upgrade(args.ping_amount.clone(), args.duration_in_seconds)
                .await;
        }
        Some(interact_cli::InteractCliCommand::Ping(args)) => {
            interact
                .ping(
                    args.token.clone(),
                    args.nonce,
                    args.amount,
                    &interact.alice_wallet_address.clone(),
                    None,
                )
                .await;
        }
        Some(interact_cli::InteractCliCommand::Pong) => {
            interact
                .pong(&interact.alice_wallet_address.clone(), None)
                .await;
        }
        Some(interact_cli::InteractCliCommand::DidUserPing(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.did_user_ping(address).await;
        }
        Some(interact_cli::InteractCliCommand::GetPongEnableTimestamp(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.get_pong_enable_timestamp(address).await;
        }
        Some(interact_cli::InteractCliCommand::GetTimeToPong(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.get_time_to_pong(address).await;
        }
        Some(interact_cli::InteractCliCommand::GetAcceptedPaymentToken) => {
            interact.accepted_payment_token_id().await;
        }
        Some(interact_cli::InteractCliCommand::GetPingAmount) => {
            interact.ping_amount().await;
        }
        Some(interact_cli::InteractCliCommand::GetDurationTimestamp) => {
            interact.duration_in_seconds().await;
        }
        Some(interact_cli::InteractCliCommand::GetUserPingTimestamp(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.user_ping_timestamp(address).await;
        }
        None => {}
    }
}

pub struct PingPongInteract {
    pub interactor: Interactor,
    pub alice_wallet_address: Bech32Address,
    pub mike_wallet_address: Bech32Address,
    pub state: State,
}

impl PingPongInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("ping-pong");
        let alice_wallet_address = interactor.register_wallet(test_wallets::alice()).await;
        let mike_wallet_address = interactor.register_wallet(test_wallets::mike()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        PingPongInteract {
            interactor,
            alice_wallet_address: alice_wallet_address.into(),
            mike_wallet_address: mike_wallet_address.into(),
            state: State::load_state(),
        }
    }

    pub async fn deploy(
        &mut self,
        ping_amount: RustBigUint,
        duration_in_seconds: u64,
        token_id: String,
    ) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.alice_wallet_address)
            .gas(30_000_000u64)
            .typed(ping_pong_proxy::PingPongProxy)
            .init(
                ping_amount,
                duration_in_seconds,
                OptionalValue::Some(get_token_identifier(token_id)),
            )
            .code(PING_PONG_CODE)
            .returns(ReturnsNewAddress)
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_ping_pong_address(Bech32Address::from_bech32_string(
                new_address_bech32.clone(),
            ));

        println!("new address: {new_address_bech32}");
    }

    pub async fn upgrade(&mut self, ping_amount: RustBigUint, duration_in_seconds: u64) {
        let upgrade_address = self
            .interactor
            .tx()
            .from(&self.alice_wallet_address)
            .to(self.state.current_ping_pong_address())
            .gas(30_000_000u64)
            .typed(ping_pong_proxy::PingPongProxy)
            .upgrade(ping_amount, duration_in_seconds)
            .code(PING_PONG_CODE)
            .returns(ReturnsNewAddress)
            .run()
            .await;

        let upgrade_address_bech32 = bech32::encode(&upgrade_address);
        self.state
            .set_ping_pong_address(Bech32Address::from_bech32_string(
                upgrade_address_bech32.clone(),
            ));

        println!("new upgrade address: {upgrade_address_bech32}");
    }

    pub async fn ping(
        &mut self,
        token_id: String,
        nonce: u64,
        amount: u64,
        sender: &Bech32Address,
        message: Option<&str>,
    ) {
        let response = self
            .interactor
            .tx()
            .from(sender)
            .to(self.state.current_ping_pong_address())
            .gas(30_000_000u64)
            .typed(ping_pong_proxy::PingPongProxy)
            .ping()
            .payment(EgldOrEsdtTokenPayment::new(
                get_token_identifier(token_id),
                nonce,
                BigUint::from(amount),
            ))
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        match response {
            Ok(_) => println!("Ping successfully executed"),
            Err(err) => {
                println!("Ping failed with message: {}", err.message);
                assert_eq!(message.unwrap_or_default(), err.message);
            }
        }
    }

    pub async fn pong(&mut self, sender: &Bech32Address, message: Option<&str>) {
        let response = self
            .interactor
            .tx()
            .from(sender)
            .to(self.state.current_ping_pong_address())
            .gas(30_000_000u64)
            .typed(ping_pong_proxy::PingPongProxy)
            .pong()
            .returns(ReturnsHandledOrError::new())
            .run()
            .await;

        match response {
            Ok(_) => println!("Pong successfully executed"),
            Err(err) => {
                println!("Pong failed with message: {}", err.message);
                assert_eq!(message.unwrap_or_default(), err.message);
            }
        }
    }

    pub async fn did_user_ping(&mut self, address: Bech32Address) -> bool {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .did_user_ping(address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn get_pong_enable_timestamp(&mut self, address: Bech32Address) -> u64 {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .get_pong_enable_timestamp(address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn get_time_to_pong(&mut self, address: Bech32Address) -> Option<u64> {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .get_time_to_pong(address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        match result_value {
            OptionalValue::Some(time) => Some(time),
            OptionalValue::None => {
                println!("Address unavailable");
                None
            }
        }
    }

    pub async fn accepted_payment_token_id(&mut self) -> String {
        let result_value = self
            .interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .accepted_payment_token_id()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        if result_value.is_egld() {
            return EGLD.to_owned();
        }

        result_value.into_esdt_option().unwrap().to_string()
    }

    pub async fn ping_amount(&mut self) -> RustBigUint {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .ping_amount()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn duration_in_seconds(&mut self) -> u64 {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .duration_in_seconds()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn user_ping_timestamp(&mut self, address: Bech32Address) -> u64 {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .user_ping_timestamp(address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }
}

fn get_token_identifier(token_id: String) -> EgldOrEsdtTokenIdentifier<StaticApi> {
    if token_id.to_uppercase().eq(EGLD) {
        EgldOrEsdtTokenIdentifier::egld()
    } else {
        EgldOrEsdtTokenIdentifier::esdt(&token_id)
    }
}
