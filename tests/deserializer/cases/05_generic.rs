use clauser_macros::duplicate_keys;

#[duplicate_keys]
struct Test<T> {
    #[from_duplicate_key]
    item: Vec<T>,
}

fn main() {}
