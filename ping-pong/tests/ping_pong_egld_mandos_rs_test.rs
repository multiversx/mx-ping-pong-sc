use elrond_wasm_debug::*;

fn world() -> BlockchainMock {
    let mut blockchain = BlockchainMock::new();
    blockchain.set_current_dir_from_workspace("");

    blockchain.register_contract(
        "file:output/ping-pong.wasm",
        Box::new(|context| Box::new(ping_pong::contract_obj(context))),
    );
    blockchain
}

#[test]
fn ping_pong_call_ping_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-call-ping.scen.json", world());
}

#[test]
fn ping_pong_call_ping_second_user_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-call-ping-second-user.scen.json", world());
}

#[test]
fn ping_pong_call_ping_twice_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-call-ping-twice.scen.json", world());
}

#[test]
fn ping_pong_call_ping_wrong_amount_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-call-ping-wrong-amount.scen.json", world());
}

#[test]
fn ping_pong_call_pong_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-call-pong.scen.json", world());
}

#[test]
fn ping_pong_call_pong_before_deadline_rs() {
    elrond_wasm_debug::mandos_rs(
        "mandos/ping-pong-call-pong-before-deadline.scen.json",
        world(),
    );
}

#[test]
fn ping_pong_call_pong_twice_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-call-pong-twice.scen.json", world());
}

#[test]
fn ping_pong_call_pong_without_ping_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-call-pong-without-ping.scen.json", world());
}

#[test]
fn ping_pong_init_rs() {
    elrond_wasm_debug::mandos_rs("mandos/ping-pong-init.scen.json", world());
}
