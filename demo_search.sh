#!/bin/bash

# 🎬 Clipboard History Manager - Search Demo Script
# This script demonstrates the powerful search capabilities of the clipboard manager

echo "🎬 =========================================="
echo "   CLIPBOARD HISTORY MANAGER - SEARCH DEMO"
echo "=========================================="
echo ""

# Colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to create colored output
print_section() {
    echo -e "${BLUE}🔹 $1${NC}"
    echo "----------------------------------------"
}

print_feature() {
    echo -e "${GREEN}✨ $1${NC}"
}

print_example() {
    echo -e "${YELLOW}💡 Example: $1${NC}"
}

print_section "INTELLIGENT SEARCH FEATURES"

print_feature "Exact Search"
echo "   Find clipboard entries with exact text matches"
print_example "Search for 'password' finds all entries containing that exact word"
echo ""

print_feature "Fuzzy Search"  
echo "   Smart matching even with typos and missing characters"
print_example "Search 'passowrd' (typo) still finds 'password' entries"
print_example "Search 'usr nm' finds 'username' entries"
echo ""

print_feature "Content-Type Detection"
echo "   Automatically identifies and categorizes content"
print_example "JSON: {\"key\": \"value\"} → Detected as JSON data"
print_example "URL: https://github.com → Detected as web URL"  
print_example "Multi-line text → Detected as formatted content"
echo ""

print_feature "Smart Previews"
echo "   Context-aware content previews with size information"
print_example "Large JSON → 'JSON: {\"data\": {...}} [1.2 KB]'"
print_example "Long URL → 'URL: https://example.com/... [156 B]'"
print_example "Code snippet → 'Code: function main() { ... [512 B]'"
echo ""

print_section "SEARCH CAPABILITIES"

echo -e "${CYAN}🔍 Search Methods:${NC}"
echo "   • Exact matching for precise results"  
echo "   • Fuzzy matching for flexible queries"
echo "   • Combined search (exact + fuzzy results)"
echo "   • Case-insensitive by default"
echo ""

echo -e "${CYAN}📊 Content Analysis:${NC}"
echo "   • JSON structure detection and validation"
echo "   • URL pattern recognition and validation"
echo "   • Multi-line text formatting detection"
echo "   • Code snippet identification"
echo "   • Binary content filtering"
echo ""

echo -e "${CYAN}🎯 Smart Features:${NC}"
echo "   • Content size formatting (B/KB/MB)"
echo "   • Timestamp formatting (YYYY-MM-DD HH:MM:SS)"
echo "   • Duplicate entry prevention"
echo "   • Content truncation with size indicators"
echo ""

print_section "DEMO SCENARIOS"

echo -e "${PURPLE}📝 Scenario 1: Code Search${NC}"
echo "   Query: 'function'"
echo "   Results: All clipboard entries containing function definitions"
echo "   Preview: 'Code: function getUserData() { ... [234 B]'"
echo ""

echo -e "${PURPLE}🌐 Scenario 2: URL Search${NC}"
echo "   Query: 'github'"  
echo "   Results: All GitHub URLs in your clipboard history"
echo "   Preview: 'URL: https://github.com/user/repo [45 B]'"
echo ""

echo -e "${PURPLE}📋 Scenario 3: JSON Data Search${NC}"
echo "   Query: 'api'"
echo "   Results: API responses and JSON configurations"
echo "   Preview: 'JSON: {\"api_key\": \"...\", \"status\": ...} [1.1 KB]'"
echo ""

echo -e "${PURPLE}🔤 Scenario 4: Fuzzy Text Search${NC}"  
echo "   Query: 'usr pwd' (typo/abbreviation)"
echo "   Results: Entries containing 'username password'"
echo "   Preview: 'Text: username: admin, password: ... [89 B]'"
echo ""

print_section "USAGE STATISTICS"

echo -e "${CYAN}📈 Track Your Clipboard Usage:${NC}"
echo "   • Total items stored"
echo "   • Total content size (MB/GB)"  
echo "   • Average item size"
echo "   • Largest item size"
echo "   • Most active content types"
echo ""

print_section "PERFORMANCE & LIMITS"

echo -e "${GREEN}⚡ Performance Features:${NC}"
echo "   • Fast in-memory search indexing"
echo "   • Async operations for responsiveness"
echo "   • Smart caching for repeated searches"
echo "   • Optimized preview generation"
echo ""

echo -e "${YELLOW}📏 Content Limits:${NC}"
echo "   • Maximum content size: 10 MB per item"
echo "   • History limit: 1,000 items (configurable)"
echo "   • Preview limit: 200 characters"
echo "   • Search results: Unlimited (performance-optimized)"
echo ""

print_section "REAL-WORLD USE CASES"

echo -e "${PURPLE}💼 Developer Workflows:${NC}"
echo "   • Find API keys and tokens quickly"
echo "   • Locate code snippets by function name"
echo "   • Search configuration files by service"
echo "   • Track error messages and logs"
echo ""

echo -e "${PURPLE}📊 Data Management:${NC}"
echo "   • Find JSON configs by property name"
echo "   • Locate CSV data by column headers"
echo "   • Search database connection strings"
echo "   • Track data transformation results"
echo ""

echo -e "${PURPLE}🌐 Web Development:${NC}"
echo "   • Find URLs by domain or endpoint"
echo "   • Locate HTML snippets by tag"
echo "   • Search CSS by selector or property"
echo "   • Track JavaScript function signatures"
echo ""

print_section "GETTING STARTED"

echo -e "${GREEN}🚀 Installation:${NC}"
echo "   1. Download binary for your platform"
echo "   2. Make executable: chmod +x clipboard-history-*"  
echo "   3. Run: ./clipboard-history-linux-x64"
echo ""

echo -e "${GREEN}🔍 Basic Search Commands:${NC}"
echo "   • Search exact: Enter search term"
echo "   • Fuzzy search: Enable fuzzy mode"
echo "   • View history: Show all items"
echo "   • Get stats: Display usage statistics"
echo ""

echo -e "${GREEN}⚙️ Advanced Features:${NC}"
echo "   • Configure content limits"
echo "   • Export/import clipboard history"
echo "   • Custom search filters"
echo "   • Automated content monitoring"
echo ""

echo ""
echo "🎯 =========================================="
echo "   Ready to supercharge your clipboard!"
echo "   Download from GitHub Releases"
echo "=========================================="
echo ""

print_example "Try searching for 'password', 'api', or 'function' to see the magic! ✨"
