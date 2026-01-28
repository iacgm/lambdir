use lambdir::*;

const TEST_DIR: &'static str = "test_gen";

#[test]
#[allow(non_snake_case)]
fn isomorphism() {
    const LIMIT: usize = 1000;

    let B: Combinator = combinator!(S (K S) K);
    let C: Combinator = combinator!(S (S (K B) S) (K K));
    let W: Combinator = combinator!(S S (S K));

    let Y: Combinator = combinator!(S S (S (S (K S) K)) (K (S (S K K) (S K K))));

    let test_cases: &[Combinator] = &[
        combinator!(S S S S),        // ~~> SS(SS)
        combinator!(K K K),          // ~~> K
        combinator!(B K K K),        // ~~> K (K K)
        combinator!(W S K K S),      // ~~> S
        combinator!(C K K K),        // ~~> K K
        combinator!(Y (K K)),        // ~~> K
        combinator!(S (K S) K K K),  // ~~> S (K (S K)) K
        combinator!(+ 2 4),          // ~~> 6
        combinator!(= (+ 2 2) 4),    // ~~> K
        combinator!(2 K K K K),      // ~~> K
        combinator!((+ 4 4) (+1) 8), // ~~> 12
        combinator!(T 2 1 2 K),      // ~~> 1
    ];

    for case in test_cases {
        println!("Case: {}", case);
        let expected = case
            .normal_form(LIMIT)
            .expect("Execution did not terminate in time");
        println!("  --> {}\n", expected);

        gen_fs(TEST_DIR, case).expect("Could not generate directory.");
        exec_fs(TEST_DIR);
        let outcome = read_fs(TEST_DIR).expect("Unable to read dir.");
        assert_eq!(outcome, expected, "{} / {}", outcome, expected);
    }

    let _ = std::fs::remove_dir_all(TEST_DIR);
}
