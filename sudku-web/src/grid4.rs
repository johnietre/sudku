#![allow(non_snake_case)]

use crate::*;
use sudku_grid::{Grid4x4, History4x4, Move, Num4x4, Pos};

#[component]
pub fn Grid4(grid: RwSignal<Grid4x4>, history: RwSignal<History4x4>) -> impl IntoView {
    provide_context(grid);
    provide_context(history);

    let notes_active = use_context::<RwSignal<bool>>().expect("missing notes_active context");
    let focused_cell = use_context::<RwSignal<CellInfo>>().expect("missing focused_cell context");

    let (completed, set_completed) = create_signal(false);
    create_effect(move |_| {
        if grid.with(Grid4x4::is_valid).is_none() {
            focused_cell.set(CellInfo::default());
            set_completed(true);
        } else {
            set_completed(false);
        }
    });

    view! {
        <div
            id="grid4"
            style:background-color=move || if completed() { "#4fff55" } else { "" }
        >
        {
            move || {
                (0..16).map(|i| view! {<Grid4Box start=(i % 4 * 4, i / 4 * 4) />}).collect_view()
            }
        }
        </div>
        <div id="numbers4-div">
            {
                (0..2).map(|i| {
                    view! {
                        <div>
                        {
                            (1 + 8 * i..=8 * (i + 1)).map(|n| view! {
                                <div
                                    style:color=move || if notes_active.get() { "gray" } else { "blue" }
                                    style:background-color=move || {
                                        if notes_active.get() {
                                            let cell_info = focused_cell.get();
                                            if cell_info.node.get().is_none() {
                                                return "";
                                            };
                                            if grid.with(|grid| grid[cell_info.pos].has_note(n as _))
                                                .unwrap_or(false) {
                                                "blue"
                                            } else {
                                                ""
                                            }
                                        } else {
                                            ""
                                        }
                                    }
                                    on:click=move |_| {
                                        let cell_info = focused_cell.get();
                                        let Some(cell) = cell_info.node.get() else {
                                            // TODO?
                                            return;
                                        };
                                        grid.update(|grid| {
                                            let num = if notes_active.get() {
                                                grid[cell_info.pos].with_toggle_note(n)
                                            } else if grid[cell_info.pos].num_or_zero() == n {
                                                Num4x4::new(0)
                                            } else if !grid.pos_is_valid(cell_info.pos, n) {
                                                return;
                                            } else {
                                                Num4x4::new(n)
                                            };
                                            history.update(|hist| hist.update(
                                                Move::new(grid[cell_info.pos], num, cell_info.pos),
                                            ));
                                            grid[cell_info.pos] = num;
                                        });
                                        cell.focus().expect("error focusing cell");
                                    }
                                    >{num_to_str(n)}</div>
                            }).collect_view()
                        }
                        </div>
                    }
                }).collect_view()
            }
        </div>
    }
}

#[component]
fn Grid4Box(start: Pos) -> impl IntoView {
    let (col, row) = start;
    view! {
        <div class="grid4-box">
        {(0..16)
            .map(|i| view! { <Grid4Cell pos=(col + (i % 4), row + (i / 4)) /> })
            .collect_view()}
        </div>
    }
}

#[component]
fn Grid4Cell(pos: Pos) -> impl IntoView {
    let grid = use_context::<RwSignal<Grid4x4>>().expect("missing grid context");
    let history = use_context::<RwSignal<History4x4>>().expect("missing history context");
    let num = grid.with(|grid| grid[pos]);
    let given = num.is_given();

    let focused_cell = use_context::<RwSignal<CellInfo>>().expect("missing focused_cell context");

    let notes_active = use_context::<RwSignal<bool>>().expect("missing notes_active context");

    let node_ref = create_node_ref();
    let cell_info = CellInfo::new(node_ref, pos);

    let display_cell = move || grid.with(|grid| {
        if let Some(notes) = grid[pos].notes() {
            notes.into_iter()
                .enumerate()
                .map(|(i, b)| view! {
                    <div class="grid4-note-div">{if b { num_to_str(i as u8 + 1) } else { " " }}</div>
                })
                .collect_view()
        } else {
            num_to_str(grid[pos].num_or_zero()).into_view()
        }
    });
    view! {
    <div tabindex={if !given { "0" } else { "" }}
        node_ref=node_ref
        class="grid4-cell"
        class:grid4-zoomed-cell=move || {
            let fc = focused_cell.get();
            fc.node.get().is_some() && fc.pos == cell_info.pos && fc.zoomed
        }
        class:grid4-note-cell=move || grid.with(|grid| grid[pos].is_note())
          class:focused-cell=move || {
            let fc = focused_cell.get();
            fc.node.get().is_some() && fc.pos == cell_info.pos
          }
        on:focusin=move |_| focused_cell.set(cell_info)
        on:click=move |_| {
            if given {
                return;
            }
            node_ref.get().expect("missing node_ref").focus().expect("error focusing");
            //focused_cell.set(cell_info);
            let fc = focused_cell.get();
            if fc.node.get().is_none() || cell_info.pos != fc.pos {
                //focused_cell.set(cell_info);
            } else {
                focused_cell.update(|fc| { fc.zoomed = !fc.zoomed; });
            }
        }
        on:keydown=move |ev| {
            let mut key = ev.key();
            let val = if key == "Backspace" || key == "Delete" {
                0
            } else if key == "Escape" || key == "Tab" {
                return;
            } else {
                key.make_ascii_lowercase();
                match key.chars().next().unwrap_or('\0') {
                    c @ '0'..='9' => c as u8 - b'0',
                    c @ 'a'..='g' => c as u8 - b'a' + 10,
                    _ => {
                        ev.prevent_default();
                        return;
                    }
                }
            };
            grid.update(|grid| {
                let num = if val == 0 {
                    Num4x4::new(0)
                } else if notes_active.get() {
                    grid[pos].with_toggle_note(val)
                } else if !grid.pos_is_valid(pos, val) {
                    ev.prevent_default();
                    return;
                } else {
                    Num4x4::new(val)
                };
                history.update(|hist| hist.update(
                    Move::new(grid[cell_info.pos], num, cell_info.pos),
                ));
                grid[cell_info.pos] = num;
            });
            ev.prevent_default();
        }
        class:given=given
    >
        {display_cell}
    </div>
    }
}
