# batch-exif
Batch Exif Offline Editor in Rust/Iced

### Genealogical EXIF & Companion Metadata Reference

| Variable | Description |
| :--- | :--- |
| **DateTimeOriginal** | The date and time when the original photograph was taken. |
| **OffsetTimeOriginal** | The UTC timezone offset corresponding to the `DateTimeOriginal` tag. |
| **DateTimeDigitized** | The date and time when the physical photo print or negative was digitally scanned. |
| **GPSLatitude** | The numerical latitude coordinate where the image was captured. |
| **GPSLatitudeRef** | Indicates whether the latitude is North (`N`) or South (`S`) of the equator. |
| **GPSLongitude** | The numerical longitude coordinate where the image was captured. |
| **GPSLongitudeRef** | Indicates whether the longitude is East (`E`) or West (`W`) of the Prime Meridian. |
| **GPSAltitude** | The elevation above or below sea level where the photo was taken. |
| **IPTC:City** | The name of the city associated with the photo location. |
| **IPTC:Province-State** | The name of the state or province where the photo was taken. |
| **IPTC:CountryName** | The full name of the country where the photo was taken. |
| **IPTC:SubLocation / LocationName** | Specific place names or addresses (e.g., cemetery or house address). |
| **IPTC:Caption-Abstract / XMP:Description** | Transcribed notes, back-of-photo inscriptions, or contextual descriptions. |
| **XMP-mwg-rs:Regions** | Bounding box coordinates used to identify and map specific facial regions. |
| **IPTC:PersonInImage** | List of individual names corresponding to the people appearing in the photo. |
| **IPTC:Credit / IPTC:Byline** | The photographer, original collector, or contributor who provided the photograph. |

### Arch Windows Cross Compilation
```bash
sudo pacman -S mingw-w64-gcc
rustup target add x86_64-pc-windows-gnu
```
```bash
cargo build --release --target x86_64-pc-windows-gnu
ls target/x86_64-pc-windows-gnu/release/batch-exif.exe
```
