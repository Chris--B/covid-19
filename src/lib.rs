use nalgebra::{Point2, Vector2};

#[derive(Copy, Clone, Debug)]
pub struct Person {
    /// Location in the simulated world.
    /// Not all possible locations may be valid, e.g. if they are obstructed by a wall.
    pub pos: Point2<f64>,

    /// Combined speed and direction in the simulated world.
    pub vel: Vector2<f64>,

    /// Susceptibility to contracting the COVID virus
    pub health: Health,
}

impl Default for Person {
    fn default() -> Person {
        Person {
            pos: Point2::new(0., 0.),
            vel: Vector2::new(0., 0.),
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

pub struct Simulation {
    people: Vec<Person>,
}

impl Simulation {
    pub fn with_population(people: impl Iterator<Item = Person>) -> Simulation {
        Simulation {
            people: people.collect(),
        }
    }
}
