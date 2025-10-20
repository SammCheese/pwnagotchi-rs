#!/bin/bash

WORKSPACE_ROOT=$(cargo locate-project --workspace --message-format plain | xargs dirname)
DIST_DIR="$WORKSPACE_ROOT/dist/plugins"
PROFILE="${1:-debug}"

mkdir -p "$DIST_DIR"

echo "Building plugins to: $DIST_DIR"
for plugin_dir in pwnagotchi-plugin-dev/src/plugins/*; do
  if [ -f "$plugin_dir/Cargo.toml" ]; then
    plugin_name=$(basename "$plugin_dir")
    echo "Building plugin: $plugin_name"
    
    if [ "$PROFILE" = "release" ]; then
      cargo build --manifest-path "$plugin_dir/Cargo.toml" --release
    else
      cargo build --manifest-path "$plugin_dir/Cargo.toml"
    fi
    
    if [ -f "$WORKSPACE_ROOT/target/$PROFILE/lib${plugin_name}.so" ]; then
      cp "$WORKSPACE_ROOT/target/$PROFILE/lib${plugin_name}.so" "$DIST_DIR/${plugin_name}.so"
      echo "Copied ${plugin_name}.so to $DIST_DIR"
    else
      echo "Failed to find lib${plugin_name}.so in target/$PROFILE/"
    fi
  fi
done

if [ "$BUILD_EXAMPLES" = "1" ]; then
  for example_dir in pwnagotchi-plugin-dev/src/examples/*; do
    if [ -f "$example_dir/Cargo.toml" ]; then
      example_name=$(basename "$example_dir")
      echo "Building example: $example_name"
      
      if [ "$PROFILE" = "release" ]; then
        cargo build --manifest-path "$example_dir/Cargo.toml" --release
      else
        cargo build --manifest-path "$example_dir/Cargo.toml"
      fi

      # Copy to dist (examples build to workspace target dir)
      if [ -f "$WORKSPACE_ROOT/target/$PROFILE/lib${example_name}.so" ]; then
        cp "$WORKSPACE_ROOT/target/$PROFILE/lib${example_name}.so" "$DIST_DIR/${example_name}.so"
        echo "Copied ${example_name}.so to $DIST_DIR"
      else
        echo "Failed to find lib${example_name}.so in target/$PROFILE/"
      fi
    fi
  done
fi

echo "Done! Plugins available in: $DIST_DIR"