use std::sync::Arc;

use dominator::{clone, events, html, with_node, Dom, DomBuilder};
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
use futures_signals::signal_vec::MutableVec;
use web_sys::HtmlElement;
use web_sys::HtmlInputElement;

use crate::app::{AppState, CategoryDefinition, DraftCategory};

// ---------------------------------------------------------------------------
// Style mixins — use with `.apply(components::xxx_styles)`
// ---------------------------------------------------------------------------

pub fn input_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("background", "#374151")
        .style("color", "white")
        .style("border", "1px solid #4b5563")
        .style("border-radius", "4px")
        .style("padding", "8px 14px")
        .style("font-size", "16px")
}

pub fn select_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("background", "#374151")
        .style("color", "white")
        .style("border", "1px solid #4b5563")
        .style("border-radius", "4px")
        .style("padding", "8px 16px")
        .style("font-size", "1rem")
        .style("width", "260px")
}

pub fn primary_button_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("background", "#16a34a")
        .style("color", "white")
        .style("border", "none")
        .style("padding", "10px 26px")
        .style("border-radius", "4px")
        .style("font-size", "16px")
}

pub fn action_button_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("background", "#2563eb")
        .style("color", "white")
        .style("border", "none")
        .style("padding", "8px 18px")
        .style("border-radius", "4px")
        .style("cursor", "pointer")
        .style("font-size", "16px")
}

pub fn secondary_button_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("background", "none")
        .style("color", "#60a5fa")
        .style("border", "1px solid #60a5fa")
        .style("border-radius", "4px")
        .style("padding", "6px 14px")
        .style("cursor", "pointer")
        .style("font-size", "15px")
}

pub fn danger_button_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("background", "none")
        .style("color", "#f87171")
        .style("border", "1px solid #f87171")
        .style("border-radius", "4px")
        .style("padding", "4px 12px")
        .style("cursor", "pointer")
        .style("font-size", "14px")
}

pub fn back_button_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("background", "none")
        .style("border", "none")
        .style("color", "#d1d5db")
        .style("cursor", "pointer")
        .style("font-size", "17px")
}

pub fn label_styles<A: AsRef<HtmlElement>>(b: DomBuilder<A>) -> DomBuilder<A> {
    b.style("color", "#d1d5db").style("font-size", "16px")
}

// ---------------------------------------------------------------------------
// Shared DOM helpers
// ---------------------------------------------------------------------------

pub fn render_status_message(msg: &Option<String>) -> Option<Dom> {
    msg.as_ref().map(|m| {
        html!("span", {
            .style("color", "#6ee7b7")
            .style("font-size", "16px")
            .text(m)
        })
    })
}

pub fn render_error_message(msg: &Option<String>) -> Option<Dom> {
    msg.as_ref().map(|m| {
        html!("span", {
            .style("color", "#f87171")
            .style("font-size", "15px")
            .text(m)
        })
    })
}

// ---------------------------------------------------------------------------
// Shared category components
// ---------------------------------------------------------------------------

pub fn render_category_keys_datalist(state: Arc<AppState>) -> Dom {
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

pub fn render_category_row(
    categories: &MutableVec<Arc<DraftCategory>>,
    category_definitions: &Mutable<Vec<CategoryDefinition>>,
    cat: Arc<DraftCategory>,
) -> Dom {
    let val_list_id = format!("cat-val-{:x}", Arc::as_ptr(&cat) as usize);
    let value_options = futures_signals::map_ref! {
        let key = cat.key.signal_cloned(),
        let defs = category_definitions.signal_cloned()
        => {
            defs.iter().find(|d| d.name == *key)
                .map(|d| d.values.clone()).unwrap_or_default()
        }
    };
    let categories_clone = categories.clone();
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
            .apply(input_styles)
            .style("width", "160px")
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
            .apply(input_styles)
            .style("width", "160px")
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
            .event(clone!(cat => move |_: events::Click| {
                let ptr = Arc::as_ptr(&cat);
                categories_clone.lock_mut().retain(|c| Arc::as_ptr(c) != ptr);
            }))
        }))
    })
}

use dwind::prelude::*;
use dwind_macros::dwclass;
