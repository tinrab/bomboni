use std::error::Error;
use std::ffi::OsStr;
use std::fs::{DirEntry, OpenOptions};
use std::io::Read;
use std::path::Path;
use std::{fs, io};

pub fn visit_files<P, E>(
    dir: P,
    extensions: &[&str],
    cb: &mut dyn FnMut(&DirEntry) -> Result<(), E>,
) -> Result<(), E>
where
    P: AsRef<Path>,
    E: Error + From<io::Error>,
{
    for entry in fs::read_dir(dir.as_ref())? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_files(&path, extensions, cb)?;
        } else {
            let ext = path.extension();
            if extensions
                .iter()
                .any(|expected| Some(OsStr::new(expected)) == ext)
            {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

pub fn visit_files_contents<P, E>(
    dir: P,
    extensions: &[&str],
    cb: &mut dyn FnMut(&DirEntry, String) -> Result<(), E>,
) -> Result<(), E>
where
    P: AsRef<Path>,
    E: Error + From<io::Error>,
{
    visit_files(dir, extensions, &mut |file| {
        let mut content = String::new();
        OpenOptions::new()
            .read(true)
            .open(file.path())?
            .read_to_string(&mut content)?;
        cb(file, content)
    })?;
    Ok(())
}
