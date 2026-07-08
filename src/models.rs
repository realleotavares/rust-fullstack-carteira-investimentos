use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
}

impl Asset {
    pub fn unit_value_fmt(&self) -> String {
        format!("{:.2}", self.unit_value)
    }
}

pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

/// Histórico individual de uma compra de ativo.
/// `bought_at` é formatado pelo PostgreSQL como "YYYY-MM-DD HH24:MI" via to_char().
#[derive(Serialize, Deserialize, Clone)]
pub struct PurchaseHistory {
    pub bought_at: String,
    pub bought_for: f64,
    pub quantity: f64,
    pub value_delta: f64,
}

impl PurchaseHistory {
    pub fn bought_for_fmt(&self) -> String { format!("{:.2}", self.bought_for) }
    pub fn value_delta_fmt(&self) -> String { format!("{:.2}", self.value_delta.abs()) }
    pub fn is_positive(&self) -> bool { self.value_delta >= 0.0 }
    pub fn qty_fmt(&self) -> String {
        if self.quantity.fract() == 0.0 {
            format!("{:.0}", self.quantity)
        } else {
            format!("{:.4}", self.quantity).trim_end_matches('0').to_string()
        }
    }
}

/// Resumo do portfólio do usuário para um ativo específico.
/// Resultado de uma query com GROUP BY + json_agg.
#[derive(Serialize, Clone)]
pub struct OwnedAsset {
    pub asset_id: i64,
    pub name: String,
    pub unit_value: f64,
    pub value_delta: f64,
    pub quantity: f64,
    pub purchase_history: sqlx::types::Json<Vec<PurchaseHistory>>,
}

impl OwnedAsset {
    pub fn unit_value_fmt(&self) -> String { format!("{:.2}", self.unit_value) }
    pub fn value_delta_fmt(&self) -> String { format!("{:.2}", self.value_delta.abs()) }
    pub fn is_positive(&self) -> bool { self.value_delta >= 0.0 }
    pub fn qty_fmt(&self) -> String {
        if self.quantity.fract() == 0.0 {
            format!("{:.0}", self.quantity)
        } else {
            format!("{:.4}", self.quantity).trim_end_matches('0').to_string()
        }
    }
}
