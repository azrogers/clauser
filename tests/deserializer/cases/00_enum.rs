use clauser_macros::duplicate_keys;

#[duplicate_keys]
enum Test {
    #[from_duplicate_key]
    Hello,
    World,
}

fn main() {}
