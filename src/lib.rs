mod random;
mod snake;

use crate::snake::Direction;
use crate::snake::SnakeGame;
use crate::snake::Vector;

use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{console, window, HtmlDivElement, HtmlElement, KeyboardEvent};

thread_local! {
    static GAME: Rc<RefCell<SnakeGame>> = Rc::new(RefCell::new(SnakeGame::new(20, 16)));

    static TICK_CLOSURE: Closure<dyn FnMut()> = Closure::wrap(Box::new({
        || {
            GAME.with(|game| game.borrow_mut().tick());
            render().unwrap_throw();
        }
    }) as Box<dyn FnMut()>);

    static HANDLE_KEYDOWN: Closure<dyn FnMut(KeyboardEvent)> = Closure::wrap(Box::new({
        |event: KeyboardEvent| {
            let direction = match &event.key()[..] {
                "ArrowUp" => Direction::Up,
                "ArrowDown" => Direction::Down,
                "ArrowLeft" => Direction::Left,
                "ArrowRight" => Direction::Right,
                " " => {
                    GAME.with(|game| game.borrow_mut().restart());
                    event.prevent_default();
                    return;
                },
                _ => return,
            };
            GAME.with(|game| game.borrow_mut().change_direction(direction));
            event.prevent_default();
        }
    }) as Box<dyn FnMut(KeyboardEvent)>);
}

#[wasm_bindgen(start)]
pub fn main() {
    console::log_1(&"Starting...".into());

    TICK_CLOSURE.with(|closure| {
        window()
            .unwrap_throw()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().dyn_ref::<Function>().unwrap_throw(),
                100,
            )
            .unwrap_throw()
    });

    HANDLE_KEYDOWN.with(|handle_keydown| {
        window()
            .unwrap_throw()
            .add_event_listener_with_callback(
                "keydown",
                handle_keydown.as_ref().dyn_ref::<Function>().unwrap_throw(),
            )
            .unwrap_throw();
    });
}

fn render() -> Result<(), JsValue> {
    let document = window().unwrap_throw().document().unwrap_throw();

    let root_container = document
        .get_element_by_id("root")
        .unwrap_throw() // we unwrap this one because it's actually an option so, it's easier to just throw here
        .dyn_into::<HtmlElement>()?;

    let height = GAME.with(|game| game.borrow().height);
    let width = GAME.with(|game| game.borrow().width);

    root_container.set_inner_html("");

    root_container
        .style()
        .set_property("display", "inline-grid")?;
    root_container.style().set_property(
        "grid-template",
        &format!("repeat({height}, auto) / repeat({width}, auto)"),
    )?;

    for y in 0..height {
        for x in 0..width {
            let pos = Vector(x, y);

            let field_element = document
                .create_element("div")?
                .dyn_into::<HtmlDivElement>()?;

            field_element.set_class_name("field");

            GAME.with(|game| {
                field_element.set_inner_text({
                    if pos == game.borrow().food {
                        "🍆"
                    } else if pos == game.borrow().snake[0] {
                        "😩"
                    } else if game.borrow().snake.contains(&pos) {
                        "🟡"
                    } else if game.borrow().hazards.contains(&pos) {
                        "💦"
                    } else {
                        ""
                    }
                });
            });

            root_container.append_child(&field_element)?;
        }
    }

    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}