use anyhow::{Result, bail};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::America::New_York;
use quick_xml::{Reader, events::Event};

pub fn naive_date_to_utc(date: NaiveDate) -> DateTime<Utc> {
    let naive_dt = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    let dt = New_York.from_local_datetime(&naive_dt).unwrap();
    dt.to_utc()
}

pub trait XMLHandler<T> {
    /// Called for `<tag ...>`
    fn start(&mut self, name: &[u8]) -> Result<()>;
    /// Called for text between start & end (already decoded to UTF-8)
    fn text(&mut self, txt: &str) -> Result<()>;
    /// Called for `</tag>`
    fn end(&mut self, name: &[u8]) -> Result<()>;
    fn items(self) -> T;
}

/// Generic parse loop (reusable for any XML)
pub fn parse_xml_with<H, T>(reader: Reader<&[u8]>, handler: H) -> Result<T>
where
    H: XMLHandler<T>,
{
    let mut buf = Vec::new();
    let mut reader = reader;
    let mut handler = handler;
    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => bail!("Error at position {}: {:?}", reader.error_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => handler.start(e.name().as_ref())?,
            Ok(Event::Text(e)) => handler.text(&e.decode()?.into_owned())?,
            Ok(Event::End(e)) => handler.end(e.name().as_ref())?,
            _ => {}
        }
        buf.clear();
    }
    Ok(handler.items())
}
