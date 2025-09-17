#!/usr/bin/env python3
"""
SP CDR Reconciliation Blockchain - Visual Architecture Generator
Creates a visual diagram of the system architecture
"""

import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import FancyBboxPatch, ConnectionPatch
import numpy as np

# Set up the figure
fig, ax = plt.subplots(1, 1, figsize=(20, 14))
ax.set_xlim(0, 20)
ax.set_ylim(0, 14)
ax.axis('off')

# Color scheme
colors = {
    'operators': '#E3F2FD',
    'api': '#FFF3E0',
    'blockchain': '#E8F5E8',
    'smart_contracts': '#F3E5F5',
    'crypto': '#FFEBEE',
    'consensus': '#E0F2F1',
    'storage': '#FFF8E1',
    'network': '#E1F5FE'
}

# Title
ax.text(10, 13.5, 'SP CDR Reconciliation Blockchain Architecture',
        ha='center', va='center', fontsize=20, fontweight='bold')

# Layer 1: Telecom Operators (Top)
operators_box = FancyBboxPatch((0.5, 11), 19, 1.8,
                               boxstyle="round,pad=0.1",
                               facecolor=colors['operators'],
                               edgecolor='black', linewidth=2)
ax.add_patch(operators_box)
ax.text(10, 12.2, 'TELECOM OPERATORS', ha='center', va='center',
        fontsize=14, fontweight='bold')

# Individual operators
operators = ['T-Mobile DE', 'Vodafone UK', 'Orange FR', 'Other SPs']
for i, op in enumerate(operators):
    x_pos = 2 + i * 4
    op_box = FancyBboxPatch((x_pos, 11.2), 3.5, 0.6,
                            boxstyle="round,pad=0.05",
                            facecolor='white', edgecolor='blue')
    ax.add_patch(op_box)
    ax.text(x_pos + 1.75, 11.5, op, ha='center', va='center', fontsize=10)

    # ZK Provers
    zk_box = FancyBboxPatch((x_pos + 0.5, 11.8), 2.5, 0.3,
                            boxstyle="round,pad=0.02",
                            facecolor='lightblue', edgecolor='darkblue')
    ax.add_patch(zk_box)
    ax.text(x_pos + 1.75, 11.95, 'ZK Prover', ha='center', va='center', fontsize=8)

# Layer 2: API Layer
api_box = FancyBboxPatch((0.5, 9.5), 19, 1.2,
                         boxstyle="round,pad=0.1",
                         facecolor=colors['api'],
                         edgecolor='black', linewidth=2)
ax.add_patch(api_box)
ax.text(1, 10.4, 'API LAYER', ha='left', va='center',
        fontsize=12, fontweight='bold')

api_endpoints = ['CDR Submission\nPOST /cdr', 'Batch Status\nGET /batch/{id}',
                 'Settlement API\nPOST /settle', 'Explorer\nGET /blocks']
for i, endpoint in enumerate(api_endpoints):
    x_pos = 2.5 + i * 4
    endpoint_box = FancyBboxPatch((x_pos, 9.7), 3, 0.8,
                                  boxstyle="round,pad=0.05",
                                  facecolor='white', edgecolor='orange')
    ax.add_patch(endpoint_box)
    ax.text(x_pos + 1.5, 10.1, endpoint, ha='center', va='center', fontsize=9)

# Layer 3: Blockchain Integration
blockchain_box = FancyBboxPatch((0.5, 8), 19, 1.2,
                                boxstyle="round,pad=0.1",
                                facecolor=colors['blockchain'],
                                edgecolor='black', linewidth=2)
ax.add_patch(blockchain_box)
ax.text(1, 8.9, 'BLOCKCHAIN INTEGRATION', ha='left', va='center',
        fontsize=12, fontweight='bold')

integration_components = ['CDR Processing\n& Batching', 'Settlement\nCalculation',
                         'Multi-Signature\nManagement']
for i, comp in enumerate(integration_components):
    x_pos = 3 + i * 5
    comp_box = FancyBboxPatch((x_pos, 8.2), 4, 0.8,
                              boxstyle="round,pad=0.05",
                              facecolor='lightgreen', edgecolor='darkgreen')
    ax.add_patch(comp_box)
    ax.text(x_pos + 2, 8.6, comp, ha='center', va='center', fontsize=9)

# Layer 4: Smart Contracts
sc_box = FancyBboxPatch((0.5, 6.2), 19, 1.5,
                        boxstyle="round,pad=0.1",
                        facecolor=colors['smart_contracts'],
                        edgecolor='black', linewidth=2)
