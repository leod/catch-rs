use std::collections::HashMap;
use time::Duration;

use ecs;
use ecs::entity::IndexedEntity;
use ecs::{Aspect, DataHelper, EntityData, EntityIter, ComponentManager};

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

    pub fn deactivated(&mut self, entity: &EntityData<T>, components: &T) {
        self.interested.remove(entity);
    }

    pub fn iter<'a>(&'a self) -> EntityIter<'a, T> {
        EntityIter::Map(self.interested.values()) 
    }
}

