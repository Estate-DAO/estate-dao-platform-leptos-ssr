// use crate::log;
// use leptos::html::{Div, Input};
// use leptos::*;
// use leptos_icons::*;
// use leptos_use::{
//     on_click_outside, use_event_listener, use_event_listener_with_options, use_window,
// };
// use wasm_bindgen::closure::Closure;
// use wasm_bindgen::JsCast;
// use web_sys::wasm_bindgen::JsValue;
// use web_sys::EventTarget;
// use web_sys::{Event, FocusEvent, KeyboardEvent, MouseEvent};
// // use crate::utils::string_ext::StringExt;
// // use std::fmt::Debug;

// /// Select hook implemented as a struct to manage dropdown state and behavior
// pub struct SelectHook<T>
// where
//     T: Clone + PartialEq + 'static,
// {
//     // State signals
//     search_text: RwSignal<String>,
//     is_open: RwSignal<bool>,
//     active_index: RwSignal<usize>,
//     selected_value: Signal<Option<T>>,

//     // Options and callbacks
//     options: MaybeSignal<Vec<T>>,
//     set_value: Callback<T>,
//     label_fn: Callback<T, String>,
//     value_fn: Callback<T, String>,

//     // DOM refs
//     input_ref: NodeRef<Input>,
//     dropdown_ref: NodeRef<Div>,
//     container_ref: NodeRef<Div>,

//     // Computed values
//     filtered_options: Memo<Vec<T>>,
// }

// /// Struct to hold all state and handlers
// pub struct SelectState<T>
// where
//     T: Clone + PartialEq + 'static,
// {
//     // State signals
//     pub search_text: Signal<String>,
//     pub is_open: Signal<bool>,
//     pub active_index: Signal<usize>,
//     pub selected_value: Signal<Option<T>>,

//     // DOM refs
//     pub input_ref: NodeRef<Input>,
//     pub dropdown_ref: NodeRef<Div>,
//     pub container_ref: NodeRef<Div>,

//     // Computed values
//     pub filtered_options: Memo<Vec<T>>,

//     // Actions
//     pub set_search_text: Callback<String>,
//     pub open_dropdown: Callback<()>,
//     pub close_dropdown: Callback<()>,
//     pub toggle_dropdown: Callback<()>,
//     pub set_active_index: Callback<usize>,
//     pub select_option: Callback<T>,

//     // Event handlers
//     pub handle_input: Callback<Event>,
//     pub handle_focus: Callback<FocusEvent>,
//     pub handle_toggle_click: Callback<MouseEvent>,
//     pub handle_key_down: Callback<KeyboardEvent>,

//     // Utilities
//     pub highlight_text: Callback<(String, String), Vec<View>>,
// }

// impl<T> SelectHook<T>
// where
//     T: Clone + PartialEq + 'static,
// {
//     /// Create a new select hook with the provided options and callbacks
//     pub fn new(
//         options: MaybeSignal<Vec<T>>,
//         value: Signal<Option<T>>,
//         set_value: Callback<T>,
//         label_fn: Callback<T, String>,
//         value_fn: Callback<T, String>,
//     ) -> Self {
//         // Initialize search text with current value's label if available
//         let initial_text = value.get().map_or(String::new(), |v| {
//             leptos::Callable::call(&label_fn, v.clone())
//         });
//         let search_text = create_rw_signal(initial_text);
//         let is_open = create_rw_signal(false);
//         let active_index = create_rw_signal(0);

//         let input_ref = create_node_ref::<Input>();
//         let dropdown_ref = create_node_ref::<Div>();
//         let container_ref = create_node_ref::<Div>();

//         // Create effect to update search text when value changes
//         create_effect(move |_| {
//             let current_text = value.get().map_or(String::new(), |v| {
//                 leptos::Callable::call(&label_fn, v.clone())
//             });
//             if !is_open.get() {
//                 // Only update when dropdown is closed to avoid interfering with user typing
//                 search_text.set(current_text);
//             }
//         });

