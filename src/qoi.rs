use std::convert::TryInto;
use std::vec;
use std::io;
use std::io::prelude::*;
use std::fs::File;

const QOI_OP_INDEX: u8 = 0x00; /* 00xxxxxx */
const QOI_OP_DIFF: u8  = 0x40; /* 01xxxxxx */
const QOI_OP_LUMA: u8  = 0x80; /* 10xxxxxx */
const QOI_OP_RUN: u8   = 0xc0; /* 11xxxxxx */
const QOI_OP_RGB: u8   = 0xfe; /* 11111110 */
const QOI_OP_RGBA: u8  = 0xff; /* 11111111 */

#[derive(Debug, Clone, PartialEq)]
pub enum QoiChannels {
  UNK, RGB, RGBA
}

#[derive(Debug, Clone, PartialEq)]
pub enum QoiColorspace {
  UNK, SRGB, LINEAR
}

#[derive(Debug, Clone, PartialEq)]
pub struct QoiPixel {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
  pub alpha: u8,
}

impl QoiPixel {

  pub fn empty() -> QoiPixel
  {
    QoiPixel { red: 0, green: 0, blue: 0, alpha: 255 }
  }

  pub fn clear() -> QoiPixel
  {
    QoiPixel { red: 0, green: 0, blue: 0, alpha: 0 }
  }

  pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> QoiPixel
  {
    QoiPixel { red: red, green: green, blue: blue, alpha: alpha }
  }

  pub fn hash(&self) -> usize
  {
    (self.red as usize * 3 + self.green as usize * 5 + self.blue as usize * 7 + self.alpha as usize * 11) % 64
  }
}

#[derive(Debug, Clone)]
pub struct QoiFile {
  pub encoded: Vec<u8>,
  pub decoded: Vec<QoiPixel>,
  pub width: u32,
  pub height: u32,
  pub channels: QoiChannels,
  pub colorspace: QoiColorspace,
}

impl QoiFile
{
  pub fn load_from_file(filename: &str) -> io::Result<QoiFile>
  {
    let mut bytes = Vec::new();
    let mut f = File::open(filename)?;

    f.read_to_end(&mut bytes)?;

    Ok(QoiFile { encoded: bytes, decoded: Vec::new(), width: 0, height: 0, channels: QoiChannels::RGB, colorspace: QoiColorspace::LINEAR })
  }

  pub fn get(&self, x: usize, y: usize) -> QoiPixel
  {
    if self.decoded.len() <= y * (self.width as usize) + x {
      return QoiPixel::empty();
    }
    self.decoded[y * (self.width as usize) + x].clone()
  }

