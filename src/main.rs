use std::cmp;

use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const LIMIT_FPS: i32 = 20;

const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};

struct Tcod {
    root: Root,
    con: Offscreen,
}

#[derive(Debug)]
struct Object {
    x: i32,
    y: i32,
    char: char,
    color: Color,
}

impl Object {
    pub fn new(x: i32, y: i32, char: char, color: Color) -> Self {
        Object { x, y, char, color }
    }

    pub fn move_by(&mut self, game: &Game, dx: i32, dy: i32) {
        let x = self.x + dx;
        let y = self.y + dy;

        if !game.tile_at(x, y).blocked {
            self.x = x;
            self.y = y;
        }
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
    }
}

#[derive(Copy, Clone, Debug)]
struct Tile {
    blocked: bool,
    block_sight: bool,
}

impl Tile {
    pub fn empty() -> Self {
        Tile {
            blocked: false,
            block_sight: false,
        }
    }

    pub fn wall() -> Self {
        Tile {
            blocked: true,
            block_sight: true,
        }
    }
}

/// A rectangle on the map, used to characterise a room.
#[derive(Clone, Copy, Debug)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }
}

type Map = Vec<Vec<Tile>>;

struct Game {
    map: Map,
}

impl Game {
    pub fn new() -> Self {
        Game {
            map: make_map(),
        }
    }

    pub fn tile_at(&self, x: i32, y: i32) -> Tile {
        self.map[x as usize][y as usize]
    }
}

fn create_room(map: &mut Map, room: Rect) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    // horizontal tunnel. `min()` and `max()` are used in case `x1 > x2`
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    // vertical tunnel. `min()` and `max()` are used in case `y1 > y2`
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn make_map() -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(50, 15, 10, 15);
    create_room(&mut map, room1);
    create_room(&mut map, room2);
    create_h_tunnel(&mut map, 25, 55, 23);

    map
}

fn main() {
    let root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Roguelike")
        .init();

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);

    let mut tcod = Tcod { root, con };

    let game = Game::new();

    let mut objects = [
        // Object::new(MAP_WIDTH / 2, MAP_HEIGHT / 2, '@', WHITE),
        // Object::new(MAP_WIDTH / 2 - 5, MAP_HEIGHT / 2, 'X', YELLOW),
        Object::new(25, 23, '@', WHITE),
        Object::new(55, 23, 'X', YELLOW),
    ];

    tcod::system::set_fps(LIMIT_FPS);

    while !tcod.root.window_closed() {
        tcod.con.clear();

        render_all(&mut tcod, &game, &objects);

        tcod.root.flush();

        let player = &mut objects[0]; // TODO: this seems icky
        let exit = handle_keys(&mut tcod, &game, player);

        if exit {
            break;
        }
    }
}

// Return true to exit, false to continue
fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    let key = tcod.root.wait_for_keypress(true);

    match key {
        Key { code: Up, .. } => player.move_by(game, 0, -1),
        Key { code: Down, .. } => player.move_by(game, 0, 1),
        Key { code: Left, .. } => player.move_by(game, -1, 0),
        Key { code: Right, .. } => player.move_by(game, 1, 0),

        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true,

        _ => {}
    }

    false
}

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    for object in objects {
        object.draw(&mut tcod.con);
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set);
            }
        }
    }

    // blit the contents of "con" to the root console and present it
    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}
