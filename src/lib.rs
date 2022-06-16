use near_sdk::json_types::U128;
use near_sdk::{env, AccountId, Balance, near_bindgen, PanicOnDefault, BorshStorageKey, Promise};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::util::*;
use crate::payment::*;
pub use crate::enumeration::*;
pub use crate::payment::PaymentJson;
pub use crate::payment::Status;

mod util;
mod payment;
mod enumeration;

const DECIMALS: u32 = 100000;

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    PayIdKey
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct PaymentShop {
    pub owner_id: AccountId,
    pub pay_id: u128,
    pub payment_fee_percent: u128,
    pub total_payment: u128,
    pub total_payment_withdraw: u128,
    pub payments: LookupMap<u128, UpgradePayment>,
    pub order_ids: UnorderedMap<u128, u128>
}

#[near_bindgen]
impl PaymentShop {
    #[init]
    pub fn new(owner_id: AccountId, payment_fee_percent: U128) -> Self {
        PaymentShop {
            owner_id,
            pay_id: 0,
            payment_fee_percent: payment_fee_percent.0,
            total_payment: 0,
            total_payment_withdraw: 0,
            payments: LookupMap::new(StorageKey::PayIdKey),
            order_ids: UnorderedMap::new(b"m")
        }
    }

    #[payable]
    pub fn req_payment(&mut self, order_id: U128, user_id: AccountId, msg: String, fee: U128) {
        let pay_id_for_order = self.order_ids.get(&order_id.0);
        assert!(pay_id_for_order.is_none(), "Order ID is set");
        assert_at_least_one_yocto();
        let shop_id = env::predecessor_account_id();
        let pay_id = self.pay_id + 1;
     
        let storage_use_before = env::storage_usage();
        let payment = Payment {
            payment_id: pay_id,
            order_id: order_id.0,
            shop: shop_id,
            user: user_id,
            msg: msg,
            fee: fee.0,
            status: Status::REQUESTING
        }; 

        let log_message = format!("Request payment: payment_id: {}, order_id: {}, account: {}, fee: {}, data: {}", &pay_id, &order_id.0, payment.user, payment.fee, payment.msg);
        self.payments.insert(&pay_id, &UpgradePayment::from(payment));
        self.order_ids.insert(&order_id.0, &pay_id);

        let storage_use_after = env::storage_usage();
        refund_deposit(storage_use_after - storage_use_before);
        self.pay_id = pay_id;
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn pay(&mut self, pay_id: U128) {
        assert_at_least_one_yocto();

        let fee = env::attached_deposit();
        let account_id = env::predecessor_account_id();

        let upgrade_payment = self.payments.get(&pay_id.0);
        assert!(upgrade_payment.is_some(), "ERR_PAYMENT_NOT_FOUND");

        let mut payment = Payment::from(upgrade_payment.unwrap());

        assert!(payment.status == Status::REQUESTING, "Invalid status");
        assert!(fee >= payment.fee, "Required FEE deposit of at least {} yoctoNEAR", payment.fee);
        assert_eq!(account_id, payment.user, "Access deny");

        payment.status = Status::PAID;
        self.payments.insert(&self.pay_id, &UpgradePayment::from(payment));

        let log_message = format!("Pay: payment_id: {}", self.pay_id);
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn confirm(&mut self, pay_id: U128) { 
        assert_one_yocto();
        let account_id = env::predecessor_account_id();

        let upgrade_payment = self.payments.get(&pay_id.0);
        assert!(upgrade_payment.is_some(), "ERR_PAYMENT_NOT_FOUND");

        let mut payment = Payment::from(upgrade_payment.unwrap());
        assert!(payment.status == Status::PAID, "Invalid status");

        assert!(account_id == payment.user || account_id == self.owner_id, "Access deny");

        payment.status = Status::CONFIRMED;
        self.payments.insert(&self.pay_id, &UpgradePayment::from(payment));

        let log_message = format!("Confirm: payment_id: {}", self.pay_id);
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn claim(&mut self, pay_id: U128) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        
        let upgrade_payment = self.payments.get(&pay_id.0);
        assert!(upgrade_payment.is_some(), "ERR_PAYMENT_NOT_FOUND");

        let mut payment = Payment::from(upgrade_payment.unwrap());
        assert!(payment.status == Status::CONFIRMED, "Invalid status");

        assert_eq!(account_id, payment.shop, "Access deny");

        let payment_fee_amount = payment.fee * self.payment_fee_percent / (DECIMALS as u128);
        let payment_recever = payment.fee - payment_fee_amount;

        payment.status = Status::CLAIMED;
        let shop_id = payment.shop.clone();
        self.payments.insert(&self.pay_id, &UpgradePayment::from(payment));

        self.total_payment += payment_fee_amount;
        Promise::new(shop_id).transfer(payment_recever);

        let log_message = format!("Shop claim: payment_id: {}, amount {}", self.pay_id, payment_recever);
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn withdraw(&mut self) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        assert_eq!(account_id, self.owner_id, "Not admin or owner");
        assert!(self.total_payment > 0, "No amount to withdraw");

        let payment_withdraw = self.total_payment - self.total_payment_withdraw;
        self.total_payment_withdraw = self.total_payment;
        Promise::new(account_id).transfer(payment_withdraw);

        self.total_payment_withdraw = payment_withdraw.clone();

        let log_message = format!("Withdraw: amount {}", payment_withdraw);
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn set_payment_fee(&mut self, payment_fee_percent: U128) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        assert_eq!(account_id, self.owner_id, "Not admin or owner");
        assert!(payment_fee_percent.0 > 0, "Invalid payment fee");

        self.payment_fee_percent = payment_fee_percent.0;

        let log_message = format!("Set payment fee: {}", payment_fee_percent.0);
        env::log(log_message.as_bytes());
    }
}