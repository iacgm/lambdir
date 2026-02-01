use lambdir::*;

const DIR: &'static str = "fib_dir";

#[allow(non_snake_case)]
fn main() {
    let I: Combinator = combinator!(S K K);
    let B: Combinator = combinator!(S (K S) K);
    let V: Combinator = combinator!(B (S I) K);

    // [a, b] -> [b, a + b]
    let recurse = combinator! {
        S
            (B (T 2) (V (K I)))       // [a,b] -> T 2 b
            (S (B + (V K)) (V (K I))) // [a,b] -> a + b
    };
    // n -> fib n
    let iter = combinator! {
        S (V recurse) (K (T 2 0 1))
    };

    let fib = combinator! {
        V K (iter 10)
    };


    let now = std::time::Instant::now();
    let normalized = fib.normal_form(None).unwrap();
    let time = std::time::Instant::now().duration_since(now).as_secs_f32();
    println!("Returned {} in {}s", get_name(&ls_dir(DIR)[0]), time);
}
