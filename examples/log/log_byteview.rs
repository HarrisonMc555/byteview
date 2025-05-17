#![allow(dead_code)]

use chrono::{DateTime, Local, Utc};
use std::borrow::Cow;

pub struct Header<'a> {
    pub start: HeaderStart<'a>,
    pub fields: Vec<FieldDefinition<'a>>,
}

impl Header<'_> {
    pub fn split_slice(bytes: &[u8]) -> Option<(Header<'_>, &[u8])> {
        let (start, bytes) = HeaderStart::split_slice(bytes).unwrap();
        let mut bytes = bytes;
        let mut fields = Vec::new();
        for _ in 0..start.num_fields() {
            let (field_def, rest_bytes) = FieldDefinition::split_slice(bytes).unwrap();
            fields.push(field_def);
            bytes = rest_bytes;
        }
        let header = Header { start, fields };
        Some((header, bytes))
    }
}

byteview::byteview_ref! {
    pub struct HeaderStart {
        _file_name: [u8; 32],
        _earliest_date_epoch: u32be,
        _latest_date_epoch: u32be,
        _log_type: u8,
        pub num_fields: u8,
    }
}

impl HeaderStart<'_> {
    /// The name of the file this header is associated with.
    pub fn file_name(&self) -> Option<String> {
        null_terminated_string(self._file_name())
    }

    /// The name of the file this header is associated with (lossily converted).
    pub fn file_name_lossy(&self) -> Cow<str> {
        null_terminated_string_lossy(self._file_name())
    }

    /// The earliest date in Utc.
    pub fn earliest_date_utc(&self) -> Option<DateTime<Utc>> {
        from_unix_epoch(self._earliest_date_epoch())
    }

    /// The earliest date in the Local time zone.
    pub fn earliest_date_local(&self) -> Option<DateTime<Local>> {
        self.earliest_date_utc().map(|dt| dt.with_timezone(&Local))
    }

    /// The latest date in Utc.
    pub fn latest_date_utc(&self) -> Option<DateTime<Utc>> {
        from_unix_epoch(self._latest_date_epoch())
    }

    /// The latest date in the Local time zone.
    pub fn latest_date_local(&self) -> Option<DateTime<Local>> {
        self.latest_date_utc().map(|dt| dt.with_timezone(&Local))
    }

    /// The [`LogType`] for this header.
    pub fn log_type(&self) -> Result<LogType, u8> {
        Ok(match self._log_type() {
            0 => LogType::System,
            1 => LogType::Comm,
            2 => LogType::Debug,
            byte => return Err(byte),
        })
    }
}

byteview::byteview_ref! {
    /// A definition for a single field.
    pub struct FieldDefinition {
        _name: [u8; 32],
        _data_info: u8,
        /// The index of the field.
        pub index: u8,
    }
}

