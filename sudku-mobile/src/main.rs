#![allow(non_snake_case)]

use leptos::*;
use leptos::html::Div;
use serde::{Deserialize, Serialize};
use sudku_grid::{Grid3x3, Grid4x4, MultiHistory3x3 as History3x3, MultiHistory4x4 as History4x4, Pos};
use wasm_bindgen::prelude::*;
use web_sys::Storage;

fn main() {
    run().expect("error running")
}

mod base64;
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
    let (settings, set_settings) = create_signal(Settings {
        num_blanks3: 18,
        num_blanks4: 18,
        using_3x3: true,
    });

    let focused_cell = create_rw_signal(CellInfo::default());
    provide_context(focused_cell);

    let notes_active = create_rw_signal(false);
    provide_context(notes_active);

    let seed3 = create_rw_signal(Grid3x3::empty());
    let seed4 = create_rw_signal(Grid4x4::empty());
    let grid3 = create_rw_signal(Grid3x3::empty());
    let grid4 = create_rw_signal(Grid4x4::empty());
    let counts3 = create_rw_signal([0usize; 9]);
    let counts4 = create_rw_signal([0usize; 16]);
    let history3 = create_rw_signal(History3x3::new());
    let history4 = create_rw_signal(History4x4::new());
    let showing_grid = create_rw_signal(false);

    {
        if let Some(storage) = get_local_storage() {
            match storage.get_item("sudku-settings") {
                Ok(Some(json)) => match serde_json::from_str(&json) {
                    Ok(settings) => set_settings(settings),
                    // TODO: Print error better
                    Err(e) => console::log!("bad settings json: {e:?}"),
                }
                Ok(None) => (),
                // TODO: Print error better
                Err(e) => console::log!("error getting settings from local storage: {e:?}"),
            }
            if let Some(grid) = load_grid3(Some(&storage), "sudku-grid3") {
                grid3.set(grid);
                let mut counts = [0usize; 9];
                for y in 0..9 {
                    for x in 0..9 {
                        let n = grid[(x, y)].num_or_zero() as usize;
                        if n != 0 {
                            counts[n - 1] += 1;
                        }
                    }
                }
                counts3.set(counts);
            }
            if let Some(grid) = load_grid4(Some(&storage), "sudku-grid4") {
                grid4.set(grid);
                for y in 0..16 {
                    for x in 0..16 {
                        let n = grid[(x, y)].num_or_zero() as usize;
                        if n != 0 {
                            counts[n - 1] += 1;
                        }
                    }
                }
                counts4.set(counts);
            }
            if let Some(seed) = load_grid3(Some(&storage), "sudku-seed3") {
                seed3.set(seed);
            }
            if let Some(seed) = load_grid4(Some(&storage), "sudku-seed4") {
                seed4.set(seed);
            }
        }
    }

    let (showing_settings, set_showing_settings) = create_signal(false);
    let new_settings = create_rw_signal(settings());

    create_effect(move |_| {
        if showing_settings() {
            new_settings.set(settings());
        }
    });

    create_effect(move |_| {
        if showing_grid.get() {
            return;
        }
        let using_3x3 = settings.with(|s| s.using_3x3);
        let is_empty = if using_3x3 {
            grid3.with(|grid| grid == &Grid3x3::EMPTY)
        } else {
            grid4.with(|grid| grid == &Grid4x4::EMPTY)
        };
        spawn_local(async move {
            if using_3x3 {
                if is_empty {
                    if decr_gtns(None, "sudku-gtns3", 5) {
                        let seed = Grid3x3::generate();
                        save_grid3(None, "sudku-seed3", &seed);
                        seed3.set(seed);
                    }
                    seed3.with(|seed| {
                        let (grid, counts) = grid3x3_from_seed(seed, settings.with(|s| s.num_blanks3));
                        grid3.set(grid);
                        counts3.set(counts);
                    });
                }
            } else {
                if is_empty {
                    if decr_gtns(None, "sudku-gtns4", 5) {
                        let seed = Grid4x4::generate();
                        save_grid4(None, "sudku-seed4", &seed);
                        seed4.set(seed);
                    }
                    seed4.with(|seed| {
                        let (grid, counts) = grid4x4_from_seed(seed, settings.with(|s| s.num_blanks4));
                        grid4.set(grid);
                        counts4.set(counts);
                    });
                }
            }
            showing_grid.set(true);
        });
    });
    create_effect(move |_| {
        settings.with(|s| {
            let Some(storage) = get_local_storage() else {
                return;
            };
            storage
                .set_item(
                    "sudku-settings",
                    &serde_json::to_string(s).expect("error serializing settings"),
                )
                .expect("error saving settings to local storage");
        });
    });
    create_effect(move |_| {
        grid3.with(|grid| {
            save_grid3(None, "sudku-grid3", grid);
        });
    });
    create_effect(move |_| {
        grid4.with(|grid| {
            save_grid4(None, "sudku-grid4", grid);
        });
    });

    view! {
        <div id="app">
            <div id="main">
                {
                    move || if showing_grid() {
                        if settings.with(|s| s.using_3x3) {
                            view! {
                                <Grid3 grid=grid3 counts=counts3 history=history3 />
                            }
                        } else {
                            view! {
                                <Grid4 grid=grid4 counts=counts4 history=history4 />
                            }
                        }
                    } else {
                        view! { <Loading /> }
                    }
                }

                <div id="bottom-buttons">
                </div>

            </div>

            <div id="side-buttons">
                <div>
                    <button
                        on:click=move |_| {
                            focused_cell.set(CellInfo::default());
                            let b = showing_settings();
                            set_showing_settings(!b);
                        }
                    >
                    {
                        move || if showing_settings() {
                            view! { <img src={BLUE_SETTINGS_IMG_SRC} /> }
                        } else {
                            view! { <img src={SETTINGS_IMG_SRC} /> }
                        }
                    }
                    </button>
                </div>
                <div>
                {
                    move || {
                        let disabled = if settings.with(|s| s.using_3x3) {
                            history3.with(|hist| !hist.can_undo())
                        } else {
                            history4.with(|hist| !hist.can_undo())
                        };
                        if !disabled {
                            view! { <button
                                //prop:disabled=disabled
                                on:click=move |_| {
                                    if settings.with(|s| s.using_3x3) {
                                        let Some(mvs) = history3.try_update(|hist| hist.undo().cloned())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid3.update(|grid| {
                                            counts3.update(|counts| {
                                                for mv in mvs {
                                                    let nn = mv.new.num_or_zero();
                                                    if nn != 0 {
                                                        counts[nn as usize - 1] -= 1;
                                                    }
                                                    let on = mv.old.num_or_zero();
                                                    if on != 0 {
                                                        counts[on as usize - 1] += 1;
                                                    }
                                                    grid[mv.pos] = mv.old;
                                                }
                                            });
                                        });
                                    } else {
                                        let Some(mvs) = history4.try_update(|hist| hist.undo().cloned())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid4.update(|grid| {
                                            counts4.update(|counts| {
                                                for mv in mvs {
                                                    let nn = mv.new.num_or_zero();
                                                    if nn != 0 {
                                                        counts[nn as usize - 1] -= 1;
                                                    }
                                                    let on = mv.old.num_or_zero();
                                                    if on != 0 {
                                                        counts[on as usize - 1] += 1;
                                                    }
                                                    grid[mv.pos] = mv.old;
                                                }
                                            });
                                        });
                                    }
                                }
                            >
                                <img src={UNDO_IMG_SRC} />
                            </button> }.into_any()
                        } else {
                            //view! { <div></div> }.into_any()
                            view! { <button disabled=true><div></div></button> }.into_any()
                        }
                    }
                }
                </div>
                <div>
                {
                    move || {
                        let disabled= if settings.with(|s| s.using_3x3) {
                            history3.with(|hist| !hist.can_redo())
                        } else {
                            history4.with(|hist| !hist.can_redo())
                        };
                        if !disabled {
                            view! { <button
                                //prop:disabled=disabled
                                on:click=move |_| {
                                    if settings.with(|s| s.using_3x3) {
                                        let Some(mvs) = history3.try_update(|hist| hist.redo().cloned())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid3.update(|grid| {
                                            counts3.update(|counts| {
                                                for mv in mvs {
                                                    let nn = mv.new.num_or_zero();
                                                    if nn != 0 {
                                                        counts[nn as usize - 1] += 1;
                                                    }
                                                    let on = mv.old.num_or_zero();
                                                    if on != 0 {
                                                        counts[on as usize - 1] -= 1;
                                                    }
                                                    grid[mv.pos] = mv.new;
                                                }
                                            });
                                        });
                                    } else {
                                        let Some(mvs) = history4.try_update(|hist| hist.redo().cloned())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid4.update(|grid| {
                                            counts4.update(|counts| {
                                                for mv in mvs {
                                                    let nn = mv.new.num_or_zero();
                                                    if nn != 0 {
                                                        counts[nn as usize - 1] += 1;
                                                    }
                                                    let on = mv.old.num_or_zero();
                                                    if on != 0 {
                                                        counts[on as usize - 1] -= 1;
                                                    }
                                                    grid[mv.pos] = mv.new;
                                                }
                                            });
                                        });
                                    }
                                }
                            >
                                <img src={REDO_IMG_SRC} />
                            </button>}.into_any()
                        } else {
                            view! { <button disabled=true><div></div></button> }.into_any()
                        }
                    }
                }
                </div>
                <div>
                    <button
                        on:click=move |_| {
                            notes_active.update(|b| *b = !*b);
                            if let Some(cell) = focused_cell.get().node.get() {
                                cell.focus().expect("error focusing cell");
                            }
                        }
                    >
                    {
                        move || if notes_active.get() {
                            view! { <img src={BLUE_PENCIL_IMG_SRC} /> }
                        } else {
                            view! { <img src={PENCIL_IMG_SRC} /> }
                        }
                    }
                    </button>
                </div>
            </div>

            <div
                id="settings"
                style:display=move || if showing_settings() { "block" } else { "none" }
            >
                <div id="settings-opts">
                    <label>
                        "Number of Blanks (9x9): "
                        <input
                            type="number" name="num-blank-input" size="5" maxlength="3"
                            min="1" max="80" step="1" pattern="[0-9]*"
                            placeholder="Blanks" inputmode="numeric"
                            prop:value=move || new_settings.with(|s| s.num_blanks3)
                            on:input=move |ev| {
                                event_target_value(&ev)
                                    .parse()
                                    .into_iter()
                                    .for_each(|num: usize| {
                                        new_settings.update(|s| s.num_blanks3 = num.min(80))
                                    });
                            }
                        />
                    </label>
                    <br />
                    <label>
                        "Number of Blanks (16x16): "
                        <input
                            type="number" name="num-blank-input" size="5" maxlength="3"
                            min="1" max="255" step="1" pattern="[0-9]*"
                            placeholder="Blanks" inputmode="numeric"
                            prop:value=move || new_settings.with(|s| s.num_blanks4)
                            on:input=move |ev| {
                                event_target_value(&ev)
                                    .parse()
                                    .into_iter()
                                    .for_each(|num: usize| {
                                        new_settings.update(|s| s.num_blanks4 = num.min(255))
                                    });
                            }
                        />
                    </label>

                    <hr />

                    <label for="use-4x4">
                        "16x16 Grid:"
                        <input
                            type="checkbox" name="use-4x4"
                            prop:checked=move || new_settings.with(|s| !s.using_3x3)
                            on:click=move |ev| {
                                if event_target_checked(&ev) {
                                    new_settings.update(|s| s.using_3x3 = false);
                                } else {
                                    new_settings.update(|s| s.using_3x3 = true);
                                }
                            }
                        />
                    </label>
                </div>
                <div id="settings-bottom-buttons">
                    <div>
                        <button
                            on:click=move |_| {
                                focused_cell.set(CellInfo::default());
                                if new_settings.with(|s| s.using_3x3) {
                                    grid3.set(Grid3x3::empty());
                                    history3.set(History3x3::new());
                                } else {
                                    grid4.set(Grid4x4::empty());
                                    history4.set(History4x4::new());
                                }
                                set_showing_settings(false);
                                showing_grid.set(false);
                                set_settings(new_settings.get());
                            }
                        >"New Game"</button>
                    </div>
                    <div>
                        <button
                            on:click=move |_| {
                                set_showing_settings(false);
                            }
                        >"Cancel"</button>
                    </div>
                    <div>
                        <button
                            on:click=move |_| {
                                if new_settings.with(|s| s.using_3x3) {
                                    if !settings.with(|s| s.using_3x3) {
                                        showing_grid.set(false);
                                    }
                                    //grid3.set(Grid3x3::empty());
                                    //history3.set(History3x3::new());
                                } else {
                                    if settings.with(|s| s.using_3x3) {
                                        showing_grid.set(false);
                                    }
                                    //grid4.set(Grid4x4::empty());
                                    //history4.set(History4x4::new());
                                }
                                set_showing_settings(false);
                                set_settings(new_settings.get());
                            }
                        >"Ok"</button>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! { <div id="loader"></div> }
}

