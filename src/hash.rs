const START: u64 = 5431;
const MULT: u64 = 847209285431;
const SHIFT: u64 = 5;

pub fn hash(text: &str) -> u64 {
    let mut hash: u64 = START;
    for byte in text.bytes() {
        hash = (hash << SHIFT)
            .wrapping_add(hash)
            .wrapping_mul(MULT)
            .wrapping_add(byte as u64);
    }
    hash = hash
        .wrapping_add(hash)
        .wrapping_mul(MULT);

    hash
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_use_hash() {
        let hello = hash("hello");
        dbg!(hello);

        let hellp = hash("hellp");
        dbg!(hellp);

        let herro = hash("herro");
        dbg!(herro);
    }

    #[test]
    fn test_smoke() {
        let test_use_hash = "let out smoke test";
        let out = hash(test_use_hash);
        println!("{out}");
    }

    #[test]
    fn test_u8_wrapping() {
        let number: u8 = 255;
        let number: u8 = number.wrapping_add(1);
    }
}
