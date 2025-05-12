#!/bin/bash

# -------------------
# payment webhook - still pending payment
# -------------------

# Set the URL to send the IPN data to
# IPN_URL="https://nofeebooking.com/ipn/webhook"
IPN_URL="http://localhost:3000/ipn/webhook"

# Example payment details (simulating the NowPayments IPN)
PAYMENT_DETAILS='{
  "payment_id":5077125051,
  "payment_status":"waiting",
  "pay_address":"0xd1cDE08A07cD25adEbEd35c3867a59228C09B606",
  "price_amount":170,
  "price_currency":"usd",
  "pay_amount":155.38559757,
  "actually_paid":10,
  "pay_currency":"USD",
  "order_id": "NP$6:ABC123$16:user@example.com",
  "order_description":"Apple Macbook Pro 2019 x 1",
  "purchase_id":"6084744717",
  "created_at":"2021-04-12T14:22:54.942Z",
  "updated_at":"2021-04-12T14:23:06.244Z",
  "outcome_amount":1131.7812095,
  "outcome_currency":"trx"
}'


# test one with alice@example.com 
# alice@example.com


# Generate a dummy signature (replace with actual signature generation logic)
# In real scenario, you would generate this signature based on a secret key
# and the PAYMENT_DETAILS payload.
DUMMY_SIGNATURE="some_dummy_signature"

# Send the POST request using curl
curl -X POST \
     -H "Content-Type: application/json" \
     -H "x-nowpayments-sig: $DUMMY_SIGNATURE" \
     -d "$PAYMENT_DETAILS" \
     "$IPN_URL"

# -------------------
# payment completed
# -------------------


# # Example payment details (simulating the NowPayments IPN)
# PAYMENT_DETAILS='{
#   "payment_id":5077125051,
#   "payment_status":"waiting",
#   "pay_address":"0xd1cDE08A07cD25adEbEd35c3867a59228C09B606",
#   "price_amount":170,
#   "price_currency":"usd",
#   "pay_amount":155.38559757,
#   "actually_paid":0,
#   "pay_currency":"mana",
#   "order_id":"2",
#   "order_description":"Apple Macbook Pro 2019 x 1",
#   "purchase_id":"6084744717",
#   "created_at":"2021-04-12T14:22:54.942Z",
#   "updated_at":"2021-04-12T14:23:06.244Z",
#   "outcome_amount":1131.7812095,
#   "outcome_currency":"trx"
# }'

# # Generate a dummy signature (replace with actual signature generation logic)
# # In real scenario, you would generate this signature based on a secret key
# # and the PAYMENT_DETAILS payload.
# DUMMY_SIGNATURE="some_dummy_signature"

# # Send the POST request using curl
# curl -X POST \
#      -H "Content-Type: application/json" \
#      -H "x-nowpayments-sig: $DUMMY_SIGNATURE" \
#      -d "$PAYMENT_DETAILS" \
#      "$IPN_URL"
