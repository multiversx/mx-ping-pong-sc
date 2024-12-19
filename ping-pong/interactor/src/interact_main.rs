
use multiversx_sc_snippets::imports::*;
use ping_pong_interact::ping_pong_cli;

#[tokio::main]
async fn main() {
    ping_pong_cli().await;
}  

