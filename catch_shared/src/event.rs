use player::PlayerId;
use net::{EntityId, EntityTypeId};
use net::{TickNumber};

#[derive(CerealData)]
pub enum GameEvent {
    PlayerJoin(PlayerId, String),
    PlayerLeave(PlayerId),
    
    CreateEntity(EntityId, EntityTypeId, PlayerId),
    RemoveEntity(EntityId),

    PlaySound(String),

    // This event is only sent to specific players, to indicate
    // that this tick contains the server-side state for the player state
    // after some input by the player was processed on the server
    CorrectState(TickNumber),
}
