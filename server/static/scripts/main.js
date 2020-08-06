//initialize globals
var outstandingRequests = [];
var errorNotifications = [];


// JS is enabled, so hide that notif
document.getElementById('error-notif').style.display = "none";


//request handling
function doAjaxGetRequest(url, source, callback, errorcallback) {
    var instance_index = addUrlFromRequests(source, url);
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            callback(this);
            updateUrlFromRequests(instance_index, 1);
            // return JSON.parse(this.response);
        } else if (this.readyState == 4 && this.status != 200) {
            globalError = true;
            errorcallback(this);
            updateUrlFromRequests(instance_index, 1);
            //document.getElementsById("loadicon").classList.add("blink");
        }
    };
    xhttp.open("GET", url, true);
    xhttp.send();
}

function addUrlFromRequests(source, url) {
    var index = outstandingRequests.push({ source: source, url: url, state: 0 }) - 1;
    updateLoaderVisibility();
    return index;
}

function updateUrlFromRequests(index, status) {
    if (index > -1) {
        outstandingRequests[index].state = status;
    }
    updateLoaderVisibility();
}

function updateLoaderVisibility(forceHide = false) {
    let pending = false;
    for (i in outstandingRequests) {
        if (outstandingRequests[i].state == 0) {
            pending = true;
            break;
        }
    }
    if (pending == false && forceHide === false) {
        //stop loader
        document.getElementById("loadicon").classList.remove("spin");
    } else {
        //start loader
        //check if globalError
        document.getElementById("loadicon").classList.add("spin");
    }
}

// error handling

function errorNotif(title, body, button1, button2, resolveself = true, skipnotifcheck = false, errorIndex = 0) {
    if (skipnotifcheck != true) {
        errorIndex = errorNotifications.push({ title: title, body: body, button1: button1, button2: button2, status: 1, resolveself: resolveself }) - 1;
    }
    let vset = errorNotifications[errorIndex - 1] || { status: 0 };
    if (vset.status == 0) {
        document.getElementById('error-notif').style.display = "block";
        document.getElementById('error-notif-title').innerHTML = title || "General Error";
        document.getElementById('error-notif-body').innerHTML = body || "Hmm, no error was specified. Try notifying <a href=\"https://github.com/mautamu/risk\">u/Mautamu</a> if this issue persists.";
        document.getElementById('error-notif-button-1').innerHTML = button1.text || "";
        document.getElementById('error-notif-button-1-container').style.display = button1.display || "block";
        document.getElementById('error-notif-button-1').onclick = function() {
            try {
                if (typeof button1.action == "function") {
                    button1.action();
                }
                if (resolveself == true) {
                    errorNotifications[errorIndex].status = 0;
                }
            } finally {
                errorOver(errorIndex);
            }
        };
        document.getElementById('error-notif-button-2').innerHTML = button2.text || "";
        document.getElementById('error-notif-button-2-container').style.display = button2.display || "block";
        document.getElementById('error-notif-button-2').onclick = function() {
            try {
                if (typeof button2.action == "function") {
                    button2.action();
                }
                if (resolveself == true) {
                    errorNotifications[errorIndex].status = 0;
                }
            } finally {
                errorOver(errorIndex);
            }
        };
    }
}

function errorOver(errorIndex) {
    if (errorNotifications[errorIndex].status == 0) {
        //move to next one or hide
        let pending = false;
        for (i in errorNotifications) {
            if (errorNotifications[i].status != 0) {
                pending = true;
                errorNotif(errorNotifications[i].title, errorNotifications[i].body, errorNotifications[i].button1, errorNotifications[i].button2, errorNotifications[i].resolveself, true, i);
                break;
            }
        }
        if (pending == false) {
            document.getElementById('error-notif').style.display = "none";
        }
    } else {
        //do nothing
    }
}


