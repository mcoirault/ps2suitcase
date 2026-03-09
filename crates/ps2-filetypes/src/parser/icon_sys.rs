use std::io::{Cursor, Read, Result};
use crate::color::Color;
use crate::sjis::{decode_sjis, encode_sjis};
use crate::util::parse_cstring;
use byteorder::{ReadBytesExt, LE};

#[derive(Clone, Copy, Debug)]
pub struct ColorF {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorF {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            f32::to_le_bytes(self.r),
            f32::to_le_bytes(self.g),
            f32::to_le_bytes(self.b),
            f32::to_le_bytes(self.a),
        ]
        .into_flattened()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector {
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![
            f32::to_le_bytes(self.x),
            f32::to_le_bytes(self.y),
            f32::to_le_bytes(self.z),
            f32::to_le_bytes(self.w),
        ]
        .into_flattened()
    }
}

/**
 * IconSys Flags
 * 00 -> PS2 Save File
 * 01 -> Software (PS2)
 * 02 -> unrecognized data
 * 03 -> Software (Pocketstation
 * 04 -> Settings (PS2)
 * 05 -> system driver
 * 06..20 -> unrecognized data
 *
 * those flags are available on most PS2 (excluding 05, which was implemented somewhere between 1.70 and 1.90 BIOS)
 *
 * Thanks israpps!
 */

#[derive(Clone, Debug)]
pub struct IconSys {
    pub flags: u16,
    pub linebreak_pos: u8,
    pub background_transparency: u32,
    pub background_colors: [Color; 4],
    pub light_directions: [Vector; 3],
    pub light_colors: [ColorF; 3],
    pub ambient_color: ColorF,
    pub title_line1: String,
    pub title_line2: String,
    pub icon_file: String,
    pub icon_copy_file: String,
    pub icon_delete_file: String,
}

impl IconSys {
    pub fn new(bytes: Vec<u8>) -> Self {
        parse_icon_sys(bytes).unwrap()
    }

    pub fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"PS2D");

        bytes.extend(self.flags.to_le_bytes());
        bytes.extend(convert_linepos(self.linebreak_pos).to_le_bytes());
        bytes.extend(0u8.to_le_bytes()); // Reserved
        bytes.extend(0u32.to_le_bytes()); // Reserved
        bytes.extend(self.background_transparency.to_le_bytes());

        for color in &self.background_colors {
            bytes.extend_from_slice(&color.to_bytes());
        }

        for direction in &self.light_directions {
            bytes.extend_from_slice(&direction.to_bytes());
        }

        for color in &self.light_colors {
            bytes.extend_from_slice(&color.to_bytes());
        }

        bytes.extend_from_slice(&self.ambient_color.to_bytes());

        let title_bytes = encode_sjis(&join_title_lines(self.title_line1.clone(), self.title_line2.clone()));
        let title_len = title_bytes.len();
        if title_len > 68 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Title length exceeds 68 bytes",
            ));
        }

        bytes.extend_from_slice(&title_bytes);
        if title_len < 68 {
            bytes.extend(vec![0; 68 - title_len]);
        }

        bytes.extend_from_slice(self.icon_file.as_bytes());

        if self.icon_file.len() < 64 {
            bytes.extend(vec![0; 64 - self.icon_file.len()]);
        }

        bytes.extend_from_slice(self.icon_copy_file.as_bytes());

        if self.icon_copy_file.len() < 64 {
            bytes.extend(vec![0; 64 - self.icon_copy_file.len()]);
        }

        bytes.extend_from_slice(self.icon_delete_file.as_bytes());

        if self.icon_delete_file.len() < 64 {
            bytes.extend(vec![0; 64 - self.icon_delete_file.len()]);
        }

        bytes.extend(vec![0; 512]);

        Ok(bytes)
    }
}

#[expect(unused)]
struct IconSysParser {
    c: Cursor<Vec<u8>>,
    len: usize,
}

