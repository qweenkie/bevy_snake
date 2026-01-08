use bevy::prelude::*;
use crossterm::{event, execute, terminal};
use rand::seq::IndexedRandom;
use std::{
    collections::{HashSet, VecDeque},
    io::{self, Write},
    time::{Duration, Instant},
};

const C_GRID_SIZE: UVec2 = uvec2(12, 12);
const C_CENTER: UVec2 = uvec2(C_GRID_SIZE.x / 2, C_GRID_SIZE.y / 2);

#[derive(Component)]
struct Snake {
    body: VecDeque<UVec2>,
    direction: UVec2,
}
impl Snake {
    #[inline]
    pub fn head(&self) -> &UVec2 {
        self.body
            .back()
            .expect("Attempted to get a head of a snake with no body.")
    }
}
fn spawn_snake_on_init(mut commands: Commands) {
    commands.spawn(Snake {
        body: VecDeque::from([C_CENTER]),
        direction: uvec2(0, C_GRID_SIZE.y - 1),
    });
}
fn move_snake(
    mut commands: Commands,
    mut snake_timer: ResMut<SnakeTimer>,
    mut snake_query: Query<&mut Snake>,
    apple_query: Query<(Entity, &Apple)>,
    mut death_writer: MessageWriter<Death>,
) {
    let mut snake = snake_query
        .single_mut()
        .expect("Multiple snakes found smh.");

    if Instant::now().duration_since(snake_timer.0)
        >= Duration::from_millis(250 + (snake.body.len() as u64) / 8)
    {
        snake_timer.0 = Instant::now();

        let head = *snake.head();
        let new_head = uvec2(
            (head.x + snake.direction.x) % C_GRID_SIZE.x,
            (head.y + snake.direction.y) % C_GRID_SIZE.y,
        );

        if snake.body.contains(&new_head) {
            death_writer.write_default();
            return;
        }
        snake.body.push_back(new_head);

        let mut eaten = false;
        for (entity, apple) in apple_query.iter() {
            if new_head == apple.0 {
                commands.entity(entity).despawn();

                eaten = true;
            }
        }
        if !eaten {
            snake.body.pop_front();
        }
    }
}

#[derive(Resource)]
struct SnakeTimer(Instant);
impl Default for SnakeTimer {
    fn default() -> Self {
        Self(Instant::now())
    }
}

#[derive(Component, PartialEq, Eq)]
struct Apple(UVec2);
fn spawn_apple(
    mut commands: Commands,
    apple_query: Query<&Apple>,
    snake_query: Query<&Snake>,
    mut victory_writer: MessageWriter<Victory>,
) {
    if apple_query.is_empty() {
        let occupied = {
            let occupied_deque = &snake_query
                .single()
                .expect("Multiple snakes found smh.")
                .body;

            let mut result = HashSet::new();
            for &pos in occupied_deque {
                result.insert(pos);
            }
            result
        };

        let mut free_cells: Vec<UVec2> =
            Vec::with_capacity((C_GRID_SIZE.x * C_GRID_SIZE.y) as usize);
        for y in 0..C_GRID_SIZE.y {
            for x in 0..C_GRID_SIZE.x {
                let pos = uvec2(x, y);
                if !occupied.contains(&pos) {
                    free_cells.push(pos);
                }
            }
        }

        if free_cells.is_empty() {
            victory_writer.write_default();
        } else {
            commands.spawn(Apple(*free_cells.choose(&mut rand::rng()).unwrap()));
        }
    }
}

#[derive(Message, Default)]
struct Victory;

#[derive(Message, Default)]
struct Death;

fn process_input(mut snake_query: Query<&mut Snake>, mut exit: MessageWriter<AppExit>) {
    if let Ok(mut snake) = snake_query.single_mut() {
        while event::poll(Duration::from_millis(0)).unwrap() {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                snake.direction = match key_event.code {
                    event::KeyCode::Left if snake.direction != uvec2(1, 0) => {
                        uvec2(C_GRID_SIZE.x - 1, 0)
                    }
                    event::KeyCode::Up if snake.direction != uvec2(0, 1) => {
                        uvec2(0, C_GRID_SIZE.y - 1)
                    }
                    event::KeyCode::Down if snake.direction != uvec2(0, C_GRID_SIZE.y - 1) => {
                        uvec2(0, 1)
                    }
                    event::KeyCode::Right if snake.direction != uvec2(C_GRID_SIZE.x - 1, 0) => {
                        uvec2(1, 0)
                    }

                    event::KeyCode::Char('c')
                        if key_event.modifiers.contains(event::KeyModifiers::CONTROL) =>
                    {
                        exit.write(AppExit::from_code(127));
                        return;
                    }
                    _ => {
                        return;
                    }
                }
            }
        }
    }
}

