use crate::tuples::Tuple;

pub struct Projectile {
    pub position: Tuple,
    pub velocity: Tuple,
}

pub struct Environment {
    pub gravity: Tuple,
    pub wind: Tuple,
}

pub fn tick(env: &Environment, proj: Projectile) -> Projectile {
    let position = proj.position + proj.velocity;
    let velocity = proj.velocity + env.gravity + env.wind;
    Projectile { position, velocity }
}

#[cfg(test)]
mod tests {
    use crate::fire_projectiles::{tick, Environment, Projectile};
    use crate::tuples::Tuple;

    #[test]
    fn test_ticking_projectile() {
        let mut p = Projectile {
            position: Tuple::point(0.0, 1.0, 0.0),
            velocity: Tuple::vector(1.0, 1.0, 0.0).normalize(),
        };

        let e = Environment {
            gravity: Tuple::vector(0.0, -0.1, 0.0),
            wind: Tuple::vector(-0.01, 0.0, 0.0),
        };

        let mut count_iterations = 0;
        while p.position.y > 0.0 {
            p = tick(&e, p);
            count_iterations += 1;
        }
        assert_eq!(count_iterations, 17);
    }
}
