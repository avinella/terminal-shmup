use crossterm::{ExecutableCommand, QueueableCommand, cursor, event::{poll, read, Event, KeyCode}, terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode}};
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{thread_rng, Rng};
use std::fs::{File, OpenOptions};
use std::time::{Duration, Instant};
use std::{
    io::{prelude::*, stdout, SeekFrom, Write},
    usize,
};
use text_engine::*;

// Map dimensions
const HEIGHT: usize = 30;
const WIDTH: usize = 30;

const PLAYER_SYM: char = '@';
const BULLET_SYM: char = '|';
const ENEMY_SYM: char = 'X';

const ENEMY_SPD: u128 = 300; // Enemy movement speed
const ENEMY_GEN_SPD: u128 = 1000; // Enemy generation speed
const BULLET_SPD: u128 = 200; // Bullet movement speed

const MAX_HEALTH: usize = 3;

/// Updates the health bar display
///
/// # Arguments
///
/// * health: the number of health dots to display
fn update_health(health: usize) {
    stdout()
        .queue(cursor::MoveTo(
            (5 * WIDTH / 4) as u16,
            ((3 * HEIGHT) / 5 + 2) as u16,
        ))
        .unwrap();
    stdout().queue(Clear(ClearType::UntilNewLine)).unwrap();
    stdout()
        .queue(cursor::MoveTo(
            (5 * WIDTH / 4) as u16,
            (3 * HEIGHT / 5 + 2) as u16,
        ))
        .unwrap();
    for _i in 0..health {
        print!("* ");
    }
    stdout().execute(cursor::RestorePosition).unwrap();
}

/// Updates all side stats: highscore, score, and health bar
///
/// # Arguments
///
/// * highscore: current highscore
/// * score: current score
/// * health: the number of health dots to display
fn update_stats(highscore: i32, score: i32, health: usize) {
    stdout()
        .queue(cursor::MoveTo((5 * WIDTH / 4) as u16, (HEIGHT / 5) as u16))
        .unwrap();
    print!("HIGHSCORE");
    stdout()
        .queue(cursor::MoveTo(
            (5 * WIDTH / 4) as u16,
            (HEIGHT / 5 + 2) as u16,
        ))
        .unwrap();
    print!("{}", highscore);

    stdout()
        .queue(cursor::MoveTo(
            (5 * WIDTH / 4) as u16,
            (2 * HEIGHT / 5) as u16,
        ))
        .unwrap();
    print!("SCORE");
    stdout()
        .queue(cursor::MoveTo(
            (5 * WIDTH / 4) as u16,
            (2 * HEIGHT / 5 + 2) as u16,
        ))
        .unwrap();
    print!("{}", score);
    stdout().execute(cursor::RestorePosition).unwrap();
    stdout().flush().unwrap();

    stdout()
        .queue(cursor::MoveTo(
            (5 * WIDTH / 4) as u16,
            ((3 * HEIGHT) / 5) as u16,
        ))
        .unwrap();
    print!("HEALTH");
    update_health(health);
}

