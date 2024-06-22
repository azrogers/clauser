use clauser_macros::duplicate_keys;

#[duplicate_keys]
struct Test {
    #[from_duplicate_key]
    item: Option<String>,
}

fn main() {}
