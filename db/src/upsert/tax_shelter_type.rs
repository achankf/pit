use common::Id;
use serde::Deserialize;
use serde_trim::string_trim;

use crate::Query;

#[derive(Debug, Deserialize)]
pub struct TaxShelterType {
    #[serde(deserialize_with = "string_trim")]
    tax_shelter_type: String,
    #[serde(deserialize_with = "string_trim")]
    tax_shelter_name: String,
}

impl Id for TaxShelterType {
    type IdType = String;

    fn id(&self) -> Self::IdType {
        self.tax_shelter_type.clone()
    }
}

impl Query for TaxShelterType {
    fn query(&self) -> crate::SqlQuery {
        sqlx::query!(
            r#"
INSERT
    OR IGNORE INTO TaxShelterType (tax_shelter_type, tax_shelter_name)
VALUES
    (?, ?) ON CONFLICT(tax_shelter_type) DO
UPDATE
SET
    tax_shelter_name = excluded.tax_shelter_name
WHERE
    tax_shelter_name <> excluded.tax_shelter_name
"#,
            self.tax_shelter_type,
            self.tax_shelter_name
        )
    }
}
