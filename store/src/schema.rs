// @generated automatically by Diesel CLI.

diesel::table! {
    attestation (epoch_index, validator_index) {
        epoch_index -> Numeric,
        validator_index -> Numeric,
        slot -> Numeric,
        committee_index -> Int2,
        attested -> Bool,
    }
}

diesel::table! {
    committee (slot, index) {
        slot -> Numeric,
        index -> Int2,
        validators -> Array<Nullable<Numeric>>,
    }
}

diesel::table! {
    epoch (index) {
        index -> Numeric,
        active_validators -> Int8,
        total_validators -> Int8,
    }
}

diesel::table! {
    validator (index) {
        index -> Numeric,
        pubkey -> Varchar,
        activation_epoch -> Numeric,
        exit_epoch -> Numeric,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    attestation,
    committee,
    epoch,
    validator,
);
