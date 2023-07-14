// @generated automatically by Diesel CLI.

diesel::table! {
    attestation (epoch, validator) {
        epoch -> Int8,
        validator -> Varchar,
        validator_index -> Int8,
        slot -> Int8,
        committee_index -> Int8,
        attested -> Bool,
    }
}

diesel::table! {
    epoch (index) {
        index -> Int8,
        active_validators -> Int8,
        total_validators -> Int8,
    }
}

diesel::table! {
    validator (pubkey) {
        pubkey -> Varchar,
        active -> Bool,
    }
}

diesel::joinable!(attestation -> epoch (epoch));
diesel::joinable!(attestation -> validator (validator));

diesel::allow_tables_to_appear_in_same_query!(
    attestation,
    epoch,
    validator,
);
