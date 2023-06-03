# Devlog

## June 2nd 2023

Started this project back up, copied it over from wt-webdev into a new
project of its own.

1. make a new cargo project with proper dependencies
2. move frontend into new folder in cargo manifest dir (parent of src)
3. axum has changed how it works so now I'm using hyper::serve
4. axum got rid of spa for some reason so im using tower_http::ServeDir
5. it needs to serve from the root or else there's problems