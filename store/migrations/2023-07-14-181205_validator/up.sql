CREATE TABLE IF NOT EXISTS "validator" (
    "index" BIGINT PRIMARY KEY NOT NULL,
    "pubkey" VARCHAR NOT NULL,
    "active" BOOLEAN NOT NULL,
    "epochs" BIGINT NOT NULL
);
