# ── Stage 1: зависимости (кешируется отдельно) ───────────────────────────────
FROM rust:1.82-alpine AS deps

RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /build

# Копируем только манифесты — Docker закеширует слой с зависимостями
# и не будет перекачивать их при каждом изменении кода.
COPY Cargo.toml Cargo.lock ./

# Создаём заглушку чтобы cargo мог скачать зависимости
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo fetch --locked

# ── Stage 2: сборка ───────────────────────────────────────────────────────────
FROM deps AS builder

# Теперь копируем настоящий код
COPY src ./src

# Release build: lto=thin, strip, panic=abort (настроены в Cargo.toml)
RUN cargo build --release --locked

# ── Stage 3: минимальный runtime-образ ───────────────────────────────────────
# distroless/cc-debian12 даёт нам libc и CA-certs без лишнего (< 20 MB)
FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

COPY --from=builder /build/target/release/versionsbot .

# Бот работает через long-polling — порты не нужны
ENV RUST_LOG=info

CMD ["./versionsbot"]
