pub trait Subscribable {
    fn subscribe_event() -> SubscribeEvent;
}

pub enum SubscribeEvent {
    Head,
    Block,
    Attestation,
    VoluntaryExit,
    FinalizedCheckpoint,
    ChainReorg,
    ContributionAndProof,
}

impl ToString for SubscribeEvent {
    fn to_string(&self) -> String {
        match self {
            SubscribeEvent::Head => "head".to_string(),
            SubscribeEvent::Block => "block".to_string(),
            SubscribeEvent::Attestation => "attestation".to_string(),
            SubscribeEvent::VoluntaryExit => "voluntary_exit".to_string(),
            SubscribeEvent::FinalizedCheckpoint => "finalized_checkpoint".to_string(),
            SubscribeEvent::ChainReorg => "chain_reorg".to_string(),
            SubscribeEvent::ContributionAndProof => "contribution_and_proof".to_string(),
        }
    }
}
