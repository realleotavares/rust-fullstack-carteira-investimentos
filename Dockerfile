# Build Stage
FROM rust:slim-bookworm AS builder
WORKDIR /app

# Copia os arquivos do projeto
COPY . .

# Variável de ambiente obrigatória para compilar o SQLx sem ter um banco local rodando na nuvem
ENV SQLX_OFFLINE=true

# Compila o projeto em modo otimizado
RUN cargo build --release -j 1

# Run Stage (Imagem final super leve)
FROM debian:bookworm-slim
WORKDIR /app

# Instala certificados SSL (necessário para conectar no Neon via HTTPS)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copia apenas o binário final gerado na etapa anterior
COPY --from=builder /app/target/release/wallet-live /app/wallet-live

# Opcional: Define que a aplicação vai rodar na porta 3000
EXPOSE 3000

# Executa a aplicação
CMD ["./wallet-live"]
