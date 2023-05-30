use nalgebra::{base::*, geometry::*};
use crossterm::{*, style::{Color, Stylize}, event::*};
use std::io::{stdout, Write};
use std::time::Duration;
use std::f64::consts::TAU;

pub fn init() -> core::result::Result<u16, Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;
    execute!(stdout(),
        terminal::EnterAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    let (c, r) = terminal::size()?;

    Ok(c.min(r << 1).min(50))
}

pub fn prep_exit() -> core::result::Result<(), Box<dyn std::error::Error>> {
    execute!(stdout(),
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;
    terminal::disable_raw_mode()?;

    Ok(())
}

pub fn push_image(image: Vec<Vec<(u8, u8, u8)>>, tfps: f64, rfps: f64) -> core::result::Result<(), Box<dyn std::error::Error>> {
    let mut so = stdout();
    queue!(so, cursor::MoveTo(0, 0))?;
    for y in image.chunks(2) {
        for xi in 0..y[0].len() {
            let t = y[0][xi];
            let b = if y.len() == 2 {y[1][xi] } else { (0, 0, 0) };

            queue!(so,
                style::PrintStyledContent("\u{2580}"
                    .with(Color::Rgb { r: t.0, g: t.1, b: t.2 })
                    .on  (Color::Rgb { r: b.0, g: b.1, b: b.2 })
                )
            )?;
        }
        queue!(so, cursor::MoveToNextLine(1))?;
    }

    queue!(so,
        terminal::Clear(terminal::ClearType::CurrentLine),
        style::PrintStyledContent(format!("r {rfps:.1} t {tfps:.1}")
            .with(Color::White)
            .on  (Color::DarkGrey)
        )
    )?;

    so.flush()?;

    Ok(())
}

pub fn handle_input(state: &mut crate::renderer::RendererState, el: Duration) -> core::result::Result<(), Box<dyn std::error::Error>> {
    if poll(
        Duration::from_millis(
            30_u128.checked_sub(el.as_millis()).unwrap_or(0) as u64
        )
    )? {
        match read()? {
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                prep_exit()?;
                std::process::exit(0);
            },
            Event::Key(KeyEvent { code: KeyCode::Char('a'), .. }) => state.cam_pos += Rotation3::new(state.cam_dir) * Vector3::new( 0.25, 0.0, 0.0),
            Event::Key(KeyEvent { code: KeyCode::Char('d'), .. }) => state.cam_pos += Rotation3::new(state.cam_dir) * Vector3::new(-0.25, 0.0, 0.0),
            Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => state.cam_pos += Rotation3::new(state.cam_dir) * Vector3::new(0.0,  0.25, 0.0), 
            Event::Key(KeyEvent { code: KeyCode::Char('e'), .. }) => state.cam_pos += Rotation3::new(state.cam_dir) * Vector3::new(0.0, -0.25, 0.0), 
            Event::Key(KeyEvent { code: KeyCode::Char('w'), .. }) => state.cam_pos += Rotation3::new(state.cam_dir) * Vector3::new(0.0, 0.0,  0.25),
            Event::Key(KeyEvent { code: KeyCode::Char('s'), .. }) => state.cam_pos += Rotation3::new(state.cam_dir) * Vector3::new(0.0, 0.0, -0.25),

            Event::Key(KeyEvent { code: KeyCode::Down, .. })  => state.rot[1] += 1.0 / 8.0 *  TAU, 
            Event::Key(KeyEvent { code: KeyCode::Up, .. })    => state.rot[1] += 1.0 / 8.0 * -TAU, 
            // Event::Key(KeyEvent { code: KeyCode::Up, .. })    => state.cam_dir.yz() = Rotation2::new(Vector2::x() * 1.0 / 8.0 * std::f32::consts::TAU) * state.cam_dir.yz(),
            // Event::Key(KeyEvent { code: KeyCode::Left, .. })  => state.cam_dir.yz() = Rotation2::new(Vector2::x() * 1.0 / 8.0 * std::f32::consts::TAU) * state.cam_dir.yz(),
            // Event::Key(KeyEvent { code: KeyCode::Right, .. }) => state.cam_dir.yz() = Rotation2::new(Vector2::x() * 1.0 / 8.0 * std::f32::consts::TAU) * state.cam_dir.yz(),
            _ => ()
        }
    }

    state.cam_dir = insert_x(state.cam_dir, rotate(state.rot[0]) * state.cam_dir.yz());
    state.cam_dir = insert_y(state.cam_dir, rotate(state.rot[1]) * state.cam_dir.xz());
    state.rot = Vector2::default();

    Ok(())
}

fn rotate(v: f64) -> Matrix2<f64> {
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
