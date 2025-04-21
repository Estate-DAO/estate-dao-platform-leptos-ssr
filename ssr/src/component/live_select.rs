use crate::log;
use crate::state::GlobalStateForLeptos;
use leptos::ev::MouseEvent;
use leptos::html::{Div, Input};
use leptos::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, KeyboardEvent, Node};

// impl<T: Clone + PartialEq + 'static> GlobalStateForLeptos for LiveSelectState<T> {}

// Global state management for LiveSelect
pub struct LiveSelectState<T>
where
    T: Clone + PartialEq + 'static,
{
    // Core state
    search_text: RwSignal<String>,
    is_open: RwSignal<bool>,
    active_index: RwSignal<usize>,
    selected_value: Signal<Option<T>>,
    set_value: Callback<T>,

    // References
    input_ref: NodeRef<Input>,
    dropdown_ref: NodeRef<Div>,
    container_ref: NodeRef<Div>,

    // Options and callbacks
    options: MaybeSignal<Vec<T>>,
    label_fn: Callback<T, String>,
    value_fn: Callback<T, String>,

    // Configuration
    debug: bool,
}

impl<T> LiveSelectState<T>
where
    T: Clone + PartialEq + 'static,
{
    fn get() -> Self {
        expect_context()
        // let this = use_context::<Self>();
        // match this {
        //     Some(x) => x,
        //     None => {
        //         Self::set_global();
        //     }
        // }
    }

    pub fn new(
        options: MaybeSignal<Vec<T>>,
        value: Signal<Option<T>>,
        set_value: Callback<T>,
        label_fn: Callback<T, String>,
        value_fn: Callback<T, String>,
        debug: bool,
    ) -> Self {
        let this = Self {
            search_text: create_rw_signal(String::new()),
            is_open: create_rw_signal(false),
            active_index: create_rw_signal(0),
            selected_value: value,
            set_value,
            input_ref: create_node_ref(),
            dropdown_ref: create_node_ref(),
            container_ref: create_node_ref(),
            options,
            label_fn,
            value_fn,
            debug,
        };
        let cloned = this.clone();
        // Set up event handlers and effects
        this.setup_keyboard_navigation();
        this.setup_outside_click_detection();

        // set this up for all components downstream?
        provide_context(this);
        cloned
    }

    // Getter methods
    pub fn get_search_text() -> Signal<String> {
        let this = Self::get();
        this.search_text.into()
    }

    pub fn get_is_open() -> Signal<bool> {
        let this = Self::get();
        this.is_open.into()
    }

    pub fn get_active_index() -> Signal<usize> {
        let this = Self::get();
        this.active_index.into()
    }

    pub fn get_selected_value() -> Signal<Option<T>> {
        let this = Self::get();
        this.selected_value
    }

    pub fn get_input_ref() -> NodeRef<Input> {
        let this = Self::get();
        this.input_ref
    }

    pub fn get_dropdown_ref() -> NodeRef<Div> {
        let this = Self::get();
        this.dropdown_ref
    }

    pub fn get_container_ref(&self) -> NodeRef<Div> {
        let this = self;
        // let this  = Self::get();
        this.container_ref
    }

    // Action methods
    pub fn set_search_text(text: String) {
        let this = Self::get();
        this.search_text.set(text);
    }

    pub fn open_dropdown() {
        let this = Self::get();
        this.is_open.set(true);
    }

    pub fn close_dropdown() {
        let this = Self::get();
        this.is_open.set(false);
    }

    pub fn toggle_dropdown() {
        let this = Self::get();
        this.is_open.update(|v| *v = !*v);
    }

    pub fn set_active_index(index: usize) {
        let this = Self::get();
        this.active_index.set(index);
    }

    pub fn select_option(opt: T) {
        let this = Self::get();
        Callable::call(&this.set_value, opt);
        Self::close_dropdown();
        Self::set_search_text(String::new());

        // Focus the input after selection
        if let Some(input) = this.input_ref.get() {
            let _ = input.focus();
        }
    }

    // Event handlers
    pub fn handle_input(ev: Event) {
        let value = event_target_value(&ev);
        let this = Self::get();
        Self::set_search_text(value);
        Self::open_dropdown();
        Self::set_active_index(0);
    }

    pub fn handle_focus(_: Event) {
        Self::open_dropdown();
    }

    pub fn handle_toggle_click(ev: MouseEvent) {
        ev.prevent_default();
        ev.stop_propagation();
        Self::toggle_dropdown();

        // Focus input when opening
        if !Self::get_is_open().get() {
            if let Some(input) = Self::get_input_ref().get() {
                let _ = input.focus();
            }
        }
    }

    // Computed values
    pub fn filtered_options() -> Memo<Vec<T>> {
        let this = Self::get();
        let options = this.options;
        let search_text = this.search_text;
        let label_fn = this.label_fn.clone();

        create_memo(move |_| {
            let search = search_text.get().to_lowercase();
            if search.is_empty() {
                return options.get();
            }

            options
                .get()
                .into_iter()
                .filter(|opt| label_fn(opt.clone()).to_lowercase().contains(&search))
                .collect::<Vec<T>>()
        })
    }

    // Set up keyboard navigation
    pub fn setup_keyboard_navigation(&self) {
        let this = self;
        // let this = Self::get();
        let input_ref = this.input_ref;
        let active_index = this.active_index;
        let is_open = this.is_open;
        let dropdown_ref = this.dropdown_ref;
        let state = this.clone();

        create_effect(move |_| {
            if let Some(input) = input_ref.get_untracked() {
                let handler = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
                    match ev.key().as_str() {
                        "ArrowDown" => {
                            ev.prevent_default();
                            if !is_open.get() {
                                Self::open_dropdown();
                            } else {
                                let filtered = Self::filtered_options().get();
                                let max = filtered.len().saturating_sub(1);
                                let next = (active_index.get() + 1).min(max);
                                Self::set_active_index(next);

                                // Scroll to view if needed
                                if let Some(dropdown) = dropdown_ref.get() {
                                    if let Some(active_item) = dropdown
                                        .query_selector(&format!("[data-index=\"{}\"]", next))
                                        .ok()
                                        .flatten()
                                    {
                                        let _ = active_item.scroll_into_view();
                                    }
                                }
                            }
                        }
                        "ArrowUp" => {
                            ev.prevent_default();
                            if is_open.get() {
                                let prev = active_index.get().saturating_sub(1);
                                Self::set_active_index(prev);

                                // Scroll to view if needed
                                if let Some(dropdown) = dropdown_ref.get() {
                                    if let Some(active_item) = dropdown
                                        .query_selector(&format!("[data-index=\"{}\"]", prev))
                                        .ok()
                                        .flatten()
                                    {
                                        let _ = active_item.scroll_into_view();
                                    }
                                }
                            }
                        }
                        "Escape" => {
                            ev.prevent_default();
                            Self::close_dropdown();
                        }
                        "Enter" => {
                            if Self::get_is_open().get() {
                                ev.prevent_default();
                                let filtered = Self::filtered_options().get();
                                if !filtered.is_empty() {
                                    let index = active_index.get().min(filtered.len() - 1);
                                    Self::select_option(filtered[index].clone());
                                }
                            }
                        }
                        _ => {}
                    }
                }) as Box<dyn FnMut(KeyboardEvent)>);

                // Add event listener
                input
                    .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
                    .expect("should add keydown event listener");

                // Clean up on drop
                on_cleanup(move || {
                    if let Some(input_elem) = input_ref.get() {
                        input_elem
                            .remove_event_listener_with_callback(
                                "keydown",
                                handler.as_ref().unchecked_ref(),
                            )
                            .expect("should remove keydown event listener");
                    }
                    // Handler will be dropped here
                });
            }
        });
    }

    // Set up outside click detection
    pub fn setup_outside_click_detection(&self) {
        let this = self;
        // let this = Self::get();
        let container_ref = this.container_ref;
        let is_open = this.is_open;
        let debug = this.debug;
        let state = this.clone();

        create_effect(move |_| {
            if is_open.get() {
                let container = container_ref.get_untracked();
                if let Some(container) = container {
                    let container_clone = container.clone();

                    // Create the event handler for outside clicks
                    let handler = Closure::wrap(Box::new(move |event: Event| {
                        let target = event.target();
                        let target_element = target
                            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                            .map(|e| e.dyn_into::<Node>().unwrap());

                        // Check if the click was outside our container
                        let outside_click = match target_element {
                            Some(element) => !container_clone.contains(Some(&element)),
                            None => true,
                        };

                        if outside_click {
                            if debug {
                                log!("Clicked outside the LiveSelect");
                            }
                            Self::close_dropdown();
                        }
                    }) as Box<dyn FnMut(Event)>);

                    // Add event listener to document
                    let document = web_sys::window()
                        .expect("window should exist")
                        .document()
                        .expect("document should exist");

                    document
                        .add_event_listener_with_callback("click", handler.as_ref().unchecked_ref())
                        .expect("should add event listener");

                    // Store the handler in a closure that will be called during cleanup
                    on_cleanup(move || {
                        document
                            .remove_event_listener_with_callback(
                                "click",
                                handler.as_ref().unchecked_ref(),
                            )
                            .expect("should remove event listener");
                        // Handler will be dropped here, automatically cleaning up the closure
                    });
                }
            }
        });
    }

    // Text highlighting function
    pub fn highlight_text(text: &str, search: &str) -> Vec<View> {
        let this = Self::get();
        if search.is_empty() {
            return vec![text.to_string().into_view()];
        }

        let search_lower = search.to_lowercase();
        let text_lower = text.to_lowercase();

        let mut parts = Vec::new();
        let mut start_index = 0;

        while let Some(match_start) = text_lower[start_index..].find(&search_lower) {
            let absolute_start = start_index + match_start;
            let match_end = absolute_start + search_lower.len();

            // Add text before match
            if absolute_start > start_index {
                parts.push(text[start_index..absolute_start].to_string().into_view());
            }

            // Add highlighted text
            parts.push(view! {
                <strong class="bg-yellow-200">{text[absolute_start..match_end].to_string()}</strong>
            }.into_view());

            start_index = match_end;
        }

        // Add remaining text after matches
        if start_index < text.len() {
            parts.push(text[start_index..].to_string().into_view());
        }

        if parts.is_empty() {
            parts.push(text.to_string().into_view());
        }

        parts
    }
}

