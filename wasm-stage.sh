# instructions:
# starting from root
# go to -> crates/ 
# go through all of the directories within crates like:
# 	oxide/
# 	snake/
#
# next run this command within each directory:
# set -euo pipefail

# RUSTFLAGS='--cfg getrandom_backend="wasm_js"' \
#    wasm-pack build --target web --no-default-features	
#
# Next you'll get something like this:
# 	oxide/
# 		pkg/
# 	snake/
# 		pkg/
#
# Next we want to move these pkg build files into to root docs/ foler for github page deployment.
#
# But we have to rename them first from pkg to their appropriate name like pkg -> oxide, pkg -> snake
#
# we should get something like this:
# docs/
# 	index.html # this is always here
# 	oxide/
# 	snake/
# 	...
