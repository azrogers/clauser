use clauser_macros::duplicate_keys;

struct CantDeser {
    val: i32,
}

#[duplicate_keys]
struct Test {
    #[from_duplicate_key]
    item: Vec<CantDeser>,
}

fn main() {}
