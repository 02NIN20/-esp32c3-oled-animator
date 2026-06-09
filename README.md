# ESP32-C3 OLED Animator

Reproduce animaciones monocromáticas en un display OLED SSD1306 usando un ESP32-C3. Los frames se convierten desde cualquier video y se compilan directamente en la flash del microcontrolador.

## Requisitos

### Hardware

- ESP32-C3 (cualquier placa, ej. ESP32-C3-DevKitM-1)
- Display OLED SSD1306 72×40 píxeles (o 128×64, ajustando la resolución)
- Cable USB-C para programar
- Conexiones I2C:

| OLED | ESP32-C3 |
|------|----------|
| SDA  | GPIO 6   |
| SCL  | GPIO 5   |
| VCC  | 3.3V     |
| GND  | GND      |

### Software

- [Rust](https://rustup.rs/) con toolchain nightly
- Toolchain **riscv32imc-esp-espidf** para ESP32-C3:
  ```bash
  rustup toolchain install nightly --component rust-src
  cargo install espflash
  cargo install ldproxy
  ```
- **ffmpeg** (para convertir el video)
- Python 3 + Pillow (para el script alternativo `generate_frames.py`):
  ```bash
  pip install Pillow
  ```

## Estructura del proyecto

```
esp32c3-oled-animator/
├── firmware/                    # Crate del ESP32-C3
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs              # Loop de reproducción en el OLED
│   │   └── anim_frames.rs       # Frames de la animación (auto-generado)
│   ├── .cargo/config.toml       # Target RISC-V + runner
│   ├── rust-toolchain.toml      # nightly + rust-src
│   └── sdkconfig.defaults       # Configuración ESP-IDF
├── video_to_anim/               # Conversor video → frames (Rust)
│   ├── Cargo.toml
│   ├── src/main.rs
│   └── view_anim/               # Preview de la animación en PC
│       ├── Cargo.toml
│       └── src/main.rs
├── prebuilt/
│   └── bindings.rs              # Bindings correctos de esp-idf-sys
├── generate_frames.py           # Alternativa Python al conversor
├── Cargo.toml                   # Workspace raíz
├── Cargo.lock
└── README.md
```

## Cómo usar

### 1. Convertir un video a frames

**Opción A: Conversor Rust**
```bash
cd video_to_anim
cargo run --release -- video.mp4 20
```
Esto genera `firmware/src/anim_frames.rs` con los frames en formato monocromo.

**Opción B: Script Python**
```bash
python generate_frames.py video.mp4 20
```

Parámetros:
- Primer argumento: ruta al video
- Segundo argumento (opcional): FPS (por defecto 30, recomendado 20)
- Tercer argumento (opcional, solo en Python): máximo de frames

El conversor aplica **dithering Floyd-Steinberg** para mejorar la calidad visual al cuantizar a blanco y negro.

### 2. Preview en PC (opcional)

Antes de flashear, puedes ver la animación en la terminal:
```bash
cd video_to_anim/view_anim
cargo run --release
```
Presiona Ctrl+C para detener.

### 3. Compilar y flashear el ESP32-C3

Conecta el ESP32-C3 por USB y ejecuta:
```bash
cd firmware
cargo run --release
```
`cargo run` usa `espflash flash --monitor` automáticamente. Si solo quieres flashear sin monitorear:
```bash
espflash flash target/riscv32imc-esp-espidf/release/bad_apple_frames
```

### Nota sobre bindings de esp-idf-sys

Si el firmware no compila por errores como `no field "flags" on type spi_transaction_t`, los bindings generados por `esp-idf-sys` están corruptos. Solución:

```bash
for dir in firmware/target/riscv32imc-esp-espidf/{debug,release}/build/esp-idf-sys-*/out/; do
    cp prebuilt/bindings.rs "$dir/bindings.rs"
done
touch firmware/target/riscv32imc-esp-espidf/{debug,release}/.fingerprint/esp-idf-sys-*
cargo build --release
```

Los bindings correctos están en `prebuilt/bindings.rs`.

## Personalizar resolución

Edita en `video_to_anim/src/main.rs` (o `generate_frames.py`):
```rust
const WIDTH: u32 = 128;   // ancho en píxeles
const HEIGHT: u32 = 64;   // alto en píxeles
```

Y en `firmware/src/main.rs` cambia `DisplaySize72x40` por el tamaño correspondiente (ej. `DisplaySize128x64`).

También aplica para el preview en `video_to_anim/view_anim/src/main.rs`.

## Cómo funciona

### video_to_anim

1. **ffmpeg** extrae cada frame del video como grises a la resolución objetivo
2. **Floyd-Steinberg dithering** mejora la calidad visual al cuantizar a 1 bit
3. Cada frame se empaqueta en `WIDTH × HEIGHT / 8` bytes en formato monocromo horizontal
4. Se genera `anim_frames.rs` con todos los frames como un `static` array

### Firmware (ESP32-C3)

1. Inicializa el periférico I2C a 400KHz
2. Configura el OLED SSD1306 en modo gráfico con buffer
3. Reproduce en loop infinito: por cada frame, lo dibuja con `embedded-graphics` y lo envía al display
4. Espera `1000/FPS` ms entre frames

## Limitaciones

- Solo blanco y negro (1 bit por píxel)
- Los frames se almacenan en la flash del microcontrolador, no hay decodificación en tiempo real
- El tamaño máximo de animación depende de la flash disponible (4MB en ESP32-C3)

## Licencia

MIT
