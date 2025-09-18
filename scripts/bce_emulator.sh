#!/bin/bash
# BCE Record Emulator
# Sends realistic BCE records to the SP blockchain via HTTP API
# Simulates operator billing systems sending settlement data

API_URL="http://localhost:9090"
TIMESTAMP=$(date +%s)

echo "🏗️  BCE Record Emulator for SP Consortium Settlement"
echo "📡 Target API: $API_URL"
echo "⏰ Timestamp: $TIMESTAMP"
echo ""

# Function to send BCE record
send_bce_record() {
    local record_id=$1
    local record_type=$2
    local home_plmn=$3
    local visited_plmn=$4
    local duration=$5
    local uplink=$6
    local downlink=$7
    local wholesale=$8
    local retail=$9
    local imsi=${10}

    echo "📋 Sending $record_type: $record_id ($home_plmn -> $visited_plmn)"

    curl -X POST "$API_URL/api/v1/bce/submit" \
        -H "Content-Type: application/json" \
        -d "{
            \"record\": {
                \"record_id\": \"$record_id\",
                \"record_type\": \"$record_type\",
                \"imsi\": \"$imsi\",
                \"home_plmn\": \"$home_plmn\",
                \"visited_plmn\": \"$visited_plmn\",
                \"session_duration\": $duration,
                \"bytes_uplink\": $uplink,
                \"bytes_downlink\": $downlink,
                \"wholesale_charge\": $wholesale,
                \"retail_charge\": $retail,
                \"currency\": \"EUR\",
                \"timestamp\": $TIMESTAMP,
                \"charging_id\": $RANDOM
            }
        }"

    echo ""
    echo "✅ Record sent"
    echo ""
}

echo "🚀 Starting BCE record simulation..."
echo ""

# Scenario 1: German tourists in UK (high data usage)
echo "📱 Scenario 1: German tourists roaming in UK"
send_bce_record \
    "BCE_${TIMESTAMP}_TMO_DE_$(printf "%09d" $RANDOM)" \
    "DATA_SESSION_CDR" \
    "26201" \
    "23410" \
    "1847" \
    "2456789" \
    "15678901" \
    "45620" \
    "62350" \
    "262011234567890"

# Scenario 2: French business call in UK
echo "📞 Scenario 2: French business call in UK"
send_bce_record \
    "BCE_${TIMESTAMP}_ORG_FR_$(printf "%09d" $RANDOM)" \
    "VOICE_CALL_CDR" \
    "20801" \
    "23415" \
    "892" \
    "0" \
    "0" \
    "23450" \
    "34500" \
    "208011234567890"

# Scenario 3: Norwegian data session in Germany
echo "📊 Scenario 3: Norwegian data session in Germany"
send_bce_record \
    "BCE_${TIMESTAMP}_TEL_NO_$(printf "%09d" $RANDOM)" \
    "DATA_SESSION_CDR" \
    "24201" \
    "26202" \
    "656" \
    "1234567" \
    "8765432" \
    "18750" \
    "28900" \
    "242011234567890"

# Scenario 4: UK visitor call in France
echo "🗼 Scenario 4: UK visitor call in France"
send_bce_record \
    "BCE_${TIMESTAMP}_VF_UK_$(printf "%09d" $RANDOM)" \
    "VOICE_CALL_CDR" \
    "23415" \
    "20810" \
    "445" \
    "0" \
    "0" \
    "15680" \
    "22400" \
    "234151234567890"

echo "🎉 BCE simulation complete!"
echo ""
echo "📊 Check pipeline statistics:"
echo "curl $API_URL/api/v1/bce/stats"
echo ""
echo "🏥 Health check:"
echo "curl $API_URL/health"
echo ""