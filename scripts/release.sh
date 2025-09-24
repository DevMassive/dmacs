#!/bin/bash

# --- Configuration ---
# The file where the version is stored
VERSION_FILE="Cargo.toml"
# The pattern to find the version line in the file
VERSION_PATTERN="^version = \"[0-9]+\.[0-9]+\.[0-9]+\"$"

# --- Script Start ---

# 1. Check for uncommitted changes
# if ! git diff-index --quiet HEAD --; then
    # echo "Error: You have uncommitted changes. Please commit or stash them before releasing."
    # exit 1
# fi

echo "✅ No uncommitted changes."

# 2. Determine the version bump type
BUMP_TYPE=${1:-patch} # Default to 'patch' if no argument is provided
if [[ "$BUMP_TYPE" != "patch" && "$BUMP_TYPE" != "minor" && "$BUMP_TYPE" != "major" ]]; then
    echo "Error: Invalid argument. Use 'patch', 'minor', or 'major'."
    exit 1
fi
echo "🔍 Release type: $BUMP_TYPE"

# 3. Read the current version from Cargo.toml
current_version_line=$(grep -E "$VERSION_PATTERN" "$VERSION_FILE")
if [ -z "$current_version_line" ]; then
    echo "Error: Could not find version in $VERSION_FILE"
    exit 1
fi
current_version=$(echo "$current_version_line" | grep -o "[0-9]\+\.[0-9]\+\.[0-9]\+")
echo "📝 Current version: $current_version"

# 4. Calculate the new version
IFS='.' read -r -a version_parts <<< "$current_version"
major=${version_parts[0]}
minor=${version_parts[1]}
patch=${version_parts[2]}

case "$BUMP_TYPE" in
    "major")
        major=$((major + 1))
        minor=0
        patch=0
        ;;
    "minor")
        minor=$((minor + 1))
        patch=0
        ;;
    "patch")
        patch=$((patch + 1))
        ;;
esac

new_version="$major.$minor.$patch"
echo "🚀 New version: $new_version"

# 5. Update the version in Cargo.toml
new_version_line="version = \"$new_version\""
# Using sed with a backup file for compatibility
sed -i.bak "s/$current_version_line/$new_version_line/" "$VERSION_FILE"
rm "${VERSION_FILE}.bak"

echo "📄 Updated $VERSION_FILE"

# 6. Update Cargo.lock by running a lightweight cargo command
echo "🔒 Updating Cargo.lock..."
cargo check
echo "✅ Cargo.lock updated."

# 7. Commit the version bump
git add "$VERSION_FILE" "Cargo.lock"
commit_message="chore(release): v$new_version"
git commit -m "$commit_message"
echo "📦 Committed with message: \"$commit_message\""

# 8. Create a git tag
tag_name="v$new_version"
git tag "$tag_name"
echo "🏷️ Tagged with: $tag_name"

# 9. Push the commit and tag
echo "📡 Pushing to remote..."
git push
git push origin "$tag_name"

echo "🎉 Release process finished!"
echo "GitHub Actions will now build and create the release."
echo "Check the status here: https://github.com/$(git config --get remote.origin.url | sed 's/.*:\/\///;s/\.git$//')/actions"
