#!/usr/bin/env python3
import json

print("🧮 Triangular Netting Calculation")
print("==================================")

# Sample roaming charges between 3 operators
roaming_data = {
    "T-Mobile-DE_to_Vodafone-UK": 500.00,  # €500
    "Vodafone-UK_to_Orange-FR": 750.00,    # €750
    "Orange-FR_to_T-Mobile-DE": 250.00,    # €250
    # Reverse directions (usually smaller)
    "Vodafone-UK_to_T-Mobile-DE": 100.00,  # €100
    "Orange-FR_to_Vodafone-UK": 150.00,    # €150
    "T-Mobile-DE_to_Orange-FR": 75.00,     # €75
}

print("📊 Gross Bilateral Settlements:")
total_gross = 0
for route, amount in roaming_data.items():
    print(f"  {route}: €{amount}")
    total_gross += amount

print(f"\nTotal Gross Amount: €{total_gross}")

# Calculate net positions
net_positions = {}
net_positions["T-Mobile-DE"] = (roaming_data["T-Mobile-DE_to_Vodafone-UK"] +
                               roaming_data["T-Mobile-DE_to_Orange-FR"]) - \
                              (roaming_data["Vodafone-UK_to_T-Mobile-DE"] +
                               roaming_data["Orange-FR_to_T-Mobile-DE"])

net_positions["Vodafone-UK"] = (roaming_data["Vodafone-UK_to_T-Mobile-DE"] +
                               roaming_data["Vodafone-UK_to_Orange-FR"]) - \
                              (roaming_data["T-Mobile-DE_to_Vodafone-UK"] +
                               roaming_data["Orange-FR_to_Vodafone-UK"])

net_positions["Orange-FR"] = (roaming_data["Orange-FR_to_T-Mobile-DE"] +
                             roaming_data["Orange-FR_to_Vodafone-UK"]) - \
                            (roaming_data["T-Mobile-DE_to_Orange-FR"] +
                             roaming_data["Vodafone-UK_to_Orange-FR"])

print("\n💰 Net Settlement Positions:")
total_net = 0
for operator, position in net_positions.items():
    if position > 0:
        print(f"  {operator}: +€{position:.2f} (receives)")
    elif position < 0:
        print(f"  {operator}: €{position:.2f} (pays)")
    else:
        print(f"  {operator}: €0.00 (balanced)")
    total_net += abs(position)

# Calculate actual settlements needed
print(f"\nTotal Net Settlement Volume: €{total_net/2:.2f}")
savings_percent = (1 - (total_net/2) / total_gross) * 100
print(f"Savings vs Bilateral: {savings_percent:.1f}%")

print(f"\n🎯 Final Settlements Needed:")
creditors = [(op, pos) for op, pos in net_positions.items() if pos > 0]
debtors = [(op, -pos) for op, pos in net_positions.items() if pos < 0]

for debtor, debt in debtors:
    for creditor, credit in creditors:
        if debt > 0 and credit > 0:
            settlement = min(debt, credit)
            print(f"  {debtor} → {creditor}: €{settlement:.2f}")
            debt -= settlement
            credit -= settlement

print(f"\n✅ Reduced from 6 bilateral settlements to ~2 net settlements")
print(f"💸 Settlement volume reduced by {savings_percent:.1f}%")
