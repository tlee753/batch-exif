#![windows_subsystem = "windows"]

use bytes::Bytes;
use iced::event::{self, Event};
use iced::font::{self, Font};
use iced::keyboard::{self, key::Key};
use iced::widget::{
    button, column, container, image, row, scrollable, space::Space, text, text_input,
};
use iced::{Color, Element, Length, Subscription, Task, Theme};
use img_parts::jpeg::{Jpeg, JpegSegment};
use img_parts::png::{Png, PngChunk};
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::rational::uR64;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

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

const XMP_HEADER: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";
const PNG_XMP_KEYWORD: &[u8] = b"XML:com.adobe.xmp\0";

pub fn main() -> iced::Result {
    iced::application(ExifApp::default, ExifApp::update, ExifApp::view)
        .title("EXIF + XMP Batch Editor (JPEG & PNG) v3.5")
        .subscription(ExifApp::subscription)
        .theme(|_: &ExifApp| Theme::Dark)
        .font(LEXEND_REGULAR_BYTES)
        .font(LEXEND_BOLD_BYTES)
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
    // Standard EXIF
    date_str: String,
    time_str: String,
    digitized_date_str: String,
    offset_time_str: String,
    latitude_str: String,
    longitude_str: String,
    altitude_str: String,
    caption_str: String,
    credit_str: String,

    // Extended XMP Fields
    city_str: String,
    state_str: String,
    country_str: String,
    sublocation_str: String,
    people_str: String,
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
    EventOccurred(Event),
}

impl Default for ExifApp {
    fn default() -> Self {
        Self {
            folder_path: None,
            file_list: Vec::new(),
            selected_files: HashSet::new(),
            fields: ExifFields::default(),
            status: "Select a folder to manage image metadata.".to_string(),
        }
    }
}

