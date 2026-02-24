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

## 10/16/2025
Ok , updated the astro version. There were major breaking changes - content collections are 
a more complicated but flexible replacement for 'posts'. Also the client router is now built into astro so I dont need flamethrower anymore.

I'm going to start a major redesign also, using picocss and flexbox-grid, so I can do some interactive 
stuff with svelte. 

```yarn add sass @picocss/pico flexboxgrid-sass```

I think i just import these in the layout now? 
I want to restrict this to a post first so i'll make a separate layout. 

moved posts to src/posts and left just the dynamic route in src/pages/posts to prevent duplicate articles
apparently the layout frontmatter property isnt special anymore? 
I think that means i have to have 1 layout for all my posts right? 
Broadly what im trying to do here is have different layouts for different posts... so that sucks.
It does work though so unfortunately I think I have to rely on it.

Ok i'm going to get some air then lets make a page with pico. 
I think i should start by seeing the unstyled markdown looks like. 

## 10/24/25
Ok i'm done optimizing for now, theres no free image cdn that im happy with.

KV Store
Rust Macros - Crabtime
Typescript Macros 

## 11/10/25
Captcha ideas - word associations, svg coloring, rhythm game

Want to make a big or customizeable details header thing but ugh formatting this articles a pain and its still a lot of work left to be done...

## 11/22/25
Deploying on linode 
bought a server for 5 bucks
adduser d 
passwd d (prompts for new password)
usermod -aG wheel d
sudo dnf install samba
sudo systemctl enable smb --now

//oop i didnt have samba on my own system so on mine
sudo dnf install samba-client cifs-utils

//back to the server
sudo smbpasswd -a d  //create a samba user with the same name
sudo nano /etc/samba/smb.conf //set [homes]/browseable to Yes

welp shit didnt work . apparently it sucks over high latency connections anyway. 
nevermind samba, dolphin worked immediately with sftp instead of smb. 

the big problem ended up being firewalld - me not knowing it existed thanks to old ass
stck overflow posts mainly.

# 11/23/25
Ok deployment time - i enabled caching and compression on the server, as well as 
the cache control header. 

i think i need to 
- create a new nologin user 'webserver'
    //shell is nologin and -r means no home dir
    sudo useradd webserver -s /sbin/nologin -r
    //userdel -r username if you mess up, chsh -s if you want to change the shell later.
- put the server files in a directory owned by root so they cant be written to by the new user, probably opt/website
    sudo cp -r ~/server /opt/server
    // ' ls -l ' to check permissions, it should be r-x for the rightmost 3 (any user)
- create a systemd service that runs the website with the cwd set to /opt/website and the user set to 'webserver'
    
Here's what i've got for systemd
    /etc/systemd/system/portfolio-server.service

    [Unit]
    Description=My Portfolio's Web Server
    After=network.target

    [Service]
    User=webserver
    WorkingDirectory=/opt/server
    ExecStart=/opt/server/release/portfolio
    Restart=on-failure

    [Install]
    WantedBy=multi-user.target

then we have to setcap it so it can run on port 80
    sudo setcap 'cap_net_bind_service=+eip' /opt/server/release/portfolio
enable it
    sudo systemctl enable portfolio-server
    systemctl status portfolio-server
//potentially, start it
    systemctl status portfolio-server

Then to limit the amount of logs it keeps
    # /etc/systemd/journald.conf
    [Journal]
    SystemMaxUse=1G 

Dont forget to open the ports in firewalld 

    sudo firewall-cmd --add-port=80/tcp --permanent
then
    sudo systemctl restart firewalld

# ok so generally now the process for updating the frontend is 

in dolphin sftp://d@{server_ip}
copy the files from ./frontend/dist over to ~/server/frontend/dist
OR DO 
rsync -a frontend/dist/* d@{server_ip}:~/server/frontend/dist

    ssh d@{server_ip}
    sudo cp -r ~/server/frontend/dist /opt/server/frontend
    sudo systemctl restart portfolio-server
then in cloudflare enable developer mode to turn off caching

# to update the backend 
in dolphin sftp://d@{server_ip} 
copy  ./target/release/portfolio over to ~/server/release/portfolio
    ssh d@{server_ip}
    sudo systemctl stop portfolio-server
    sudo cp -r ~/server/release/portfolio /opt/server/release/portfolio
    sudo setcap 'cap_net_bind_service=+eip' /opt/server/release/portfolio
    sudo systemctl start portfolio-server
then in cloudflare enable developer mode turn caching off and on again.

# installing snap
sudo dnf install snapd 
sudo systemctl enable snapd
sudo systemctl enable snapd.apparmor
sudo snap install snap-store
