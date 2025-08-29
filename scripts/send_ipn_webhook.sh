#!/bin/bash

# Script to send an IPN webhook with dynamic order_id and payment_id

# Set the URL to send the IPN data to
IPN_URL="http://localhost:3002/ipn/webhook"

# Check if order_id and payment_id are provided
if [ -z "$1" ] || [ -z "$2" ]; then
  echo "Usage: $0 <order_id> <payment_id> [payment_status]"
  echo "  <order_id>: The order ID for the payment (e.g., NP$6:ABC123$34:email@example.com)"
  echo "  <payment_id>: The payment ID (e.g., 5077125051)"
  echo "  [payment_status]: Optional. The status of the payment (default: waiting)"
  exit 1
fi

ORDER_ID="$1"
PAYMENT_ID="$2"
PAYMENT_STATUS="${3:-waiting}" # Default to 'waiting' if not provided

# Construct the PAYMENT_DETAILS JSON with dynamic values
PAYMENT_DETAILS='{
  "payment_id":'$PAYMENT_ID',
  "payment_status":"'$PAYMENT_STATUS'",
  "pay_address":"0xd1cDE08A07cD25adEbEd35c3867a59228C09B606",
  "price_amount":170,
  "price_currency":"usd",
  "pay_amount":155.38559757,
  "actually_paid":10,
  "pay_currency":"USD",
  "order_id": "'$ORDER_ID'",
  "order_description":"Hotel Room Booking",
  "purchase_id":"6084744717",
  "created_at":"2021-04-12T14:22:54.942Z",
  "updated_at":"2021-04-12T14:23:06.244Z",
  "outcome_amount":1131.7812095,
  "outcome_currency":"trx"
}'

# Generate a dummy signature (replace with actual signature generation logic)
DUMMY_SIGNATURE="some_dummy_signature"

echo "Sending IPN webhook with:"
echo "  Order ID: $ORDER_ID"
echo "  Payment ID: $PAYMENT_ID"
echo "  Payment Status: $PAYMENT_STATUS"
echo "  IPN URL: $IPN_URL"

# Send the POST request using curl
curl -X POST \
     -H "Content-Type: application/json" \
     -H "x-nowpayments-sig: $DUMMY_SIGNATURE" \
     -d "$PAYMENT_DETAILS" \
     "$IPN_URL"

echo "\nWebhook sent."
