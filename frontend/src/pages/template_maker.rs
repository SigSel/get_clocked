use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::SignalExt;
use futures_signals::signal_vec::SignalVecExt;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

use crate::app::{AppPage, AppState, DraftCategory, TemplateMakerState};

#[derive(serde::Serialize)]
struct SaveTemplateArgs {
    folder: String,
    name: String,
    categories: Vec<(String, String)>,
}

pub fn render(state: Arc<AppState>) -> Dom {
    let tm = state.template_maker.clone();
    html!("div", {
        .dwclass!("w-full h-screen bg-gray-900 flex flex-col")
        .style("color", "white")
        .child(render_category_keys_datalist(state.clone()))
        .child(render_header(state.clone()))
        .child(html!("div", {
            .dwclass!("flex flex-col items-center justify-center")
            .style("flex", "1")
            .style("overflow-y", "auto")
            .style("padding", "20px 0")
            .child(html!("div", {
                .dwclass!("flex flex-col gap-4")
                .style("width", "640px")
                .child(render_name_input(tm.clone()))
                .child(render_categories(tm.clone(), state.clone()))
                .child(render_add_category_button(tm.clone()))
                .child_signal(tm.error_msg.signal_ref(|msg| {
                    msg.as_ref().map(|m| html!("span", {
                        .style("color", "#f87171")
                        .style("font-size", "15px")
                        .text(m)
                    }))
                }))
                .child_signal(tm.status_msg.signal_ref(|msg| {
                    msg.as_ref().map(|m| html!("span", {
                        .style("color", "#6ee7b7")
                        .style("font-size", "16px")
                        .text(m)
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
            .text("Create Template")
        }))
    })
}

fn render_name_input(tm: Arc<TemplateMakerState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .child(html!("label", {
            .dwclass!("font-medium")
            .style("color", "#d1d5db")
            .style("font-size", "16px")
            .text("Template Name")
        }))
        .child(html!("input" => HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", "Template name")
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "10px 16px")
            .style("font-size", "17px")
            .style("width", "100%")
            .style("box-sizing", "border-box")
            .prop_signal("value", tm.name.signal_cloned())
            .with_node!(el => {
                .event(clone!(tm => move |_: events::Input| {
                    tm.name.set(el.value());
                }))
            })
        }))
    })
}

fn render_categories(tm: Arc<TemplateMakerState>, state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .children_signal_vec(tm.categories.signal_vec_cloned().map(clone!(tm, state => move |cat| {
            render_category_row(tm.clone(), state.clone(), cat)
        })))
    })
}

fn render_category_row(tm: Arc<TemplateMakerState>, state: Arc<AppState>, cat: Arc<DraftCategory>) -> Dom {
    let val_list_id = format!("cat-val-{:x}", Arc::as_ptr(&cat) as usize);
    let value_options = futures_signals::map_ref! {
        let key = cat.key.signal_cloned(),
        let defs = state.category_definitions.signal_cloned()
        => {
            defs.iter().find(|d| d.name == *key)
                .map(|d| d.values.clone()).unwrap_or_default()
        }
    };
    html!("div", {
        .dwclass!("flex items-center gap-2")
        .child(html!("datalist", {
            .attr("id", &val_list_id)
            .children_signal_vec(
                value_options
                    .map(|vals| vals.into_iter()
                        .map(|v| html!("option", { .attr("value", &v) }))
                        .collect::<Vec<_>>())
                    .to_signal_vec()
            )
        }))
        .child(html!("input" => HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", "Category")
            .attr("list", "cat-keys-datalist")
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "8px 14px")
            .style("width", "160px")
            .style("font-size", "16px")
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
            .attr("list", &val_list_id)
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "8px 14px")
            .style("width", "160px")
            .style("font-size", "16px")
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
            .event(clone!(tm, cat => move |_: events::Click| {
                let ptr = Arc::as_ptr(&cat);
                tm.categories.lock_mut().retain(|c| Arc::as_ptr(c) != ptr);
            }))
        }))
    })
}

fn render_category_keys_datalist(state: Arc<AppState>) -> Dom {
    html!("datalist", {
        .attr("id", "cat-keys-datalist")
        .children_signal_vec(
            state.category_definitions.signal_cloned()
                .map(|defs| defs.into_iter()
                    .map(|d| html!("option", { .attr("value", &d.name) }))
                    .collect::<Vec<_>>())
                .to_signal_vec()
        )
    })
}

fn render_add_category_button(tm: Arc<TemplateMakerState>) -> Dom {
    html!("button", {
        .style("background", "none")
        .style("color", "#60a5fa")
        .style("border", "1px solid #60a5fa")
        .style("border-radius", "4px")
        .style("padding", "6px 14px")
        .style("cursor", "pointer")
        .style("font-size", "15px")
        .style("align-self", "flex-start")
        .text("+ Category")
        .event(clone!(tm => move |_: events::Click| {
            tm.categories.lock_mut().push_cloned(DraftCategory::new());
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
        .text("Save Template")
        .event(clone!(state => move |_: events::Click| {
            let state = state.clone();
            spawn_local(async move {
                let tm = &state.template_maker;
                let name = tm.name.lock_ref().clone();
                if name.trim().is_empty() {
                    tm.error_msg.set(Some("Template name is required.".to_string()));
                    return;
                }
                let folder = state.template_folder.lock_ref().clone();
                if folder.is_empty() {
                    tm.error_msg.set(Some("Template folder is not set. Configure it in Settings.".to_string()));
                    return;
                }
                let categories = tm.categories.lock_ref().iter()
                    .map(|c| (c.key.lock_ref().clone(), c.value.lock_ref().clone()))
                    .filter(|(k, _)| !k.is_empty())
                    .collect::<Vec<_>>();
                tm.error_msg.set(None);
                let raw_args = SaveTemplateArgs { folder, name, categories };
                let args = match tauri_wasm::args(&raw_args) {
                    Err(e) => {
                        tm.error_msg.set(Some(format!("Error: {:?}", e)));
                        return;
                    }
                    Ok(a) => a,
                };
                match tauri_wasm::invoke("save_template").with_args(args).await {
                    Ok(_) => {
                        tm.reset();
                        tm.status_msg.set(Some("Template saved!".to_string()));
                    }
                    Err(e) => {
                        tm.error_msg.set(Some(format!("Save failed: {:?}", e)));
                    }
                }
            });
        }))
    })
}
