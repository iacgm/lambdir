use lambdir::*;

const TEST_DIR: &'static str = "test_gen";

#[test]
fn serdes() {
    #[allow(non_snake_case)]
    let test_cases: &[Combinator] = &[
        combinator!(Y (K K)), // ~~> K
    ];

    for case in test_cases {
        let mut case = case.clone();
        case.simplify();
        gen_fs(TEST_DIR, &case).expect("Could not generate dir.");
        assert_eq!(&case, &read_fs(TEST_DIR).unwrap());
    }

    let _ = std::fs::remove_dir_all(TEST_DIR);
}
