use std::error::Error;
use std::env;

fn main() -> Result<(), Box<dyn Error>> {
	let args = env::args().skip(1).collect::<Vec<_>>();
	for arg in args {
		let binary_value = parse(&arg.replace(' ', ""))?;
		println!("binary: {:032b}", binary_value);
		let transmuted = f32::from_bits(binary_value); // clippy was smart enough to notice the transmute to a f32
		println!("transmuted: {:?}", transmuted);
		println!("interpreted: {:?}", interpret_single(binary_value));
	}
	Ok(())
}

fn parse(binary_number: &str) -> Result<u32, Box<dyn Error>> {
	if binary_number.len() != 32 {
		return Err("argument not 32 bits".into());
	}
	let mut value: u32 = 0;
	for (idx, digit) in binary_number.chars().enumerate() {
		match digit {
			'0' => {},
			'1' => value += 1 << (31-idx),
			x => return Err(format!("argument contains more than 0s and 1s: {:?}", x).into())
		}
	}
	Ok(value)
}

#[test]
fn parse_test() {
	assert_eq!(parse("10110000111111110101010110101010").unwrap(), 0b10110000111111110101010110101010);
}

fn interpret_single(float: u32) -> f32 {
	let sign = float.rotate_left(1) & 1;
	let sign = if sign == 1 { -1.0 } else { 0.0 };
	let characteristic = float.rotate_left(9) & 0b1111_1111;
	let mantissa = float & 0b1111_1111_1111_1111_1111_111;

	if characteristic == 0 {
		if mantissa != 0 {
			eprintln!("can't process denormalized floats (yet)");
			return 0.0;
		} else {
			return sign * 0.0;
		}
	} else if characteristic == 255 {
		if mantissa != 0 {
			return f32::NAN;
		} else {
			return sign * f32::INFINITY;
		}
	}

	let mut power = 2.0f32.powi(characteristic as i32 - 127);
	let mut value = sign * power;
	for idx in 0..23 {
		let digit = (mantissa >> (22 - idx)) & 1;
		power /= 2.0;
		if digit == 1 {
			value += sign * power;
		}
	}
	value
}

#[test]
fn interpret_single_test() {
	let tests = [
		(0b1100_0000_1110_1000_0000_0000_0000_0000, -7.25),
		(0b1011_1111_0101_0000_0000_0000_0000_0000, -0.8125)
	];
	for test in &tests {
		assert_eq!(interpret_single(test.0), test.1);
	}
}
