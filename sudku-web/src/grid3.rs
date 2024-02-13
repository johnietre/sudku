#![allow(non_snake_case)]

use crate::*;
use sudku_grid::{Grid3x3, History3x3, Move, Num3x3, Pos};

#[component]
pub fn Grid3(grid: RwSignal<Grid3x3>, history: RwSignal<History3x3>) -> impl IntoView {
    provide_context(grid);
    provide_context(history);

    let notes_active = use_context::<RwSignal<bool>>().expect("missing notes_active context");
    let focused_cell = use_context::<RwSignal<CellInfo>>().expect("missing focused_cell context");

    let (completed, set_completed) = create_signal(false);
    create_effect(move |_| {
        if grid.with(Grid3x3::is_valid).is_none() {
            focused_cell.set(CellInfo::default());
            set_completed(true);
        } else {
            set_completed(false);
        }
    });

    view! {
        <div
            id="grid3"
            style:background-color=move || if completed() { "#4fff55" } else { "" }
        >
        {
            move || {
                (0..9).map(|i| view! {<Grid3Box start=(i % 3 * 3, i / 3 * 3) />}).collect_view()
            }
        }
        </div>
        <div id="numbers3-div">
            {
                (1..=9).map(|n| view! {
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
                                    Num3x3::new(0)
                                } else if !grid.pos_is_valid(cell_info.pos, n) {
                                    return;
                                } else {
                                    Num3x3::new(n)
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
}

#[component]
fn Grid3Box(start: Pos) -> impl IntoView {
    let (col, row) = start;
    view! {
        <div class="grid3-box">
        {(0..9)
            .map(|i| view! { <Grid3Cell pos=(col + (i % 3), row + (i / 3)) /> })
            .collect_view()}
        </div>
    }
}

#[component]
fn Grid3Cell(pos: Pos) -> impl IntoView {
    let grid = use_context::<RwSignal<Grid3x3>>().expect("missing grid context");
    let history = use_context::<RwSignal<History3x3>>().expect("missing history context");
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
                    <div class="grid3-note-div">{if b { num_to_str(i as u8 + 1) } else { " " }}</div>
                })
                .collect_view()
        } else {
            num_to_str(grid[pos].num_or_zero()).into_view()
        }
    });
    view! {
    <div tabindex={if !given { "0" } else { "" }}
        node_ref=node_ref
        class="grid3-cell"
        class:grid3-note-cell=move || grid.with(|grid| grid[pos].is_note())
        class:focused-cell=move || {
            let fc = focused_cell.get();
            fc.node.get().is_some() && fc.pos == cell_info.pos
        }
        on:click=move |_| {
            if !given {
                node_ref.get().expect("missing node_ref").focus().expect("error focusing");
            }
        }
        on:focusin=move |_| focused_cell.set(cell_info)
        on:keydown=move |ev| {
            let key = ev.key();
            let val = if key == "Backspace" || key == "Delete" {
                0
            } else if key == "Escape" || key == "Tab" {
                return;
            } else {
                let n = key.chars().next().unwrap_or('\0');
                if !n.is_ascii_digit() {
                    ev.prevent_default();
                    return;
                }
                n as u8 - b'0'
            };
            grid.update(|grid| {
                let num = if val == 0 {
                    Num3x3::new(0)
                } else if notes_active.get() {
                    grid[pos].with_toggle_note(val)
                } else if !grid.pos_is_valid(pos, val) {
                    ev.prevent_default();
                    return;
                } else {
                    Num3x3::new(val)
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