//         let options_clone = options.clone();
//         // Create filtered options memo
//         let filtered_options = create_memo(move |_| {
//             let search = search_text.get().to_lowercase();
//             let all_options = options.get();

//             if search.is_empty() {
//                 return all_options;
//             }

//             all_options
//                 .into_iter()
//                 .filter(|opt| {
//                     leptos::Callable::call(&label_fn, opt.clone())
//                         .to_lowercase()
//                         .contains(&search)
//                 })
//                 .collect::<Vec<T>>()
//         });

//         Self {
//             search_text,
//             is_open,
//             active_index,
//             selected_value: value,
//             options: options_clone,
//             set_value,
//             label_fn,
//             value_fn,
//             input_ref,
//             dropdown_ref,
//             container_ref,
//             filtered_options,
//         }
//     }

//     /// Set the search text
//     pub fn set_search_text(&self) -> Callback<String> {
//         let search_text = self.search_text;
//         Callback::new(move |text| {
//             search_text.set(text);
//         })
//     }

//     /// Open the dropdown
//     pub fn open_dropdown(&self) -> Callback<()> {
//         let is_open = self.is_open;
//         Callback::new(move |_| {
//             is_open.set(true);
//         })
//     }

//     /// Close the dropdown
//     pub fn close_dropdown(&self) -> Callback<()> {
//         let is_open = self.is_open;
//         Callback::new(move |_| {
//             is_open.set(false);
//         })
//     }

//     /// Toggle the dropdown
//     pub fn toggle_dropdown(&self) -> Callback<()> {
//         let is_open = self.is_open;
//         Callback::new(move |_| {
//             is_open.update(|open| *open = !*open);
//         })
//     }

//     /// Set the active index
//     pub fn set_active_index(&self) -> Callback<usize> {
//         let active_index = self.active_index;
//         Callback::new(move |idx| {
//             active_index.set(idx);
//         })
//     }

//     /// Select an option
//     pub fn select_option(&self) -> Callback<T> {
//         let set_value = self.set_value.clone();
//         let close_dropdown = self.close_dropdown();
//         let set_search_text = self.set_search_text();
//         let input_ref = self.input_ref;
//         let label_fn = self.label_fn.clone();

//         Callback::new(move |opt: T| {
//             let label = leptos::Callable::call(&label_fn, opt.clone());
//             leptos::Callable::call(&set_value, opt);
//             leptos::Callable::call(&close_dropdown, ());
//             leptos::Callable::call(&set_search_text, label.clone());

//             // Focus the input after selection
//             if let Some(input) = input_ref.get() {
//                 let _ = input.focus();
//             }
//         })
//     }

//     /// Handle input events
//     pub fn handle_input(&self) -> Callback<Event> {
//         let set_search_text = self.set_search_text();
//         let open_dropdown = self.open_dropdown();
//         let set_active_index = self.set_active_index();

//         Callback::new(move |ev: Event| {
//             let value = event_target_value(&ev);
//             leptos::Callable::call(&set_search_text, value);
//             leptos::Callable::call(&open_dropdown, ());
//             leptos::Callable::call(&set_active_index, 0);
//         })
//     }

//     /// Handle focus events
//     pub fn handle_focus(&self) -> Callback<FocusEvent> {
//         let search_text = self.search_text;
//         let selected_value = self.selected_value;
//         let label_fn = self.label_fn.clone();

//         Callback::new(move |_: FocusEvent| {
//             // Only set search text if it's empty and there's a selected value
//             if search_text.get().is_empty() {
//                 if let Some(value) = selected_value.get() {
//                     let label = leptos::Callable::call(&label_fn, value);
//                     search_text.set(label);
//                 }
//             }
//         })
//     }

//     /// Handle click events for toggling
//     pub fn handle_toggle_click(&self) -> Callback<MouseEvent> {
//         let toggle_dropdown = self.toggle_dropdown();
//         let input_ref = self.input_ref;

//         Callback::new(move |ev: MouseEvent| {
//             ev.stop_propagation();
//             leptos::Callable::call(&toggle_dropdown, ());

