use crate::serial_println;

pub fn dummy_driver() {
	serial_println!("Hello from dummy");
	loop { }
}