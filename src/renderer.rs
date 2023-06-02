use nalgebra::base::*;
use rayon::prelude::*;
use rand::Rng;

pub const LIGHT_BOUNCES : usize = 16;
pub const SAMPLES_LVL   : usize = 16;
pub const RNG_LIMIT     : usize = 256;

// pub const SKY_LIGHT: Vector3<f64> = Vector3::new(1.0, 1.0, 0.8);
pub const SKY_LIGHT: Vector3<f64> = Vector3::new(0.0, 0.0, 0.0);

#[derive(Default)]
pub struct RendererState<'a> {
    pub cam_pos: Vector3<f64>,
    pub rot: Vector2<f64>,
    pub scene: Vec<Object<'a>>,

    pub focus: f64,
    pub aperture: f64
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

#[derive(Clone)]
pub struct Material {
    pub color: Vector3<f64>,
    pub emit_color: Vector3<f64>,
    pub shininess: f64,
    pub rough: f64
}

impl Default for Material {
    fn default() -> Self {
        Material {
            color: Vector3::new(0.5, 0.0, 0.0),
            emit_color: Vector3::default(),
            shininess: 0.3,
            rough: 0.7
        }
    }
}

pub trait ObjectKind: Sync + Send {
    fn try_ray(&self, ray: &Ray) -> HitInfo;
}

// #[derive(Default)]
pub struct HitInfo {
    pub p: Vector3<f64>,
    pub n: Vector3<f64>,
    pub t: f64,
}

impl Default for HitInfo {
    fn default() -> Self {
        Self { p: Vector3::default(), n: Vector3::default(), t: -1.0 }
    }
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

pub fn render(rs: &mut RendererState, size: usize, prev_img: &mut Vec<Vec<Vector3<f64>>>, passes_done: usize) -> Vec<Vec<(u8, u8, u8)>> {
    let scr_f = prev_img;
    let scr_i = vec![vec![(0, 0, 0); size]; size];
    scr_f.into_par_iter().enumerate().for_each(|(ay, scr_f)| {
        for ax in 0..size {
            let x = (size - ax - 1) as f64;
            let y = (size - ay - 1) as f64;

            let mut c = Vector3::default();

            for _ in 0..SAMPLES_LVL {
                let px = x / size as f64 * 2.0 - 1.0;
                let py = y / size as f64 * 2.0 - 1.0;

                let ray_pos = rs.cam_pos + generate_random_circle() * (0.05 * rs.aperture);

                let ray_tar = Vector3::new(px, py, 1.0) * rs.focus.max(0.05) - ray_pos;

                let ray_dir = rotate((ray_tar + rs.cam_pos).normalize(), rs.rot);

                let ray = Ray::new(ray_pos, ray_dir);

                let r = ray.get_color(&rs.scene, 0);
                c += apply_light(r.0, r.1);
            }

            c /= SAMPLES_LVL as f64;

            scr_f[ax] += c;
            c = scr_f[ax] / passes_done as f64;
            
            let scr_i = unsafe {
                &mut *(&scr_i
                    as *const Vec<Vec<(u8, u8, u8)>>
                    as *mut Vec<Vec<(u8, u8, u8)>>)
            };
            scr_i[ay][ax] = (map(c[0]), map(c[1]), map(c[2]));
        }
    });
    scr_i
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

#[derive(Debug)]
pub struct Triangle {
    pub vp: [Vector3<f64>; 3],
    pub vn: Option<[Vector3<f64>; 3]>,
}

impl ObjectKind for Triangle {
    fn try_ray(&self, r: &Ray) -> HitInfo {
        let mut hi = HitInfo::default();

        let v0v1 = self.vp[1] - self.vp[0];
        let v0v2 = self.vp[2] - self.vp[0];
        let pvec = r.direction.cross(&v0v2);
        let det = v0v1.dot(&pvec);

        if det < 0.0 { return hi }

        let inv_det = 1.0 / det;

        let tvec = r.origin - self.vp[0];
        let u = tvec.dot(&pvec) * inv_det;
        if u < 0.0 || u > 1.0 { return hi }

        let qvec = tvec.cross(&v0v1);
        let v = r.direction.dot(&qvec) * inv_det;
        if v < 0.0 || u+v > 1.0 { return hi }

        let t = v0v2.dot(&qvec) * inv_det;
        
        hi.t = t;
        hi.p = r.at(t);
        hi.n = match self.vn {
            Some(vn) => (1.0-u-v) * vn[0] + u * vn[1] + v * vn[2],
            None => v0v1.cross(&v0v2).normalize(),
        };

        hi
    }
}

pub struct Mesh {
    pub ts: Vec<Triangle>
}

impl ObjectKind for Mesh {
    fn try_ray(&self, r: &Ray) -> HitInfo {
        let mut fhi = None;
        let mut t = f64::INFINITY;
        for i in self.ts.iter() {
            let h = i.try_ray(r);
            if h.t > 0.001 && h.t < t {
                t = h.t;
                fhi = Some(h)
            }
        }
        fhi.unwrap_or(HitInfo { p: Vector3::default(), n: Vector3::default(), t: -1.0 })
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

    pub fn try_hit<'a>(&self, scene: &'a Vec<Object<'a>>) -> Option<(HitInfo, &'a Object<'a>)> {
        let mut r = None;
        let mut t = f64::INFINITY;
        for i in scene {
            let h = i.kind.try_ray(self);
            if h.t > 0.001 && h.t < t {
                t = h.t;
                r = Some((h, i))
            }
        }
        r
    }

    pub fn get_color<'a>(&self, s: &Vec<Object<'a>>, i: usize) -> (Vector3<f64>, Vector3<f64>) {
        if i == LIGHT_BOUNCES {
            return (
                Vector3::default(),
                Vector3::default()
            )
        }

        let h = self.try_hit(s);
        if h.is_some() {
            let (h, o) = h.unwrap();
            let specular_dir = self.direction - 2.0 * self.direction.dot(&h.n) * h.n;
            let diffuse_dir  = h.n + generate_random_sphere().normalize();

            let indirect_ray = Ray::new(h.p, specular_dir.lerp(&diffuse_dir, o.material.rough));
            let srr = indirect_ray.get_color(s, i+1);

            (
                srr.0 * o.material.shininess
                + o.material.color * (1.0 - o.material.shininess),
                (
                    srr.1 * (0.35 + o.material.shininess * (1.0 - 0.35))
                    + o.material.emit_color
                ) * (1.0 - (h.t.abs() / (h.t.abs() + 100.0)))
            )
        } else {
            let t = 0.5 * (self.direction[1] + 1.0);
            let sc = (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);
            (sc, SKY_LIGHT * (t + 0.5))
        }
    }
}

fn generate_random_sphere() -> Vector3<f64> {
    let mut rng = rand::thread_rng();
    for _ in 0..RNG_LIMIT {
        let p = Vector3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        );
        if (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]) < 1.0 {
            return p;
        }
    }
    Vector3::default()
}

fn generate_random_circle() -> Vector3<f64> {
    let mut rng = rand::thread_rng();
    for _ in 0..RNG_LIMIT {
        let p = Vector3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            0.0,
        );
        if (p[0] * p[0] + p[1] * p[1]) < 1.0 {
            return p;
        }
    }
    Vector3::default()
}

fn apply_light(c: Vector3<f64>, l: Vector3<f64>) -> Vector3<f64> {
    let mut o = Vector3::default();
    o[0] = c[0] * l[0];
    o[1] = c[1] * l[1];
    o[2] = c[2] * l[2];
    o
}
