use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::Mutable;
use futures_signals::signal_vec::SignalVecExt;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

use crate::app::{AppPage, AppState, CategoryDefinition, CategoryManagerState, DraftCategoryDefinition};

#[derive(serde::Serialize)]
struct SaveCategoriesArgs {
    categories: Vec<CategoryDefinition>,
}

pub fn render(state: Arc<AppState>) -> Dom {
    let cm = state.category_manager.clone();

    // Populate definitions from state.category_definitions on mount
    {
        let defs = state.category_definitions.lock_ref().clone();
        let mut lock = cm.definitions.lock_mut();
        lock.clear();
        for def in &defs {
            lock.push_cloned(DraftCategoryDefinition::from_definition(def));
        }
    }

    html!("div", {
        .dwclass!("w-full h-screen bg-gray-900 flex flex-col")
        .style("color", "white")
        .child(render_header(state.clone()))
        .child(html!("div", {
            .dwclass!("flex flex-col items-center")
            .style("flex", "1")
            .style("overflow-y", "auto")
            .style("padding", "20px 0")
            .child(html!("div", {
                .dwclass!("flex flex-col gap-4")
                .style("width", "640px")
                .child_signal(cm.status_msg.signal_ref(|msg| {
                    msg.as_ref().map(|m| html!("span", {
                        .style("color", "#6ee7b7")
                        .style("font-size", "16px")
                        .text(m)
                    }))
                }))
                .child_signal(cm.error_msg.signal_ref(|msg| {
                    msg.as_ref().map(|m| html!("span", {
                        .style("color", "#f87171")
                        .style("font-size", "15px")
                        .text(m)
                    }))
                }))
                .children_signal_vec(cm.definitions.signal_vec_cloned().map(clone!(cm => move |def| {
                    render_definition_card(cm.clone(), def)
                })))
                .child(html!("button", {
                    .style("background", "none")
                    .style("color", "#60a5fa")
                    .style("border", "1px solid #60a5fa")
                    .style("border-radius", "4px")
                    .style("padding", "6px 14px")
                    .style("cursor", "pointer")
                    .style("font-size", "15px")
                    .style("align-self", "flex-start")
                    .text("+ Add Category")
                    .event(clone!(cm => move |_: events::Click| {
                        cm.definitions.lock_mut().push_cloned(DraftCategoryDefinition::new());
                    }))
                }))
                .child(render_save_button(state.clone()))
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
            .style("font-size", "17px")
            .text("← Back")
            .event(clone!(state => move |_: events::Click| {
                state.page.set(AppPage::Home);
            }))
        }))
        .child(html!("h2", {
            .dwclass!("text-xl font-semibold")
            .text("Category Manager")
        }))
    })
}

fn render_definition_card(cm: Arc<CategoryManagerState>, def: Arc<DraftCategoryDefinition>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .style("background", "#1f2937")
        .style("border", "1px solid #374151")
        .style("border-radius", "6px")
        .style("padding", "16px")
        .child(html!("div", {
            .dwclass!("flex items-center gap-2")
            .child(html!("label", {
                .style("color", "#d1d5db")
                .style("font-size", "16px")
                .style("min-width", "80px")
                .text("Name")
            }))
            .child(html!("input" => HtmlInputElement, {
                .attr("type", "text")
                .attr("placeholder", "Category name")
                .style("background", "#374151")
                .style("color", "white")
                .style("border", "1px solid #4b5563")
                .style("border-radius", "4px")
                .style("padding", "8px 14px")
                .style("width", "200px")
                .style("font-size", "16px")
                .prop_signal("value", def.name.signal_cloned())
                .with_node!(el => {
                    .event(clone!(def => move |_: events::Input| {
                        def.name.set(el.value());
                    }))
                })
            }))
            .child(html!("button", {
                .style("background", "none")
                .style("color", "#f87171")
                .style("border", "none")
                .style("cursor", "pointer")
                .style("font-size", "14px")
                .style("margin-left", "auto")
                .text("✕ Remove")
                .event(clone!(cm, def => move |_: events::Click| {
                    let ptr = Arc::as_ptr(&def);
                    cm.definitions.lock_mut().retain(|d| Arc::as_ptr(d) != ptr);
                }))
            }))
        }))
        .children_signal_vec(def.values.signal_vec_cloned().map(clone!(def => move |val| {
            render_value_row(def.clone(), val)
        })))
        .child(html!("button", {
            .style("background", "none")
            .style("color", "#60a5fa")
            .style("border", "1px solid #60a5fa")
            .style("border-radius", "4px")
            .style("padding", "4px 12px")
            .style("cursor", "pointer")
            .style("font-size", "14px")
            .style("align-self", "flex-start")
            .text("+ Add Value")
            .event(clone!(def => move |_: events::Click| {
                def.values.lock_mut().push_cloned(Arc::new(Mutable::new(String::new())));
            }))
        }))
    })
}

fn render_value_row(def: Arc<DraftCategoryDefinition>, val: Arc<Mutable<String>>) -> Dom {
    html!("div", {
        .dwclass!("flex items-center gap-2")
        .style("padding-left", "88px")
        .child(html!("input" => HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", "Value")
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "6px 12px")
            .style("width", "200px")
            .style("font-size", "15px")
            .prop_signal("value", val.signal_cloned())
            .with_node!(el => {
                .event(clone!(val => move |_: events::Input| {
                    val.set(el.value());
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
            .event(clone!(def, val => move |_: events::Click| {
                let ptr = Arc::as_ptr(&val);
                def.values.lock_mut().retain(|v| Arc::as_ptr(v) != ptr);
            }))
        }))
    })
}

fn render_save_button(state: Arc<AppState>) -> Dom {
    html!("button", {
        .dwclass!("cursor-pointer font-semibold")
        .style("background", "#16a34a")
        .style("color", "white")
        .style("border", "none")
        .style("padding", "10px 26px")
        .style("border-radius", "4px")
        .style("align-self", "flex-start")
        .style("margin-top", "8px")
        .style("font-size", "16px")
        .text("Save Categories")
        .event(clone!(state => move |_: events::Click| {
            let state = state.clone();
            spawn_local(async move {
                let cm = &state.category_manager;
                let categories: Vec<CategoryDefinition> = cm.definitions.lock_ref().iter()
                    .map(|def| {
                        let name = def.name.lock_ref().clone();
                        let values: Vec<String> = def.values.lock_ref().iter()
                            .map(|v| v.lock_ref().clone())
                            .filter(|v| !v.is_empty())
                            .collect();
                        CategoryDefinition { name, values }
                    })
                    .filter(|c| !c.name.trim().is_empty())
                    .collect();
                let raw_args = SaveCategoriesArgs { categories: categories.clone() };
                let args = match tauri_wasm::args(&raw_args) {
                    Err(e) => {
                        cm.error_msg.set(Some(format!("Error: {:?}", e)));
                        return;
                    }
                    Ok(a) => a,
                };
                match tauri_wasm::invoke("save_categories").with_args(args).await {
                    Ok(_) => {
                        state.category_definitions.set(categories);
                        cm.error_msg.set(None);
                        cm.status_msg.set(Some("Saved!".to_string()));
                    }
                    Err(e) => {
                        cm.error_msg.set(Some(format!("Save failed: {:?}", e)));
                    }
                }
            });
        }))
    })
}
