use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
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
