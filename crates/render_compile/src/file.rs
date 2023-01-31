use std::{fs::DirEntry, path::Path};

use crate::CompileShaderError;

pub fn visit_dirs<F: FnMut(&DirEntry) -> Result<(), CompileShaderError>, P: AsRef<Path>>(
    path: P,
    cb: &mut F,
) -> Result<(), CompileShaderError> {
    let path = path.as_ref();
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}
