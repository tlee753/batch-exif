#![windows_subsystem = "windows"]

use iced::font::{self, Font};
use iced::widget::{Space, button, column, container, image, row, scrollable, text, text_input};
use iced::{Color, Element, Length, Task, Theme};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// Lexend Font
const LEXEND_REGULAR_BYTES: &[u8] = include_bytes!("lexend-regular.ttf");
const LEXEND_BOLD_BYTES: &[u8] = include_bytes!("lexend-bold.ttf");
const LEXEND_FONT_NAME: &str = "Lexend";
const LEXEND_REGULAR: Font = Font {
    family: font::Family::Name(LEXEND_FONT_NAME),
    weight: font::Weight::Normal,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};
const LEXEND_BOLD: Font = Font {
    family: font::Family::Name(LEXEND_FONT_NAME),
    weight: font::Weight::Bold,
    stretch: font::Stretch::Normal,
    style: font::Style::Normal,
};

pub fn main() -> iced::Result {
    iced::application(ExifApp::default, ExifApp::update, ExifApp::view)
        .title("EXIF Batch Tool v2.1")
        .theme(|_: &ExifApp| Theme::Dark)
        .font(LEXEND_REGULAR_BYTES) // Register Regular variant into font database
        .font(LEXEND_BOLD_BYTES) // Register Bold variant into font database
        .default_font(LEXEND_REGULAR)
        .window(iced::window::Settings {
            size: iced::Size::new(1600.0, 900.0),
            min_size: Some(iced::Size::new(1000.0, 600.0)),
            ..Default::default()
        })
        .run()
}

#[derive(Default, Clone, PartialEq, Eq)]
struct ExifFields {
    date_str: String,
    time_str: String,
    digitized_date_str: String,
    offset_time_str: String,

    latitude_str: String,
    longitude_str: String,
    altitude_str: String,

    city_str: String,
    state_str: String,
    country_str: String,
    sublocation_str: String,

    caption_str: String,
    people_str: String,
    credit_str: String,
}

struct ExifApp {
    folder_path: Option<PathBuf>,
    file_list: Vec<PathBuf>,
    selected_files: HashSet<PathBuf>,
    fields: ExifFields,
    status: String,
}

#[derive(Debug, Clone)]
enum Message {
    SelectFolder,
    FolderSelected(Option<PathBuf>),
    ToggleFileSelection(PathBuf),
    SelectAllFiles,
    DeselectAllFiles,

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
            selected_files: HashSet::new(),
            fields: ExifFields::default(),
            status: "Select a folder to load archive photos.".to_string(),
        }
    }
}

