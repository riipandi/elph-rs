/// Fast deterministic hash to shorten long strings.
pub fn short_hash(s: &str) -> String {
    let mut h1: u32 = 0xdeadbeef;
    let mut h2: u32 = 0x41c6ce57;
    for ch in s.chars() {
        let c = ch as u32;
        h1 = h1.wrapping_mul(2654435761).wrapping_add(h1 ^ c);
        h2 = h2.wrapping_mul(1597334677).wrapping_add(h2 ^ c);
    }
    h1 = (h1 ^ (h1 >> 16)).wrapping_mul(2246822507) ^ (h2 ^ (h2 >> 13)).wrapping_mul(3266489909);
    h2 = (h2 ^ (h2 >> 16)).wrapping_mul(2246822507) ^ (h1 ^ (h1 >> 13)).wrapping_mul(3266489909);
    format!("{:x}{:x}", h2, h1)
}
