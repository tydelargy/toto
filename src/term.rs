use crate::cli::{Todo, TodoStatus};
use terminal_size::{Height, Width, terminal_size};

const SMALL_TERM: u16 = 80;

// Create splash screen based on the terminal size.

fn splash_large() {
    println!(
        "{:^36} | {:^30} | {:^20} | {:^2} | {:^10} | CREATED",
        "ID", "TITLE", "DESCRIPTION", "PRIORITY", "STATUS",
    );
}

fn splash_small() {
    println!("{:^8} | {:^10} | STATUS", "ID", "TITLE");
}

pub fn splash() {
    // Open the standard output terminal.
    let size = terminal_size();

    // get_winsize() returns an Option with (width, height)
    if let Some((Width(w), Height(_h))) = size {
        if w > SMALL_TERM {
            splash_large();
        } else {
            splash_small();
        }
    } else {
        splash_small();
    }
}

pub fn print_todo(verbose: bool, todo: &Todo, id: usize) {
    // Open the standard output terminal.
    let size = terminal_size();
    // get_winsize() returns an Option with (width, height)
    if let Some((Width(w), Height(_h))) = size {
        if w > SMALL_TERM {
            print_todo_large(verbose, todo, id);
        } else {
            print_todo_small(verbose, todo, id);
        }
    } else {
        print_todo_small(verbose, todo, id);
    }
}

/// Prints a compact summary of a todo item suitable for a ~20-column terminal.
/// It displays a short id, a truncated title, and a one-letter status indicator.
pub fn print_todo_small(verbose: bool, todo: &Todo, id: usize) {
    // Use the full UUID if verbose, otherwise the human-readable id.
    // For small output, we truncate the UUID to its first 8 characters.
    let id_str = if verbose {
        let uuid_str = todo.id.to_string();
        if uuid_str.len() > 8 {
            uuid_str[..8].to_string()
        } else {
            uuid_str.to_string()
        }
    } else {
        // Format the human-readable id as a string.
        id.to_string()
    };

    // For the title, allow a maximum of 10 characters.
    let max_title_len = 10;
    let title = if todo.data.title.len() > max_title_len {
        // Leave room for the ellipsis.
        format!("{}...", &todo.data.title[..max_title_len.saturating_sub(3)])
    } else {
        todo.data.title.clone()
    };

    // Use a one-character indicator for the status.
    let status_initial = match todo.data.status {
        TodoStatus::Pending => "P",
        TodoStatus::InProgress => "I",
        TodoStatus::Completed => "C",
        TodoStatus::Deleted => "D",
    };

    // Print in a compact format.
    // We allocate 8 characters for the id, 10 for the title, plus the status.
    println!("{:^8} | {:^10} | {}", id_str, title, status_initial);
}

/// Prints a detailed summary of a todo item suitable for a ~50-60 column terminal.
/// It displays a longer id, a longer title, a truncated description if available,
/// the priority, status, and the creation date.
pub fn print_todo_large(verbose: bool, todo: &Todo, id: usize) {
    // Use the full UUID or human-readable id.
    let id_str = if verbose {
        todo.id.to_string()
    } else {
        id.to_string()
    };

    // For the title, allow up to 30 characters.
    let max_title_len = 30;
    let title = if todo.data.title.len() > max_title_len {
        format!("{}...", &todo.data.title[..max_title_len.saturating_sub(3)])
    } else {
        todo.data.title.clone()
    };

    // For the description, allow up to 20 characters if it exists.
    let max_desc_len = 20;
    let description = match &todo.data.description {
        Some(desc) => {
            if desc.len() > max_desc_len {
                format!("{}...", &desc[..max_desc_len.saturating_sub(3)])
            } else {
                desc.clone()
            }
        }
        None => String::from(""),
    };

    let status = format!("{:?}", todo.data.status);
    let created_at = todo.data.created_at.format("%Y-%m-%d").to_string();

    // Print the detailed view.
    // Adjust column widths to fit within about 60 characters.
    println!(
        "{:^36} | {:^30} | {:^20} | {:^2} | {:^10} | {}",
        id_str, title, description, todo.data.priority, status, created_at
    );
}
