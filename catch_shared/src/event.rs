use NetEntityId;

enum GameEvent {
    PlayerJoin(PlayerId, String),
    PlayerLeave(PlayerId),
    
    CreateEntity(PlayerId),
    RemoveEntity(),

    PlaySound()
}