impl ExifApp {
    fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    fn extract_fields_from_path(path: &PathBuf) -> ExifFields {
        let mut fields = ExifFields::default();

        // 1. Read EXIF via little_exif
        if let Ok(metadata) = Metadata::new_from_path(path) {
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

            if let Some(ExifTag::CreateDate(val)) =
                metadata.get_tag(&ExifTag::CreateDate(String::new())).next()
            {
                fields.digitized_date_str = val.clone();
            }

            if let Some(ExifTag::OffsetTimeOriginal(val)) = metadata
                .get_tag(&ExifTag::OffsetTimeOriginal(String::new()))
                .next()
            {
                fields.offset_time_str = val.clone();
            }

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

            if let Some(ExifTag::GPSAltitude(rats)) =
                metadata.get_tag(&ExifTag::GPSAltitude(Vec::new())).next()
            {
                if let Some(alt) = rats.first() {
                    if alt.denominator != 0 {
                        fields.altitude_str = (alt.nominator / alt.denominator).to_string();
                    }
                }
            }

            if let Some(ExifTag::ImageDescription(val)) = metadata
                .get_tag(&ExifTag::ImageDescription(String::new()))
                .next()
            {
                fields.caption_str = val.clone();
            }

            if let Some(ExifTag::Artist(val)) =
                metadata.get_tag(&ExifTag::Artist(String::new())).next()
            {
                fields.credit_str = val.clone();
            }
        }

        // 2. Read Extended Fields from XMP via img-parts
        if let Ok(file_bytes) = fs::read(path) {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let xmp_string: Option<String> = if ext == "png" {
                if let Ok(png) = Png::from_bytes(Bytes::from(file_bytes)) {
                    png.chunks().iter().find_map(|chunk| {
                        if chunk.kind() == *b"iTXt" {
                            let raw = chunk.contents();
                            if let Some(pos) = raw
                                .windows(11)
                                .position(|w| w == b"<x:xmpmeta " || w == b"<rdf:RDF ")
                            {
                                std::str::from_utf8(&raw[pos..]).ok().map(|s| s.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                if let Ok(jpeg) = Jpeg::from_bytes(Bytes::from(file_bytes)) {
                    jpeg.segments().iter().find_map(|segment| {
                        let contents = segment.contents();
                        if contents.starts_with(XMP_HEADER) {
                            std::str::from_utf8(&contents[XMP_HEADER.len()..])
                                .ok()
                                .map(|s| s.to_string())
                        } else if let Some(pos) = contents
                            .windows(11)
                            .position(|w| w == b"<x:xmpmeta " || w == b"<rdf:RDF ")
                        {
                            std::str::from_utf8(&contents[pos..])
                                .ok()
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            };

            if let Some(xmp_str) = xmp_string {
                fields.city_str = parse_xmp_tag(&xmp_str, "photoshop:City");
                fields.state_str = parse_xmp_tag(&xmp_str, "photoshop:State");
                fields.country_str = parse_xmp_tag(&xmp_str, "photoshop:Country");
                fields.sublocation_str = parse_xmp_tag(&xmp_str, "iptcCore:Location");
                fields.people_str = parse_xmp_bag(&xmp_str, "dc:subject");
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
                if common.caption_str != current.caption_str {
                    common.caption_str.clear();
                }
                if common.credit_str != current.credit_str {
                    common.credit_str.clear();
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
                if common.people_str != current.people_str {
                    common.people_str.clear();
                }
            }

            self.fields = common;
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyPressed {
                key, ..
            })) => {
                match key.as_ref() {
                    Key::Character("a") | Key::Character("A") => {
                        return self.update(Message::SelectAllFiles);
                    }
                    Key::Character("n") | Key::Character("N") | Key::Character("d") | Key::Character("D") => {
                        return self.update(Message::DeselectAllFiles);
                    }
                    Key::Character("o") | Key::Character("O") => {
                        return self.update(Message::SelectFolder);
                    }
                    _ => Task::none(),
                }
            }
            Message::EventOccurred(_) => Task::none(),

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
                if self.folder_path.is_none() || self.selected_files.is_empty() {
                    self.status = "Error: No files selected!".to_string();
                    return Task::none();
                }

                let lat_opt: Option<f64> = self.fields.latitude_str.trim().parse().ok();
                let lon_opt: Option<f64> = self.fields.longitude_str.trim().parse().ok();
                let alt_opt: Option<u32> = self.fields.altitude_str.trim().parse().ok();

                let mut success_count = 0;
                let mut error_count = 0;

                for path in &self.selected_files {
                    let existing = Self::extract_fields_from_path(path);

                    let final_city = if !self.fields.city_str.trim().is_empty() {
                        self.fields.city_str.trim()
                    } else {
                        &existing.city_str
                    };

                    let final_state = if !self.fields.state_str.trim().is_empty() {
                        self.fields.state_str.trim()
                    } else {
                        &existing.state_str
                    };

                    let final_country = if !self.fields.country_str.trim().is_empty() {
                        self.fields.country_str.trim()
                    } else {
                        &existing.country_str
                    };

                    let final_sublocation = if !self.fields.sublocation_str.trim().is_empty() {
                        self.fields.sublocation_str.trim()
                    } else {
                        &existing.sublocation_str
                    };

                    let final_people_raw = if !self.fields.people_str.trim().is_empty() {
                        self.fields.people_str.trim()
                    } else {
                        &existing.people_str
                    };

                    // --- STEP 1: Write EXIF via little_exif ---
                    let mut metadata =
                        Metadata::new_from_path(path).unwrap_or_else(|_| Metadata::new());

                    if !self.fields.date_str.trim().is_empty()
                        || !self.fields.time_str.trim().is_empty()
                    {
                        let new_date = if !self.fields.date_str.trim().is_empty() {
                            self.fields.date_str.trim()
                        } else {
                            &existing.date_str
                        };
                        let new_time = if !self.fields.time_str.trim().is_empty() {
                            self.fields.time_str.trim()
                        } else {
                            &existing.time_str
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

                    if !self.fields.caption_str.trim().is_empty() {
                        metadata.set_tag(ExifTag::ImageDescription(
                            self.fields.caption_str.trim().to_string(),
                        ));
                    }

                    if !self.fields.credit_str.trim().is_empty() {
                        metadata
                            .set_tag(ExifTag::Artist(self.fields.credit_str.trim().to_string()));
                    }

                    let _ = metadata.write_to_file(path);

                    // --- STEP 2: Write Extended XMP via img-parts ---
                    if let Ok(raw_bytes) = fs::read(path) {
                        let ext = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        let people_list: Vec<&str> = final_people_raw
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();

                        let people_rdf = people_list
                            .iter()
                            .map(|p| format!("<rdf:li>{}</rdf:li>", p))
                            .collect::<String>();

                        let xmp_xml = format!(
                            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="Adobe XMP Core 5.6-c140">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:photoshop="http://ns.adobe.com/photoshop/1.0/"
    xmlns:iptcCore="http://iptc.org/std/Iptc4xmpCore/1.0/xmlns/"
    xmlns:dc="http://purl.org/dc/elements/1.1/">
   <photoshop:City>{}</photoshop:City>
   <photoshop:State>{}</photoshop:State>
   <photoshop:Country>{}</photoshop:Country>
   <iptcCore:Location>{}</iptcCore:Location>
   <dc:subject>
    <rdf:Bag>
     {}
    </rdf:Bag>
   </dc:subject>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#,
                            final_city, final_state, final_country, final_sublocation, people_rdf
                        );

                        let write_res = if ext == "png" {
                            if let Ok(mut png) = Png::from_bytes(Bytes::from(raw_bytes)) {
                                let mut itxt_payload = Vec::new();
                                itxt_payload.extend_from_slice(PNG_XMP_KEYWORD);
                                itxt_payload.extend_from_slice(&[0, 0]);
                                itxt_payload.extend_from_slice(b"\0");
                                itxt_payload.extend_from_slice(b"\0");
                                itxt_payload.extend_from_slice(xmp_xml.as_bytes());

                                png.chunks_mut().retain(|chunk| {
                                    if chunk.kind() == *b"iTXt" {
                                        let contents = chunk.contents();
                                        !contents.starts_with(PNG_XMP_KEYWORD)
                                            && !contents
                                                .windows(11)
                                                .any(|w| w == b"<x:xmpmeta " || w == b"<rdf:RDF ")
                                    } else {
                                        true
                                    }
                                });

                                let insert_pos = png
                                    .chunks()
                                    .iter()
                                    .position(|c| c.kind() == *b"IDAT")
                                    .unwrap_or(png.chunks().len());

                                png.chunks_mut().insert(
                                    insert_pos,
                                    PngChunk::new(*b"iTXt", Bytes::from(itxt_payload)),
                                );

                                let mut output_buffer = Vec::new();
                                if png.encoder().write_to(&mut output_buffer).is_ok() {
                                    fs::write(path, output_buffer).is_ok()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            if let Ok(mut jpeg) = Jpeg::from_bytes(Bytes::from(raw_bytes)) {
                                let mut payload = Vec::from(XMP_HEADER);
                                payload.extend_from_slice(xmp_xml.as_bytes());

                                jpeg.segments_mut().retain(|seg| {
                                    let contents = seg.contents();
                                    !contents.starts_with(XMP_HEADER)
                                        && !contents
                                            .windows(11)
                                            .any(|w| w == b"<x:xmpmeta " || w == b"<rdf:RDF ")
                                });

                                let segment =
                                    JpegSegment::new_with_contents(0xE1, Bytes::from(payload));
                                let idx = if jpeg.segments().is_empty() { 0 } else { 1 };
                                jpeg.segments_mut().insert(idx, segment);

                                let mut output_buffer = Vec::new();
                                if jpeg.encoder().write_to(&mut output_buffer).is_ok() {
                                    fs::write(path, output_buffer).is_ok()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        if write_res {
                            success_count += 1;
                            continue;
                        }
                    }
                    error_count += 1;
                }

                self.status = format!(
                    "Updated: {} succeeded, {} failed.",
                    success_count, error_count
                );

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

        let select_all_btn = button(text("All").size(12).font(LEXEND_REGULAR))
            .on_press(Message::SelectAllFiles)
            .style(small_button_style)
            .padding(4);

        let deselect_all_btn = button(text("None").size(12).font(LEXEND_REGULAR))
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

        let explorer_panel = container(column![explorer_header, scrollable_files])
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .padding(10);

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

        let apply_btn_label = if self.selected_files.len() > 1 {
            "Batch Apply EXIF + XMP"
        } else {
            "Apply EXIF + XMP"
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

        let editor_column = column![
            bold_title("EXIF Dates & Timestamps"),
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
            bold_title("EXIF GPS Coordinates"),
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
            bold_title("EXIF Context"),
            make_input_row(
                "Description:",
                "Caption",
                &self.fields.caption_str,
                Message::CaptionChanged
            ),
            make_input_row(
                "Credit:",
                "Photographer",
                &self.fields.credit_str,
                Message::CreditChanged
            ),
            Space::new().height(10),
            bold_title("XMP Extended Location"),
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
                "Landmark / Venue",
                &self.fields.sublocation_str,
                Message::SublocationChanged
            ),
            Space::new().height(10),
            bold_title("XMP Tagged People"),
            make_input_row(
                "People:",
                "Comma separated names",
                &self.fields.people_str,
                Message::PeopleChanged
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
        .padding(10);

        let primary_selected = self
            .file_list
            .iter()
            .rfind(|p| self.selected_files.contains(*p));

        let preview_content: Element<Message> = match primary_selected {
            Some(path) => {
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                let file_size_str = if let Ok(meta) = fs::metadata(path) {
                    let bytes = meta.len();
                    if bytes >= 1_048_576 {
                        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
                    } else {
                        format!("{:.1} KB", bytes as f64 / 1_024.0)
                    }
                } else {
                    "Unknown Size".to_string()
                };

                let dimensions_str = if let Ok(dim) = imagesize::size(path) {
                    format!("{} × {} px", dim.width, dim.height)
                } else {
                    "Dimensions unknown".to_string()
                };

                container(
                    column![
                        txt(format!("File: {}", file_name)).size(14),
                        txt(format!(
                            "Size: {} | Dimensions: {}",
                            file_size_str, dimensions_str
                        ))
                        .size(12)
                        .color(Color::from_rgb8(0xAA, 0xAA, 0xAA)),
                        Space::new().height(10),
                        image(path.to_string_lossy().to_string())
                            .width(Length::Fill)
                            .height(Length::Fill),
                    ]
                    .align_x(iced::Alignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            None => container(txt_str("Select a photo from the left menu to preview").size(14))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center)
                .into(),
        };

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

fn parse_xmp_tag(xmp: &str, tag_name: &str) -> String {
    let raw_name = tag_name.split(':').last().unwrap_or(tag_name);

    for open_pattern in &[format!("<{}", tag_name), format!("<{}", raw_name)] {
        if let Some(start_idx) = xmp.find(open_pattern) {
            if let Some(tag_close) = xmp[start_idx..].find('>') {
                let content_start = start_idx + tag_close + 1;
                for close_pattern in &[format!("</{}>", tag_name), format!("</{}>", raw_name)] {
                    if let Some(end_offset) = xmp[content_start..].find(close_pattern) {
                        let value = xmp[content_start..content_start + end_offset].trim();
                        if !value.is_empty() && !value.contains('<') {
                            return value.to_string();
                        }
                    }
                }
            }
        }
    }

    for attr_pattern in &[format!("{}=", tag_name), format!("{}=", raw_name)] {
        if let Some(start_idx) = xmp.find(attr_pattern) {
            let rest = &xmp[start_idx + attr_pattern.len()..];
            let quote_char = rest.chars().next().unwrap_or('"');
            if quote_char == '"' || quote_char == '\'' {
                let val_start = 1;
                if let Some(val_end) = rest[val_start..].find(quote_char) {
                    return rest[val_start..val_start + val_end].trim().to_string();
                }
            }
        }
    }

    String::new()
}

fn parse_xmp_bag(xmp: &str, tag_name: &str) -> String {
    let raw_name = tag_name.split(':').last().unwrap_or(tag_name);

    let bag_start = [format!("<{}", tag_name), format!("<{}", raw_name)]
        .iter()
        .find_map(|p| xmp.find(p));

    if let Some(start) = bag_start {
        let sub_xmp = &xmp[start..];
        let mut items = Vec::new();
        let mut search_idx = 0;

        while let Some(li_start) = sub_xmp[search_idx..].find("<rdf:li") {
            let actual_start = search_idx + li_start;
            if let Some(tag_close) = sub_xmp[actual_start..].find('>') {
                let val_start = actual_start + tag_close + 1;
                if let Some(li_end) = sub_xmp[val_start..].find("</rdf:li>") {
                    let val = sub_xmp[val_start..val_start + li_end].trim();
                    if !val.is_empty() {
                        items.push(val);
                    }
                    search_idx = val_start + li_end;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if !items.is_empty() {
            return items.join(", ");
        }
    }

    String::new()
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
