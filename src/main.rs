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
    
    let m1 = Material {
        color: Vector3::new(0.5, 0.0, 0.0),
        emit_color: Vector3::default(),
        reflective: 0.7,
        rough: 0.5,
    };
    let m2 = Material {
        color: Vector3::new(0.0, 0.5, 0.0),
        emit_color: Vector3::default(),
        reflective: 0.6,
        rough: 0.2,
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
    state.scene.push(o4);

    let mut fno = 1;
    loop {
        let s = SystemTime::now();

        let rr = render(&mut state, size as usize, &mut img, fno);

        terminal::push_image(rr, &format!("t {fps:.1} r {:.1} fno {fno}", 1000.0 / s.elapsed()?.as_millis() as f64))?;
        if terminal::handle_input(&mut state, s.elapsed()?)? {
            img = vec![vec![Vector3::default(); size as usize]; size as usize];
            fno = 0;
        };

        fps = 1000.0 / s.elapsed()?.as_millis() as f64;

        fno += 1;
    }
}
