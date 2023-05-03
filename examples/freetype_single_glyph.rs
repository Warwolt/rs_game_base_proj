extern crate freetype as ft;

const WIDTH: i32 = 32;
const HEIGHT: i32 = 24;

fn draw_bitmap(bitmap: ft::Bitmap, x: i32, y: i32) -> [[u8; WIDTH as usize]; HEIGHT as usize] {
    let mut figure = [[0; WIDTH as usize]; HEIGHT as usize];
    let mut p = 0;
    let mut q = 0;
    let w = bitmap.width();
    let x_max = x + w;
    let y_max = y + bitmap.rows();

    for i in x..x_max {
        for j in y..y_max {
            if 0 <= i && i < WIDTH && 0 <= j && j < HEIGHT {
                figure[j as usize][i as usize] |= bitmap.buffer()[(q * w + p) as usize];
                q += 1;
            }
        }
        q = 0;
        p += 1;
    }
    figure
}

fn main() {
    let ref font = "./resources/font/arial.ttf";
    let character = b'A' as usize;
    let library = ft::Library::init().unwrap();
    let face = library.new_face(font, 0).unwrap();

    face.set_char_size(40 * 64, 0, 50, 0).unwrap();
    face.load_char(character, ft::face::LoadFlag::RENDER)
        .unwrap();

    let glyph = face.glyph();
    let x = glyph.bitmap_left();
    let y = HEIGHT - glyph.bitmap_top();
    let figure = draw_bitmap(glyph.bitmap(), x, y);

    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            print!(
                "{}",
                match figure[i as usize][j as usize] {
                    p if p == 0 => " ",
                    p if p < 128 => "*",
                    _ => "+",
                }
            );
        }
        println!("");
    }
}
