use near_sdk::json_types::U128;
use near_sdk::{env, AccountId, Balance, near_bindgen, PanicOnDefault, BorshStorageKey, Promise, PromiseOrValue};
use near_sdk::collections::LookupMap;
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
use near_sdk::serde::{Deserialize, Serialize};

use crate::util::*;
use crate::payment::*;

mod util;
mod payment;

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
    pub payment_fee: u128,
    pub payments: LookupMap<u128, UpgradePayment>
}

#[near_bindgen]
impl PaymentShop {
    #[init]
    pub fn new(owner_id: AccountId, payment_fee: U128) -> Self {
        PaymentShop {
            owner_id,
            pay_id: 0,
            payment_fee: payment_fee.0,
            payments: LookupMap::new(StorageKey::PayIdKey)
        }
    }

    #[payable]
    pub fn req_payment(&mut self, user_id: AccountId, data: String, fee: Balance) {
        assert_at_least_one_yocto();
        let shop_id = env::predecessor_account_id();
        self.pay_id += 1;

        let storage_use_before = env::storage_usage();
        let payment = Payment {
            payment_id: self.pay_id,
            shop: shop_id,
            user: user_id,
            data: data,
            fee: fee,
            status: Status::REQUESTING
        }; 

        self.payments.insert(&self.pay_id, &UpgradePayment::from(payment));

        let storage_use_after = env::storage_usage();
        refund_deposit(storage_use_after - storage_use_before);

        let log_message = format!("Request payment: payment_id: {}, account: {}, fee: {}, data: {}", self.pay_id, payment.user, fee, payment.data);
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn pay(&mut self, pay_id: U128) {
        let fee = env::attached_deposit();
        let account_id = env::predecessor_account_id();

        let upgrade_payment = self.payments.get(&pay_id.0);
        assert!(upgrade_payment.is_some(), "ERR_PAYMENT_NOT_FOUND");
        let mut payment = Payment::from(upgrade_payment.unwrap());
        assert!(payment.status == Status::REQUESTING, "Invalid status");
        assert!(fee >= payment.fee, "Required FEE deposit of at least {} yoctoNEAR", payment.fee);
        assert_eq!(account_id, payment.user, "Deny access");

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

        assert_eq!(account_id, payment.user, "Deny access");

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

        assert_eq!(account_id, payment.shop, "Deny access");


        let payment_fee_amount = payment.fee * self.payment_fee / DECIMALS as u128;
        let payment_recever = payment.fee - payment_fee_amount;

        Promise::new(payment.shop).transfer(payment_recever);

        payment.status = Status::CLAIMED;
        
        self.payments.insert(&self.pay_id, &UpgradePayment::from(payment));

        let log_message = format!("Shop claim: payment_id: {}, amount {}", self.pay_id, payment_recever);
        env::log(log_message.as_bytes());
    }

    #[payable]
    pub fn withdraw(&mut self) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        assert_eq!(account_id, self.owner_id, "Not admin or owner");
    }
}