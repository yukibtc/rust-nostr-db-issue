use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let database = SQLiteDatabase::open("nostr.db").await?;
    //let database = NdbDatabase::open("nostr")?;

    let client: Client = ClientBuilder::default()
        .database(database)
        .build();

    client.add_relay("wss://relay.damus.io").await?;

    client.connect().await;

    let yuki_public_key: PublicKey = PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;

    // Negentropy reconcile
    let filter = Filter::new().author(yuki_public_key);
    client
        .reconcile(filter, NegentropyOptions::default())
        .await?;

    // Query events from database
    let filter = Filter::new().author(yuki_public_key).kind(Kind::TextNote).limit(5);
    let events = client.database().query(vec![filter], Order::Desc).await?;

    for event in events.into_iter() {
        println!("{}", event.as_json());
    }

    Ok(())
}