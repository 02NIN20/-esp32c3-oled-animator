use std::{
    fs::File,
    io::{BufWriter, Write, Read},   // <- añadido Read
    path::Path,
    process::{Command, Stdio},
};

use anyhow::Context;

const WIDTH: u32 = 72;
const HEIGHT: u32 = 40;
const FRAME_BYTES: usize = (WIDTH * HEIGHT / 8) as usize; // 360

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: {} <video.mp4> [fps]", args[0]);
        return Ok(());
    }
    let input = &args[1];
    let fps: f32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(30.0);

    if !Path::new(input).exists() {
        anyhow::bail!("El archivo {} no existe.", input);
    }

    // Llamar a ffmpeg
    let mut child = Command::new("ffmpeg")
        .args([
            "-i", input,
            "-f", "rawvideo",
            "-pix_fmt", "gray",
            "-s", &format!("{}x{}", WIDTH, HEIGHT),
            "-r", &fps.to_string(),
            "-",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let stdout = child.stdout.take()
        .context("No se pudo capturar la salida de ffmpeg")?;
    let mut reader = std::io::BufReader::new(stdout);

    let mut frames: Vec<[u8; FRAME_BYTES]> = Vec::new();
    // Línea corregida (sin paréntesis extra)
    let mut gray_frame = vec![0u8; WIDTH as usize * HEIGHT as usize];

    loop {
        match reader.read_exact(&mut gray_frame) {
            Ok(()) => {
                dither_floyd_steinberg(&mut gray_frame);
                let packed = pack_frame(&gray_frame);
                frames.push(packed);
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => anyhow::bail!("Error leyendo frames: {}", e),
        }
    }
    child.wait()?;

    println!("Frames extraídos: {}", frames.len());
    if frames.is_empty() {
        anyhow::bail!("No se obtuvo ningún frame. Revisa el vídeo.");
    }

    write_rust_module(&frames, "anim_frames.rs")?;
    println!("Archivo 'anim_frames.rs' generado correctamente.");
    Ok(())
}

fn dither_floyd_steinberg(gray: &mut [u8]) {
    let w = WIDTH as usize;
    let h = HEIGHT as usize;

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let old_pixel = gray[idx] as f32;
            let new_pixel = if old_pixel > 127.0 { 255.0 } else { 0.0 };
            gray[idx] = new_pixel as u8;
            let error = old_pixel - new_pixel;

            // Difundir error a los vecinos (Floyd-Steinberg)
            if x + 1 < w {
                gray[idx + 1] = (gray[idx + 1] as f32 + error * 7.0 / 16.0)
                    .round().clamp(0.0, 255.0) as u8;
            }
            if y + 1 < h {
                if x > 0 {
                    gray[idx + w - 1] = (gray[idx + w - 1] as f32 + error * 3.0 / 16.0)
                        .round().clamp(0.0, 255.0) as u8;
                }
                gray[idx + w] = (gray[idx + w] as f32 + error * 5.0 / 16.0)
                    .round().clamp(0.0, 255.0) as u8;
                if x + 1 < w {
                    gray[idx + w + 1] = (gray[idx + w + 1] as f32 + error * 1.0 / 16.0)
                        .round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
}

fn pack_frame(gray: &[u8]) -> [u8; FRAME_BYTES] {
    let mut packed = [0u8; FRAME_BYTES];
    let mut byte_idx = 0;
    for row in 0..HEIGHT as usize {
        for col_byte in 0..(WIDTH as usize / 8) {
            let mut byte_val = 0u8;
            for bit in 0..8 {
                let x = col_byte * 8 + bit;
                let pixel = gray[row * WIDTH as usize + x];
                if pixel > 128 {
                    byte_val |= 1 << (7 - bit);
                }
            }
            packed[byte_idx] = byte_val;
            byte_idx += 1;
        }
    }
    packed
}

fn write_rust_module(frames: &[[u8; FRAME_BYTES]], filename: &str) -> anyhow::Result<()> {
    let mut file = BufWriter::new(File::create(filename)?);
    writeln!(file, "// Generado automáticamente por video_to_anim")?;
    writeln!(file, "// Resolución: {}x{}, {} frames", WIDTH, HEIGHT, frames.len())?;
    writeln!(file)?;
    writeln!(file, "pub const FRAME_COUNT: usize = {};", frames.len())?;
    writeln!(file, "pub const FRAME_BYTES: usize = {};", FRAME_BYTES)?;
    writeln!(file, "pub static FRAMES: &[[u8; FRAME_BYTES]] = &[")?;
    for frame in frames.iter() {
        write!(file, "    [")?;
        for (j, byte) in frame.iter().enumerate() {
            write!(file, "0x{:02X},", byte)?;
            if j % 12 == 11 {
                writeln!(file)?;
                write!(file, "     ")?;
            }
        }
        writeln!(file, "],")?;
    }
    writeln!(file, "];")?;
    Ok(())
}
