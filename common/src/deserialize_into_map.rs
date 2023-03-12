use std::{collections::BTreeMap, fs::File, path::PathBuf};

use serde::de;

use crate::Id;

pub async fn deserialize_into_map<T>(
    csv_path: &PathBuf,
) -> Result<BTreeMap<T::IdType, T>, Box<dyn std::error::Error>>
where
    T: Id + de::DeserializeOwned,
{
    let file = File::open(csv_path)?;

    let mut parsed_records = BTreeMap::new();

    for result in csv::Reader::from_reader(file).deserialize::<T>() {
        let record = result?;
        let id = record.id();

        if parsed_records.insert(id.clone(), record).is_some() {
            return Err(format!("duplicate id: {id:#?}").into());
        }
    }

    Ok(parsed_records)
}
