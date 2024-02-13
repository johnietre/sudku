#![allow(non_snake_case)]

//use base64::engine::{general_purpose::STANDARD as B64_ENG, Engine};
use leptos::*;
use leptos::html::Div;
use serde::{Deserialize, Serialize};
use sudku_grid::{Grid3x3, Grid4x4, History3x3, History4x4, Pos};
use wasm_bindgen::prelude::*;

mod base64;
pub mod console;
mod grid3;
use grid3::*;
mod grid4;
use grid4::*;
//mod settings;
//use settings::*;

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

    let grid3 = create_rw_signal(Grid3x3::empty());
    let grid4 = create_rw_signal(Grid4x4::empty());
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
            match storage.get_item("sudku-grid3") {
                Ok(Some(enc)) => match base64::decode(enc).map(Grid3x3::from_encoded) {
                    Some(Some(grid)) => grid3.set(grid),
                    Some(None) => console::log!("bad grid encoding"),
                    None => console::log!("bad grid3 base64 encoding"),
                }
                Ok(None) => (),
                // TODO: Print error better
                Err(e) => console::log!("error getting grid3 from local storage: {e:?}"),
            }
            match storage.get_item("sudku-grid4") {
                Ok(Some(enc)) => match base64::decode(enc).map(Grid4x4::from_encoded) {
                    Some(Some(grid)) => grid4.set(grid),
                    Some(None) => console::log!("bad grid encoding"),
                    None => console::log!("bad grid4 base64 encoding"),
                }
                Ok(None) => console::log!("no grid4"),
                // TODO: Print error better
                Err(e) => console::log!("error getting grid4 from local storage: {e:?}"),
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
        spawn_local(async move {
            if settings.with(|s| s.using_3x3) {
                if grid3.with(|grid| grid == &Grid3x3::EMPTY) {
                    grid3.set(create_grid3x3(settings.with(|s| s.num_blanks3)).await);
                }
            } else {
                if grid4.with(|grid| grid == &Grid4x4::EMPTY) {
                    grid4.set(create_grid4x4(settings.with(|s| s.num_blanks4)).await);
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
            let Some(storage) = get_local_storage() else {
                return;
            };
            storage
                .set_item("sudku-grid3", &base64::encode(grid.encode()))
                .expect("error saving grid3");
        });
    });
    create_effect(move |_| {
        grid4.with(|grid| {
            let Some(storage) = get_local_storage() else {
                return;
            };
            storage
                .set_item("sudku-grid4", &base64::encode(grid.encode()))
                .expect("error saving grid4");
        });
    });

    view! {
        <div id="app">
            <div id="main">
                {
                    move || if showing_grid() {
                        if settings.with(|s| s.using_3x3) {
                            view! {
                                <Grid3 grid=grid3 history=history3 />
                            }
                        } else {
                            view! {
                                <Grid4 grid=grid4 history=history4 />
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
                                        let Some(mv) = history3.try_update(|hist| hist.undo().copied())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid3.update(|grid| grid[mv.pos] = mv.old);
                                    } else {
                                        let Some(mv) = history4.try_update(|hist| hist.undo().copied())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid4.update(|grid| grid[mv.pos] = mv.old);
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
                                        let Some(mv) = history3.try_update(|hist| hist.redo().copied())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid3.update(|grid| grid[mv.pos] = mv.new);
                                    } else {
                                        let Some(mv) = history4.try_update(|hist| hist.redo().copied())
                                            .expect("bad history try_update") else {
                                            // TODO: Disable?
                                            return;
                                        };
                                        grid4.update(|grid| grid[mv.pos] = mv.new);
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
                        "Number of Blanks (3x3): "
                        <input
                            type="number" name="num-blank-input" size="5" maxlength="3"
                            min="1" max="80" step="1"
                            placeholder="Blanks"
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
                        "Number of Blanks (4x4): "
                        <input
                            type="number" name="num-blank-input" size="5" maxlength="3"
                            min="1" max="255" step="1"
                            placeholder="Blanks"
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
                        "4x4 Grid:"
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

fn get_local_storage() -> Option<web_sys::Storage> {
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
