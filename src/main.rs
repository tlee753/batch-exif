use iced::widget::{Space, button, column, row, text, text_input};
use iced::{Alignment, Element, Task};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;
use std::fs;
use std::path::PathBuf;

pub fn main() -> iced::Result {
    iced::application(ExifApp::default, ExifApp::update, ExifApp::view)
        .title("EXIF Batch Updater")
        .run()
}

struct ExifApp {
    folder_path: Option<PathBuf>,
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
            date_str: "2026:09:01".to_string(),
            time_str: "12:00:00".to_string(),
            latitude_str: "33.9519".to_string(),
            longitude_str: "-83.3576".to_string(),
            status: "Select a folder to begin.".to_string(),
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
                    self.status = format!("Selected: {}", p.display());
                    self.folder_path = Some(p);
                }
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
                if let Ok(entries) = fs::read_dir(folder) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png" {
                                let mut metadata = Metadata::new();

                                metadata.set_tag(ExifTag::DateTimeOriginal(datetime_exif.clone()));
                                metadata.set_tag(ExifTag::GPSLatitude(lat_deg.clone()));
                                metadata.set_tag(ExifTag::GPSLatitudeRef(lat_ref.to_string()));
                                metadata.set_tag(ExifTag::GPSLongitude(lon_deg.clone()));
                                metadata.set_tag(ExifTag::GPSLongitudeRef(lon_ref.to_string()));

                                if metadata.write_to_file(&path).is_ok() {
                                    count += 1;
                                }
                            }
                        }
                    }
                }

                self.status = format!("Updated EXIF data for {} files.", count);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let folder_label = match &self.folder_path {
            Some(p) => p.to_string_lossy().to_string(),
            None => "No folder selected".to_string(),
        };

        let folder_row = row![
            button("Choose Folder").on_press(Message::SelectFolder),
            text(folder_label),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let date_input = row![
            text("Date (YYYY:MM:DD):").width(150),
            text_input("YYYY:MM:DD", &self.date_str).on_input(Message::DateChanged),
        ]
        .spacing(10);

        let time_input = row![
            text("Time (HH:MM:SS):").width(150),
            text_input("HH:MM:SS", &self.time_str).on_input(Message::TimeChanged),
        ]
        .spacing(10);

        let lat_input = row![
            text("Latitude (decimal):").width(150),
            text_input("e.g. 33.9519", &self.latitude_str).on_input(Message::LatitudeChanged),
        ]
        .spacing(10);

        let lon_input = row![
            text("Longitude (decimal):").width(150),
            text_input("e.g. -83.3576", &self.longitude_str).on_input(Message::LongitudeChanged),
        ]
        .spacing(10);

        let apply_btn = button("Batch Apply EXIF Updates")
            .on_press(Message::ApplyChanges)
            .padding(10);

        column![
            text("Batch Photo EXIF Editor").size(24),
            Space::new().height(10),
            folder_row,
            Space::new().height(10),
            date_input,
            time_input,
            lat_input,
            lon_input,
            Space::new().height(15),
            apply_btn,
            Space::new().height(15),
            text(&self.status),
        ]
        .padding(20)
        .spacing(10)
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
