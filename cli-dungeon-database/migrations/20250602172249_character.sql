CREATE TABLE characters (
    name TEXT NOT NULL,
    secret INTEGER NOT NULL,
    attack_dice TEXT NOT NULL,
    hit_bonus INTEGER NOT NULL,
    max_health INTEGER NOT NULL,
    current_health INTEGER NOT NULL,
    armor_points INTEGER NOT NULL
);

CREATE TABLE active_character (
    id INTEGER NOT NULL,
    secret INTEGER NOT NULL
);
