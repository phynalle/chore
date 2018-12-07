use std::env;
use std::path::{Component, Path, PathBuf};

pub fn normalize<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let path: &Path = path.as_ref();
    let components = path.components();

    if path.is_relative() {
        let mut cur = env::current_dir().ok()?;
        for comp in components {
            match comp {
                Component::ParentDir => {
                    if !cur.pop() {
                        return None;
                    }
                }
                Component::CurDir => continue,
                Component::Normal(s) => cur.push(s),
                _ => unreachable!(),
            }
        }
        Some(cur)
    } else {
        Some(path.to_path_buf())
    }
}
