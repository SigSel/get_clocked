use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::SignalExt;
use futures_signals::signal_vec::SignalVecExt;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::HtmlInputElement;

use crate::app::{AppPage, AppState, DraftCategory, WorkdayState, WorkEntry};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportArgs {
    folder: String,
    format: String,
    date: String,
    entries: Vec<WorkEntry>,
}

pub fn render(state: Arc<AppState>) -> Dom {
    let wd = state.workday.clone();
    html!("div", {
        .dwclass!("relative w-full h-screen bg-gray-900 flex flex-col")
        .style("color", "white")
        .child(render_header(state.clone()))
        .child(html!("div", {
            .dwclass!("flex flex-col items-center")
            .style("overflow-y", "auto")
            .style("flex", "1")
            .style("padding", "16px 0")
            .child(html!("div", {
                .dwclass!("flex flex-col gap-4")
                .style("width", "520px")
                .child(render_date_section(wd.clone()))
                .child(render_entries_list(wd.clone()))
                .child(render_add_entry_button(wd.clone()))
                .child(render_draft_form(wd.clone()))
                .child(render_action_bar(state.clone()))
            }))
        }))
    })
}

fn render_header(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("flex items-center gap-4 p-4")
        .style("border-bottom", "1px solid #374151")
        .child(html!("button", {
            .style("background", "none")
            .style("border", "none")
            .style("color", "#d1d5db")
            .style("cursor", "pointer")
            .style("font-size", "14px")
            .text("← Back")
            .event(clone!(state => move |_: events::Click| {
                state.workday.reset();
                state.page.set(AppPage::Home);
            }))
        }))
        .child(html!("h2", {
            .dwclass!("text-lg font-semibold")
            .text("Register Workday")
        }))
    })
}

fn render_date_section(wd: Arc<WorkdayState>) -> Dom {
    html!("div", {
        .dwclass!("flex items-center gap-4")
        .child(html!("label", {
            .style("color", "#d1d5db")
            .style("font-size", "14px")
            .text("Date")
        }))
        .child(html!("input" => HtmlInputElement, {
            .attr("type", "date")
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "6px 10px")
            .style("font-size", "14px")
            .prop_signal("value", wd.date.signal_cloned())
            .with_node!(el => {
                .event(clone!(wd => move |_: events::Input| {
                    wd.date.set(el.value());
                }))
            })
        }))
    })
}

fn render_entries_list(wd: Arc<WorkdayState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .child(html!("div", {
            .dwclass!("flex items-center gap-2")
            .style("font-size", "13px")
            .child(html!("span", {
                .style("color", "#9ca3af")
                .text("Total hours:")
            }))
            .child(html!("span", {
                .style("color", "#a3e635")
                .style("font-weight", "600")
                .text_signal(wd.total_hours.signal_ref(|h| format!("{:.1}", h)))
            }))
        }))
        .children_signal_vec(wd.entries.signal_vec_cloned().map(|entry| {
            html!("div", {
                .dwclass!("flex items-center gap-4")
                .style("background", "#1f2937")
                .style("border-radius", "4px")
                .style("padding", "8px 12px")
                .style("font-size", "13px")
                .child(html!("span", {
                    .style("color", "#a3e635")
                    .style("font-weight", "600")
                    .text(&format!("{:.1} h", entry.hours))
                }))
                .children(entry.categories.iter().map(|(k, v)| {
                    html!("span", {
                        .style("color", "#d1d5db")
                        .text(&format!("{}: {}", k, v))
                    })
                }))
            })
        }))
    })
}

fn render_add_entry_button(wd: Arc<WorkdayState>) -> Dom {
    html!("div", {
        .child_signal(wd.draft_visible.signal().map(clone!(wd => move |visible| {
            if visible {
                None
            } else {
                Some(html!("button", {
                    .style("background", "#2563eb")
                    .style("color", "white")
                    .style("border", "none")
                    .style("border-radius", "4px")
                    .style("padding", "6px 14px")
                    .style("cursor", "pointer")
                    .style("font-size", "13px")
                    .text("+ Add Entry")
                    .event(clone!(wd => move |_: events::Click| {
                        wd.draft_visible.set(true);
                    }))
                }))
            }
        })))
    })
}

