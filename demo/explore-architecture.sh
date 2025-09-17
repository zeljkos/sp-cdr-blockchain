#!/bin/bash
echo "🏗️  Architecture Explorer"
echo "========================"

echo "📁 Project Structure:"
echo "===================="
find .. -type f -name "*.rs" | head -25 | while read file; do
    lines=$(wc -l < "$file" 2>/dev/null || echo "0")
    echo "  $file ($lines lines)"
done

echo ""
echo "🔧 Key Components Found:"
echo "======================="

echo "Storage Components:"
find .. -name "*storage*" -o -name "*mdbx*" | while read file; do
    echo "  📦 $file"
done

echo ""
echo "Crypto Components:"
find .. -name "*crypto*" -o -name "*bls*" -o -name "*zkp*" | while read file; do
    echo "  🔐 $file"
done

echo ""
echo "Smart Contracts:"
find .. -name "*contract*" -o -name "*vm*" | while read file; do
    echo "  📜 $file"
done

echo ""
echo "Blockchain Core:"
find .. -name "*block*" -o -name "*consensus*" -o -name "*validator*" | while read file; do
    echo "  ⛓️  $file"
done

echo ""
echo "🔍 Code Statistics:"
echo "=================="
total_lines=0
total_files=0

for file in $(find .. -name "*.rs" -type f); do
    lines=$(wc -l < "$file" 2>/dev/null || echo "0")
    total_lines=$((total_lines + lines))
    total_files=$((total_files + 1))
done

echo "Total Rust files: $total_files"
echo "Total lines of code: $total_lines"
echo "Average file size: $((total_lines / total_files)) lines"

echo ""
echo "📊 Component Breakdown:"
echo "====================="

for component in storage crypto blockchain smart_contracts zkp common lib; do
    files=$(find .. -path "*/$component/*" -name "*.rs" | wc -l)
    lines=$(find .. -path "*/$component/*" -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}' || echo "0")
    echo "  $component: $files files, $lines lines"
done

echo ""
echo "✅ Architecture exploration complete!"
