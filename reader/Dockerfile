FROM rust

RUN apt update

RUN apt install clang -y
RUN apt install llvm -y
RUN apt install docker -y

RUN mkdir /app

WORKDIR /app

VOLUME [ "/database", "/database2" ]

COPY Cargo.toml Cargo.lock ./
COPY src ./src

CMD ["cargo", "run"]