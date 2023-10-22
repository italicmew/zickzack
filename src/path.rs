use rand::{thread_rng, distributions::Alphanumeric, Rng};

const SEGMENT_LENGTH: usize = 10;
const NUMBER_OF_SEGMENTS: usize = 3;

fn generate_random_path_segment() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SEGMENT_LENGTH)
        .map(char::from)
        .collect()
}

pub fn generate_random_path() -> String {
    let segments: Vec<String> = (0..NUMBER_OF_SEGMENTS)
        .map(|_| generate_random_path_segment())
        .collect();

    segments.join("/")
}