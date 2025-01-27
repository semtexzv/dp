use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    #[serde(rename = "type")]
    pub typ: String,
    pub currency: String,
    #[serde(deserialize_with = "f64_from_str")]
    pub amount: f64,
    #[serde(deserialize_with = "f64_from_str")]
    pub available: f64,
}

#[derive(Debug, Clone, )]
pub struct NewOrderPayload {
    pub symbol: TradePair,
    pub amount: f64,
    pub buy: bool,
}

impl Serialize for NewOrderPayload {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error> where
        S: Serializer {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct RawPayload {
            symbol: String,
            amount: String,
            price: String,
            exchange: String,
            side: String,
            #[serde(rename = "type")]
            typ: String,
        }

        let mut p = RawPayload {
            symbol: self.symbol.to_bfx_pair(),
            amount: f64::abs(self.amount).to_string(),
            price: 1.to_string(),
            exchange: "bitfinex".into(),
            side: (if self.buy { "buy" } else { "sell" }).to_string(),
            typ: "exchange market".into(),
        };
        Serialize::serialize(&p, serializer)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDetail {
    pub pair: String,
    pub price_precision: usize,
    #[serde(deserialize_with = "f64_from_str")]
    pub minimum_order_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub id: usize,

    #[serde(rename = "symbol")]
    #[serde(deserialize_with = "tradepair_from_bfx")]
    pub pair: TradePair,
    pub exchange: String,

    #[serde(deserialize_with = "f64_from_str")]
    pub price: f64,

    pub side: String,
    #[serde(rename = "type")]
    pub typ: String,

    pub is_live: bool,
    pub is_cancelled: bool,
    pub is_hidden: bool,

    #[serde(deserialize_with = "f64_from_str")]
    pub original_amount: f64,

    #[serde(deserialize_with = "f64_from_str")]
    pub remaining_amount: f64,

    #[serde(deserialize_with = "f64_from_str")]
    pub executed_amount: f64,

}