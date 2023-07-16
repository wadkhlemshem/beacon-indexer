CREATE TABLE IF NOT EXISTS "epoch" (
    "index" NUMERIC(20,0) PRIMARY KEY NOT NULL,
    "active_validators" NUMERIC(20,0) NOT NULL,
    "total_validators" NUMERIC(20,0) NOT NULL
);
