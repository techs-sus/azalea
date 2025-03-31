//! License compliant port of https://github.com/vence722/base122-go/blob/main/encoder.go
//! It is highly recommended you use the [`Base122OriginalEncoder`] over the [`Base122Writer`] as
//! the writer is not as tested as the original encoder.

use std::io::Write;

/* {
	0, -- null
	10, -- newline
	13, -- carriage return
	34, -- double quote
	SKIP! 38, -- ampersand, without this being illegal we have base123 (aura!!!)
	92, -- backslash
*/
const ILLEGAL_BYTES: [u8; 5] = [0, 10, 13, 34, 92];
const SHORTENED: u8 = 0b111;

#[derive(Eq, PartialEq, Debug)]
pub enum Error {
	EndOfStream,
}

pub struct Base122OriginalEncoder<'a> {
	data: &'a [u8],
	current_byte: usize,
	current_bit: u8,
}

impl<'a> Base122OriginalEncoder<'a> {
	pub const fn new(data: &'a [u8]) -> Self {
		Base122OriginalEncoder {
			data,
			current_byte: 0,
			current_bit: 0,
		}
	}

	fn next_7_bits(&mut self) -> Result<u8, Error> {
		if self.current_byte >= self.data.len() {
			return Err(Error::EndOfStream);
		}

		let first_byte = self.data[self.current_byte];

		let mut first_encoded_byte = 0;
		let next_bit = self.current_bit + 7;
		if self.current_bit < 8 {
			// extract the highest (7 - self.current_bit) bits of the first byte, and shift 1 bit to the right
			// e.g. if self.current_bit == 6, 0b11111111 ---> 0b01100000
			first_encoded_byte = ((0b11111110 >> self.current_bit) & first_byte) << self.current_bit >> 1;
		}

		// no need to encode the second byte, return the first byte
		if next_bit < 8 {
			self.current_bit = next_bit;
			return Ok(first_encoded_byte);
		};

		// encode the next byte
		self.current_byte += 1;
		self.current_bit = next_bit - 8;

		if self.current_byte >= self.data.len() {
			return Ok(first_encoded_byte);
		}

		let second_byte = self.data[self.current_byte];
		let bits_to_move = 8 - self.current_bit;
		let second_encoded_byte = if bits_to_move < 8 {
			(((0b11111111 >> bits_to_move) << bits_to_move) & second_byte) >> bits_to_move
		} else {
			0
		};
		Ok(first_encoded_byte | second_encoded_byte)
	}

	pub fn encode(&mut self) -> Result<Vec<u8>, Error> {
		let mut out_data = Vec::with_capacity(self.data.len() * 2);

		while let Ok(first_seven_bits_byte) = self.next_7_bits() {
			// check if the byte hits the illegal bytes,
			// and get the index of first byte (which is the illegal byte)
			// inside the illegal byte list
			// e.g new line \r (10) --> 1
			let illegal_byte_index = ILLEGAL_BYTES
				.iter()
				.position(|&illegal_byte| illegal_byte == first_seven_bits_byte);
			if illegal_byte_index.is_none() {
				// if no hit, just save the single 7-bits byte into the result
				out_data.push(first_seven_bits_byte);
				continue;
			}

			/* logic below is if the byte is illegal */

			let illegal_byte_index: u8 = illegal_byte_index
				.unwrap()
				.try_into()
				.expect("to truncate illegal byte index into u8");

			let mut first_encoded_byte: u8 = 0b11000010;
			let mut second_encoded_byte: u8 = 0b10000000;

			// if the byte hits the illegal bytes, need to encode it
			// into 2-bytes format together with the next 7-bits byte
			let mut second_seven_bits_byte = match self.next_7_bits() {
				Err(Error::EndOfStream) => None,
				Ok(byte) => Some(byte),
			};

			// hasNextByte
			if second_seven_bits_byte.is_some() {
				first_encoded_byte |= (0b00000111 & illegal_byte_index) << 2;
			} else {
				first_encoded_byte |= SHORTENED << 2;
				second_seven_bits_byte = Some(first_seven_bits_byte); // encode the first 7-bits byte into the last byte, since we have no next byte
			};

			let second_seven_bits_byte: u8 = second_seven_bits_byte.unwrap();

			// put the first bit into the first byte to encode
			first_encoded_byte |= (0b01000000 & second_seven_bits_byte) >> 6;
			// put the rest 6 bits into the second byte to encode
			second_encoded_byte |= 0b00111111 & second_seven_bits_byte;

			out_data.push(first_encoded_byte);
			out_data.push(second_encoded_byte);
		}

		Ok(out_data)
	}
}