#[Component]
fn About(showing_about: RwSignal<bool>) -> impl IntoView {
    view! {
        <div id="about">
            <div id="about-the-app">
                <h3>"About The App"</h3>
                <p>"A simple Sudoku app that, as of right now, neither looks nor functions spectacularly."</p>
                <p>"One thing to take special note of is that generated Sudoku boards are not guaranteed to be well-formed, that is, it is not guaranteed that a generated with a given number of blanks will have only one solution."
                <p>"Feedback is appreciated, but do not harass me. :)"</p>
                </p>
            </div>
            <div id="how-to-play">
                <h3>"How To Play"</h3>
                <p>"A very simple explanation of how to play is as follows."</p>
                <p>"The goal is to fill all empty cells on the board. Each cell must have a number that is unique within its row, column, and enclosing 3x3 box. If you attempt to enter a number into a cell but it does not work, the most likely reason is that the number entered violates this uniqueness rule."</p>
                <p>"If you are stuck in a situation where you feel it could be a toss-up as to what number goes in a cell, it is possible that the generated board is not well-formed and there are multiple solutions. This is usually discovered when choosing between the same pair of numbers for multiple cells. In this case, either number will likely work. If not, Undo is your friend. :)"</p>
                <p>"Notes can be placed in cells by activating the pencil icon by the number input."</p>
                <p>"Evertying is the same when playing with the 16x16 board."</p>
                <p>"If you are still confused, go to the Internet. ;]"</p>
            </div>
            <button on:click=move |_| showing_about.set(false)>"Close"</button>
        </div>
    }
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
    pub zoomed: bool,
}

