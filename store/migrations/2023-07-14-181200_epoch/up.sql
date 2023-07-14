CREATE TABLE IF NOT EXISTS "epoch" (
    "index" BIGINT PRIMARY KEY NOT NULL,
    "active_validators" BIGINT NOT NULL,
    "total_validators" BIGINT NOT NULL
);
