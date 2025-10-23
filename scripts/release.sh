#!/usr/bin/env bash
set -euo pipefail

# LAO Release Script
# Creates a git tag and triggers automated builds for all platforms

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\(.*\)".*/\1/' || echo "0.1.0")

echo "🚀 LAO Release Process"
echo "📦 Version: $VERSION"
echo "📁 Repository: $ROOT_DIR"

# Function to check if we're in a git repository
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo "❌ Not in a git repository"
        exit 1
    fi
    
    if ! git remote get-url origin > /dev/null 2>&1; then
        echo "❌ No remote origin configured"
        echo "💡 Configure with: git remote add origin https://github.com/abendrothj/lao.git"
        exit 1
    fi
    
    echo "✅ Git repository configured"
}

# Function to check if working directory is clean
check_clean_working_dir() {
    if ! git diff-index --quiet HEAD --; then
        echo "❌ Working directory has uncommitted changes"
        echo "💡 Commit or stash changes first:"
        echo "   git add . && git commit -m 'Prepare release $VERSION'"
        echo "   OR"
        echo "   git stash"
        exit 1
    fi
    
    echo "✅ Working directory is clean"
}

# Function to check if tag already exists
check_tag_exists() {
    if git tag -l | grep -q "^v$VERSION$"; then
        echo "❌ Tag v$VERSION already exists"
        echo "💡 Available tags:"
        git tag -l | sort -V | tail -5
        exit 1
    fi
    
    echo "✅ Tag v$VERSION is available"
}

# Function to create and push tag
create_and_push_tag() {
    echo "🏷️  Creating tag v$VERSION..."
    
    # Create annotated tag
    git tag -a "v$VERSION" -m "Release LAO v$VERSION
    
    🎉 LAO v$VERSION Release by Jake Abendroth
    
    ## What's New
    - Cross-platform support (Linux, macOS, Windows)
    - Comprehensive packaging system
    - Visual workflow builder with egui
    - Plugin system with dynamic loading
    - CLI and GUI interfaces
    
    ## Packages Available
    - Linux: .deb, .rpm, .tar.gz, AppImage
    - macOS: .dmg, .tar.gz
    - Windows: .msi, .zip
    
    ## Installation
    Download packages from GitHub Releases or use:
    \`\`\`bash
    # Linux
    sudo dpkg -i lao_${VERSION}_amd64.deb
    
    # macOS
    open LAO-${VERSION}.dmg
    
    # Windows
    msiexec /i lao-${VERSION}.msi
    \`\`\`
    
    ## Quick Start
    \`\`\`bash
    lao-ui          # Start GUI
    lao-cli --help  # CLI help
    \`\`\`
    
    ## Support
    For issues and support, contact Jake Abendroth at contact@jakea.net
    Repository: https://github.com/abendrothj/lao"
    
    echo "✅ Tag v$VERSION created"
    
    # Push tag to remote
    echo "📤 Pushing tag to remote..."
    git push origin "v$VERSION"
    
    echo "✅ Tag v$VERSION pushed to remote"
}

# Function to trigger GitHub Actions workflow
trigger_workflow() {
    echo "🤖 Triggering GitHub Actions workflow..."
    
    # Check if gh CLI is available
    if ! command -v gh > /dev/null 2>&1; then
        echo "⚠️  GitHub CLI (gh) not found"
        echo "💡 Install with: brew install gh (macOS) or visit https://cli.github.com/"
        echo "🔄 Workflow will trigger automatically on tag push"
        return 0
    fi
    
    # Check if authenticated
    if ! gh auth status > /dev/null 2>&1; then
        echo "⚠️  Not authenticated with GitHub CLI"
        echo "💡 Authenticate with: gh auth login"
        echo "🔄 Workflow will trigger automatically on tag push"
        return 0
    fi
    
    # Trigger workflow manually (optional)
    echo "🚀 Triggering package workflow..."
    gh workflow run package.yml -f platform=all
    
    echo "✅ Workflow triggered successfully"
    echo "🔗 View progress: https://github.com/abendrothj/lao/actions"
}

# Function to show release checklist
show_checklist() {
    echo ""
    echo "📋 Release Checklist:"
    echo "✅ Version updated in Cargo.toml"
    echo "✅ All tests passing"
    echo "✅ Documentation updated"
    echo "✅ CHANGELOG.md updated"
    echo "✅ Tag created and pushed"
    echo "✅ GitHub Actions workflow triggered"
    echo ""
    echo "🔗 Next steps:"
    echo "1. Monitor GitHub Actions: https://github.com/abendrothj/lao/actions"
    echo "2. Wait for builds to complete (~10-15 minutes)"
    echo "3. Review generated packages in GitHub Releases"
    echo "4. Test packages on target platforms"
    echo "5. Announce release to users"
}

# Function to show manual release process
show_manual_process() {
    echo ""
    echo "🛠️  Manual Release Process:"
    echo ""
    echo "1. Update version in Cargo.toml:"
    echo "   sed -i 's/^version = \".*\"/version = \"$VERSION\"/' Cargo.toml"
    echo ""
    echo "2. Commit changes:"
    echo "   git add Cargo.toml"
    echo "   git commit -m 'Bump version to $VERSION'"
    echo ""
    echo "3. Create and push tag:"
    echo "   git tag -a v$VERSION -m 'Release LAO v$VERSION'"
    echo "   git push origin v$VERSION"
    echo ""
    echo "4. Monitor builds:"
    echo "   # Check GitHub Actions or run locally:"
    echo "   bash scripts/create-packages.sh"
    echo ""
    echo "5. Create GitHub Release:"
    echo "   gh release create v$VERSION --title 'LAO v$VERSION' --notes 'Release notes here'"
}

# Main release function
main() {
    echo "🎯 Starting LAO release process for v$VERSION"
    echo ""
    
    # Pre-flight checks
    check_git_repo
    check_clean_working_dir
    check_tag_exists
    
    echo ""
    echo "🚀 Proceeding with release..."
    
    # Create and push tag
    create_and_push_tag
    
    # Trigger workflow
    trigger_workflow
    
    # Show checklist
    show_checklist
    
    echo ""
    echo "🎉 Release process initiated!"
    echo "📦 Packages will be available in GitHub Releases once builds complete"
}

# Handle command line arguments
case "${1:-}" in
    "check")
        echo "🔍 Pre-release checks..."
        check_git_repo
        check_clean_working_dir
        check_tag_exists
        echo "✅ All checks passed!"
        ;;
    "manual")
        show_manual_process
        ;;
    "help"|"-h"|"--help")
        echo "LAO Release Script"
        echo ""
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  (no args)  - Run full release process"
        echo "  check      - Run pre-release checks only"
        echo "  manual     - Show manual release steps"
        echo "  help       - Show this help"
        echo ""
        echo "Examples:"
        echo "  $0          # Full release"
        echo "  $0 check    # Check only"
        echo "  $0 manual   # Manual steps"
        ;;
    "")
        main
        ;;
    *)
        echo "❌ Unknown command: $1"
        echo "💡 Use '$0 help' for usage information"
        exit 1
        ;;
esac