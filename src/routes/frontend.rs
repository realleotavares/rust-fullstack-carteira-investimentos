use askama::Template;
use axum::{
    Form, Router,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

use crate::{
    app::AppState,
    auth::user::{UnauthenticatedUser, User},
    error::AppError,
    models::{Asset, OwnedAsset},
    repository::Repository,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login_page).post(login))
        .route("/logout", get(logout))
        .route("/assets", get(assets_page))
        .route("/assets/purchase", axum::routing::post(purchase_asset))
}

// ─── Templates ───────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPage;

#[derive(Template)]
#[template(path = "assets/index.html")]
struct AssetsPage {
    user_name: String,
    owned_assets: Vec<OwnedAsset>,
    available_assets: Vec<Asset>,
    total_invested: f64,
    total_delta: f64,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn login_page() -> Result<Html<String>, AppError> {
    let html = LoginPage.render()?;
    Ok(Html(html))
}

#[derive(Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

async fn login(
    repository: Repository,
    jar: CookieJar,
    Form(request): Form<LoginForm>,
) -> Result<impl IntoResponse, AppError> {
    let unauth_user = UnauthenticatedUser::new(request.username, request.password);
    let user = match unauth_user.authenticate(&repository).await {
        Ok(user) => user,
        Err(AppError::UserDoesNotExist) => unauth_user.register(&repository).await?,
        Err(other_err) => return Err(other_err),
    };

    let token = user.auth_token()?;
    let cookie = Cookie::build(("token", token)).http_only(true);

    Ok((jar.add(cookie), Redirect::to("/assets")))
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar.remove(Cookie::from("token"));
    (jar, Redirect::to("/login"))
}

/// Rota raiz: redireciona para /assets se logado, senão para /login.
async fn index(maybe_user: Option<User>) -> Response {
    match maybe_user {
        Some(_) => Redirect::to("/assets").into_response(),
        None => Redirect::to("/login").into_response(),
    }
}

/// Dashboard principal: exibe o portfólio do usuário.
async fn assets_page(
    user: User,
    repository: Repository,
) -> Result<Html<String>, AppError> {
    let (owned_assets, available_assets) = tokio::try_join!(
        repository.list_owned_assets(user.id()),
        repository.list_assets(),
    )?;

    // Calcula totais para o sumário do portfólio
    let total_invested: f64 = owned_assets
        .iter()
        .map(|a| {
            a.purchase_history
                .0
                .iter()
                .map(|p| p.bought_for * p.quantity)
                .sum::<f64>()
        })
        .sum();
    let total_delta: f64 = owned_assets.iter().map(|a| a.value_delta).sum();

    let html = AssetsPage {
        user_name: user.username().clone(),
        owned_assets,
        available_assets,
        total_invested,
        total_delta,
    }
    .render()?;

    Ok(Html(html))
}

/// Registra uma nova compra de ativo.
#[derive(Deserialize)]
struct PurchaseForm {
    asset_id: i64,
    quantity: f64,
    bought_for: f64,
}

async fn purchase_asset(
    user: User,
    repository: Repository,
    Form(request): Form<PurchaseForm>,
) -> Result<impl IntoResponse, AppError> {
    repository
        .add_owned_asset(user.id(), request.asset_id, request.quantity, request.bought_for)
        .await?;

    Ok(Redirect::to("/assets"))
}

// ─── Askama Filters ──────────────────────────────────────────────────────────
// Os filtros abaixo ficam disponíveis nos templates via sintaxe |nome_do_filtro

pub mod filters {
    use askama::Result;

    /// Formata um f64 com 2 casas decimais. Ex: {{ value|currency }}
    pub fn currency(value: &f64) -> Result<String> {
        Ok(format!("{:.2}", value))
    }

    /// Valor absoluto de f64. Ex: {{ value|abs_val|currency }}
    pub fn abs_val(value: &f64) -> Result<f64> {
        Ok(value.abs())
    }

    /// Formata a quantidade de unidades (até 4 casas). Ex: {{ qty|qty_fmt }}
    pub fn qty_fmt(value: &f64) -> Result<String> {
        if value.fract() == 0.0 {
            Ok(format!("{:.0}", value))
        } else {
            Ok(format!("{:.4}", value).trim_end_matches('0').to_string())
        }
    }
}