// Add Clone implementation for LiveSelectState
impl<T> Clone for LiveSelectState<T>
where
    T: Clone + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        Self {
            search_text: self.search_text,
            is_open: self.is_open,
            active_index: self.active_index,
            selected_value: self.selected_value,
            set_value: self.set_value.clone(),
            input_ref: self.input_ref,
            dropdown_ref: self.dropdown_ref,
            container_ref: self.container_ref,
            options: self.options.clone(),
            label_fn: self.label_fn.clone(),
            value_fn: self.value_fn.clone(),
            debug: self.debug,
        }
    }
}

type LSS<T> = LiveSelectState<T>;

#[component]
pub fn LiveSelect<T>(
    #[prop(optional)] options: MaybeSignal<Vec<T>>,
    value: Signal<Option<T>>,
    set_value: Callback<T>,
    label_fn: Callback<T, String>,
    value_fn: Callback<T, String>,
    #[prop(optional)] placeholder: MaybeSignal<String>,
    #[prop(optional)] id: MaybeSignal<String>,
    #[prop(optional)] class: MaybeSignal<String>,
    #[prop(optional)] debug: bool,
) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
{
    // Create a state management instance
    let state = LiveSelectState::new(options, value, set_value, label_fn, value_fn, debug);

    // Get computed/derived values
    let filtered_options = LSS::<T>::filtered_options();

    view! {
        <div
            _ref=state.get_container_ref()
            class=move || format!("relative w-full {}", class.get())
            id=move || id.get()
        >
            <div class="relative">
                <input
                    type="text"
                    class="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-300"
                    placeholder=move || placeholder.get()
                    value=move || LSS::<T>::get_search_text().get()
                    on:input=move |ev| LSS::<T>::handle_input(ev)
                    on:focus=move |ev| LSS::<T>::handle_focus(ev)
                    _ref=LSS::<T>::get_input_ref()
                    aria-expanded=move || LSS::<T>::get_is_open().get().to_string()
                    aria-autocomplete="list"
                    role="combobox"></input>
                <button
                    type="button"
                    class="absolute inset-y-0 right-0 flex items-center pr-2"
                    on:click=move |ev| LSS::<T>::handle_toggle_click(ev)
                    aria-label="Toggle dropdown"
                >
                    <svg
                        class="w-5 h-5 text-gray-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d=move || if LSS::<T>::get_is_open().get() {
                                "M5 15l7-7 7 7"
                            } else {
                                "M19 9l-7 7-7-7"
                            }
                        />
                    </svg>
                </button>
            </div>

            <Show when=move || LSS::<T>::get_is_open().get()>
                <div
                    class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-md shadow-lg max-h-60 overflow-auto"
                    _ref=LSS::<T>::get_dropdown_ref()
                    role="listbox"
                >
                    <Show
                        when=move || !filtered_options.get().is_empty()
                        fallback=move || view! {
                            <div class="px-3 py-2 text-gray-500">No results found</div>
                        }
                    >
                        <For
                            each=move || filtered_options.get()
                            key=move |opt| LSS::<T>::value_fn(opt.clone())
                            let:item
                        >
                            {move || {
                                let opt = item.clone();
                                let opt_for_select = opt.clone();
                                let opt_for_compare = opt.clone();
                                let index = filtered_options.get()
                                    .iter()
                                    .position(|x| x == &opt)
                                    .unwrap_or(0);
                                let is_active = create_memo(move |_| LSS::<T>::get_active_index().get() == index);
                                let label = LSS::<T>::label_fn(opt.clone());
                                let state_clone = state.clone();

                                view! {
                                    <div
                                        class=move || format!(
                                            "px-3 py-2 cursor-pointer hover:bg-gray-100 {} {}",
                                            if is_active.get() { "bg-blue-50" } else { "" },
                                            if LSS::<T>::get_selected_value().get().as_ref() == Some(&opt_for_compare) { "font-medium" } else { "" }
                                        )
                                        on:click=move |_| state_clone.select_option(opt_for_select.clone())
                                        data-index=index.to_string()
                                        role="option"
                                        aria-selected=move || (LSS::<T>::get_selected_value().get().as_ref() == Some(&opt)).to_string()
                                    >
                                        {LSS::<T>::highlight_text(&label, &LSS::<T>::get_search_text().get())}
                                    </div>
                                }
                            }}
                        </For>
                    </Show>
                </div>
            </Show>
        </div>
    }
}

