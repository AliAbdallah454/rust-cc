FROM rust

RUN apt update

RUN mkdir /app

WORKDIR /app

COPY Cargo.toml Cargo.lock Rocket.toml ./
# RUN mkdir src && echo "fn main() {}" > src/main.rs
# RUN cargo build --release

COPY ./src ./src
# RUN cargo build --release

EXPOSE 8000

CMD ["cargo", "run"]