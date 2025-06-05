CREATE TABLE characters (
    name TEXT NOT NULL,
    secret INTEGER NOT NULL,
    player BOOLEAN NOT NULL,
    gold INTEGER NOT NULL,
    experience INTEGER NOT NULL,
    base_ability_scores TEXT NOT NULL,
    current_health INTEGER NOT NULL,
    equipped_weapon TEXT,
    equipped_offhand TEXT,
    equipped_armor TEXT,
    weapon_inventory TEXT NOT NULL,
    armor_inventory TEXT NOT NULL
);

CREATE TABLE active_character (
    id INTEGER NOT NULL,
    secret INTEGER NOT NULL
);
