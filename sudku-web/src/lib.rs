#![allow(non_snake_case)]

use leptos::*;
use leptos::html::{Div, Input};
use sudku_grid::complex::{Grid3x3, Grid4x4, History, Move, Pos};
use wasm_bindgen::prelude::*;

pub mod console;
mod grid3;
use grid3::*;
mod grid4;
use grid4::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), wasm_bindgen::JsValue> {
    init_wasm_hooks();
    mount_to_body(App);
    Ok(())
}

fn init_wasm_hooks() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}

#[component]
fn App() -> impl IntoView {
    let num_blank_ref: NodeRef<Input> = create_node_ref();
    let (num_blank, set_num_blank) = create_signal(18usize);
    provide_context(num_blank);

    let focused_cell = create_rw_signal(CellInfo::default());
    provide_context(focused_cell);

    let notes_active = create_rw_signal(false);
    provide_context(notes_active);

    let (history, set_history) = create_signal(History::new());
    let (last_mv, set_last_mv) = create_signal(None::<Move>);

    let (using_3x3, set_using_3x3) = create_signal(true);

    let grid3 = create_rw_signal(Grid3x3::empty());
    let grid4 = create_rw_signal(Grid4x4::empty());
    let showing_grid = create_rw_signal(false);

    create_effect(move |_| {
        if showing_grid.get() {
            return;
        }
        spawn_local(async move {
            if using_3x3() {
                if grid3.with(|grid| grid == &Grid3x3::EMPTY) {
                        grid3.set(create_grid3x3(num_blank()).await);
                }
            } else {
                if grid4.with(|grid| grid == &Grid4x4::EMPTY) {
                        grid4.set(create_grid4x4(num_blank()).await);
                }
            }
            showing_grid.set(true);
        });
    });

    view! {
        <div id="main">
            <div id="top-buttons">
                <div>
                    <button
                        on:click=move |_| {
                            let input = num_blank_ref.get().expect("missing num_blank_ref");
                            let num = input.value().parse::<usize>().unwrap_or(18);
                            focused_cell.set(CellInfo::default());
                            set_using_3x3.update(|is_3x3| {
                                if !*is_3x3 {
                                    if num > 80 {
                                        input.set_value("80");
                                        set_num_blank(80);
                                    }
                                }
                                *is_3x3 = !*is_3x3
                            });
                            showing_grid.set(false);
                        }
                    >
                    {move || if using_3x3() { "Switch to 4x4" } else { "Switch to 3x3" }}
                    </button>
                </div>
                <div>
                    <button
                        on:click=move |_| {
                            if using_3x3() {
                                grid3.set(Grid3x3::empty());
                            } else {
                                grid4.set(Grid4x4::empty());
                            }
                            showing_grid.set(false);
                            /*
                            grid.update(|grid| {
                                *grid = Grid3x3::randomized();
                                grid.remove_nums(num_blank());
                                render_table.set(true);
                            });
                            */
                        }
                    >
                        "Restart"
                    </button>
                </div>
                <div>
                    <label for="num-blank-input">"Number of Blanks: "</label>
                    <input
                        node_ref=num_blank_ref
                        type="number" name="num-blank-input" size="5" maxlength="3"
                        min="1" max=move || { if using_3x3() { "80" } else { "255" }} step="1"
                        placeholder="Blanks"
                        prop:value=num_blank
                        on:input=move |ev| {
                            if let Ok(mut num) = event_target_value(&ev).parse() {
                                if using_3x3() && num > 80 {
                                    num = 80;
                                }
                                set_num_blank(num);
                            }
                        }
                        />
                </div>
            </div>

            {
                move || if showing_grid() {
                    if using_3x3() {
                        view! {
                            <Grid3 grid=grid3 />
                        }
                    } else {
                        view! {
                            <Grid4 grid=grid4 />
                        }
                    }
                } else {
                    view! { <Loading /> }
                }
            }

            <div id="bottom-buttons">
                <button
                    on:click=move |_| {
                        /*
                        if let Some(mv) = set_history.try_update(|hist| hist.undo()).flatten() {
                            set_last_mv(Some(mv));
                            grid.update(|grid| grid[mv.pos.0][mv.pos.1] = mv.prev_num);
                            console::log!("{}", mv.prev_num);
                        }
                        */
                    }
                    prop:disabled=move || history.with(|hist| !hist.can_undo())
                >
                    "Undo"
                </button>
                <button
                    on:click=move |_| {
                        notes_active.update(|b| *b = !*b);
                        if let Some(cell) = focused_cell.get().node.get() {
                            cell.focus().expect("error focusing cell");
                        }
                    }
                >
                {move || if notes_active.get() { "Activate Pen" } else { "Activate Pencil" }}
                </button>
                <button
                    on:click=move |_| {
                        /*
                        if let Some(mv) = set_history.try_update(|hist| hist.redo()).flatten() {
                            set_last_mv(Some(mv));
                            grid.update(|grid| grid[mv.pos.0][mv.pos.1] = mv.num);
                            console::log!("{}", mv.num);
                        }
                        */
                    }
                    prop:disabled=move || history.with(|hist| !hist.can_redo())
                >
                    "Redo"
                </button>
            </div>
        </div>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div id="loader"></div> }
}

#[inline(always)]
pub const fn num_to_str(n: u8) -> &'static str {
    match n {
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        10 => "A",
        11 => "B",
        12 => "C",
        13 => "D",
        14 => "E",
        15 => "F",
        16 => "G",
        _ => "",
    }
}

#[derive(Clone, Copy, Default)]
pub struct CellInfo {
    pub node: NodeRef<Div>,
    pub pos: Pos,
}

impl CellInfo {
    pub fn new(node: NodeRef<Div>, pos: Pos) -> Self {
        Self { node, pos }
    }
}

async fn create_grid3x3(num_blank: usize) -> Grid3x3 {
    let mut grid = Grid3x3::randomized();
    grid.remove_nums(num_blank);
    grid.set_given();
    grid
}

async fn create_grid4x4(num_blank: usize) -> Grid4x4 {
    let mut grid = Grid4x4::randomized();
    grid.remove_nums(num_blank);
    grid.set_given();
    grid
}

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}
