use crate::domain::DomainHotelAfterSearch;
use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = updateHotelMap)]
    fn update_hotel_map(hotels: JsValue, on_marker_click: &Closure<dyn FnMut(String)>);
}

#[component]
pub fn HotelMap(
    hotels: RwSignal<Vec<DomainHotelAfterSearch>>,
    highlighted_hotel: RwSignal<Option<String>>,
) -> impl IntoView {
    let map_div_ref = create_node_ref::<html::Div>();

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
            window.hotelMapInstance = null;
            window.hotelMarkers = [];

            window.cleanupHotelMap = function() {
                if (window.hotelMapInstance) {
                    window.hotelMapInstance.remove();
                    window.hotelMapInstance = null;
                }
                window.hotelMarkers = [];
            };

            window.updateHotelMap = function(hotels, onMarkerClick) {
                console.log('UpdateHotelMap called with ' + (hotels ? hotels.length : 0) + ' hotels');
                let container = document.getElementById('hotel-map');
                if (!container) {
                    console.log('Hotel map container not found');
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
                        console.log('Set container height to: ' + parentHeight + 'px');
                    } else {
                        // Fallback: use calc style
                        container.style.height = 'calc(100vh - 180px)';
                        container.style.width = '100%';
                        console.log('Using fallback height: calc(100vh - 180px)');
                    }
                }

                // Check for detached map instance
                if (window.hotelMapInstance) {
                    let mapContainer = window.hotelMapInstance.getContainer();
                    if (mapContainer !== container) {
                        console.log('Map container mismatch (detached), recreating');
                        window.hotelMapInstance.remove();
                        window.hotelMapInstance = null;
                    }
                }

                if (!window.hotelMapInstance) {
                    if (typeof L === 'undefined') {
                        console.error('Leaflet not loaded');
                        return;
                    }
                    console.log('Initializing Hotel Map');
                    
                    // Default center (Singapore)
                    let center = [1.3521, 103.8198];
                    let zoom = 11;

                    // Try to center on first hotel if available
                    if (hotels && hotels.length > 0) {
                        for (let i = 0; i < hotels.length; i++) {
                            if (hotels[i].location && hotels[i].location.latitude && hotels[i].location.longitude) {
                                center = [hotels[i].location.latitude, hotels[i].location.longitude];
                                console.log('Setting initial center to first hotel:', center);
                                break;
                            }
                        }
                    }

                    window.hotelMapInstance = L.map('hotel-map').setView(center, zoom);
                    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                        maxZoom: 19,
                        attribution: '© OpenStreetMap'
                    }).addTo(window.hotelMapInstance);
                }
                
                // Clear existing markers
                if (window.hotelMapInstance && window.hotelMarkers) {
                    window.hotelMarkers.forEach(m => window.hotelMapInstance.removeLayer(m));
                }
                window.hotelMarkers = [];

                if (!hotels) return;

                let bounds = L.latLngBounds();
                let hasBounds = false;

                hotels.forEach(hotel => {
                    if (hotel.location && hotel.location.latitude && hotel.location.longitude) {
                        
                        // Create custom price pill icon
                        let priceText = hotel.price ? (hotel.price.currency_code + ' ' + hotel.price.room_price) : 'View';
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
                            .addTo(window.hotelMapInstance);

                        // Popup content with image
                        let imageHtml = hotel.hotel_picture ? 
                            '<div style=\"width: 100%; height: 120px; background-image: url(\'' + hotel.hotel_picture + '\'); background-size: cover; background-position: center; border-radius: 8px 8px 0 0; margin-bottom: 8px;\"></div>' : 
                            '';
                        
                        let popupContent = 
                            '<div style=\"min-width: 200px; font-family: ui-sans-serif, system-ui, sans-serif;\">' +
                                imageHtml +
                                '<div style=\"padding: 0 4px;\">' +
                                    '<h3 style=\"margin: 0 0 4px 0; font-size: 16px; font-weight: 600; color: #0f172a;\">' + hotel.hotel_name + '</h3>' +
                                    '<p style=\"margin: 0; font-size: 14px; color: #64748b;\">' + priceText + '</p>' +
                                '</div>' +
                            '</div>';

                        marker.bindPopup(popupContent);
                        
                        marker.on('click', () => {
                            if (onMarkerClick) onMarkerClick(hotel.hotel_code);
                        });
                        
                        window.hotelMarkers.push(marker);
                        bounds.extend([hotel.location.latitude, hotel.location.longitude]);
                        hasBounds = true;
                    }
                });
                
                // Force layout recalculation AND then fit bounds
                // Use longer delay to ensure container is laid out
                setTimeout(() => {
                    if (window.hotelMapInstance) {
                        let container = document.getElementById('hotel-map');
                        console.log('Container dimensions:', container ? container.offsetWidth + 'x' + container.offsetHeight : 'N/A');
                        
                        window.hotelMapInstance.invalidateSize();
                        
                        if (hasBounds && bounds.isValid()) {
                            console.log('Fitting bounds:', bounds.toBBoxString());
                            // Add maxZoom to prevent zooming in too far
                            window.hotelMapInstance.fitBounds(bounds, {padding: [50, 50], maxZoom: 15});
                        } else {
                            console.log('No valid bounds to fit');
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
                    update_hotel_map(hotels_js, cb);
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
                let _ = js_sys::Reflect::get(&window, &JsValue::from_str("cleanupHotelMap"))
                    .and_then(|f| f.dyn_into::<js_sys::Function>())
                    .and_then(|f| f.call0(&JsValue::NULL));
            }
        }
    });

    view! {
        <div class="relative w-full h-[calc(100vh-180px)] min-h-[500px] rounded-lg overflow-hidden border border-gray-200 shadow-sm">
            <div id="hotel-map" class="absolute inset-0" node_ref=map_div_ref></div>
        </div>
    }
}
