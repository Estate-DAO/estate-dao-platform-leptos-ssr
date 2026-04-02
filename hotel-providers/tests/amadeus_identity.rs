#[cfg(feature = "amadeus")]
use hotel_providers::amadeus::AmadeusDriver;
#[cfg(feature = "amadeus")]
use hotel_providers::HotelProviderPort;

#[cfg(feature = "amadeus")]
#[test]
fn amadeus_driver_reports_expected_identity() {
    let driver = AmadeusDriver::new_mock();

    assert_eq!(driver.key(), "amadeus");
    assert_eq!(driver.name(), "Amadeus");
}
