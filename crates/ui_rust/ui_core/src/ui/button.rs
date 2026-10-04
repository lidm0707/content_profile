use dioxus::prelude::*;

/// Shared props for [Button].
#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    /// Visual style variant.
    variant: ButtonVariant,
    /// Disable the button and dim it.
    disabled: bool,
    /// Click handler.
    onclick: EventHandler<MouseEvent>,
    children: Element,
}

/// Button styles keyed by intent — the single source for the palette so
/// pages stop repeating long Tailwind strings.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Primary action — orange fill.
    Primary,
    /// Secondary action — purple outline.
    Secondary,
    /// Quiet action — neutral outline.
    Ghost,
}

impl ButtonVariant {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => {
                "inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-orange-500 text-base font-medium text-white hover:bg-orange-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-orange-400 sm:text-sm"
            }
            Self::Secondary => {
                "inline-flex justify-center rounded-md border border-purple-400/40 shadow-sm px-4 py-2 bg-purple-500/10 text-base font-medium text-purple-200 hover:bg-purple-500/20 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-400 sm:text-sm"
            }
            Self::Ghost => {
                "inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-transparent text-base font-medium text-gray-700 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-purple-400 sm:text-sm"
            }
        }
    }
}

/// Shared button across pages.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "{props.variant.class()}",
            disabled: props.disabled,
            onclick: move |e| props.onclick.call(e),
            {props.children}
        }
    }
}
