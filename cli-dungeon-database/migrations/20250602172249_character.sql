CREATE TABLE characters (
    name TEXT NOT NULL,
    secret INTEGER NOT NULL,
    base_ability_scores TEXT NOT NULL,
    current_health INTEGER NOT NULL
);

CREATE TABLE active_character (
    id INTEGER NOT NULL,
    secret INTEGER NOT NULL
);
