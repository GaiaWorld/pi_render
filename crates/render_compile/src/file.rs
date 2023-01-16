use std::{fs::DirEntry, path::Path};


pub fn visit_dirs<F: FnMut(&DirEntry), P: AsRef<Path>>(path: P, cb: &mut F) -> std::io::Result<()> {
	let path = path.as_ref();
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}