struct Base122Writer<W: Write> {
	inner: W,
	current_byte: usize,
	current_bit: u8,
	buffer: Vec<u8>,
}

impl<W: Write> Base122Writer<W> {
	pub const fn new(writer: W) -> Self {
		Self {
			inner: writer,
			current_byte: 0,
			current_bit: 0,
			buffer: Vec::new(),
		}
	}

	fn next_7_bits(&mut self, data: &[u8]) -> Result<u8, Error> {
		if self.current_byte >= data.len() {
			return Err(Error::EndOfStream); // Indicate end of input
		}

		let first_byte = data[self.current_byte];
		let mut first_encoded_byte = 0;
		let next_bit = self.current_bit + 7;
		if self.current_bit < 8 {
			// extract the highest (7 - self.current_bit) bits of the first byte, and shift 1 bit to the right
			// e.g. if self.current_bit == 6, 0b11111111 ---> 0b01100000
			first_encoded_byte = ((0b11111110 >> self.current_bit) & first_byte) << self.current_bit >> 1;
		}

		// no need to encode the second byte, return the first byte
		if next_bit < 8 {
			self.current_bit = next_bit;
			return Ok(first_encoded_byte);
		};

		// encode the next byte
		self.current_byte += 1;
		self.current_bit = next_bit - 8;

		if self.current_byte >= data.len() {
			return Ok(first_encoded_byte);
		}

		let second_byte = data[self.current_byte];
		let bits_to_move = 8 - self.current_bit;
		let second_encoded_byte = if bits_to_move < 8 {
			(((0b11111111 >> bits_to_move) << bits_to_move) & second_byte) >> bits_to_move
		} else {
			0
		};

		Ok(first_encoded_byte | second_encoded_byte)
	}

	fn encode_chunk(&mut self, data: &[u8]) -> Result<(), Error> {
		// reset current_byte and current_bit for each chunk
		self.current_byte = 0;
		// do not reset self.current_bit or the implementation is not sound

		while self.current_byte < data.len() {
			let first_seven_bits_byte = match self.next_7_bits(data) {
				Ok(byte) => byte,
				Err(Error::EndOfStream) => break,
			};

			// check if the byte hits the illegal bytes,
			// and get the index of the first byte (which is the illegal byte)
			// inside the illegal byte list
			let illegal_byte_index = ILLEGAL_BYTES
				.iter()
				.position(|&illegal_byte| illegal_byte == first_seven_bits_byte);

			if illegal_byte_index.is_none() {
				// if no hit, just save the single 7-bits byte into the result
				self.buffer.push(first_seven_bits_byte);
				continue;
			}

			let illegal_byte_index: u8 = illegal_byte_index
				.unwrap()
				.try_into()
				.expect("to truncate illegal byte index into u8");

			let mut first_encoded_byte: u8 = 0b11000010;
			let mut second_encoded_byte: u8 = 0b10000000;

			// if the byte hits the illegal bytes, encode it
			// into 2-bytes format together with the next 7-bits byte
			let mut second_seven_bits_byte = match self.next_7_bits(data) {
				Err(Error::EndOfStream) => None,
				Ok(byte) => Some(byte),
			};

			// has next byte
			if second_seven_bits_byte.is_some() {
				first_encoded_byte |= (0b00000111 & illegal_byte_index) << 2;
			} else {
				first_encoded_byte |= SHORTENED << 2;
				second_seven_bits_byte = Some(first_seven_bits_byte); // encode the first 7-bits byte into the last byte since we have no next byte
			};

			let second_seven_bits_byte: u8 = second_seven_bits_byte.unwrap();

			// put the first bit into the first byte to encode
			first_encoded_byte |= (0b01000000 & second_seven_bits_byte) >> 6;
			// put the rest 6 bits into the second byte to encode
			second_encoded_byte |= 0b00111111 & second_seven_bits_byte;

			self.buffer.push(first_encoded_byte);
			self.buffer.push(second_encoded_byte);
		}

		Ok(())
	}

