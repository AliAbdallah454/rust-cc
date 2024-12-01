FROM rust

RUN apt update

RUN mkdir /app

WORKDIR /app
COPY Cargo.toml Cargo.lock Rocket.toml ./
COPY src ./src

COPY my-consistent-hashing ./my-consistent-hashing

EXPOSE 8000

CMD ["cargo", "run"]