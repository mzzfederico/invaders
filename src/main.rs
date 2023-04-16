use std::{error::Error, io, time::{Duration, Instant}, sync::mpsc, thread, env};
use crossterm::{terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, ExecutableCommand, cursor::{Hide, Show}, event::{self, Event, KeyCode}};
use invaders::{render, frame::{new_frame, Drawable}, player::Player, invaders::Invaders};
use rusty_audio::Audio;

fn main() -> Result <(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let mut enemy_lines = 4;
    match args[1].parse::<u32>() {
        Ok(lines)   => {
            if lines > 1 && lines < 7 {
                enemy_lines = lines;
            }
        },
        Err(_) => {
            enemy_lines = 4;
        }
    };

    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");
    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();

    let render_handle = thread::spawn(move || {
        let mut last_frame = new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new(enemy_lines as usize);
    'gameloop: loop {
        // Per frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }
        // Updates
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.check_hits(&mut invaders.army) {
            audio.play("explode");
        }
        if invaders.army.len() == 0 {
            invaders::render::clear(&mut stdout);
            audio.play("win");
            break 'gameloop;
        }

        // Draw
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }

        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
