use crate::models::{Project, Todo};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

#[cfg(not(target_os = "android"))]
use directories::ProjectDirs;

#[cfg(target_os = "android")]
use {
    jni::{objects::JObject, JavaVM},
    ndk_context::android_context,
};

#[cfg(target_os = "android")]
fn app_data_dir() -> io::Result<PathBuf> {
    // Use internal app files directory: Context.getFilesDir()
    unsafe {
        let ctx = android_context();
        let jvm = JavaVM::from_raw(ctx.vm().cast()).map_err(|e| io::Error::other(format!("jvm from raw: {e}")))?;
        let mut env = jvm
            .attach_current_thread()
            .map_err(|e| io::Error::other(format!("attach thread: {e}")))?;
        let activity = JObject::from_raw(ctx.context() as jni::sys::jobject);
        let file_obj: JObject = env
            .call_method(activity, "getFilesDir", "()Ljava/io/File;", &[])
            .and_then(|v| v.l())
            .map_err(|e| io::Error::other(format!("getFilesDir: {e}")))?;
        let path_obj: JObject = env
            .call_method(file_obj, "getAbsolutePath", "()Ljava/lang/String;", &[])
            .and_then(|v| v.l())
            .map_err(|e| io::Error::other(format!("getAbsolutePath: {e}")))?;
        let path: String = env
            .get_string(&jni::objects::JString::from(path_obj))
            .map_err(|e| io::Error::other(format!("jstring: {e}")))?
            .into();
        let dir = PathBuf::from(path).join("data");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }
}

#[cfg(not(target_os = "android"))]
fn app_data_dir() -> io::Result<PathBuf> {
    let proj = ProjectDirs::from("com", "dx", "dx_todo_app")
        .ok_or_else(|| io::Error::other("unable to get project dirs"))?;
    let dir = proj.data_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn todos_file_path() -> io::Result<PathBuf> {
    let dir = app_data_dir()?;
    Ok(dir.join("todos.json"))
}

fn projects_file_path() -> io::Result<PathBuf> {
    let dir = app_data_dir()?;
    Ok(dir.join("projects.json"))
}

pub fn load_or_migrate_projects() -> Vec<Project> {
    // Report the resolved storage dir for debugging
    if let Ok(dir) = app_data_dir() {
        println!("[Storage] Data dir: {}", dir.display());
    }
    // Preferred: projects.json
    if let Ok(path) = projects_file_path() {
        println!("[Storage] Loading projects from {}", path.display());
        if let Ok(mut f) = File::open(&path) {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                if let Ok(list) = serde_json::from_str::<Vec<Project>>(&s) {
                    println!("[Storage] Loaded {} project(s)", list.len());
                    return list;
                }
            } else {
                eprintln!("[Storage] Failed to read projects.json");
            }
        } else {
            println!("[Storage] projects.json not found yet");
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
    match projects_file_path() {
        Ok(path) => {
            println!("[Storage] Saving {} project(s) to {}", projects.len(), path.display());
            match File::create(&path) {
                Ok(mut f) => match serde_json::to_string_pretty(projects) {
                    Ok(s) => {
                        if let Err(e) = f.write_all(s.as_bytes()) {
                            eprintln!("[Storage] Write error: {e}");
                        }
                    }
                    Err(e) => eprintln!("[Storage] Serialize error: {e}"),
                },
                Err(e) => eprintln!("[Storage] Create file error {}: {e}", path.display()),
            }
        }
        Err(e) => eprintln!("[Storage] projects_file_path error: {e}"),
    }
}
