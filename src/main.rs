use iced::widget::{Space, button, column, container, image, row, scrollable, text, text_input};
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

    // Temporal Metadata
    date_str: String,
    time_str: String,
    digitized_date_str: String,
    offset_time_str: String,

    // Location (GPS) Metadata
    latitude_str: String,
    longitude_str: String,
    altitude_str: String,

    // Descriptive Location Metadata
    city_str: String,
    state_str: String,
    country_str: String,
    sublocation_str: String,

    // Biographical & Archival Metadata
    caption_str: String,
    people_str: String,
    credit_str: String,

    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectFolder,
    FolderSelected(Option<PathBuf>),
    SelectFile(PathBuf),

    // Form Input Messages
    DateChanged(String),
    TimeChanged(String),
    DigitizedDateChanged(String),
    OffsetTimeChanged(String),
    LatitudeChanged(String),
    LongitudeChanged(String),
    AltitudeChanged(String),
    CityChanged(String),
    StateChanged(String),
    CountryChanged(String),
    SublocationChanged(String),
    CaptionChanged(String),
    PeopleChanged(String),
    CreditChanged(String),

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
            digitized_date_str: "2026:01:10".to_string(),
            offset_time_str: "-05:00".to_string(),

            latitude_str: "33.9519".to_string(),
            longitude_str: "-83.3576".to_string(),
            altitude_str: "200".to_string(),

            city_str: "Athens".to_string(),
            state_str: "Georgia".to_string(),
            country_str: "United States".to_string(),
            sublocation_str: "Family Homestead, 124 Main St".to_string(),

            caption_str: "Family gathering on the porch during summer.".to_string(),
            people_str: "John Doe, Mary Doe, James Smith".to_string(),
            credit_str: "Doe Family Archives".to_string(),

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

            // Input handlers
            Message::DateChanged(val) => {
                self.date_str = val;
                Task::none()
            }
            Message::TimeChanged(val) => {
                self.time_str = val;
                Task::none()
            }
            Message::DigitizedDateChanged(val) => {
                self.digitized_date_str = val;
                Task::none()
            }
            Message::OffsetTimeChanged(val) => {
                self.offset_time_str = val;
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
            Message::AltitudeChanged(val) => {
                self.altitude_str = val;
                Task::none()
            }
            Message::CityChanged(val) => {
                self.city_str = val;
                Task::none()
            }
            Message::StateChanged(val) => {
                self.state_str = val;
                Task::none()
            }
            Message::CountryChanged(val) => {
                self.country_str = val;
                Task::none()
            }
            Message::SublocationChanged(val) => {
                self.sublocation_str = val;
                Task::none()
            }
            Message::CaptionChanged(val) => {
                self.caption_str = val;
                Task::none()
            }
            Message::PeopleChanged(val) => {
                self.people_str = val;
                Task::none()
            }
            Message::CreditChanged(val) => {
                self.credit_str = val;
                Task::none()
            }

            Message::ApplyChanges => {
                if self.folder_path.is_none() {
                    self.status = "Error: No folder selected!".to_string();
                    return Task::none();
                }

                let datetime_exif = format!("{} {}", self.date_str, self.time_str);
                let lat: f64 = self.latitude_str.parse().unwrap_or(0.0);
                let lon: f64 = self.longitude_str.parse().unwrap_or(0.0);
                let alt: u32 = self.altitude_str.parse().unwrap_or(0);

                let lat_ref = if lat >= 0.0 { "N" } else { "S" };
                let lon_ref = if lon >= 0.0 { "E" } else { "W" };

                let lat_deg = decimal_to_dms_rationals(lat.abs());
                let lon_deg = decimal_to_dms_rationals(lon.abs());
                let alt_rat = vec![uR64 {
                    nominator: alt,
                    denominator: 1,
                }];

                let mut count = 0;
                for path in &self.file_list {
                    let mut metadata = Metadata::new();

                    // Standard EXIF Metadata
                    metadata.set_tag(ExifTag::DateTimeOriginal(datetime_exif.clone()));
                    metadata.set_tag(ExifTag::CreateDate(self.digitized_date_str.clone()));
                    metadata.set_tag(ExifTag::OffsetTimeOriginal(self.offset_time_str.clone()));
                    metadata.set_tag(ExifTag::GPSLatitude(lat_deg.clone()));
                    metadata.set_tag(ExifTag::GPSLatitudeRef(lat_ref.to_string()));
                    metadata.set_tag(ExifTag::GPSLongitude(lon_deg.clone()));
                    metadata.set_tag(ExifTag::GPSLongitudeRef(lon_ref.to_string()));
                    metadata.set_tag(ExifTag::GPSAltitude(alt_rat.clone()));
                    metadata.set_tag(ExifTag::GPSAltitudeRef(vec![0])); // 0 = Above Sea Level

                    // Image/Archival Description
                    metadata.set_tag(ExifTag::ImageDescription(self.caption_str.clone()));
                    metadata.set_tag(ExifTag::Artist(self.credit_str.clone()));

                    if metadata.write_to_file(path).is_ok() {
                        count += 1;
                    }
                }

                self.status = format!("Updated genealogy metadata for {} files.", count);
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
            .width(220)
            .height(Length::Fill)
            .padding(10);

        // --- COLUMN 2: Full Genealogy EXIF & Metadata Form ---
        let editor_column = column![
            text("Genealogy Metadata").size(18),
            Space::new().height(5),
            // Temporal Fields
            text("Dates & Timestamps").size(14),
            row![
                text("Original Date:").width(110),
                text_input("YYYY:MM:DD", &self.date_str).on_input(Message::DateChanged)
            ],
            row![
                text("Original Time:").width(110),
                text_input("HH:MM:SS", &self.time_str).on_input(Message::TimeChanged)
            ],
            row![
                text("Digitized Date:").width(110),
                text_input("YYYY:MM:DD", &self.digitized_date_str)
                    .on_input(Message::DigitizedDateChanged)
            ],
            row![
                text("UTC Offset:").width(110),
                text_input("-05:00", &self.offset_time_str).on_input(Message::OffsetTimeChanged)
            ],
            Space::new().height(10),
            // Location GPS Fields
            text("GPS Coordinates").size(14),
            row![
                text("Latitude:").width(110),
                text_input("Decimal Lat", &self.latitude_str).on_input(Message::LatitudeChanged)
            ],
            row![
                text("Longitude:").width(110),
                text_input("Decimal Lon", &self.longitude_str).on_input(Message::LongitudeChanged)
            ],
            row![
                text("Altitude (m):").width(110),
                text_input("Meters", &self.altitude_str).on_input(Message::AltitudeChanged)
            ],
            Space::new().height(10),
            // Location Descriptive Fields
            text("Location Details").size(14),
            row![
                text("City:").width(110),
                text_input("City Name", &self.city_str).on_input(Message::CityChanged)
            ],
            row![
                text("State/Province:").width(110),
                text_input("State / Prov", &self.state_str).on_input(Message::StateChanged)
            ],
            row![
                text("Country:").width(110),
                text_input("Country", &self.country_str).on_input(Message::CountryChanged)
            ],
            row![
                text("Sub-Location:").width(110),
                text_input("Address / Landmark", &self.sublocation_str)
                    .on_input(Message::SublocationChanged)
            ],
            Space::new().height(10),
            // Biographical & Archival Fields
            text("People & Context").size(14),
            row![
                text("Caption / Notes:").width(110),
                text_input("Photo Description", &self.caption_str)
                    .on_input(Message::CaptionChanged)
            ],
            row![
                text("People in Image:").width(110),
                text_input("Names", &self.people_str).on_input(Message::PeopleChanged)
            ],
            row![
                text("Credit / Source:").width(110),
                text_input("Archive Owner", &self.credit_str).on_input(Message::CreditChanged)
            ],
            Space::new().height(15),
            button("Batch Apply All Attributes")
                .on_press(Message::ApplyChanges)
                .padding(10),
            Space::new().height(10),
            text(&self.status).size(12),
        ]
        .spacing(8);

        let editor_panel = container(scrollable(editor_column))
            .width(360)
            .height(Length::Fill)
            .padding(10);

        // --- COLUMN 3: Image Preview Panel ---
        let preview_content: Element<Message> = match &self.selected_file {
            Some(path) => container(
                column![
                    text(format!(
                        "Preview: {}",
                        path.file_name().unwrap().to_string_lossy()
                    ))
                    .size(14),
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

        // Root 3-Column Layout
        row![
            explorer_panel,
            container(Space::new().width(1)).height(Length::Fill),
            editor_panel,
            container(Space::new().width(1)).height(Length::Fill),
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
