#!/usr/bin/env python3
"""
Convierte un video a frames en formato de mapa de bits horizontal (72x40)
y genera src/anim_frames.rs listo para usar con ImageRaw.
Uso: python generate_frames.py video.mp4 [fps=10] [max_frames=0]
"""
import sys, os, subprocess, shutil
from PIL import Image

WIDTH, HEIGHT = 72, 40
# Cada fila de 72 píxeles se guarda en 9 bytes (72/8 = 9)
ROW_BYTES = (WIDTH + 7) // 8
FRAME_BYTES = ROW_BYTES * HEIGHT   # 9 * 40 = 360 bytes (¡coincide!)
TEMP_DIR = "temp_frames"

def extract_frames(video, fps=10):
    os.makedirs(TEMP_DIR, exist_ok=True)
    subprocess.run([
        "ffmpeg", "-i", video,
        "-vf", f"fps={fps},scale={WIDTH}:{HEIGHT},format=monob",
        "-q:v", "1", os.path.join(TEMP_DIR, "frame_%04d.png")
    ], check=True, capture_output=True)
    return sorted(f for f in os.listdir(TEMP_DIR) if f.endswith(".png"))

def png_to_buffer_horizontal(path):
    img = Image.open(path)  # Modo '1' (monocromo ya hecho por ffmpeg)
    buf = bytearray()
    for y in range(HEIGHT):
        row = 0
        for x in range(WIDTH):
            if img.getpixel((x, y)):    # True = blanco
                row |= 1 << (x % 8)     # bit 0 = píxel más a la izquierda del byte
            if (x % 8) == 7 or x == WIDTH - 1:
                buf.append(row)
                row = 0
    return buf

def main():
    if len(sys.argv) < 2:
        print("Uso: python generate_frames.py video.mp4 [fps=10] [max_frames=0]")
        sys.exit(1)

    video = sys.argv[1]
    fps = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    max_frames = int(sys.argv[3]) if len(sys.argv) > 3 else 0

    print(f"Extrayendo frames a {fps} fps...")
    files = extract_frames(video, fps)
    if max_frames > 0:
        files = files[:max_frames]

    print(f"Convirtiendo {len(files)} frames...")
    frames = []
    for i, fname in enumerate(files):
        buf = png_to_buffer_horizontal(os.path.join(TEMP_DIR, fname))
        frames.append(buf)

    print(f"Escribiendo src/anim_frames.rs...")
    os.makedirs("src", exist_ok=True)
    with open("src/anim_frames.rs", "w") as f:
        f.write("// Formato: mapa de bits horizontal, compatible con ImageRaw\n")
        f.write(f"pub const FRAME_COUNT: usize = {len(frames)};\n")
        f.write(f"pub const FRAME_BYTES: usize = {FRAME_BYTES};\n")
        f.write("pub static FRAMES: &[[u8; FRAME_BYTES]] = &[\n")
        for frame in frames:
            f.write("    [\n")
            for i in range(0, len(frame), 18):
                line = ", ".join(f"0x{b:02X}" for b in frame[i:i+18])
                f.write(f"        {line},\n")
            f.write("    ],\n")
        f.write("];\n")

    shutil.rmtree(TEMP_DIR)
    size_kb = (len(frames) * FRAME_BYTES) / 1024
    print(f"✅ Listo: {len(frames)} frames, {size_kb:.1f} KB en src/anim_frames.rs")

if __name__ == "__main__":
    main()
