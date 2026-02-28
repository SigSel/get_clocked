use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlSelectElement;

use crate::app::{AppPage, AppState, DateFormat, ExportFormat};

pub fn render(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("w-full h-screen bg-gray-900 text-white flex flex-col")
        .child(render_header(state.clone()))
        .child(render_content(state.clone()))
    })
}

fn render_header(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("flex items-center p-4")
        .style("border-bottom", "1px solid #374151")
        .child(
            html!("button", {
                .dwclass!("cursor-pointer font-medium mr-4")
                .style("background", "none")
                .style("border", "none")
                .style("color", "#d1d5db")
                .style("padding", "6px 12px")
                .style("font-size", "16px")
                .text("← Back")
                .event(clone!(state => move |_: events::Click| {
                    state.page.set(AppPage::Home);
                }))
            })
        )
        .child(
            html!("h2", {
                .dwclass!("text-xl font-semibold text-white")
                .text("Settings")
            })
        )
    })
}

fn render_content(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col items-center justify-center")
        .style("flex", "1")
        .child(html!("div", {
            .dwclass!("flex flex-col gap-6")
            .style("width", "560px")
            .child(render_folder_section(state.clone()))
            .child(render_template_folder_section(state.clone()))
            .child(render_format_section(state.clone()))
            .child(render_date_format_section(state.clone()))
            .child(render_save_button(state.clone()))
        }))
    })
}

fn render_folder_section(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .child(
            html!("label", {
                .dwclass!("font-medium")
                .style("color", "#d1d5db")
                .style("font-size", "16px")
                .text("Export Folder")
            })
        )
        .child(
            html!("div", {
                .dwclass!("flex items-center gap-4")
                .child(
                    html!("span", {
                        .style("color", "#9ca3af")
                        .style("min-width", "260px")
                        .style("font-size", "16px")
                        .text_signal(state.export_folder.signal_ref(|folder| {
                            if folder.is_empty() {
                                "Not set".to_string()
                            } else {
                                folder.clone()
                            }
                        }))
                    })
                )
                .child(
                    html!("button", {
                        .dwclass!("cursor-pointer font-medium")
                        .style("background", "#2563eb")
                        .style("color", "white")
                        .style("border", "none")
                        .style("padding", "8px 16px")
                        .style("border-radius", "4px")
                        .style("font-size", "16px")
                        .text("Browse...")
                        .event(clone!(state => move |_: events::Click| {
                            let state = state.clone();
                            spawn_local(async move {
                                if let Ok(js_val) = tauri_wasm::invoke("pick_folder").await {
                                    if let Ok(Some(path)) =
                                        serde_wasm_bindgen::from_value::<Option<String>>(js_val)
                                    {
                                        state.export_folder.set(path);
                                    }
                                }
                            });
                        }))
                    })
                )
            })
        )
    })
}

fn render_template_folder_section(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .child(
            html!("label", {
                .dwclass!("font-medium")
                .style("color", "#d1d5db")
                .style("font-size", "16px")
                .text("Template Folder")
            })
        )
        .child(
            html!("div", {
                .dwclass!("flex items-center gap-4")
                .child(
                    html!("span", {
                        .style("color", "#9ca3af")
                        .style("min-width", "260px")
                        .style("font-size", "16px")
                        .text_signal(state.template_folder.signal_ref(|folder| {
                            if folder.is_empty() {
                                "Not set".to_string()
                            } else {
                                folder.clone()
                            }
                        }))
                    })
                )
                .child(
                    html!("button", {
                        .dwclass!("cursor-pointer font-medium")
                        .style("background", "#2563eb")
                        .style("color", "white")
                        .style("border", "none")
                        .style("padding", "8px 16px")
                        .style("border-radius", "4px")
                        .style("font-size", "16px")
                        .text("Browse...")
                        .event(clone!(state => move |_: events::Click| {
                            let state = state.clone();
                            spawn_local(async move {
                                if let Ok(js_val) = tauri_wasm::invoke("pick_folder").await {
                                    if let Ok(Some(path)) =
                                        serde_wasm_bindgen::from_value::<Option<String>>(js_val)
                                    {
                                        state.template_folder.set(path);
                                    }
                                }
                            });
                        }))
                    })
                )
            })
        )
    })
}

