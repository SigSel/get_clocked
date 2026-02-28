use std::sync::Arc;

use dominator::{clone, events, html, svg, Dom};
use dwind::prelude::*;
use dwind_macros::dwclass;

use crate::app::{AppPage, AppState};

pub fn render(state: Arc<AppState>) -> Dom {
    html!("div", {
        .dwclass!("relative w-full h-screen bg-gray-900")
        .child(
            // Gear icon button — top-right
            html!("button", {
                .dwclass!("absolute top-4 right-4 cursor-pointer text-gray-400")
                .style("background", "none")
                .style("border", "none")
                .style("padding", "4px")
                .style("line-height", "0")
                .attr("title", "Settings")
                .event(clone!(state => move |_: events::Click| {
                    state.page.set(AppPage::Settings);
                }))
                .child(gear_icon())
            })
        )
        .child(
            // Centered content
            html!("div", {
                .dwclass!("flex flex-col items-center justify-center gap-8 w-full h-screen")
                .child(html!("h1", {
                    .dwclass!("text-5xl font-bold text-white")
                    .text("Get Clocked")
                }))
                .child(html!("button", {
                    .dwclass!("cursor-pointer text-xl font-semibold")
                    .style("background", "#2563eb")
                    .style("color", "white")
                    .style("border", "none")
                    .style("padding", "14px 36px")
                    .style("border-radius", "6px")
                    .text("Register Workday")
                    .event(clone!(state => move |_: events::Click| {
                        state.workday.reset();
                        state.page.set(AppPage::RegisterWorkday);
                    }))
                }))
                .child(html!("button", {
                    .dwclass!("cursor-pointer font-medium")
                    .style("background", "none")
                    .style("color", "#93c5fd")
                    .style("border", "1px solid #3b82f6")
                    .style("padding", "10px 30px")
                    .style("border-radius", "6px")
                    .style("font-size", "16px")
                    .text("Templates")
                    .event(clone!(state => move |_: events::Click| {
                        state.template_maker.reset();
                        state.page.set(AppPage::TemplateMaker);
                    }))
                }))
                .child(html!("button", {
                    .dwclass!("cursor-pointer font-medium")
                    .style("background", "none")
                    .style("color", "#93c5fd")
                    .style("border", "1px solid #3b82f6")
                    .style("padding", "10px 30px")
                    .style("border-radius", "6px")
                    .style("font-size", "16px")
                    .text("Categories")
                    .event(clone!(state => move |_: events::Click| {
                        state.page.set(AppPage::CategoryManager);
                    }))
                }))
            })
        )
    })
}

fn gear_icon() -> Dom {
    svg!("svg", {
        .attr("xmlns", "http://www.w3.org/2000/svg")
        .attr("viewBox", "0 0 24 24")
        .attr("fill", "currentColor")
        .attr("width", "30")
        .attr("height", "30")
        .child(svg!("path", {
            .attr("fill-rule", "evenodd")
            .attr("d", "M11.078 2.25c-.917 0-1.699.663-1.85 1.567L9.05 4.889c-.02.12-.115.26-.297.348a7.493 7.493 0 0 0-.986.57c-.166.115-.334.126-.45.083L6.3 5.508a1.875 1.875 0 0 0-2.282.819l-.922 1.597a1.875 1.875 0 0 0 .432 2.385l.84.692c.095.078.17.229.154.43a7.598 7.598 0 0 0 0 1.139c.015.2-.059.352-.153.43l-.841.692a1.875 1.875 0 0 0-.432 2.385l.922 1.597a1.875 1.875 0 0 0 2.282.818l1.019-.382c.115-.043.283-.031.45.082.312.214.641.405.985.57.182.088.277.228.297.35l.178 1.071c.151.904.933 1.567 1.85 1.567h1.844c.916 0 1.699-.663 1.85-1.567l.178-1.072c.02-.12.114-.26.297-.349.344-.165.673-.356.985-.57.167-.114.335-.125.45-.082l1.02.382a1.875 1.875 0 0 0 2.28-.819l.923-1.597a1.875 1.875 0 0 0-.432-2.385l-.84-.692c-.095-.078-.17-.229-.154-.43a7.614 7.614 0 0 0 0-1.139c-.016-.2.059-.352.153-.43l.84-.692c.708-.582.891-1.59.433-2.385l-.922-1.597a1.875 1.875 0 0 0-2.282-.818l-1.02.382c-.114.043-.282.031-.449-.083a7.49 7.49 0 0 0-.985-.57c-.183-.087-.277-.227-.297-.348l-.179-1.072a1.875 1.875 0 0 0-1.85-1.567h-1.843ZM12 15.75a3.75 3.75 0 1 0 0-7.5 3.75 3.75 0 0 0 0 7.5Z")
            .attr("clip-rule", "evenodd")
        }))
    })
}
