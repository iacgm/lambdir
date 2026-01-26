use lambdir::*;

const TEST_DIR: &'static str = "test_gen";

#[test]
fn serdes() {
    #[allow(non_snake_case)]
    let Y: Combinator = combinator!(S S (S (S (K S) K)) (K (S (S K K) (S K K))));
    let test_cases: &[Combinator] = &[
        combinator!(Y (K K)), // ~~> K
    ];

    for case in test_cases {
        let mut case = case.clone();
        case.sk_ify();
        gen_fs(TEST_DIR, &case).expect("Could not generate dir.");
        assert_eq!(&case, &read_fs(TEST_DIR).unwrap());
    }

    let _ = std::fs::remove_dir_all(TEST_DIR);
}