impl ExifApp {
    fn extract_fields_from_path(path: &PathBuf) -> ExifFields {
        let mut fields = ExifFields::default();

        if let Ok(metadata) = Metadata::new_from_path(path) {
            // Original Date & Time
            if let Some(ExifTag::DateTimeOriginal(val)) = metadata
                .get_tag(&ExifTag::DateTimeOriginal(String::new()))
                .next()
            {
                let parts: Vec<&str> = val.split_whitespace().collect();
                if parts.len() >= 2 {
                    fields.date_str = parts[0].to_string();
                    fields.time_str = parts[1].to_string();
                } else {
                    fields.date_str = val.clone();
                }
            }

            // Digitized / Create Date
            if let Some(ExifTag::CreateDate(val)) =
                metadata.get_tag(&ExifTag::CreateDate(String::new())).next()
            {
                fields.digitized_date_str = val.clone();
            }

            // Timezone Offset
            if let Some(ExifTag::OffsetTimeOriginal(val)) = metadata
                .get_tag(&ExifTag::OffsetTimeOriginal(String::new()))
                .next()
            {
                fields.offset_time_str = val.clone();
            }

            // Latitude
            if let Some(ExifTag::GPSLatitude(rats)) =
                metadata.get_tag(&ExifTag::GPSLatitude(Vec::new())).next()
            {
                if rats.len() >= 3 {
                    let deg = rats[0].nominator as f64 / rats[0].denominator as f64;
                    let min = rats[1].nominator as f64 / rats[1].denominator as f64;
                    let sec = rats[2].nominator as f64 / rats[2].denominator as f64;
                    let mut lat = deg + (min / 60.0) + (sec / 3600.0);

                    if let Some(ExifTag::GPSLatitudeRef(ref_str)) = metadata
                        .get_tag(&ExifTag::GPSLatitudeRef(String::new()))
                        .next()
                    {
                        if ref_str == "S" {
                            lat = -lat;
                        }
                    }
                    fields.latitude_str = format!("{:.6}", lat);
                }
            }

            // Longitude
            if let Some(ExifTag::GPSLongitude(rats)) =
                metadata.get_tag(&ExifTag::GPSLongitude(Vec::new())).next()
            {
                if rats.len() >= 3 {
                    let deg = rats[0].nominator as f64 / rats[0].denominator as f64;
                    let min = rats[1].nominator as f64 / rats[1].denominator as f64;
                    let sec = rats[2].nominator as f64 / rats[2].denominator as f64;
                    let mut lon = deg + (min / 60.0) + (sec / 3600.0);

                    if let Some(ExifTag::GPSLongitudeRef(ref_str)) = metadata
                        .get_tag(&ExifTag::GPSLongitudeRef(String::new()))
                        .next()
                    {
                        if ref_str == "W" {
                            lon = -lon;
                        }
                    }
                    fields.longitude_str = format!("{:.6}", lon);
                }
            }

            // Altitude
            if let Some(ExifTag::GPSAltitude(rats)) =
                metadata.get_tag(&ExifTag::GPSAltitude(Vec::new())).next()
            {
                if let Some(alt) = rats.first() {
                    if alt.denominator != 0 {
                        fields.altitude_str = (alt.nominator / alt.denominator).to_string();
                    }
                }
            }

            // Description / Caption
            if let Some(ExifTag::ImageDescription(val)) = metadata
                .get_tag(&ExifTag::ImageDescription(String::new()))
                .next()
            {
                fields.caption_str = val.clone();
            }

            // Photographer / Credit
            if let Some(ExifTag::Artist(val)) =
                metadata.get_tag(&ExifTag::Artist(String::new())).next()
            {
                fields.credit_str = val.clone();
            }

            // Location Metadata
            if let Some(ExifTag::Software(val)) =
                metadata.get_tag(&ExifTag::Software(String::new())).next()
            {
                let parts: Vec<&str> = val.split('|').collect();
                if parts.len() == 4 {
                    fields.city_str = parts[0].to_string();
                    fields.state_str = parts[1].to_string();
                    fields.country_str = parts[2].to_string();
                    fields.sublocation_str = parts[3].to_string();
                }
            }

            if let Some(ExifTag::UserComment(val)) =
                metadata.get_tag(&ExifTag::UserComment(Vec::new())).next()
            {
                if let Ok(s) = String::from_utf8(val.clone()) {
                    fields.people_str = s.trim_start_matches("ASCII\0\0\0").to_string();
                }
            }
        }

        fields
    }

