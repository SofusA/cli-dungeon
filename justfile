init-database:
    cargo sqlx db create && cargo sqlx migrate run --source cli-dungeon-database/migrations

create-character:
    cargo run -- create-character --name Swordington -s 0 -d 6 -c 4
    cargo run -- shop buy --item dagger
    cargo run -- character equip --main-hand dagger
    cargo run -- character quest

create-character-bin:
    cli-dungeon create-character --name Swordington -s 0 -d 6 -c 4
    cli-dungeon shop buy --item dagger
    cli-dungeon character equip --main-hand dagger