fn main() {
    // Set up map and player
    let mut player = Player::new((WIDTH / 2, HEIGHT - 2), PLAYER_SYM, MAX_HEALTH);
    let mut map = Map::new(WIDTH, HEIGHT);
    map.generate();
    execute!(stdout(), EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    execute!(stdout(), Clear(ClearType::All)).unwrap();
    stdout().execute(cursor::MoveTo(0, 0)).unwrap();

    let mut mode = GameMode::Title;

    // Current bullets and enemies active
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut enemies: Vec<Enemy> = Vec::new();

    // Timers for moving and generating bullets/enemies
    let mut new_enemy = Instant::now();
    let mut move_enemy = Instant::now();
    let mut move_bullet = Instant::now();

    // Read in current highscore
    let mut file = File::open("highscore.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut highscore = contents.parse::<i32>().unwrap();
    let mut score = 0;

    loop {
        // Hide cursor and save the beginning position of the cursor
        stdout().execute(cursor::Hide).unwrap();
        stdout().execute(cursor::MoveTo(0, 0)).unwrap();
        stdout().execute(cursor::SavePosition).unwrap();

        match mode {
            GameMode::Playing => {
                // Move existing enemies
                if move_enemy.elapsed().as_millis() > ENEMY_SPD && !(&enemies).is_empty() {
                    let mut to_remove: Vec<(usize, usize)> = Vec::new();
                    for mut e in &mut enemies {
                        // Move enemies down one
                        if e.pos.1 < HEIGHT - 3 {
                            map.set(e.pos, ' ');
                            map.set((e.pos.0 + 1, e.pos.1), ' ');
                            e.pos.1 += 1;
                            map.set((e.pos.0, e.pos.1 + 1), ENEMY_SYM);
                            map.set((e.pos.0 + 1, e.pos.1 + 1), ENEMY_SYM);

                            // Check if ran into the player
                            let pos = hit_enemy(&mut player, e, &mut map);
                            if pos != (0, 0) {
                                to_remove.push(pos);
                                update_health(player.health);
                            }
                        } else {
                            // If it reached the bottom
                            player.health -= 1;
                            update_health(player.health);
                            to_remove.push(e.pos);
                            map.set(e.pos, ' ');
                            map.set((e.pos.0 + 1, e.pos.1), ' ');
                            map.set((e.pos.0, e.pos.1 + 1), ' ');
                            map.set((e.pos.0 + 1, e.pos.1 + 1), ' ');
                        }
                    }
                    // Remove any enemies no longer in play
                    for e in to_remove {
                        let index = enemies.iter().position(|x| x.pos == e).unwrap();
                        enemies.remove(index);
                    }
                    move_enemy = Instant::now();
                }

                // Move existing bullets forwards
                if move_bullet.elapsed().as_millis() > BULLET_SPD && !(&bullets).is_empty() {
                    let mut to_remove: Vec<(usize, usize)> = Vec::new();
                    for mut b in &mut bullets {
                        map.set(b.pos, ' ');
                        if b.pos.1 > 1 {
                            b.pos.1 -= 1;
                            // Check for collisions with enemies
                            if map.get(b.pos) == ENEMY_SYM {
                                to_remove.push(b.pos);
                                let index = enemies
                                    .iter()
                                    .position(|x| {
                                        (x.pos.1 == b.pos.1 || x.pos.1 + 1 == b.pos.1)
                                            && (x.pos.0 == b.pos.0 || x.pos.0 + 1 == b.pos.0)
                                    })
                                    .unwrap();
                                map.set(enemies[index].pos, ' ');
                                map.set((enemies[index].pos.0 + 1, enemies[index].pos.1), ' ');
                                map.set((enemies[index].pos.0, enemies[index].pos.1 + 1), ' ');
                                map.set((enemies[index].pos.0 + 1, enemies[index].pos.1 + 1), ' ');
                                enemies.remove(index);

                                // Add to score
                                score += 50;
                                if score > highscore {
                                    highscore = score;
                                }
                            } else {
                                map.set(b.pos, BULLET_SYM);
                            }
                        } else {
                            to_remove.push(b.pos);
                        }
                    }
                    // Remove unnecessary bullets
                    for b in to_remove {
                        let index = bullets.iter().position(|x| x.pos == b).unwrap();
                        bullets.remove(index);
                    }
                    move_bullet = Instant::now();
                }

                // Generate an enemy every ENEMY_GEN_SPD seconds
                if new_enemy.elapsed().as_millis() > ENEMY_GEN_SPD {
                    let x = thread_rng().gen_range(1, WIDTH - 2);
                    enemies.push(Enemy::new((x, 1), ENEMY_SYM));
                    map.add_enemy((x, 1), ENEMY_SYM);
                    new_enemy = Instant::now();
                }

                // Refresh screen/map to update changes
                stdout().queue(cursor::RestorePosition).unwrap();
                map.set(player.pos, player.symbol);
                map.display();

                // Display side stats (highscore, score, and health bar)
                update_stats(highscore, score, player.health);

                // Respond to key inputs
                // WASD to move player
                // Up to shoot
                // Esc to go to quit menu
                let key_pressed;
                if poll(Duration::from_millis(200)).unwrap() {
                    if let Event::Key(event) = read().unwrap() {
                        key_pressed = event;
                        match key_pressed.code {
                            KeyCode::Esc => {
                                mode = GameMode::Pause;
                            }
                            KeyCode::Up => {
                                let pos = player.pos;
                                bullets.push(Bullet::new((pos.0, pos.1 - 1), BULLET_SYM));
                                map.set((pos.0, pos.1 - 1), BULLET_SYM);
                            }
                            KeyCode::Char('s') => {
                                if player.pos.1 < (HEIGHT - 2) {
                                    map.set(player.pos, ' ');
                                    player.pos.1 += 1
                                }
                            }
                            KeyCode::Char('w') => {
                                if player.pos.1 > 1 {
                                    map.set(player.pos, ' ');
                                    player.pos.1 -= 1
                                }
                            }
                            KeyCode::Char('a') => {
                                if player.pos.0 > 1 {
                                    map.set(player.pos, ' ');
                                    player.pos.0 -= 1
                                }
                            }
                            KeyCode::Char('d') => {
                                if player.pos.0 < (WIDTH - 2) {
                                    map.set(player.pos, ' ');
                                    player.pos.0 += 1
                                }
                            }
                            _ => (),
                        }
                    }
                }
                // Check if player ran into enemy
                let mut to_remove: Vec<(usize, usize)> = Vec::new();
                for e in &enemies {
                    let pos = hit_enemy(&mut player, e, &mut map);
                    if pos != (0, 0) {
                        to_remove.push(pos);
                        update_health(player.health);
                    }
                }
                for e in to_remove {
                    let index = enemies.iter().position(|x| x.pos == e).unwrap();
                    enemies.remove(index);
                }

                if player.health == 0 {
                    mode = GameMode::GameOver;
                }
            }
            GameMode::Pause => {
                // Display pause menu
                let mut text = String::new();
                let mut file = File::open("pause.txt").unwrap();
                file.read_to_string(&mut text).unwrap();
                stdout()
                    .queue(cursor::MoveTo(0, (HEIGHT / 3) as u16))
                    .unwrap();
                print!("{}", text);
                stdout().execute(cursor::RestorePosition).unwrap();

                // Esc to return to game
                // Enter to quit
                let key_pressed;
                if let Event::Key(event) = read().unwrap() {
                    key_pressed = event;
                    if let KeyCode::Enter = key_pressed.code {
                        disable_raw_mode().unwrap();
                        execute!(stdout(), LeaveAlternateScreen).unwrap();
                        break;
                    }
                    if let KeyCode::Esc = key_pressed.code {
                        mode = GameMode::Playing;
                    }
                }
            }
            GameMode::Title => {
                // Display rules and map, wait for input
                map.display();

                // Title text and rules
                stdout().execute(cursor::MoveTo(0, 0)).unwrap();
                let mut text = String::new();
                let mut file = File::open("title.txt").unwrap();
                file.read_to_string(&mut text).unwrap();
                stdout()
                    .queue(cursor::MoveTo(0, (HEIGHT / 3) as u16))
                    .unwrap();
                print!("{}", text);
                stdout().execute(cursor::MoveTo(0, 0)).unwrap();

                let key_pressed;
                if let Event::Key(event) = read().unwrap() {
                    key_pressed = event;
                    if let KeyCode::Enter = key_pressed.code {
                        stdout().execute(Clear(ClearType::All)).unwrap();
                        stdout().execute(cursor::RestorePosition).unwrap();
                        mode = GameMode::Playing;
                    }
                    if let KeyCode::Esc = key_pressed.code {
                        disable_raw_mode().unwrap();
                        execute!(stdout(), LeaveAlternateScreen).unwrap();
                        break;
                    }
                }
            }
            GameMode::GameOver => {
                // Display game over text
                stdout()
                    .queue(cursor::MoveTo((WIDTH / 4 + 2) as u16, (HEIGHT / 2) as u16))
                    .unwrap();
                print!("GAME OVER");
                stdout()
                    .queue(cursor::MoveTo((WIDTH / 8) as u16, (HEIGHT / 2 + 1) as u16))
                    .unwrap();
                print!("Press ENTER to play again");
                stdout().flush().unwrap();

                // Check if the player made a new highscore and update as necessary
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("highscore.txt")
                    .unwrap();
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let old = contents.parse::<i32>().unwrap();
                if highscore > old {
                    file.seek(SeekFrom::Start(0)).unwrap();
                    file.write_all(highscore.to_string().as_bytes()).unwrap();
                }

                // Esc to quit
                // Enter to play again
                let key_pressed;
                if let Event::Key(event) = read().unwrap() {
                    key_pressed = event;
                    if let KeyCode::Esc = key_pressed.code {
                        disable_raw_mode().unwrap();
                        execute!(stdout(), LeaveAlternateScreen).unwrap();
                        break;
                    }
                    if let KeyCode::Enter = key_pressed.code {
                        stdout().execute(Clear(ClearType::All)).unwrap();
                        mode = GameMode::Playing;

                        // Reset Game
                        map = Map::new(WIDTH, HEIGHT);
                        map.generate();
                        player = Player::new((WIDTH / 2, HEIGHT - 2), PLAYER_SYM, MAX_HEALTH);
                        enemies = Vec::new();
                        bullets = Vec::new();
                        score = 0;
                        stdout().execute(cursor::RestorePosition).unwrap();
                    }
                }
            }
        }
    }
}
