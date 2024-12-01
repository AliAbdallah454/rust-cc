FROM rust

RUN apt update

RUN mkdir /app

WORKDIR /app

EXPOSE 8000

COPY Cargo.toml Cargo.lock Rocket.toml ./
COPY src ./src

CMD ["cargo", "run"]