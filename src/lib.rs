/// Game map represented by a 2d vector
pub struct Map {
    width: usize,
    height: usize,
    pub grid: Vec<Vec<char>>,
}
impl Map {
    /// Returns a new empty map
    ///
    /// # Arguments
    /// * width: the width of the map in terminal
    /// * height: the height of the map in terminal
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![vec![' '; width]; height],
        }
    }

    /// Adds walls around the borders of a map
    pub fn generate(&mut self) {
        // Populates grid and sets walls

        self.grid[0] = vec!['#'; self.width];
        self.grid[self.height - 1] = vec!['#'; self.width];
        for row in self.grid.iter_mut() {
            row[0] = '#';
            row[self.width - 1] = '#';
        }
    }

    /// Print the map in terminal
    pub fn display(&self) {
        // Prints the map to stdout
        for row in &self.grid {
            for c in row {
                print!("{}", c);
            }
            print!("\r\n");
        }
    }

    /// Set a space in the map to a character
    ///
    /// # Arguments
    /// * (x, y): coordinates to add
    /// * c: the char to be added
    pub fn set(&mut self, (x, y): (usize, usize), c: char) {
        // Sets a space on the map to a given character
        self.grid[y][x] = c;
    }

    /// Returns the character in a space
    ///
    /// # Arguments
    /// * (x, y): coordinates to search
    pub fn get(&self, (x, y): (usize, usize)) -> char {
        // Gets the character in a given space on the map
        self.grid[y][x]
    }

    /// Adds a 2x2 enemy to the map
    ///
    /// # Arguments
    /// * (x, y): the upper-left corner of the enemy
    /// * c: the char to represent the enemy
    pub fn add_enemy(&mut self, (x, y): (usize, usize), c: char) {
        self.set((x, y), c);
        self.set((x, y + 1), c);
        self.set((x + 1, y), c);
        self.set((x + 1, y + 1), c);
    }
}

/// Player entity
pub struct Player {
    pub pos: (usize, usize),
    pub symbol: char,
    pub health: usize,
}
impl Player {
    /// Returns a new player
    ///
    /// # Arguments
    /// * (x, y): the starting position on the map
    /// * c: the char to represent the character
    /// * health: health of the player
    pub fn new((x, y): (usize, usize), c: char, health: usize) -> Self {
        Self {
            pos: (x, y),
            symbol: c,
            health,
        }
    }
}

/// Enemy entity
pub struct Enemy {
    pub pos: (usize, usize), // The upper-left coordinate if larger than 1x1
    pub symbol: char,
}
impl Enemy {
    /// Returns a new enemy
    ///
    /// # Arguments
    /// * (x, y): the starting position on the map
    /// * c: the char to represent the enemy
    pub fn new((x, y): (usize, usize), symbol: char) -> Self {
        Self {
            pos: (x, y),
            symbol,
        }
    }
}

/// Bullet entity
pub struct Bullet {
    pub pos: (usize, usize),
    pub symbol: char,
}
impl Bullet {
    /// Returns a new bullet
    ///
    /// # Arguments
    /// * (x, y): the starting position on the map
    /// * c: the char to represent the bullet
    pub fn new((x, y): (usize, usize), c: char) -> Self {
        Self {
            pos: (x, y),
            symbol: c,
        }
    }
}

/// To keep track of the current game mode
pub enum GameMode {
    Playing,  // Currently playing
    Pause,    // Pause screen
    Title,    // Title screen
    GameOver, // Game over screen
}

/// Checks if the player has hit a 2x2 enemy
/// If they have, lower the player's health, remove the enemy from the map,
/// and return the enemy's position (its upper-left coordinates)
/// Otherwise return (0,0)
///
/// # Arguments
///
/// * p: the player to check
/// * e: the enemy to check
/// * map: the current map
pub fn hit_enemy(p: &mut Player, e: &Enemy, map: &mut Map) -> (usize, usize) {
    if (e.pos.1 == p.pos.1 || e.pos.1 + 1 == p.pos.1)
        && (e.pos.0 == p.pos.0 || e.pos.0 + 1 == p.pos.0)
    {
        p.health -= 1;
        map.set(e.pos, ' ');
        map.set((e.pos.0 + 1, e.pos.1), ' ');
        map.set((e.pos.0, e.pos.1 + 1), ' ');
        map.set((e.pos.0 + 1, e.pos.1 + 1), ' ');
        return e.pos;
    }
    (0, 0)
}
