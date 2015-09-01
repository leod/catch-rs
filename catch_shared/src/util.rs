use time::Duration;

pub struct PeriodicTimer {
    period: Duration,
    accum: Duration
}

impl PeriodicTimer {
    pub fn new(period: Duration) -> PeriodicTimer {
        PeriodicTimer {
            period: period,
            accum: Duration::zero()
        }
    }

    pub fn add(&mut self, a: Duration) {
        self.accum = self.accum + a;
    }

    pub fn next(&mut self) -> bool {
        if self.accum >= self.period {
            self.accum = self.accum - self.period;
            true
        } else {
            false
        }
    }
}
