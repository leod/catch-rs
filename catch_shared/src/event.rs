use cereal::{CerealData, CerealError, CerealResult};

use net::{PlayerId, NetEntityId, NetEntityTypeId};

#[derive(CerealData)]
enum GameEvent {
    PlayerJoin(PlayerId, String),
    PlayerLeave(PlayerId),
    
    CreateEntity(PlayerId, NetEntityId, NetEntityTypeId),
    RemoveEntity(NetEntityId),

    PlaySound(String)
}