fn render_format_section(state: Arc<AppState>) -> Dom {
    let current_format = state.export_format.lock_ref().clone();

    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .child(
            html!("label", {
                .dwclass!("font-medium")
                .style("color", "#d1d5db")
                .style("font-size", "16px")
                .text("Export Format")
            })
        )
        .child(
            html!("select" => HtmlSelectElement, {
                .style("background", "#374151")
                .style("color", "white")
                .style("border", "1px solid #4b5563")
                .style("border-radius", "4px")
                .style("padding", "8px 16px")
                .style("font-size", "1rem")
                .style("width", "260px")
                .children(&mut [
                    html!("option", {
                        .attr("value", "csv")
                        .text("CSV")
                        .apply(|b| {
                            if current_format == ExportFormat::Csv {
                                b.attr("selected", "")
                            } else {
                                b
                            }
                        })
                    }),
                    html!("option", {
                        .attr("value", "xlsx")
                        .text("XLSX")
                        .apply(|b| {
                            if current_format == ExportFormat::Xlsx {
                                b.attr("selected", "")
                            } else {
                                b
                            }
                        })
                    }),
                ])
                .with_node!(element => {
                    .event(clone!(state => move |_: events::Change| {
                        state.export_format.set(ExportFormat::from_str(&element.value()));
                    }))
                })
            })
        )
    })
}

fn render_date_format_section(state: Arc<AppState>) -> Dom {
    let current_format = state.date_format.lock_ref().clone();

    html!("div", {
        .dwclass!("flex flex-col gap-2")
        .child(
            html!("label", {
                .dwclass!("font-medium")
                .style("color", "#d1d5db")
                .style("font-size", "16px")
                .text("Date Format")
            })
        )
        .child(
            html!("select" => HtmlSelectElement, {
                .style("background", "#374151")
                .style("color", "white")
                .style("border", "1px solid #4b5563")
                .style("border-radius", "4px")
                .style("padding", "8px 16px")
                .style("font-size", "1rem")
                .style("width", "260px")
                .children(&mut [
                    html!("option", {
                        .attr("value", "YYYY-MM-DD")
                        .text("YYYY-MM-DD")
                        .apply(|b| {
                            if current_format == DateFormat::YyyyMmDd {
                                b.attr("selected", "")
                            } else {
                                b
                            }
                        })
                    }),
                    html!("option", {
                        .attr("value", "YYYY.MM.DD")
                        .text("YYYY.MM.DD")
                        .apply(|b| {
                            if current_format == DateFormat::YyyyDotMmDotDd {
                                b.attr("selected", "")
                            } else {
                                b
                            }
                        })
                    }),
                    html!("option", {
                        .attr("value", "DD-MM-YYYY")
                        .text("DD-MM-YYYY")
                        .apply(|b| {
                            if current_format == DateFormat::DdMmYyyy {
                                b.attr("selected", "")
                            } else {
                                b
                            }
                        })
                    }),
                    html!("option", {
                        .attr("value", "DD.MM.YYYY")
                        .text("DD.MM.YYYY")
                        .apply(|b| {
                            if current_format == DateFormat::DdDotMmDotYyyy {
                                b.attr("selected", "")
                            } else {
                                b
                            }
                        })
                    }),
                ])
                .with_node!(element => {
                    .event(clone!(state => move |_: events::Change| {
                        state.date_format.set(DateFormat::from_str(&element.value()));
                    }))
                })
            })
        )
    })
}

fn render_save_button(state: Arc<AppState>) -> Dom {
    html!("button", {
        .dwclass!("cursor-pointer font-medium")
        .style("background", "#16a34a")
        .style("color", "white")
        .style("border", "none")
        .style("padding", "10px 26px")
        .style("border-radius", "4px")
        .style("align-self", "flex-start")
        .style("margin-top", "8px")
        .style("font-size", "16px")
        .text("Save")
        .event(clone!(state => move |_: events::Click| {
            let state = state.clone();
            spawn_local(async move {
                if AppState::save(state.clone()).await.is_ok() {
                    state.page.set(AppPage::Home);
                }
            });
        }))
    })
}
