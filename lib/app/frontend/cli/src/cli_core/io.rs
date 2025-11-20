use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{self, Value};

use super::{CliError, CliResult};

pub fn input_reader(path: Option<&PathBuf>) -> CliResult<Box<dyn BufRead>> {
    if let Some(path) = path {
        let file = File::open(path)
            .map_err(|err| CliError::io(format!("failed to open {}: {err}", path.display())))?;
        Ok(Box::new(BufReader::new(file)))
    } else {
        Ok(Box::new(BufReader::new(io::stdin())))
    }
}

pub fn read_to_string(path: &Path) -> CliResult<String> {
    let mut file = File::open(path)
        .map_err(|err| CliError::io(format!("failed to open {}: {err}", path.display())))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|err| CliError::io(format!("failed to read {}: {err}", path.display())))?;
    Ok(contents)
}

pub fn parse_json_file<T: DeserializeOwned>(path: &Path) -> CliResult<T> {
    let file = File::open(path)
        .map_err(|err| CliError::io(format!("failed to open {}: {err}", path.display())))?;
    serde_json::from_reader(file).map_err(CliError::from)
}

pub fn write_record<W: Write, T: Serialize>(
    writer: &mut W,
    kind: &'static str,
    value: &T,
) -> CliResult<()> {
    let record = serde_json::json!({
        "kind": kind,
        "value": value
    });
    serde_json::to_writer(&mut *writer, &record)?;
    writer.write_all(b"\n").map_err(CliError::from)
}

pub fn json_stream<T: DeserializeOwned>(
    reader: Box<dyn BufRead>,
) -> JsonStream<Box<dyn BufRead>, T> {
    JsonStream::new(reader)
}

pub struct JsonStream<R, T>
where
    R: BufRead,
    T: DeserializeOwned,
{
    stream: serde_json::StreamDeserializer<'static, serde_json::de::IoRead<R>, serde_json::Value>,
    pending: VecDeque<T>,
    _marker: PhantomData<T>,
}

impl<R, T> JsonStream<R, T>
where
    R: BufRead,
    T: DeserializeOwned,
{
    pub fn new(reader: R) -> Self {
        let deserializer = serde_json::Deserializer::from_reader(reader);
        let stream = deserializer.into_iter::<Value>();
        Self {
            stream,
            pending: VecDeque::new(),
            _marker: PhantomData,
        }
    }

    fn push_value(&mut self, value: Value) -> CliResult<()> {
        match value {
            Value::Array(values) => {
                for value in values {
                    self.push_value(value)?;
                }
                Ok(())
            }
            Value::Null => Ok(()),
            other => {
                let parsed = serde_json::from_value(other)?;
                self.pending.push_back(parsed);
                Ok(())
            }
        }
    }
}

impl<R, T> Iterator for JsonStream<R, T>
where
    R: BufRead,
    T: DeserializeOwned,
{
    type Item = CliResult<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.pending.pop_front() {
            return Some(Ok(value));
        }
        match self.stream.next()? {
            Ok(value) => {
                if let Err(err) = self.push_value(value) {
                    return Some(Err(err));
                }
                self.pending.pop_front().map(Ok)
            }
            Err(err) => Some(Err(err.into())),
        }
    }
}
