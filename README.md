# Carteira de Investimentos - Rustful Stack

Aplicacao Fullstack desenvolvida inteiramente em Rust para gerenciamento de uma carteira de investimentos pessoal. Construida como parte do desafio da [DIO](https://www.dio.me), evoluindo o projeto base com novas funcionalidades.

---

## O que o projeto faz

- **Cadastro e autenticacao** de usuarios com senhas criptografadas (Argon2) e sessao via JWT em cookie HTTP-only
- **API REST** para gerenciamento de ativos (criacao, listagem, atualizacao) protegida por header de admin
- **Dashboard interativo** que exibe o portfolio do usuario com:
  - Total investido e resultado geral (lucro/prejuizo)
  - Cada ativo com cotacao atual, quantidade total e resultado acumulado
  - Historico expandivel de cada compra realizada
- **Registro de compras** via modal nativo HTML `<dialog>`
- **Logout** com remocao do cookie de sessao

---

## Tecnologias Utilizadas

| Tecnologia | Finalidade |
|---|---|
| **Rust** (edition 2024) | Linguagem principal |
| **Axum 0.8** | Framework web HTTP |
| **SQLx 0.8** | Queries PostgreSQL com checagem em tempo de compilacao |
| **PostgreSQL 18** | Banco de dados (via Docker) |
| **Askama 0.15** | Templates HTML server-side |
| **JWT Simple** | Tokens JWT para autenticacao stateless |
| **password-auth** | Hash/verificacao de senhas (Argon2) |
| **axum-extra** | Gestao de cookies |
| **Tokio** | Runtime assincrono |
| **Tailwind CSS** (CDN) | Estilizacao das paginas |

---

## Como Executar

### Pre-requisitos

- [Rust](https://rustup.rs/) (stable)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli): `cargo install sqlx-cli --no-default-features --features postgres`

### Passo a passo

```bash
# 1. Clone o repositorio
git clone https://github.com/realleotavares/rust-fullstack-carteira-investimentos.git
cd rust-fullstack-carteira-investimentos

# 2. Suba o banco de dados PostgreSQL
docker compose up -d

# 3. Execute as migracoes
sqlx migrate run

# 4. Inicie a aplicacao
cargo run
```

A aplicacao estara disponivel em **http://localhost:3000**

### Variaveis de Ambiente

O arquivo `.env` ja esta configurado com os valores padrao para desenvolvimento local:

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
```

---

## Melhorias Implementadas

Em relacao ao projeto base das aulas, foram implementadas as seguintes evolucoes:

### 1. Dashboard de Portfolio Completo (Aula 6)
O projeto base encerrava com a tela de login e um "Hello, {username}". Foi implementado o dashboard completo:
- **Cards de sumario** com total de posicoes, total investido e resultado geral
- **Lista de ativos** com expansao via `<details>` HTML nativo (zero JavaScript)
- **Indicadores visuais** verde/vermelho para lucro/prejuizo em cada ativo e compra

### 2. Historico de Compras por Ativo
Implementacao da tabela `owned_assets` com toda a estrutura de persistencia:
- Migration SQL da tabela com chaves estrangeiras para `users` e `assets`
- Query com `GROUP BY` + `json_agg` para agregar historico por ativo
- Calculo de `value_delta` (resultado) por compra e por ativo total
- Formatacao de data no PostgreSQL via `to_char()` para evitar complexidade de serde

### 3. Modal de Compra com `<dialog>` Nativo
Formulario para registrar novas posicoes usando o elemento `<dialog>` HTML5 - sem dependencias JavaScript externas.

### 4. Logout e Navegacao
- Rota `/logout` que remove o cookie de sessao
- Navbar fixa com nome do usuario e botao de saida
- Rota `/` que redireciona inteligentemente para `/assets` ou `/login`

### 5. Filtros Askama Customizados
Modulo `filters` em `frontend.rs` com:
- `|currency` - formata f64 com 2 casas decimais
- `|abs_val` - valor absoluto para exibicao de prejuizos
- `|qty_fmt` - formatacao inteligente de quantidades (remove zeros desnecessarios)

### 6. Paralelismo nas Queries
O handler do dashboard usa `tokio::try_join!` para buscar owned_assets e available_assets simultaneamente, reduzindo latencia.

---

## Como Testar

### Testes Automatizados (API)

Os testes de integracao usam `sqlx::test` (banco isolado por teste) e snapshots `insta`:

```bash
cargo test
```

Para aprovar novos snapshots:

```bash
cargo insta review
```

### Testes Manuais

1. Acesse http://localhost:3000 - voce sera redirecionado para `/login`
2. Digite qualquer username/password - sera criada uma conta automaticamente no primeiro acesso
3. Faca login novamente com as mesmas credenciais para autenticar
4. Cadastre ativos via API (requer header de admin):
   ```bash
   curl -X POST http://localhost:3000/api/assets \
     -H "Authorization: im-the-admin" \
     -H "Content-Type: application/json" \
     -d '{"name": "Bitcoin", "unit_value": 350000.00}'
   ```
5. No dashboard, clique em **+ registrar compra** e registre uma posicao
6. Veja o resultado com lucro/prejuizo e historico expandivel

---

## O que aprendi durante o desafio

- Como estruturar um projeto Rust fullstack com separacao clara de responsabilidades (Repository, Auth, Routes)
- O padrao de **extractors** do Axum para injecao de dependencia sem frameworks adicionais
- Como o `sqlx::query_as!` valida SQL em tempo de compilacao - e por que isso e um superpoder
- A potencia de `json_agg` + `json_build_object` no PostgreSQL para evitar multiplas queries
- Como templates Askama compilam para Rust puro, tornando os erros de template erros de compilacao
- A diferenca entre autenticacao stateful (sessao no servidor) e stateless (JWT em cookie)
- Como `tokio::try_join!` permite paralelizar operacoes async sem complexidade adicional

---

## Estrutura do Projeto

```
src/
├── main.rs              # Entry point
├── app.rs               # AppState + inicializacao do servidor
├── error.rs             # Enum de erros + conversao para HTTP response
├── models.rs            # Structs: Asset, UserRecord, OwnedAsset, PurchaseHistory
├── repository.rs        # Queries SQLx + implementacao de FromRequestParts
├── auth/
│   ├── admin.rs         # Autenticacao de admin via header Authorization
│   └── user.rs          # JWT, cookies, User extractor
└── routes/
    ├── api.rs           # GET/POST/PATCH /api/assets + testes
    └── frontend.rs      # Paginas HTML + filtros Askama customizados

templates/
├── login.html           # Tela de login/registro
└── assets/
    └── index.html       # Dashboard da carteira

migrations/
├── *_create_assets.*    # Tabela de ativos disponiveis
├── *_create_users.*     # Tabela de usuarios
└── *_create_owned_assets.* # Tabela de posicoes do portfolio
```
