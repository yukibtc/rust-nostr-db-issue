fn main() {
    panic!("Run `cargo test`");
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::LazyLock;

    use nostr_sdk::prelude::*;
    use tokio::sync::OnceCell;

    const RELAY_URL: &str = "wss://relay.damus.io";
    static PUBLIC_KEY: LazyLock<PublicKey> = LazyLock::new(|| {
        PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet").unwrap()
    });
    static SYNC_FILTER: LazyLock<Filter> = LazyLock::new(|| {
        Filter::new()
            .author(*PUBLIC_KEY)
            .kind(Kind::TextNote)
            .until(Timestamp::from_secs(1732980781))
    });

    static LMDB_CLIENT: OnceCell<Client> = OnceCell::const_new();
    static NDB_CLIENT: OnceCell<Client> = OnceCell::const_new();

    async fn setup<T>(database: T) -> Result<Client>
    where
        T: IntoNostrDatabase,
    {
        // Construct client
        let client: Client = Client::builder().database(database).build();

        // Add relay and connect
        client.add_relay(RELAY_URL).await?;
        client.connect().await;

        // Negentropy reconcile
        client
            .sync(SYNC_FILTER.clone(), &SyncOptions::default())
            .await?;

        Ok(client)
    }

    async fn lmdb() -> &'static Client {
        LMDB_CLIENT
            .get_or_init(|| async {
                let database = NostrLMDB::open("nostr-lmdb").unwrap();
                setup(database).await.unwrap()
            })
            .await
    }

    async fn ndb() -> &'static Client {
        NDB_CLIENT
            .get_or_init(|| async {
                let database = NdbDatabase::open("nostr-ndb").unwrap();
                setup(database).await.unwrap()
            })
            .await
    }

    fn assert_events(lmdb: Events, ndb: Events) {
        // Collect IDs
        let lmdb_ids: Vec<String> = lmdb.into_iter().map(|e| e.id.to_hex()).collect();
        let ndb_ids: Vec<String> = ndb.into_iter().map(|e| e.id.to_hex()).collect();

        // Check len
        if lmdb_ids.len() != ndb_ids.len() {
            println!("Difference number of events:");
            println!("LMDB: {}", lmdb_ids.len());
            println!("NDB: {}", ndb_ids.len());

            // Convert to hashset
            let set1: HashSet<String> = lmdb_ids.into_iter().collect();
            let set2: HashSet<String> = ndb_ids.into_iter().collect();

            // hashset difference
            let difference: HashSet<_> = set1.symmetric_difference(&set2).collect();

            println!("Difference IDs:");
            for id in difference {
                println!("- {}", id);
            }

            panic!("Query output len NOT match");
        }

        assert_eq!(lmdb_ids, ndb_ids, "rust-nostr LMDB implementation and nostrdb return different in outputs");
    }

    async fn run_test(query_filter: Filter) {
        let lmdb_client = lmdb().await;
        let ndb_client = ndb().await;

        let lmdb_events = lmdb_client
            .database()
            .query(vec![query_filter.clone()])
            .await
            .unwrap();
        let ndb_events = ndb_client
            .database()
            .query(vec![query_filter])
            .await
            .unwrap();

        assert_events(lmdb_events, ndb_events);
    }

    #[tokio::test]
    async fn query_by_author_and_kind() {
        let query_filter = Filter::new().author(*PUBLIC_KEY).kind(Kind::TextNote);
        run_test(query_filter).await;
    }

    #[tokio::test]
    async fn query_by_author_kind_and_limit() {
        let query_filter = Filter::new()
            .author(*PUBLIC_KEY)
            .kind(Kind::TextNote)
            .limit(3);
        run_test(query_filter).await;
    }
}
