CREATE TABLE IF NOT EXISTS "attestation" (
    "epoch_index" NUMERIC(20,0) NOT NULL,
    "validator_index" BIGINT NOT NULL,
    "slot" BIGINT NOT NULL,
    "committee_index" SMALLINT NOT NULL,
    "attested" BOOLEAN NOT NULL,
    PRIMARY KEY ("epoch_index", "validator_index"),
    CONSTRAINT "fk_attestation_epoch"
        FOREIGN KEY ("epoch_index")
        REFERENCES "epoch" ("index"),
    CONSTRAINT "fk_attestation_validator"
        FOREIGN KEY ("validator_index")
        REFERENCES "validator" ("index")
);