	pub const fn get_inner_writer(&self) -> &W {
		&self.inner
	}
}

impl<W: Write> Write for Base122Writer<W> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self
			.encode_chunk(buf)
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e:?}")))?;

		self.inner.write_all(&self.buffer)?;
		self.buffer.clear();
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.inner.flush()
	}
}

macro_rules! generate_test {
	($original:expr, $encoded:expr, $name:ident) => {
		::concat_idents::concat_idents!(fn_name = original_, $name {
			#[test]
			fn fn_name() {
				assert_eq!(
					Base122OriginalEncoder::new($original).encode().unwrap(),
					$encoded
				);
			}
		});

		::concat_idents::concat_idents!(fn_name = writer_, $name {
			#[test]
			fn fn_name() {
				let mut encoded = Vec::new();
				{
					let mut writer = Base122Writer::new(&mut encoded);
					writer.write_all($original).unwrap();
					writer.flush().unwrap();
				}
				assert_eq!(encoded, $encoded);
			}
		});
	};
}

// TODO: Implement fuzzing with either cargo-fuzz (libfuzzer_sys) or AFL (american fuzzy lop)
// TODO: When we fuzz, we should compare the resulting bytes from the original implementation and
// the Writer implementation
mod tests {
	use super::*;

	generate_test!(
		b"hello world",
		vec![52, 25, 45, 70, 99, 60, 64, 119, 55, 211, 141, 70, 32],
		encode_hello_world
	);

	generate_test!(
		b"very very very very very very very very very very very very very very very very long text!!!",
		vec![
			59, 25, 46, 39, 73, 1, 108, 101, 57, 30, 36, 7, 51, 21, 100, 121, 16, 29, 76, 87, 19, 100,
			64, 118, 50, 211, 143, 18, 3, 89, 74, 114, 60, 72, 14, 102, 43, 73, 114, 32, 59, 25, 46, 39,
			73, 1, 108, 101, 57, 30, 36, 7, 51, 21, 100, 121, 16, 29, 76, 87, 19, 100, 64, 118, 50, 211,
			143, 18, 3, 89, 74, 114, 60, 72, 14, 102, 43, 73, 114, 32, 59, 25, 46, 39, 73, 1, 108, 101,
			57, 30, 36, 6, 99, 61, 211, 167, 16, 29, 12, 87, 67, 80, 66, 33, 16, 64
		],
		encode_very_long_text
	);

	generate_test!(&[0, 0], vec![194, 128, 222, 128], encode_two_zeros);
	generate_test!(&[0, 0, 0], vec![194, 128, 194, 128], encode_three_zeros);
	generate_test!(
		&[0, 0, 0, 0],
		vec![194, 128, 194, 128, 222, 128],
		encode_four_zeros
	);
	generate_test!(
		&[0, 0, 0, 0, 0],
		vec![194, 128, 194, 128, 194, 128],
		encode_five_zeros
	);

	generate_test!(
		&ILLEGAL_BYTES,
		vec![194, 130, 65, 82, 18, 112],
		encode_illegal_bytes
	);

	#[test]
	fn writer_ensure_correctness() {
		let mut encoded = Vec::new();
		let mut writer = Base122Writer::new(&mut encoded);
		writer.write_all(&[0]).unwrap();
		assert_eq!(**writer.get_inner_writer(), vec![194, 128]);
		writer.write_all(&[0]).unwrap();
		assert_eq!(**writer.get_inner_writer(), vec![194, 128, 222, 128]);
	}
}
