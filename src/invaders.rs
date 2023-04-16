use std::{time::Duration, cmp::max};

use rusty_time::Timer;
use crate::{NUM_COLS, NUM_ROWS, frame::Drawable};

pub struct Invader {
  pub x: usize,
  pub y: usize,
}

pub struct Invaders {
  pub army: Vec<Invader>,
  move_timer: Timer,
  direction: i32,
  last_line: bool
}

impl Invaders {
    pub fn new(lines: usize) -> Self {
      let mut army = Vec::new();
      for x in 0..NUM_COLS {
        for y in 0..NUM_ROWS {
          if (x > 1)
            && (x < NUM_COLS - 2)
            && (y > 0)
            && (y < (lines * 2) + 1)
            && (x % 2 == 0)
            && (y % 2 == 0) {
            army.push(Invader { x, y });
          }
        }
      }
      Self {
        army,
        move_timer: Timer::from_millis(2000),
        direction: 1,
        last_line: false
      }
    }
    pub fn have_won(&self) -> bool {
      self.last_line
    }
    pub fn update(&mut self, delta: Duration) -> bool {
      self.move_timer.update(delta);
      if self.move_timer.ready {
        self.move_timer.reset();
        let mut downwards = false;
        if self.direction == -1 {
          let min_x = self.army.iter().map(|invader| invader.x).min().unwrap_or(0);
          let is_leftmost: bool = min_x == 0;
          if is_leftmost {
            self.direction = 1;
            downwards = true;
          }
        } else {
          let max_x = self.army.iter().map(|invader| invader.x).max().unwrap_or(0);
          if max_x == NUM_COLS - 1 {
            self.direction = -1;
            downwards = true;
          }
        }
        if downwards {
          let new_duration = max(self.move_timer.duration.as_millis() - 250, 250);
          self.move_timer = Timer::from_millis(new_duration as u64);
          for invader in self.army.iter_mut() {
            invader.y += 1;
            if invader.y == NUM_ROWS - 1 {
              self.last_line = true;
              return true;
            }
          }
        } else {
          for invader in self.army.iter_mut() {
            invader.x = (invader.x as i32 + self.direction) as usize;
          }
        }
        return true;
      }
      return false;
    }
}

impl Drawable for Invaders {
  fn draw(&self, frame: &mut crate::frame::Frame) {
      for invader in self.army.iter() {
        // When time left is over half the duration
        let alt_character: bool = (
          self
            .move_timer
            .time_left
            .as_secs_f32()
            / self
            .move_timer
            .duration
            .as_secs_f32()
          ) > 0.5;
        frame[invader.x][invader.y] = if alt_character { "x" } else { "+" };
      }
  }
}