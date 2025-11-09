use iced::{
    executor, Application, Command, Element, Theme, Alignment, Length,
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
    ScanCompleted(Vec<(String, u64)>),
    DeleteFile(String),
    DeleteDuplicates(Vec<String>),
    MakeDuplicate(String),
    MoveFile(String),
    MoveDestinationPicked(String, Option<String>),
    ExitApp,
    ViewDuplicates,
    ToggleTheme,
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
    pub theme: Theme, 
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
                theme: Theme::Light,
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
            Message::ScanCompleted(files) => {
                self.files = files.clone();
                self.folders = aggregate_folder_sizes(&files);
                self.duplicates = find_duplicates(&files);
                self.screen = Screen::Visualization;
                Command::none()
            }
            Message::DeleteFile(path) => {
                let _ = trash::delete(&Path::new(&path));
                self.files.retain(|(p, _)| p != &path);
                self.folders = aggregate_folder_sizes(&self.files);
                self.duplicates = find_duplicates(&self.files);
                Command::none()
            }
            Message::DeleteDuplicates(paths) => {
                for path in paths {
                    let _ = trash::delete(&Path::new(&path));
                    self.files.retain(|(p, _)| p != &path);
                }
                self.folders = aggregate_folder_sizes(&self.files);
                self.duplicates = find_duplicates(&self.files);
                Command::none()
            }
            Message::MakeDuplicate(path) => {
                if let Some(new_path) = make_duplicate(&path) {
                    self.files.push((new_path.clone(), fs::metadata(&new_path).map(|m| m.len()).unwrap_or(0)));
                    self.folders = aggregate_folder_sizes(&self.files);
                    self.duplicates = find_duplicates(&self.files);
                }
                Command::none()
            }
            Message::MoveFile(path) => {
                // Pick destination folder to move the file
                Command::perform(pick_folder(), move |dest| Message::MoveDestinationPicked(path.clone(), dest))
            }
            Message::MoveDestinationPicked(path, dest_opt) => {
                if let Some(dest) = dest_opt {
                    if move_file(&path, &dest) {
                        // Remove from current list and refresh
                        self.files.retain(|(p, _)| p != &path);
                        self.folders = aggregate_folder_sizes(&self.files);
                        self.duplicates = find_duplicates(&self.files);
                    }
                }
                Command::none()
            }
            Message::ViewDuplicates => {
                self.screen = Screen::Duplicates;
                Command::none()
            }


            Message::ToggleTheme => {
                self.theme = match self.theme {
                    Theme::Light => Theme::Dark,
                    Theme::Dark => Theme::Light,
                    _ => Theme::Light, 
                };
                Command::none()
            }


            Message::ExitApp => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match self.screen {
            Screen::Home => self.view_home(),
            Screen::FolderSelect => self.view_folder_select(),
            Screen::Visualization => self.view_visualization(),
            Screen::Duplicates => self.view_duplicates(),
        }
    }


    fn theme(&self) -> Theme {
    self.theme.clone()
    }

}


impl DiskVisualizer {
    //Make the UI look better
    fn view_home(&self) -> Element<'_, Message> {
        use iced::widget::Container;

        let content = Column::new()
            .push(
                Container::new(
                    Text::new("DATA VISUALIZER")
                        .size(50)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .center_x(),
            )
            .push(
                Container::new(
                    Text::new("By Saaim & Ryu")
                        .size(20)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .center_x(),
            )
            .push(
                Container::new(
                    Text::new("SELECT DIRECTORY")
                        .size(30)
                        .horizontal_alignment(iced::alignment::Horizontal::Center),
                )
                .width(Length::Fill)
                .center_x(),
            )
            .push(
                Button::new("                   Next")
                    .on_press(Message::GoTo(Screen::FolderSelect))
                    .width(Length::Fixed(200.0)),
            )

            .push(
    Button::new("                Theme")
                .on_press(Message::ToggleTheme)
                .width(Length::Fixed(200.0))
                .style(iced::theme::Button::Primary),
        )


            .push(
                Button::new("                   EXIT")
                    .on_press(Message::ExitApp)
                    .width(Length::Fixed(200.0)),
            )

            .spacing(20)
            .align_items(Alignment::Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .padding(50)
        .into()
    }


    fn view_folder_select(&self) -> Element<'_, Message> {
    use iced::widget::Container;

    let content = Column::new()
        .push(Text::new("Select a folder to scan").size(30))
        .push(
            Button::new("                 Browse")
                .on_press(Message::PickFolder)
                .width(Length::Fixed(200.0)),
        )
        .push(
            Button::new("                   Back")
                .on_press(Message::GoTo(Screen::Home))
                .width(Length::Fixed(200.0)),
        )
        .spacing(20)
        .align_items(Alignment::Center);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
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

        let total_size: u64 = self.files.iter().map(|(_, s)| *s).sum();

        column = column.push(Text::new("Top Files:"));
        for (name, size) in self.files.iter().take(5) {
            let ratio = *size as f32 / total_size as f32;
            column = column
                .push(Text::new(format!("{name} ({:.2} MB)", *size as f32 / 1_000_000.0)))
                .push(ProgressBar::new(0.0..=1.0, ratio))
                .push(
                    Column::new()
                        .spacing(10)
                        .push(Button::new("Delete").on_press(Message::DeleteFile(name.clone())))
                        .push(Button::new("Make Duplicate").on_press(Message::MakeDuplicate(name.clone())))
                        .push(Button::new("Move File").on_press(Message::MoveFile(name.clone())))
                );
        }

        column = column.push(Text::new("Top Folders:"));
        for (folder, size) in self.folders.iter().take(5) {
            let ratio = *size as f32 / total_size as f32;
            column = column
                .push(Text::new(format!("{folder} ({:.2} MB)", *size as f32 / 1_000_000.0)))
                .push(ProgressBar::new(0.0..=1.0, ratio));
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

// Async scan
async fn scan_folder_async(path: String) -> Vec<(String, u64)> {
    let mut files = vec![];

    for entry in WalkDir::new(&path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if let Ok(metadata) = fs::metadata(entry.path()) {
            files.push((
                entry.path().display().to_string(),
                metadata.len(),
            ));
        }
    }

    files.sort_by(|a, b| b.1.cmp(&a.1));
    files
}

// Folder size aggregation
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


// File hash
fn compute_hash(path: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 4096];

    while let Ok(n) = reader.read(&mut buffer) {
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }

    Some(hasher.finalize().to_hex().to_string())
}

// Duplicate detection
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

// Create a duplicate file
fn make_duplicate(original_path: &str) -> Option<String> {
    let original = Path::new(original_path);
    let parent = original.parent()?;
    let file_name = original.file_name()?.to_string_lossy();
    let new_name = format!("{}_copy", file_name);
    let new_path = parent.join(new_name);

    fs::copy(original, &new_path).ok()?;
    Some(new_path.display().to_string())
}

// Move file to destination folder
fn move_file(original_path: &str, dest_folder: &str) -> bool {
    let src = Path::new(original_path);
    if let Some(file_name) = src.file_name() {
        let dest = Path::new(dest_folder).join(file_name);
        if let Err(e) = fs::rename(src, &dest) {
            eprintln!("Failed to move file: {}", e);
            return false;
        }
        return true;
    }
    false
}

