use player::PlayerId;
use net::{EntityId, EntityTypeId};

#[derive(CerealData)]
pub enum GameEvent {
    PlayerJoin(PlayerId, String),
    PlayerLeave(PlayerId),
    
    CreateEntity(PlayerId, EntityId, EntityTypeId),
    RemoveEntity(EntityId),

    PlaySound(String)
}
