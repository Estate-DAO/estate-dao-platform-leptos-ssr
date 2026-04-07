pub const HOTEL_IMAGE_PLACEHOLDER_PATH: &str = "/img/hotel-placeholder.svg";

pub fn resolve_hotel_card_image(image_url: &str) -> String {
    let trimmed = image_url.trim();

    if trimmed.is_empty() {
        HOTEL_IMAGE_PLACEHOLDER_PATH.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_hotel_card_image, HOTEL_IMAGE_PLACEHOLDER_PATH};

    #[test]
    fn returns_local_placeholder_when_image_is_empty() {
        assert_eq!(resolve_hotel_card_image(""), HOTEL_IMAGE_PLACEHOLDER_PATH);
    }

    #[test]
    fn returns_local_placeholder_when_image_is_whitespace() {
        assert_eq!(
            resolve_hotel_card_image("   "),
            HOTEL_IMAGE_PLACEHOLDER_PATH
        );
    }

    #[test]
    fn preserves_non_empty_image_urls() {
        let image_url = "https://cdn.example.com/hotel.jpg";

        assert_eq!(resolve_hotel_card_image(image_url), image_url);
    }
}