function getUserInfo(resolve, reject) {
    try {
        doAjaxGetRequest('/api/me', 'UserLoader', function(userObject) {
                window.userObject = JSON.parse(userObject.response);
                //see if user has a team, if not, prompt them and halt
                let active_team = window.userObject.active_team || {
                    name: null
                };
                if (active_team.name == null) {
                    //select a new team 4 the season! whoohoo!
                    if (window.userObject.team == null) {
                        //select a team in general!! whoohoo!
                        select_team = "<p>Welcome! <br/> To get started, you will need to select a team.</p><form action=\"auth/join\" method=\"GET\" id=\"team-submit-form\"> <select name=\"team\" id=\"team\">";
                        teamsObject.forEach(function(team) {
                            select_team += "<option name=\"team\" value=\"" + team.id + "\">" + team.name + "</option>";
                        });
                        select_team += "</select><div id=\"team-submit-form-error\"></div></form>";
                        errorNotif('Select a Team', select_team, {
                            text: "Join",
                            action: function() {
                                doAjaxGetRequest(encodeURI('/auth/join?team='.concat(document.getElementById("team").value)), 'TeamSelector', function(status) {
                                    if (status.status == 200) {
                                        location.reload();
                                    }
                                }, function(status) {
                                    if (status.status == 409) {
                                        //user has team, 
                                    } else if (status.status == 403) {
                                        //team has no territories!
                                        document.getElementById('team-submit-form-error').innerHTML = "<br/><br/> <b style=\"color:red;\">Sorry, but this team is out of the running. Try another.</b>";
                                    } else {
                                        document.getElementsById('team-submit-form-error').innerHTML = "<br/><br/><b style=\"red\">Hmm, something went wrong. Try again?</b>";
                                    }
                                });
                            }
                        }, {
                            display: "none",
                            action: function() {}
                        });
                    } else {
                        //oh no! your team has been e l i m i n a t e d 
                        select_team = "<p>Oh no! Your team has been <b>eliminated.</b> Select a new one to play as: </p><form action=\"auth/join\" method=\"GET\" id=\"team-submit-form\"> <select name=\"team\" id=\"team\">";
                        teamsObject.forEach(function(team) {
                            select_team += "<option name=\"team\" value=\"" + team.id + "\">" + team.name + "</option>";
                        });
                        select_team += "</select><div id=\"team-submit-form-error\"></div></form>";
                        errorNotif('Select a Team', select_team, {
                            text: "Join",
                            action: function() {
                                doAjaxGetRequest(encodeURI('/auth/join?team='.concat(document.getElementById("team").value)), 'TeamSelector', function(status) {
                                    if (status.status == 200) {
                                        location.reload();
                                    }
                                }, function(status) {
                                    if (status.status == 409) {
                                        //user has team, 
                                    } else if (status.status == 403) {
                                        //team has no territories!
                                        document.getElementById('team-submit-form-error').innerHTML = "<br/><br/> <b style=\"color:red;\">Sorry, but this team is out of the running. Try another.</b>";
                                    } else {

                                    }
                                });
                            }
                        }, {
                            display: "none",
                        });
                    }
                    reject("No team");
                } else {
                    doAjaxGetRequest(encodeURI('/api/stats/team?team='.concat(window.userObject.team.name)).replace(/&/, '%26'), 'TeamLoader', function(teamObject) {
                        teamObject = JSON.parse(teamObject.response);
                        userObject = window.userObject;
                        var template = document.getElementById("templatePlayerCard");

                        var templateHtml = template.innerHTML;

                        var listHtml = "";
                        var index = 0;
                        for (i in window.teamsObject) {
                            if (window.teamsObject[i].name == teamObject.team) {
                                index = i;
                            }
                        }
                        listHtml += templateHtml
                            .replace(/{{user_name}}/g, userObject.name)
                            .replace(/{{user_team_color}}/, userObject.team.colors.primary)
                            .replace(/{{overall}}/g, "✯".repeat(userObject.ratings.overall))
                            .replace(/{{total_turns_stars}}/g, "✯".repeat(userObject.ratings.totalTurns))
                            .replace(/{{round_turns_stars}}/g, "✯".repeat(userObject.ratings.gameTurns))
                            .replace(/{{mvps_stars}}/g, "✯".repeat(userObject.ratings.mvps))
                            .replace(/{{streak_stars}}/g, "✯".repeat(userObject.ratings.streak))
                            .replace(/{{cfb_stars_stars}}/g, "✯".repeat(userObject.ratings.awards))
                            .replace(/{{total_turns}}/g, userObject.stats.totalTurns)
                            .replace(/{{round_turns}}/g, userObject.stats.gameTurns)
                            .replace(/{{mvps}}/g, userObject.stats.mvps)
                            .replace(/{{streak}}/g, userObject.stats.streak)
                            .replace(/{{cfb_stars}}/g, userObject.stats.awards)
                            .replace(/{{team}}/g, teamObject.team || "")
                            .replace(/{{team_players_yesterday}}/g, teamObject.players || "0")
                            .replace(/{{team_mercs_yesterday}}/g, teamObject.mercs || "0")
                            .replace(/{{team_star_power_yesterday}}/g, teamObject.stars || "0")
                            .replace(/{{team_territories_yesterday}}/g, teamObject.territories || "0")
                            .replace(/{{team_logo}}/g, window.teamsObject[index].logo || "0");
                        document.getElementById("playerCard").innerHTML = listHtml;
                        resolve("Okay");
                    }, function() {
                        reject("Error");
                    });
                }
            },
            function() {
                //display reddit login info
                document.getElementById("playerCard").classList.add("redditlogin");
                document.getElementById("playerCard").innerHTML = "<a href=\"/login/reddit\"><div style=\"margin-top:50%;\" ><img src=\"images/reddit-logo.png\"><br/><br/>LOGIN</div></a>";
                resolve("Okay");
            });
    } catch {
        reject("Error setting up user card");
    }
}