impl FieldDefinition<'_> {
    /// The name of the field.
    pub fn name(&self) -> Option<String> {
        null_terminated_string(self._name())
    }

    /// The name of the field (lossily converted).
    pub fn name_lossy(&self) -> Cow<str> {
        null_terminated_string_lossy(self._name())
    }

    /// The [`DataInfo`] for the field.
    pub fn data_info(&self) -> DataInfo {
        DataInfo(self._data_info())
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub enum LogType {
    System,
    Comm,
    Debug,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub struct DataInfo(u8);

impl DataInfo {
    /// The kind of data this field contains.
    pub fn kind(self) -> Result<DataKind, u8> {
        let kind_num = self.0 >> 4;
        Ok(match kind_num {
            0 => DataKind::SignedInteger,
            1 => DataKind::UnsignedInteger,
            2 => DataKind::Float,
            3 => DataKind::String,
            4 => DataKind::Bool,
            _ => return Err(kind_num),
        })
    }

    /// The length of the data.
    pub fn length(self) -> u8 {
        self.0 & 0x0F
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
pub enum DataKind {
    SignedInteger,
    UnsignedInteger,
    Float,
    String,
    Bool,
}

fn null_terminated_bytes(bytes: &[u8]) -> &[u8] {
    match bytes.iter().position(|&b| b == 0) {
        Some(index) => &bytes[..index],
        None => bytes,
    }
}

fn null_terminated_string(bytes: &[u8]) -> Option<String> {
    String::from_utf8(null_terminated_bytes(bytes).to_vec()).ok()
}

fn null_terminated_string_lossy(bytes: &[u8]) -> Cow<str> {
    String::from_utf8_lossy(null_terminated_bytes(bytes))
}

fn from_unix_epoch(num_seconds: u32) -> Option<DateTime<Utc>> {
    if num_seconds == 0 {
        return None;
    }
    DateTime::<Utc>::from_timestamp(num_seconds.into(), 0)
}

// pub fn main() {
//     // Read all fields
//     let contents = std::fs::read("sample.log").unwrap();
//     println!("contents: {contents:?}");
//     let (header_start, rest_bytes) = HeaderStart::split_slice(&contents).unwrap();
//     let file_name_res = String::from_utf8(header_start.file_name().to_owned());
//     println!("name: {:?}", file_name_res);
//     const FORMAT: &str = "%Y-%m-%d %H:%M:%S";
//     print!("earliest: ");
//     match header_start.earliest_date_local() {
//         Some(earliest) => println!("{}", earliest.format(FORMAT)),
//         None => println!("N/A"),
//     };
//     print!("latest: ");
//     match header_start.latest_date_local() {
//         Some(latest) => println!("{}", latest.format(FORMAT)),
//         None => println!("N/A"),
//     };
//     println!("log type: {:?}", header_start.log_type());
//     println!("num fields: {}", header_start.num_fields());
//     println!();

//     let mut cur_bytes = rest_bytes;
//     let mut field_defs = Vec::new();
//     for _ in 0..header_start.num_fields() {
//         let (field_def, rest_bytes) = FieldDefinition::split_slice(cur_bytes).unwrap();
//         field_defs.push(field_def);
//         cur_bytes = rest_bytes;
//     }

//     for (index, field_def) in field_defs.iter().enumerate() {
//         println!("Field {index}:");
//         println!(
//             "\tName: {}",
//             String::from_utf8(field_def.name().to_owned()).unwrap()
//         );
//         println!("\tKind: {:?}", field_def.data_info().kind());
//         println!("\tLength: {}", field_def.data_info().length());
//         println!("\tIndex: {}", field_def.index());
//         println!();
//     }

//     // Read only the last field
//     use std::io::prelude::*;
//     let mut file = std::fs::File::open("sample.log").unwrap();
//     let mut header_bytes = [0; HeaderStart::NUM_BYTES];
//     file.read_exact(&mut header_bytes).unwrap();
//     let header = HeaderStart::from_array(&header_bytes);
//     let num_fields = header.num_fields();
//     let last_field_index = num_fields - 1;
//     let offset = last_field_index as i64 * FieldDefinition::NUM_BYTES as i64;
//     file.seek_relative(offset).unwrap();
//     let mut field_def_bytes = [0; FieldDefinition::NUM_BYTES];
//     file.read_exact(&mut field_def_bytes).unwrap();
//     let last_field_def = FieldDefinition::from_array(&field_def_bytes);
//     println!(
//         "\tName: {}",
//         String::from_utf8(last_field_def.name().to_owned()).unwrap()
//     );
//     println!("\tKind: {:?}", last_field_def.data_info().kind());
//     println!("\tLength: {}", last_field_def.data_info().length());
//     println!("\tIndex: {}", last_field_def.index());
//     println!();

//     // Read all fields with ref macro
//     let contents = std::fs::read("sample.log").unwrap();
//     println!("contents: {contents:?}");
//     let (header_start, rest_bytes) = HeaderStart::split_slice(&contents).unwrap();
//     let file_name_res = String::from_utf8(header_start.file_name().to_owned());
//     println!("name: {:?}", file_name_res);
//     print!("earliest: ");
//     match header_start.earliest_date_local() {
//         Some(earliest) => println!("{}", earliest.format(FORMAT)),
//         None => println!("N/A"),
//     };
//     print!("latest: ");
//     match header_start.latest_date_local() {
//         Some(latest) => println!("{}", latest.format(FORMAT)),
//         None => println!("N/A"),
//     };
//     println!("log type: {:?}", header_start.log_type());
//     println!("num fields: {}", header_start.num_fields());
//     println!();

//     let mut cur_bytes = rest_bytes;
//     let mut field_defs = Vec::new();
//     for _ in 0..header_start.num_fields() {
//         let (field_def, rest_bytes) = FieldDefinition::split_slice(cur_bytes).unwrap();
//         field_defs.push(field_def);
//         cur_bytes = rest_bytes;
//     }

//     for (index, field_def) in field_defs.iter().enumerate() {
//         println!("Field {index}:");
//         println!(
//             "\tName: {}",
//             String::from_utf8(field_def.name().to_owned()).unwrap()
//         );
//         println!("\tKind: {:?}", field_def.data_info().kind());
//         println!("\tLength: {}", field_def.data_info().length());
//         println!("\tIndex: {}", field_def.index());
//         println!();
//     }
// }