impl CellInfo {
    pub fn new(node: NodeRef<Div>, pos: Pos) -> Self {
        Self { node, pos, zoomed: false }
    }
}

fn load_grid3(storage: Option<&Storage>, name: &str) -> Option<Grid3x3> {
    let mut stor = None;
    let Some(storage) = storage.or_else(|| {
        stor = get_local_storage();
        stor.as_ref()
    }) else {
        console::log!("could not get local storage");
        return None;
    };
    match storage.get_item(name) {
        Ok(Some(enc)) => match base64::decode(enc).map(Grid3x3::from_encoded) {
            Some(Some(grid)) => Some(grid),
            Some(None) => {
                console::log!("bad grid3 encoding for {name}");
                None
            }
            None => {
                console::log!("bad grid3 base64 encoding for {name}");
                None
            }
        }
        Ok(None) => None,
        // TODO: Print error better
        Err(e) => {
            console::log!("error getting {name} from local storage: {e:?}");
            None
        }
    }
}

fn load_grid4(storage: Option<&Storage>, name: &str) -> Option<Grid4x4> {
    let mut stor = None;
    let Some(storage) = storage.or_else(|| {
        stor = get_local_storage();
        stor.as_ref()
    }) else {
        console::log!("could not get local storage");
        return None;
    };
    match storage.get_item(name) {
        Ok(Some(enc)) => match base64::decode(enc).map(Grid4x4::from_encoded) {
            Some(Some(grid)) => Some(grid),
            Some(None) => {
                console::log!("bad grid4 encoding for {name}");
                None
            }
            None => {
                console::log!("bad grid4 base64 encoding for {name}");
                None
            }
        }
        Ok(None) => None,
        // TODO: Print error better
        Err(e) => {
            console::log!("error getting {name} from local storage: {e:?}");
            None
        }
    }
}

