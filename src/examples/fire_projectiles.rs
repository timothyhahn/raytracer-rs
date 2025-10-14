use crate::core::tuples::{Point, Vector};

pub struct Projectile {
    pub position: Point,
    pub velocity: Vector,
}

pub struct Environment {
    pub gravity: Vector,
    pub wind: Vector,
}

pub fn tick(env: &Environment, proj: Projectile) -> Projectile {
    let position = proj.position + proj.velocity;
    let velocity = proj.velocity + env.gravity + env.wind;
    Projectile { position, velocity }
}

#[cfg(test)]
mod tests {
    use super::{tick, Environment, Projectile};
    use crate::core::tuples::{Point, Tuple, Vector};

    #[test]
    fn ticking_projectile() {
        let mut p = Projectile {
            position: Point::new(0.0, 1.0, 0.0),
            velocity: Vector::new(1.0, 1.0, 0.0).normalize(),
        };

        let e = Environment {
            gravity: Vector::new(0.0, -0.1, 0.0),
            wind: Vector::new(-0.01, 0.0, 0.0),
        };

        let mut count_iterations = 0;
        while p.position.y > 0.0 {
            p = tick(&e, p);
            count_iterations += 1;
        }
        assert_eq!(count_iterations, 17);
    }
}