//             // Focus the input when toggling
//             if let Some(input) = input_ref.get() {
//                 let _ = input.focus();
//             }
//         })
//     }

//     /// Handle keyboard events
//     pub fn handle_key_down(&self) -> Callback<KeyboardEvent> {
//         let is_open = self.is_open;
//         let active_index = self.active_index;
//         let filtered_options = self.filtered_options;
//         let open_dropdown = self.open_dropdown();
//         let close_dropdown = self.close_dropdown();
//         let set_active_index = self.set_active_index();
//         let select_option = self.select_option();

//         Callback::new(move |ev: KeyboardEvent| match ev.key().as_str() {
//             "ArrowDown" => {
//                 ev.prevent_default();

//                 if !is_open.get() {
//                     leptos::Callable::call(&open_dropdown, ());
//                     return;
//                 }

//                 let filtered = filtered_options.get();
//                 if filtered.is_empty() {
//                     return;
//                 }

//                 let current = active_index.get();
//                 let next = if current >= filtered.len() - 1 {
//                     0
//                 } else {
//                     current + 1
//                 };

//                 leptos::Callable::call(&set_active_index, next);
//             }
//             "ArrowUp" => {
//                 ev.prevent_default();

//                 if !is_open.get() {
//                     leptos::Callable::call(&open_dropdown, ());
//                     return;
//                 }

//                 let filtered = filtered_options.get();
//                 if filtered.is_empty() {
//                     return;
//                 }

//                 let current = active_index.get();
//                 let next = if current == 0 {
//                     filtered.len() - 1
//                 } else {
//                     current - 1
//                 };

//                 leptos::Callable::call(&set_active_index, next);
//             }
//             "Enter" => {
//                 if is_open.get() {
//                     ev.prevent_default();

//                     let filtered = filtered_options.get();
//                     let current = active_index.get();

//                     if !filtered.is_empty() && current < filtered.len() {
//                         leptos::Callable::call(&select_option, filtered[current].clone());
//                     }
//                 }
//             }
//             "Escape" => {
//                 if is_open.get() {
//                     ev.prevent_default();
//                     leptos::Callable::call(&close_dropdown, ());
//                 }
//             }
//             "Tab" => {
//                 if is_open.get() {
//                     leptos::Callable::call(&close_dropdown, ());
//                 }
//             }
//             "Home" => {
//                 if is_open.get() {
//                     ev.prevent_default();
//                     leptos::Callable::call(&set_active_index, 0);
//                 }
//             }
//             "End" => {
//                 if is_open.get() {
//                     ev.prevent_default();
//                     let filtered = filtered_options.get();
//                     if !filtered.is_empty() {
//                         leptos::Callable::call(&set_active_index, filtered.len() - 1);
//                     }
//                 }
//             }
//             _ => {}
//         })
//     }

//     /// Highlight matched text in search results
//     pub fn highlight_text(&self) -> Callback<(String, String), Vec<View>> {
//         Callback::new(|(text, search): (String, String)| {
//             if search.is_empty() {
//                 return vec![text.into_view()];
//             }

//             let mut result = Vec::new();
//             let search_lower = search.to_lowercase();
//             let mut remaining = text.clone();

//             while let Some(start_idx) = remaining.to_lowercase().find(&search_lower) {
//                 if start_idx > 0 {
//                     result.push(remaining[..start_idx].to_string().into_view());
//                 }

//                 let end_idx = start_idx + search.len();
//                 result.push(
//                     view! {
//                         <span class="font-bold text-blue-600">
//                             {remaining[start_idx..end_idx].to_string()}
//                         </span>
//                     }
//                     .into_view(),
//                 );

//                 remaining = remaining[end_idx..].to_string();
//             }

//             if !remaining.is_empty() {
//                 result.push(remaining.into_view());
//             }

//             result
//         })
//     }

//     /// Initialize click outside handling
//     pub fn init_click_outside(&self) {
//         let container = self.container_ref;
//         let close_dropdown = self.close_dropdown();
//         let is_open = self.is_open;