fn render_draft_form(wd: Arc<WorkdayState>) -> Dom {
    html!("div", {
        .visible_signal(wd.draft_visible.signal())
        .dwclass!("flex flex-col gap-4")
        .style("border", "1px solid #374151")
        .style("border-radius", "6px")
        .style("padding", "16px")

        // Hours row
        .child(html!("div", {
            .dwclass!("flex items-center gap-4")
            .child(html!("label", {
                .style("color", "#d1d5db")
                .style("font-size", "13px")
                .style("width", "50px")
                .text("Hours")
            }))
            .child(html!("input" => HtmlInputElement, {
                .attr("type", "number")
                .attr("min", "0")
                .attr("step", "0.5")
                .style("background", "#374151")
                .style("color", "white")
                .style("border", "1px solid #4b5563")
                .style("border-radius", "4px")
                .style("padding", "6px 10px")
                .style("width", "90px")
                .style("font-size", "13px")
                .prop_signal("value", wd.draft.hours.signal_cloned())
                .with_node!(el => {
                    .event(clone!(wd => move |_: events::Input| {
                        wd.draft.hours.set(el.value());
                    }))
                })
            }))
        }))

        // Category rows
        .child(html!("div", {
            .dwclass!("flex flex-col gap-2")
            .children_signal_vec(wd.draft.categories.signal_vec_cloned().map(clone!(wd => move |cat| {
                render_category_row(wd.clone(), cat)
            })))
        }))

        // "+ Category" button
        .child(html!("button", {
            .style("background", "none")
            .style("color", "#60a5fa")
            .style("border", "1px solid #60a5fa")
            .style("border-radius", "4px")
            .style("padding", "4px 10px")
            .style("cursor", "pointer")
            .style("font-size", "12px")
            .style("align-self", "flex-start")
            .text("+ Category")
            .event(clone!(wd => move |_: events::Click| {
                wd.draft.categories.lock_mut().push_cloned(DraftCategory::new());
            }))
        }))

        // Validation error
        .child_signal(wd.error_msg.signal_ref(|msg| {
            msg.as_ref().map(|m| html!("span", {
                .style("color", "#f87171")
                .style("font-size", "12px")
                .text(m)
            }))
        }))

        // Add + Cancel buttons
        .child(html!("div", {
            .dwclass!("flex gap-4")
            .child(render_commit_button(wd.clone()))
            .child(html!("button", {
                .style("background", "none")
                .style("color", "#9ca3af")
                .style("border", "1px solid #4b5563")
                .style("border-radius", "4px")
                .style("padding", "6px 12px")
                .style("cursor", "pointer")
                .style("font-size", "13px")
                .text("Cancel")
                .event(clone!(wd => move |_: events::Click| {
                    wd.draft.reset();
                    wd.draft_visible.set(false);
                    wd.error_msg.set(None);
                }))
            }))
        }))
    })
}

fn render_category_row(wd: Arc<WorkdayState>, cat: Arc<DraftCategory>) -> Dom {
    html!("div", {
        .dwclass!("flex items-center gap-2")
        .child(html!("input" => HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", "Category")
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "6px 10px")
            .style("width", "130px")
            .style("font-size", "13px")
            .prop_signal("value", cat.key.signal_cloned())
            .with_node!(el => {
                .event(clone!(cat => move |_: events::Input| {
                    cat.key.set(el.value());
                }))
            })
        }))
        .child(html!("input" => HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", "Value")
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "6px 10px")
            .style("width", "130px")
            .style("font-size", "13px")
            .prop_signal("value", cat.value.signal_cloned())
            .with_node!(el => {
                .event(clone!(cat => move |_: events::Input| {
                    cat.value.set(el.value());
                }))
            })
        }))
        .child(html!("button", {
            .style("background", "none")
            .style("color", "#f87171")
            .style("border", "none")
            .style("cursor", "pointer")
            .style("font-size", "14px")
            .text("✕")
            .event(clone!(wd, cat => move |_: events::Click| {
                let ptr = Arc::as_ptr(&cat);
                wd.draft.categories.lock_mut().retain(|c| Arc::as_ptr(c) != ptr);
            }))
        }))
    })
}

fn render_commit_button(wd: Arc<WorkdayState>) -> Dom {
    html!("button", {
        .style("background", "#16a34a")
        .style("color", "white")
        .style("border", "none")
        .style("border-radius", "4px")
        .style("padding", "6px 14px")
        .style("cursor", "pointer")
        .style("font-size", "13px")
        .text("Add")
        .event(clone!(wd => move |_: events::Click| {
            let hours_str = wd.draft.hours.lock_ref().clone();
            match hours_str.trim().parse::<f64>() {
                Err(_) => {
                    wd.error_msg.set(Some("Hours must be a valid number.".to_string()));
                }
                Ok(h) if h < 0.0 => {
                    wd.error_msg.set(Some("Hours cannot be negative.".to_string()));
                }
                Ok(hours) => {
                    let categories = wd.draft.categories.lock_ref().iter()
                        .map(|c| (c.key.lock_ref().clone(), c.value.lock_ref().clone()))
                        .filter(|(k, _)| !k.is_empty())
                        .collect::<Vec<_>>();
                    wd.entries.lock_mut().push_cloned(WorkEntry { hours, categories });
                    let new_total: f64 = wd.entries.lock_ref().iter().map(|e| e.hours).sum();
                    wd.total_hours.set(new_total);
                    wd.draft.reset();
                    wd.draft_visible.set(false);
                    wd.error_msg.set(None);
                }
            }
        }))
    })
}

