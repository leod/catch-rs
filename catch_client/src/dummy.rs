use time;
use rand;

use shared::{PlayerInput};
use shared::player::{NUM_INPUT_KEYS};
use shared::net::{ClientMessage, TimedPlayerInput};
use shared::util::PeriodicTimer;

use client::Client;

const INPUT_PERIOD_S: f32 = 0.01;

pub struct DummyClient {
    client: Client,

    input_timer: PeriodicTimer,
    input: PlayerInput,
}

impl DummyClient {
    pub fn new(client: Client) -> DummyClient {
        DummyClient {
            client: client,
            input_timer: PeriodicTimer::new(INPUT_PERIOD_S),
            input: PlayerInput::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let frame_start_s = time::precise_time_s() as f32;

            loop {
                match self.client.service().unwrap() {
                    Some(_) => continue,
                    None => break
                }

                while self.client.num_ticks() > 0 {
                    self.client.pop_next_tick();
                }
            }

            while self.input_timer.next() {
                self.mutate_input();

                self.client.send(&ClientMessage::PlayerInput(
                    TimedPlayerInput {
                        duration_s: INPUT_PERIOD_S,
                        input: self.input.clone(),
                    }
                ));
            }

            let frame_end_s = time::precise_time_s() as f32;

            self.input_timer.add(frame_end_s - frame_start_s);
        }
    }

    fn mutate_input(&mut self) {
        const NUM_CHANGES: usize = 2;

        let num = NUM_CHANGES;
        for _ in 0..num {
            let i = rand::random::<usize>() % NUM_INPUT_KEYS;
            self.input.pressed[i] = !self.input.pressed[i];
        }
    }
}
