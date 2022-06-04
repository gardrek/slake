use crate::random;
use std::collections::VecDeque;

#[derive(PartialEq, Eq, Clone, Default)]
pub struct Vector(pub isize, pub isize);

impl std::ops::Add<&Vector> for &Vector {
    type Output = Vector;

    fn add(self, other: &Vector) -> Vector {
        Vector(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::AddAssign<&Vector> for Vector {
    fn add_assign(&mut self, other: &Vector) {
        self.0 = self.0 + other.0;
        self.1 = self.1 + other.1;
    }
}

fn remove_from_vec<T: std::cmp::PartialEq>(vec: &mut Vec<T>, search_element: &T) {
    if let Some(index) = vec.iter().position(|value| *value == *search_element) {
        vec.swap_remove(index);
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
    fn default() -> Self {
        Direction::Left
    }
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

    // keep track of which grid tiles are available to spawn objects
    pub free_positions: Vec<Vector>,

    // Snake's head is at the front of the queue. in other words, `snake.get(0)` gets the head
    pub snake: VecDeque<Vector>,
    direction: Direction,
    next_direction: Direction,
    pub hazards: Vec<Vector>,
    pub food: Vec<Vector>,
    //~ pub food: Vector,
    game_over: bool,
    pub score: usize,
    high_score: usize,
    pub high_score_display: usize,
}

impl SnakeGame {
    pub fn new(width: isize, height: isize) -> SnakeGame {
        assert!(width >= 5);
        assert!(height >= 3);

        let snake = VecDeque::with_capacity((width * height).try_into().unwrap());
        let free_positions = Vec::with_capacity((width * height).try_into().unwrap());

        let mut game = SnakeGame {
            width,
            height,
            snake,
            free_positions,
            ..SnakeGame::default()
        };

        game.restart();

        game
    }

    pub fn restart(&mut self) {
        let width = self.width;
        let height = self.height;

        self.clear_board();

        let tail = Vector(width - 1, height / 2);
        self.push_snake_head(tail);

        let head = Vector(width - 2, height / 2);
        self.push_snake_head(head);

        self.add_food(1);

        self.direction = Direction::Left;
        self.next_direction = Direction::Left;
        self.game_over = false;
        self.high_score_display = self.high_score;
        self.score = 0;
    }

    fn clear_board(&mut self) {
        self.snake.clear();
        self.hazards.clear();
        self.food.clear();
        self.init_free_positions();
    }

    fn push_snake_head(&mut self, head: Vector) {
        remove_from_vec(&mut self.free_positions, &head);
        self.snake.push_front(head);
    }

    fn pop_snake_tail(&mut self) {
        let pos = self.snake.pop_back().unwrap();
        if !self.hazards.contains(&pos) {
            self.free_positions.push(pos);
        }
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

    fn init_free_positions(&mut self) {
        self.free_positions.clear();

        self.free_positions.extend(
            (0..self.height)
                .flat_map(|y| (0..self.width).map(move |x| Vector(x, y)))
                .filter(|pos| {
                    !self.snake.contains(pos)
                        && !self.hazards.contains(pos)
                        && !self.food.contains(pos)
                }),
        );
    }

    pub fn tick(&mut self) {
        if self.game_over {
            return;
        }

        self.direction = self.next_direction.clone();

        // get new head position
        let new_head = {
            let old_head = self.snake.get(0).unwrap();

            &self.direction.to_vector() + old_head
            
        };

        if !self.is_within_board(&new_head) {
            self.end_game("avoid walls");
            return;
        }

        if self.snake.contains(&new_head) {
            self.end_game("avoid crashing into your own tail");
            return;
        }

        if self.hazards.contains(&new_head) {
            self.end_game("don't slip on the leftovers");
            return;
        }

        // add new head
        self.push_snake_head(new_head.clone());

        // check for eating
        if self.food.contains(&new_head) {
            self.score += 1;

            let tail_pos = self.snake.back().unwrap();

            // note that we don't check if there's a hazard here. in the uncommon event that
            // two food items are directly next to each other, two hazards can spawn in the same
            // space. experts say this is "fine"
            self.hazards.push(tail_pos.clone());

            remove_from_vec(&mut self.food, &new_head);

            //~ self.add_food(self.score);
            self.add_food(1);
        } else {
            // remove tail if only if not eating; in other words, we grow if we eat
            self.pop_snake_tail();
        }
    }

    pub fn get_semi_open_tiles(&self) -> Vec<Vector> {
        let snake_head = self.snake[0].clone();

        // Couldn't figure out how to do this with iterators haha
        // should compile down about the same
        let mut vec = vec![];

        for pos in self.adjacent_tiles(&snake_head) {
            vec.push(pos);
        }

        for fruit in self.food.iter() {
            for pos in self.adjacent_tiles(fruit) {
                vec.push(pos);
            }
        }

        vec
    }

    fn adjacent_tiles(&self, position: &Vector) -> impl Iterator<Item = Vector> + '_ {
        [
            Vector(position.0 - 1, position.1),
            Vector(position.0 + 1, position.1),
            Vector(position.0, position.1 - 1),
            Vector(position.0, position.1 + 1),
        ]
        .into_iter()
        .filter(|pos| self.is_within_board(pos))
    }

    fn add_food(&mut self, number: usize) {
        // TODO: make this take into account the "semi-open" tiles to either completely avoid
        // placing food in them, or to reduce the chances

        for _i in 0..number {
            if self.free_positions.is_empty() {
                // Kill screen
                self.end_game("can't believe you made it this far");
            } else {
                let position_index =
                    random::get_u16() as usize % self.free_positions.len() as usize;

                // removes the element at the index and replaces it with the last element
                let position = self.free_positions.swap_remove(position_index);

                self.food.push(position);
            }
        }
    }

    fn end_game(&mut self, message: &'static str) {
        self.game_over = true;

        if self.score >= self.high_score {
            self.high_score = self.score;
        }

        let score_text = format!(
            "{} / Score: {} / High Score: {}",
            message, self.score, self.high_score
        );

        crate::log(&score_text);
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
