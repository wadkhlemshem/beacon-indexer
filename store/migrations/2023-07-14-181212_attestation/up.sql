CREATE TABLE IF NOT EXISTS "attestation" (
    "epoch_index" NUMERIC(20,0) NOT NULL,
    "validator_index" NUMERIC(20,0) NOT NULL,
    "slot" NUMERIC(20,0) NOT NULL,
    "committee_index" SMALLINT NOT NULL,
    "attested" BOOLEAN NOT NULL,
    PRIMARY KEY ("epoch_index", "validator_index")
);