// Example usage
#[component]
pub fn LiveSelectExample() -> impl IntoView {
    // Example data
    let options = vec![
        "Afghanistan".to_string(),
        "Albania".to_string(),
        "Algeria".to_string(),
        "Andorra".to_string(),
        "Angola".to_string(),
        "Antigua and Barbuda".to_string(),
        "Argentina".to_string(),
        "Armenia".to_string(),
        "Australia".to_string(),
        "Austria".to_string(),
        "Azerbaijan".to_string(),
        "Bahamas".to_string(),
        "Bahrain".to_string(),
        "Bangladesh".to_string(),
        "Barbados".to_string(),
        "Belarus".to_string(),
        "Belgium".to_string(),
        "Belize".to_string(),
        "Benin".to_string(),
        "Bhutan".to_string(),
    ];

    let options_sig = create_rw_signal(options);
    let (selected_country, set_selected_country) = create_signal(None::<String>);

    view! {
        <div class="p-4 max-w-md mx-auto">
            <h2 class="text-lg font-semibold mb-2">Select a Country</h2>

            <LiveSelect
                options=options_sig.into()
                value=Signal::derive(move || selected_country.get())
                set_value=Callback::new(move |val: String| set_selected_country.set(Some(val)))
                label_fn=Callback::new(|s: String| s)
                value_fn=Callback::new(|s: String| s)
                placeholder="Search countries...".into()
                id="country-select".into()
                class="mb-4".into()
                debug=false
            />

            <div class="mt-4">
                <Show
                    when=move || selected_country.get().is_some()
                    fallback=|| view! { <p>No country selected</p> }
                >
                    <p>Selected country: {move || selected_country.get().unwrap_or_default()}</p>
                </Show>
            </div>
        </div>
    }
}
