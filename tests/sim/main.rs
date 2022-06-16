use near_sdk::{serde_json::json, json_types::U128};
use near_sdk_sim::{init_simulator, UserAccount, DEFAULT_GAS, STORAGE_AMOUNT, to_yocto};
use payment_shop_rust::{PaymentJson, PaymentShopJson, Status};
use near_sdk_sim::transaction::{ExecutionStatus};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes!{
    PAYMENT_SHOP_CONTRACT_WASM_FILE => "out/payment-shop-contract.wasm",
}

const PAYMENT_SHOP_CONTRACT_ID: &str = "payment_shop_contract";
const FEE_CONTRACT_PERCENT: &str = "20000"; // 20%
const BOD_FEE_AMOUNT: u128 = 10000000000000000000000000; // 10 NEAR

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
            "payment_fee_percent": FEE_CONTRACT_PERCENT
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
            "order_id":U128(1),
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": U128(BOD_FEE_AMOUNT)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    let order_id: U128 = root.view(
        payment_shop_contract.account_id(), 
        "get_payid_for_orderid", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(order_id, U128(1));

    let payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, U128(1));
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, bod.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(10000000000000000000000000));
    assert_eq!(payment_json.status, Status::REQUESTING);

    let outcome_req = alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "order_id":U128(1),
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": U128(BOD_FEE_AMOUNT)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome_req.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("Order ID is set"));
    } else {
        unreachable!()
    }
}

#[test]
pub fn test_pay() {
    let (root, alice, bod, payment_shop_contract) = init();

    alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "order_id":U128(1),
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": U128(BOD_FEE_AMOUNT)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        BOD_FEE_AMOUNT 
    );

    let payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, U128(1));
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, bod.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(10000000000000000000000000));
    assert_eq!(payment_json.status, Status::PAID);
}

#[test]
pub fn test_confirm() {
    let (root, alice, bod, payment_shop_contract) = init();

    alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "order_id":U128(1),
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": U128(BOD_FEE_AMOUNT)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    let mut outcome = alice.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01")
    );

    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("Required FEE deposit of at least 10000000000000000000000000 yoctoNEAR"));
    } else {
        unreachable!()
    }

    outcome = alice.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        BOD_FEE_AMOUNT
    );

    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("Access deny"));
    } else {
        unreachable!()
    }

    bod.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        BOD_FEE_AMOUNT
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "confirm", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1 
    );

    let payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, U128(1));
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, bod.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(10000000000000000000000000));
    assert_eq!(payment_json.status, Status::CONFIRMED);
}

#[test]
pub fn test_claim() {
    let (root, alice, bod, payment_shop_contract) = init();

    alice.call(
        payment_shop_contract.account_id(), 
        "req_payment", 
        &json!({
            "order_id":U128(1),
            "user_id": bod.account_id(),
            "msg": "Hello",
            "fee": U128(BOD_FEE_AMOUNT)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        to_yocto("0.01") 
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "pay", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        BOD_FEE_AMOUNT
    );

    bod.call(
        payment_shop_contract.account_id(), 
        "confirm", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1 
    );

    let mut outcome = bod.call(
        payment_shop_contract.account_id(), 
        "claim", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1
    );

    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("Access deny"));
    } else {
        unreachable!()
    }

    outcome = alice.call(
        payment_shop_contract.account_id(), 
        "claim", 
        &json!({
            "pay_id": U128(2)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1
    );

    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("ERR_PAYMENT_NOT_FOUND"));
    } else {
        unreachable!()
    }

    alice.call(
        payment_shop_contract.account_id(), 
        "claim", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1
    );

    let mut payment_json: PaymentJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": U128(1)
        }).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_json.payment_id, U128(1));
    assert_eq!(payment_json.shop, alice.account_id());
    assert_eq!(payment_json.user, bod.account_id());
    assert_eq!(payment_json.msg, "Hello");
    assert_eq!(payment_json.fee, U128(10000000000000000000000000));
    assert_eq!(payment_json.status, Status::CLAIMED);

    let payment_shop_json: PaymentShopJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_shop_info", 
        &json!({}).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_shop_json.pay_id, U128(1));
    assert_eq!(payment_shop_json.payment_fee_percent, U128(20000));
    assert_eq!(payment_shop_json.total_payment, U128(10000000000000000000000000 * 20000 / 100000));


    payment_json = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_info", 
        &json!({
            "pay_id": U128(10)
        }).to_string().as_bytes()
    ).unwrap_json();
    println!("payment_json {:#?}", payment_json)

}

#[test]
pub fn test_set_payment_fee() {
    let (root, alice, bod, payment_shop_contract) = init();

    let mut outcome = bod.call(
        payment_shop_contract.account_id(), 
        "set_payment_fee", 
        &json!({
            "payment_fee_percent": U128(30000)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1
    );

    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("Not admin or owner"));
    } else {
        unreachable!()
    }

    outcome = alice.call(
        payment_shop_contract.account_id(), 
        "set_payment_fee", 
        &json!({
            "payment_fee_percent": U128(0)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1
    );

    assert_eq!(outcome.promise_errors().len(), 1);

    // assert error type
    if let ExecutionStatus::Failure(error) = &outcome.promise_errors().remove(0).unwrap().outcome().status {
        println!("Excute error: {}", error.to_string());
        assert!(error.to_string().contains("Invalid payment fee"));
    } else {
        unreachable!()
    }

    alice.call(
        payment_shop_contract.account_id(), 
        "set_payment_fee", 
        &json!({
            "payment_fee_percent": U128(30000)
        }).to_string().as_bytes(), 
        DEFAULT_GAS,
        1
    );

    let payment_shop_json: PaymentShopJson = root.view(
        payment_shop_contract.account_id(), 
        "get_payment_shop_info", 
        &json!({}).to_string().as_bytes()
    ).unwrap_json();

    assert_eq!(payment_shop_json.pay_id, U128(0));
    assert_eq!(payment_shop_json.payment_fee_percent, U128(30000));

}