use std::collections::HashSet;
use nostr_sdk::prelude::*;

const RELAY_URL: &str = "wss://relay.damus.io";
const PUBLIC_KEY: &str = "npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet"; // Yuki public key

#[tokio::main]
async fn main() -> Result<()> {
    // Sync and query events
    let lmdb_events = lmdb().await?;
    let ndb_events = ndb().await?;

    // Collect IDs
    let lmdb_output_ids: Vec<String> = lmdb_events.into_iter().map(|e| e.id.to_hex()).collect();
    let ndb_output_ids: Vec<String> = ndb_events.into_iter().map(|e| e.id.to_hex()).collect();

    if lmdb_output_ids.len() != ndb_output_ids.len() {
        println!("Query output len NOT match:");
        println!("- LMDB: {}", lmdb_output_ids.len());
        println!("- NDB: {}", ndb_output_ids.len());

        // Convert to hashset
        let set1: HashSet<String> = lmdb_output_ids.iter().cloned().collect();
        let set2: HashSet<String> = ndb_output_ids.iter().cloned().collect();

        let difference: HashSet<_> = set1.symmetric_difference(&set2).collect();

        println!("Difference:");
        for id in difference {
            println!("- {}", id);
        }
    } else {
        println!("Query output len match");
    }

    // Expected IDs
    // abc5a7ed07d60de9e5eb21da34c8369af21e04455c9d3930bd122b2e049d9791
    // f2d71a515ce3576d238aaaeaa48fde97388162d08208f729b540a4c3f9723e6b
    // 670303f9cbb24568c705b545c277be1f5172ad84795cc9e700aeea5bb248fd74

    // Output nostrdb
    // fae111325f4c4030c214fa2d2eb6020acf3961d142c922d270a8eb45eabd81e4
    // fde4199cf10077baa9f08d146f20d5a7a1e62696749f9f2d69459e134c0480e3
    // fc3eb4ad9359733dc55b5a3c5835c29eb5f47fb11ab7f536e8040191f46f6811

    assert_eq!(lmdb_output_ids, ndb_output_ids);

    Ok(())
}

async fn lmdb() -> Result<Events> {
    let database = NostrLMDB::open("nostr-lmdb")?;
    run(database).await
}

async fn ndb() -> Result<Events> {
    let database = NdbDatabase::open("nostr-ndb")?;
    run(database).await
}

async fn run<T>(database: T) -> Result<Events>
where
    T: IntoNostrDatabase,
{
    // Construct client
    let client: Client = Client::builder()
        .database(database)
        .build();

    // Add relay and connect
    client.add_relay(RELAY_URL).await?;
    client.connect().await;

    // Parse
    let public_key: PublicKey = PublicKey::parse(PUBLIC_KEY)?;

    // Negentropy reconcile
    let filter = Filter::new().author(public_key).kind(Kind::TextNote).until(Timestamp::from_secs(1732980781));
    client
        .sync(filter, &SyncOptions::default())
        .await?;

    // Query events from database
    let filter = Filter::new().author(public_key).kind(Kind::TextNote).limit(3);
    println!("Filter: {}", filter.as_json());
    let events = client.database().query(vec![filter]).await?;

    Ok(events)
}
