use std::io::{Read, Cursor};
use std::fs::File;
use std::collections::HashMap;

use na::Vec2;

use rodio;
use rodio::Endpoint;

pub struct Sounds {
    endpoint: Endpoint,
    sounds: HashMap<String, Vec<u8>>
}

impl Sounds {
    pub fn load() -> Result<Sounds, String> {
        let endpoint = match rodio::get_default_endpoint() {
            Some(endpoint) => endpoint,
            None => return Err("No sound device available".to_string()),
        };

        let mut s = Sounds {
            endpoint: endpoint,
            sounds: HashMap::new(),
        };

        try!(s.load_sound("dash", "data/sounds/270553__littlerobotsoundfactory__warpdrive-00.wav"));
        try!(s.load_sound("take_item", "data/sounds/162476__kastenfrosch__gotitem.wav"));
        try!(s.load_sound("equip_item", "data/sounds/254079__robinhood76__05505-punching-deploy-shot.wav"));

        Ok(s)
    }

    pub fn play(&self, name: &str, p: Vec2<f32>) {
        // TODO: ???
        let input = Cursor::new(self.sounds.get(name).unwrap().clone());
        rodio::play_once(&self.endpoint, input);
    }

    fn load_sound(&mut self, name: &str, file_name: &str) -> Result<(), String> {
        info!("loading sound {} from {}", name, file_name);

        let mut file = match File::open(file_name) {
            Ok(file) => file,
            Err(error) => return Err(format!("Could not load sound file {}: {:?}", file_name,
                                             error))
        };

        let mut contents: Vec<u8> = Vec::new();
        let result = file.read_to_end(&mut contents);

        if let Some(error) = result.err() {
            return Err(format!("Could not load sound file {}: {:?}", file_name, error));
        }
        if contents.is_empty() {
            return Err(format!("Sound empty {}", file_name));
        }

        self.sounds.insert(name.to_string(), contents);

        Ok(())
    }

}
