CREATE TABLE IF NOT EXISTS "validator" (
    "index" NUMERIC(20,0) PRIMARY KEY NOT NULL,
    "pubkey" VARCHAR NOT NULL,
    "activation_epoch" NUMERIC(20,0) NOT NULL,
    "exit_epoch" NUMERIC(20,0) NOT NULL
);
