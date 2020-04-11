use ultraviolet::vec::Vec2;

use std::time;

#[derive(Copy, Clone, Debug)]
pub struct Person {
    /// Location in the simulated world.
    /// Not all possible locations may be valid, e.g. if they are obstructed by a wall.
    pub pos: Vec2,

    /// Combined speed and direction in the simulated world.
    pub vel: Vec2,

    /// Susceptibility to contracting the COVID virus
    pub health: Health,
}

impl Default for Person {
    fn default() -> Person {
        Person {
            pos: Vec2::new(0., 0.),
            vel: Vec2::new(0., 0.),
            health: Health::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Health {
    /// This person has never had the COVID virus
    Healthy,

    /// This person has the virus and is contageous
    Sick,

    /// This person has had the virus, but is no longer contageous and cannot re-acquire the disease.
    Recovered,
}

impl Default for Health {
    fn default() -> Self {
        Health::Healthy
    }
}

#[derive(Clone, Debug)]
pub struct Simulation {
    /// Lower left-hand corner of the simulation world.
    /// This is used with `upper` to bound the world into a box.
    lower: Vec2,

    /// Upper right-hand corner of the simulation world.
    /// This is used with `lower` to bound the world into a box.
    upper: Vec2,

    /// All persons in the simulation that may interact
    people: Vec<Person>,
}

impl Simulation {
    // Create a simulation with some test data
    pub fn sample_set() -> Simulation {
        let people = vec![Person {
            pos: Vec2::new(0., 0.),
            vel: Vec2::new(10., 10.),
            ..Person::default()
        }];

        Simulation {
            lower: Vec2::new(-100., -100.),
            upper: Vec2::new(100., 100.),
            people,
        }
    }

    /// Advance the simulation by one step
    pub fn tick(&mut self, dt: time::Duration) {
        // Each hit against a wall reduces the velocity by this factor
        // Factors > 1 "speed up" the system

        const DECAY_FACTOR: f32 = 0.95;
        let dt_s = dt.as_secs_f32();

        for i in 0..self.people.len() {
            let p = &mut self.people[i];

            let mut next_p = p.pos + dt_s * p.vel;
            let mut next_v = p.vel;

            if next_p.x <= self.lower.x {
                next_p.x = self.lower.x;
                next_v.x = -next_v.x;
                next_v *= DECAY_FACTOR;
            } else if next_p.x >= self.upper.x {
                next_p.x = self.upper.x;
                next_v.x = -next_v.x;
                next_v *= DECAY_FACTOR;
            }

            if next_p.y <= self.lower.y {
                next_p.y = self.lower.y;
                next_v.y = -next_v.y;
                next_v *= DECAY_FACTOR;
            } else if next_p.y >= self.upper.y {
                next_p.y = self.upper.y;
                next_v.y = -next_v.y;
                next_v *= DECAY_FACTOR;
            }

            p.pos = next_p;
            p.vel = next_v;
        }
    }
}
