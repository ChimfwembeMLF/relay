pub mod invoices;
pub mod queries;
pub mod reports;
pub mod wallet_seeds;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
