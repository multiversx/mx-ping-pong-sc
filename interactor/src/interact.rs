mod interact_cli;
mod interact_config;
mod interact_state;
mod ping_pong_proxy;

use clap::Parser;
pub use interact_config::Config;
use interact_state::State;
use multiversx_sc_snippets::imports::*;

const PING_PONG_CODE: MxscPath = MxscPath::new("../output/ping-pong.mxsc.json");
pub const EGLD: &str = "EGLD";

pub async fn ping_pong_cli() {
    env_logger::init();

    let config = Config::load_config();

    let mut interact = PingPongInteract::new(config).await;
    let cli = interact_cli::InteractCli::parse();

    match &cli.command {
        Some(interact_cli::InteractCliCommand::Deploy(args)) => {
            let duration = DurationMillis::new(args.duration);
            interact.deploy(args.amount, duration, &args.token_id).await;
        }
        Some(interact_cli::InteractCliCommand::Upgrade(args)) => {
            let duration = DurationMillis::new(args.duration);
            interact.upgrade(args.amount, duration).await;
        }
        Some(interact_cli::InteractCliCommand::Ping(args)) => {
            let sender = interact.wallet_address_1.clone();
            interact
                .ping(&args.token, args.nonce, args.amount, &sender, None)
                .await;
        }
        Some(interact_cli::InteractCliCommand::Pong) => {
            let sender = interact.wallet_address_1.clone();
            interact.pong(&sender, None).await;
        }
        Some(interact_cli::InteractCliCommand::DidUserPing(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.did_user_ping(&address).await;
        }
        Some(interact_cli::InteractCliCommand::GetPongEnableTimestamp(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.get_pong_enable_timestamp(&address).await;
        }
        Some(interact_cli::InteractCliCommand::GetTimeToPong(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.get_time_to_pong(&address).await;
        }
        Some(interact_cli::InteractCliCommand::GetAcceptedPaymentToken) => {
            interact.accepted_payment_token_id().await;
        }
        Some(interact_cli::InteractCliCommand::GetPingAmount) => {
            interact.ping_amount().await;
        }
        Some(interact_cli::InteractCliCommand::GetDurationTimestamp) => {
            interact.duration_in_millis().await;
        }
        Some(interact_cli::InteractCliCommand::GetUserPingTimestamp(args)) => {
            let address = Bech32Address::from_bech32_string(args.address.clone());
            interact.user_ping_timestamp(&address).await;
        }
        None => {}
    }
}

pub struct PingPongInteract {
    pub interactor: Interactor,
    pub wallet_address_1: Bech32Address,
    pub wallet_address_2: Bech32Address,
    pub state: State,
}

impl PingPongInteract {
    pub async fn new(config: Config) -> Self {
        let mut interactor = Interactor::new(config.gateway_uri())
            .await
            .use_chain_simulator(config.use_chain_simulator());

        interactor.set_current_dir_from_workspace("interactor");
        let wallet_address_1 = interactor.register_wallet(test_wallets::alice()).await;
        let wallet_address_2 = interactor.register_wallet(test_wallets::mike()).await;

        // Useful in the chain simulator setting
        // generate blocks until ESDTSystemSCAddress is enabled
        interactor.generate_blocks_until_epoch(1).await.unwrap();

        PingPongInteract {
            interactor,
            wallet_address_1: wallet_address_1.into(),
            wallet_address_2: wallet_address_2.into(),
            state: State::load_state(),
        }
    }

    pub async fn deploy(&mut self, amount: u128, duration: DurationMillis, token_id: &str) {
        let managed_token_id = ManagedBuffer::from(token_id);
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address_1)
            .gas(30_000_000u64)
            .typed(ping_pong_proxy::PingPongProxy)
            .init(
                amount,
                duration,
                OptionalValue::Some(EgldOrEsdtTokenIdentifier::parse(managed_token_id)),
            )
            .code(PING_PONG_CODE)
            .returns(ReturnsNewBech32Address)
            .run()
            .await;

        println!("new address: {new_address}");
        self.state.set_ping_pong_address(new_address);
    }

    pub async fn upgrade(&mut self, amount: u128, duration: DurationMillis) {
        let upgrade_address = self
            .interactor
            .tx()
            .from(&self.wallet_address_1)
            .to(self.state.current_ping_pong_address())
            .gas(30_000_000u64)
            .typed(ping_pong_proxy::PingPongProxy)
            .upgrade(amount, duration)
            .code(PING_PONG_CODE)
            .returns(ReturnsNewBech32Address)
            .run()
            .await;

        println!("new upgrade address: {upgrade_address}");
        self.state.set_ping_pong_address(upgrade_address);
    }

    pub async fn ping(
        &mut self,
        token_id: &str,
        nonce: u64,
        amount: u128,
        sender: &Bech32Address,
        message: Option<&str>,
    ) {
        let managed_token_id = ManagedBuffer::from(token_id);
        let payment = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::parse(managed_token_id),
            nonce,
            BigUint::from(amount),
        );

        let response = self
            .interactor
            .tx()
            .from(sender)
            .to(self.state.current_ping_pong_address())
            .gas(30_000_000u64)
            .typed(ping_pong_proxy::PingPongProxy)
            .ping()
            .payment(payment)
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

    pub async fn did_user_ping(&mut self, address: &Bech32Address) -> bool {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .did_user_ping(address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn get_pong_enable_timestamp(&mut self, address: &Bech32Address) -> TimestampMillis {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .get_pong_enable_timestamp(address)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn get_time_to_pong(&mut self, address: &Bech32Address) -> Option<DurationMillis> {
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

    pub async fn duration_in_millis(&mut self) -> DurationMillis {
        self.interactor
            .query()
            .to(self.state.current_ping_pong_address())
            .typed(ping_pong_proxy::PingPongProxy)
            .duration_in_milliseconds()
            .returns(ReturnsResultUnmanaged)
            .run()
            .await
    }

    pub async fn user_ping_timestamp(&mut self, address: &Bech32Address) -> TimestampMillis {
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
