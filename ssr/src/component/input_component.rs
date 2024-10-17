use crate::state::search_state::SearchCtx;
use leptos::*;

macro_rules! input_component {
    ($name:ident, $input_element:ident, $input_type:ident, $attrs:expr) => {
        #[component]
        pub fn $name<T: 'static, U: Fn(T) + 'static + Copy, V: Fn(String) -> Option<T> + 'static + Copy>(
            // #[prop(into)] heading: String,
            #[prop(into)] placeholder: String,
            #[prop(optional)] initial_value: Option<String>,
            #[prop(optional, into)] input_type: Option<String>,
            updater: U,
            validator: V,
        ) -> impl IntoView {
            let ctx: SearchCtx = expect_context();
            let error = create_rw_signal(initial_value.is_none());
            let show_error = create_rw_signal(false);
            // if error.get_untracked() {
            //     ctx.invalid_cnt.update(|c| *c += 1);
            // }
            let input_ref = create_node_ref::<html::$input_type>();
            let on_input = move || {
                let Some(input) = input_ref() else {
                    return;
                };
                let value = input.value();
                match validator(value) {
                    Some(v) => {
                        if error.get_untracked() {
                            // ctx.invalid_cnt.update(|c| *c -= 1);
                        }
                        error.set(false);
                        updater(v);
                    },
                    None => {
                        show_error.set(true);
                        if error.get_untracked() {
                            return;
                        }
                        error.set(true);
                        // ctx.invalid_cnt.update(|c| *c += 1);
                        }
                    }
            };
            // create_effect(move |prev| {
            //     ctx.on_form_reset.track();
            //     // Do not trigger on render
            //     if prev.is_none() {
            //         return;
            //     }
            //     let cur_show_err = show_error.get_untracked();
            //     on_input();
            //     // this is necessary
            //     // if the user had not previously input anything,
            //     // we don't want to show an error
            //     if !cur_show_err {
            //         show_error.set(false);
            //     }
            // });

            let input_class =move ||  match show_error() && error() {
                false => format!("w-full p-3  md:p-4 md:py-5 text-black outline-none bg-white/10 border-2 border-solid border-white/20 text-xs  rounded-xl placeholder-neutral-600"),
                _ =>  format!("w-full p-3  md:p-4 md:py-5 text-black outline-none bg-white/10 border-2 border-solid border-red-500 text-xs  rounded-xl placeholder-neutral-600")
            };
            view! {
                <div class="flex flex-col grow gap-y-1 text-sm md:text-base">
                    //  <span class="text-black font-semibold">{heading.clone()}</span>
                     <$input_element
                        _ref=input_ref
                        value={initial_value.unwrap_or_default()}
                        on:input=move |_| on_input()
                        placeholder=placeholder
                        class=move || input_class()
                        type=input_type.unwrap_or_else(|| "text".into() )
                    />
                    <span class="text-red-500 font-semibold text-xs">
                        <Show when=move || show_error() && error()>
                                "Invalid "
                        </Show>
                    </span>
                </div>
            }
        }
    }
}


input_component!(InputBox, input, Input, {});

pub fn non_empty_string_validator(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}
