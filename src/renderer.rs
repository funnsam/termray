use nalgebra::{base::*, geometry::*};

pub const LIGHT_BOUNCES : usize = 16;
pub const SAMPLES_LVL   : usize = 4;

// #[derive(Default)]
pub struct RendererState<'a> {
    pub cam_pos: Vector3<f64>,
    // pub cam_dir: Vector3<f64>,

    pub rot: Vector2<f64>,
    
    pub scene: Vec<Object<'a>>
}

impl<'a> Default for RendererState<'a> {
    fn default() -> Self {
        Self {
            cam_pos: Vector3::default(),
//            cam_dir: Vector3::z(),
            rot: Vector2::default(),
            scene: Vec::new()
        }
    }
}

pub struct Object<'a> {
    pub kind: &'a dyn ObjectKind,
    pub material: &'a Material
}

impl<'a> Object<'a> {
    pub fn new(k: &'a dyn ObjectKind, m: &'a Material) -> Self {
        Self { kind: k, material: m }
    }
}

pub struct Material {
    pub color: Vector3<f64>,
    pub rfness: f64
}

pub trait ObjectKind {
    fn try_ray(&self, ray: &Ray) -> HitInfo;
}

#[derive(Default)]
pub struct HitInfo {
    pub p: Vector3<f64>,
    pub n: Vector3<f64>,
    pub t: f64,
}

pub fn rotate(p: Vector3<f64>, r: Vector2<f64>) -> Vector3<f64> {
    fn rot(v: f64) -> Matrix2<f64> {
        let s = v.sin();
        let c = v.cos();
        Matrix2::new(c, -s, s, c)
    }

    fn insert_x(va: Vector3<f64>, vb: Vector2<f64>) -> Vector3<f64> {
        Vector3::new(va[0], vb[0], vb[1])
    }

    fn insert_y(va: Vector3<f64>, vb: Vector2<f64>) -> Vector3<f64> {
        Vector3::new(vb[0], va[1], vb[1])
    }

    let mut rt = p.normalize();
    rt = insert_x(rt, rot(r[0]) * rt.yz());
    rt = insert_y(rt, rot(r[1]) * rt.xz());

    rt
}

pub fn render(rs: &mut RendererState, size: usize) -> Vec<Vec<(u8, u8, u8)>> {
    let mut scr = vec![vec![(0, 0, 0); size]; size];
    for ay in 0..size {
        for ax in 0..size {
            let x = (size - ax - 1) as f64;
            let y = (size - ay - 1) as f64;

            let mut c = Vector3::new(0.0, 0.0, 0.0);

            for ix in 0..SAMPLES_LVL {
                for iy in 0..SAMPLES_LVL {


                    let offx = (ix as f64 / SAMPLES_LVL as f64) - 0.5;
                    let offy = (iy as f64 / SAMPLES_LVL as f64) - 0.5;
                    let px = (x + offx) / size as f64 * 2.0 - 1.0;
                    let py = (y + offy) / size as f64 * 2.0 - 1.0;

                    let ray_pos = rs.cam_pos;

                    let mut ray_dir = rotate(Vector3::new(px, py, 1.0).normalize(), rs.rot);

                    let ray = Ray::new(ray_pos, ray_dir);

                    c += ray.get_color(&rs.scene, 0);
                }
            }

            c /= (SAMPLES_LVL * SAMPLES_LVL) as f64;
            scr[ay][ax] = (map(c[0]), map(c[1]), map(c[2]));
        }
    }
    scr
}

fn map(v: f64) -> u8 { (v.sqrt() * 255.0).min(255.0) as u8 }

pub struct Sphere {
    pub c: Vector3<f64>,
    pub r: f64,
}

impl ObjectKind for Sphere {
    fn try_ray(&self, r: &Ray) -> HitInfo {
        let o = r.origin - self.c;
        let a = r.direction.dot(&r.direction);
        let b = o.dot(&r.direction) * 2.0;
        let c = o.dot(&o) - self.r * self.r;
        let d = b * b - 4.0 * a * c;

        let mut hi = HitInfo::default();
        if d < 0.0 {
            hi.t = -1.0;
        } else {
            hi.t = (-b - d.sqrt()) / (2.0 * a);
            hi.p = r.at(hi.t);
            hi.n = (hi.p - self.c) / self.r;
        }

        hi
    }
}

pub struct Ray {
    origin: Vector3<f64>,
    direction: Vector3<f64>
}

impl Ray {
    pub fn new(origin: Vector3<f64>, direction: Vector3<f64>) -> Self {
        Self { origin, direction }
    }

    pub fn at(&self, t: f64) -> Vector3<f64> {
        self.origin + self.direction * t
    }

    fn try_hit<'a>(&self, scene: &'a Vec<Object<'a>>) -> Option<(HitInfo, &'a Object<'a>)> {
        let mut r = None;
        let mut t = f64::INFINITY;
        for i in scene {
            let h = i.kind.try_ray(self);
            if h.t > 0.0 && h.t < t {
                t = h.t;
                r = Some((h, i))
            }
        }
        r
    }

    pub fn get_color<'a>(&self, s: &Vec<Object<'a>>, i: usize) -> Vector3<f64> {
        if i == LIGHT_BOUNCES { return Vector3::new(0.0, 0.0, 0.0) }

        let h = self.try_hit(s);
        if h.is_some() {
            let (h, o) = h.unwrap();
            let specular_dir = self.direction - 2.0 * self.direction.dot(&h.n) * h.n;
            let specular_ray = Ray::new(h.p, specular_dir);

            // let reflect_ray = Ray::new(h.p, h.n);
            specular_ray.get_color(s, i+1) * o.material.rfness + o.material.color * (1.0 - o.material.rfness)
        } else {
            let t = 0.5 * (self.direction[1] + 1.0);
            (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) +
                t * Vector3::new(0.5, 0.7, 1.0)
        }
    }
}
