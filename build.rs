use std::fs;

fn main() {
    let paths = vec!["target/release", "target/debug"];

    for path in paths {
        let dir = std::path::Path::new(path);
        if !dir.exists() || !dir.is_dir() {
            fs::create_dir_all(dir).unwrap();
        }
        let target = dir.join("config.json");
        fs::copy("./config.json", target).unwrap();
    }
}
