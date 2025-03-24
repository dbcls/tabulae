ARG RUST_VERSION=1.85
ARG APP_NAME=tabulae

FROM docker.io/library/rust:${RUST_VERSION}-bullseye AS build

RUN cargo new --bin app
WORKDIR /app

COPY ./Cargo.lock ./Cargo.toml .
RUN cargo build --locked --release

RUN curl -fsSL https://bun.sh/install | bash
ENV BUN_INSTALL="/root/.bun"
ENV PATH="$BUN_INSTALL/bin:$PATH"
RUN bun -version

RUN rm -r ./src
COPY ./src ./src
COPY ./build.rs ./
COPY ./frontend ./frontend
RUN touch ./src/main.rs
RUN cargo build --locked --release --features frontend

FROM docker.io/library/debian:bullseye-slim
ARG APP_NAME
COPY --from=build /app/target/release/${APP_NAME} /usr/local/bin

WORKDIR /work
ENTRYPOINT ["tabulae"]
