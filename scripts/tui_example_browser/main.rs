use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    path::Path,
    process::Command,
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
};

#[derive(Debug, Clone)]
struct Example {
    package: String,
    name: String,
}

impl Example {
    fn identifier(&self) -> String {
        format!("{}:{}", self.package, self.name)
    }

    fn display_text(&self) -> String {
        format!("{} - {}", self.package, self.name)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default packages to look for examples if none are specified.
    const ALL_PACKAGES: &[&str] = &["examples", "generative_design", "nature_of_code"];

    // Process command-line arguments to determine which packages to consider.
    // If none are provided, default to ALL_PACKAGES.
    let mut args = env::args();
    args.next(); // skip executable name
    let specified: Vec<String> = args.collect();
    let packages: Vec<String> = if specified.is_empty() {
        ALL_PACKAGES.iter().map(|s| s.to_string()).collect()
    } else {
        specified
    };

    // Determine the workspace root by moving two levels up from this package's Cargo.toml.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_manifest_dir = Path::new(manifest_dir)
        .parent()
        .unwrap() // e.g. nannou/scripts
        .parent()
        .unwrap(); // e.g. nannou

    // First, collect examples from each specified package.
    let mut examples: Vec<Example> = Vec::new();
    for package in packages.iter() {
        let package_dir = workspace_manifest_dir.join(package);
        // Expect the manifest at "Cargo.toml" in the package directory.
        let manifest_path = package_dir.join("Cargo").with_extension("toml");
        let manifest_contents = fs::read(&manifest_path)
            .unwrap_or_else(|_| panic!("Failed to read manifest at {:?}", manifest_path));
        let manifest_str = std::str::from_utf8(&manifest_contents)
            .unwrap_or_else(|_| panic!("Manifest is not valid UTF-8 at {:?}", manifest_path));
        let toml_value: toml::Value = toml::from_str(manifest_str)
            .unwrap_or_else(|_| panic!("Failed to parse manifest at {:?}", manifest_path));

        // Retrieve the examples array from the manifest (assumed to be under the key "example").
        let ex_arr = toml_value
            .get("example")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("Failed to get 'example' array in {:?}", manifest_path));

        for ex in ex_arr {
            let name = ex
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("Failed to get example name in {:?}", manifest_path));
            examples.push(Example {
                package: package.clone(),
                name: name.to_string(),
            });
        }
    }

    // Next, collect binary targets from the workspace.
    // Running "cargo run --bin" with no argument produces an error message listing available binaries.
    let bin_output = Command::new("cargo")
        .args(&["run", "--bin"])
        .current_dir(workspace_manifest_dir)
        .output()?;
    let bin_stderr = String::from_utf8_lossy(&bin_output.stderr);
    if let Some(idx) = bin_stderr.find("Available binaries:") {
        // Take all lines after "Available binaries:".
        let bin_list_str = &bin_stderr[idx..];
        for line in bin_list_str.lines() {
            let trimmed = line.trim();
            // Skip the header line and any empty lines.
            if trimmed.is_empty() || trimmed == "Available binaries:" {
                continue;
            }
            // Each remaining line is a binary name.
            examples.push(Example {
                package: "bin".to_string(),
                name: trimmed.to_string(),
            });
        }
    }

    // Sort the combined list by identifier.
    examples.sort_by(|a, b| a.identifier().cmp(&b.identifier()));
    if examples.is_empty() {
        println!("No examples or binaries found!");
        return Ok(());
    }

    // Determine the run history file location in the workspace root.
    let history_path = workspace_manifest_dir.join("run_history.txt");
    let mut run_history: HashSet<String> = HashSet::new();
    if let Ok(contents) = fs::read_to_string(&history_path) {
        for line in contents.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                run_history.insert(trimmed.to_string());
            }
        }
    }

    // Set up terminal in raw mode with an alternate screen.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        Clear(ClearType::All)
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up TUI list state.
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    'main_loop: loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            // Build list items—if an example has been run, display it in blue.
            let items: Vec<ListItem> = examples
                .iter()
                .map(|ex| {
                    let mut item = ListItem::new(ex.display_text());
                    if run_history.contains(&ex.identifier()) {
                        item = item.style(Style::default().fg(Color::Blue));
                    }
                    item
                })
                .collect();

            let title = format!(
                "Select target ({} items found, Esc or q to exit)",
                examples.len()
            );
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(Style::default().fg(Color::Yellow))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[0], &mut list_state);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => break 'main_loop,
                    KeyCode::Down => {
                        let i = match list_state.selected() {
                            Some(i) if i >= examples.len() - 1 => i,
                            Some(i) => i + 1,
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match list_state.selected() {
                            Some(0) | None => 0,
                            Some(i) => i - 1,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = list_state.selected() {
                            let ex = &examples[idx];
                            // Tear down the TUI before launching the target.
                            disable_raw_mode()?;
                            execute!(
                                terminal.backend_mut(),
                                LeaveAlternateScreen,
                                DisableMouseCapture
                            )?;
                            terminal.show_cursor()?;

                            // Run the selected target.
                            if ex.package == "bin" {
                                println!("Running: cargo run --bin {}", ex.name);
                                let mut child = Command::new("cargo")
                                    .args(&["run", "--bin", &ex.name])
                                    .spawn()
                                    .expect("Failed to spawn process");
                                let status = child.wait().expect("Failed to wait for process");
                                println!("Process exited with status: {}", status);
                            } else {
                                println!(
                                    "Running: cargo run -p {} --example {}",
                                    ex.package, ex.name
                                );
                                let mut child = Command::new("cargo")
                                    .args(&["run", "-p", &ex.package, "--example", &ex.name])
                                    .spawn()
                                    .expect("Failed to spawn process");
                                let status = child.wait().expect("Failed to wait for process");
                                println!("Process exited with status: {}", status);
                            }

                            // Update run history.
                            if run_history.insert(ex.identifier()) {
                                let history_data: Vec<String> =
                                    run_history.iter().cloned().collect();
                                fs::write(&history_path, history_data.join("\n"))?;
                            }

                            // Flush stray events.
                            while event::poll(Duration::from_millis(0))? {
                                let _ = event::read();
                            }
                            thread::sleep(Duration::from_millis(50));

                            // Reinitialize the terminal.
                            enable_raw_mode()?;
                            let mut stdout = io::stdout();
                            execute!(
                                stdout,
                                EnterAlternateScreen,
                                EnableMouseCapture,
                                Clear(ClearType::All)
                            )?;
                            terminal = Terminal::new(CrosstermBackend::new(stdout))?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal state on exit.
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        Clear(ClearType::All)
    )?;
    terminal.show_cursor()?;
    Ok(())
}