ax.add_patch(sc_box)
ax.text(1, 7.4, 'SMART CONTRACT LAYER', ha='left', va='center',
        fontsize=12, fontweight='bold')

# Contract VM
vm_box = FancyBboxPatch((2, 6.9), 6, 0.6,
                        boxstyle="round,pad=0.05",
                        facecolor='white', edgecolor='purple')
ax.add_patch(vm_box)
ax.text(5, 7.2, 'CONTRACT VM (Stack-based Execution)', ha='center', va='center',
        fontsize=10, fontweight='bold')

# Contract types
contracts = ['CDR Privacy\nContract', 'Settlement\nContract', 'Multi-Party\nValidation']
for i, contract in enumerate(contracts):
    x_pos = 2.5 + i * 5
    contract_box = FancyBboxPatch((x_pos, 6.4), 4, 0.4,
                                  boxstyle="round,pad=0.02",
                                  facecolor='lavender', edgecolor='purple')
    ax.add_patch(contract_box)
    ax.text(x_pos + 2, 6.6, contract, ha='center', va='center', fontsize=8)

# Layer 5: Cryptographic Layer
crypto_box = FancyBboxPatch((0.5, 4.2), 19, 1.8,
                            boxstyle="round,pad=0.1",
                            facecolor=colors['crypto'],
                            edgecolor='black', linewidth=2)
ax.add_patch(crypto_box)
ax.text(1, 5.7, 'CRYPTOGRAPHIC LAYER', ha='left', va='center',
        fontsize=12, fontweight='bold')

# ZK Proof System
zk_system_box = FancyBboxPatch((2, 5.2), 8, 0.6,
                               boxstyle="round,pad=0.05",
                               facecolor='white', edgecolor='red')
ax.add_patch(zk_system_box)
ax.text(6, 5.5, 'ZK PROOF SYSTEM (arkworks/BN254/Groth16)',
        ha='center', va='center', fontsize=10, fontweight='bold')

# BLS Signature System
bls_system_box = FancyBboxPatch((11, 5.2), 8, 0.6,
                                boxstyle="round,pad=0.05",
                                facecolor='white', edgecolor='red')
ax.add_patch(bls_system_box)
ax.text(15, 5.5, 'BLS SIGNATURE SYSTEM (BLS12-381)',
        ha='center', va='center', fontsize=10, fontweight='bold')

# Crypto components
crypto_components = ['Settlement\nVerifying Key', 'CDR Privacy\nVerifying Key',
                     'Poseidon\nHash', 'Aggregate\nSignatures', 'Threshold\nSchemes']
for i, comp in enumerate(crypto_components):
    x_pos = 2.5 + i * 3
    comp_box = FancyBboxPatch((x_pos, 4.4), 2.5, 0.6,
                              boxstyle="round,pad=0.02",
                              facecolor='mistyrose', edgecolor='darkred')
    ax.add_patch(comp_box)
    ax.text(x_pos + 1.25, 4.7, comp, ha='center', va='center', fontsize=8)

# Layer 6: Consensus Layer
consensus_box = FancyBboxPatch((0.5, 2.4), 19, 1.5,
                               boxstyle="round,pad=0.1",
                               facecolor=colors['consensus'],
                               edgecolor='black', linewidth=2)
ax.add_patch(consensus_box)
ax.text(1, 3.6, 'CONSENSUS LAYER (ALBATROSS)', ha='left', va='center',
        fontsize=12, fontweight='bold')

# Albatross components
albatross_components = ['Micro Blocks\n(CDR Txns)', 'Macro Blocks\n(Finality)',
                        'Validator Set\n(SP Operators)', 'Epoch Management\n(Elections)']
for i, comp in enumerate(albatross_components):
    x_pos = 2.5 + i * 4
    comp_box = FancyBboxPatch((x_pos, 2.6), 3.5, 1.1,
                              boxstyle="round,pad=0.05",
                              facecolor='lightcyan', edgecolor='teal')
    ax.add_patch(comp_box)
    ax.text(x_pos + 1.75, 3.15, comp, ha='center', va='center', fontsize=9)

# Layer 7: Storage Layer
storage_box = FancyBboxPatch((0.5, 0.5), 19, 1.6,
                             boxstyle="round,pad=0.1",
                             facecolor=colors['storage'],
                             edgecolor='black', linewidth=2)
ax.add_patch(storage_box)
ax.text(1, 1.8, 'STORAGE LAYER (MDBX DATABASE)', ha='left', va='center',
        fontsize=12, fontweight='bold')

