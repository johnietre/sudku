#![allow(non_snake_case)]

use crate::*;
use sudku_grid::{Grid3x3, MultiHistory3x3 as History3x3, Move3x3 as Move, Num3x3, Pos};

#[component]
pub fn Grid3(
    grid: RwSignal<Grid3x3>, counts: RwSignal<Counts3x3>, history: RwSignal<History3x3>,
) -> impl IntoView {
    provide_context(grid);
    provide_context(counts);
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
                        //style:color=move || if notes_active.get() { "gray" } else { "blue" }
                        style:color=move || {
                            if notes_active.get() {
                                let cell_info = focused_cell.get();
                                if cell_info.node.get().is_none() {
                                    return "gray";
                                };
                                if grid.with(|grid| grid[cell_info.pos].has_note(n as _))
                                    .unwrap_or(false) {
                                    //"blue"
                                    "aqua"
                                } else {
                                    "gray"
                                }
                            } else {
                                if counts.with(|counts| counts[n as usize - 1] < 9) {
                                    "blue"
                                } else {
                                    "gray"
                                }
                            }
                        }
                        on:click=move |_| {
                            let cell_info = focused_cell.get();
                            if grid.with(|grid| grid[cell_info.pos].is_given()) {
                                return;
                            }
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
                                /*
                                history.update(|hist| hist.update(
                                    vec![Move::new(grid[cell_info.pos], num, cell_info.pos)],
                                ));
                                grid[cell_info.pos] = num;
                                */
                                counts.update(|counts| {
                                    if let Some(mvs) = grid_update_rcb(
                                        grid, counts, cell_info.pos, num,
                                    ) {
                                        history.update(|hist| hist.update(mvs));
                                    }
                                });
                            });
                            cell.focus().expect("error focusing cell");
                        }
                        >
                        /*
                        {
                            move || if counts.with(|counts| counts[n as usize - 1] < 9) {
                                num_to_str(n)
                            } else {
                                " "
                            }
                        }
                        */
                        {num_to_str(n)}
                        </div>
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
    let counts = use_context::<RwSignal<Counts3x3>>().expect("missing counts context");
    let history = use_context::<RwSignal<History3x3>>().expect("missing history context");
    let num = grid.with_untracked(|grid| grid[pos]);
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
    //<div tabindex={if !given { "0" } else { "" }}
    <div tabindex="0"
        node_ref=node_ref
        class:secondary-focus=move|| {
            let fc = focused_cell.get();
            if fc.node.get().is_none() {
                return false;
            } else if fc.pos == cell_info.pos {
                return given;
            }
            let (this_num, other_num) = grid.with(|grid| (grid[cell_info.pos], grid[fc.pos]));
            let (tn, on) = (this_num.num_or_zero(), other_num.num_or_zero());
            on != 0 && (tn == on || this_num.has_note(on).unwrap_or(false))
        }
        class="grid3-cell"
        class:grid3-note-cell=move || grid.with(|grid| grid[pos].is_note())
        class:focused-cell=move || {
            let fc = focused_cell.get();
            !given && fc.node.get().is_some() && fc.pos == cell_info.pos
        }
        on:click=move |_| {
            node_ref.get().expect("missing node_ref").focus().expect("error focusing");
        }
        on:focusin=move |_| focused_cell.set(cell_info)
        on:keydown=move |ev| {
            if given {
                return;
            }
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
                let num = if val == 0 || val == grid[pos].num_or_zero() {
                    Num3x3::new(0)
                } else if notes_active.get() {
                    grid[pos].with_toggle_note(val)
                } else if !grid.pos_is_valid(pos, val) {
                    ev.prevent_default();
                    return;
                } else {
                    Num3x3::new(val)
                };
                /*
                history.update(|hist| hist.update(
                    vec![Move::new(grid[cell_info.pos], num, cell_info.pos)],
                ));
                grid[cell_info.pos] = num;
                */
                counts.update(|counts| {
                    if let Some(mvs) = grid_update_rcb(grid, counts, cell_info.pos, num) {
                        history.update(|hist| hist.update(mvs));
                    }
                });
            });
            ev.prevent_default();
        }
        class:given=given
    >
        {display_cell}
    </div>
    }
}

fn grid_update_rcb(
    grid: &mut Grid3x3, counts: &mut Counts3x3, pos: Pos, num: Num3x3,
) -> Option<Vec<Move>> {
    let n = num.num_or_zero();
    if !grid.pos_is_valid(pos, n) {
        return None;
    }
    let mv = Move::new(grid[pos], num, pos);
    let mut mvs = vec![mv];
    if n != 0 {
        counts[n as usize - 1] += 1;
        let (x, y) = pos;
        for x in 0..9 {
            let pos = (x, y);
            if grid[pos].has_note(n).unwrap_or(false) {
                let old = grid[pos];
                let new = grid[pos].with_toggle_note(n);
                mvs.push(Move::new(old, new, pos));
                grid[pos] = new;
            }
        }
        for y in 0..9 {
            let pos = (x, y);
            if grid[pos].has_note(n).unwrap_or(false) {
                let old = grid[pos];
                let new = grid[pos].with_toggle_note(n);
                mvs.push(Move::new(old, new, pos));
                grid[pos] = new;
            }
        }
        for y in y / 3 * 3..y / 3 * 3 + 3 {
            for x in x / 3 * 3..x / 3 * 3 + 3 {
                let pos = (x, y);
                if grid[pos].has_note(n).unwrap_or(false) {
                    let old = grid[pos];
                    let new = grid[pos].with_toggle_note(n);
                    mvs.push(Move::new(old, new, pos));
                    grid[pos] = new;
                }
            }
        }
    }
    let old = grid[pos].num_or_zero() as usize;
    if old != 0 {
        counts[old - 1] -= 1;
    }
    grid[pos] = num;
    Some(mvs)
}
