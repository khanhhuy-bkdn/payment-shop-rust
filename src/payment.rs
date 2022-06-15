use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Payment {
    pub payment_id: u128,
    pub shop: AccountId,
    pub user: AccountId,
    pub msg: String,
    pub fee: Balance,
    pub status: Status,
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    REQUESTING,
    PAID,
    CONFIRMED,
    CLAIMED,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum UpgradePayment {
    Current(Payment),
}

impl Default for Payment {
    fn default() -> Self {
        Payment {
            payment_id: 0,
            shop: "".to_string(),
            user: "".to_string(),
            msg: "".to_string(),
            fee: 0,
            status: Status::REQUESTING,
        }
    }
}

impl From<UpgradePayment> for Payment {
    fn from(upgradable_payment: UpgradePayment) -> Self {
        match upgradable_payment {
            UpgradePayment::Current(payment) => payment,
        }
    }
}

impl From<Payment> for UpgradePayment {
    fn from(payment: Payment) -> Self {
        UpgradePayment::Current(payment)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PaymentJson {
    pub payment_id: U128,
    pub shop: AccountId,
    pub user: AccountId,
    pub msg: String,
    pub fee: U128,
    pub status: Status,
}

impl PaymentJson {
    pub fn from(payment_id: u128, payment: Payment) -> Self {
        PaymentJson {
            payment_id: U128(payment_id),
            shop: payment.shop,
            user: payment.user,
            msg: payment.msg,
            fee: U128(payment.fee),
            status: payment.status,
        }
    }
}
