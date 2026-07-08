# Carteira de Investimentos - Rustful Stack

Aplicação Fullstack desenvolvida inteiramente em Rust para gerenciamento de uma carteira de investimentos pessoal. Construída como parte do desafio da DIO / Santander, evoluindo o projeto base com novas funcionalidades.

---

## O que o projeto faz

- **Cadastro e autenticação** de usuários com senhas criptografadas (Argon2) e sessão via JWT em cookie HTTP-only
- **API REST** para gerenciamento de ativos (criação, listagem, atualização) protegida por header de admin
- **Dashboard interativo** que exibe o portfólio do usuário com:
  - Total investido e resultado geral (lucro/prejuízo)
  - Cada ativo com cotação atual, quantidade total e resultado acumulado
  - Histórico expandível de cada compra realizada
- **Registro de compras** via modal nativo HTML `<dialog>`
- **Logout** com remoção do cookie de sessão

---

## Tecnologias Utilizadas

| Tecnologia | Finalidade |
|---|---|
| **Rust** (edition 2024) | Linguagem principal |
| **Axum 0.8** | Framework web HTTP |
| **SQLx 0.8** | Queries PostgreSQL com checagem em tempo de compilação |
| **PostgreSQL 18** | Banco de dados (via Docker) |
| **Askama 0.15** | Templates HTML server-side |
| **JWT Simple** | Tokens JWT para autenticação stateless |
| **password-auth** | Hash/verificação de senhas (Argon2) |
| **axum-extra** | Gestão de cookies |
| **Tokio** | Runtime assíncrono |
| **Tailwind CSS** (CDN) | Estilização das páginas |

---

## Como Executar

### Pré-requisitos

- [Rust](https://rustup.rs/) (stable)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli): `cargo install sqlx-cli --no-default-features --features postgres`

### Passo a passo

```bash
# 1. Clone o repositório
git clone https://github.com/realleotavares/rust-fullstack-carteira-investimentos.git
cd rust-fullstack-carteira-investimentos

# 2. Suba o banco de dados PostgreSQL
docker compose up -d

# 3. Execute as migrações
sqlx migrate run

# 4. Inicie a aplicação
cargo run
```

A aplicação estará disponível em **http://localhost:3000**

### Variáveis de Ambiente

O arquivo `.env` já está configurado com os valores padrão para desenvolvimento local:

```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
```

---

## Melhorias Implementadas

Em relação ao projeto base das aulas, foram implementadas as seguintes evoluções:

### 1. Dashboard de Portfólio Completo (Aula 6)
O projeto base encerrava com a tela de login e um "Hello, {username}". Foi implementado o dashboard completo:
- **Cards de sumário** com total de posições, total investido e resultado geral
- **Lista de ativos** com expansão via `<details>` HTML nativo (zero JavaScript)
- **Indicadores visuais** verde/vermelho para lucro/prejuízo em cada ativo e compra

### 2. Histórico de Compras por Ativo
Implementação da tabela `owned_assets` com toda a estrutura de persistência:
- Migration SQL da tabela com chaves estrangeiras para `users` e `assets`
- Query com `GROUP BY` + `json_agg` para agregar histórico por ativo
- Cálculo de `value_delta` (resultado) por compra e por ativo total
- Formatação de data no PostgreSQL via `to_char()` para evitar complexidade de serde

### 3. Modal de Compra com `<dialog>` Nativo
Formulário para registrar novas posições usando o elemento `<dialog>` HTML5 - sem dependências JavaScript externas.

### 4. Logout e Navegação
- Rota `/logout` que remove o cookie de sessão
- Navbar fixa com nome do usuário e botão de saída
- Rota `/` que redireciona inteligentemente para `/assets` ou `/login`

### 5. Filtros Askama Customizados
Módulo `filters` em `frontend.rs` com:
- `|currency` - formata f64 com 2 casas decimais
- `|abs_val` - valor absoluto para exibição de prejuízos
- `|qty_fmt` - formatação inteligente de quantidades (remove zeros desnecessários)

### 6. Paralelismo nas Queries
O handler do dashboard usa `tokio::try_join!` para buscar owned_assets e available_assets simultaneamente, reduzindo latência.

---

## Como Testar

### Testes Automatizados (API)

Os testes de integração usam `sqlx::test` (banco isolado por teste) e snapshots `insta`:

```bash
cargo test
```

Para aprovar novos snapshots:

```bash
cargo insta review
```

### Testes Manuais

1. Acesse http://localhost:3000 - você será redirecionado para `/login`
2. Digite qualquer username/password - será criada uma conta automaticamente no primeiro acesso
3. Faça login novamente com as mesmas credenciais para autenticar
4. Cadastre ativos via API (requer header de admin):
   ```bash
   curl -X POST http://localhost:3000/api/assets \
     -H "Authorization: im-the-admin" \
     -H "Content-Type: application/json" \
     -d '{"name": "Bitcoin", "unit_value": 350000.00}'
   ```
5. No dashboard, clique em **+ registrar compra** e registre uma posição
6. Veja o resultado com lucro/prejuízo e histórico expandível

---

## O que aprendi durante o desafio

- Como estruturar um projeto Rust fullstack com separação clara de responsabilidades (Repository, Auth, Routes)
- O padrão de **extractors** do Axum para injeção de dependência sem frameworks adicionais
- Como o `sqlx::query_as!` valida SQL em tempo de compilação - e por que isso é um superpoder
- A potência de `json_agg` + `json_build_object` no PostgreSQL para evitar múltiplas queries
- Como templates Askama compilam para Rust puro, tornando os erros de template erros de compilação
- A diferença entre autenticação stateful (sessão no servidor) e stateless (JWT em cookie)
- Como `tokio::try_join!` permite paralelizar operações async sem complexidade adicional

---

## Estrutura do Projeto

```
src/
├── main.rs              # Entry point
├── app.rs               # AppState + inicialização do servidor
├── error.rs             # Enum de erros + conversão para HTTP response
├── models.rs            # Structs: Asset, UserRecord, OwnedAsset, PurchaseHistory
├── repository.rs        # Queries SQLx + implementação de FromRequestParts
├── auth/
│   ├── admin.rs         # Autenticação de admin via header Authorization
│   └── user.rs          # JWT, cookies, User extractor
└── routes/
    ├── api.rs           # GET/POST/PATCH /api/assets + testes
    └── frontend.rs      # Páginas HTML + filtros Askama customizados

templates/
├── login.html           # Tela de login/registro
└── assets/
    └── index.html       # Dashboard da carteira

migrations/
├── *_create_assets.*    # Tabela de ativos disponíveis
├── *_create_users.*     # Tabela de usuários
└── *_create_owned_assets.* # Tabela de posições do portfólio
```
