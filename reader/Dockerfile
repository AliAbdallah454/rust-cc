FROM rust

RUN apt update

RUN mkdir /app

WORKDIR /app

VOLUME [ "/database", "/database2" ]

EXPOSE 9000

COPY Cargo.toml Cargo.lock Rocket.toml ./
COPY src ./src

CMD ["cargo", "run"]