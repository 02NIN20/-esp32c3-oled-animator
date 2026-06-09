use std::{
    fs,
    io::{stdout, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

const ANIM_RS: &str = "../../firmware/src/anim_frames.rs";
const WIDTH: usize = 72;
const HEIGHT: usize = 40;
const FRAME_BYTES: usize = 360;

fn main() -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    let frames = parse_frames(ANIM_RS)?;
    println!("Frames cargados: {}", frames.len());
    if frames.is_empty() {
        println!("⚠️  No se encontraron frames. Revisa la ruta y el formato del archivo.");
        return Ok(());
    }
    println!("Presiona Ctrl+C para detener");
    print!("\x1b[?25l");
    stdout().flush()?;
    let frame_duration = Duration::from_millis(66);

    while running.load(Ordering::SeqCst) {
        for frame in &frames {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            print!("\x1b[H");
            render_frame(frame);
            stdout().flush()?;
            thread::sleep(frame_duration);
        }
    }

    print!("\x1b[?25h");
    print!("\x1b[2J");
    stdout().flush()?;
    Ok(())
}

fn parse_frames(filepath: &str) -> anyhow::Result<Vec<[u8; FRAME_BYTES]>> {
    let text = fs::read_to_string(filepath)?;
    println!("Archivo leído, {} caracteres", text.len());

    let marker = "= &[";
    let idx = text.find(marker)
        .ok_or_else(|| anyhow::anyhow!("No se encontró '= &[' en el archivo"))?;
    println!("Marker encontrado en la posición {}", idx);

    let start = text[idx + marker.len() - 1..]
        .find('[')
        .map(|p| idx + marker.len() - 1 + p)
        .ok_or_else(|| anyhow::anyhow!("No se encontró '[' tras '= &'"))?;
    println!("Inicio del array exterior (start): {}", start);

    let mut count = 0;
    let mut end = start;
    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate().skip(start) {
        if c == '[' { count += 1; }
        else if c == ']' { count -= 1; if count == 0 { end = i; break; } }
    }
    println!("Fin del array exterior (end): {}", end);
    let content = &text[start + 1..end];
    println!("Contenido entre corchetes: {} caracteres", content.len());
    // Mostrar los primeros 200 caracteres para depurar
    let preview = if content.len() > 200 { &content[..200] } else { content };
    println!("Inicio del contenido:\n{}", preview);

    let mut frames = Vec::new();
    let mut current: Vec<u8> = Vec::with_capacity(FRAME_BYTES);
    let chars: Vec<char> = content.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '[' => {
                current.clear();
                i += 1;
            }
            ']' => {
                if current.len() == FRAME_BYTES {
                    let mut arr = [0u8; FRAME_BYTES];
                    arr.copy_from_slice(&current);
                    frames.push(arr);
                } else if !current.is_empty() {
                    eprintln!("⚠️  Frame con {} bytes", current.len());
                }
                current.clear();
                i += 1;
            }
            '0' if i + 1 < chars.len() && chars[i + 1] == 'x' => {
                i += 2;
                let mut hex_str = String::new();
                while i < chars.len() && chars[i].is_ascii_hexdigit() {
                    hex_str.push(chars[i]);
                    i += 1;
                }
                if let Ok(b) = u8::from_str_radix(&hex_str, 16) {
                    current.push(b);
                }
            }
            _ => i += 1,
        }
    }
    Ok(frames)
}

fn render_frame(frame: &[u8; FRAME_BYTES]) {
    let mut output = String::with_capacity(WIDTH * HEIGHT * 4);
    for y in 0..HEIGHT {
        for x_byte in 0..(WIDTH / 8) {
            let byte = frame[y * (WIDTH / 8) + x_byte];
            for bit in (0..8).rev() {
                if (byte >> bit) & 1 == 1 {
                    output.push('█');
                    output.push('█');
                } else {
                    output.push(' ');
                    output.push(' ');
                }
            }
        }
    }
    print!("{}", output);
}
