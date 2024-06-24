use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use clauser::{error::Error, value::Value};

fn collect_files(dir: &Path, dirs: &mut Vec<DirEntry>) -> std::io::Result<()> {
    let dir: PathBuf = PathBuf::from("tests/mods/data/").join(dir);

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, dirs)?;
            } else if let Some(ext) = path.extension()
                && ext != "md"
            {
                dirs.push(entry);
            }
        }
    }
    Ok(())
}

fn all_files(dir: &Path) -> std::io::Result<Vec<DirEntry>> {
    let mut v = Vec::new();
    collect_files(dir, &mut v)?;
    Ok(v)
}

#[test]
pub fn the_great_war() -> Result<(), Error> {
    let files = all_files(Path::new("the_great_war")).unwrap();
    for f in files {
        let body = fs::read_to_string(f.path()).unwrap();
        Value::from_str(&body)?;
    }

    Ok(())
}
