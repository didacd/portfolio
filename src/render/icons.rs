use dioxus::prelude::*;
use dioxus_mdx::MdxIcon;

// Additional Lucide icons not available through dioxus-mdx's icon mapping.
// See assets/icons/lucide-LICENSE.txt for attribution.
#[component]
pub(super) fn LucideIcon(name: String) -> Element {
    match name.as_str() {
        "flame" | "circle-x" | "quote" | "calendar" => rsx! {
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                view_box: "0 0 24 24",
                width: "24",
                height: "24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                "aria-hidden": "true",
                "focusable": "false",
                match name.as_str() {
                    "calendar" => rsx! {
                        path { d: "M8 2v4" }
                        path { d: "M16 2v4" }
                        rect { x: "3", y: "4", width: "18", height: "18", rx: "2" }
                        path { d: "M3 10h18" }
                    },
                    "flame" => rsx! {
                        path { d: "M12 3q1 4 4 6.5t3 5.5a1 1 0 0 1-14 0 5 5 0 0 1 1-3 1 1 0 0 0 5 0c0-2-1.5-3-1.5-5q0-2 2.5-4" }
                    },
                    "circle-x" => rsx! {
                        circle { cx: "12", cy: "12", r: "10" }
                        path { d: "m15 9-6 6" }
                        path { d: "m9 9 6 6" }
                    },
                    _ => rsx! {
                        path { d: "M16 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z" }
                        path { d: "M5 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z" }
                    },
                }
            }
        },
        "circle-help" => rsx! { MdxIcon { name: "help".to_owned() } },
        _ => rsx! { MdxIcon { name } },
    }
}
