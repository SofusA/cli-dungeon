CREATE TABLE characters (
    name TEXT NOT NULL,
    secret INTEGER NOT NULL,
    character_type TEXT NOT NULL,
    gold INTEGER NOT NULL DEFAULT 0,
    experience INTEGER NOT NULL DEFAULT 0,
    base_ability_scores TEXT NOT NULL,
    current_health INTEGER NOT NULL,
    equipped_weapon TEXT,
    equipped_offhand TEXT,
    equipped_armor TEXT,
    equipped_jewelry TEXT NOT NULL,
    weapon_inventory TEXT NOT NULL,
    armor_inventory TEXT NOT NULL,
    item_inventory TEXT NOT NULL,
    jewelry_inventory TEXT NOT NULL,
    level_up_choices TEXT NOT NULL,
    party_id INTEGER NOT NULL,
    encounter_id INTEGER,
    status TEXT NOT NULL,
    quest_points INTEGER NOT NULL DEFAULT 0,
    short_rests INTEGER NOT NULL DEFAULT 0,
    active_conditions TEXT NOT NULL
);

CREATE TABLE active_character (
    id INTEGER NOT NULL,
    secret INTEGER NOT NULL
);

CREATE TABLE encounters (
    rotation TEXT NOT NULL,
    dead_characters TEXT NOT NULL
);

CREATE TABLE party_counter (
    value INTEGER
);


INSERT INTO active_character (id, secret) VALUES (0, 0);
INSERT INTO party_counter (value) VALUES (0);

