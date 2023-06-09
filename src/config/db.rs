use sqlx::Postgres;

// connect to database
pub async fn init_db() -> Result<sqlx::Pool<Postgres>, Box<dyn std::error::Error>> {
    let db = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::pool::PoolOptions::default()
        .max_connections(100)
        .connect(&db)
        .await?;

    sqlx::query_file!("src/config/sql/create.table.sql").execute(&pool).await?;
    sqlx::query_file!("src/config/sql/get.user.sql").execute(&pool).await?;
    sqlx::query_file!("src/config/sql/confirm.user.sql").execute(&pool).await?;
    sqlx::query_file!("src/config/sql/delete.user.sql").execute(&pool).await?;
    sqlx::query_file!("src/config/sql/insert.user.sql").execute(&pool).await?;

    Ok(pool)
}
