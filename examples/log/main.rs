use zerocopy::TryFromBytes;

mod log_byteview;
mod log_zerocopy;

pub fn main() {
    println!("=== Try with good bytes ===");
    let contents = include_bytes!("sample.log");

    println!("= byteview =");
    let (header, _rest_bytes) = log_byteview::Header::split_slice(contents).unwrap();
    let log_byteview::Header { start, fields } = header;
    println!("File name: \"{:?}\"", start.file_name());
    println!("Earliest Date: {:?}", start.earliest_date_utc());
    println!("Latest Date: {:?}", start.latest_date_utc());
    println!("Log Type: {:?}", start.log_type());
    println!("Num Fields: {}", start.num_fields());
    for (index, field) in fields.iter().enumerate() {
        println!("\tField at index {index}:");
        println!("\t\tName: {}", field.name_lossy());
        println!("\t\tKind: {:?}", field.data_info().kind());
        println!("\t\tLength: {}", field.data_info().length());
        println!("\t\tIndex: {}", field.index());
    }
    println!();

    println!("= zerocopy =");
    let (header, _rest_bytes) = log_zerocopy::Header::try_ref_from_prefix(contents).unwrap();
    let log_zerocopy::Header { start, fields } = header;
    println!("File name: \"{}\"", start.file_name_lossy());
    println!(
        "Earliest Date: {:?} ({:?} UTC)",
        start.earliest_date_epoch.get_local(),
        start.earliest_date_epoch.get_utc()
    );
    println!(
        "Latest Date: {:?} ({:?} UTC)",
        start.latest_date_epoch.get_local(),
        start.latest_date_epoch.get_utc()
    );
    println!("Log Type: {:?}", start.log_type);
    println!("Num Fields: {}", start.num_fields);
    for (index, field) in fields.iter().enumerate() {
        println!("\tField at index {index}:");
        println!("\t\tName: {}", field.name_lossy());
        println!("\t\tKind: {:?}", field.data_info.kind());
        println!("\t\tLength: {}", field.data_info.length());
        println!("\t\tIndex: {}", field.index);
    }
    println!();

    println!("=== Try with bad log_type value ===");
    let mut contents = contents.to_vec();
    contents[40] = 7; // Index of log_type = 40, 7 is NOT a valid LogType value

    println!("= byteview =");
    let (header, _rest_bytes) = log_byteview::Header::split_slice(&contents).unwrap();
    let log_byteview::Header { start, fields } = header;
    println!("File name: \"{:?}\"", start.file_name());
    println!(
        "Earliest Date: {:?} ({:?} UTC)",
        start.earliest_date_local(),
        start.earliest_date_utc()
    );
    println!(
        "Latest Date: {:?} ({:?} UTC)",
        start.latest_date_local(),
        start.latest_date_utc()
    );
    println!("Log Type: {:?}", start.log_type());
    println!("Num Fields: {}", start.num_fields());
    for (index, field) in fields.iter().enumerate() {
        println!("\tField at index {index}:");
        println!("\t\tName: {}", field.name_lossy());
        println!("\t\tKind: {:?}", field.data_info().kind());
        println!("\t\tLength: {}", field.data_info().length());
        println!("\t\tIndex: {}", field.index());
    }
    println!();

    println!("= zerocopy =");
    let parse_result = log_zerocopy::HeaderStart::try_ref_from_prefix(&contents);
    println!("{parse_result:?}");
    // println!("\tField at index {index}:");
    // println!("\t\tName: {}", field.name_lossy());
    // println!("\t\tKind: {:?}", field.data_info().kind());
    // println!("\t\tLength: {}", field.data_info().length());
    // println!("\t\tIndex: {}", field.index());
    println!();
}
