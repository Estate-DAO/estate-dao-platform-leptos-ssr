source .env 

url="${BASE_URL}/HotelCityList"


echo "$url"

curl  --location "$url" \
--header "x-Username: ${USERNAME}" \
--header "x-DomainKey: ${DOMAINKEY}" \
--header "x-System: ${SYSTEM}" \
--header "x-Password: ${PASSWORD}" \
--header "Content-type: application/json" \
--header "X-API-Key: $API_KEY"


# --header "Accept-Encoding:gzip, deflate" \