//         let close_fn = move |_| {
//             if is_open.get() {
//                 leptos::Callable::call(&close_dropdown, ());
//             }
//         };

//         if let Some(_) = container.get() {
//             // Use capture phase to ensure our handler runs before parent handlers
//             let _ = on_click_outside(container, close_fn);

//             // Also listen for clicks on the document to handle cases where the dropdown is open
//             // but the click is outside the container
//             if let Some(window) = web_sys::window() {
//                 let document = window.document().expect("no global `document` exists");

//                 let closure = Closure::<dyn Fn(web_sys::Event)>::new(move |ev: web_sys::Event| {
//                     if is_open.get() {
//                         if let Some(target) = ev.target() {
//                             if let Ok(element) = target.dyn_into::<web_sys::Element>() {
//                                 if container
//                                     .get()
//                                     .map_or(false, |c| !c.contains(Some(&element)))
//                                 {
//                                     leptos::Callable::call(&close_dropdown, ());
//                                     ev.stop_propagation();
//                                 }
//                             }
//                         }
//                     }
//                 });

//                 let _ = document
//                     .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref());

//                 // Store the closure so it's not dropped
//                 closure.forget();
//             }
//         }
//     }

//     /// Get all the hooks state and handlers for use in components
//     pub fn use_state(&self) -> SelectState<T> {
//         SelectState {
//             search_text: self.search_text.into(),
//             is_open: self.is_open.into(),
//             active_index: self.active_index.into(),
//             selected_value: self.selected_value,

//             input_ref: self.input_ref,
//             dropdown_ref: self.dropdown_ref,
//             container_ref: self.container_ref,

//             filtered_options: self.filtered_options,

//             set_search_text: self.set_search_text(),
//             open_dropdown: self.open_dropdown(),
//             close_dropdown: self.close_dropdown(),
//             toggle_dropdown: self.toggle_dropdown(),
//             set_active_index: self.set_active_index(),
//             select_option: self.select_option(),

//             handle_input: self.handle_input(),
//             handle_focus: self.handle_focus(),
//             handle_toggle_click: self.handle_toggle_click(),
//             handle_key_down: self.handle_key_down(),

//             highlight_text: self.highlight_text(),
//         }
//     }
// }

// #[component]
// fn LiveSelectOption<T>(
//     option: T,
//     #[prop(into)] active: Signal<bool>,
//     on_select: Callback<T>,
//     label_fn: Callback<T, String>,
//     #[prop(into)] search_text: Signal<String>,
//     highlight_text: Callback<(String, String), Vec<View>>,
//     index: usize,
// ) -> impl IntoView
// where
//     T: Clone + PartialEq + 'static,
// {
//     let option_id = format!("option-{index}");
//     let label = leptos::Callable::call(&label_fn, option.clone());

//     view! {
//         <li
//             id=option_id
//             class=move || {
//                 let base = "list-none px-4 py-2.5 cursor-pointer hover:bg-blue-50/50";
//                 if active.get() {
//                     format!("{} bg-blue-50 text-blue-500", base)
//                 } else {
//                     base.to_string()
//                 }
//             }
//             role="option"
//             aria-selected=move || active.get().to_string()
//             on:click=move |_| leptos::Callable::call(&on_select, option.clone())
//             on:mouseenter=move |_| { }
//         >
//             {move || {
//                 leptos::Callable::call(&highlight_text, (label.clone(), search_text.get()))
//             }}
//         </li>
//     }
// }

