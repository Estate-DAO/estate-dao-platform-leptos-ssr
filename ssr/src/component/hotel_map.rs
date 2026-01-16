use crate::domain::DomainHotelAfterSearch;
use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = updateHotelMap)]
    fn update_hotel_map(
        map_id: &str,
        hotels: JsValue,
        on_marker_click: &Closure<dyn FnMut(String)>,
    );
}

#[component]
pub fn HotelMap(
    /// Unique ID for this map instance (e.g., "hotel-map-desktop", "hotel-map-mobile")
    #[prop(default = "hotel-map".to_string())]
    map_id: String,
    hotels: RwSignal<Vec<DomainHotelAfterSearch>>,
    highlighted_hotel: RwSignal<Option<String>>,
) -> impl IntoView {
    let map_div_ref = create_node_ref::<html::Div>();
    let map_id_effect = map_id.clone();
    let map_id_cleanup = map_id.clone();
    let map_id_view = map_id.clone();

    // Callback for marker clicks
    // We use a stored closure to keep it alive
    let on_marker_click = store_value(Closure::wrap(Box::new(move |hotel_code: String| {
        highlighted_hotel.set(Some(hotel_code));
    }) as Box<dyn FnMut(String)>));

    create_effect(move |_| {
        let current_hotels = hotels.get();
        #[cfg(feature = "hydrate")]
        {
            if let Ok(hotels_js) = serde_wasm_bindgen::to_value(&current_hotels) {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                // Check if global function exists
                let has_fn = js_sys::Reflect::has(&window, &JsValue::from_str("updateHotelMap"))
                    .unwrap_or(false);

                if !has_fn {
                    // Inject script manually to avoid hydration issues
                    let script_content = r#"
            // Store map instances by ID
            window.hotelMapInstances = window.hotelMapInstances || {};
            window.hotelMarkersMap = window.hotelMarkersMap || {};

            window.cleanupHotelMap = function(mapId) {
                mapId = mapId || 'hotel-map';
                if (window.hotelMapInstances[mapId]) {
                    window.hotelMapInstances[mapId].remove();
                    delete window.hotelMapInstances[mapId];
                }
                delete window.hotelMarkersMap[mapId];
            };

            window.updateHotelMap = function(mapId, hotels, onMarkerClick) {
                console.log('UpdateHotelMap called for ' + mapId + ' with ' + (hotels ? hotels.length : 0) + ' hotels');
                let container = document.getElementById(mapId);
                if (!container) {
                    console.log('Hotel map container not found: ' + mapId);
                    return;
                }

                // CRITICAL: Set explicit height on container BEFORE Leaflet init
                // Because Leaflet needs a container with non-zero dimensions
                let parent = container.parentElement;
                if (parent) {
                    let parentHeight = parent.clientHeight;
                    if (parentHeight > 0) {
                        container.style.height = parentHeight + 'px';
                        container.style.width = '100%';
                        container.style.position = 'absolute';
                        container.style.top = '0';
                        container.style.left = '0';
                        console.log('[' + mapId + '] Set container height to: ' + parentHeight + 'px');
                    } else {
                        // Fallback: Try to find any parent with height
                        let searchParent = parent.parentElement;
                        let foundHeight = 0;
                        while (searchParent && foundHeight === 0) {
                            foundHeight = searchParent.clientHeight;
                            searchParent = searchParent.parentElement;
                        }
                        if (foundHeight > 0) {
                            container.style.height = (foundHeight - 100) + 'px';
                            container.style.width = '100%';
                            console.log('[' + mapId + '] Using parent ancestor height: ' + (foundHeight - 100) + 'px');
                        } else {
                            // Ultimate fallback: use viewport calculation
                            container.style.height = 'calc(100vh - 220px)';
                            container.style.width = '100%';
                            console.log('[' + mapId + '] Using viewport fallback height: calc(100vh - 220px)');
                        }
                    }
                }

                // Get or create map instance for this ID
                let mapInstance = window.hotelMapInstances[mapId];
                let markers = window.hotelMarkersMap[mapId] || [];

                // Check for detached map instance
                if (mapInstance) {
                    let mapContainer = mapInstance.getContainer();
                    if (mapContainer !== container) {
                        console.log('[' + mapId + '] Map container mismatch (detached), recreating');
                        mapInstance.remove();
                        mapInstance = null;
                        delete window.hotelMapInstances[mapId];
                    }
                }

                if (!mapInstance) {
                    if (typeof L === 'undefined') {
                        console.error('Leaflet not loaded');
                        return;
                    }
                    console.log('[' + mapId + '] Initializing Hotel Map');
                    
                    // Default center (Singapore)
                    let center = [1.3521, 103.8198];
                    let zoom = 11;

                    // Try to center on first hotel if available
                    if (hotels && hotels.length > 0) {
                        for (let i = 0; i < hotels.length; i++) {
                            if (hotels[i].location && hotels[i].location.latitude && hotels[i].location.longitude) {
                                center = [hotels[i].location.latitude, hotels[i].location.longitude];
                                console.log('[' + mapId + '] Setting initial center to first hotel:', center);
                                break;
                            }
                        }
                    }

                    mapInstance = L.map(mapId).setView(center, zoom);
                    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                        maxZoom: 19,
                        attribution: '© OpenStreetMap'
                    }).addTo(mapInstance);
                    window.hotelMapInstances[mapId] = mapInstance;
                }
                
                // Clear existing markers
                if (mapInstance && markers) {
                    markers.forEach(m => mapInstance.removeLayer(m));
                }
                markers = [];
                window.hotelMarkersMap[mapId] = markers;

                if (!hotels) return;

                let bounds = L.latLngBounds();
                let hasBounds = false;

                hotels.forEach(hotel => {
                    if (hotel.location && hotel.location.latitude && hotel.location.longitude) {
                        
                        // Create custom price pill icon - FORMAT PRICE AS INTEGER
                        let priceText = hotel.price ? (hotel.price.currency_code + ' ' + Math.round(hotel.price.room_price)) : 'View';
                        let customIcon = L.divIcon({
                            className: 'custom-map-marker',
                            html: '<div style=\"' +
                                'background-color: white; ' +
                                'color: #1e293b; ' +
                                'padding: 4px 8px; ' +
                                'border-radius: 9999px; ' +
                                'font-weight: bold; ' +
                                'font-size: 14px; ' +
                                'box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06); ' +
                                'border: 1px solid #e2e8f0;' +
                                'text-align: center;' +
                                'white-space: nowrap;' +
                                'width: auto;' +
                                'display: inline-block;' +
                                '\">' + priceText + '</div>',
                            iconSize: [null, null], // Auto size
                            iconAnchor: [20, 40] // Adjust anchor
                        });

                        let marker = L.marker([hotel.location.latitude, hotel.location.longitude], {icon: customIcon})
                            .addTo(mapInstance);

                        // Build hotel details URL from current page URL params
                        let currentUrl = new URL(window.location.href);
                        let params = currentUrl.searchParams;
                        let hotelDetailsUrl = '/hotel-details?' +
                            'hotelCode=' + encodeURIComponent(hotel.hotel_code) +
                            '&checkin=' + (params.get('checkin') || '') +
                            '&checkout=' + (params.get('checkout') || '') +
                            '&adults=' + (params.get('adults') || '2') +
                            '&children=' + (params.get('children') || '0') +
                            '&rooms=' + (params.get('rooms') || '1');

                        // Popup content with CLICKABLE image that navigates to hotel details
                        let imageHtml = hotel.hotel_picture ? 
                            '<a href=\"' + hotelDetailsUrl + '\" target=\"_blank\" style=\"display:block; cursor:pointer;\">' +
                                '<div style=\"width: 100%; height: 120px; background-image: url(\'' + hotel.hotel_picture + '\'); background-size: cover; background-position: center; border-radius: 8px 8px 0 0; margin-bottom: 8px;\"></div>' +
                            '</a>' : 
                            '';
                        
                        let popupContent = 
                            '<div style=\"min-width: 200px; font-family: ui-sans-serif, system-ui, sans-serif;\">' +
                                imageHtml +
                                '<div style=\"padding: 0 4px;\">' +
                                    '<a href=\"' + hotelDetailsUrl + '\" target=\"_blank\" style=\"text-decoration: none; color: inherit;\">' +
                                        '<h3 style=\"margin: 0 0 4px 0; font-size: 16px; font-weight: 600; color: #0f172a; cursor: pointer;\">' + hotel.hotel_name + '</h3>' +
                                    '</a>' +
                                    '<p style=\"margin: 0; font-size: 14px; color: #64748b;\">' + priceText + '</p>' +
                                '</div>' +
                            '</div>';

                        // Bind popup with autoPan disabled to prevent zoom changes
                        marker.bindPopup(popupContent, { autoPan: false });
                        
                        // On marker click: highlight + scroll to card WITHIN the scrollable container
                        marker.on('click', () => {
                            if (onMarkerClick) onMarkerClick(hotel.hotel_code);
                            
                            // Scroll the corresponding hotel card into view within its scrollable container
                            let cardElement = document.getElementById('hotel-card-' + hotel.hotel_code);
                            if (cardElement) {
                                // Find the scrollable parent container (the one with overflow-y-auto)
                                let scrollableParent = cardElement.closest('.overflow-y-auto');
                                if (scrollableParent) {
                                    // Calculate scroll position to center the card
                                    let cardTop = cardElement.offsetTop;
                                    let containerHeight = scrollableParent.clientHeight;
                                    let cardHeight = cardElement.offsetHeight;
                                    let scrollTo = cardTop - (containerHeight / 2) + (cardHeight / 2);
                                    scrollableParent.scrollTo({ top: scrollTo, behavior: 'smooth' });
                                }
                                
                                // Add a brief highlight effect
                                cardElement.style.transition = 'box-shadow 0.3s, transform 0.3s';
                                cardElement.style.boxShadow = '0 0 0 3px #3b82f6';
                                cardElement.style.transform = 'scale(1.02)';
                                setTimeout(() => {
                                    cardElement.style.boxShadow = '';
                                    cardElement.style.transform = '';
                                }, 2000);
                            }
                        });
                        
                        markers.push(marker);
                        bounds.extend([hotel.location.latitude, hotel.location.longitude]);
                        hasBounds = true;
                    }
                });
                
                window.hotelMarkersMap[mapId] = markers;
                
                // Force layout recalculation AND then fit bounds
                // Use longer delay to ensure container is laid out
                setTimeout(() => {
                    let instance = window.hotelMapInstances[mapId];
                    if (instance) {
                        let container = document.getElementById(mapId);
                        console.log('[' + mapId + '] Container dimensions:', container ? container.offsetWidth + 'x' + container.offsetHeight : 'N/A');
                        
                        instance.invalidateSize();
                        
                        if (hasBounds && bounds.isValid()) {
                            console.log('[' + mapId + '] Fitting bounds:', bounds.toBBoxString());
                            // Add maxZoom to prevent zooming in too far
                            instance.fitBounds(bounds, {padding: [50, 50], maxZoom: 15});
                        } else {
                            console.log('[' + mapId + '] No valid bounds to fit');
                        }
                    }
                }, 500);
            };
            "#;
                    let script = document.create_element("script").unwrap();
                    script.set_text_content(Some(script_content));
                    document.body().unwrap().append_child(&script).unwrap();
                }

                // Call the function
                on_marker_click.with_value(|cb| {
                    update_hotel_map(&map_id_effect, hotels_js, cb);
                });
            }
        }
    });

    on_cleanup(move || {
        #[cfg(feature = "hydrate")]
        {
            let window = web_sys::window().unwrap();
            let has_fn = js_sys::Reflect::has(&window, &JsValue::from_str("cleanupHotelMap"))
                .unwrap_or(false);
            if has_fn {
                // Pass map_id to cleanup
                let args = js_sys::Array::new();
                args.push(&JsValue::from_str(&map_id_cleanup));
                let _ = js_sys::Reflect::get(&window, &JsValue::from_str("cleanupHotelMap"))
                    .and_then(|f| f.dyn_into::<js_sys::Function>())
                    .and_then(|f| f.apply(&JsValue::NULL, &args));
            }
        }
    });

    view! {
        <div
            class="relative w-full h-full min-h-[400px] rounded-lg overflow-hidden border border-gray-200 shadow-sm"
            style="min-height: max(400px, calc(100vh - 300px));"
        >
            <div id=map_id_view class="absolute inset-0" node_ref=map_div_ref></div>
        </div>
    }
}
