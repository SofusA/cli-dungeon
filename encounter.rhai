let enemies = state.enemies;

// Sort enemies by health
enemies.sort(|b, a| a.health - b.health);

// Attack with both action and bonus action on lowest health
let target = state.enemies[0];
react("attack", target.id, "attack", target.id)