fn render_action_bar(state: Arc<AppState>) -> Dom {
    let wd = state.workday.clone();
    html!("div", {
        .dwclass!("flex flex-col gap-4")
        .style("padding-top", "8px")
        .style("border-top", "1px solid #374151")

        .child_signal(wd.status_msg.signal_ref(|msg| {
            msg.as_ref().map(|m| html!("span", {
                .style("color", "#6ee7b7")
                .style("font-size", "13px")
                .text(m)
            }))
        }))

        .child(html!("div", {
            .dwclass!("flex gap-4")
            .child(html!("button", {
                .style("background", "#374151")
                .style("color", "white")
                .style("border", "1px solid #4b5563")
                .style("border-radius", "4px")
                .style("padding", "6px 14px")
                .style("cursor", "pointer")
                .style("font-size", "13px")
                .text("Copy to Clipboard")
                .event(clone!(wd => move |_: events::Click| {
                    let entries = wd.entries.lock_ref().to_vec();
                    if entries.is_empty() {
                        wd.status_msg.set(Some("No entries to copy.".to_string()));
                        return;
                    }
                    let date = wd.date.lock_ref().clone();
                    let text = format_as_tsv(&entries, &date);
                    copy_to_clipboard(text, wd.clone());
                }))
            }))
            .child(html!("button", {
                .style("background", "#d97706")
                .style("color", "white")
                .style("border", "none")
                .style("border-radius", "4px")
                .style("padding", "6px 14px")
                .style("cursor", "pointer")
                .style("font-size", "13px")
                .text("Export")
                .event(clone!(state => move |_: events::Click| {
                    let state = state.clone();
                    spawn_local(async move { do_export(state).await; });
                }))
            }))
        }))
    })
}

fn format_as_tsv(entries: &[WorkEntry], date: &str) -> String {
    let mut cols: Vec<String> = Vec::new();
    for e in entries {
        for (k, _) in &e.categories {
            if !cols.contains(k) {
                cols.push(k.clone());
            }
        }
    }
    let mut rows = vec![
        std::iter::once("Date".to_string())
            .chain(cols.iter().cloned())
            .chain(std::iter::once("Hours".to_string()))
            .collect::<Vec<_>>()
            .join("\t"),
    ];
    for e in entries {
        let map: std::collections::HashMap<&str, &str> =
            e.categories.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let row = std::iter::once(date.to_string())
            .chain(cols.iter().map(|c| map.get(c.as_str()).unwrap_or(&"").to_string()))
            .chain(std::iter::once(format!("{:.1}", e.hours)))
            .collect::<Vec<_>>()
            .join("\t");
        rows.push(row);
    }
    rows.join("\n")
}

fn copy_to_clipboard(text: String, wd: Arc<WorkdayState>) {
    spawn_local(async move {
        let window = web_sys::window().expect("no window");
        let cb = window.navigator().clipboard();
        match JsFuture::from(cb.write_text(&text)).await {
            Ok(_) => wd.status_msg.set(Some("Copied to clipboard!".to_string())),
            Err(_) => wd.status_msg.set(Some("Failed to copy.".to_string())),
        }
    });
}

async fn do_export(state: Arc<AppState>) {
    let wd = &state.workday;
    let entries = wd.entries.lock_ref().to_vec();
    if entries.is_empty() {
        wd.status_msg.set(Some("No entries to export.".to_string()));
        return;
    }

    let folder = {
        let f = state.export_folder.lock_ref().clone();
        if !f.is_empty() {
            f
        } else {
            match tauri_wasm::invoke("pick_folder").await {
                Ok(js_val) => {
                    match serde_wasm_bindgen::from_value::<Option<String>>(js_val) {
                        Ok(Some(p)) => {
                            state.export_folder.set(p.clone());
                            let _ = AppState::save(state.clone()).await;
                            p
                        }
                        _ => {
                            wd.status_msg.set(Some("No folder selected.".to_string()));
                            return;
                        }
                    }
                }
                Err(_) => {
                    wd.status_msg.set(Some("Could not open folder picker.".to_string()));
                    return;
                }
            }
        }
    };

    let raw_args = ExportArgs {
        folder,
        format: state.export_format.lock_ref().as_str().to_string(),
        date: wd.date.lock_ref().clone(),
        entries,
    };
    let args = match tauri_wasm::args(&raw_args) {
        Ok(a) => a,
        Err(e) => {
            wd.status_msg.set(Some(format!("Error: {:?}", e)));
            return;
        }
    };
    match tauri_wasm::invoke("export_workday").with_args(args).await {
        Ok(_) => wd.status_msg.set(Some("Exported!".to_string())),
        Err(e) => wd.status_msg.set(Some(format!("Export failed: {:?}", e))),
    }
}
