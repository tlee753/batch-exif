use iced::widget::{button, column, container, image, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length, Task};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;
use std::fs;
use std::path::PathBuf;

pub fn main() -> iced::Result {
    iced::application(ExifApp::default, ExifApp::update, ExifApp::view)
        .title("Genealogy EXIF Batch Manager - v2.0")
        .run()
}

struct ExifApp {
    folder_path: Option<PathBuf>,
    file_list: Vec<PathBuf>,
    selected_file: Option<PathBuf>,
    // Metadata Fields
    date_str: String,
    time_str: String,
    latitude_str: String,
    longitude_str: String,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectFolder,
    FolderSelected(Option<PathBuf>),
    SelectFile(PathBuf),
    DateChanged(String),
    TimeChanged(String),
    LatitudeChanged(String),
    LongitudeChanged(String),
    ApplyChanges,
}

impl Default for ExifApp {
    fn default() -> Self {
        Self {
            folder_path: None,
            file_list: Vec::new(),
            selected_file: None,
            date_str: "1920:06:15".to_string(),
            time_str: "12:00:00".to_string(),
            latitude_str: "33.9519".to_string(),
            longitude_str: "-83.3576".to_string(),
            status: "Select a folder to load family archive photos.".to_string(),
        }
    }
}

impl ExifApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectFolder => Task::future(async {
                let handle = rfd::AsyncFileDialog::new().pick_folder().await;
                Message::FolderSelected(handle.map(|h| h.path().to_path_buf()))
            }),
            Message::FolderSelected(path) => {
                if let Some(p) = path {
                    self.status = format!("Loaded folder: {}", p.display());
                    self.folder_path = Some(p.clone());
                    self.file_list.clear();
                    self.selected_file = None;

                    if let Ok(entries) = fs::read_dir(p) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if let Some(ext) = path.extension() {
                                let ext_str = ext.to_string_lossy().to_lowercase();
                                if ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png" {
                                    self.file_list.push(path);
                                }
                            }
                        }
                    }
                    self.file_list.sort();
                    if let Some(first) = self.file_list.first().cloned() {
                        self.selected_file = Some(first);
                    }
                }
                Task::none()
            }
            Message::SelectFile(file_path) => {
                self.selected_file = Some(file_path);
                Task::none()
            }
            Message::DateChanged(val) => {
                self.date_str = val;
                Task::none()
            }
            Message::TimeChanged(val) => {
                self.time_str = val;
                Task::none()
            }
            Message::LatitudeChanged(val) => {
                self.latitude_str = val;
                Task::none()
            }
            Message::LongitudeChanged(val) => {
                self.longitude_str = val;
                Task::none()
            }
            Message::ApplyChanges => {
                let Some(folder) = &self.folder_path else {
                    self.status = "Error: No folder selected!".to_string();
                    return Task::none();
                };

                let datetime_exif = format!("{} {}", self.date_str, self.time_str);
                let lat: f64 = self.latitude_str.parse().unwrap_or(0.0);
                let lon: f64 = self.longitude_str.parse().unwrap_or(0.0);

                let lat_ref = if lat >= 0.0 { "N" } else { "S" };
                let lon_ref = if lon >= 0.0 { "E" } else { "W" };

                let lat_deg = decimal_to_dms_rationals(lat.abs());
                let lon_deg = decimal_to_dms_rationals(lon.abs());

                let mut count = 0;
                for path in &self.file_list {
                    let mut metadata = Metadata::new();
                    metadata.set_tag(ExifTag::DateTimeOriginal(datetime_exif.clone()));
                    metadata.set_tag(ExifTag::GPSLatitude(lat_deg.clone()));
                    metadata.set_tag(ExifTag::GPSLatitudeRef(lat_ref.to_string()));
                    metadata.set_tag(ExifTag::GPSLongitude(lon_deg.clone()));
                    metadata.set_tag(ExifTag::GPSLongitudeRef(lon_ref.to_string()));

                    if metadata.write_to_file(path).is_ok() {
                        count += 1;
                    }
                }

                self.status = format!("Batch updated metadata for {} archive photos.", count);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- COLUMN 1: File Explorer Panel ---
        let mut file_column = column![
            button("Open Archive Directory").on_press(Message::SelectFolder),
            Space::new().height(10),
            text(format!("Files found: {}", self.file_list.len())).size(12),
            Space::new().height(10),
        ]
        .spacing(5);

        for file in &self.file_list {
            let file_name = file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let is_selected = self.selected_file.as_ref() == Some(file);
            let label = if is_selected {
                format!("▶ {}", file_name)
            } else {
                format!("  {}", file_name)
            };

            file_column = file_column.push(
                button(text(label).size(13))
                    .on_press(Message::SelectFile(file.clone()))
                    .width(Length::Fill),
            );
        }

        let explorer_panel = container(scrollable(file_column))
            .width(240)
            .height(Length::Fill)
            .padding(10);

        // --- COLUMN 2: Genealogy EXIF Metadata Editor Panel ---
        let date_input = row![
            text("Date:").width(90),
            text_input("YYYY:MM:DD", &self.date_str).on_input(Message::DateChanged),
        ]
        .spacing(10);

        let time_input = row![
            text("Time:").width(90),
            text_input("HH:MM:SS", &self.time_str).on_input(Message::TimeChanged),
        ]
        .spacing(10);

        let lat_input = row![
            text("Latitude:").width(90),
            text_input("e.g. 33.9519", &self.latitude_str).on_input(Message::LatitudeChanged),
        ]
        .spacing(10);

        let lon_input = row![
            text("Longitude:").width(90),
            text_input("e.g. -83.3576", &self.longitude_str).on_input(Message::LongitudeChanged),
        ]
        .spacing(10);

        let apply_btn = button("Batch Write EXIF Data")
            .on_press(Message::ApplyChanges)
            .padding(10);

        let editor_panel = container(
            column![
                text("Genealogy EXIF Attributes").size(18),
                Space::new().height(10),
                date_input,
                time_input,
                lat_input,
                lon_input,
                Space::new().height(15),
                apply_btn,
                Space::new().height(15),
                text(&self.status).size(12),
            ]
            .spacing(10),
        )
        .width(320)
        .height(Length::Fill)
        .padding(10);

        // --- COLUMN 3: Dynamic Image Preview Panel ---
        let preview_content: Element<Message> = match &self.selected_file {
            Some(path) => container(
                column![
                    text(format!("Preview: {}", path.file_name().unwrap().to_string_lossy())).size(14),
                    Space::new().height(10),
                    image(path.to_string_lossy().to_string())
                        .width(Length::Fill)
                        .height(Length::Fill),
                ]
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            None => container(text("Select a photo from the left menu to preview").size(14))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
        };

        // Main 3-Column Root Layout
        row![
            explorer_panel,
            container(Space::new().width(1)).height(Length::Fill), // Separator spacer
            editor_panel,
            container(Space::new().width(1)).height(Length::Fill), // Separator spacer
            preview_content,
        ]
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

fn decimal_to_dms_rationals(decimal: f64) -> Vec<uR64> {
    let degrees = decimal.floor() as u32;
    let minutes_full = (decimal - degrees as f64) * 60.0;
    let minutes = minutes_full.floor() as u32;
    let seconds = ((minutes_full - minutes as f64) * 60.0 * 100.0).round() as u32;

    vec![
        uR64 {
            nominator: degrees,
            denominator: 1,
        },
        uR64 {
            nominator: minutes,
            denominator: 1,
        },
        uR64 {
            nominator: seconds,
            denominator: 100,
        },
    ]
}