// #[component]
// fn LiveSelectDropdown<T>(
//     #[prop(into)] filtered_options: Signal<Vec<T>>,
//     #[prop(into)] active_index: Signal<usize>,
//     select_option: Callback<T>,
//     label_fn: Callback<T, String>,
//     #[prop(into)] search_text: Signal<String>,
//     highlight_text: Callback<(String, String), Vec<View>>,
//     #[prop(into)] dropdown_id: String,
//     dropdown_ref: NodeRef<Div>,
//     #[prop(optional, into)] dropdown_class: MaybeSignal<String>,
// ) -> impl IntoView
// where
//     T: Clone + PartialEq + 'static,
// {
//     view! {
//         <div
//             _ref=dropdown_ref
//             id=dropdown_id
//             class=move || format!("fixed z-[200] left-0 top-[33vh] w-full h-[67vh] md:top-16 md:w-1/3 md:h-[40vh] bg-white shadow-lg rounded-lg overflow-y-auto {}", dropdown_class.get())
//             role="listbox"
//         >
//             <div class="sticky top-0 bg-white border-b border-gray-200 p-4 flex items-center md:hidden">
//                 <h2 class="text-lg font-semibold flex-1">Select Destination</h2>
//                 <button
//                     class="text-gray-500 hover:text-gray-700"
//                     aria-label="Close"
//                     on:click=move |_| {
//                         // Close dropdown functionality will be handled by click outside
//                     }
//                 >
//                 "x"
//                 </button>
//             </div>
//             <Suspense fallback=move || view! { <div>Loading...</div> }>
//             {move || {
//                 let options = filtered_options.get();
//                 let active_idx = active_index.get();

//                 options.iter().enumerate().map(|(i, opt)| {
//                     view! {
//                         <LiveSelectOption
//                             option=opt.clone()
//                             active=Signal::derive(move || i == active_idx)
//                             on_select=select_option.clone()
//                             label_fn=label_fn.clone()
//                             search_text=search_text
//                             highlight_text=highlight_text.clone()
//                             index=i
//                         />
//                     }
//                 }).collect_view()
//             }}
//             </Suspense>

//             <Show when=move || filtered_options.get().is_empty()>
//             {move || {
//                 view! {
//                     <li class="list-none px-4 py-2.5 text-gray-400 text-[15px]">
//                         "No results found"
//                     </li>
//                 }
//             }}
//             </Show>
//         </div>
//     }
// }

// #[component]
// fn LiveSelectInput(
//     #[prop(into)] search_text: Signal<String>,
//     #[prop(into)] is_open: Signal<bool>,
//     #[prop(into)] placeholder: MaybeSignal<String>,
//     handle_input: Callback<Event>,
//     handle_focus: Callback<FocusEvent>,
//     handle_key_down: Callback<KeyboardEvent>,
//     handle_toggle_click: Callback<MouseEvent>,
//     #[prop(into)] id: String,
//     #[prop(into)] dropdown_id: String,
//     #[prop(into)] active_index: Signal<usize>,
//     input_ref: NodeRef<Input>,
//     #[prop(optional, into)] input_class: MaybeSignal<String>,
// ) -> impl IntoView {
//     let input_placeholder = create_memo(move |_| placeholder.get());
//     let icon = create_memo(move |_| {
//         if is_open.get() {
//             icondata::BiChevronUpRegular
//         } else {
//             icondata::BiChevronDownRegular
//         }
//     });

//     view! {
//         <div class="relative flex items-center w-full h-full z-[103] px-6" on:focus=handle_focus on:click=move |ev| ev.stop_propagation()>
//         <div>
//             <input
//                 type="text"
//                 _ref=input_ref
//                 id=id.clone()
//                 class=move || format!("live-select-input {}", input_class.get())
//                 placeholder=input_placeholder
//                 autocomplete="off"
//                 aria-autocomplete="list"
//                 aria-controls=dropdown_id.clone()
//                 aria-expanded=move || is_open.get().to_string()
//                 aria-activedescendant=move || format!("{}-option-{}", id, active_index.get())
//                 role="combobox"
//                 value=search_text
//                 on:input=move |ev| leptos::Callable::call(&handle_input, ev)
//                 on:focus=move |ev| leptos::Callable::call(&handle_focus, ev)
//                 on:keydown=move |ev| leptos::Callable::call(&handle_key_down, ev)
//                 on:click=move |ev| leptos::Callable::call(&handle_toggle_click, ev)
//             />
//         </div>


