use db::Db;

pub async fn refresh_market_price() -> Result<(), Box<dyn std::error::Error>> {
    let mut db = Db::new().await?;
    let mut transaction = db.begin_wrapped_transaction().await?;

    transaction.refresh_market_price().await?;

    transaction.commit().await?;
    db.optimize().await?;

    Ok(())
}