// TODO: What to return
fn save_grid3(storage: Option<&Storage>, name: &str, grid: &Grid3x3) -> bool {
    let mut stor = None;
    let Some(storage) = storage.or_else(|| {
        stor = get_local_storage();
        stor.as_ref()
    }) else {
        console::log!("could not get local storage");
        return false;
    };
    // TODO: What to do
    /*
    storage
        .set_item(name, &base64::encode(grid.encode()))
        .expect("error saving {name}");
    */
    if let Err(e) = storage.set_item(name, &base64::encode(grid.encode())) {
        console::log!("error saving {name}: {e:?}");
        return false;
    }
    true
}

// TODO: What to return
fn save_grid4(storage: Option<&Storage>, name: &str, grid: &Grid4x4) -> bool {
    let mut stor = None;
    let Some(storage) = storage.or_else(|| {
        stor = get_local_storage();
        stor.as_ref()
    }) else {
        console::log!("could not get local storage");
        return false;
    };
    // TODO: What to do
    /*
    storage
        .set_item(name, &base64::encode(grid.encode()))
        .expect("error saving {name}");
    */
    if let Err(e) = storage.set_item(name, &base64::encode(grid.encode())) {
        console::log!("error saving {name}: {e:?}");
        return false;
    }
    true
}

/// Decrements the count for number of games until a new seed should be generated (gtns = games til
/// new seed). If the count is 0 or was not set, it is reset with the provided value.
/// Returns true if the count was reset. Returns true if there is any kind of error getting the
/// count.
fn decr_gtns(storage: Option<&Storage>, name: &str, reset_val: usize) -> bool {
    let mut stor = None;
    let Some(storage) = storage.or_else(|| {
        stor = get_local_storage();
        stor.as_ref()
    }) else {
        console::log!("could not get local storage");
        return true;
    };
    match storage.get_item(name) {
        Ok(Some(num_str)) => {
            let num = if let Ok(num) = num_str.parse::<usize>() {
                if num == 0 { reset_val } else { num - 1 }
            } else {
                reset_val
            };
            // TODO: Handle differently?
            if let Err(e) = storage.set_item(name, &num.to_string()) {
                console::log!("error saving {name}: {e:?}");
            }
            num == reset_val
        }
        Ok(None) => {
            if let Err(e) = storage.set_item(name, &reset_val.to_string()) {
                console::log!("error saving {name}: {e:?}");
            }
            true
        }
        // TODO: Print error better
        Err(e) => {
            console::log!("error getting {name} from local storage: {e:?}");
            true
        }
    }
}

