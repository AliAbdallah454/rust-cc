FROM python

RUN apt update

RUN mkdir /app

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

VOLUME [ "/database" ]

EXPOSE 8000

CMD ["cargo", "run"]