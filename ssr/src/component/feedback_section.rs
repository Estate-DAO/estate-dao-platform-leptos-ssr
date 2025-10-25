use leptos::prelude::*;

#[component]
pub fn FeedbackSection() -> impl IntoView {
    view! {
        <section class="bg-[#fdf9f4] py-16 px-6">
            <div class="max-w-6xl mx-auto text-start mb-12">
                <h2 class="text-3xl font-bold">"Our Customer Feedback"</h2>
                <p class="text-gray-600 mt-2">"Don’t take our word for it. Trust our customers!"</p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-6xl mx-auto">
                <TestimonialCard
                    name="Floyd"
                    role=None
                    text="The experience exceeded my expectations. The rooms were comfortable, clean, and beautifully maintained. Highly recommended for anyone looking for a relaxing stay."
                    rating=4
                />

                <TestimonialCard
                    name="Ronald"
                    role=None
                    text="Exceptional service from start to finish. The staff were attentive and always available to help. It made our trip stress-free and enjoyable."
                    rating=5
                />

                <TestimonialCard
                    name="Savannah"
                    role=None
                    text="A great hotel with modern amenities and a welcoming atmosphere. It’s the perfect getaway spot for both business and leisure travelers."
                    rating=4
                />
            </div>
        </section>
    }
}

#[component]
pub fn TestimonialCard(
    name: &'static str,
    role: Option<&'static str>, // if you want to show role/title later
    text: &'static str,
    rating: u8, // from 1–5
) -> impl IntoView {
    let stars = (1..=5).map(move |i| {
        view! {
            <svg
                class=if i <= rating {
                    "w-5 h-5 text-yellow-500 fill-current"
                } else {
                    "w-5 h-5 text-gray-300"
                }
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 20 20"
                fill="currentColor"
            >
                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"/>
            </svg>
        }
    }).collect_view();

    view! {
        <div class="bg-white rounded-md shadow-sm p-6 flex flex-col items-start space-y-4 w-full">
            <h3 class="font-bold text-lg">{name}</h3>
            <div class="flex">{stars}</div>
            <p class="text-gray-600 text-sm">{text}</p>
        </div>
    }
}
