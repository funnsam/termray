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

const BALLS_SQRT: i32 = 10;
// https://coolors.co/palette/f94144-f3722c-f8961e-f9844a-f9c74f-90be6d-43aa8b-4d908e-577590-277da1
const COLORS: &[(u8, u8, u8)] = &[
    (0xF9, 0x41, 0x44),
    (0xF3, 0x72, 0x2C),
    (0xF8, 0x96, 0x1E),
    (0xF9, 0x84, 0x4A),
    (0xF9, 0xC7, 0x4F),
    (0x90, 0xBE, 0x6D),
    (0x43, 0xAA, 0x8B),
    (0x4D, 0x90, 0x8E),
    (0x57, 0x75, 0x90),
    (0x27, 0x7D, 0xA1),
];

fn generate_balls() -> Vec<(Sphere, Material)> {
    let mut buf = Vec::with_capacity(BALLS_SQRT as usize * BALLS_SQRT as usize + 1);
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

    for x in -BALLS_SQRT/2..BALLS_SQRT/2 {
        for z in -BALLS_SQRT/2..BALLS_SQRT/2 {
            let c = Vector3::new(
                rng.gen_range(0.0..0.9) + x as f64,
                0.2,
                rng.gen_range(0.0..0.9) + z as f64,
            );

            let color = COLORS[rng.gen_range(0..COLORS.len())];
            let color = Vector3::new(
                color.0 as f64 / 255.0,
                color.1 as f64 / 255.0,
                color.2 as f64 / 255.0,
            );
            let emit_color = if rng.gen() { color * 2.0 } else { Vector3::default() };

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
