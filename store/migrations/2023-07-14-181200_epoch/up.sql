CREATE TABLE IF NOT EXISTS "epoch" (
    "index" NUMERIC(20,0) PRIMARY KEY NOT NULL,
    "active_validators" BIGINT NOT NULL,
    "total_validators" BIGINT NOT NULL
);
