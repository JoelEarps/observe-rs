#!/bin/bash
# Analyze binary sizes for different feature flag combinations
# Outputs a table suitable for README documentation

set -e

echo "🔨 Analyzing binary sizes for different feature combinations..."
echo "Note: For combinations without 'standalone', we measure the library crate size"
echo "      since the binary requires tokio (standalone feature)"
echo ""

# Array of feature combinations: "features" "description"
declare -a FEATURES=(
    "prometheus|Minimal (prometheus only)"
    "prometheus,standalone|Default (prometheus + standalone)"
    "prometheus,mock|Prometheus + mock (testing)"
    "prometheus,standalone,json-config|Prometheus + standalone + JSON config"
    "prometheus,standalone,yaml-config|Prometheus + standalone + YAML config"
    "prometheus,standalone,json-config,yaml-config|Prometheus + standalone + all config formats"
    "prometheus,standalone,axum-integration|Prometheus + standalone + Axum integration"
    "prometheus,otlp|Prometheus + OpenTelemetry"
    "prometheus,otlp,standalone|Prometheus + OpenTelemetry + standalone"
    "full|Full (all features)"
)

# Function to format size
format_size() {
    local bytes=$1
    local kb=$((bytes / 1024))
    local mb=$((kb / 1024))
    
    if [ $kb -lt 1024 ]; then
        echo "${kb} KB"
    else
        echo "${mb}.$(( (kb % 1024) * 100 / 1024 )) MB"
    fi
}

# Function to get size in KB
get_size_kb() {
    local bytes=$1
    echo $((bytes / 1024))
}

# Function to check if features include standalone
has_standalone() {
    local features=$1
    [[ "$features" == *"standalone"* ]]
}

# Build baseline first (minimal - library only)
echo "Building baseline (minimal library)..."
cargo clean --quiet
cargo build --release --lib --no-default-features --features prometheus --quiet 2>/dev/null
# Find the library file
lib_file=$(find target/release/deps -name "libobserve_rs*.rlib" 2>/dev/null | head -1)
if [ -z "$lib_file" ]; then
    echo "Error: Could not find library file" >&2
    exit 1
fi
baseline_size=$(stat -f%z "$lib_file" 2>/dev/null || stat -c%s "$lib_file" 2>/dev/null)
baseline_kb=$(get_size_kb $baseline_size)

echo ""
echo "## Build Size Comparison"
echo ""
echo "| Feature Combination | Description | Binary Size | Size (KB) | Relative to Minimal |"
echo "|---------------------|-------------|-------------|-----------|---------------------|"

# Process each feature combination
for item in "${FEATURES[@]}"; do
    IFS='|' read -r features description <<< "$item"
    
    echo "Building: $features" >&2
    
    # Clean and build
    cargo clean --quiet
    
    if has_standalone "$features"; then
        # Build example binary (crate has no default binary; standalone example uses standalone feature)
        if [ "$features" = "full" ]; then
            cargo build --release --features "$features" --quiet 2>/dev/null
            size_file="target/release/observe-rs"
        else
            cargo build --release --no-default-features --features "$features" --quiet 2>/dev/null
            size_file="target/release/observe-rs"
        fi
        size_file="target/release/examples/standalone_prometheus"
        size_type="binary"
    else
        # Build library only (no standalone, so binary won't compile)
        if [ "$features" = "full" ]; then
            cargo build --release --lib --features "$features" --quiet 2>/dev/null
        else
            cargo build --release --lib --no-default-features --features "$features" --quiet 2>/dev/null
        fi
        # Find the library file
        size_file=$(find target/release/deps -name "libobserve_rs*.rlib" 2>/dev/null | head -1)
        size_type="library"
    fi
    
    if [ ! -f "$size_file" ]; then
        echo "⚠️  Build failed for: $features" >&2
        continue
    fi
    
    # Get size
    size=$(stat -f%z "$size_file" 2>/dev/null || stat -c%s "$size_file" 2>/dev/null)
    size_kb=$(get_size_kb $size)
    size_formatted=$(format_size $size)
    
    # Calculate relative size
    diff_kb=$((size_kb - baseline_kb))
    if [ $diff_kb -eq 0 ]; then
        relative="0 KB (baseline)"
    elif [ $diff_kb -gt 0 ]; then
        relative="+${diff_kb} KB"
    else
        relative="${diff_kb} KB"
    fi
    
    # Add note about library vs binary
    if [ "$size_type" = "library" ] && [ "$features" != "prometheus" ]; then
        description="$description (lib only)"
    fi
    
    # Output table row
    echo "| \`$features\` | $description | $size_formatted | $size_kb KB | $relative |"
done

echo ""
echo "### Notes"
echo ""
echo "- All builds are in release mode (\`--release\`)"
echo "- Minimal build measures library crate size (binary requires \`standalone\` feature)"
echo "- Builds with \`standalone\` measure binary size; others measure library crate size"
echo "- Library crate size is what matters when used as a dependency"
echo "- Binary sizes can vary based on platform and Rust toolchain version"
echo "- For library usage, only the code you actually use will be included in your final binary"
echo ""
echo "✅ Analysis complete! Copy the table above to your README."
