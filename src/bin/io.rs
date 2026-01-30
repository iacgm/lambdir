use lambdir::*;

const TEST_DIR: &'static str = "test_gen";

#[allow(non_snake_case)]
fn main() {
    let comb = combinator!(? (S ! K)); // ~~> K; echoes back n

    gen_fs(TEST_DIR, &comb).expect("Could not create directory");
    exec_fs(TEST_DIR);
    let _ = std::fs::remove_dir_all(TEST_DIR);
}
