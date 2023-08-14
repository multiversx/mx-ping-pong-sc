use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    ScenarioWorld::vm_go()
}

#[test]
fn ping_pong_call_ping_go() {
    world().run("scenarios/ping-pong-call-ping.scen.json");
}

#[test]
fn ping_pong_call_ping_second_user_go() {
    world().run("scenarios/ping-pong-call-ping-second-user.scen.json");
}

#[test]
fn ping_pong_call_ping_twice_go() {
    world().run("scenarios/ping-pong-call-ping-twice.scen.json");
}

#[test]
fn ping_pong_call_ping_wrong_amount_go() {
    world().run("scenarios/ping-pong-call-ping-wrong-amount.scen.json");
}

#[test]
fn ping_pong_call_pong_go() {
    world().run("scenarios/ping-pong-call-pong.scen.json");
}

#[test]
fn ping_pong_call_pong_before_deadline_go() {
    world().run("scenarios/ping-pong-call-pong-before-deadline.scen.json");
}

#[test]
fn ping_pong_call_pong_twice_go() {
    world().run("scenarios/ping-pong-call-pong-twice.scen.json");
}

#[test]
fn ping_pong_call_pong_without_ping_go() {
    world().run("scenarios/ping-pong-call-pong-without-ping.scen.json");
}

#[test]
fn ping_pong_init_go() {
    world().run("scenarios/ping-pong-init.scen.json");
}
