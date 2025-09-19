// use leptos::*;

// #[component]
// pub fn UserProfileIcon(
//     /// The letter to display in the icon
//     #[prop(into)]
//     letter: String,
//     /// Size of the icon (width and height in pixels)
//     #[prop(optional, default = 32)]
//     size: u32,
//     /// Background color class for Tailwind
//     #[prop(optional, into)]
//     bg_color: Option<String>,
// ) -> impl IntoView {
//     // <!-- Generate a consistent color based on the letter -->
//     let bg_class = bg_color.unwrap_or_else(|| {
//         let colors = vec![
//             "bg-red-500",
//             "bg-blue-500",
//             "bg-green-500",
//             "bg-yellow-500",
//             "bg-purple-500",
//             "bg-pink-500",
//             "bg-indigo-500",
//             "bg-teal-500",
//             "bg-orange-500",
//             "bg-cyan-500",
//             "bg-rose-500",
//             "bg-emerald-500",
//         ];

//         // <!-- Use the first character's ASCII value to pick a color -->
//         let char_code = letter.chars().next().unwrap_or('A') as usize;
//         colors[char_code % colors.len()].to_string()
//     });

//     let first_letter = letter
//         .chars()
//         .next()
//         .unwrap_or('U')
//         .to_uppercase()
//         .to_string();

//     view! {
//         <div
//             class=format!("inline-flex items-center justify-center rounded-full text-white font-semibold {}", bg_class)
//             style=format!("width: {}px; height: {}px; font-size: {}px;", size, size, size / 2)
//         >
//             {first_letter}
//         </div>
//     }
// }

// #[component]
// pub fn UserProfileIconSvg(
//     /// The letter to display in the icon
//     #[prop(into)]
//     letter: String,
//     /// Size of the icon (width and height in pixels)
//     #[prop(optional, default = 32)]
//     size: u32,
//     /// Custom color (hex format, e.g., "#ff0000")
//     #[prop(optional, into)]
//     color: Option<String>,
// ) -> impl IntoView {
//     // <!-- Generate a consistent color based on the letter -->
//     let bg_color = color.unwrap_or_else(|| {
//         let colors = vec![
//             "#ef4444", "#3b82f6", "#10b981", "#f59e0b", "#8b5cf6", "#ec4899", "#6366f1", "#14b8a6",
//             "#f97316", "#06b6d4", "#f43f5e", "#059669",
//         ];

//         // <!-- Use the first character's ASCII value to pick a color -->
//         let char_code = letter.chars().next().unwrap_or('A') as usize;
//         colors[char_code % colors.len()].to_string()
//     });

//     let first_letter = letter
//         .chars()
//         .next()
//         .unwrap_or('U')
//         .to_uppercase()
//         .to_string();
//     let font_size = size / 2;
//     let center = size / 2;

//     view! {
//         <svg
//             width=size
//             height=size
//             viewBox=format!("0 0 {} {}", size, size)
//             xmlns="http://www.w3.org/2000/svg"
//             class="inline-block"
//         >
//             <circle
//                 cx=center
//                 cy=center
//                 r=center
//                 fill=bg_color
//             />
//             <text
//                 x="50%"
//                 y="50%"
//                 text-anchor="middle"
//                 dominant-baseline="central"
//                 fill="white"
//                 font-family="system-ui, sans-serif"
//                 font-size=font_size
//                 font-weight="600"
//             >
//                 {first_letter}
//             </text>
//         </svg>
//     }
// }
