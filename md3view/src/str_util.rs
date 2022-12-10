use std::{
	borrow::Cow,
	mem
};

pub trait StringFromBytes {
	/// Convert a byte slice to a string, starting at the first valid character,
	/// and stopping at the first invalid character.
	fn from_utf8_stop(bytes: &[u8]) -> Cow<'_, str>;
}

impl StringFromBytes for String {
	fn from_utf8_stop(bytes: &[u8]) -> Cow<'_, str> {
		// TODO: Use Utf8Chunks API when it's stable
		let valid = |b: &u8| b.is_ascii() && !b.is_ascii_control();
		let first_valid = bytes.iter().position(valid);
		if let None = first_valid { return Cow::Borrowed(""); }
		let first_valid = first_valid.unwrap();
		let first_invalid = bytes.iter().skip(first_valid).position(|b| !valid(b));
		let last_valid = match first_invalid {
			Some(first_invalid) => first_valid + first_invalid,
			None => bytes.len(),
		};
		let valid_slice = unsafe { mem::transmute(&bytes[first_valid..last_valid]) };
		Cow::Borrowed(valid_slice)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn start() {
		let bytes = b"YEE HAW\0\0\0\0\0\0\0\0\0";
		let expected = "YEE HAW";
		let actual = String::from_utf8_stop(bytes);
		assert_eq!(expected, actual)
	}

	#[test]
	fn middle() {
		let bytes = b"\0\0\0\0YEE HAW\0\0\0\0\0";
		let expected = "YEE HAW";
		let actual = String::from_utf8_stop(bytes);
		assert_eq!(expected, actual)
	}

	#[test]
	fn ending() {
		let bytes = b"\0\0\0\0\0\0\0\0\0YEE HAW";
		let expected = "YEE HAW";
		let actual = String::from_utf8_stop(bytes);
		assert_eq!(expected, actual)
	}

	#[test]
	fn nothing() {
		let bytes = b"\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
		let expected = "";
		let actual = String::from_utf8_stop(bytes);
		assert_eq!(expected, actual)
	}
}
