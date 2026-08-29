use dioxus::prelude::*;

/// Modal for editing the display size of the image under the caret.
/// Width/height are pixel values; leaving both empty makes the image render
/// responsively again (no `#img=` fragment).
#[component]
pub fn ImageSizeModal(
    width: Signal<String>,
    height: Signal<String>,
    alt: String,
    url: String,
    on_apply: EventHandler<()>,
    on_remove: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center overflow-y-auto",
            // Overlay backdrop — clicking it cancels. z-40 keeps it below
            // the card (z-50) so the modal stays clickable.
            div {
                class: "fixed inset-0 bg-gray-500/30 transition-opacity z-40",
                onclick: move |_| on_cancel.call(()),
            }

            div {
                class: "relative bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:max-w-lg w-full mx-4 z-50",
                // Header
                div {
                    class: "bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4",
                        h3 {
                            class: "text-lg leading-6 font-medium text-gray-900",
                            "Image Size"
                        }
                        div {
                            class: "mt-2 text-sm text-gray-500 break-all",
                            p { "Alt: {alt}" }
                            p {
                                class: "mt-1",
                                span { "URL: " }
                                span { class: "font-mono text-xs", "{url}" }
                            }
                        }

                        // Inputs
                        div {
                            class: "mt-4 grid grid-cols-2 gap-4",
                            div {
                                label {
                                    class: "block text-sm font-medium text-gray-700",
                                    "Width (px)"
                                }
                                input {
                                    r#type: "number",
                                    class: "mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                                    value: "{width}",
                                    placeholder: "auto",
                                    min: "1",
                                    oninput: move |e: Event<FormData>| {
                                        *width.write() = e.value();
                                    }
                                }
                            }
                            div {
                                label {
                                    class: "block text-sm font-medium text-gray-700",
                                    "Height (px)"
                                }
                                input {
                                    r#type: "number",
                                    class: "mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm",
                                    value: "{height}",
                                    placeholder: "auto",
                                    min: "1",
                                    oninput: move |e: Event<FormData>| {
                                        *height.write() = e.value();
                                    }
                                }
                            }
                        }
                        p {
                            class: "mt-3 text-xs text-gray-400",
                            "Leave both empty to render the image responsively (fills the container)."
                        }
                    }

                    // Footer
                    div {
                        class: "bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse",
                        button {
                            r#type: "button",
                            class: "w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-indigo-600 text-base font-medium text-white hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:ml-3 sm:w-auto sm:text-sm",
                            onclick: move |_| on_apply.call(()),
                            "Apply"
                        }
                        button {
                            r#type: "button",
                            class: "mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm",
                            onclick: move |_| on_remove.call(()),
                            "Reset to auto"
                        }
                        button {
                            r#type: "button",
                            class: "mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm",
                            onclick: move |_| on_cancel.call(()),
                            "Cancel"
                        }
                    }
                }
            }
    }
}

