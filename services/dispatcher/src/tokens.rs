use rand::{distributions::Alphanumeric, rngs::StdRng, Rng, SeedableRng};

pub fn alphanumeric(prefix: &str, len: usize) -> String {
    let random_part: String = StdRng::from_entropy()
        .sample_iter(&Alphanumeric)
        .take(len - prefix.len())
        .map(char::from)
        .collect();
    format!("{}{}", prefix, random_part)
}
