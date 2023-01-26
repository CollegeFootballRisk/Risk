# Getting Started
**NOTE: This version is for version 0.2.x**

## Table of contents

- [Getting Started](#getting-started)
  - [Requirements](#requirements)
  - [Installation of Rust](#installation-of-rust)
  - [Installation of Postgresql](#installation-of-postgresql)
  - [Installation of Rust-Risk](#installation-of-rust-risk)
  - [Reddit Setup](#reddit-setup)
  - [Rust-Risk Setup](#rust-risk-setup)
    - [Postgresql Setup](#postgresql-setup)
    - [Territories and Map Setup](#territories-and-map-setup)
    - [Team Setup](#team-setup)
    - [Environmental Variables](#environmental-variables)
  - [Building the Server and Ringmaster](#building-the-server-and-ringmaster)
    - [Building the Server](#building-the-server)
    - [Building the Ringmaster](#building-the-ringmaster)
    - [Setting up Cron](#setting-up-cron)
  - [Setting up NGINX](#setting-up-nginx)
  - [Starting the Server](#starting-the-server)
  - [Running the Ringmaster](#running-the-ringmaster)

## Requirements
To begin, you'll need a server or computer with at least 512 mb or more RAM and at least 8 GB of free space (Rust builds take a lot of storage space). Preferably, you will also have an existing NGINX (or Apache) configuration and a domain name or subdomain setup (see NGINX setup, way below). You will need [Git](https://git-scm.com/download/win) installed.

**NOTE:** I have not tested this program on Windows and do not know if it works, but have included a rough how-to for Windows in case it does. As very few servers use Windows, it is not a priority.

Other programmes you will need access to:
* Inkscape - Optional, to create maps
* Libreoffice Calc OR Microsoft Office Excel OR Google Docs (OR some programming language) - Optional, to populate map adjacency

## Installation of Rust
To install this programme, install Rust nightly for your system. This can be accomplished with rustup or (in Unix-like operating systems) install.sh:

**NOTE: YOU NEED TO SELECT "NIGHTLY" RUST WHEN PROMPTED BETWEEN NIGHTLY/STABLE/BETA**

### Unix-like Systems
You can use the install.sh file at the root of this directory. Clone this repo, as per [Rust Risk Setup](#rust-risk-setup) and then execute

 ```bash
 ./install.sh -b
 ```
 
 Alternatively, install Rust with Rustup:

```bash
curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly
```

### Windows, Others
See [Rust Installation](https://forge.rust-lang.org/infra/other-installation-methods.html).


## Installation of Postgresql
To install this programme, install Postgresql for your system. 

See [https://www.postgresql.org/download/](https://www.postgresql.org/download/). 

## Installation of Rust-Risk
To install Rust-Risk, you will first clone this repository. Go to a comfortable place for new files, such as `$HOME/Projects/` on Unix-like Systems or `C:\Users\{Your Username}\Projects` on Windows. Then clone this repository using Git:

### Unix-like Systems
```bash
git clone https://github.com/mautamu/Risk.git
```

### Windows, Others
Using CMD or git,
```bash
git clone https://github.com/mautamu/Risk.git
```

## Reddit Setup
You will now need to create an application in Reddit. If you have a domain name already pointing to your server, then use that. If not, we will use localhost (but this should not be for production use).

1. Go to https://www.reddit.com/prefs/apps
2. Click "Create another app..."
3. Set the name to "{Name}-Risk" (append -local if you're using localhost)
4. Add a short description, e.g. "The University of Alabama Rust-Risk Installation"
5. Add an about url that points exactly to the /info endpoint on your domain (`https://aggierisk.com/info` for example). If using local, use `http://localhost:8000/info`. 
6. **IMPORTANT:** Reddit is very picky about the redirect_url. Your port number and domain must match your FQDN (domain name) EXACTLY. `http://localhost:8000/auth/reddit` is NOT `https://localhost:8000/auth/reddit` is not `http://localhost/info`, etc. For production use, it will look similar to: `https://aggierisk.com/auth/reddit`. For local use, it will probably be `http://localhost:8000/auth/reddit`.
7. Click "Create App." Make note of your application id (hereafter, APP_ID), which is displayed right under the blue application name next to the logo (currently a question mark). Also make note of your secret, (hereafter, SECRET), and your redirect url (hereafter, REDIRECT_URL).
8. (Optional) If you have already created a logo, upload it now.

## Rust-Risk Setup

### Postgresql Setup

First, we'll create a new database and user named `risk` (if you're not familiar with SQL, look up "Create new user postgresql" for graphical approaches):

1. Login as postgres (see [https://wiki.postgresql.org/wiki/First_steps](https://wiki.postgresql.org/wiki/First_steps)). 
2. Execute (substituting %password% with a good password you don't use anywhere else, perhaps [a random string](https://www.random.org/passwords/)):
```sql
create database risk;
create user risk with password '%password%';
```
3. Now we will run `psql risk -f new.sql`. You will need to know to where you downloaded Rust-Risk, and navigate to it (on Command Prompt, `cd C:\Users\{{your user}}\Projects\Risk\db`, on POSIX Shell, `cd $HOME/Projects/Risk/db/`). Then `psql risk -f new.sql` (it may require you login, in which case append `-U risk -p %password%` again substituting your password for %password%).
4. The step above should complete without errors. If it argues about Citext or something else, feel free to contact me or look into it on your search engine of choice.

### Territories and Map Setup
If you want to just use the Texas map and territories, run texas.sql like in step 3 above.

If not, you will need to generate a map. There are many ways to do this, but the best is to download a map from Wikipedia. Look for 'SVG county map of (entity)' and find an **SVG** where each county is its own path (e.g. https://commons.wikimedia.org/wiki/File:Alabama_counties.svg). To ensure license compliance, try to select a public domain or attribution-licensed image. Wikimedia has a lot of those. 

(For those creating a new map only):
1. Open the SVG in inkscape and select the first county, then go to the lower bar with all the colors and select a color, like red. The county should turn red. Do this for all the counties you want in a territory. Then go to the next county set and choose blue or another color, etc, so that all territories are different colors. 
2. Select a territory that is red, for example. Then go to `Edit> Select Same > Fill Color` then hit CTRL and + and + (or `Path> Union`). Repeat for all territories/colors. If some internal lines are left, (`Path> Outset`) and shift+click are your friends to delete the internal nodes. 
3. For each territory, you will need to edit some things in XML. First, you will need to provide a region, e.g. 1, an id that is the territories name without ANY spaces, and finally the territories name. Click xml editor (right hand panel `<>`). Then click each territory and make it look like so:
 
    | id     |  CollegeStation |
    |--------|:---------------:|
    | region | 1               |
    | name   | College Station |
    | style  |  (don't change) |
    | d      | (don't change)  |


4. Save the SVG and open in Notepad, Notepad++, Firefox, etc. Then after the last `</g>` tag, add:
```
    <foreignobject
     x="00"
     y="400"
     width="300"
     height="150">
    <body>
      <div style="font-size:32px;"><b id="map-county-info">{{State}}, USA</b><br/></div>
      <div
         color="white" id="map-owner-info"></div>
         <br/>
      <div class="action-container">
      <button id="attack-button" class="disp-button" disabled="">Attack</button>
      <button id="defend-button" class="disp-button" disabled="">Defend</button>
      <button id="visit-button" class="disp-button" disabled="">History</button>
      <button id="hover-button" class="hover-button disp-button" onclick="returnHover();" disabled="">Hover</button></div>
    </body>
  </foreignobject>
  ```

5. Replace {{State}} with whatever you please, and adjust x and y as necessary.

6. Save the image to Risk/server/static/assets/images/{{mapname}}.svg and edit Risk/server/static/assets/scripts/main.js line 20:    
```
map: '/images/map4.svg',
```
 to match, e.g. if 'bama:
 ```
 map: '/images/alabama.svg',
 ```

7. Phew, we're now about half done. Next is to insert territories. The easiest way to do this is to make a list of territory names in excel and numbers like: `id | name | region` where id = 1, 2, 3, and name = Abilene, Alpine, Bryan . . . Then use `concat()` to develop the sql query: `insert into territories (id, name, region) values (id, name, region);` e.g.: `=concat('insert into territories (id, name, region) values (',A1,',',B1,',',C1'); ')` then to copy all of these and paste into postgresql to execute them.

8. You'll next want to prepare a list of territories and their neighbors (include the territory itslef as its first neighbor). The easiest way I've found is to make a column in Libreoffice Calc/Excel that is the territory names alphabetically, then for each territory write its neighbors across the row in columns B-J. Then go to replace each territory with its territory number, and finally use the same concat trick as in 7:
```
=IF(ISBLANK(F2),"",CONCAT("insert into territory_adjacency(territory_id, adjacent_id) values (",$B2,",",F2,");"))
```
which results in
```
insert into territory_adjacency(territory_id, adjacent_id) values (1,1);
```
**IMPORTANT**: MAKE SURE EACH TERRITORY IS INCLUDED AS ITS OWN NEIGHBOR!

9. Run the queries generated by 8 in sql.

### Team Setup
1. Now insert teams into sql using the following query (replace team names, the two colors, and the logo with whatever you like):
```sql
insert into teams (tname, tshortname, color_1, color_2, logo, seasons) values ('Team Name', 'Team', 'rgba(0,0,0,0.5)', 'rgba(255,255,255,0.5)', '/images/houston.svg', '{1}');
```
2. Now set territory ownership. Your ownership query will be like (see step 8 of Territories and Map Setup to see how to do this using concat in excel for all territories):
```sql
insert into territory_ownership (territory_id, owner_id, day, season, random_number) values (1, 1, 1, 1, 0);
```
3. Now set turninfo:
```sql
insert into turninfo (season, day, complete, active, finale, chaosrerolls, chaosweight) values (1, 1, false, true, false, 0, 1);
```
4. For each team, upload "day 0" stats (replace {{team}} with team's id number and {{territories}} with the number of territories they own):
```sql
insert into stats (sequence, season, day, team, rank, territorycount, playercount, merccount, starpower, efficiency, effectivepower, ones, twos, threes, fours, fives) values (0, 1, 1, {{team}}, 1, {{territories}}, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
```

### Environmental Variables
Enter the server/ directory. Edit Rocket.toml (replacing the {{}} as above, and change ports if necessary):
```toml
[global.databases.postgres_global]
url = "postgresql://{{username}}:{{password}}@{{hostname}}:{{port}}/{{database}}"

[global.oauth.reddit]
provider = "Reddit"
client_id = "{{APP_ID}}"
client_secret = "{{SECRET}}"
redirect_uri = "{{same redirect_URI as above, leave the apostrophes not the brackets}}"

[global.oauth.discord]
provider = "Discord"
client_id = "{{APP_ID}}"
client_secret = "{{SECRET}}"
redirect_uri = "{{same redirect_URI as above, leave the apostrophes not the brackets}}"

[global.risk]
name = "{{The name of service you want}}"
base_url = "{{The base url, e.g. localhost:8000 or aggierisk.com}}"
cookie_key = "{{base64 string, DO NOT USE THE SAME AS secret_key}}"

[default]
address="127.0.0.1"
port=8080
log_level="normal"
keep_alive=5
limits = { forms = 32768 }
secret_key = "{{base64 string, see rocket.rs/, DO NOT USE THE SAME AS cookie_key}}"
```

That should be all the configuration, it only gets easier from here!

## Building the Server and Ringmaster
### Building the Server
1. Enter Risk/server
2. Run `cargo build --release`. This will take about ten minutes the first time but gets faster later.
3. Make sure it runs, type `cargo run --release` and navigate to http://localhost:8000/ (if on local machine, if not we can't test yet, gotta set up NGINX).

### Building the Ringmaster
1. Enter Risk/ringmaster
2. Run `cargo build --release`. This will take about ten minutes the first time but gets faster later.

### Setting up Cron
On unix-like systems, you can setup cron by running `crontab -e`. These are the settings I use, where /var/www/Risk/ is the path to my Risk installation:
```cron
0 4 * * * cd /var/www/Risk/ringmaster/ && ./target/release/ringmaster
0,15,30,45 * * * * /bin/sh /var/www/Risk/server/run.sh >> ~/cron.log 2>&1
```

## Setting up NGINX
(for non-localhost installations only):
Here's the file that I use in /etc/nginx/sites-enabled/aggierisk.com.conf/
Replace aggierisk.com with your domain.
```
server {
    server_name aggierisk.com;
    root /var/www/aggierisk.com/;
    client_max_body_size 10M;
    access_log /var/log/nginx/aggierisk.com-access.log;
    error_log /var/log/nginx/aggierisk.com-error.log;
        location /  {
        proxy_set_header X-Real-IP  $remote_addr;
        proxy_set_header X-Forwarded-For $remote_addr;
        proxy_set_header Host $host;
        proxy_pass http://127.0.0.1:8080;
        }
    listen [::]:80;
    listen 80;
}
```

Then run `nginx -t` to ensure it's valid. Then restart nginx:
`sudo systemctl restart nginx`. 

Optional: certbot setup, see https://certbot.eff.org/docs/using.html


## Starting the Server
To start the server, cd to Risk/server and type `sh run.sh` (Unix-like OSes). Idk if it even runs on Windows.

## Running the Ringmaster
To run the ringmaster/change days, cd to Risk/ringmaster and type `cargo run --release` (Unix-like OSes). Idk if it even runs on Windows.
