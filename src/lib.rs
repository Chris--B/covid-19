#[derive(Copy, Clone, Debug, Default)]
pub struct Person {
    /// Location in the simulated world.
    /// Not all possible locations may be valid, e.g. if they are obstructed by a wall.
    pos: (),

    /// Combined speed and direction in the simulated world.
    vel: (),

    /// Susceptibility to contracting the COVID virus
    health: Health,
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
