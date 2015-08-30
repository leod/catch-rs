use net;

#[derive(CerealData)]
pub enum GameEvent {
    PlayerJoin(net::PlayerId, String),
    PlayerLeave(net::PlayerId),
    
    CreateEntity(net::PlayerId, net::EntityId, net::EntityTypeId),
    RemoveEntity(net::EntityId),

    PlaySound(String)
}
