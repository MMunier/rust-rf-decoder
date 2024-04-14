use std::path::{Path, PathBuf};

macro_rules! ensure_exists {
    ($e:expr) => {
        match $e {
            Ok(_e) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            e => e.unwrap(),
        }
    };
}

const TEST_OUT_PATH: &str = "debug/test-output";
pub fn create_output_directory(name: &str) -> PathBuf {
    let test_out_path = Path::new(TEST_OUT_PATH);
    let out_path = test_out_path.join(name);

    ensure_exists!(std::fs::create_dir(TEST_OUT_PATH));
    ensure_exists!(std::fs::create_dir(&out_path));
    out_path
}
