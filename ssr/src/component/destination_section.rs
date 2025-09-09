use leptos::*;

#[component]
pub fn DestinationsSection() -> impl IntoView {
    view! {
        <section class="py-16 px-6">
            <div class="max-w-6xl mx-auto text-start mb-12">
                <h2 class="text-3xl font-bold">"Enjoy your dream vacation"</h2>
                <p class="text-gray-600 mt-2">
                    "Plan and book our perfect trip with expert advice, travel tips,
                     destination information and inspiration from us"
                </p>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6 max-w-6xl mx-auto">
                <DestinationCard
                    name="Australia"
                    image_url="/img/australia.jpg"
                    properties=57161
                />

                <DestinationCard
                    name="Japan"
                    image_url="/img/japan.jpg"
                    properties=46356
                />

                <DestinationCard
                    name="New Zealand"
                    image_url="/img/new-zealand.jpg"
                    properties=16861
                />

                <DestinationCard
                    name="Greece"
                    image_url="/img/greece.jpg"
                    properties=97019
                />
            </div>
        </section>
    }
}

#[component]
pub fn DestinationCard(
    name: &'static str,
    image_url: &'static str,
    properties: u32,
) -> impl IntoView {
    view! {
        <div class="flex flex-col space-y-2 cursor-pointer hover:scale-[1.02] transition-transform">
            <img
                src={image_url}
                alt={name}
                class="w-full h-48 object-cover rounded-xl shadow-sm"
            />
            <h3 class="font-semibold text-lg">{name}</h3>
            <p class="text-gray-500 text-sm">{properties} " properties"</p>
        </div>
    }
}
