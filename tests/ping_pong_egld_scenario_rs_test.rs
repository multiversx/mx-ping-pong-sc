use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "mxsc:output/ping-pong.mxsc.json",
        ping_pong::ContractBuilder,
    );
    blockchain
}

#[test]
fn ping_pong_call_ping_rs() {
    world().run("scenarios/ping-pong-call-ping.scen.json");
}

#[test]
fn ping_pong_call_ping_second_user_rs() {
    world().run("scenarios/ping-pong-call-ping-second-user.scen.json");
}

#[test]
fn ping_pong_call_ping_twice_rs() {
    world().run("scenarios/ping-pong-call-ping-twice.scen.json");
}

#[test]
fn ping_pong_call_ping_wrong_amount_rs() {
    world().run("scenarios/ping-pong-call-ping-wrong-amount.scen.json");
}

#[test]
fn ping_pong_call_pong_rs() {
    world().run("scenarios/ping-pong-call-pong.scen.json");
}

#[test]
fn ping_pong_call_pong_before_deadline_rs() {
    world().run("scenarios/ping-pong-call-pong-before-deadline.scen.json");
}

#[test]
fn ping_pong_call_pong_twice_rs() {
    world().run("scenarios/ping-pong-call-pong-twice.scen.json");
}

#[test]
fn ping_pong_call_pong_without_ping_rs() {
    world().run("scenarios/ping-pong-call-pong-without-ping.scen.json");
}

#[test]
fn ping_pong_init_rs() {
    world().run("scenarios/ping-pong-init.scen.json");
}
