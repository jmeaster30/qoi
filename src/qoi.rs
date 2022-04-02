use std::convert::TryInto;

/*
QOI Header

char[4] magic     <- 4 bytes
uint32 width      <- 4 bytes
uint32 height     <- 4 bytes
uint8 channels    <- 1 byte
uint8 colorspace  <- 1 byte

*/

#[derive(Debug, Clone)]
struct QoiEncodeError;

#[derive(Debug, Clone)]
struct QoiDecodeError;


pub fn qoi_encode(bytes: &[u8], width: u32, height: u32) -> &[u8]
{
  &[10]
}

pub fn qoi_decode(bytes: &[u8]) -> (u32, u32, &[u8])
{
  // check that we have enough bytes for the header at least
  if bytes.len() < 14 {
    panic!("Not enough bytes for a valid qoi file");
  }

  let magic = match std::str::from_utf8(&bytes[0..4]) {
    Ok(s) => s,
    Err(e) => panic!("Invalid UTF-8 sequence: {}", e)
  };

  let width = u32::from_be_bytes((&bytes[4..8]).try_into().expect("slice with incorrect length"));
  let height = u32::from_be_bytes((&bytes[8..12]).try_into().expect("slice with incorrect length"));
  let channels = &bytes[12];
  let colorspace = &bytes[13];

  println!("{}", magic);
  println!("{}", width);
  println!("{}", height);
  println!("{} - {}", channels, colorspace);

  (width, heightgit co, &[10])
}