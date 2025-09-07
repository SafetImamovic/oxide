#!/bin/bash
set -euo pipefail

# Root directories
CRATES_DIR="crates/examples"
DOCS_DIR="docs"
RESOURCES_DIR="resources"

# Ensure docs folder exists
mkdir -p "$DOCS_DIR"

# Loop through each directory in crates/
for crate_path in "$CRATES_DIR"/*/; do
    crate_name=$(basename "$crate_path")

    echo "Building crate: $crate_name"

    pushd "$crate_path" > /dev/null

    # Build with wasm-pack
    RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
    wasm-pack build --target web --no-default-features

    # Move and rename pkg folder to docs/<crate_name>
    if [ -d "pkg" ]; then
        echo "Moving pkg to $DOCS_DIR/$crate_name"
        rm -rf "../../../$DOCS_DIR/$crate_name"
        mv pkg "../../../$DOCS_DIR/$crate_name"

        # Copy resources to the docs folder
        if [ -d "$RESOURCES_DIR" ]; then
            echo "Copying resources to $DOCS_DIR/$crate_name/resources/"
            mkdir -p "../../../$DOCS_DIR/$crate_name/resources"
            cp -r "$RESOURCES_DIR"/* "../../../$DOCS_DIR/$crate_name/resources/"
        fi

        # Remove .gitignore files in the moved folder
        find "../../../$DOCS_DIR/$crate_name" -name ".gitignore" -type f -exec rm -f {} \;

        # Create an HTML entrypoint
        html_file="../../../$DOCS_DIR/$crate_name/index.html"
        echo "Creating HTML entrypoint at $html_file"
        cat > "$html_file" <<EOL
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Primjer | $crate_name</title>
    <style>
        canvas {
          background-color: black;
        }

        * {
          box-sizing: border-box;
          margin: 0;
          padding: 0;
        }

        html {
          padding: 2rem;
        }

        body {
          height: calc(100vh - 4rem);
          width: calc(100vw - 4rem);
        }
    </style>
</head>
<body>
    <canvas id="canvas"></canvas>
    <script type="module">
        import init from './${crate_name}.js';

        init().then(() => {
          console.log("WASM Loaded");
        })
        .catch(console.error);
    </script>
</body>
</html>
EOL

    else
        echo "Warning: pkg folder not found for $crate_name"
    fi

    popd > /dev/null
done

echo "All crates built, moved to $DOCS_DIR, resources copied, .gitignore files removed, and HTML entrypoints created."