function setupMapHover(resolve, reject) {
    document.addEventListener('mouseover', function(event) {
        if (!event.target.matches('path')) return;
        event.preventDefault();
        document.getElementById("map-county-info").innerHTML = event.target.attributes["name"].value;
        document.getElementById("map-owner-info").innerHTML = event.target.attributes["owner"].value;
        event.target.style.fill = event.target.style.fill.replace('-primary', '-secondary');
    }, false);
    document.addEventListener('mouseout', function(event) {
        if (!event.target.matches('path')) return;
        event.preventDefault();
        document.getElementById("map-county-info").innerHTML = event.target.attributes["name"].value;
        document.getElementById("map-owner-info").innerHTML = event.target.attributes["owner"].value;
        event.target.style.fill = event.target.style.fill.replace('-secondary', '-primary');
    }, false);
    resolve(true);
}

function getTeamInfo(resolve, reject) {
    try {
        doAjaxGetRequest('/api/teams', 'Teams', function(team_data) {
            window.teamsObject = JSON.parse(team_data.response);
            //console.log(window.teamsObject);
            for (team in window.teamsObject) {
                document.documentElement.style
                    .setProperty('--'.concat(teamsObject[team].name.replace(/\W/g, '')).concat('-primary'), teamsObject[team].colors.primary);
                document.documentElement.style
                    .setProperty('--'.concat(teamsObject[team].name.replace(/\W/g, '')).concat('-secondary'), teamsObject[team].colors.secondary);
            }
            resolve(window.teamsObject);
        }, function() { reject("Error"); });
    } catch {
        reject("Error loading team info");
    }
}

function makeMove(id) {
    let endCycleColor = getComputedStyle(document.documentElement).getPropertyValue('--theme-bg').concat("");
    document.documentElement.style.setProperty("--theme-bg", "rgba(255,0,255,1)");
    var timeStamp = Math.floor(Date.now() / 1000); //use timestamp to override cache
    doAjaxGetRequest("/auth/move?target=".concat(id, '&timestamp=', timeStamp.toString()), 'Make Move', function() {
        document.documentElement.style.setProperty('--theme-bg', endCycleColor);
        errorNotif('Move Submitted', 'Your move has been submitted and received succesfully.', {
            text: "Okay"
        }, {
            display: "none"
        });
        return 0;
    }, function() {
        document.documentElement.style.setProperty('--theme-bg', 'rgba(255,0,0,1)')
        errorNotif('Could not make move', 'Hmm, couldn\'t set that as your move for the day.', {
            text: "Okay"
        }, {
            display: "none"
        });
    });
}

