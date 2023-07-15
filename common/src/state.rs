use std::fmt::Display;

pub enum StateId {
    Head,
    Genesis,
    Finalized,
    Justified,
    Slot(u64),
    StateRoot(String),
}

impl Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateId::Head => write!(f, "head"),
            StateId::Genesis => write!(f, "genesis"),
            StateId::Finalized => write!(f, "finalized"),
            StateId::Justified => write!(f, "justified"),
            StateId::Slot(slot) => write!(f, "{slot}"),
            StateId::StateRoot(root) => write!(f, "{root}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_id_display() {
        assert_eq!(format!("{}", StateId::Head), "head");
        assert_eq!(format!("{}", StateId::Genesis), "genesis");
        assert_eq!(format!("{}", StateId::Finalized), "finalized");
        assert_eq!(format!("{}", StateId::Justified), "justified");
        assert_eq!(format!("{}", StateId::Slot(123)), "123");
        assert_eq!(
            format!("{}", StateId::StateRoot("0x1234567890abcdef".to_string())),
            "0x1234567890abcdef"
        );
    }
}
