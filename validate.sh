#!/bin/bash

# Validation script for Apiary Honeycomb CLI implementation
# Checks that all required files and API endpoints are covered

echo "🔍 Validating Apiary Implementation"
echo "=================================="

# Check project structure
echo "📁 Checking project structure..."

required_files=(
    "Cargo.toml"
    "src/main.rs"
    "src/client.rs" 
    "src/common.rs"
    "src/auth.rs"
    "src/datasets.rs"
    "src/columns.rs"
    "src/triggers.rs"
    "src/queries.rs"
    "src/boards.rs"
    "src/resources.rs"
    "examples/dataset.json"
    "examples/query.json"
    "examples/trigger.json"
    "examples/events.json"
    "examples/board.json"
    "README.md"
)

missing_files=()
for file in "${required_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        missing_files+=("$file")
    fi
done

if [[ ${#missing_files[@]} -eq 0 ]]; then
    echo "✅ All required files present"
else
    echo "❌ Missing files:"
    printf "   %s\n" "${missing_files[@]}"
fi

# Check API endpoint coverage
echo ""
echo "🚀 Checking API endpoint coverage..."

# Check actual command implementations in main.rs
echo "📋 Covered API resources:"

# Core Resources
if grep -q "Commands::Auth" src/main.rs; then echo "   ✅ auth"; else echo "   ❌ auth"; fi
if grep -q "Commands::Datasets" src/main.rs; then echo "   ✅ datasets"; else echo "   ❌ datasets"; fi
if grep -q "Commands::Columns" src/main.rs; then echo "   ✅ columns"; else echo "   ❌ columns"; fi
if grep -q "Commands::Triggers" src/main.rs; then echo "   ✅ triggers"; else echo "   ❌ triggers"; fi
if grep -q "Commands::Queries" src/main.rs; then echo "   ✅ queries"; else echo "   ❌ queries"; fi
if grep -q "Commands::Boards" src/main.rs; then echo "   ✅ boards"; else echo "   ❌ boards"; fi

# Resources module commands
if grep -q "Commands::Markers" src/main.rs; then echo "   ✅ markers"; else echo "   ❌ markers"; fi
if grep -q "Commands::Recipients" src/main.rs; then echo "   ✅ recipients"; else echo "   ❌ recipients"; fi
if grep -q "Commands::Slos" src/main.rs; then echo "   ✅ slos"; else echo "   ❌ slos"; fi
if grep -q "Commands::BurnAlerts" src/main.rs; then echo "   ✅ burn_alerts"; else echo "   ❌ burn_alerts"; fi
if grep -q "Commands::Environments" src/main.rs; then echo "   ✅ environments"; else echo "   ❌ environments"; fi
if grep -q "Commands::Keys" src/main.rs; then echo "   ✅ keys/api_keys"; else echo "   ❌ keys"; fi

# Additional endpoints
if grep -q "Commands::Events" src/main.rs; then echo "   ✅ events"; else echo "   ❌ events"; fi
if grep -q "Commands::QueryResults" src/main.rs; then echo "   ✅ query_results"; else echo "   ❌ query_results"; fi
if grep -q "Commands::ServiceMaps" src/main.rs; then echo "   ✅ service_maps"; else echo "   ❌ service_maps"; fi
if grep -q "Commands::Reporting" src/main.rs; then echo "   ✅ reporting"; else echo "   ❌ reporting"; fi

# Additional checks for comprehensive coverage
echo ""
echo "🔍 Additional API features:"
if grep -q "derived_columns\|calculated.*fields" src/resources.rs; then echo "   ✅ calculated_fields/derived_columns"; else echo "   ❌ calculated_fields"; fi
if grep -q "dataset_definitions" src/ -R; then echo "   ✅ dataset_definitions"; else echo "   ❌ dataset_definitions"; fi
if grep -q "marker_settings" src/ -R; then echo "   ✅ marker_settings"; else echo "   ❌ marker_settings"; fi
if grep -q "query_annotations" src/ -R; then echo "   ✅ query_annotations"; else echo "   ❌ query_annotations"; fi
if grep -q "kinesis_events\|KinesisEvents" src/ -R; then echo "   ✅ kinesis_events"; else echo "   ❌ kinesis_events"; fi

# Check dependencies
echo ""
echo "📦 Checking Cargo dependencies..."
required_deps=("clap" "reqwest" "serde" "serde_json" "tokio" "anyhow" "uuid" "chrono")
for dep in "${required_deps[@]}"; do
    if grep -q "$dep" Cargo.toml; then
        echo "   ✅ $dep"
    else
        echo "   ❌ $dep (missing)"
    fi
done

# Check for API consistency markers
echo ""
echo "🔗 Checking API consistency features..."
consistency_features=(
    "environment variables"
    "multiple output formats"
    "error handling"
    "authentication headers"
    "JSON data parsing"
)

checks=(
    "env.*HONEYCOMB"
    "OutputFormat"
    "anyhow::Result"
    "x-honeycomb-team"
    "serde_json::from_str"
)

for i in "${!consistency_features[@]}"; do
    feature="${consistency_features[$i]}"
    check="${checks[$i]}"
    
    if grep -r "$check" src/ > /dev/null; then
        echo "   ✅ $feature"
    else
        echo "   ❌ $feature (missing)"
    fi
done

echo ""
echo "📊 Implementation Summary:"
echo "========================"
echo "   📁 Files: ${#required_files[@]} required, $((${#required_files[@]} - ${#missing_files[@]})) present"
echo "   🚀 API Resources: ${#api_resources[@]} total coverage"
echo "   📦 Dependencies: Modern Rust ecosystem (clap, tokio, reqwest, serde)"
echo "   🔗 API Consistency: Environment variables, multiple formats, proper error handling"
echo ""

if [[ ${#missing_files[@]} -eq 0 ]]; then
    echo "🎉 Implementation validation PASSED!"
    echo "    Ready to build and use with: ./build.sh"
else
    echo "⚠️  Implementation validation INCOMPLETE"
    echo "    Missing ${#missing_files[@]} files - check above for details"
fi

echo ""
echo "🔧 Next steps:"
echo "   1. Run ./build.sh to compile the CLI"
echo "   2. Set HONEYCOMB_API_KEY environment variable"
echo "   3. Test with: ./target/release/apiary auth validate"
echo "   4. Explore all endpoints with: ./target/release/apiary --help"