fn render(
    snake_query: Query<&Snake>,
    apple_query: Query<&Apple>,
    victory_reader: MessageReader<Victory>,
    death_reader: MessageReader<Death>,
    mut exit: MessageWriter<AppExit>,
) {
    if !victory_reader.is_empty() {
        let _ = execute!(io::stdout(), terminal::Clear(terminal::ClearType::All));
        let mut out = io::stdout().lock();

        let _ = write!(
            out,
            " __  __  ______   ____    ______  _____   ____    __    __  __  __  __     \r\n"
        );
        let _ = write!(
            out,
            "/\\ \\/\\ \\/\\__  _\\ /\\  _`\\ /\\__  _\\/\\  __`\\/\\  _`\\ /\\ \\  /\\ \\/\\ \\/\\ \\/\\ \\    \r\n"
        );
        let _ = write!(
            out,
            "\\ \\ \\ \\ \\/_/\\ \\/ \\ \\ \\/\\_\\/_/\\ \\/\\ \\ \\/\\ \\ \\ \\L\\ \\ `\\`\\\\/'/\\ \\ \\ \\ \\ \\ \\   \r\n"
        );
        let _ = write!(
            out,
            " \\ \\ \\ \\ \\ \\ \\ \\  \\ \\ \\/_/_ \\ \\ \\ \\ \\ \\ \\ \\ \\ ,  /`\\ `\\ /'  \\ \\ \\ \\ \\ \\ \\  \r\n"
        );
        let _ = write!(
            out,
            "  \\ \\ \\_/ \\ \\_\\ \\__\\ \\ \\L\\ \\ \\ \\ \\ \\ \\ \\_\\ \\ \\ \\\\ \\ `\\ \\ \\   \\ \\_\\ \\_\\ \\_\\ \r\n"
        );
        let _ = write!(
            out,
            "   \\ `\\___/ /\\_____\\\\ \\____/  \\ \\_\\ \\ \\_____\\ \\_\\ \\_\\ \\ \\_\\   \\/\\_\\/\\_\\/\\_\\\r\n"
        );
        let _ = write!(
            out,
            "    `\\/__/  \\/_____/ \\/___/    \\/_/  \\/_____/\\/_/\\/ /  \\/_/    \\/_/\\/_/\\/_/\r\n"
        );
        for _ in 0..(C_GRID_SIZE.y - 7) {
            let _ = write!(out, "\r\n");
        }
        let _ = out.flush();

        exit.write(AppExit::Success);
        return;
    }
    if !death_reader.is_empty() {
        let _ = execute!(io::stdout(), terminal::Clear(terminal::ClearType::All));
        let mut out = io::stdout().lock();

        let _ = write!(
            out,
            " __    __  _____   __  __       __       _____   ____    ____      \r\n"
        );
        let _ = write!(
            out,
            "/\\ \\  /\\ \\/\\  __`\\/\\ \\/\\ \\     /\\ \\     /\\  __`\\/\\  _`\\ /\\  _`\\    \r\n"
        );
        let _ = write!(
            out,
            "\\ `\\`\\\\/'/\\ \\ \\/\\ \\ \\ \\ \\ \\    \\ \\ \\    \\ \\ \\/\\ \\ \\,\\L\\_\\ \\ \\L\\_\\  \r\n"
        );
        let _ = write!(
            out,
            " `\\ `\\ /'  \\ \\ \\ \\ \\ \\ \\ \\ \\    \\ \\ \\  __\\ \\ \\ \\ \\/_\\__ \\\\ \\  _\\L  \r\n"
        );
        let _ = write!(
            out,
            "   `\\ \\ \\   \\ \\ \\_\\ \\ \\ \\_\\ \\    \\ \\ \\L\\ \\\\ \\ \\_\\ \\/\\ \\L\\ \\ \\ \\L\\ \\\r\n"
        );
        let _ = write!(
            out,
            "     \\ \\_\\   \\ \\_____\\ \\_____\\    \\ \\____/ \\ \\_____\\ `\\____\\ \\____/\r\n"
        );
        let _ = write!(
            out,
            "      \\/_/    \\/_____/\\/_____/     \\/___/   \\/_____/\\/_____/\\/___/ \r\n"
        );
        let _ = out.flush();

        exit.write(AppExit::Success);
        return;
    }

    let mut buffer = vec![vec!["  "; C_GRID_SIZE.x as usize]; C_GRID_SIZE.y as usize];

    for apple in apple_query {
        let pos = apple.0;
        buffer[pos.y as usize][pos.x as usize] = " X";
    }
    if let Ok(snake) = snake_query.single() {
        for pos in &snake.body {
            buffer[pos.y as usize][pos.x as usize] = " O";
        }

        let head_pos = snake.head();
        let head_char = match snake.direction {
            d if d == uvec2(C_GRID_SIZE.x - 1, 0) => " ←",
            d if d == uvec2(0, C_GRID_SIZE.y - 1) => " ↑",
            d if d == uvec2(0, 1) => " ↓",
            d if d == uvec2(1, 0) => " →",
            _ => unreachable!(),
        };
        buffer[head_pos.y as usize][head_pos.x as usize] = head_char;
    }

    let mut out = io::stdout().lock();
    let _ = write!(out, "\x1B[H");

    let _ = write!(out, "+");
    for _ in 0..C_GRID_SIZE.x {
        let _ = write!(out, "--");
    }
    let _ = write!(out, "+\r\n");

    for row in buffer {
        let line: String = row.into_iter().collect();
        let _ = write!(out, "|{}|\r\n", line);
    }

    let _ = write!(out, "+");
    for _ in 0..C_GRID_SIZE.x {
        let _ = write!(out, "--");
    }
    let _ = write!(out, "+\r\n");

    let _ = out.flush();
}

fn main() {
    terminal::enable_raw_mode().expect("Couldn't enable raw mode.");
    let _ = execute!(io::stdout(), terminal::Clear(terminal::ClearType::All),);

    App::new()
        .add_plugins(MinimalPlugins)
        .init_resource::<SnakeTimer>()
        .add_message::<Victory>()
        .add_message::<Death>()
        .add_systems(Startup, spawn_snake_on_init)
        .add_systems(FixedPreUpdate, process_input)
        .add_systems(FixedUpdate, move_snake)
        .add_systems(FixedPostUpdate, spawn_apple)
        .add_systems(FixedLast, render)
        .run();

    terminal::disable_raw_mode().expect("Couldn't reset terminal.");
}
