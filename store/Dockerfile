FROM rust:slim-bookworm

RUN apt update -qq && \
    apt install -y libpq-dev libsqlite3-dev default-libmysqlclient-dev wget && \
    rm -rf /var/lib/apt/lists/* && \
    \
    cargo install diesel_cli

RUN wget https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh && \
    chmod +x wait-for-it.sh

COPY ./migrations /migrations
COPY ./docker/entrypoint.sh /entrypoint.sh

ENTRYPOINT /bin/bash -c "./wait-for-it.sh $DATABASE_HOST_PORT --strict --timeout=30 -- /entrypoint.sh"