function drawActionBoard(resolve, reject) {
    let territories = window.territories;
    try {
        console.log("oh dear");
        let userteam = window.userObject.active_team.name;
        console.log(userteam);
        let attackable_territories = {};
        let defendable_territories = {};
        console.log(territories);
        for (i in territories) {
            if (territories[i].owner == userteam) {
                defendable_territories[territories[i].id] = territories[i];
                for (j in territories[i].neighbors) {
                    if (territories[i].neighbors[j].owner != userteam) {
                        attackable_territories[territories[i].neighbors[j].id] = territories[i].neighbors[j];
                    }
                }
            }
        }
        document.getElementById('action-container').style.display = "flex";
        let action_item = "<button onclick=\"makeMove({{id}});\">{{name}}</button>"
        for (k in attackable_territories) {
            document.getElementById('attack-list').innerHTML += action_item.replace(/{{name}}/, attackable_territories[k].name).replace(/{{id}}/, attackable_territories[k].id);
        }
        for (l in defendable_territories) {
            document.getElementById('defend-list').innerHTML += action_item.replace(/{{name}}/, defendable_territories[l].name).replace(/{{id}}/, defendable_territories[l].id);
        }
        console.log("Territory actions drawn");
        resolve("Okay");
    } catch (error) {
        console.log('could not do territory analysis');
        console.log(error);
        reject("Error");
    }
}

function drawMap(resolve, reject) {
    doAjaxGetRequest('images/map.svg', 'Map', function(data) {
        document.getElementById('map-container').innerHTML = data.response;
        //now to fetch territory ownership!!
        doAjaxGetRequest('/api/territories', 'Territories', function(territory_data) {
            window.territories = JSON.parse(territory_data.response);
            for (territory in window.territories) {
                document.getElementById('map').getElementById(window.territories[territory].name.replace(/ /, "")).style.fill = 'var(--'.concat(territories[territory].owner.replace(/\W/g, '').concat('-primary)'));
                document.getElementById('map').getElementById(window.territories[territory].name.replace(/ /, "")).setAttribute('owner', territories[territory].owner);
            }
            resolve(window.territories);
        }, function() {
            reject("Error");
        });
    });
}

function drawLeaderboard(season, day) {
    doAjaxGetRequest('/api/stats/leaderboard', 'leaderboard request', function(leaderboard_data) {
        //console.log(data);
        let leaderboardObject = JSON.parse(leaderboard_data.response);
        let display_headings = ["rank", "name", "territoryCount", "playerCount", "mercCount", "starPower", "efficiency"];

        var obj = {
            // Quickly get the headings
            headings: ["Rank", "Name", "Territories", "Players", "Mercernaries", "Stars", "Efficiency"],

            // data array
            data: []
        };

        // Loop over the objects to get the values
        for (var i = 0; i < leaderboardObject.length; i++) {

            obj.data[i] = [];

            for (var p in leaderboardObject[i]) {
                if (leaderboardObject[i].hasOwnProperty(p) && display_headings.indexOf(p) != -1) {
                    if (p == 'name') {
                        obj.data[i].push("<img width='30px' src='" + leaderboardObject[i]['logo'] + "'/>".concat(leaderboardObject[i][p]));
                    } else {
                        obj.data[i].push(leaderboardObject[i][p]);
                    }
                }
            }
        }

        var datatable = new DataTable("#leaderboard-table", {
            data: obj,
            columns: obj.columns,
            searchable: false,
            perPageSelect: false,
            footer: false,
            labels: {
                info: "",
            }
        });
    });
}


let doge = Promise.all([drawLeaderboard(0, 0), new Promise(drawMap), new Promise(getTeamInfo)])
    .then((values) => {
        console.log(values);
    })
    .then(() => {
        return new Promise((resolve, reject) => {
            setupMapHover(resolve, reject);
        })
    })
    .then(() => {
        return new Promise((resolve, reject) => {
            getUserInfo(resolve, reject);
        })
    })
    .then(() => {
        return new Promise((resolve, reject) => {
            drawActionBoard(resolve, reject);
        })
    })
    .catch((values) => { console.log(values) });