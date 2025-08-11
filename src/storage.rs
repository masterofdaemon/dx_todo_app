use crate::models::{Project, Todo};
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

fn todos_file_path() -> io::Result<PathBuf> {
    let proj = ProjectDirs::from("com", "dx", "dx_todo_app")
        .ok_or_else(|| io::Error::other("unable to get project dirs"))?;
    let dir = proj.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir.join("todos.json"))
}

fn projects_file_path() -> io::Result<PathBuf> {
    let proj = ProjectDirs::from("com", "dx", "dx_todo_app")
        .ok_or_else(|| io::Error::other("unable to get project dirs"))?;
    let dir = proj.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir.join("projects.json"))
}

pub fn load_or_migrate_projects() -> Vec<Project> {
    // Preferred: projects.json
    if let Ok(path) = projects_file_path() {
        if let Ok(mut f) = File::open(&path) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(list) = serde_json::from_str::<Vec<Project>>(&s) {
                    return list;
                }
            }
        }
    }

    // Migration: wrap old todos.json into a Default Project
    if let Ok(tpath) = todos_file_path() {
        if let Ok(mut f) = File::open(&tpath) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(todos) = serde_json::from_str::<Vec<Todo>>(&s) {
                    let projects = vec![Project { id: 1, name: "Default Project".into(), todos }];
                    save_projects(&projects);
                    return projects;
                }
            }
        }
    }
    Vec::new()
}

pub fn save_projects(projects: &[Project]) {
    if let Ok(path) = projects_file_path() {
        if let Ok(mut f) = File::create(path) {
            if let Ok(s) = serde_json::to_string_pretty(projects) {
                let _ = f.write_all(s.as_bytes());
            }
        }
    }
}
