# Devlog

## June 2nd 2023

Started this project back up, copied it over from wt-webdev into a new
project of its own.

1. make a new cargo project with proper dependencies
2. move frontend into new folder in cargo manifest dir (parent of src)
3. axum has changed how it works so now I'm using hyper::serve
4. axum got rid of spa for some reason so im using tower_http::ServeDir
5. it needs to serve from the root or else there's problems

DEPLOYING
1. cross compiling to linux doesnt really work so instead we need to install cargo and node on our vm
2. Login to the vm as the root (sysadmin basically)
    ssh root@ipaddress
3. it'll prompt for a password. it won't show. type that in.
4. we need to create a user and give them sudo permission.
   adduser username
    *it should prompt for a password, make it strong
   usermod -aG sudo username
5. ~~before we logout from root, lets install cargo~~ jk that'll only work for root if we do it now, do it after logging in as the new user
    curl https://sh.rustup.rs -sSf | sh
6. logout from root and login as the user name using ssh username@ipaddress and the new password
7. ~~we need to install nodejs~~ skip this step, upload the whole node modules folder, it'll be faster. If you choose to ignore this advice, download the tarball manually and copy paste its contents according to resource 2. 
    ~~sudo apt install nodejs~~
    ~~sudo apt install npm~~
    ~~ sudo npm install --global yarn~~
8. set the port to 8080 (for now)  and the port to 0.0.0.0
8. now use winscp to put your project files on the machine,~~ initialize the frontend~~ and build the backend.
    ~~in the 'frontend' folder : "yarn install"~~
    cargo build -r 
9. it should be reachable from the outside world now. 
10. later we have to configure a systemctl to launch it at startup but yea, thats all for now. 



resources 
https://www.redhat.com/sysadmin/install-apache-web-server

https://stackoverflow.com/questions/57957973/npm-cannot-find-module-semver-after-reinstall

for putting a gui on it : https://www.youtube.com/watch?v=ODhGNe0s4lI