//         <div
//         class="ml-auto right-3 p-1 text-xl focus:outline-none"
//         tabindex="-1"
//         aria-label="Toggle dropdown"
//         on:click=handle_toggle_click
//     >
//         {move || {
//             view! {
//                 <Icon icon=icon class="w-4 h-4" />
//             }
//         }}
//     </div>
//     </div>

//     }
// }

// #[component]
// pub fn LiveSelect<T>(
//     #[prop(optional, into)] options: MaybeSignal<Vec<T>>,
//     #[prop(into)] value: Signal<Option<T>>,
//     set_value: Callback<T>,
//     label_fn: Callback<T, String>,
//     value_fn: Callback<T, String>,
//     #[prop(optional, into)] placeholder: MaybeSignal<String>,
//     #[prop(optional, into)] id: MaybeSignal<String>,
//     #[prop(optional, into)] class: MaybeSignal<String>,
//     #[prop(optional, into)] input_class: MaybeSignal<String>,
//     #[prop(optional, into)] dropdown_class: MaybeSignal<String>,
//     #[prop(optional)] debug: bool,
// ) -> impl IntoView
// where
//     T: Clone + PartialEq + 'static,
// {
//     let id_value = id.get_untracked();
//     let dropdown_id = format!("{}-dropdown", id_value);

//     // Create the hook
//     let hook = SelectHook::new(options, value, set_value, label_fn, value_fn);

//     // Get all the state and handlers as a struct
//     let state = hook.use_state();

//     // Initialize click outside handler
//     hook.init_click_outside();

//     // Set initial search text based on selected value
//     create_effect(move |_| {
//         if let Some(opt) = state.selected_value.get() {
//             let selected_label = leptos::Callable::call(&hook.label_fn, opt);
//             if state.search_text.get().is_empty() {
//                 leptos::Callable::call(&state.set_search_text, selected_label);
//             }
//         }
//     });

//     // Close dropdown when options change to empty
//     create_effect(move |_| {
//         let options_list = hook.options.get();
//         if options_list.is_empty() && state.is_open.get() {
//             leptos::Callable::call(&state.close_dropdown, ());
//         }
//     });

//     // Debug mode
//     if debug {
//         create_effect(move |_| {
//             log!("Search text: {}", state.search_text.get());
//             log!("Is open: {}", state.is_open.get());
//             log!("Active index: {}", state.active_index.get());
//             log!(
//                 "Selected value: {:?}",
//                 state
//                     .selected_value
//                     .get()
//                     .map(|v| leptos::Callable::call(&hook.label_fn, v))
//             );
//             log!("Filtered options: {:?}", state.filtered_options.get().len());
//         });
//     }

//     view! {
//         <div
//             class=move || format!("relative z-[99] {}", class.get())
//         >
//             <LiveSelectInput
//                 search_text=state.search_text
//                 is_open=state.is_open
//                 placeholder=placeholder
//                 handle_input=state.handle_input
//                 handle_focus=state.handle_focus
//                 handle_key_down=state.handle_key_down
//                 handle_toggle_click=state.handle_toggle_click
//                 id=id_value
//                 dropdown_id=dropdown_id.clone()
//                 active_index=state.active_index
//                 input_ref=state.input_ref
//                 input_class=input_class
//             />
//             {move || {
//                 if state.is_open.get() {
//                     view! {
//                 <LiveSelectDropdown
//                     filtered_options=Signal::derive(move || state.filtered_options.get())
//                     active_index=state.active_index
//                     select_option=state.select_option
//                     label_fn=hook.label_fn.clone()
//                     search_text=state.search_text
//                     highlight_text=state.highlight_text
//                     dropdown_id=dropdown_id.clone()
//                     dropdown_ref=state.dropdown_ref
//                     dropdown_class=dropdown_class.clone()
//                 />
//                     }.into_view()
//                 } else {
//                     view! { <></> }.into_view()
//                 }
//             }}
//         </div>
//     }
// }
