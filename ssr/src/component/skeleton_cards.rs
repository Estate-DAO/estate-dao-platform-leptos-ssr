use leptos::*;

#[component]
pub fn SkeletonCards() -> impl IntoView {
    view! {
        <div class="w-full grid grid-cols-1 sm:grid-cols-2  md:grid-cols-3 lg:grid-cols-4 gap-4 p-4 items-center justify-center self-center">
            // // First Skeleton Card
            // <div class="w-64 h-80 bg-gray-200 rounded-md shadow-lg animate-pulse">
            //     <div class="h-4 bg-gray-300 rounded mt-4 mx-4"></div>
            //     <div class="h-4 bg-gray-300 rounded mt-2 mx-4"></div>
            //     <div class="h-40 bg-gray-300 rounded mt-4 mx-4"></div>
            //     <div class="flex justify-around mt-4 mx-4">
            //         <div class="w-16 h-6 bg-gray-300 rounded"></div>
            //         <div class="w-16 h-6 bg-gray-300 rounded"></div>
            //         <div class="w-16 h-6 bg-gray-300 rounded"></div>
            //     </div>
            //     <div class="w-24 h-6 bg-gray-300 rounded mt-4 mx-4"></div>
            // </div>

            // // Second Skeleton Card
            // <div class="w-64 h-80 bg-gray-200 rounded-md shadow-lg animate-pulse">
            //     <div class="h-4 bg-gray-300 rounded mt-4 mx-4"></div>
            //     <div class="h-4 bg-gray-300 rounded mt-2 mx-4"></div>
            //     <div class="h-40 bg-gray-300 rounded mt-4 mx-4"></div>
            //     <div class="flex justify-around mt-4 mx-4">
            //         <div class="w-16 h-6 bg-gray-300 rounded"></div>
            //         <div class="w-16 h-6 bg-gray-300 rounded"></div>
            //         <div class="w-16 h-6 bg-gray-300 rounded"></div>
            //     </div>
            //     <div class="w-24 h-6 bg-gray-300 rounded mt-4 mx-4"></div>
            // </div>

            // Third Skeleton Card
            <div class="w-64 h-80 bg-gray-200 rounded-md shadow-lg animate-pulse">
                <div class="h-4 bg-gray-300 rounded mt-4 mx-4"></div>
                <div class="h-4 bg-gray-300 rounded mt-2 mx-4"></div>
                <div class="h-40 bg-gray-300 rounded mt-4 mx-4"></div>
                <div class="flex justify-around mt-4 mx-4">
                    <div class="w-16 h-6 bg-gray-300 rounded"></div>
                    <div class="w-16 h-6 bg-gray-300 rounded"></div>
                    <div class="w-16 h-6 bg-gray-300 rounded"></div>
                </div>
                <div class="w-24 h-6 bg-gray-300 rounded mt-4 mx-4"></div>
            </div>
        </div>
    }
}
