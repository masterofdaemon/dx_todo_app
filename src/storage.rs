use crate::models::Todo;
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

fn data_file_path() -> io::Result<PathBuf> {
    let proj = ProjectDirs::from("com", "dx", "dx_todo_app")
        .ok_or_else(|| io::Error::other("unable to get project dirs"))?;
    let dir = proj.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir.join("todos.json"))
}

pub fn load_todos() -> Vec<Todo> {
    if let Ok(path) = data_file_path() {
        if let Ok(mut f) = File::open(path) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(list) = serde_json::from_str::<Vec<Todo>>(&s) {
                    return list;
                }
            }
        }
    }
    Vec::new()
}

pub fn save_todos(todos: &[Todo]) {
    if let Ok(path) = data_file_path() {
        if let Ok(mut f) = File::create(path) {
            if let Ok(s) = serde_json::to_string_pretty(todos) {
                let _ = f.write_all(s.as_bytes());
            }
        }
    }
}
