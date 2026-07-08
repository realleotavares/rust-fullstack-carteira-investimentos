use std::convert::Infallible;

use axum::extract::FromRequestParts;
use sqlx::PgPool;

use crate::{
    app::AppState,
    models::{Asset, OwnedAsset, UserRecord},
};

pub struct Repository {
    db: PgPool,
}

impl Repository {
    pub async fn list_assets(&self) -> sqlx::Result<Vec<Asset>> {
        sqlx::query_as!(
            Asset,
            "SELECT id, name, unit_value
             FROM assets;"
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn create_asset(&self, name: String, unit_value: f64) -> sqlx::Result<Asset> {
        sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name, unit_value)
             VALUES ($1, $2)
             RETURNING id, name, unit_value;",
            name,
            unit_value
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn update_asset(
        &self,
        asset_id: i64,
        name: Option<String>,
        unit_value: Option<f64>,
    ) -> sqlx::Result<Option<Asset>> {
        sqlx::query_as!(
            Asset,
            "UPDATE assets
             SET name=COALESCE($2, name),
                 unit_value=COALESCE($3, unit_value)
             WHERE id=$1
             RETURNING id, name, unit_value;",
            asset_id,
            name,
            unit_value
        )
        .fetch_optional(&self.db)
        .await
    }

    pub async fn add_user(&self, username: &str, password_hash: &str) -> sqlx::Result<UserRecord> {
        sqlx::query_as!(
            UserRecord,
            "INSERT INTO users (username, password_hash)
             VALUES ($1, $2)
             RETURNING id, username, password_hash;",
            username,
            password_hash,
        )
        .fetch_one(&self.db)
        .await
    }

    pub async fn get_user_by_name(&self, username: &str) -> sqlx::Result<Option<UserRecord>> {
        sqlx::query_as!(
            UserRecord,
            "SELECT id, username, password_hash
             FROM users
             WHERE username = $1;",
            username
        )
        .fetch_optional(&self.db)
        .await
    }

    /// Lista todos os ativos que o usuário possui, agrupados por ativo.
    /// Calcula delta total (lucro/prejuízo) e agrega o histórico de compras como JSON.
    pub async fn list_owned_assets(&self, user_id: i64) -> sqlx::Result<Vec<OwnedAsset>> {
        sqlx::query_as!(
            OwnedAsset,
            r#"
            SELECT
                a.id         AS asset_id,
                a.name       AS name,
                a.unit_value AS unit_value,
                SUM((a.unit_value - oa.bought_for) * oa.quantity) AS "value_delta!",
                SUM(oa.quantity)                                   AS "quantity!",
                json_agg(
                    json_build_object(
                        'bought_at',   to_char(oa.bought_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI'),
                        'bought_for',  oa.bought_for,
                        'quantity',    oa.quantity,
                        'value_delta', (a.unit_value - oa.bought_for) * oa.quantity
                    )
                    ORDER BY oa.bought_at DESC
                ) AS "purchase_history!"
            FROM assets a
            JOIN owned_assets oa ON oa.asset_id = a.id
            WHERE oa.user_id = $1
            GROUP BY a.id, a.name, a.unit_value
            ORDER BY a.name;
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await
    }

    /// Registra uma nova compra de ativo para o usuário.
    pub async fn add_owned_asset(
        &self,
        user_id: i64,
        asset_id: i64,
        quantity: f64,
        bought_for: f64,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO owned_assets (user_id, asset_id, quantity, bought_for)
             VALUES ($1, $2, $3, $4);",
            user_id,
            asset_id,
            quantity,
            bought_for
        )
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

impl FromRequestParts<AppState> for Repository {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self {
            db: state.db.clone(),
        })
    }
}

#[cfg(test)]
impl From<PgPool> for Repository {
    fn from(db: PgPool) -> Self {
        Self { db }
    }
}
