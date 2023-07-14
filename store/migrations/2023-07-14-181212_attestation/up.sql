CREATE TABLE IF NOT EXISTS "attestation" (
    "epoch_index" BIGINT NOT NULL,
    "validator_index" BIGINT NOT NULL,
    "slot" SMALLINT NOT NULL,
    "committee_index" BIGINT NOT NULL,
    "attested" BOOLEAN NOT NULL,
    PRIMARY KEY ("epoch", "validator"),
    CONSTRAINT "fk_attestation_epoch"
        FOREIGN KEY ("epoch_index")
        REFERENCES "epoch" ("index"),
    CONSTRAINT "fk_attestation_validator"
        FOREIGN KEY ("validator_index")
        REFERENCES "validator" ("index")
);
