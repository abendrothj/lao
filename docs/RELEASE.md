# 🚀 LAO Release Process

This guide explains how to push a tag and trigger automated builds for all platforms.

## 📋 Prerequisites

1. **Git Repository**: Ensure you're in a git repository with remote origin
2. **GitHub CLI** (optional): For enhanced workflow management
3. **Clean Working Directory**: All changes committed or stashed

## 🎯 Quick Release Process

### **Option 1: Automated Release Script**
```bash
# Run the automated release script
bash scripts/release.sh

# Or run pre-release checks only
bash scripts/release.sh check

# Or see manual steps
bash scripts/release.sh manual
```

### **Option 2: Manual Release Process**

#### **1. Update Version**
```bash
# Update version in Cargo.toml (if needed)
sed -i 's/^version = ".*"/version = "0.1.0"/' Cargo.toml
```

#### **2. Commit Changes**
```bash
git add .
git commit -m "Prepare release v0.1.0"
```

#### **3. Create and Push Tag**
```bash
# Create annotated tag
git tag -a v0.1.0 -m "Release LAO v0.1.0

🎉 LAO v0.1.0 Release by Jake Abendroth

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
Download packages from GitHub Releases

## Support
For issues and support, contact Jake Abendroth at contact@jakea.net
Repository: https://github.com/abendrothj/lao"

# Push tag to trigger builds
git push origin v0.1.0
```

#### **4. Monitor Builds**
```bash
# Check GitHub Actions (automatically triggered)
open https://github.com/abendrothj/lao/actions

# Or trigger manually with GitHub CLI
gh workflow run package.yml -f platform=all
```

## 🔄 What Happens After Tag Push

### **Automatic Triggers**
1. **GitHub Actions**: Detects tag push (`v*`)
2. **Multi-Platform Builds**: Runs on Linux, macOS, Windows
3. **Package Creation**: Generates platform-specific packages
4. **Release Creation**: Automatically creates GitHub Release

### **Build Timeline**
- **Linux**: ~8 minutes
- **macOS**: ~10 minutes  
- **Windows**: ~12 minutes
- **Total**: ~15 minutes for all platforms

## 📦 Generated Packages

After successful builds, packages will be available in GitHub Releases:

### **Linux Packages**
- `lao_0.1.0_amd64.deb` - Debian/Ubuntu package
- `lao-0.1.0-1.*.rpm` - Red Hat/Fedora package
- `lao-0.1.0-linux-x86_64.tar.gz` - Universal Linux archive
- `LAO-0.1.0-x86_64.AppImage` - Portable Linux app

### **macOS Packages**
- `LAO-0.1.0.dmg` - macOS disk image installer
- `lao-0.1.0-macos-x86_64.tar.gz` - Universal macOS archive

### **Windows Packages**
- `lao-0.1.0.msi` - Windows installer package
- `lao-0.1.0-windows-x86_64.zip` - Portable Windows archive

## 🛠️ Manual Package Creation

If you want to create packages locally:

```bash
# Create packages for current platform
bash scripts/create-simple-packages.sh

# Create all package formats
bash scripts/create-packages.sh
```

## 🔍 Troubleshooting

### **Common Issues**

#### **Tag Already Exists**
```bash
# Check existing tags
git tag -l

# Delete local tag
git tag -d v0.1.0

# Delete remote tag
git push origin --delete v0.1.0
```

#### **Uncommitted Changes**
```bash
# Commit changes
git add .
git commit -m "Prepare release"

# Or stash changes
git stash
```

#### **No Remote Origin**
```bash
# Add remote origin
git remote add origin https://github.com/abendrothj/lao.git
```

#### **GitHub CLI Not Authenticated**
```bash
# Authenticate with GitHub
gh auth login

# Check authentication status
gh auth status
```

### **Build Failures**

#### **Check Build Logs**
```bash
# View GitHub Actions
open https://github.com/abendrothj/lao/actions

# Check specific workflow run
gh run list --workflow=package.yml
gh run view <run-id>
```

#### **Local Testing**
```bash
# Test build locally
cargo build --release

# Test package creation
bash scripts/create-simple-packages.sh
```

## 📊 Release Checklist

Before pushing a tag, ensure:

- [ ] Version updated in `Cargo.toml`
- [ ] All tests passing (`cargo test`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if exists)
- [ ] Working directory clean
- [ ] Remote origin configured
- [ ] Tag doesn't already exist

## 🎉 Success!

After pushing a tag:

1. **Monitor**: Watch GitHub Actions at https://github.com/abendrothj/lao/actions
2. **Wait**: ~15 minutes for all builds to complete
3. **Review**: Check generated packages in GitHub Releases
4. **Test**: Download and test packages on target platforms
5. **Announce**: Share release with users

## 📞 Support

For issues with the release process:
- **Email**: contact@jakea.net
- **Repository**: https://github.com/abendrothj/lao
- **Issues**: https://github.com/abendrothj/lao/issues
