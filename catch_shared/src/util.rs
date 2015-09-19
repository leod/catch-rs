use std::collections::HashMap;
use time::Duration;

use ecs;
use ecs::entity::IndexedEntity;
use ecs::{Aspect, EntityData, EntityIter, ComponentManager};

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

    // Percentual progress until next period
    pub fn progress(&self) -> f64 {
        // NOTE: Apparently this fails when either num_nanoseconds() exceeds 2^63 = ~292 years

        self.accum.num_nanoseconds().unwrap() as f64 /
        self.period.num_nanoseconds().unwrap() as f64
    }
}

pub struct CachedAspect<T: ComponentManager> {
    aspect: Aspect<T>,
    interested: HashMap<ecs::Entity, ecs::IndexedEntity<T>>,
}

impl<T: ComponentManager> CachedAspect<T> {
    pub fn new(aspect: Aspect<T>) -> CachedAspect<T> {
        CachedAspect {
            aspect: aspect,
            interested: HashMap::new(),
        }
    }

    pub fn activated(&mut self, entity: &EntityData<T>, components: &T) {
        if self.aspect.check(entity, components) {
            self.interested.insert(***entity, (**entity).__clone());
        }
    }

    pub fn reactivated(&mut self, entity: &EntityData<T>, components: &T) {
        if self.interested.contains_key(entity) {
            if !self.aspect.check(entity, components) {
                self.interested.remove(entity);
            }
        }
        else if self.aspect.check(entity, components) {
            self.interested.insert(***entity, (**entity).__clone());
        }
    }

    pub fn deactivated(&mut self, entity: &EntityData<T>, _: &T) {
        self.interested.remove(entity);
    }

    pub fn iter<'a>(&'a self) -> EntityIter<'a, T> {
        EntityIter::Map(self.interested.values()) 
    }
}

#[macro_export]
macro_rules! impl_cached_system { 
    ($c:ident, $s:ident, $x:ident, $($y:ident),*) => {
        impl ::ecs::System for $x {
            type Components = $c;
            type Services = $s;

            fn activated(&mut self, entity: &::ecs::EntityData<$c>, components: &$c,
                         _: &mut $s) {
                $(
                    self.$y.activated(entity, components);
                )*
            }

            fn reactivated(&mut self, entity: &::ecs::EntityData<$c>, components: &$c,
                           _: &mut $s) {
                $(
                    self.$y.reactivated(entity, components);
                )*
            }

            fn deactivated(&mut self, entity: &::ecs::EntityData<$c>, components: &$c,
                           _: &mut $s) {
                $(
                    self.$y.deactivated(entity, components);
                )*
            }
        }
    }
}
