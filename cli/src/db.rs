use surrealdb::{
    Result, Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};

pub async fn generate_client(connection_url: String) -> Result<Surreal<Client>> {
    let db = Surreal::new::<Ws>(connection_url).with_capacity(10).await?;
    db.use_ns("dauly-bugle").use_db("daily-bugle").await?;
    db.signin(Root {
        username: "root",
        password: "secret",
    })
    .await?;
    Ok(db)
}
