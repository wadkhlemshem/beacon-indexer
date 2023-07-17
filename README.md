# Ethereum Beacon Chain Indexer
## HOW TO RUN
This project uses `docker` for the database. To run, simply run the following command:
```shell
docker compose up
```

Then, to run the indexer, run the following command:
```shell
cargo run --bin indexer
```

You can use the GraphQL playground to query the database at `http://localhost:8080`. To start the GraphQL server, run the following command:
```shell
cargo run --bin api
```
