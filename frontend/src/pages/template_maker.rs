use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
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
        .child(render_header(state.clone()))
        .child(html!("div", {
            .dwclass!("flex flex-col items-center justify-center")
            .style("flex", "1")
            .style("overflow-y", "auto")
            .style("padding", "16px 0")
            .child(html!("div", {
                .dwclass!("flex flex-col gap-4")
                .style("width", "480px")
                .child(render_name_input(tm.clone()))
                .child(render_categories(tm.clone()))
                .child(render_add_category_button(tm.clone()))
                .child_signal(tm.error_msg.signal_ref(|msg| {
                    msg.as_ref().map(|m| html!("span", {
                        .style("color", "#f87171")
                        .style("font-size", "12px")
                        .text(m)
                    }))
                }))
                .child_signal(tm.status_msg.signal_ref(|msg| {
                    msg.as_ref().map(|m| html!("span", {
                        .style("color", "#6ee7b7")
                        .style("font-size", "13px")
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
            .style("font-size", "14px")
            .text("← Back")
            .event(clone!(state => move |_: events::Click| {
                state.page.set(AppPage::Home);
            }))
        }))
        .child(html!("h2", {
            .dwclass!("text-lg font-semibold")
            .text("Create Template")
        }))
    })
}

fn render_name_input(tm: Arc<TemplateMakerState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .child(html!("label", {
            .dwclass!("text-sm font-medium")
            .style("color", "#d1d5db")
            .text("Template Name")
        }))
        .child(html!("input" => HtmlInputElement, {
            .attr("type", "text")
            .attr("placeholder", "Template name")
            .style("background", "#374151")
            .style("color", "white")
            .style("border", "1px solid #4b5563")
            .style("border-radius", "4px")
            .style("padding", "8px 12px")
            .style("font-size", "14px")
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

fn render_categories(tm: Arc<TemplateMakerState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .children_signal_vec(tm.categories.signal_vec_cloned().map(clone!(tm => move |cat| {
            render_category_row(tm.clone(), cat)
        })))
    })
}

fn render_category_row(tm: Arc<TemplateMakerState>, cat: Arc<DraftCategory>) -> Dom {
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
            .event(clone!(tm, cat => move |_: events::Click| {
                let ptr = Arc::as_ptr(&cat);
                tm.categories.lock_mut().retain(|c| Arc::as_ptr(c) != ptr);
            }))
        }))
    })
}

fn render_add_category_button(tm: Arc<TemplateMakerState>) -> Dom {
    html!("button", {
        .style("background", "none")
        .style("color", "#60a5fa")
        .style("border", "1px solid #60a5fa")
        .style("border-radius", "4px")
        .style("padding", "4px 10px")
        .style("cursor", "pointer")
        .style("font-size", "12px")
        .style("align-self", "flex-start")
        .text("+ Category")
        .event(clone!(tm => move |_: events::Click| {
            tm.categories.lock_mut().push_cloned(DraftCategory::new());
        }))
    })
}

fn render_save_button(state: Arc<AppState>) -> Dom {
    html!("button", {
        .dwclass!("cursor-pointer text-sm font-semibold")
        .style("background", "#16a34a")
        .style("color", "white")
        .style("border", "none")
        .style("padding", "8px 20px")
        .style("border-radius", "4px")
        .style("align-self", "flex-start")
        .style("margin-top", "8px")
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