    fn update_fields_from_selection(&mut self) {
        if self.selected_files.is_empty() {
            self.fields = ExifFields::default();
            return;
        }

        let mut iter = self.selected_files.iter();
        if let Some(first_path) = iter.next() {
            let mut common = Self::extract_fields_from_path(first_path);

            for path in iter {
                let current = Self::extract_fields_from_path(path);

                if common.date_str != current.date_str {
                    common.date_str.clear();
                }
                if common.time_str != current.time_str {
                    common.time_str.clear();
                }
                if common.digitized_date_str != current.digitized_date_str {
                    common.digitized_date_str.clear();
                }
                if common.offset_time_str != current.offset_time_str {
                    common.offset_time_str.clear();
                }

                if common.latitude_str != current.latitude_str {
                    common.latitude_str.clear();
                }
                if common.longitude_str != current.longitude_str {
                    common.longitude_str.clear();
                }
                if common.altitude_str != current.altitude_str {
                    common.altitude_str.clear();
                }

                if common.city_str != current.city_str {
                    common.city_str.clear();
                }
                if common.state_str != current.state_str {
                    common.state_str.clear();
                }
                if common.country_str != current.country_str {
                    common.country_str.clear();
                }
                if common.sublocation_str != current.sublocation_str {
                    common.sublocation_str.clear();
                }

                if common.caption_str != current.caption_str {
                    common.caption_str.clear();
                }
                if common.people_str != current.people_str {
                    common.people_str.clear();
                }
                if common.credit_str != current.credit_str {
                    common.credit_str.clear();
                }
            }

            self.fields = common;
        }
    }

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
                    self.selected_files.clear();

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
                        self.selected_files.insert(first);
                    }
                    self.update_fields_from_selection();
                }
                Task::none()
            }
            Message::ToggleFileSelection(file_path) => {
                if self.selected_files.contains(&file_path) {
                    self.selected_files.remove(&file_path);
                } else {
                    self.selected_files.insert(file_path);
                }

                self.update_fields_from_selection();
                Task::none()
            }
            Message::SelectAllFiles => {
                self.selected_files = self.file_list.iter().cloned().collect();
                self.update_fields_from_selection();
                Task::none()
            }
            Message::DeselectAllFiles => {
                self.selected_files.clear();
                self.update_fields_from_selection();
                Task::none()
            }

            Message::DateChanged(val) => {
                self.fields.date_str = val;
                Task::none()
            }
            Message::TimeChanged(val) => {
                self.fields.time_str = val;
                Task::none()
            }
            Message::DigitizedDateChanged(val) => {
                self.fields.digitized_date_str = val;
                Task::none()
            }
            Message::OffsetTimeChanged(val) => {
                self.fields.offset_time_str = val;
                Task::none()
            }
            Message::LatitudeChanged(val) => {
                self.fields.latitude_str = val;
                Task::none()
            }
            Message::LongitudeChanged(val) => {
                self.fields.longitude_str = val;
                Task::none()
            }
            Message::AltitudeChanged(val) => {
                self.fields.altitude_str = val;
                Task::none()
            }
            Message::CityChanged(val) => {
                self.fields.city_str = val;
                Task::none()
            }
            Message::StateChanged(val) => {
                self.fields.state_str = val;
                Task::none()
            }
            Message::CountryChanged(val) => {
                self.fields.country_str = val;
                Task::none()
            }
            Message::SublocationChanged(val) => {
                self.fields.sublocation_str = val;
                Task::none()
            }
            Message::CaptionChanged(val) => {
                self.fields.caption_str = val;
                Task::none()
            }
            Message::PeopleChanged(val) => {
                self.fields.people_str = val;
                Task::none()
            }
            Message::CreditChanged(val) => {
                self.fields.credit_str = val;
                Task::none()
            }

            Message::ApplyChanges => {
                if self.folder_path.is_none() {
                    self.status = "Error: No folder selected!".to_string();
                    return Task::none();
                }

                if self.selected_files.is_empty() {
                    self.status = "Error: No target files selected to apply changes!".to_string();
                    return Task::none();
                }

                let lat_opt: Option<f64> = self.fields.latitude_str.trim().parse().ok();
                let lon_opt: Option<f64> = self.fields.longitude_str.trim().parse().ok();
                let alt_opt: Option<u32> = self.fields.altitude_str.trim().parse().ok();

                let mut success_count = 0;
                let mut error_count = 0;

                for path in &self.selected_files {
                    let mut metadata =
                        Metadata::new_from_path(path).unwrap_or_else(|_| Metadata::new());

                    // --- Dates & Times ---
                    if !self.fields.date_str.trim().is_empty()
                        || !self.fields.time_str.trim().is_empty()
                    {
                        let mut curr_date = String::new();
                        let mut curr_time = String::new();
                        if let Some(ExifTag::DateTimeOriginal(val)) = metadata
                            .get_tag(&ExifTag::DateTimeOriginal(String::new()))
                            .next()
                        {
                            let parts: Vec<&str> = val.split_whitespace().collect();
                            if parts.len() >= 2 {
                                curr_date = parts[0].to_string();
                                curr_time = parts[1].to_string();
                            } else {
                                curr_date = val.clone();
                            }
                        }

                        let new_date = if !self.fields.date_str.trim().is_empty() {
                            self.fields.date_str.trim()
                        } else {
                            &curr_date
                        };

                        let new_time = if !self.fields.time_str.trim().is_empty() {
                            self.fields.time_str.trim()
                        } else {
                            &curr_time
                        };

                        metadata.set_tag(ExifTag::DateTimeOriginal(format!(
                            "{} {}",
                            new_date, new_time
                        )));
                    }

                    if !self.fields.digitized_date_str.trim().is_empty() {
                        metadata.set_tag(ExifTag::CreateDate(
                            self.fields.digitized_date_str.trim().to_string(),
                        ));
                    }

                    if !self.fields.offset_time_str.trim().is_empty() {
                        metadata.set_tag(ExifTag::OffsetTimeOriginal(
                            self.fields.offset_time_str.trim().to_string(),
                        ));
                    }

                    // --- GPS Coordinates ---
                    if let Some(lat) = lat_opt {
                        let lat_ref = if lat >= 0.0 { "N" } else { "S" };
                        metadata.set_tag(ExifTag::GPSLatitude(decimal_to_dms_rationals(lat.abs())));
                        metadata.set_tag(ExifTag::GPSLatitudeRef(lat_ref.to_string()));
                    }

                    if let Some(lon) = lon_opt {
                        let lon_ref = if lon >= 0.0 { "E" } else { "W" };
                        metadata
                            .set_tag(ExifTag::GPSLongitude(decimal_to_dms_rationals(lon.abs())));
                        metadata.set_tag(ExifTag::GPSLongitudeRef(lon_ref.to_string()));
                    }

                    if let Some(alt) = alt_opt {
                        let alt_rat = vec![uR64 {
                            nominator: alt,
                            denominator: 1,
                        }];
                        metadata.set_tag(ExifTag::GPSAltitude(alt_rat));
                        metadata.set_tag(ExifTag::GPSAltitudeRef(vec![0]));
                    }

                    // --- Description & Photographer ---
                    if !self.fields.caption_str.trim().is_empty() {
                        metadata.set_tag(ExifTag::ImageDescription(
                            self.fields.caption_str.trim().to_string(),
                        ));
                    }

                    if !self.fields.credit_str.trim().is_empty() {
                        metadata
                            .set_tag(ExifTag::Artist(self.fields.credit_str.trim().to_string()));
                    }

                    // --- Selective Location Update ---
                    let has_city = !self.fields.city_str.trim().is_empty();
                    let has_state = !self.fields.state_str.trim().is_empty();
                    let has_country = !self.fields.country_str.trim().is_empty();
                    let has_sublocation = !self.fields.sublocation_str.trim().is_empty();

                    if has_city || has_state || has_country || has_sublocation {
                        let mut curr_city = String::new();
                        let mut curr_state = String::new();
                        let mut curr_country = String::new();
                        let mut curr_sublocation = String::new();

                        if let Some(ExifTag::Software(val)) =
                            metadata.get_tag(&ExifTag::Software(String::new())).next()
                        {
                            let parts: Vec<&str> = val.split('|').collect();
                            if parts.len() == 4 {
                                curr_city = parts[0].to_string();
                                curr_state = parts[1].to_string();
                                curr_country = parts[2].to_string();
                                curr_sublocation = parts[3].to_string();
                            }
                        }

                        let final_city = if has_city {
                            self.fields.city_str.trim()
                        } else {
                            &curr_city
                        };
                        let final_state = if has_state {
                            self.fields.state_str.trim()
                        } else {
                            &curr_state
                        };
                        let final_country = if has_country {
                            self.fields.country_str.trim()
                        } else {
                            &curr_country
                        };
                        let final_sublocation = if has_sublocation {
                            self.fields.sublocation_str.trim()
                        } else {
                            &curr_sublocation
                        };

                        let loc_payload = format!(
                            "{}|{}|{}|{}",
                            final_city, final_state, final_country, final_sublocation
                        );
                        metadata.set_tag(ExifTag::Software(loc_payload));
                    }

                    // --- People Tag ---
                    if !self.fields.people_str.trim().is_empty() {
                        let mut comment_bytes =
                            vec![0x41, 0x53, 0x43, 0x49, 0x49, 0x00, 0x00, 0x00];
                        comment_bytes.extend(self.fields.people_str.trim().bytes());
                        metadata.set_tag(ExifTag::UserComment(comment_bytes));
                    }

                    match metadata.write_to_file(path) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            eprintln!("Failed to write EXIF for {:?}: {:?}", path, e);
                            error_count += 1;
                        }
                    }
                }

                self.update_fields_from_selection();

                if error_count > 0 {
                    self.status = format!(
                        "Updated {} file(s), failed on {} file(s).",
                        success_count, error_count
                    );
                } else {
                    self.status = format!("Successfully saved EXIF to {} file(s).", success_count);
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let highlight_green = Color::from_rgb8(0x00, 0xFF, 0xAF);
        let border_gray = Color::from_rgb8(0x33, 0x33, 0x33);

        let txt = |content: String| text(content).font(LEXEND_REGULAR);
        let txt_str = |content: &'static str| text(content).font(LEXEND_REGULAR);

        let bold_title = |label: &'static str| {
            container(
                text(label)
                    .size(15)
                    .color(highlight_green)
                    .font(LEXEND_BOLD),
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
        };

        let white_button_style = |_theme: &Theme, _status: button::Status| button::Style {
            background: Some(iced::Background::Color(Color::WHITE)),
            text_color: Color::BLACK,
            border: iced::Border::default().rounded(4.0),
            ..Default::default()
        };

        let small_button_style = |_theme: &Theme, _status: button::Status| button::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x33, 0x33, 0x33))),
            text_color: Color::WHITE,
            border: iced::Border::default().rounded(4.0),
            ..Default::default()
        };

        let custom_input_style = move |_theme: &Theme, status: text_input::Status| match status {
            text_input::Status::Focused { .. } => text_input::Style {
                background: iced::Background::Color(Color::from_rgb8(0x18, 0x18, 0x18)),
                border: iced::Border::default()
                    .rounded(4.0)
                    .width(1.5)
                    .color(highlight_green),
                icon: Color::from_rgb8(0x88, 0x88, 0x88),
                placeholder: Color::from_rgb8(0x66, 0x66, 0x66),
                value: Color::WHITE,
                selection: Color::from_rgb8(0x00, 0x55, 0x3A),
            },
            text_input::Status::Hovered => text_input::Style {
                background: iced::Background::Color(Color::from_rgb8(0x1E, 0x1E, 0x1E)),
                border: iced::Border::default()
                    .rounded(4.0)
                    .width(1.0)
                    .color(Color::from_rgb8(0x55, 0x55, 0x55)),
                icon: Color::from_rgb8(0x88, 0x88, 0x88),
                placeholder: Color::from_rgb8(0x66, 0x66, 0x66),
                value: Color::WHITE,
                selection: Color::from_rgb8(0x00, 0x55, 0x3A),
            },
            _ => text_input::Style {
                background: iced::Background::Color(Color::from_rgb8(0x18, 0x18, 0x18)),
                border: iced::Border::default()
                    .rounded(4.0)
                    .width(1.0)
                    .color(border_gray),
                icon: Color::from_rgb8(0x88, 0x88, 0x88),
                placeholder: Color::from_rgb8(0x66, 0x66, 0x66),
                value: Color::WHITE,
                selection: Color::from_rgb8(0x00, 0x55, 0x3A),
            },
        };

        let custom_scrollable_style = move |_theme: &Theme, status: scrollable::Status| {
            let scroller_bg = match status {
                scrollable::Status::Hovered { .. } | scrollable::Status::Dragged { .. } => {
                    Color::from_rgb8(0x33, 0xFF, 0xC4)
                }
                _ => highlight_green,
            };

            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: scrollable::Rail {
                    background: None,
                    border: iced::Border::default(),
                    scroller: scrollable::Scroller {
                        background: iced::Background::Color(scroller_bg),
                        border: iced::Border::default().rounded(2.0),
                    },
                },
                horizontal_rail: scrollable::Rail {
                    background: None,
                    border: iced::Border::default(),
                    scroller: scrollable::Scroller {
                        background: iced::Background::Color(Color::TRANSPARENT),
                        border: iced::Border::default(),
                    },
                },
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: iced::Background::Color(highlight_green),
                    border: iced::Border::default().rounded(2.0),
                    shadow: iced::Shadow::default(),
                    icon: Color::BLACK,
                },
            }
        };

        // --- COLUMN 1: File Explorer Panel ---
        let open_folder_btn = button(
            container(text("Open Folder").font(LEXEND_BOLD).color(Color::BLACK))
                .width(Length::Fill)
                .height(Length::Fixed(36.0))
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .on_press(Message::SelectFolder)
        .width(Length::Fill)
        .padding(0)
        .style(white_button_style);

        let select_all_btn = button(text("Select All").size(11).font(LEXEND_REGULAR))
            .on_press(Message::SelectAllFiles)
            .style(small_button_style)
            .padding(4);

        let deselect_all_btn = button(text("Clear").size(11).font(LEXEND_REGULAR))
            .on_press(Message::DeselectAllFiles)
            .style(small_button_style)
            .padding(4);

        let explorer_header = column![
            open_folder_btn,
            Space::new().height(10),
            txt(self.status.clone()).size(12),
            Space::new().height(5),
            row![
                txt(format!(
                    "Selected: {}/{}",
                    self.selected_files.len(),
                    self.file_list.len()
                ))
                .size(12),
                Space::new().width(Length::Fill),
                select_all_btn,
                deselect_all_btn,
            ]
            .align_y(iced::Alignment::Center)
            .spacing(5),
            Space::new().height(10),
        ]
        .spacing(5);

        let mut file_items = column![].spacing(5).width(Length::Fill);

        for file in &self.file_list {
            let file_name = file
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let is_selected = self.selected_files.contains(file);
            let target_path = file.clone();

            let item_button = button(
                container(
                    text(file_name)
                        .font(LEXEND_REGULAR)
                        .size(14)
                        .width(Length::Fill)
                        .wrapping(iced::widget::text::Wrapping::Word)
                        .color(if is_selected {
                            Color::BLACK
                        } else {
                            Color::WHITE
                        }),
                )
                .width(Length::Fixed(204.0))
                .padding(iced::Padding {
                    top: 6.0,
                    bottom: 6.0,
                    left: 8.0,
                    right: 8.0,
                })
                .align_x(iced::alignment::Horizontal::Left)
                .align_y(iced::alignment::Vertical::Center),
            )
            .on_press(Message::ToggleFileSelection(target_path))
            .width(Length::Fixed(220.0))
            .clip(false)
            .padding(0)
            .style(move |_theme, _status| {
                if is_selected {
                    button::Style {
                        background: Some(iced::Background::Color(highlight_green)),
                        border: iced::Border::default().rounded(4.0),
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        background: None,
                        border: iced::Border::default()
                            .rounded(4.0)
                            .width(1.0)
                            .color(border_gray),
                        ..Default::default()
                    }
                }
            });

            file_items = file_items.push(item_button);
        }

        let scrollable_files = scrollable(file_items)
            .style(custom_scrollable_style)
            .spacing(6)
            .width(Length::Fill)
            .height(Length::Fill);

        let explorer_panel = container(column![explorer_header, scrollable_files,])
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .clip(false)
            .padding(iced::Padding {
                top: 10.0,
                bottom: 10.0,
                left: 10.0,
                right: 10.0,
            });

        // --- COLUMN 2: Full EXIF & Metadata Form ---
        let apply_btn_label = if self.selected_files.len() > 1 {
            "Batch Apply Attributes"
        } else {
            "Apply Attributes"
        };

        let apply_batch_btn = button(
            container(text(apply_btn_label).font(LEXEND_BOLD).color(Color::BLACK))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
        )
        .on_press(Message::ApplyChanges)
        .width(Length::Fill)
        .padding(10)
        .style(white_button_style);

        let make_input_row = |label: &'static str,
                              placeholder: &'static str,
                              val: &str,
                              on_change: fn(String) -> Message| {
            row![
                txt_str(label).width(120),
                text_input(placeholder, val)
                    .font(LEXEND_REGULAR)
                    .on_input(on_change)
                    .style(custom_input_style)
            ]
            .align_y(iced::Alignment::Center)
        };

        let editor_column = column![
            bold_title("Dates & Timestamps"),
            make_input_row(
                "Original Date:",
                "YYYY:MM:DD",
                &self.fields.date_str,
                Message::DateChanged
            ),
            make_input_row(
                "Original Time:",
                "HH:MM:SS",
                &self.fields.time_str,
                Message::TimeChanged
            ),
            make_input_row(
                "Digitized Date:",
                "YYYY:MM:DD",
                &self.fields.digitized_date_str,
                Message::DigitizedDateChanged
            ),
            make_input_row(
                "UTC Offset:",
                "-05:00",
                &self.fields.offset_time_str,
                Message::OffsetTimeChanged
            ),
            Space::new().height(10),
            bold_title("GPS Coordinates"),
            make_input_row(
                "Latitude:",
                "Decimal Lat",
                &self.fields.latitude_str,
                Message::LatitudeChanged
            ),
            make_input_row(
                "Longitude:",
                "Decimal Lon",
                &self.fields.longitude_str,
                Message::LongitudeChanged
            ),
            make_input_row(
                "Altitude:",
                "Meters",
                &self.fields.altitude_str,
                Message::AltitudeChanged
            ),
            Space::new().height(10),
            bold_title("Location Details"),
            make_input_row("City:", "City", &self.fields.city_str, Message::CityChanged),
            make_input_row(
                "State:",
                "State",
                &self.fields.state_str,
                Message::StateChanged
            ),
            make_input_row(
                "Country:",
                "Country",
                &self.fields.country_str,
                Message::CountryChanged
            ),
            make_input_row(
                "Sub-Location:",
                "Address / Landmark",
                &self.fields.sublocation_str,
                Message::SublocationChanged
            ),
            Space::new().height(10),
            bold_title("People & Context"),
            make_input_row(
                "Description:",
                "Description",
                &self.fields.caption_str,
                Message::CaptionChanged
            ),
            make_input_row(
                "People:",
                "Names",
                &self.fields.people_str,
                Message::PeopleChanged
            ),
            make_input_row(
                "Credit:",
                "Photographer",
                &self.fields.credit_str,
                Message::CreditChanged
            ),
            Space::new().height(15),
            apply_batch_btn,
        ]
        .spacing(8);

        let editor_panel = container(
            scrollable(editor_column)
                .style(custom_scrollable_style)
                .spacing(6),
        )
        .width(Length::Fixed(360.0))
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 10.0,
            bottom: 10.0,
            left: 10.0,
            right: 10.0,
        });

        // --- COLUMN 3: Image Preview Panel ---
        let primary_selected = self
            .file_list
            .iter()
            .rfind(|p| self.selected_files.contains(*p));

        let preview_content: Element<Message> = match primary_selected {
            Some(path) => container(
                column![
                    txt(format!(
                        "Preview: {}",
                        path.file_name().unwrap().to_string_lossy()
                    ))
                    .size(14),
                    Space::new().height(10),
                    image(path.to_string_lossy().to_string())
                        .width(Length::Fill)
                        .height(Length::Fill),
                ]
                .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            None => container(txt_str("Select a photo from the left menu to preview").size(14))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .into(),
        };

        // Root 3-Column Layout
        let content = row![
            explorer_panel,
            container(Space::new().width(1)).height(Length::Fill),
            editor_panel,
            container(Space::new().width(1)).height(Length::Fill),
            preview_content,
        ]
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style {
                background: Some(iced::Background::Color(Color::BLACK)),
                ..Default::default()
            })
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
