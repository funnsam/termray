use std::time::SystemTime;

mod terminal;
mod renderer;

use nalgebra::base::*;
use renderer::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let size = terminal::init()?;
    let mut fps = 0.0;
    let mut state = renderer::RendererState::default();
    let mut img = vec![vec![Vector3::default(); size as usize]; size as usize];
    
    /*let m1 = Material {
        color: Vector3::new(0.5, 0.0, 0.0),
        emit_color: Vector3::default(),
        reflective: 0.7,
        rough: 0.5,
    };
    let m2 = Material {
        color: Vector3::new(0.0, 0.5, 0.0),
        emit_color: Vector3::default(),
        reflective: 0.7,
        rough: 0.0,
    };
    let m3 = Material {
        color: Vector3::new(0.3, 0.5, 1.0),
        emit_color: Vector3::default(),
        reflective: 0.4,
        rough: 0.75,
    };
    let m4 = Material {
        color: Vector3::new(1.0, 1.0, 1.0),
        emit_color: Vector3::new(2.0, 2.0, 2.0),
        reflective: 0.0,
        rough: 0.0,
    };

    let s1 = Sphere {
        c: Vector3::new(-0.6, 0.0, 1.0),
        r: 0.5,
    };
    let s2 = Sphere {
        c: Vector3::new(0.6, 0.0, 1.0),
        r: 0.5,
    };
    let s3 = Sphere {
        c: Vector3::new(0.0, -201.0, 0.0),
        r: 200.0,
    };
    let s4 = Sphere {
        c: Vector3::new(0.0, 1.0, 1.15),
        r: 0.5,
    };

    let o1 = Object::new(&s1, &m1);
    let o2 = Object::new(&s2, &m2);
    let o3 = Object::new(&s3, &m3);
    let o4 = Object::new(&s4, &m4);

    state.scene.push(o1);
    state.scene.push(o2);
    state.scene.push(o3);
    state.scene.push(o4);*/

    let s = generate_balls();
    for (o, m) in s.iter() {
        state.scene.push(Object::new(o, m));
    }

    let mut fno = 1;
    loop {
        let s = SystemTime::now();

        let rr = render(&mut state, size as usize, &mut img, fno);

        terminal::push_image(rr, &format!("t {fps:.1} r {:.1} fno {fno} focus {} aperture {}", 1000.0 / s.elapsed()?.as_millis() as f64, state.focus, state.aperture))?;
        if terminal::handle_input(&mut state, s.elapsed()?)? {
            img = vec![vec![Vector3::default(); size as usize]; size as usize];
            fno = 0;
        };

        fps = 1000.0 / s.elapsed()?.as_millis() as f64;

        fno += 1;
    }
}

fn generate_balls() -> Vec<(Sphere, Material)> {
    fn len(v: Vector3<f64>) -> f64 {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }

    let mut buf = Vec::with_capacity(101);
    buf.push((Sphere {
        c: Vector3::new(0.0, -1000.0, 0.0),
        r: 1000.0,
    }, Material {
        color: Vector3::new(0.3, 0.5, 1.0),
        emit_color: Vector3::default(),
        reflective: 0.4,
        rough: 0.75,
    }));

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for x in -5..5 {
        for z in -5..5 {
            let c = Vector3::new(
                rng.gen_range(0.0..0.9) + x as f64,
                0.2,
                rng.gen_range(0.0..0.9) + z as f64,
            );

            if len(c - Vector3::new(4.0, 0.2, 0.0)) <= 0.9 { continue; }

            let color = Vector3::new(
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
            );
            let emit_color = if (color.sum() / 3.0) > 0.75 { color } else { Vector3::default() };

            buf.push((Sphere {
                c, r: 0.2
            }, Material {
                color, emit_color,
                reflective: rng.gen_range(0.0..1.0),
                rough: rng.gen_range(0.0..1.0),
            }));
        }
    }

    buf
}