fn parse_icon_sys(bytes: Vec<u8>) -> Result<IconSys> {
    let mut c = Cursor::new(bytes);

    let mut magic = vec![0u8; 4];
    c.read_exact(&mut magic)?;

    let flags = c.read_u16::<LE>()?;
    let linebreak_pos = convert_linepos(c.read_u8()?);
    _ = c.read_u8(); // Reserved, always 0
    _ = c.read_u32::<LE>(); // Reserved, always 0
    let background_transparency = c.read_u32::<LE>()?;

    let background_colors = [
        parse_color(&mut c)?,
        parse_color(&mut c)?,
        parse_color(&mut c)?,
        parse_color(&mut c)?,
    ];

    let light_directions = [
        parse_direction(&mut c)?,
        parse_direction(&mut c)?,
        parse_direction(&mut c)?,
    ];

    let light_colors = [
        parse_colorf(&mut c)?,
        parse_colorf(&mut c)?,
        parse_colorf(&mut c)?,
    ];

    let ambient_color = parse_colorf(&mut c)?;

    let mut title_buf = vec![0u8; 68];
    c.read_exact(&mut title_buf)?;

    let mut icon_file_buf = vec![0u8; 64];
    c.read_exact(&mut icon_file_buf)?;
    let mut icon_copy_file_buf = vec![0u8; 64];
    c.read_exact(&mut icon_copy_file_buf)?;
    let mut icon_delete_file_buf = vec![0u8; 64];
    c.read_exact(&mut icon_delete_file_buf)?;

    let title = parse_sjis_string(&title_buf);
    let (title_line1, title_line2) = split_title(linebreak_pos, title);

    Ok(IconSys {
        flags,
        linebreak_pos,
        background_transparency,
        background_colors,
        light_directions,
        light_colors,
        ambient_color,
        title_line1,
        title_line2,
        icon_file: parse_cstring(&icon_file_buf),
        icon_copy_file: parse_cstring(&icon_copy_file_buf),
        icon_delete_file: parse_cstring(&icon_delete_file_buf),
    })
}

fn parse_sjis_string(c: &[u8]) -> String {
    let title = decode_sjis(c);

    parse_cstring(&title.as_bytes())
}

fn parse_color(c: &mut Cursor<Vec<u8>>) -> Result<Color> {
    let r = c.read_u32::<LE>()? as u8;
    let g = c.read_u32::<LE>()? as u8;
    let b = c.read_u32::<LE>()? as u8;
    let a = c.read_u32::<LE>()? as u8;

    Ok(Color { r, g, b, a })
}

fn parse_colorf(c: &mut Cursor<Vec<u8>>) -> Result<ColorF> {
    let r = c.read_f32::<LE>()?;
    let g = c.read_f32::<LE>()?;
    let b = c.read_f32::<LE>()?;
    let a = c.read_f32::<LE>()?;

    Ok(ColorF { r, g, b, a })
}

fn parse_direction(c: &mut Cursor<Vec<u8>>) -> Result<Vector> {
    let x = c.read_f32::<LE>()?;
    let y = c.read_f32::<LE>()?;
    let z = c.read_f32::<LE>()?;
    let w = c.read_f32::<LE>()?;

    Ok(Vector { x, y, z, w })
}

/**
 * The linebreak is stored at byte 0x06 in the 4 lower bits, but with the least significant bit first.
 * For example a linebreak on the 5rd (0b 0000 0101) character will be represented as 0b 0000 1010.
 * The 4 higher bits are seemingly ignored in the context of OSDMENU.
 */
fn convert_linepos(source: u8) -> u8 {
    // zero out the 4 most significant bits
    let low_half = source & 0b00001111;
    low_half.reverse_bits() >> 4
}

fn split_title(linebreak_pos: u8, title: String) -> (String, String) {
    if linebreak_pos >= title.len() as u8 {
        return (title, "".to_string());
    }
    let (title_line1, title_line2) = title.split_at(linebreak_pos as usize);
    // trim the leading space on line2 necessary for proper linebreak
    (title_line1.into(), title_line2.trim_start().into())
}

fn join_title_lines(title_line1: String, title_line2: String) -> String {
    if title_line2.len() == 0 {
        return title_line1;
    }
    format!("{} {}", title_line1, title_line2)
}
