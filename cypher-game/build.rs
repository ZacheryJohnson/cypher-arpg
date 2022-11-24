use std::path::PathBuf;

fn main() {
    {
        let mut dir_path = std::env::current_dir().unwrap();
        dir_path.push("assets");
        dir_path.push("game_data");
        let _ = std::fs::create_dir(dir_path);
    }

    {
        let mut from_path = std::env::current_dir().unwrap();
        from_path.push("..");
        from_path.push("cypher-core");
        from_path.push("data");

        println!("cargo:rerun-if-changed={}", from_path.to_str().unwrap());

        from_path.push("affix.json");

        let mut to_path = std::env::current_dir().unwrap();
        to_path.push("assets");
        to_path.push("game_data");
        to_path.push("affix.json");

        copy_data(from_path, to_path);
    }

    {
        let mut from_path = std::env::current_dir().unwrap();
        from_path.push("..");
        from_path.push("cypher-core");
        from_path.push("data");

        println!("cargo:rerun-if-changed={}", from_path.to_str().unwrap());

        from_path.push("affix_pool.json");

        let mut to_path = std::env::current_dir().unwrap();
        to_path.push("assets");
        to_path.push("game_data");
        to_path.push("affix_pool.json");

        copy_data(from_path, to_path);
    }

    {
        let mut from_path = std::env::current_dir().unwrap();
        from_path.push("..");
        from_path.push("cypher-item");
        from_path.push("data");

        println!("cargo:rerun-if-changed={}", from_path.to_str().unwrap());

        from_path.push("item.json");

        let mut to_path = std::env::current_dir().unwrap();
        to_path.push("assets");
        to_path.push("game_data");
        to_path.push("item.json");

        copy_data(from_path, to_path);
    }

    {
        let mut from_path = std::env::current_dir().unwrap();
        from_path.push("..");
        from_path.push("cypher-item");
        from_path.push("data");

        println!("cargo:rerun-if-changed={}", from_path.to_str().unwrap());

        from_path.push("loot_pool.json");

        let mut to_path = std::env::current_dir().unwrap();
        to_path.push("assets");
        to_path.push("game_data");
        to_path.push("loot_pool.json");

        copy_data(from_path, to_path);
    }
}

fn copy_data(from: PathBuf, to: PathBuf) {
    std::fs::copy(from, to).unwrap();
}
