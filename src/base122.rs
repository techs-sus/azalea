//! License compliant port of <https://github.com/vence722/base122-go/blob/main/encoder.go>.
//! A writer once lived here, but it was not implemented correctly.

/*
	0, -- null
	10, -- newline
	13, -- carriage return
	34, -- double quote
	(skipped) 38, -- ampersand, without this being illegal we have base123
	92, -- backslash
*/

/// While this implementation does conform to Base122, we use allow ampersands for optimal usage with a Lua double quoted string.
pub const ILLEGAL_BYTES: [u8; 5] = [0, 10, 13, 34, 92];
const SHORTENED: u8 = 0b111;

/// You will probably never encounter this enum when using the encoder.
#[derive(Eq, PartialEq, Debug)]
pub enum Error {
	EndOfStream,
}

/// Fully conformant Base122 encoder, [`ILLEGAL_BYTES`] can be customized to change behaviour / increase encoder efficency.
pub struct Base122OriginalEncoder<'data> {
	data: &'data [u8],
	current_byte: usize,
	current_bit: u8,
}

impl<'data> Base122OriginalEncoder<'data> {
	pub const fn new(data: &'data [u8]) -> Self {
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

	/// Consumes and encodes the data stored in [`self`].
	/// This method consumes self as the Encoder is designed to be single use, and multiple invocations of encode lead to incorrect output.
	#[must_use]
	pub fn encode(mut self) -> Vec<u8> {
		// 1.75 * self.data.len()
		let mut output = Vec::with_capacity((7 * self.data.len()) / 4);

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
				output.push(first_seven_bits_byte);
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
			// put the rest of the 6 bits into the second byte to encode
			second_encoded_byte |= 0b00111111 & second_seven_bits_byte;

			output.push(first_encoded_byte);
			output.push(second_encoded_byte);
		}

		output
	}
}

/// A more concise version of [`Base122OriginalEncoder::encode`].
#[must_use]
pub fn encode(data: &[u8]) -> Vec<u8> {
	Base122OriginalEncoder::new(data).encode()
}

mod tests {
	use super::*;

	/// Allows you to write a simple expected output test for the encoder.
	///
	/// Example:
	/// ```ignore
	/// generate_test!(&[], vec![], empty_vectors_alike);
	/// ```
	macro_rules! generate_test {
		($original:expr, $encoded:expr, $name:ident) => {
			#[test]
			fn $name() {
				assert_eq!(encode($original), $encoded);
			}
		};
	}

	generate_test!(&[], &[], empty_vectors_alike);

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

	generate_test!(
		&[
			0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
			26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
			49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71,
			72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94,
			95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113,
			114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131,
			132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149,
			150, 151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165, 166, 167,
			168, 169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182, 183, 184, 185,
			186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203,
			204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221,
			222, 223, 224, 225, 226, 227, 228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239,
			240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255
		],
		&[
			194, 128, 32, 32, 24, 16, 198, 134, 3, 66, 1, 16, 80, 44, 24, 202, 135, 3, 98, 1, 8, 72, 38,
			20, 199, 133, 66, 113, 64, 100, 52, 27, 14, 7, 35, 97, 121, 195, 130, 206, 145, 73, 4, 82,
			49, 28, 80, 41, 21, 199, 165, 66, 105, 56, 94, 48, 24, 76, 70, 51, 33, 84, 108, 55, 28, 14,
			39, 35, 89, 112, 122, 62, 31, 80, 8, 20, 18, 202, 136, 69, 35, 17, 105, 4, 74, 41, 22, 76,
			38, 83, 73, 117, 2, 69, 36, 83, 42, 21, 42, 101, 58, 97, 50, 90, 45, 87, 11, 85, 114, 125,
			64, 97, 49, 24, 108, 70, 43, 25, 78, 104, 52, 90, 77, 54, 99, 53, 211, 175, 56, 28, 46, 39,
			27, 81, 106, 118, 59, 94, 15, 23, 83, 109, 120, 125, 63, 31, 112, 8, 12, 198, 135, 4, 66, 97,
			80, 120, 68, 38, 21, 11, 70, 35, 49, 104, 124, 66, 35, 18, 73, 101, 18, 89, 52, 94, 49, 25,
			77, 38, 115, 73, 108, 122, 63, 32, 80, 104, 84, 58, 37, 22, 77, 39, 84, 42, 53, 42, 93, 50,
			91, 46, 87, 108, 22, 27, 21, 78, 105, 53, 91, 45, 119, 11, 77, 106, 119, 60, 94, 111, 87,
			124, 6, 7, 5, 67, 98, 49, 56, 108, 62, 35, 19, 74, 101, 115, 25, 211, 182, 63, 33, 81, 105,
			52, 122, 77, 46, 91, 47, 88, 108, 118, 91, 61, 102, 119, 61, 95, 112, 56, 60, 46, 31, 19, 75,
			102, 115, 122, 29, 30, 87, 47, 89, 109, 119, 59, 126, 15, 15, 75, 103, 116, 122, 125, 94,
			127, 71, 103, 117, 123, 126, 63, 63, 111, 120
		],
		encode_all_ascii_bytes
	);
}
