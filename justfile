init-database:
    cargo sqlx db create && cargo sqlx migrate run --source cli-dungeon-database/migrations

create-env-file:
    echo 'DATABASE_URL="sqlite:///tmp/cli-dungeon.db"' > .env
