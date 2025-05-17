#![allow(dead_code)]

use chrono::prelude::*;
use std::{borrow::Cow, error::Error};
use zerocopy::{TryCastError, TryFromBytes, big_endian::U32};
use zerocopy_derive::*;

#[repr(C)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable, Unaligned)]
pub struct Header<'a> {
    pub start: &'a HeaderStart,
    pub fields: Vec<&'a FieldDefinition>,
}

impl<'a> Header<'a> {
    pub fn try_ref_from_prefix(
        bytes: &'a [u8],
    ) -> Result<(Header<'a>, &'a [u8]), Box<dyn Error + 'a>> {
        let mut bytes = bytes;
        let (start, rest_bytes) = HeaderStart::try_ref_from_prefix(bytes)?;
        bytes = rest_bytes;
        let (fields, rest_bytes) = start.fields_from_prefix(bytes)?;
        bytes = rest_bytes;
        let header = Header { start, fields };
        Ok((header, bytes))
    }
}

#[repr(C)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable, Unaligned)]
pub struct HeaderStart {
    file_name: [u8; 32],
    pub earliest_date_epoch: Timestamp,
    pub latest_date_epoch: Timestamp,
    pub log_type: LogType,
    pub num_fields: u8,
}

impl HeaderStart {
    pub fn file_name(&self) -> Option<String> {
        null_terminated_string(&self.file_name)
    }

    pub fn file_name_lossy(&self) -> Cow<str> {
        null_terminated_string_lossy(&self.file_name)
    }

    #[allow(clippy::type_complexity)]
    pub fn fields_from_prefix<'a>(
        &self,
        bytes: &'a [u8],
    ) -> Result<(Vec<&'a FieldDefinition>, &'a [u8]), TryCastError<&'a [u8], FieldDefinition>> {
        let mut bytes = bytes;
        let mut fields = Vec::new();
        for _ in 0..self.num_fields {
            let (field, rest_bytes) = FieldDefinition::try_ref_from_prefix(bytes)?;
            fields.push(field);
            bytes = rest_bytes;
        }
        Ok((fields, bytes))
    }
}

#[repr(C)]
#[derive(Debug, TryFromBytes, KnownLayout, Immutable, Unaligned)]
pub struct FieldDefinition {
    name: [u8; 32],
    pub data_info: DataInfo,
    pub index: u8,
}

impl FieldDefinition {
    /// The name of the field.
    pub fn name(&self) -> Option<String> {
        null_terminated_string(&self.name)
    }

    /// The name of the field (lossily converted).
    pub fn name_lossy(&self) -> Cow<str> {
        null_terminated_string_lossy(&self.name)
    }
}

#[repr(u8)]
#[derive(
    Debug,
    TryFromBytes,
    KnownLayout,
    Immutable,
    Unaligned,
    Hash,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
pub enum LogType {
    System = 0,
    Comm = 1,
    Debug = 2,
}

#[repr(C)]
#[derive(
    Debug,
    TryFromBytes,
    KnownLayout,
    Immutable,
    Unaligned,
    Hash,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
pub struct Timestamp(U32);

impl Timestamp {
    pub fn get_utc(self) -> Option<DateTime<Utc>> {
        let num_seconds = self.0.get();
        if num_seconds == 0 {
            return None;
        }
        DateTime::<Utc>::from_timestamp(num_seconds.into(), 0)
    }

    pub fn get_local(self) -> Option<DateTime<Local>> {
        self.get_utc().map(|t| t.with_timezone(&Local))
    }
}

#[repr(C)]
#[derive(
    Debug,
    TryFromBytes,
    KnownLayout,
    Immutable,
    Unaligned,
    Hash,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
)]
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
