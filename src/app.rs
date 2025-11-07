use iced::{
    executor, Application, Command, Element, Theme,
    widget::{Button, Column, ProgressBar, Scrollable, Text},
};

use rfd::FileDialog;
use std::{collections::HashMap, fs, path::Path};
use std::fs::File;
use std::io::{BufReader, Read};
use trash;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub enum Message {
    GoTo(Screen),
    PickFolder,
    FolderPicked(Option<String>),
    ScanCompleted(Result<Vec<(String, u64)>, String>),
    DeleteFile(String),
    DeleteDuplicates(Vec<String>),
    MakeDuplicate(String),
    MoveFile(String),
    MoveDestinationPicked(String, Option<String>),
    ExitApp,
    ViewDuplicates,
    ShowError(String),
    ClearError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Home,
    FolderSelect,
    Visualization,
    Duplicates,
}

pub struct DiskVisualizer {
    pub screen: Screen,
    pub selected_folder: Option<String>,
    pub files: Vec<(String, u64)>,
    pub folders: Vec<(String, u64)>,
    pub duplicates: HashMap<String, Vec<String>>,
    pub error_message: Option<String>,
}

impl Application for DiskVisualizer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                screen: Screen::Home,
                selected_folder: None,
                files: vec![],
                folders: vec![],
                duplicates: HashMap::new(),
                error_message: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Disk Usage Visualizer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::GoTo(screen) => {
                self.screen = screen;
                self.error_message = None;
                Command::none()
            }

            Message::PickFolder => Command::perform(pick_folder(), Message::FolderPicked),

            Message::FolderPicked(folder) => {
                if let Some(path) = folder.clone() {
                    self.selected_folder = Some(path.clone());
                    return Command::perform(scan_folder_async(path), Message::ScanCompleted);
                }
                Command::none()
            }

            Message::ScanCompleted(result) => match result {
                Ok(files) => {
                    self.files = files.clone();
                    self.folders = aggregate_folder_sizes(&files);
                    self.duplicates = find_duplicates(&files);
                    self.screen = Screen::Visualization;
                    self.error_message = None;
                    Command::none()
                }
                Err(err) => Command::perform(async move { err }, Message::ShowError),
            },

            Message::DeleteFile(path) => {
                if let Err(e) = trash::delete(&Path::new(&path)) {
                    return Command::perform(async move { e.to_string() }, Message::ShowError);
                }
                self.files.retain(|(p, _)| p != &path);
                self.refresh_data();
                Command::none()
            }

            Message::DeleteDuplicates(paths) => {
                for path in &paths {
                    if let Err(e) = trash::delete(&Path::new(path)) {
                        return Command::perform(async move { e.to_string() }, Message::ShowError);
                    }
                }
                self.files.retain(|(p, _)| !paths.contains(p));
                self.refresh_data();
                Command::none()
            }

            Message::MakeDuplicate(path) => {
                match make_duplicate(&path) {
                    Ok(new_path) => {
                        if let Ok(size) = fs::metadata(&new_path).map(|m| m.len()) {
                            self.files.push((new_path.clone(), size));
                            self.refresh_data();
                        }
                    }
                    Err(e) => return Command::perform(async move { e }, Message::ShowError),
                }
                Command::none()
            }

            Message::MoveFile(path) => {
                Command::perform(pick_folder(), move |dest| Message::MoveDestinationPicked(path.clone(), dest))
            }

            Message::MoveDestinationPicked(path, dest_opt) => {
                if let Some(dest) = dest_opt {
                    if let Err(e) = move_file(&path, &dest) {
                        return Command::perform(async move { e }, Message::ShowError);
                    }
                    self.files.retain(|(p, _)| p != &path);
                    self.refresh_data();
                }
                Command::none()
            }

            Message::ViewDuplicates => {
                self.screen = Screen::Duplicates;
                Command::none()
            }

            Message::ShowError(err) => {
                self.error_message = Some(err);
                Command::none()
            }

            Message::ClearError => {
                self.error_message = None;
                Command::none()
            }

            Message::ExitApp => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut content = match self.screen {
            Screen::Home => self.view_home(),
            Screen::FolderSelect => self.view_folder_select(),
            Screen::Visualization => self.view_visualization(),
            Screen::Duplicates => self.view_duplicates(),
        };

        // If error message exists, show it at bottom
        if let Some(ref err) = self.error_message {
            content = Column::new()
                .push(content)
                .push(Text::new(format!("⚠ Error: {}", err)).size(18))
                .push(Button::new("Dismiss").on_press(Message::ClearError))
                .spacing(10)
                .into();
        }

        content
    }
}

impl DiskVisualizer {
    fn refresh_data(&mut self) {
        self.folders = aggregate_folder_sizes(&self.files);
        self.duplicates = find_duplicates(&self.files);
    }

