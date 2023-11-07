use nalgebra::{vector, Vector2, Vector3};

const SAFE_HEIGHT: f32 = 17.0;
const WRIT_HEIGHT: f32 = 15.0;
const WIDTH: f32 = 12.5;
const HEIGHT: f32 = 14.0;
const POINTS: [Vector2<f32>; 9] = [
    vector![-5.0, -5.0],
    vector![-5.0, 0.0],
    vector![-5.0, 5.0],
    vector![0.0, 5.0],
    vector![5.0, 5.0],
    vector![5.0, 0.0],
    vector![5.0, -5.0],
    vector![0.0, -5.0],
    vector![0.0, 0.0],
];

const SEGMENTS: [[usize; 2]; 16] = [
    // Outer 0-7
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 4],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 0],
    // Inside 8-15
    [0, 8],
    [1, 8],
    [2, 8],
    [3, 8],
    [4, 8],
    [5, 8],
    [6, 8],
    [7, 8],
];

const LETTERS: [u16; 27] = [
    0b1111001100010001, // A
    0b1111110001010100, // B
    0b1100111100000000, // C
    0b1111110001000100, // D
    0b1100111100000001, // E
    0b1100001100000010, // F
    0b1101111100010000, // G
    0b0011001100010001, // H
    0b1100110001000100, // I
    0b0011111000000000, // J
    0b0000001100101001, // K
    0b0000111100000000, // L
    0b0011001110100000, // M
    0b0011001110001000, // N
    0b1111111100000000, // O
    0b1110001100010001, // P
    0b1111111100001000, // Q
    0b1110001100011001, // R
    0b1101110100010001, // S
    0b1100000001000100, // T
    0b0011111100000000, // U
    0b0000001100100010, // V
    0b0011001100001010, // W,
    0b0000000010101010, // X,
    0b0000000010100100, // Y,
    0b1100110000100010, // Z
    0b0000000000000010, // { (/)
];

fn parse_letter(c: u8) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();

    let segments = LETTERS[c as usize - b'a' as usize];
    for (i, &segment) in SEGMENTS.iter().enumerate() {
        if (segments << i) & 0x8000 == 0x8000 {
            let [p_0, p_1] = segment;
            let p_0 = POINTS[p_0];
            let p_1 = POINTS[p_1];

            locs.push(vector![p_0.x, p_0.y, SAFE_HEIGHT]);
            locs.push(vector![p_0.x, p_0.y, WRIT_HEIGHT]);
            locs.push(vector![p_1.x, p_1.y, WRIT_HEIGHT]);
            locs.push(vector![p_1.x, p_1.y, SAFE_HEIGHT]);
        }
    }

    locs
}

pub fn parse_signature(string: &str, pos: &Vector3<f32>) -> Vec<Vector3<f32>> {
    let mut locs = Vec::new();
    let mut cursor = *pos;

    for &c in string.as_bytes() {
        match c {
            b'\n' => {
                cursor.x += HEIGHT;
            }
            b'\r' => {
                cursor.y = pos.y;
            }
            b'\x08' => {
                // Backspace
                cursor.y -= WIDTH;
            }
            _ => {
                locs.extend(
                    parse_letter(c)
                        .iter()
                        .map(|v| cursor + vector![v.x, v.y, v.z]),
                );

                cursor.y += WIDTH;
            }
        }
    }

    locs
}
