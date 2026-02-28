use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use futures_signals::signal::SignalExt;
use futures_signals::signal_vec::SignalVecExt;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

use crate::app::{AppPage, AppState, DraftCategory, TemplateMakerState, TemplateData};

#[derive(serde::Serialize)]
struct SaveTemplateArgs {
    folder: String,
    name: String,
    categories: Vec<(String, String)>,
}

#[derive(serde::Serialize)]
struct DeleteTemplateArgs {
    folder: String,
    name: String,
}

async fn invoke_delete_template(folder: String, name: String) {
    if let Ok(args) = tauri_wasm::args(&DeleteTemplateArgs { folder, name }) {
        let _ = tauri_wasm::invoke("delete_template").with_args(args).await;
    }
}

async fn load_templates(tm: Arc<TemplateMakerState>, folder: String) {
    if folder.is_empty() {
        return;
    }
    #[derive(serde::Serialize)]
    struct ListArgs {
        folder: String,
    }
    if let Ok(args) = tauri_wasm::args(&ListArgs { folder }) {
        if let Ok(js_val) = tauri_wasm::invoke("list_templates").with_args(args).await {
            if let Ok(list) = serde_wasm_bindgen::from_value::<Vec<TemplateData>>(js_val) {
                tm.templates.lock_mut().replace_cloned(list);
            }
        }
    }
}

pub fn render(state: Arc<AppState>) -> Dom {
    let tm = state.template_maker.clone();
    {
        let tm = tm.clone();
        let folder = state.template_folder.lock_ref().clone();
        spawn_local(async move { load_templates(tm, folder).await; });
    }
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
                .child(render_action_buttons(state.clone()))
                .child(render_templates_list(state.clone()))
            }))
        }))
    })
}

fn render_header(state: Arc<AppState>) -> Dom {
    let tm = state.template_maker.clone();
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
                state.template_maker.reset();
                state.page.set(AppPage::Home);
            }))
        }))
        .child_signal(tm.editing_original_name.signal_ref(|name| {
            let title = if name.is_some() { "Edit Template" } else { "Create Template" };
            Some(html!("h2", { .dwclass!("text-xl font-semibold").text(title) }))
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

fn render_action_buttons(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("flex gap-4 items-center")
        .child_signal(state.template_maker.editing_original_name.signal_ref(
            clone!(state => move |name| Some(render_save_button(state.clone(), name.is_some())))
        ))
        .child_signal(state.template_maker.editing_original_name.signal_ref(
            clone!(state => move |name| {
                if name.is_some() {
                    Some(html!("button", {
                        .style("background", "none")
                        .style("color", "#9ca3af")
                        .style("border", "1px solid #6b7280")
                        .style("border-radius", "4px")
                        .style("padding", "10px 26px")
                        .style("cursor", "pointer")
                        .style("font-size", "16px")
                        .text("Cancel")
                        .event(clone!(state => move |_: events::Click| {
                            state.template_maker.reset();
                        }))
                    }))
                } else {
                    None
                }
            })
        ))
    })
}

fn render_save_button(state: Arc<AppState>, is_editing: bool) -> Dom {
    let label = if is_editing { "Update Template" } else { "Save Template" };
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
        .text(label)
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

                // If editing, delete the old file first (handles renames)
                let original_name = tm.editing_original_name.lock_ref().clone();
                if let Some(orig) = original_name {
                    invoke_delete_template(folder.clone(), orig).await;
                }

                let raw_args = SaveTemplateArgs { folder: folder.clone(), name, categories };
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
                        load_templates(tm.clone(), folder.clone()).await;
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

fn render_templates_list(state: Arc<AppState>) -> Dom {
    let tm = state.template_maker.clone();
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .style("margin-top", "16px")
        .child(html!("h3", {
            .dwclass!("font-semibold")
            .style("color", "#d1d5db")
            .style("font-size", "16px")
            .text("Saved Templates")
        }))
        .children_signal_vec(tm.templates.signal_vec_cloned()
            .map(clone!(state => move |t| render_template_row(state.clone(), t))))
    })
}

fn render_template_row(state: Arc<AppState>, template: TemplateData) -> Dom {
    let tm = state.template_maker.clone();
    html!("div", {
        .dwclass!("flex items-center gap-4")
        .style("padding", "8px 12px")
        .style("background", "#1f2937")
        .style("border-radius", "4px")
        .child(html!("span", {
            .style("flex", "1")
            .style("color", "#e5e7eb")
            .style("font-size", "15px")
            .text(&template.name)
        }))
        .child(html!("button", {
            .style("background", "none")
            .style("color", "#60a5fa")
            .style("border", "1px solid #60a5fa")
            .style("border-radius", "4px")
            .style("padding", "4px 12px")
            .style("cursor", "pointer")
            .style("font-size", "14px")
            .text("Edit")
            .event(clone!(tm, template => move |_: events::Click| {
                tm.name.set(template.name.clone());
                {
                    let mut cats = tm.categories.lock_mut();
                    cats.clear();
                    for (k, v) in &template.categories {
                        let cat = DraftCategory::new();
                        cat.key.set(k.clone());
                        cat.value.set(v.clone());
                        cats.push_cloned(cat);
                    }
                }
                tm.error_msg.set(None);
                tm.status_msg.set(None);
                tm.editing_original_name.set(Some(template.name.clone()));
            }))
        }))
        .child(html!("button", {
            .style("background", "none")
            .style("color", "#f87171")
            .style("border", "1px solid #f87171")
            .style("border-radius", "4px")
            .style("padding", "4px 12px")
            .style("cursor", "pointer")
            .style("font-size", "14px")
            .text("Delete")
            .event(clone!(state, template => move |_: events::Click| {
                let state = state.clone();
                let template_name = template.name.clone();
                spawn_local(async move {
                    let tm = &state.template_maker;
                    let folder = state.template_folder.lock_ref().clone();
                    invoke_delete_template(folder.clone(), template_name.clone()).await;
                    load_templates(tm.clone(), folder).await;
                    let editing = tm.editing_original_name.lock_ref().clone();
                    if editing.as_deref() == Some(template_name.as_str()) {
                        tm.reset();
                    }
                    tm.status_msg.set(Some(format!("'{}' deleted.", template_name)));
                });
            }))
        }))
    })
}
