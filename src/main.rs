use rppal::i2c::I2c;
use enum_repr::EnumRepr;
use std::fs::File;
use rgb::*;
use png::ColorType;

const SCREEN_ADDR: u16 = 0x3c;
const COMMAND: u8 = 0;
const SCREEN: u8 = 0x40;
const SCREEN_BUFFER_SIZE: usize = 128 * (32/8);
const IMAGE_PATH: &str = "test.png";
const IMAGE_SIZE: usize = 4096;
const WIDTH: usize = 128;
const HEIGHT: usize = 32;
const PAGES: usize = HEIGHT/8;

#[EnumRepr(type = "u8")]
enum SSD1306Commands {
    DisplayOff = 0xAE,
    SetDisplayClockDiv = 0xD5,
    SetMultiplex = 0xA8,
    SetDisplayOffset = 0xD3,
    SetStartLine = 0x40,
    ChargePump = 0x8D,
    MemoryMode = 0x20,
    SegRemap = 0xA0,
    ComScanDec = 0xC8,
    SetCompIns = 0xDA,
    SetContrast = 0x81,
    SetPreCharge = 0xD9,
    SetVComDetect = 0xD8,
    DisplayAllOnResume = 0xA4,
    NormalDisplay = 0xA6,
    DisplayOn = 0xAF,
    ColumnAddr = 0x21,
    PageAddr = 0x22,
}

fn init_display(i2c: &I2c) {
    i2c.block_write(COMMAND, &[SSD1306Commands::DisplayOff.repr()]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetDisplayClockDiv.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0x80]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetMultiplex.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0x1F]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetDisplayOffset.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetStartLine.repr()]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::ChargePump.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0x14]).unwrap(); // SwitchCapVcc
    i2c.block_write(COMMAND, &[SSD1306Commands::MemoryMode.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SegRemap.repr() | 1]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::ComScanDec.repr()]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetCompIns.repr()]).unwrap();
    i2c.block_write(COMMAND, &[2]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetContrast.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0x8F]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetPreCharge.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0xF1]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::SetVComDetect.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0x40]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::DisplayAllOnResume.repr()]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::NormalDisplay.repr()]).unwrap();
    i2c.block_write(COMMAND, &[SSD1306Commands::DisplayOn.repr()]).unwrap();
}

fn write_screen_buffer(i2c: &I2c, buf: &[u8]) {
    i2c.block_write(COMMAND, &[SSD1306Commands::ColumnAddr.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0]).unwrap();
    i2c.block_write(COMMAND, &[128-1]).unwrap(); // Width
    i2c.block_write(COMMAND, &[SSD1306Commands::PageAddr.repr()]).unwrap();
    i2c.block_write(COMMAND, &[0]).unwrap();
    i2c.block_write(COMMAND, &[(32/8)-1]).unwrap(); // Height
    for i in (0..SCREEN_BUFFER_SIZE).step_by(16) {
        i2c.block_write(SCREEN, &buf[i..i+16]).unwrap()
    }
}

fn main() {
    // Open I2C
    let mut i2c = I2c::new().unwrap();
    i2c.set_slave_address(SCREEN_ADDR).unwrap();

    // Initialize the Display
    init_display(&i2c);

    // Create Screen Buffer and blank display
    let mut screen_buffer = vec![0u8; SCREEN_BUFFER_SIZE];
    write_screen_buffer(&i2c, &screen_buffer);

    // Open PNG
    let dec = png::Decoder::new(File::open(IMAGE_PATH).unwrap());
    let (info, mut reader) = dec.read_info().unwrap();
    match (info.width, info.height) {
        (128, 32) => {}
        _ => panic!("Unexpected Image Res"),
    }
    let mut image_buffer = vec![0; info.buffer_size()];
    reader.next_frame(&mut image_buffer).unwrap();

    // Convert PNG to Screen Buffer
    match info.color_type {
        ColorType::RGB => {
            let pixels: &[RGB8] = image_buffer.as_rgb();
            let mut i = 0;
            for page in 0..PAGES {
                for column in 0..WIDTH {
                    let mut byte = 0;
                    for bit in 0..8 {
                        let p = pixels[(((7-bit) + page * 8) * WIDTH) + column];
                        if (p.r/3 + p.g/3 + p.b/3) > 127 {
                            byte |= 1 << (7-bit);
                        }
                    }
                    screen_buffer[i] = byte;
                    i += 1;
                }
            }
        }
        _ => panic!("Unsupported PNG Type"),
    }

    // Write Image
    write_screen_buffer(&i2c, &screen_buffer);
}
