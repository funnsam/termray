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
    
    let s = load_obj()?;
    for (o, m) in s.iter() {
        state.scene.push(Object::new(&**o, m));
    }
    let s = generate_balls();
    for (o, m) in s.iter() {
        state.scene.push(Object::new(&**o, m));
    }
    let (fo, fm) = generate_floor();
    state.scene.push(Object::new(&fo, &fm));

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

fn generate_floor() -> (Mesh, Material) {
    (
        Mesh { ts: vec![
            Triangle {
                v: [
                    Vector3::new(-1000.0, FLOOR_HEIGHT,  1000.0),
                    Vector3::new( 1000.0, FLOOR_HEIGHT,  1000.0),
                    Vector3::new( 1000.0, FLOOR_HEIGHT, -1000.0),
                ]
            },
            Triangle {
                v: [
                    Vector3::new(-1000.0, FLOOR_HEIGHT,  1000.0),
                    Vector3::new( 1000.0, FLOOR_HEIGHT, -1000.0),
                    Vector3::new(-1000.0, FLOOR_HEIGHT, -1000.0),
                ]
            },
        ]},
        Material {
            color: Vector3::new(0.3, 0.5, 1.0),
            emit_color: Vector3::default(),
            shininess: 0.4,
            rough: 0.75,
        }
    )
}

const OBJ_SCALE: f64 = 10.0;
const OBJ_OFFSET: Vector3<f64> = Vector3::new(0.0, 2.0, -3.0);
const FLOOR_HEIGHT: f64 = 0.0;

fn load_obj() -> Result<Vec<(Box<dyn ObjectKind>, Material)>, Box<dyn std::error::Error>> {
    let obj = tobj::load_obj("model.obj", &tobj::GPU_LOAD_OPTIONS)?;
    let (models, materials) = obj;

    let mut buf: Vec<(Box<dyn ObjectKind>, Material)> = Vec::with_capacity(models.len());

    for i in models.iter() {
        let mesh = &i.mesh;

        let mut ts = Vec::with_capacity(mesh.positions.len() / 3);

        for j in mesh.indices.chunks(3) {
            let pos0 = Vector3::new(
                mesh.positions[j[0] as usize * 3 + 0],
                mesh.positions[j[0] as usize * 3 + 1],
                mesh.positions[j[0] as usize * 3 + 2],
            );
            let pos1 = Vector3::new(
                mesh.positions[j[1] as usize * 3 + 0],
                mesh.positions[j[1] as usize * 3 + 1],
                mesh.positions[j[1] as usize * 3 + 2],
            );
            let pos2 = Vector3::new(
                mesh.positions[j[2] as usize * 3 + 0],
                mesh.positions[j[2] as usize * 3 + 1],
                mesh.positions[j[2] as usize * 3 + 2],
            );

            ts.push(Triangle { v: [pos0*OBJ_SCALE+OBJ_OFFSET, pos1*OBJ_SCALE+OBJ_OFFSET, pos2*OBJ_SCALE+OBJ_OFFSET] })
        }

        let mat = match mesh.material_id {
            Some(i) => {
                fn to(a: [f64;3]) -> Vector3<f64> {
                    Vector3::new(a[0], a[1], a[2])
                }

                let om = &materials.as_ref().unwrap()[i];
                let mut nm = Material::default();

                if om.diffuse.is_some() { nm.color = to(om.diffuse.unwrap()); }
                if om.ambient.is_some() { nm.emit_color = to(om.ambient.unwrap()); }
                if om.shininess.is_some() { nm.shininess = om.shininess.unwrap(); }
                if om.dissolve.is_some() { nm.rough = om.dissolve.unwrap(); }

                nm
            },
            None => Material::default()
        };

        buf.push((Box::new(Mesh { ts }), mat))
    }

    Ok(buf)
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

fn generate_balls() -> Vec<(Box<dyn ObjectKind>, Material)> {
    let mut buf: Vec<(Box<dyn ObjectKind>, Material)> = Vec::with_capacity(BALLS_SQRT as usize * BALLS_SQRT as usize + 2);

    use rand::Rng;
    let mut rng = rand::thread_rng();

    for x in -BALLS_SQRT/2..BALLS_SQRT/2 {
        for z in -BALLS_SQRT/2..BALLS_SQRT/2 {
            let c = Vector3::new(
                rng.gen_range(0.0..0.8) + x as f64,
                rng.gen_range(0.2..2.5),
                rng.gen_range(0.0..0.8) + z as f64,
            );

            let a = c - OBJ_OFFSET;
            if a[0] * a[0] + a[1] * a[1] + a[2] * a[2] < 2.5 {
                continue
            }

            let color = COLORS[rng.gen_range(0..COLORS.len())];
            let color = Vector3::new(
                color.0 as f64 / 255.0,
                color.1 as f64 / 255.0,
                color.2 as f64 / 255.0,
            );
            let emit_color = if rng.gen() { color * rng.gen_range(5.0..20.0) } else { Vector3::default() };

            buf.push((Box::new(Sphere {
                c, r: 0.2
            }), Material {
                color, emit_color,
                shininess: rng.gen_range(0.0..1.0),
                rough: rng.gen_range(0.0..1.0),
            }));
        }
    }

    buf
}