    fn view_home(&self) -> Element<'_, Message> {
        Column::new()
            .push(Text::new("DATA VISUALIZER").size(40))
            .push(Text::new("By Saaim & Ryu").size(20))
            .push(Text::new("SELECT DIRECTORY").size(24))
            .push(Button::new("BROWSE").on_press(Message::PickFolder))
            .push(Button::new("EXIT").on_press(Message::ExitApp))
            .spacing(20)
            .padding(50)
            .into()
    }

    fn view_folder_select(&self) -> Element<'_, Message> {
        Column::new()
            .push(Text::new("Select a folder to scan").size(24))
            .push(Button::new("Browse").on_press(Message::PickFolder))
            .push(Button::new("Back").on_press(Message::GoTo(Screen::Home)))
            .spacing(20)
            .padding(40)
            .into()
    }

    fn view_visualization(&self) -> Element<'_, Message> {
        let mut column = Column::new()
            .push(Text::new("Disk Usage Visualization").size(24))
            .spacing(20)
            .padding(20);

        if let Some(folder) = &self.selected_folder {
            column = column.push(Text::new(format!("Folder: {}", folder)));
        }

        if self.files.is_empty() {
            return column
                .push(Text::new("No files found or failed to scan folder."))
                .push(Button::new("Back").on_press(Message::GoTo(Screen::FolderSelect)))
                .into();
        }

        let total_size: u64 = self.files.iter().map(|(_, s)| *s).sum();
        column = column.push(Text::new("Top Files:"));
        for (name, size) in self.files.iter().take(5) {
            let ratio = *size as f32 / total_size as f32;
            column = column
                .push(Text::new(format!("{name} ({:.2} MB)", *size as f32 / 1_000_000.0)))
                .push(ProgressBar::new(0.0..=1.0, ratio))
                .push(
                    Column::new()
                        .push(Button::new("Delete").on_press(Message::DeleteFile(name.clone())))
                        .push(Button::new("Duplicate").on_press(Message::MakeDuplicate(name.clone())))
                        .push(Button::new("Move").on_press(Message::MoveFile(name.clone()))),
                );
        }

        column = column
            .push(Button::new("View Duplicates").on_press(Message::ViewDuplicates))
            .push(Button::new("Back").on_press(Message::GoTo(Screen::FolderSelect)));

        Scrollable::new(column).into()
    }

    fn view_duplicates(&self) -> Element<'_, Message> {
        let mut column = Column::new()
            .push(Text::new("Duplicate Files").size(24))
            .spacing(20)
            .padding(20);

        if self.duplicates.is_empty() {
            column = column.push(Text::new("No duplicates found."));
        } else {
            for group in self.duplicates.values() {
                column = column.push(Text::new(format!("Group of {} duplicates:", group.len())));
                for path in group {
                    column = column.push(Text::new(format!("• {}", path)));
                }
                column = column.push(Button::new("Delete All").on_press(Message::DeleteDuplicates(group.clone())));
            }
        }

        column = column.push(Button::new("Back").on_press(Message::GoTo(Screen::Visualization)));

        Scrollable::new(column).into()
    }
}

// Async folder picker
async fn pick_folder() -> Option<String> {
    FileDialog::new().pick_folder().map(|p| p.display().to_string())
}

// Async scan with error handling
async fn scan_folder_async(path: String) -> Result<Vec<(String, u64)>, String> {
    let mut files = vec![];
    for entry in WalkDir::new(&path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        match fs::metadata(entry.path()) {
            Ok(metadata) => files.push((entry.path().display().to_string(), metadata.len())),
            Err(e) => eprintln!("Failed to read metadata for {}: {}", entry.path().display(), e),
        }
    }

    if files.is_empty() {
        return Err("No readable files found or access denied.".into());
    }

    files.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(files)
}

fn aggregate_folder_sizes(files: &[(String, u64)]) -> Vec<(String, u64)> {
    let mut folder_sizes: HashMap<String, u64> = HashMap::new();
    for (path, size) in files {
        if let Some(parent) = Path::new(path).parent() {
            let folder = parent.display().to_string();
            *folder_sizes.entry(folder).or_insert(0) += *size;
        }
    }
    let mut folder_vec: Vec<(String, u64)> = folder_sizes.into_iter().collect();
    folder_vec.sort_by(|a, b| b.1.cmp(&a.1));
    folder_vec
}

fn compute_hash(path: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 4096];
    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Some(hasher.finalize().to_hex().to_string())
}

fn find_duplicates(files: &[(String, u64)]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for (path, _) in files {
        if let Some(hash) = compute_hash(path) {
            map.entry(hash).or_default().push(path.clone());
        }
    }
    map.into_iter()
        .filter(|(_, group)| group.len() > 1)
        .collect()
}

fn make_duplicate(original_path: &str) -> Result<String, String> {
    let original = Path::new(original_path);
    let parent = original
        .parent()
        .ok_or("Failed to get parent directory.")?;
    let file_name = original
        .file_name()
        .ok_or("Failed to read file name.")?
        .to_string_lossy();
    let new_name = format!("{}_copy", file_name);
    let new_path = parent.join(new_name);

    fs::copy(original, &new_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;
    Ok(new_path.display().to_string())
}

fn move_file(original_path: &str, dest_folder: &str) -> Result<(), String> {
    let src = Path::new(original_path);
    if let Some(file_name) = src.file_name() {
        let dest = Path::new(dest_folder).join(file_name);
        fs::rename(src, &dest)
            .map_err(|e| format!("Failed to move file: {}", e))?;
        Ok(())
    } else {
        Err("Invalid file path.".into())
    }
}
