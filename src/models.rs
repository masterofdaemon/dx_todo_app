use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
    #[serde(default)]
    pub subtasks: Vec<Subtask>,
    #[serde(default)]
    pub description: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Subtask {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Filter {
    All,
    Active,
    Completed,
}
