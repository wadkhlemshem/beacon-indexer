// @generated automatically by Diesel CLI.

diesel::table! {
    attestation (epoch_index, validator_index) {
        epoch_index -> Int8,
        validator_index -> Int8,
        slot -> Int8,
        committee_index -> Int2,
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
    validator (index) {
        index -> Int8,
        pubkey -> Varchar,
    }
}

diesel::table! {
    validator_history (validator_index, epoch_index) {
        validator_index -> Int8,
        epoch_index -> Int8,
        is_active -> Bool,
    }
}

diesel::joinable!(attestation -> epoch (epoch_index));
diesel::joinable!(attestation -> validator (validator_index));
diesel::joinable!(validator_history -> epoch (epoch_index));
diesel::joinable!(validator_history -> validator (validator_index));

diesel::allow_tables_to_appear_in_same_query!(
    attestation,
    epoch,
    validator,
    validator_history,
);
