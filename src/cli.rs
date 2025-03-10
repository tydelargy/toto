/*
use std::env;
*/

use anyhow::{Context, Result};
use chrono::{
    DateTime, Utc,
    serde::{ts_seconds, ts_seconds_option},
};
use clap::{Arg, ArgAction, Command, command, value_parser};
use serde::{Deserialize, Serialize};
use std::process::Command as ProcessCommand;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
};
use uuid::Uuid;

use crate::term;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub data: TodoData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoData {
    pub title: String,
    pub description: Option<String>,
    pub priority: u8,
    pub status: TodoStatus,
    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "ts_seconds_option")]
    pub in_progress_at: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(with = "ts_seconds_option")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Deleted,
}

pub struct Cli {
    file_path: String,
    todo_map: HashMap<Uuid, TodoData>,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            file_path: String::from("."),
            todo_map: HashMap::new(),
        }
    }
}

impl Cli {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            todo_map: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let matches = command!()
            .subcommand_required(true)
            .subcommand(
                Command::new("add")
                    .long_flag("add")
                    .short_flag('a')
                    .about("Add a todo")
                    .arg(Arg::new("title").required(true))
                    .arg(
                        Arg::new("description")
                            .required(false)
                            .long("description")
                            .short('d')
                            .help("Description of the todo")
                            .value_parser(value_parser!(String)),
                    )
                    .arg(
                        Arg::new("priority")
                            .required(true)
                            .long("priority")
                            .short('p')
                            .help("Priority of the todo")
                            .value_parser(value_parser!(u8))
                            .default_value("255"),
                    )
                    .arg(
                        Arg::new("in-progress")
                            .required(false)
                            .long("in-progress")
                            .short('i')
                            .help("Mark the todo as in progress")
                            .action(ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                Command::new("list")
                    .long_flag("list")
                    .short_flag('l')
                    .about("List all todos")
                    .arg(
                        Arg::new("verbose")
                            .required(false)
                            .long("verbose")
                            .short('v')
                            .help("Verbose output")
                            .action(ArgAction::SetTrue),
                    ),
            )
            .subcommand(
                Command::new("update")
                    .long_flag("update")
                    .short_flag('u')
                    .about("Update a todo")
                    .arg(
                        Arg::new("id")
                            .required(true)
                            .long("id")
                            .short('i')
                            .help("ID of the todo")
                            .value_parser(value_parser!(Uuid)),
                    )
                    .arg(
                        Arg::new("title")
                            .required(false)
                            .long("title")
                            .short('t')
                            .help("Title of the todo")
                            .value_parser(value_parser!(String)),
                    )
                    .arg(
                        Arg::new("description")
                            .required(false)
                            .long("description")
                            .short('d')
                            .help("Description of the todo")
                            .value_parser(value_parser!(String)),
                    )
                    .arg(
                        Arg::new("priority")
                            .required(false)
                            .long("priority")
                            .short('p')
                            .help("Priority of the todo")
                            .value_parser(value_parser!(u8)),
                    )
                    .arg(
                        Arg::new("in-progress")
                            .required(false)
                            .long("in-progress")
                            .short('i')
                            .help("Mark the todo as in progress")
                            .value_parser(value_parser!(bool)),
                    )
                    .arg(
                        Arg::new("completed")
                            .required(false)
                            .long("completed")
                            .short('c')
                            .help("Mark the todo as completed")
                            .value_parser(value_parser!(bool)),
                    )
                    .arg(
                        Arg::new("deleted")
                            .required(false)
                            .long("deleted")
                            .short('d')
                            .help("Mark the todo as deleted")
                            .value_parser(value_parser!(bool)),
                    ),
            )
            .subcommand(
                Command::new("complete")
                    .long_flag("complete")
                    .short_flag('c')
                    .about("Complete a todo")
                    .arg(
                        Arg::new("id")
                            .required(true)
                            .long("id")
                            .short('i')
                            .help("ID of the todo")
                            .value_parser(value_parser!(Uuid)),
                    ),
            )
            .subcommand(
                Command::new("start")
                    .long_flag("start")
                    .short_flag('s')
                    .about("Mark a todo as in progress")
                    .arg(
                        Arg::new("id")
                            .required(true)
                            .long("id")
                            .short('i')
                            .help("ID of the todo")
                            .value_parser(value_parser!(Uuid)),
                    ),
            )
            .subcommand(
                Command::new("delete")
                    .long_flag("delete")
                    .short_flag('d')
                    .about("Delete a todo")
                    .arg(
                        Arg::new("id")
                            .required(true)
                            .long("id")
                            .short('i')
                            .help("ID of the todo")
                            .value_parser(value_parser!(String)),
                    ),
            )
            .subcommand(Command::new("sync").about("Sync with git"))
            .get_matches();
        self.load_todos()?;

        match matches.subcommand() {
            Some(("add", add_matches)) => {
                let title = add_matches.get_one::<String>("title").unwrap();
                let description = add_matches.get_one::<String>("description");
                let priority = add_matches.get_one::<u8>("priority").unwrap();
                let in_progress = add_matches.get_one::<bool>("in-progress").unwrap();
                self.add_todo(title, description, priority, in_progress);
            }
            Some(("list", list_matches)) => {
                let verbose = list_matches.get_flag("verbose");
                self.list_todos(verbose);
            }
            Some(("update", update_matches)) => {
                let id = update_matches.get_one::<String>("id").unwrap();
                let title = update_matches.get_one::<String>("title");
                let description = update_matches.get_one::<String>("description");
                let priority = update_matches.get_one::<u8>("priority");
                let in_progress = update_matches.get_one::<bool>("in-progress");
                let completed = update_matches.get_one::<bool>("completed");
                let deleted = update_matches.get_one::<bool>("deleted");
                self.update_todo(
                    id,
                    title,
                    description,
                    priority,
                    in_progress,
                    completed,
                    deleted,
                )?;
            }
            Some(("start", start_matches)) => {
                let id = start_matches.get_one::<String>("id").unwrap();
                self.start_todo(id)?;
            }
            Some(("complete", complete_matches)) => {
                let id = complete_matches.get_one::<String>("id").unwrap();
                self.complete_todo(id)?;
            }
            Some(("delete", delete_matches)) => {
                let id = delete_matches.get_one::<String>("id").unwrap();
                self.delete_todo(id)?;
            }
            Some(("sync", _)) => {
                self.sync()?;
            }
            _ => {}
        };

        self.save_todos()?;

        Ok(())
    }

    fn load_todos(&mut self) -> Result<()> {
        let file_path = format!("{}/todos.json", self.file_path);

        // Attempt to open the file create it if it doesn't exist
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(file_path)
            .context("Failed to open or create todo file")?;

        let metadata = file.metadata().context("Failed to get file metadata")?;

        // Early return if we created the file and it is empty.
        if metadata.len() == 0 {
            return Ok(());
        }

        let reader = BufReader::new(file);
        let todos: Vec<Todo> =
            serde_json::from_reader(reader).context("Failed to deserialize todo list")?;

        for todo in todos {
            self.todo_map.insert(todo.id, todo.data);
        }
        Ok(())
    }

    fn save_todos(&self) -> Result<()> {
        let file_path = format!("{}/todos.json", self.file_path);

        // Attempt to open the file create it if it doesn't exist
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(file_path)
            .context("Failed to open or create todo file")?;

        let writer = BufWriter::new(file);

        let todos: Vec<Todo> = self
            .todo_map
            .iter()
            .map(|(&id, data)| Todo {
                id,
                data: data.clone(),
            })
            .collect();

        serde_json::to_writer_pretty(writer, &todos).context("Failed to serialize todo list")?;
        Ok(())
    }

    fn add_todo(
        &mut self,
        title: &String,
        description: Option<&String>,
        priority: &u8,
        in_progress: &bool,
    ) {
        let id = Uuid::new_v4();
        let todo = Todo {
            id,
            data: TodoData {
                title: title.clone(),
                description: description.map(|s| s.clone()),
                priority: *priority,
                in_progress_at: if *in_progress { Some(Utc::now()) } else { None },
                created_at: Utc::now(),
                completed_at: None,
                deleted_at: None,
                status: if *in_progress {
                    TodoStatus::InProgress
                } else {
                    TodoStatus::Pending
                },
            },
        };
        self.todo_map.insert(id, todo.data);
    }
    fn update_todo(
        &mut self,
        id: &String,
        title: Option<&String>,
        description: Option<&String>,
        priority: Option<&u8>,
        in_progress: Option<&bool>,
        completed: Option<&bool>,
        deleted: Option<&bool>,
    ) -> Result<()> {
        if let Ok(todo_id) = self.parse_todo_id(id) {
            if let Some(todo) = self.todo_map.get_mut(&todo_id) {
                if let Some(title) = title {
                    todo.title = title.clone();
                }
                if let Some(description) = description {
                    // Only update description if some value is provided
                    todo.description = Some(description.clone());
                }
                if let Some(priority) = priority {
                    todo.priority = *priority;
                }
                if let Some(in_progress) = in_progress {
                    if *in_progress {
                        if todo.in_progress_at.is_none() {
                            todo.in_progress_at = Some(Utc::now());
                        }
                        todo.status = TodoStatus::InProgress;
                    }
                }
                if let Some(completed) = completed {
                    if *completed {
                        if todo.completed_at.is_none() {
                            todo.completed_at = Some(Utc::now());
                        }
                        todo.status = TodoStatus::Completed;
                    }
                }
                if let Some(deleted) = deleted {
                    if *deleted {
                        if todo.deleted_at.is_none() {
                            todo.completed_at = Some(Utc::now());
                        }
                        todo.status = TodoStatus::Deleted;
                    }
                }
            } else {
                println!("Todo not found");
            }
        }
        Ok(())
    }

    fn start_todo(&mut self, id: &String) -> Result<()> {
        if let Ok(todo_id) = self.parse_todo_id(id) {
            if let Some(todo) = self.todo_map.get_mut(&todo_id) {
                if todo.in_progress_at.is_none() {
                    todo.in_progress_at = Some(Utc::now());
                }
                todo.status = TodoStatus::InProgress;
            } else {
                println!("Todo not found");
            }
        }
        Ok(())
    }
    fn complete_todo(&mut self, id: &String) -> Result<()> {
        if let Ok(todo_id) = self.parse_todo_id(id) {
            if let Some(todo) = self.todo_map.get_mut(&todo_id) {
                if todo.completed_at.is_none() {
                    todo.completed_at = Some(Utc::now());
                    todo.status = TodoStatus::Completed;
                } else {
                    println!("Todo is already completed");
                }
            } else {
                eprintln!("Todo not found");
            }
        }
        Ok(())
    }

    fn delete_todo(&mut self, id: &String) -> Result<()> {
        if let Ok(todo_id) = self.parse_todo_id(id) {
            if let Some(todo) = self.todo_map.get_mut(&todo_id) {
                if todo.deleted_at.is_none() {
                    todo.deleted_at = Some(Utc::now());
                    todo.status = TodoStatus::Deleted;
                } else {
                    println!("Todo is already deleted");
                }
            } else {
                eprintln!("Todo not found");
            }
        }
        Ok(())
    }

    fn list_todos(&self, verbose: bool) {
        term::splash();
        let todos = self.ordered_todos();
        for (id, todo) in todos.iter().enumerate() {
            term::print_todo(verbose, todo, id);
        }
        todo!("Fill in - call the formatters");
    }

    pub fn sync(&mut self) -> Result<()> {
        // Collect all keys whose TodoData indicates completion or deletion.
        let keys_to_archive: Vec<Uuid> = self
            .todo_map
            .iter()
            .filter(|(_, data)| data.completed_at.is_some() || data.deleted_at.is_some())
            .map(|(&uuid, _)| uuid)
            .collect();

        // Create a vector of archived todos.
        let mut archived_todos: Vec<Todo> = Vec::new();
        for key in keys_to_archive {
            if let Some(todo_data) = self.todo_map.remove(&key) {
                archived_todos.push(Todo {
                    id: key,
                    data: todo_data,
                });
            }
        }

        // If there are no todos to archive, we can exit early.
        if archived_todos.is_empty() {
            println!("No completed or deleted todos to archive.");
            return Ok(());
        }

        // Build filename with current date in YYYYMMDD format.
        let date_str = Utc::now().format("%Y%m%d").to_string();
        let read_file_path = format!("{}/completed_{}.json", self.file_path, date_str);
        let write_file_path = format!("{}/completed_{}.json", self.file_path, date_str);

        // Attempt to open the file create it if it doesn't exist
        let read_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(read_file_path)
            .context("Failed to open or create todo file")?;

        let metadata = read_file
            .metadata()
            .context("Failed to get file metadata")?;

        let mut archive = Vec::new();
        if metadata.len() > 0 {
            // Read existing archived todos from file if it exists.
            let reader = BufReader::new(read_file);
            archive = serde_json::from_reader(reader).context("Failed to deserialize todo list")?;
        }

        // Extend the existing todos with the newly archived ones.
        archive.extend(archived_todos);

        let write_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(write_file_path)
            .context("Failed to open or create todo file")?;

        // Serialize the combined vector to pretty JSON.
        let writer = BufWriter::new(write_file);

        serde_json::to_writer_pretty(writer, &archive).context("Failed to serialize todo list")?;

        // Change to a specific directory and run git commands.
        // Replace the following path with your target directory.
        let target_dir = self.file_path.clone();

        // Run "git add ."
        let status = ProcessCommand::new("cd ")
            .arg(target_dir)
            .arg(" \\ git add .".to_string())
            .arg(format!("\\ git commit -m \"archive {}\"", date_str).to_string())
            .output();
        if status.is_ok() {
            println!("Git add and commit executed successfully.");
        }

        let status = ProcessCommand::new("git push").output();
        if status.is_ok() {
            println!("Git push executed successfully.");
        }

        Ok(())
    }

    fn parse_todo_id(&self, id: &String) -> Result<Uuid> {
        let todos = self.ordered_todos();

        if let Ok(human_id) = id.parse::<usize>() {
            if human_id < todos.len() {
                return Ok(todos[human_id].id);
            }
        }
        let uuid =
            Uuid::parse_str(&id).with_context(|| format!("Failed to parse todo id: {}", id))?;
        Ok(uuid)
    }

    fn ordered_todos(&self) -> Vec<Todo> {
        let mut todos: Vec<Todo> = self
            .todo_map
            .iter()
            .map(|(&id, data)| Todo {
                id,
                data: data.clone(),
            })
            .collect();

        todos.sort_by(|a, b| {
            let priority_cmp = a.data.priority.cmp(&b.data.priority);

            if priority_cmp == Ordering::Equal {
                // If priority is equal, sort by created_at
                a.data.created_at.cmp(&b.data.created_at)
            } else {
                priority_cmp
            }
        });

        todos
    }
}
