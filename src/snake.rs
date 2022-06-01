use crate::random;
use std::collections::VecDeque;

#[derive(PartialEq, Eq, Clone, Default)]
pub struct Vector(pub isize, pub isize);

impl Vector {
    fn add(&self, other: &Vector) -> Vector {
        Vector(self.0 + other.0, self.1 + other.1)
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Default for Direction {
    fn default() -> Self { Direction::Left }
}

impl Direction {
    fn to_vector(&self) -> Vector {
        use Direction::*;
        match self {
            Up => Vector(0, -1),
            Right => Vector(1, 0),
            Down => Vector(0, 1),
            Left => Vector(-1, 0),
        }
    }

    fn opposite(&self) -> Direction {
        use Direction::*;
        match self {
            Up => Down,
            Right => Left,
            Down => Up,
            Left => Right,
        }
    }
}

#[derive(Default)]
pub struct SnakeGame {
    pub width: isize,
    pub height: isize,
    // Snake's head is at the front of the queue. in other words, `snake.get(0)` gets the head
    pub snake: VecDeque<Vector>,
    direction: Direction,
    next_direction: Direction,
    pub hazards: Vec<Vector>,
    pub food: Vector,
    game_over: bool,
    score: usize,
    high_score: usize,
}

impl SnakeGame {
    pub fn new(width: isize, height: isize) -> SnakeGame {
        assert!(width >= 5);
        assert!(height >= 3);

        let mut game = SnakeGame{
            width,
            height,
            ..SnakeGame::default()
        };

        game.restart();

        game
    }

    pub fn restart(&mut self) {
        let width = self.width;
        let height = self.height;

        let head = Vector(width - 2, height / 2);
        let tail = Vector(width - 1, height / 2);

        let mut snake = VecDeque::with_capacity((width * height).try_into().unwrap());

        snake.push_front(head);
        snake.push_back(tail);

        self.snake = snake;
        self.direction = Direction::Left;
        self.next_direction = Direction::Left;
        self.food = Vector(width / 2, height / 2);
        self.hazards = vec![];
        self.game_over = false;
        self.score = 1;
    }

    pub fn change_direction(&mut self, direction: Direction) {
        if self.direction == direction || self.direction.opposite() == direction {
            return;
        }

        self.next_direction = direction;
    }

    fn is_within_board(&self, &Vector(x, y): &Vector) -> bool {
        x >= 0 && y >= 0 && x < self.width && y < self.height
    }

    pub fn tick(&mut self) {
        if self.game_over {
            return;
        }

        self.direction = self.next_direction.clone();

        // assume snake length is at least 2 to avoid weird edge cases we don't need
        assert!(self.snake.len() >= 2);

        // get new head position
        let new_head = {
            let old_head = self.snake.get(0).unwrap();

            self.direction.to_vector().add(old_head)
        };

        if !self.is_within_board(&new_head)
            || self.snake.contains(&new_head)
            || self.hazards.contains(&new_head)
        {
            self.end_game();
            return;
        }

        let tail_pos = self.snake.back().unwrap().clone();

        // check for eating
        if new_head == self.food {
            self.score += 1;
            
            self.hazards.push(tail_pos);

            let free_positions = (0..self.height)
                .flat_map(|y| (0..self.width).map(move |x| Vector(x, y)))
                .filter(|pos| !self.snake.contains(pos) && !self.hazards.contains(pos))
                .collect::<Vec<_>>();

            if free_positions.is_empty() {
                self.end_game();
            } else {
                let position_index = random::get_u16() as usize % free_positions.len() as usize;

                self.food = free_positions[position_index].clone();
            }
        } else {
            // remove tail if only if not eating; in other words, we grow if we eat
            self.snake.pop_back().unwrap();
        }

        // add new head
        self.snake.push_front(new_head);
    }

    pub fn end_game(&mut self) {
        self.game_over = true;

        if self.score >= self.high_score {
            self.high_score = self.score;
        }
        
        crate::log(&format!("Score: {} / High Score: {}", self.score, self.high_score));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut game = SnakeGame::new(5, 5);

        //~ dbg!(&game);

        for _i in 0..4 {
            game.tick();
            //~ dbg!(&game);
        }

        assert!(game.game_over);
    }
}
