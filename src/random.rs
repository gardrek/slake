use prng::Prng16;

use std::cell::RefCell;

thread_local! {
    pub static PRNG: RefCell<Prng16> = RefCell::new(Prng16::new(get_prng_seed()));
}

#[cfg(not(target_family = "wasm"))]
fn get_prng_seed() -> [u16; 2] {
    use std::time::SystemTime;

    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Duration since UNIX_EPOCH failed");

    let number0 = duration.as_secs();
    let number1 = duration.subsec_nanos() & 0xffff0000 >> 16;

    [number0 as u16, number1 as u16]
}

#[cfg(target_family = "wasm")]
fn get_prng_seed() -> [u16; 2] {
    let number0 = (crate::random() * 65536.0).floor();
    let number1 = (crate::random() * 65536.0).floor();

    [number0 as u16, number1 as u16]
}

pub fn get_u16() -> u16 {
    PRNG.with(|prng| prng.borrow_mut().next().unwrap())
}
