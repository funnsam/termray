use nalgebra::base::*;
use crossterm::{*, style::{Color, Stylize}, event::*};
use std::io::{stdout, Write};
use std::time::*;
use std::f64::consts::TAU;

pub const SCREENSHOT_SIZE: usize = 512;
pub const SCREENSHOT_SAMPLES: usize = 8;

pub static mut LOGS: Vec<(SystemTime, String)> = Vec::new();

pub fn push_log(s: &str) {
    unsafe {
        let t = SystemTime::now();
        LOGS.push((t, s.to_string()));
    }
}

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

pub fn push_image(image: Vec<Vec<(u8, u8, u8)>>, msg: &str) -> core::result::Result<(), Box<dyn std::error::Error>> {
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
        style::PrintStyledContent(
            msg .with(Color::White)
                .on  (Color::DarkGrey)
        )
    )?;

    so.flush()?;

    Ok(())
}

pub fn handle_input(state: &mut crate::renderer::RendererState, el: Duration) -> core::result::Result<bool, Box<dyn std::error::Error>> {
    let pr = poll(
        Duration::from_millis(
            30_u128.checked_sub(el.as_millis()).unwrap_or(0) as u64
        )
    )?;
    if pr {
        match read()? {
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                unsafe {
                    let mut s = String::new();

                    for i in &LOGS {
                        s += &format!("{:.02}s ago: {}\n", i.0.elapsed()?.as_secs_f64(), i.1);
                    }

                    std::fs::write("logs.txt", s)?;
                }

                prep_exit()?;
                std::process::exit(0);
            },
            Event::Key(KeyEvent { code: KeyCode::F(12), kind: KeyEventKind::Press, .. }) => {
                use std::path::Path;
                use std::fs::File;
                use std::io::BufWriter;

                let mut so = stdout();
                show(&mut so, "Rendering...")?;
                push_log("Start render");

                let path = Path::new(r"image_out.png");
                let file = File::create(path)?;
                let ref mut w = BufWriter::new(file);

                let mut encoder = png::Encoder::new(w, SCREENSHOT_SIZE as u32, SCREENSHOT_SIZE as u32);
                encoder.set_color(png::ColorType::Rgb);
                encoder.set_depth(png::BitDepth::Eight);

                let mut writer = encoder.write_header().unwrap();
                let mut buf = Vec::with_capacity(SCREENSHOT_SIZE * SCREENSHOT_SIZE * 3);

                let mut img = vec![vec![Vector3::default(); SCREENSHOT_SIZE]; SCREENSHOT_SIZE];
                for i in 1..SCREENSHOT_SAMPLES {
                    show(&mut so, &format!("Rendering sample {i}"))?;
                    crate::renderer::render(state, SCREENSHOT_SIZE, &mut img, i);
                }
                show(&mut so, &format!("Rendering last sample"))?;
                let img = crate::renderer::render(state, SCREENSHOT_SIZE, &mut img, SCREENSHOT_SAMPLES);

                for y in img.iter() {
                    for x in y.iter() {
                        buf.push(x.0);
                        buf.push(x.1);
                        buf.push(x.2);
                    }
                }
                writer.write_image_data(&buf)?;
                push_log("End render");
            },
  
            Event::Key(KeyEvent { code: KeyCode::Char('a'), kind: KeyEventKind::Press, .. }) => state.cam_pos += crate::renderer::rotate(Vector3::x() *  0.1225, state.rot),
            Event::Key(KeyEvent { code: KeyCode::Char('d'), kind: KeyEventKind::Press, .. }) => state.cam_pos += crate::renderer::rotate(Vector3::x() * -0.1225, state.rot),
            Event::Key(KeyEvent { code: KeyCode::Char('q'), kind: KeyEventKind::Press, .. }) => state.cam_pos += crate::renderer::rotate(Vector3::y() *  0.1225, state.rot), 
            Event::Key(KeyEvent { code: KeyCode::Char('e'), kind: KeyEventKind::Press, .. }) => state.cam_pos += crate::renderer::rotate(Vector3::y() * -0.1225, state.rot), 
            Event::Key(KeyEvent { code: KeyCode::Char('w'), kind: KeyEventKind::Press, .. }) => state.cam_pos += crate::renderer::rotate(Vector3::z() *  0.1225, state.rot),
            Event::Key(KeyEvent { code: KeyCode::Char('s'), kind: KeyEventKind::Press, .. }) => state.cam_pos += crate::renderer::rotate(Vector3::z() * -0.1225, state.rot),

            Event::Key(KeyEvent { code: KeyCode::Down , kind: KeyEventKind::Press, .. }) => state.rot[0] += 1.0 / 16.0 * TAU, 
            Event::Key(KeyEvent { code: KeyCode::Up   , kind: KeyEventKind::Press, .. }) => state.rot[0] -= 1.0 / 16.0 * TAU, 
            Event::Key(KeyEvent { code: KeyCode::Right, kind: KeyEventKind::Press, .. }) => state.rot[1] += 1.0 / 16.0 * TAU, 
            Event::Key(KeyEvent { code: KeyCode::Left , kind: KeyEventKind::Press, .. }) => state.rot[1] -= 1.0 / 16.0 * TAU, 

            Event::Key(KeyEvent { code: KeyCode::Home, kind: KeyEventKind::Press, .. }) => state.focus += 0.125,
            Event::Key(KeyEvent { code: KeyCode::End , kind: KeyEventKind::Press, .. }) => state.focus -= 0.125,
            Event::Key(KeyEvent { code: KeyCode::Backspace, kind: KeyEventKind::Press, .. }) => {
                let r = crate::renderer::Ray::new(state.cam_pos, crate::renderer::rotate(Vector3::z(), state.rot));
                let h = r.try_hit(&state.scene);
                if h.is_some() {
                    let (h, _) = h.unwrap();
                    state.focus = h.t
                }
            },
            Event::Key(KeyEvent { code: KeyCode::PageUp  , kind: KeyEventKind::Press, .. }) => state.aperture += 0.25,
            Event::Key(KeyEvent { code: KeyCode::PageDown, kind: KeyEventKind::Press, .. }) => state.aperture -= 0.25,
            _ => ()
        }
    }

    Ok(pr)
}

fn show(so: &mut std::io::Stdout, s: &str) -> core::result::Result<(), Box<dyn std::error::Error>> {
    execute!(so,
        cursor::MoveTo(0, 0),
        style::PrintStyledContent(s
            .with(Color::White)
            .on  (Color::DarkGrey)
        )
    )?;
    Ok(())
}
