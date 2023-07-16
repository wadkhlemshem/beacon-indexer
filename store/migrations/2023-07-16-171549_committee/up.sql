CREATE TABLE IF NOT EXISTS "committee" (
    "slot" NUMERIC(20, 0) NOT NULL,
    "index" SMALLINT NOT NULL,
    "validators" NUMERIC(20,0)[] NOT NULL,
    PRIMARY KEY ("slot", "index")
);
