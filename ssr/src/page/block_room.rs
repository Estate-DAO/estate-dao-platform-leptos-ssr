use leptos::*;
use leptos_router::use_navigate;

use crate::api::get_room;
use crate::component::SkeletonCards;
use crate::state::search_state::HotelInfoResults;
use crate::{
    api::hotel_info,
    app::AppRoutes,
    component::{FilterAndSortBy, PriceDisplay, StarRating, Divider},
    page::{InputGroup, Navbar},
    state::{search_state::SearchListResults, view_state::HotelInfoCtx},
};
use leptos::logging::log;

#[component]
pub fn BlockRoomPage() -> impl IntoView {
    let search_list_page: SearchListResults = expect_context();

    let disabled_submit: Signal<bool> = Signal::derive(move || {
        let val = search_list_page.search_result.get().is_none();
        // let val = search_list_page.search_result.get().is_some();
        // log!("disabled ig - {}", val);
        // log!(
        //     "search_list_page.search_result.get(): {:?}",
        //     search_list_page.search_result.get()
        // );
        val
    });
    
    // view! {
    //     <section class="relative h-screen">
    //         <Navbar />
            
    //         <div class="mx-auto px-20 grid justify-items-center grid-cols-1 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  
                
                
    //         </div>

    //         <div class="mx-auto">
    //             <div class="px-20 grid justify-items-center grid-cols-1 sm:grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">

    //                 <Show
    //                     when=move || search_list_page.search_result.get().is_some()
    //                 >
    //                     // <Transition fallback=fallback>
    //                         {move || {
    //                             search_list_page
    //                                 .search_result
    //                                 .get()
    //                                 .unwrap()
    //                                 .hotel_results()
    //                                 .iter()
    //                                 .map(|hotel_result| {
    //                                     view! {
    //                                         <HotelCard
    //                                             img=hotel_result.hotel_picture.clone()
    //                                             rating=hotel_result.star_rating
    //                                             hotel_name=hotel_result.hotel_name.clone()
    //                                             price=hotel_result.price.room_price
    //                                             hotel_code=hotel_result.hotel_code.clone()
    //                                         />
    //                                     }
    //                                 })
    //                                 .collect::<Vec<_>>()
    //                         }}

    //                     // </Transition>
    //                 </Show>
    //             </div>
    //         </div>
    //     </section>
    // }
    
    view! {
    <section class="relative h-screen">
        <Navbar />
        
        <div class="relative h-screen flex items-center justify-center py-10 px-10 place-content-center ">
            <div class="container mx-auto justify-between gap-6">
                <button type="text" class="text-3xl font-bold">
                    "<- You're just one step away!"
                </button>
                // <!-- Booking Details Section -->
                <div class="w-1/3 p-6">
                    <h3 class="text-2xl font-bold ">"Your Booking Details"</h3>
                    <div class="details mb-4 flex">
                        <img src="/img/home.webp" alt="Riva Beach Resort" class="w-24 h-24 object-cover rounded-lg"/>
                        <div>
                            <h3 class="font-semibold">"Riva Beach Resort"</h3>
                            <p class="text-gray-600">"North Goa, India"</p>
                        </div>
                    </div>
                    <div class="details">
                        <p class="mt-2"><strong>"Dates: "</strong>"Thu, Aug 22 – Mon, Aug 27"</p>
                        <p class="mt-2"><strong>"Guests: "</strong>"4 adults, 2 rooms"</p>
                    </div>
                    <Divider />


                    <h3 class="text-xl font-semibold mt-6">"Pay with"</h3>
                    <div class="payment-methods mt-4">

                        <div class="mt-6">
                            <label for="card-payment" class="block font-semibold">"Credit or Debit Card"</label>
                            <input type="text" placeholder="Card number" class="w-full mt-2 p-2 border border-gray-300 rounded-md"/>
                            <div class="flex mt-2 gap-4">
                                <input type="text" placeholder="Expiration" class="w-1/2 p-2 border border-gray-300 rounded-md"/>
                                <input type="text" placeholder="CVV" class="w-1/2 p-2 border border-gray-300 rounded-md"/>
                            </div>
                            <input type="text" placeholder="Country" class="w-full mt-2 p-2 border border-gray-300 rounded-md"/>
                        </div>
                    </div>
    
                    <Divider />

                    // <!-- Cancellation Policy -->
                    <h2 class="text-2xl font-bold">"Cancellation Policy"</h2>

                    
                    <div class="cancellation-policy mt-6 text-sm text-gray-600">
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse felis massa, dignissim eu luctus vel, interdum facilisis orci."
                        <a href="#" class="text-blue-500 hover:underline">"Read more"</a>.
                    </div>
                    
                    <Divider />

                    // <!-- Terms Agreement -->
                    <div class="mt-6">
                        <input type="checkbox" id="agree" class="mr-2"/>
                        <label for="agree" class="text-sm text-gray-600">
                            "I also agree to the updated "
                            <a href="#" class="text-blue-500 hover:underline">"Terms of Service"</a>
                            ", "
                            <a href="#" class="text-blue-500 hover:underline">"Payments Terms of Service"</a>
                            " and acknowledge the "
                            <a href="#" class="text-blue-500 hover:underline">"Privacy Policy"</a>
                        </label>
                    </div>
                    
                    <button class="mt-6 w-full bg-blue-600 text-white py-3 rounded-full hover:bg-blue-700">
                        "Confirm and pay"
                    </button>
                </div>
            </div>
            // <!-- Payment Summary Section -->
            <div class="w-1/3 bg-white p-6 rounded-lg shadow-lg">
                <h2 class="text-xl font-bold mb-4">"₹29,999/night"</h2>
                <Divider />
                <div class="price-breakdown">
                    <div class="flex justify-between">
                        <span>"₹29,999 x 5 nights"</span>
                        <span>"₹1,49,995"</span>
                    </div>
                    <div class="flex justify-between">
                        <span>"Taxes and fees"</span>
                        <span>"₹8,912"</span>
                    </div>
                    <div class="flex justify-between price-total mt-4 font-bold">
                        <span>"Total"</span>
                        <span>"₹1,58,907"</span>
                    </div>
                </div>
            </div>
        </div>
        
    </section>
    }
}
