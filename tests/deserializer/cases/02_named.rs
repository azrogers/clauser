use clauser_macros::duplicate_keys;

#[duplicate_keys]
struct Test(Vec<String>);

fn main() {}
