PEM_FILE="./ping-pong.pem"
PING_PONG_CONTRACT="output/ping-pong.wasm"

PROXY_ARGUMENT="--proxy=https://devnet-api.elrond.com"
CHAIN_ARGUMENT="--chain=D"

build_ping_pong() {
    (set -x; erdpy --verbose contract build "$PING_PONG_CONTRACT")
}

deploy_ping_pong() {
    # local TOKEN_ID=0x45474c44 # "EGLD"
    local PING_AMOUNT=1500000000000000000 # 1.5 EGLD
    local DURATION=86400 # 1 day in seconds
    # local ACTIVATION_TIMESTAMP= # skipped
    # local MAX_FUNDS= #skipped
    
    local OUTFILE="out.json"
    (set -x; erdpy contract deploy --bytecode="$PING_PONG_CONTRACT" \
        --pem="$PEM_FILE" \
        $PROXY_ARGUMENT $CHAIN_ARGUMENT \
        --outfile="$OUTFILE" --recall-nonce --gas-limit=60000000 \
        --arguments ${PING_AMOUNT} ${DURATION} --send \
        || return)

    local RESULT_ADDRESS=$(erdpy data parse --file="$OUTFILE" --expression="data['emitted_tx']['address']")
    local RESULT_TRANSACTION=$(erdpy data parse --file="$OUTFILE" --expression="data['emitted_tx']['hash']")

    echo ""
    echo "Deployed contract with:"
    echo "  \$RESULT_ADDRESS == ${RESULT_ADDRESS}"
    echo "  \$RESULT_TRANSACTION == ${RESULT_TRANSACTION}"
    echo ""
}

number_to_u64() {
    local NUMBER=$1
    printf "%016x" $NUMBER
}
