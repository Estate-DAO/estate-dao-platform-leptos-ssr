use leptos::*;
use leptos_icons::Icon;

use crate::{
    component::{FilterAndSortBy, PriceComponent, StarRating},
    page::{InputGroup, Navbar},
};

#[derive(Clone)]
struct Amenity {
    icon: icondata::Icon,
    text: &'static str,
}

// let icon_map = HashMap::from([
//     ("Free wifi", icondata::IoWifiSharp),
//     ("Free parking", icondata::LuParkingCircle),
//     ("Swimming pool", icondata::BiSwimRegular),
//     ("Spa", icondata::BiSpaRegular),
//     ("Private beach area", icondata::BsUmbrella),
//     ("Bar", icondata::IoWineSharp),
//     ("Family Rooms", icondata::RiHomeSmile2BuildingsLine),
// ]);


#[component]
pub fn HotelDetailsPage() -> impl IntoView {
    let price = create_rw_signal(40500);

    let rating = create_rw_signal(4);

    let amenities = vec![
        Amenity {
            icon: icondata::IoWifiSharp,
            text: "Free wifi",
        },
        Amenity {
            icon: icondata::LuParkingCircle,
            text: "Free parking",
        },
        Amenity {
            icon: icondata::BiSwimRegular,
            text: "Swimming pool",
        },
        Amenity {
            icon: icondata::BiSpaRegular,
            text: "Spa",
        },
        Amenity {
            icon: icondata::BsUmbrella,
            text: "Private beach area",
        },
        Amenity {
            icon: icondata::IoWineSharp,
            text: "Bar",
        },
        Amenity {
            icon: icondata::RiHomeSmile2BuildingsLine,
            text: "Family rooms",
        },
    ];
    // let amenities: Vec<Amenity> = amenity_texts.iter()
    // .filter_map(|text| icon_map.get(text.to_lowercase().as_str()).map(|&icon| Amenity { icon, text }))
    // .collect();

    view! {
        <section class="relative h-screen">
            <Navbar />
            <div class="flex flex-col items-center mt-6 p-4">
                <InputGroup />
                <FilterAndSortBy />
            </div>

            <div class="max-w-4xl mx-auto py-8 border border-red-400">
                <div class="flex flex-col">
                    <StarRating rating=rating />
                    <div class="text-3xl font-semibold">Riva Beach Resort</div>
                </div>

                // image gallery component

                <div class="py-64 border border-dotted border-green-500 ">
                    IMAGE GALLERY COMPONENT
                </div>

                // bottom half

                <div class="flex">

                    // left side div
                    <div class="basis-4/6">
                        // About component
                        <div class="flex flex-col space-y-4">
                            <div class="text-xl">About</div>
                            <div class="mb-8">
                                "As the sun dipped below the horizon, casting a golden glow across the tranquil lake, Sarah found herself lost in thought. The gentle lapping of waves against the shore provided a soothing backdrop to her contemplation. Suddenly, a rustling in the nearby bushes caught her attention. To her amazement, a magnificent deer emerged, its antlers silhouetted against the fading light. Their eyes met for a brief moment, a connection bridging the gap between human and nature. Just as quickly as it appeared, the deer vanished into the forest, leaving Sarah with a sense of wonder and a story she'd cherish forever. The encounter reminded her of life's unexpected beauty and the importance of being present in each moment."
                            </div>
                        </div>
                        <hr class="mt-14 mb-10 border-t border-gray-300" />
                        
                        // amenities component
                        <div class=" flex flex-col space-y-8 mt-8">
                        <div class="text-xl">Amenities</div>
                        <div class="grid grid-cols-3 gap-4">
                            <For
                                each=move || amenities.clone()
                                key=|amenity| amenity.text
                                let:amenity
                            >
                                <AmenitiesIconText icon=amenity.icon text=amenity.text />
                            </For>
                        </div>
                        </div>
                    </div>

                    // right side div
                    <div class="basis-2/6">
                        // pricing component
                            // card component
                            <div class="flex flex-col space-y-4 shadow-lg p-4 rounded-lg">
                                <div>
                                    <PriceComponent price=price  price_class="text-2xl font-semibold".into() />
                                </div>
                            
                            </div>

                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn AmenitiesIconText(icon: icondata::Icon, #[prop(into)] text: String) -> impl IntoView {
    view! {
        <div class="flex items-center">
            <Icon class="inline text-xl" icon=icon />
            <span class="inline ml-2">{text}</span>
        </div>
    }
}
