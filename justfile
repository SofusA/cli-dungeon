init-database:
    cargo sqlx db create && cargo sqlx migrate run --source cli-dungeon-database/migrations
