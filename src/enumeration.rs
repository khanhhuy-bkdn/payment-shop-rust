use crate::*;

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PaymentShopJson {
    pub pay_id: U128,
    pub payment_fee: U128,
    pub total_payment: U128
}

#[near_bindgen]
impl PaymentShop {
    pub fn get_payment_info(&self, pay_id: U128) -> PaymentJson {
        let upgradable_payment = self.payments.get(&pay_id.0).unwrap();

        let payment = Payment::from(upgradable_payment);
        PaymentJson::from(pay_id.0.clone(), payment)
    }

    pub fn get_payment_shop_info(&self) -> PaymentShopJson {

        PaymentShopJson {
            pay_id: U128(self.pay_id),
            payment_fee: U128(self.payment_fee),
            total_payment: U128(self.total_payment)
        }
    }
}