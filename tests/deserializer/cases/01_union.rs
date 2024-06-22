use clauser_macros::duplicate_keys;

#[duplicate_keys]
union test {
    item: i32,
    other: f32,
}

fn main() {}
