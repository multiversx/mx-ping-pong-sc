use multiversx_sc_snippets::imports::*;
use ping_pong_interact::{Config, PingPongInteract, EGLD};

#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_ping_pong_cs() {
    let mut interactor = PingPongInteract::new(Config::chain_simulator_config()).await;

    let alice = interactor.alice_wallet_address.clone();
    let mike = interactor.mike_wallet_address.clone();
    let amount = RustBigUint::from(1u32);
    let time = 15u64;

    interactor.deploy(amount, time, EGLD.to_string()).await;

    interactor
        .ping(
            EGLD.to_string(),
            0,
            2u64,
            &alice,
            Some("The payment must match the fixed ping amount"),
        )
        .await;
    interactor
        .ping(EGLD.to_string(), 0, 1u64, &alice, None)
        .await;
    assert!(interactor.did_user_ping(alice.clone()).await);

    assert!(!interactor.did_user_ping(mike.clone()).await);
    interactor
        .ping(EGLD.to_string(), 0, 1u64, &mike, None)
        .await;

    assert_eq!(Some(15), interactor.get_time_to_pong(mike.clone()).await);
    assert_eq!(EGLD, interactor.accepted_payment_token_id().await);
    assert_eq!(RustBigUint::from(1u64), interactor.ping_amount().await);
    assert_eq!(time, interactor.duration_in_seconds().await);

    interactor.pong(&alice, None).await;
    interactor.pong(&alice, Some("Must ping first")).await;
}