pub type Counts3x3 = [usize; 9];
pub type Counts4x4 = [usize; 16];

fn grid3x3_from_seed(seed: &Grid3x3, num_blank: usize) -> (Grid3x3, Counts3x3) {
    let mut grid = seed.clone();
    grid.randomize();
    grid.remove_nums(num_blank);
    grid.set_given();
    let mut counts = [0usize; 9];
    for y in 0..9 {
        for x in 0..9 {
            let n = grid[(x, y)].num_or_zero() as usize;
            if n != 0 {
                counts[n - 1] += 1;
            }
        }
    }
    (grid, counts)
}

fn grid4x4_from_seed(seed: &Grid4x4, num_blank: usize) -> (Grid4x4, Counts4x4) {
    let mut grid = seed.clone();
    grid.randomize();
    grid.remove_nums(num_blank);
    grid.set_given();
    let mut counts = [0usize; 16];
    for y in 0..16 {
        for x in 0..16 {
            let n = grid[(x, y)].num_or_zero() as usize;
            if n != 0 {
                counts[n - 1] += 1;
            }
        }
    }
    (grid, counts)
}

fn get_local_storage() -> Option<Storage> {
    match window().local_storage() {
        Ok(Some(stor)) => Some(stor),
        Ok(None) => {
            console::log!("missing local storage");
            None
        }
        Err(e) => {
            // TODO: Print better
            console::log!("error getting local storage: {e:?}");
            None
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub using_3x3: bool,
    pub num_blanks3: usize,
    pub num_blanks4: usize,
}

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

const SETTINGS_IMG_SRC: &str = concat!(
    "data:image/png;base64,", include_str!("../assets/settings.png.base64")
);
const BLUE_SETTINGS_IMG_SRC: &str = concat!(
    "data:image/png;base64,", include_str!("../assets/settings-blue.png.base64")
);
const UNDO_IMG_SRC: &str = concat!(
    "data:image/png;base64,", include_str!("../assets/undo.png.base64")
);
const REDO_IMG_SRC: &str = concat!(
    "data:image/png;base64,", include_str!("../assets/redo.png.base64")
);
const PENCIL_IMG_SRC: &str = concat!(
    "data:image/png;base64,", include_str!("../assets/pencil.png.base64")
);
const BLUE_PENCIL_IMG_SRC: &str = concat!(
    "data:image/png;base64,", include_str!("../assets/pencil-blue.png.base64")
);
