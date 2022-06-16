use crate::*;

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PaymentShopJson {
    pub pay_id: U128,
    pub payment_fee_percent: U128,
    pub total_payment: U128
}

#[near_bindgen]
impl PaymentShop {
    pub fn get_payment_info(&self, pay_id: U128) -> PaymentJson {
        let upgradable_payment = self.payments.get(&pay_id.0);
        let payment;
        match upgradable_payment {
            Some(item) => payment = Payment::from(item),
            None => payment = Payment::default()
        };
        PaymentJson::from(pay_id.0.clone(), payment)
    }

    pub fn get_payment_shop_info(&self) -> PaymentShopJson {

        PaymentShopJson {
            pay_id: U128(self.pay_id),
            payment_fee_percent: U128(self.payment_fee_percent),
            total_payment: U128(self.total_payment)
        }
    }

    pub fn get_payid_from_orderid(&self, order_id: U128) -> U128 {
        match self.order_ids.get(&order_id.0) {
            Some(value) => {
                U128(value)
            },
            None => U128(0)
        }
    }
}