# Storage components
storage_components = ['Blockchain\nData', 'Contract\nState', 'ZK Keys\nStorage',
                      'Operator\nCertificates']
for i, comp in enumerate(storage_components):
    x_pos = 3 + i * 4
    comp_box = FancyBboxPatch((x_pos, 0.7), 3, 0.8,
                              boxstyle="round,pad=0.05",
                              facecolor='lightyellow', edgecolor='goldenrod')
    ax.add_patch(comp_box)
    ax.text(x_pos + 1.5, 1.1, comp, ha='center', va='center', fontsize=9)

# Add arrows between layers
arrow_props = dict(arrowstyle='->', lw=2, color='darkblue')

# Operators to API
for i in range(4):
    x_start = 3.75 + i * 4
    ax.annotate('', xy=(x_start, 10.7), xytext=(x_start, 11.2),
                arrowprops=arrow_props)

# API to Blockchain Integration
ax.annotate('', xy=(10, 9.2), xytext=(10, 9.5),
            arrowprops=arrow_props)

# Blockchain to Smart Contracts
ax.annotate('', xy=(10, 7.7), xytext=(10, 8.0),
            arrowprops=arrow_props)

# Smart Contracts to Crypto
ax.annotate('', xy=(10, 6.0), xytext=(10, 6.2),
            arrowprops=arrow_props)

# Crypto to Consensus
ax.annotate('', xy=(10, 3.9), xytext=(10, 4.2),
            arrowprops=arrow_props)

# Consensus to Storage
ax.annotate('', xy=(10, 2.1), xytext=(10, 2.4),
            arrowprops=arrow_props)

# Add side annotations
ax.text(20.2, 11.8, 'Encrypted CDR\n+ ZK Proofs', ha='left', va='center',
        fontsize=9, style='italic', color='darkblue')
ax.text(20.2, 8.6, 'CDR Netting &\nTriangulation', ha='left', va='center',
        fontsize=9, style='italic', color='darkgreen')
ax.text(20.2, 5.1, 'Privacy-Preserving\nVerification', ha='left', va='center',
        fontsize=9, style='italic', color='darkred')
ax.text(20.2, 3.1, 'Byzantine Fault\nTolerant', ha='left', va='center',
        fontsize=9, style='italic', color='teal')
ax.text(20.2, 1.3, 'ACID Transactions\n& Persistence', ha='left', va='center',
        fontsize=9, style='italic', color='goldenrod')

# Network topology (bottom right)
network_box = FancyBboxPatch((15.5, 0.2), 4, 2,
                             boxstyle="round,pad=0.05",
                             facecolor=colors['network'],
                             edgecolor='navy', linewidth=1)
ax.add_patch(network_box)
ax.text(17.5, 2, 'P2P NETWORK', ha='center', va='center',
        fontsize=10, fontweight='bold')

# Network nodes
node_positions = [(16.5, 1.5), (17.5, 1.7), (18.5, 1.5), (17, 1), (18, 1)]
for i, (x, y) in enumerate(node_positions):
    node = plt.Circle((x, y), 0.15, facecolor='lightblue', edgecolor='navy')
    ax.add_patch(node)
    if i < 4:
        ax.text(x, y-0.35, f'Node{i+1}', ha='center', va='center', fontsize=7)

# Connect nodes
connections = [(0,1), (1,2), (0,3), (1,4), (2,4), (3,4)]
for start, end in connections:
    x1, y1 = node_positions[start]
    x2, y2 = node_positions[end]
    ax.plot([x1, x2], [y1, y2], 'navy', linewidth=1)

plt.tight_layout()
plt.savefig('/home/zeljko/src/sp_cdr_reconciliation_bc/architecture_diagram.png',
            dpi=300, bbox_inches='tight', facecolor='white')
plt.savefig('/home/zeljko/src/sp_cdr_reconciliation_bc/architecture_diagram.pdf',
            bbox_inches='tight', facecolor='white')

print("Architecture diagram saved as:")
print("- architecture_diagram.png (high resolution)")
print("- architecture_diagram.pdf (vector format)")
print("\nThe diagram shows the complete 7-layer architecture:")
print("1. Telecom Operators (with ZK Provers)")
print("2. API Layer (REST endpoints)")
print("3. Blockchain Integration (CDR processing)")
print("4. Smart Contract Layer (VM + contracts)")
print("5. Cryptographic Layer (ZK proofs + BLS)")
print("6. Consensus Layer (Albatross)")
print("7. Storage Layer (MDBX database)")
print("\nP2P network topology shown in bottom right.")