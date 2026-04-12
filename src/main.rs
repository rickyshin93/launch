mod config;

fn main() {
    config::ensure_dirs().expect("Failed to create launch directories");
    println!("launch CLI - scaffold OK");
}