  pub fn encode(mut self) -> QoiFile
  {
    self.encoded = Vec::new();

    self.encoded.push('q' as u8);
    self.encoded.push('o' as u8);
    self.encoded.push('i' as u8);
    self.encoded.push('f' as u8);

    self.encoded.append(&mut write_32(self.width));
    self.encoded.append(&mut write_32(self.height));
    self.encoded.push(match self.channels {
      QoiChannels::RGB => 3,
      QoiChannels::RGBA => 4,
      QoiChannels::UNK => 1,
    });

    self.encoded.push(match self.colorspace {
      QoiColorspace::SRGB => 0,
      QoiColorspace::LINEAR => 1,
      QoiColorspace::UNK => 2,
    });

    let mut running_array = vec![QoiPixel::clear(); 64];
    let mut previous_pixel = QoiPixel::empty();
    let mut run_length: u8 = 0;
    for pixel in &self.decoded
    {
     // println!("[{}] {}, {}, {}, {} -- {}, {}, {}, {}", self.encoded.len(), pixel.red, pixel.green, pixel.blue, pixel.alpha, previous_pixel.red, previous_pixel.green, previous_pixel.blue, pixel.alpha);
      if pixel.clone() == previous_pixel {
        run_length += 1;
        if run_length == 62 {
          //println!("run {}", run_length);
          self.encoded.push(QOI_OP_RUN | (run_length - 1));
          run_length = 0;
        }
      }
      else
      {
        if run_length > 0
        {
          //println!("run {}", run_length);
          self.encoded.push(QOI_OP_RUN | (run_length - 1));
          run_length = 0;
        }

        if running_array[pixel.hash()] == pixel.clone()
        {
          self.encoded.push(QOI_OP_INDEX | pixel.hash() as u8);
        }
        else
        {
          running_array[pixel.hash()] = pixel.clone();

          if pixel.alpha == previous_pixel.alpha
          {
            let vr: i32 = sub(pixel.red, previous_pixel.red);
            let vg: i32 = sub(pixel.green, previous_pixel.green);
            let vb: i32 = sub(pixel.blue, previous_pixel.blue);
  
            let vg_r: i32 = vr - vg;
            let vg_b: i32 = vb - vg;

            //println!("[{}] {}, {}, {} -- {}, {}", self.encoded.len(), vr, vg, vb, vg_r, vg_b);

            if vr > -3 && vr < 2 && vg > -3 && vg < 2 && vb > -3 && vb < 2
            {
              //println!("diff");
              self.encoded.push(QOI_OP_DIFF | (((vr + 2) as u8) << 4) | (((vg + 2) as u8) << 2) | (vb + 2) as u8);
            }
            else if vg_r > -9 && vg_r < 8 && vg > -33 && vg < 32 && vg_b > -9 && vg_b < 8
            {
              //println!("luma");
              self.encoded.push(QOI_OP_LUMA | (vg + 32) as u8);
              self.encoded.push((((vg_r + 8) as u8) << 4) | (vg_b + 8) as u8);
            }
            else
            {
              //println!("rgb");
              self.encoded.push(QOI_OP_RGB);
              self.encoded.push(pixel.red);
              self.encoded.push(pixel.green);
              self.encoded.push(pixel.blue);
            }
          }
          else
          {
            //println!("rgba");
            self.encoded.push(QOI_OP_RGBA);
            self.encoded.push(pixel.red);
            self.encoded.push(pixel.green);
            self.encoded.push(pixel.blue);
            self.encoded.push(pixel.alpha);
          }
        }
        
      }

      previous_pixel = pixel.clone();
    }

    if run_length > 0 {
      self.encoded.push(QOI_OP_RUN | (run_length - 1));
    }

    self.encoded.push(0);
    self.encoded.push(0);
    self.encoded.push(0);
    self.encoded.push(0);
    self.encoded.push(0);
    self.encoded.push(0);
    self.encoded.push(0);
    self.encoded.push(1);

    self
  }

