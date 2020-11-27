use oxidizer::*;

pub struct TestCustomPrimaryKey {
    name: String,

    email: String,
}

#[oxidizer::async_trait]
impl oxidizer::entity::IEntity for TestCustomPrimaryKey {
    async fn save(&mut self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<bool> {
        let mut creating = false;
        let primary_key_default: String = Default::default();
        if self.name == primary_key_default {
            creating = true;
        }
        let query = concat!(
            "INSERT INTO \"",
            "test_custom_primary_key",
            "\"",
            " (",
            stringify!(name, email),
            ") values (",
            "$1,$2",
            ") ON CONFLICT (",
            stringify!(name),
            ") DO ",
            "UPDATE SET ",
            concat!(stringify ! (email =), "$2"),
            " RETURNING ",
            stringify!(name),
            ";"
        );
        let rows = db.query(query, args![self.name, self.email]).await?;
        if let Some(first_row) = rows.first() {
            self.name = first_row.get(stringify!(name)).extract()?;
        } else if creating {
            return Err(oxidizer::db::Error::Other(
                "Error while saving entity".to_string(),
            ));
        }
        Ok(creating)
    }
    async fn delete(&mut self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<bool> {
        let key_default: String = Default::default();
        if self.name == key_default {
            return Ok(false);
        }
        let condition = format!("{} = $1", stringify!(name));
        let query = format!(
            "DELETE FROM \"{}\" WHERE {}",
            "test_custom_primary_key", condition
        );
        match db.execute(&query, args![self.name]).await? {
            0 => Ok(false),
            _ => {
                self.name = key_default;
                Ok(true)
            }
        }
    }
    fn is_synced_with_db(&self) -> bool {
        let key_default: String = Default::default();
        self.name != key_default
    }
    async fn find(
        db: &oxidizer::db::DB,
        condition: &str,
        params: &'_ [oxidizer::Value<'_>],
    ) -> oxidizer::db::DBResult<Vec<TestCustomPrimaryKey>> {
        let query = format!(
            "SELECT * FROM \"{}\" WHERE {}",
            "test_custom_primary_key", condition
        );
        let rows = db.query(&query, params).await?;
        let mut results: Vec<TestCustomPrimaryKey> = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            results.push(Self::from_row(row)?);
        }
        Ok(results)
    }
    async fn first(
        db: &oxidizer::db::DB,
        condition: &str,
        params: &'_ [oxidizer::Value<'_>],
    ) -> oxidizer::db::DBResult<std::option::Option<TestCustomPrimaryKey>> {
        let query = format!(
            "SELECT * FROM \"{}\" WHERE {} LIMIT 1",
            "test_custom_primary_key", condition
        );
        let rows = db.query(&query, params).await?;
        let mut results: Vec<TestCustomPrimaryKey> = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            results.push(Self::from_row(row)?);
        }
        match results.len() {
            0 => Ok(None),
            _ => Ok(Some(results.remove(0))),
        }
    }
    fn from_row(row: oxidizer::ResultRow) -> oxidizer::db::DBResult<Self> {
        let mut obj: Self = Self {
            name: (row.get(concat!(stringify!(name))).extract()?),
            email: (row.get(concat!(stringify!(email))).extract()?),
        };
        Ok(obj)
    }
    fn get_table_name() -> String {
        "test_custom_primary_key".to_string()
    }
}
