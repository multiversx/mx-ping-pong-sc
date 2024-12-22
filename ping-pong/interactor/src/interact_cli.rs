use clap::{Args, Parser, Subcommand};
use multiversx_sc_snippets::imports::RustBigUint;

/// Ping Pong Interact CLI
#[derive(Default, PartialEq, Eq, Debug, Parser)]
#[command(version, about)]
#[command(propagate_version = true)]
pub struct InteractCli {
    #[command(subcommand)]
    pub command: Option<InteractCliCommand>,
}

/// Ping Pong Interact CLI Commands
#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum InteractCliCommand {
    #[command(name = "deploy", about = "Deploy contract.")]
    Deploy(DeployArgs),
    #[command(name = "upgrade", about = "Upgrade contract.")]
    Upgrade(UpgradeArgs),
    #[command(
        name = "ping",
        about = "User sends some EGLD to be locked in the contract for a period of time."
    )]
    Ping(PingArgs),
    #[command(name = "pong", about = "User can take back funds from the contract.")]
    Pong,
    #[command(name = "did-user-ping", about = "Returns if a user ping-ed or not")]
    DidUserPing(DidUserPingArgs),
    #[command(
        name = "pong-enable",
        about = "Returns the timestamp when pong is enabled."
    )]
    GetPongEnableTimestamp(GetPongEnableTimestampArgs),
    #[command(name = "time-to-pong", about = "Returns the time left to pong.")]
    GetTimeToPong(GetTimeToPongArgs),
    #[command(name = "token", about = "Returns accepted token to ping.")]
    GetAcceptedPaymentToken,
    #[command(name = "ping-amount", about = "Returns the ping amount.")]
    GetPingAmount,
    #[command(name = "duration", about = "Returns the duration in seconds.")]
    GetDurationTimestamp,
    #[command(
        name = "user-ping",
        about = "Returns the timestamp at which the user pinged"
    )]
    GetUserPingTimestamp(GetUserPingTimestampArgs),
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct DeployArgs {
    #[arg(short = 'p', long = "ping-amount")]
    pub ping_amount: RustBigUint,

    #[arg(short = 'd', long = "duration-in-seconds")]
    pub duration_in_seconds: u64,

    #[arg(short = 't', long = "token-id", default_value = "EGLD")]
    pub token_id: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct UpgradeArgs {
    #[arg(short = 'p', long = "ping-amount")]
    pub ping_amount: RustBigUint,

    #[arg(short = 'd', long = "duration-in-seconds")]
    pub duration_in_seconds: u64,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct PingArgs {
    #[arg(short = 't', long = "token")]
    pub token: String,

    #[arg(short = 'n', long = "nonce")]
    pub nonce: u64,

    #[arg(short = 'a', long = "amount")]
    pub amount: u64,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct DidUserPingArgs {
    #[arg(short = 'a', long = "address")]
    pub address: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct GetPongEnableTimestampArgs {
    #[arg(short = 'a', long = "address")]
    pub address: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct GetTimeToPongArgs {
    #[arg(short = 'a', long = "address")]
    pub address: String,
}

#[derive(Default, Clone, PartialEq, Eq, Debug, Args)]
pub struct GetUserPingTimestampArgs {
    #[arg(short = 'a', long = "address")]
    pub address: String,
}
