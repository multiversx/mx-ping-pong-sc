use multiversx_sc_snippets::imports::*;
use ping_pong_interact::{Config, PingPongInteract, EGLD};

#[tokio::test]
#[cfg_attr(not(feature = "chain-simulator-tests"), ignore)]
async fn test_ping_pong_cs() {
    let mut interactor = PingPongInteract::new(Config::chain_simulator_config()).await;

    let alice = interactor.wallet_address_1.clone();
    let mike = interactor.wallet_address_2.clone();
    let amount = 1u128;
    let duration = DurationMillis::new(15000u64);

    interactor.deploy(amount, duration, EGLD).await;

    interactor
        .ping(
            EGLD,
            0,
            2,
            &alice,
            Some("The payment must match the fixed ping amount"),
        )
        .await;
    interactor.ping(EGLD, 0, 1, &alice, None).await;
    assert!(interactor.did_user_ping(&alice).await);

    assert!(!interactor.did_user_ping(&mike).await);
    interactor.ping(EGLD, 0, 1, &mike, None).await;

    assert_eq!(Some(duration), interactor.get_time_to_pong(&mike).await);
    assert_eq!(EGLD, interactor.accepted_payment_token_id().await);
    assert_eq!(RustBigUint::from(amount), interactor.ping_amount().await);
    assert_eq!(duration, interactor.duration_in_millis().await);

    interactor.pong(&alice, None).await;
    interactor.pong(&alice, Some("Must ping first")).await;
}
