use near_sdk::{serde_json::json, json_types::U128};
use near_sdk_sim::{init_simulator, UserAccount, DEFAULT_GAS, STORAGE_AMOUNT, to_yocto};
use payment_shop_contract_tutorial::AccountJson;
use near_sdk_sim::transaction::{ExecutionStatus};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes!{
    PAYMENT_SHOP_CONTRACT_WASM_FILE => "out/payment-shop.wasm",
}

const PAYMENT_SHOP_CONTRACT_ID: &str = "payment_shop_contract";
const STAKING_FT_AMOUNT: &str = "50000000000000000000000000000";
const ALICE_DEPOSIT_AMOUNT: &str = "10000000000000000000000000000";
const FEE_CONTRACT_PERCENT: &str = "20000"; // 20%
const BOD_FEE_AMOUNT: &str = "100000000000000000000000000"; // 10 NEAR

pub fn init() -> (UserAccount, UserAccount, UserAccount, UserAccount) {
    let root = init_simulator(None);
    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    let bod = root.create_user("bod".to_string(), to_yocto("100"));

    // Deploy and init
    let payment_shop_contract = root.deploy_and_init(
        &PAYMENT_SHOP_CONTRACT_WASM_FILE,
        PAYMENT_SHOP_CONTRACT_ID.to_string(),
        "new", 
        &json!({
            "owner_id": alice.account_id(),
            "payment_fee": FEE_CONTRACT_PERCENT
        }).to_string().as_bytes(), 
        STORAGE_AMOUNT,
        DEFAULT_GAS
    );

    (root, alice, bod, payment_shop_contract)
}

#[test]
pub fn test_req_payment() {
    let (root, alice, bod, payment_shop_contract) = init();

    alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": BOD_FEE_AMOUNT
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    let payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, "1");
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, alice.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(100000000000000000000000000));
    assert_eq!(payment_json.status, "REQUESTING");
}

#[test]
pub fn test_pay() {
    let (root, alice, bod, payment_shop_contract) = init();

    alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": BOD_FEE_AMOUNT
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        BOD_FEE_AMOUNT 
    );

    let payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, "1");
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, alice.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(100000000000000000000000000));
    assert_eq!(payment_json.status, "PAID");
}

#[test]
pub fn test_confirm() {
    let (root, alice, bod, payment_shop_contract) = init();

    alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": BOD_FEE_AMOUNT
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01")
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "confirm", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        BOD_FEE_AMOUNT 
    );

    let payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, "1");
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, alice.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(100000000000000000000000000));
    assert_eq!(payment_json.status, "CONFIRMED");
}

#[test]
pub fn test_claim() {
    let (root, alice, bod, payment_shop_contract) = init();

    alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": BOD_FEE_AMOUNT
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01")
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "confirm", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        BOD_FEE_AMOUNT 
    );

    alice.call(
        payment_shop_contract.account_id(), 
        "claim", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    let payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": "1"
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, "1");
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, alice.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(100000000000000000000000000));
    assert_eq!(payment_json.status, "CLAIMED");

    let payment_shop_json: PaymentShopJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_shop_info", 
        &json!({}).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_shop_json.pay_id, "1");
    assert_eq!(payment_shop_json.payment_fee, U128(20000));
    assert_eq!(payment_json.total_payment, U128(100000000000000000000000000 * 20000 / 100000));

}