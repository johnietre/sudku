#![allow(non_snake_case)]

use crate::*;
use sudku_grid::complex::{Grid3x3, Num3x3, Pos};

#[component]
pub fn Grid3(grid: RwSignal<Grid3x3>) -> impl IntoView {
    //let num_blank = use_context::<ReadSignal<usize>>().expect("missing num_blank context");
    //let mut grid = Grid3x3::randomized();
    //grid.remove_nums(num_blank());
    //let grid = create_rw_signal(grid);
    provide_context(grid);

    let notes_active = use_context::<RwSignal<bool>>().expect("missing notes_active context");
    let focused_cell = use_context::<RwSignal<CellInfo>>().expect("missing focused_cell context");

    create_effect(move |_| {
        //log!("{:?}", last_mv());
        /*
        if let Some(_mv) = last_mv() {
            //let (x, y) = mv.pos;
            //log!("{}", grid.with(|grid| grid[y][x]));
        }
        */
    });
    let render_table = create_rw_signal(true);
    view! {
        <div id="grid3">
        {
            move || {
                if render_table.get() {
                    render_table.set(false);
                }
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
                            //let cell = cell_info.node.get().expect("missing node_ref");
                            let Some(cell) = cell_info.node.get() else {
                                // TODO?
                                return;
                            };
                            grid.update(|grid| {
                                if notes_active.get() {
                                    grid[cell_info.pos] = grid[cell_info.pos].with_toggle_note(n);
                                    return;
                                }
                                if !grid.pos_is_valid(cell_info.pos, n) {
                                }
                                grid[cell_info.pos] = Num3x3::new(n);
                                if grid.is_valid().is_none() {
                                    alert("Completed!");
                                }
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
    let num = grid.with(|grid| grid[pos]);
    let given = num.is_given();

    let focused_cell = use_context::<RwSignal<CellInfo>>().expect("missing focused_cell context");

    let notes_active = use_context::<RwSignal<bool>>().expect("missing notes_active context");

    let node_ref = create_node_ref();
    let cell_info = CellInfo::new(node_ref, pos);
    /*
    grid.update(|grid| {
        if num == 0 {
            grid[pos] = Num3x3::new(0).with_notes([true; 9]);
        }
    });
    */
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
        class="grid3-cell"
        class:grid3-note-cell=move || grid.with(|grid| grid[pos].is_note())
        node_ref=node_ref
        on:click=move |_| {
            if !given {
                node_ref.get().expect("missing node_ref").focus().expect("error focusing");
                focused_cell.set(cell_info);
            }
        }
        on:keydown=move |ev| {
            let key = ev.key();
            if key == "Backspace" || key == "Delete" {
                grid.update(|grid| {
                    grid[pos] = Num3x3::new(0);
                });
                return;
            } else if key == "Escape" || key == "Tab" {
                return;
            }
            let num = key.chars().next().unwrap_or('\0');
            if !num.is_ascii_digit() {
                ev.prevent_default();
                return;
            }
            let val = num as u8 - b'0';
            grid.update(|grid| {
                if notes_active.get() {
                    grid[pos] = grid[pos].with_toggle_note(val);
                    ev.prevent_default();
                    return;
                }
                let valid = grid.pos_is_valid(pos, val);
                if !valid {
                    ev.prevent_default();
                    return;
                }
                /*
                set_history.update(|hist| {
                    hist.push(Move::new(pos, val, grid[pos]));
                }); 
                */
                grid[pos] = Num3x3::new(val);
                if grid.is_valid().is_none() {
                    alert("Completed!");
                }
            });
            ev.prevent_default();
        }
        class:given=given
    >
        {display_cell}
    </div>
    }
}
