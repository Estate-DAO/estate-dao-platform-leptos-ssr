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
        on_map_move: &Closure<dyn FnMut(f64, f64)>,
    );
}

#[component]
pub fn HotelMap(
    /// Unique ID for this map instance (e.g., "hotel-map-desktop", "hotel-map-mobile")
    #[prop(default = "hotel-map".to_string())]
    map_id: String,
    hotels: RwSignal<Vec<DomainHotelAfterSearch>>,
    highlighted_hotel: RwSignal<Option<String>>,
    /// Callback when map is moved/panned, provides (lat, lng) of new center
    #[prop(optional)]
    on_map_move: Option<Callback<(f64, f64)>>,
) -> impl IntoView {
    let map_div_ref = create_node_ref::<html::Div>();
    let map_id_effect = map_id.clone();
    let map_id_cleanup = map_id.clone();
    let map_id_view = map_id.clone();

    // Callback for marker clicks
    // We use a stored closure to keep it alive
    let on_marker_click_closure = store_value(Closure::wrap(Box::new(move |hotel_code: String| {
        highlighted_hotel.set(Some(hotel_code));
    }) as Box<dyn FnMut(String)>));

    // Callback for map move events
    let on_map_move_closure = store_value(Closure::wrap(Box::new(move |lat: f64, lng: f64| {
        if let Some(callback) = on_map_move {
            leptos::Callable::call(&callback, (lat, lng));
        }
    }) as Box<dyn FnMut(f64, f64)>));

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
            window.hotelMapMoveCallbacks = window.hotelMapMoveCallbacks || {};

            window.cleanupHotelMap = function(mapId) {
                mapId = mapId || 'hotel-map';
                if (window.hotelMapInstances[mapId]) {
                    window.hotelMapInstances[mapId].remove();
                    delete window.hotelMapInstances[mapId];
                }
                delete window.hotelMarkersMap[mapId];
                delete window.hotelMapMoveCallbacks[mapId];
            };

            window.updateHotelMap = function(mapId, hotels, onMarkerClick, onMapMove) {
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
                    let zoom = 13;

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

                    // Add moveend event listener for map panning
                    if (onMapMove) {
                        window.hotelMapMoveCallbacks[mapId] = onMapMove;
                        mapInstance.on('moveend', function() {
                            let center = mapInstance.getCenter();
                            console.log('[' + mapId + '] Map moved to:', center.lat, center.lng);
                            if (window.hotelMapMoveCallbacks[mapId]) {
                                window.hotelMapMoveCallbacks[mapId](center.lat, center.lng);
                            }
                        });
                    }
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

                        // Note: We don't use marker.bindPopup() here because we have
                        // a custom floating card component that displays the selected hotel
                        
                        // On marker click: trigger the callback to show the floating card
                        marker.on('click', () => {
                            if (onMarkerClick) onMarkerClick(hotel.hotel_code);
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
                            instance.fitBounds(bounds, {padding: [50, 50], maxZoom: 15, minZoom: 12});
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
                on_marker_click_closure.with_value(|marker_cb| {
                    on_map_move_closure.with_value(|move_cb| {
                        update_hotel_map(&map_id_effect, hotels_js, marker_cb, move_cb);
                    });
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
