byteview::byteview_ref! {
    /// The header for an entry (reference version).
    #[derive(Debug)]
    pub struct EntryHeaderRef {
        /// The index of the entry.
        pub index: u32be,
        _kind: u8,
        _: u8,
        _name: [u8; 16],
    }
}

impl<'a> EntryHeaderRef<'a> {
    /// What [`Kind`] of entry this is.
    pub fn kind(&self) -> Option<Kind> {
        Some(match self._kind() {
            0 => Kind::Foo,
            1 => Kind::Bar,
            2 => Kind::Baz,
            _ => return None,
        })
    }

    /// The name of the entry.
    pub fn name(&self) -> &[u8] {
        let name = self._name();
        match name.into_iter().position(|b| *b == 0) {
            Some(i) => &name[..i],
            None => name,
        }
    }
}

byteview::byteview_owned! {
    /// The header for an entry (owned version).
    #[derive(Debug)]
    pub struct EntryHeaderOwned {
        /// The index of the entry.
        pub index: u32be,
        _kind: u8,
        _: u8,
        /// The number of items in the entry.
        _name: [u8; 16],
    }
}

impl EntryHeaderOwned {
    /// What [`Kind`] of entry this is.
    pub fn kind(&self) -> Option<Kind> {
        Some(match self._kind() {
            0 => Kind::Foo,
            1 => Kind::Bar,
            2 => Kind::Baz,
            _ => return None,
        })
    }

    /// The name of the entry.
    pub fn name(&self) -> &[u8] {
        let name = self._name();
        match name.into_iter().position(|b| *b == 0) {
            Some(i) => &name[..i],
            None => name,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Kind {
    Foo,
    Bar,
    Baz,
}

impl Kind {
    pub fn from_byte(byte: u8) -> Option<Kind> {
        Some(match byte {
            0 => Kind::Foo,
            1 => Kind::Bar,
            2 => Kind::Baz,
            _ => return None,
        })
    }
}

#[test]
fn test_entry() {
    let bytes = b"\x00\x00\x07\x01\x02\x2AMy Field Name\x00\x00\x00";

    let entry_header = EntryHeaderRef::from_array(bytes);
    print!("{entry_header:#?}");
    assert_eq!(1793, entry_header.index());
    assert_eq!(Some(Kind::Baz), entry_header.kind());
    assert_eq!(b"My Field Name", entry_header.name());

    let entry_header = EntryHeaderOwned::from_array(bytes.to_owned());
    print!("{entry_header:#?}");
    assert_eq!(1793, entry_header.index());
    assert_eq!(Some(Kind::Baz), entry_header.kind());
    assert_eq!(b"My Field Name", entry_header.name());
}
