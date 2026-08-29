use dioxus::prelude::*;

/// Shared props for [TextField].
#[derive(Clone, PartialEq, Props)]
pub struct TextFieldProps {
    /// Label rendered above the input.
    label: String,
    /// Bound value.
    value: String,
    /// Called on every keystroke with the new value.
    oninput: EventHandler<String>,
    /// Helper text rendered under the input.
    #[props(default = String::new())]
    hint: String,
    /// Disable the field.
    disabled: bool,
}

/// Shared labelled text input.
#[component]
pub fn TextField(props: TextFieldProps) -> Element {
    rsx! {
        div {
            label {
                class: "block text-sm font-medium text-gray-700",
                "{props.label}"
            }
            input {
                r#type: "text",
                value: "{props.value}",
                class: "mt-1 block w-full border border-gray-300 rounded-md shadow-sm py-2 px-3 focus:outline-none focus:ring-purple-400 focus:border-purple-400 sm:text-sm",
                oninput: move |e: Event<FormData>| props.oninput.call(e.value()),
                disabled: props.disabled,
            }
            if !props.hint.is_empty() {
                p {
                    class: "mt-1 text-xs text-gray-500",
                    "{props.hint}"
                }
            }
        }
    }
}

/// Shared props for [SelectField].
#[derive(Clone, PartialEq, Props)]
pub struct SelectFieldProps {
    /// Label rendered above the select.
    label: String,
    /// Currently selected value.
    value: String,
    /// (value, label) options.
    options: Vec<(String, String)>,
    /// Called when the selection changes.
    onchange: EventHandler<String>,
    /// Disable the field.
    disabled: bool,
}

/// Shared labelled select input.
#[component]
pub fn SelectField(props: SelectFieldProps) -> Element {
    rsx! {
        div {
            label {
                class: "block text-sm font-medium text-gray-700",
                "{props.label}"
            }
            select {
                value: "{props.value}",
                class: "mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-purple-400 focus:border-purple-400 sm:text-sm rounded-md",
                onchange: move |e: Event<FormData>| props.onchange.call(e.value()),
                disabled: props.disabled,

                for (val, label) in props.options.iter() {
                    option {
                        value: "{val}",
                        "{label}"
                    }
                }
            }
        }
    }
}
