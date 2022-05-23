use crate::*;

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, Clone)]
#[serde(crate="near_sdk::serde")]
pub struct Payment {
    pub payment_id: u128,
    pub shop: AccountId,
    pub user: AccountId,
    pub data: String,
    pub fee: Balance,
    pub status: Status
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PartialEq, Clone, Copy)]
#[serde(crate="near_sdk::serde")]
pub enum Status {
    REQUESTING, 
    PAID, 
    CONFIRMED, 
    CLAIMED
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum UpgradePayment {
    Current(Payment)
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

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PaymentJson {
    pub payment_id: U128,
    pub shop: AccountId,
    pub user: AccountId,
    pub data: String,
    pub fee: U128,
    pub status: Status
}

impl PaymentJson {
    pub fn from(payment_id: u128, payment: Payment) -> Self {
        PaymentJson {
            payment_id: U128(payment_id),
            shop: payment.shop,
            user: payment.user,
            data: payment.data,
            fee: U128(payment.fee),
            status: payment.status
        }
    }
}