  pub fn decode(mut self) -> QoiFile
  {
    // check that we have enough bytes for the header at least
    if self.encoded.len() < 14 {
      panic!("Not enough bytes for a valid qoi file :(");
    }

    let magic = match std::str::from_utf8(&self.encoded[0..4]) {
      Ok(s) => s,
      Err(e) => panic!("Invalid UTF-8 sequence '{:?}': {}", &self.encoded[0..4], e)
    };

    if magic != "qoif" {
      panic!("Expected magic number 'qoif' but got '{}' :(", magic)
    }

    self.width = u32::from_be_bytes((&self.encoded[4..8]).try_into().expect("slice with incorrect length"));
    self.height = u32::from_be_bytes((&self.encoded[8..12]).try_into().expect("slice with incorrect length"));
    self.channels = match &self.encoded[12] {
      3 => QoiChannels::RGB,
      4 => QoiChannels::RGBA,
      _ => QoiChannels::UNK
    };
    self.colorspace = match &self.encoded[13] {
      0 => QoiColorspace::SRGB,
      1 => QoiColorspace::LINEAR,
      _ => QoiColorspace::UNK
    };

    self.decoded = Vec::with_capacity(self.width as usize * self.height as usize);
  
    let mut running_array = vec![QoiPixel::clear(); 64];
    let mut previous_pixel = QoiPixel::empty();

    let mut i = 14;
    while i < self.encoded.len() - 8 {
      let current_byte = self.encoded[i];
      let current_pixel: QoiPixel;
      let mut run_length: u8 = 1;

      if is_op_rgb(current_byte) {
        let next_bytes = &self.encoded[(i + 1)..(i + 4)];
        i += 3;
        current_pixel = QoiPixel::new(next_bytes[0], next_bytes[1], next_bytes[2], previous_pixel.alpha);
      } else if is_op_rgba(current_byte) {
        let next_bytes = &self.encoded[(i + 1)..(i + 5)];
        i += 4;
        current_pixel = QoiPixel::new(next_bytes[0], next_bytes[1], next_bytes[2], next_bytes[3]);
      } else if is_op_index(current_byte) {
        current_pixel = running_array[current_byte as usize].clone();
      } else if is_op_diff(current_byte) {
        let dr = sub((current_byte & 48) >> 4, 2) as u8;
        let dg = sub((current_byte & 12) >> 2, 2) as u8;
        let db = sub(current_byte & 3, 2) as u8;
        current_pixel = QoiPixel::new(add(previous_pixel.red, dr), add(previous_pixel.green, dg), add(previous_pixel.blue, db), previous_pixel.alpha);
      } else if is_op_luma(current_byte) {
        let diff_green = sub(current_byte & 63, 32) as u8;
        let next_byte = &self.encoded[i + 1];
        i += 1;
        let drdg = (next_byte >> 4) & 15;
        let dbdg = next_byte & 15;
        current_pixel = QoiPixel::new(add3(previous_pixel.red, sub(drdg, 8) as u8, diff_green), add(previous_pixel.green, diff_green), add3(previous_pixel.blue, sub(dbdg, 8) as u8, diff_green), previous_pixel.alpha);
      } else if is_op_run(current_byte) {
        current_pixel = previous_pixel.clone();
        run_length = (current_byte & 63) + 1;
      } else {
        panic!("Unexpected byte {:?}", current_byte);
      }

      running_array[current_pixel.hash()] = current_pixel.clone();

      while run_length > 0 {
        self.decoded.push(current_pixel.clone());
        run_length -= 1;
      }

      previous_pixel = current_pixel.clone();
      i += 1;
    }

    let padding = [0, 0, 0, 0, 0, 0, 0, 1];
    while i < self.encoded.len() {
      if self.encoded[i] != padding[i + 8 - self.encoded.len()] 
      {
        panic!("There is an error in the padding :(");
      };
      i += 1;
    }

    self
  }
}

// helper functions 
fn add(a: u8, b: u8) -> u8
{
  (((a as u32) + (b as u32)) % 256) as u8
}

fn add3(a: u8, b: u8, c: u8) -> u8
{
  (((a as u32) + (b as u32) + (c as u32)) % 256) as u8
}

fn sub(a: u8, b: u8) -> i32
{
  ((a as i32) - (b as i32)) % 256
}

fn is_op_rgb(byte: u8) -> bool
{
  byte == 254
}

fn is_op_rgba(byte: u8) -> bool
{
  byte == 255
}

fn is_op_index(byte: u8) -> bool
{
  (byte & 192) == 0
}

fn is_op_diff(byte: u8) -> bool
{
  (byte & 192) == 64
}

fn is_op_luma(byte: u8) -> bool
{
  (byte & 192) == 128
}

fn is_op_run(byte: u8) -> bool
{
  (byte & 192) == 192
}


fn write_32(v: u32) -> Vec<u8>
{
  let mut split = Vec::new();

  split.push(((0xff000000 & v) >> 24) as u8);
  split.push(((0x00ff0000 & v) >> 16) as u8);
  split.push(((0x0000ff00 & v) >> 8) as u8);
  split.push((0x000000ff & v) as u8);

  split
}
