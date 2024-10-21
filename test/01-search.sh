
source .env 

url="${BASE_URL}/Search"


echo "$url"

# --header "Accept-Encoding:gzip, deflate" \

curl  --location "$url" \
--header "x-Username: ${USERNAME}" \
--header "x-DomainKey: ${DOMAINKEY}" \
--header "x-System: ${SYSTEM}" \
--header "x-Password: ${PASSWORD}" \
--header "Content-type: application/json" \
--header "X-API-Key: $API_KEY" \
-X POST \
--data '{
  "CheckInDate": "25-10-2024",
  "NoOfNights": 1,
  "CountryCode": "FR",
  "CityId": 82,
  "GuestNationality": "IN",
  "NoOfRooms": 1,
  "RoomGuests": [
    {
      "NoOfAdults": 2,
      "NoOfChild": 0
    }
  ]
}'

