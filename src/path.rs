use std::path::{PathBuf, Path};

use rand::{thread_rng, distributions::Alphanumeric, Rng};

use crate::errs::EditorError;

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

pub fn sanitize_path(input: &str) -> Result<PathBuf, EditorError> {
    let base  = Path::new("./data");
    // Step 1: Remove problematic sequences
    let sanitized_input = input
        .replace("..", "")
        .replace("//", "/");

    // Step 2: Construct a path from the sanitized input
    let path = base.join(sanitized_input);

    // Step 3: Check if the path is still within the base directory
    if !path.starts_with(base) {
        return Err(EditorError::InvalidPath);
    }

    Ok(path)
}