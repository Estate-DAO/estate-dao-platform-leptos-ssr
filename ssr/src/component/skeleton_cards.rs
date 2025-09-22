use leptos::*;

#[component]
pub fn SkeletonCards() -> impl IntoView {
    view! {
        <div class="w-full grid grid-cols-1 gap-4 p-4">
            // Skeleton Hotel Card
            <div class="flex flex-col md:flex-row bg-white rounded-lg shadow-md border border-gray-200 overflow-hidden animate-pulse w-full">
                // Left: Image placeholder
                <div class="relative w-full md:basis-[30%] md:flex-shrink-0 h-48 md:h-auto md:self-stretch bg-gray-300"></div>

                // Right: Content placeholders
                <div class="flex-1 min-w-0 flex flex-col justify-between p-4 md:p-6">
                    // Title + Address
                    <div>
                        <div class="h-5 w-2/3 bg-gray-300 rounded mb-2"></div>
                        <div class="h-4 w-1/2 bg-gray-200 rounded mb-4"></div>

                        // Amenities row
                        <div class="flex flex-wrap gap-2 mt-3">
                            <div class="h-5 w-16 bg-gray-300 rounded"></div>
                            <div class="h-5 w-20 bg-gray-300 rounded"></div>
                            <div class="h-5 w-14 bg-gray-300 rounded"></div>
                        </div>
                    </div>

                    // Rating + Price + CTA
                    <div class="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-3 mt-6">
                        <div class="flex gap-2 items-center">
                            <div class="h-4 w-20 bg-gray-300 rounded"></div>
                            <div class="h-6 w-10 bg-gray-400 rounded"></div>
                        </div>
                        <div class="text-right">
                            <div class="h-6 w-24 bg-gray-300 rounded mb-2 ml-auto"></div>
                            <div class="h-4 w-32 bg-gray-200 rounded mb-3 ml-auto"></div>
                            <div class="h-8 w-32 bg-gray-400 rounded ml-auto"></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
