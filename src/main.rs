mod file_mapping;
mod manifest;

fn main() {
    match manifest::Manifest::new_from_file(String::from("hermione.yml")) {
        Ok(manifest) => {
            for mapping in manifest.mappings {
                println!("{}", mapping.display_line());
            }
        }
        Err(e) => eprintln!("{}", e.to_string()),
    }
}
