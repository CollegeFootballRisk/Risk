// @license magnet:?xt=urn:btih:3877d6d54b3accd4bc32f8a48bf32ebc0901502a&dn=mpl-2.0.txt Mozilla-Public-2.0
// if .ml, redirect to .com
if (window.location.hostname === "aggierisk.ml") {
    window.location = 'https://aggierisk.com/';
}

//initialize globals
var appInfo = {
    outstandingRequests: [],
    errorNotifications: [],
    rollTime: new Date("December 12, 2020 04:00:00"),
    loadTime: new Date(),
    burger: false,
    burgerTrigger: false,
    teamsObject: null,
    userObject: null,
    lockDisplay: false,
    dDay: new Date("December 24, 2020 04:00:00"),
    fullOpacity: 0,
    map: '/images/map2.svg?v=12',
    viewbox: '0 0 800 700'
}

appInfo.dDay.setUTCHours(4);

appInfo.rollTime.setUTCHours(4, 0, 0, 0);

if (appInfo.rollTime < new Date()) {
    appInfo.rollTime = new Date();
    appInfo.rollTime.setUTCHours(4, 0, 0, 0);
    if (appInfo.rollTime < new Date()) {
        appInfo.rollTime.setUTCDate(appInfo.rollTime.getUTCDate() + 1)
    }
}

// JS is enabled, so hide that notif
_('error-notif').style.display = "none";


function returnHover() {
    appInfo.lockDisplay = false;
    try {
        _('hover-button').disabled = true;
    } catch {
        _('oddmap_hover-button').disabled = true;
        _('heatmap_hover-button').disabled = true;
    }
    let temptags = document.getElementsByTagName("path");
    for (tt = 0; tt < temptags.length; tt++) {
        temptags[tt].style.fill = temptags[tt].style.fill.replace('-secondary', '-primary');
    }
}
// link handling
document.addEventListener('click', function(event) {
    switch (event.target.tagName) {
        case 'path':
            if (appInfo.lockDisplay || event.target.attributes['mapname'] == 'odds') {
                mapDisplayUpdate(event, false, true);
            } else {
                appInfo.lockDisplay = true;
                document.onkeydown = function(evt) {
                    evt = evt || window.event;
                    if (evt.keyCode == 27) {
                        returnHover();
                    }
                };
                mapDisplayUpdate(event, false, true);
            }
            //window.history.pushState("Rust Risk", "Rust Risk", '/territory/'.concat(event.target.attributes['name'].value));
            break;
        case 'A':
            if (link_is_external(event.target)) return;
            event.preventDefault();
            window.history.pushState("Rust Risk", "Rust Risk", event.target.href);
            break;
        default:
            return;
    }
}, false);

_('burger').addEventListener('click', function(event) {
    appInfo.burger = !appInfo.burger;
    appInfo.burgerTrigger = true;
    _('nav').style.display = (appInfo.burger) ? 'flex' : 'none';
});

function goToTerritory(territory) {
    window.history.pushState("Rust Risk", "Rust Risk", '/territory/'.concat(territory));
}

//request handling
function doAjaxGetRequest(url, source, callback, errorcallback = defaultErrorNotif) {
    var instance_index = addUrlFromRequests(source, url);
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function() {
        if (this.readyState == 4 && this.status == 200) {
            if (typeof callback == 'function') {
                try {
                    callback(this);
                } catch (err) {
                    console.log("Error with callback function");
                    console.log(err);
                }
            } else {
                return JSON.parse(this.response);
            }
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
    var index = appInfo.outstandingRequests.push({ source: source, url: url, state: 0 }) - 1;
    updateLoaderVisibility();
    return index;
}

function updateUrlFromRequests(index, status) {
    if (index > -1) {
        appInfo.outstandingRequests[index].state = status;
    }
    updateLoaderVisibility();
}

function updateLoaderVisibility(forceHide = false) {
    let pending = false;
    for (i in appInfo.outstandingRequests) {
        if (appInfo.outstandingRequests[i].state == 0) {
            pending = true;
            break;
        }
    }
    if (!pending && !forceHide) {
        //stop loader
        _("loadicon").classList.remove("spin");
    } else {
        //start loader
        //check if globalError
        _("loadicon").classList.add("spin");
    }
}

/*** Error Notifications ***/

function errorNotif(title, body, button1, button2, resolveself = true, skipnotifcheck = false, errorIndex = 0) {
    if (skipnotifcheck != true) {
        errorIndex = appInfo.errorNotifications.push({ title: title, body: body, button1: button1, button2: button2, status: 1, resolveself: resolveself }) - 1;
    }
    let vset = appInfo.errorNotifications[errorIndex - 1] || { status: 0 };
    if (vset.status == 0) {
        _('error-notif').style.display = "block";
        _('error-notif-title').innerHTML = title || "General Error";
        _('error-notif-body').innerHTML = body || "Hmm, no error was specified. Try notifying <a href=\"https://github.com/mautamu/risk\">u/Mautamu</a> if this issue persists.";
        _('error-notif-button-1').innerHTML = button1.text || "";
        _('error-notif-button-1-container').style.display = button1.display || "block";
        _('error-notif-button-1').onclick = function() {
            try {
                if (typeof button1.action == "function") {
                    button1.action();
                }
                if (resolveself) {
                    appInfo.errorNotifications[errorIndex].status = 0;
                }
            } finally {
                errorOver(errorIndex);
            }
        };
        _('error-notif-button-2').innerHTML = button2.text || "";
        _('error-notif-button-2-container').style.display = button2.display || "block";
        _('error-notif-button-2').onclick = function() {
            try {
                if (typeof button2.action == "function") {
                    button2.action();
                }
                if (resolveself) {
                    appInfo.errorNotifications[errorIndex].status = 0;
                }
            } finally {
                errorOver(errorIndex);
            }
        };
    }
}

function errorOver(errorIndex) {
    if (appInfo.errorNotifications[errorIndex].status == 0) {
        //move to next one or hide
        let pending = false;
        for (i in appInfo.errorNotifications) {
            if (appInfo.errorNotifications[i].status != 0) {
                pending = true;
                errorNotif(appInfo.errorNotifications[i].title, appInfo.errorNotifications[i].body, appInfo.errorNotifications[i].button1, appInfo.errorNotifications[i].button2, appInfo.errorNotifications[i].resolveself, true, i);
                break;
            }
        }
        if (!pending) {
            _('error-notif').style.display = "none";
        }
    } else {
        //do nothing
    }
}

function defaultErrorNotif(data) {
    errorNotif(
        'Fetch Error',
        '<h1>Howdy partner</h1>, unfortunately we encountered an error. Not sure what it\'s about. <br/><br/> If this keeps occuring, please <a href="mailto:risk@aggierisk.ml">email us.</a>', {
            text: "Okay",
            action: function() {}
        }, {
            display: "none",
            action: function() {}
        }
    )
}

function drawPlayerCard(userObject, teamObject) {
    var template = _("templatePlayerCard");

    var templateHtml = template.innerHTML;

    var listHtml = "";
    var index = 0;
    for (i in appInfo.teamsObject) {
        if (appInfo.teamsObject[i].name == teamObject.team) {
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
        .replace(/{{team_2}}/g, (userObject.active_team.name != userObject.team.name) ? "Playing as " + userObject.active_team.name || "" : ".")
        .replace(/{{team_players_yesterday}}/g, teamObject.players || "0")
        .replace(/{{team_mercs_yesterday}}/g, teamObject.mercs || "0")
        .replace(/{{team_star_power_yesterday}}/g, teamObject.stars || "0")
        .replace(/{{team_territories_yesterday}}/g, teamObject.territories || "0")
        .replace(/{{team_logo}}/g, appInfo.teamsObject[index].logo || "0");
    _("playerCard").innerHTML = listHtml;
}

/*** Get Data Fxs ***/
function getUserInfo(resolve, reject) {
    try {
        doAjaxGetRequest('/api/me', 'UserLoader', function(userObject) {
                console.log("Making req");
                appInfo.userObject = JSON.parse(userObject.response);
                //see if user has a team, if not, prompt them and halt
                let active_team = appInfo.userObject.active_team || {
                    name: null
                };
                if (active_team.name == null) {
                    //select a new team 4 the season! whoohoo!
                    if (appInfo.userObject.team == null) {
                        //select a team in general!! whoohoo!
                        select_team = "<p>Welcome! <br/> To get started, you will need to select a team.</p><form action=\"auth/join\" method=\"GET\" id=\"team-submit-form\"> <select name=\"team\" id=\"team\">";
                        season = window.turnsObject[window.turnsObject.length - 1].season;
                        approved_teams = [];
                        for (n = 0; n < window.territories.length; n++) {
                            if (!approved_teams.includes(window.territories[n].owner)) {
                                approved_teams.push(window.territories[n].owner);
                            }
                        }

                        appInfo.teamsObject.forEach(function(team) {
                            if (team.seasons.includes(season) && team.name != "Unjoinable Placeholder" && approved_teams.includes(team.name)) {
                                select_team += "<option name=\"team\" value=\"" + team.id + "\">" + team.name + "</option>";
                            }
                        });
                        select_team += "</select><div id=\"team-submit-form-error\"></div></form>";
                        errorNotif('Select a Team', select_team, {
                            text: "Join",
                            action: function() {
                                doAjaxGetRequest(encodeURI('/auth/join?team='.concat(_("team").value)), 'TeamSelector', function(status) {
                                    if (status.status == 200) {
                                        location.reload();
                                    }
                                }, function(status) {
                                    if (status.status == 409) {
                                        //user has team, 
                                    } else if (status.status == 403) {
                                        //team has no territories!
                                        _('team-submit-form-error').innerHTML = "<br/><br/> <b style=\"color:red;\">Sorry, but this team is out of the running. Try another.</b>";
                                    } else {
                                        _('team-submit-form-error').innerHTML = "<br/><br/><b style=\"red\">Hmm, something went wrong. Try again?</b>";
                                    }
                                });
                            }
                        }, {
                            display: "none",
                            action: function() {}
                        });
                    } else {
                        //oh no! your team has been e l i m i n a t e d 
                        console.log("Elimed");
                        select_team = "<p>Oh no! Your team has been <b>eliminated.</b> Select a new one to play as: </p><form action=\"auth/join\" method=\"GET\" id=\"team-submit-form\"> <select name=\"team\" id=\"team\">";
                        approved_teams = [];
                        season = window.turnsObject[window.turnsObject.length - 1].season;
                        for (n = 0; n < window.territories.length; n++) {
                            if (!approved_teams.includes(window.territories[n].owner)) {
                                approved_teams.push(window.territories[n].owner);
                            }
                        }
                        appInfo.teamsObject.forEach(function(team) {
                            if (team.seasons.includes(season) && team.name != "Unjoinable Placeholder" && approved_teams.includes(team.name)) {
                                select_team += "<option name=\"team\" value=\"" + team.id + "\">" + team.name + "</option>";
                            }
                        });
                        select_team += "</select><div id=\"team-submit-form-error\"></div></form>";
                        errorNotif('Select a Team', select_team, {
                            text: "Join",
                            action: function() {
                                doAjaxGetRequest(encodeURI('/auth/join?team='.concat(_("team").value)), 'TeamSelector', function(status) {
                                    if (status.status == 200) {
                                        location.reload();
                                    }
                                }, function(status) {
                                    if (status.status == 409) {
                                        //user has team, 
                                    } else if (status.status == 403) {
                                        //team has no territories!
                                        _('team-submit-form-error').innerHTML = "<br/><br/> <b style=\"color:red;\">Sorry, but this team is out of the running. Try another.</b>";
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
                    doAjaxGetRequest(encodeURI('/api/stats/team?team='.concat(appInfo.userObject.team.name)).replace(/&/, '%26'), 'TeamLoader', function(teamObject) {
                        drawPlayerCard(appInfo.userObject, JSON.parse(teamObject.response));
                        resolve("Okay");
                    }, function() {
                        reject("Error");
                    });
                }
            },
            function() {
                //display reddit login info
                _("playerCard").classList.add("redditlogin");
                _("reddit-login-top").style.display = "flex";
                _("playerCard").innerHTML = "<a href=\"/login/reddit\"><div style=\"margin-top:50%;\" ><img src=\"images/reddit-logo.png\"><br/><br/>LOGIN</div></a>";
                _("roll-container").innerHTML = _("playerCard").outerHTML;
                resolve("Okay");
            });
    } catch {
        reject("Error setting up user card");
    }
}

function mapDisplayUpdate(event, change, override = false) {
    if (!appInfo.lockDisplay || override) {
        twid = "hover-button";
        switch (event.target.attributes["mapname"].value) {
            case "odds":
                // code block
                _("oddmap_map-county-info").innerHTML = event.target.attributes["name"].value;
                _("oddmap_map-owner-info").innerHTML = event.target.hasAttribute("odds") ? "Odds:  " + parseFloat(event.target.attributes["odds"].value).toFixed(2) : "Odds: 0.00";
                twid = "oddmap_" + twid;
                break;
            case "heat":
                // code block
                _("heatmap_map-county-info").innerHTML = event.target.attributes["name"].value;
                _("heatmap_map-owner-info").innerHTML = event.target.hasAttribute("players") ? "Players:  " + event.target.attributes["players"].value : "Players: 0";
                twid = "heatmap_" + twid;
                break;
            case "map":
                _("map-county-info").innerHTML = event.target.attributes["name"].value;
                _("map-owner-info").innerHTML = "Owner:  " + event.target.attributes["owner"].value + "<br />Region: " + event.target.attributes["region"].value;
                try {
                    if (appInfo.attackable_territory_names.includes(event.target.attributes["name"].value)) {
                        _("attack-button").disabled = false;
                        _("attack-button").onclick = function() { makeMove(event.target.attributes["territoryid"].value) };
                    } else {
                        _("attack-button").disabled = true;
                    }
                    if (appInfo.defendable_territory_names.includes(event.target.attributes["name"].value)) {
                        _("defend-button").disabled = false;
                        _("defend-button").onclick = function() { makeMove(event.target.attributes["territoryid"].value) };
                    } else {
                        _("defend-button").disabled = true;
                    }
                } catch {
                    //user not logged in. Oh well..
                }

                _("visit-button").disabled = false;
                _("visit-button").onclick = function() { goToTerritory(event.target.attributes["name"].value) };
                break;
            case "leaderboard":
                _("map-county-info").innerHTML = event.target.attributes["name"].value;
                _("map-owner-info").innerHTML = "Owner:  " + event.target.attributes["owner"].value + "<br /> Power:" + event.target.attributes["power"].value + " Players: " + event.target.attributes["players"].value;
                _("visit-button").disabled = false;
                _("visit-button").onclick = function() { goToTerritory(event.target.attributes["name"].value) };
                break;
            default:
                _("map-county-info").innerHTML = event.target.attributes["name"].value;
                _("map-owner-info").innerHTML = "Owner:  " + event.target.attributes["owner"].value;
                break;
        }
        if (override) {
            let temptags = document.getElementsByTagName("path");
            for (tt = 0; tt < temptags.length; tt++) {
                temptags[tt].style.fill = temptags[tt].style.fill.replace('-secondary', '-primary');
            }
            _(twid).disabled = false;
        }
        if (change) {
            event.target.style.fill = event.target.style.fill.replace('-secondary', '-primary');

        } else {
            event.target.style.fill = event.target.style.fill.replace('-primary', '-secondary');
        }
    }
}

function mapHover(event) {
    if (!event.target.matches('path')) return;
    type = event.type;
    switch (type) {
        case 'mouseover':
            event.preventDefault();
            mapDisplayUpdate(event, false);
            break;
        case 'mouseout':
            event.preventDefault();
            mapDisplayUpdate(event, true);
            break;
        default:
            break;
    }
}

function setupMapHover(resolve, reject) {
    document.addEventListener('mouseover', mapHover, false);
    document.addEventListener('mouseout', mapHover, false);
    resolve(true);
}

function removeMapHover(resolve, reject) {
    document.removeEventListener('mouseover', mapHover, false);
    document.removeEventListener('mouseout', mapHover, false);
    resolve(true);
}

function getTeamInfo(resolve, reject) {
    try {
        doAjaxGetRequest('/api/teams', 'Teams', function(team_data) {
            appInfo.teamsObject = JSON.parse(team_data.response);
            //console.log(appInfo.teamsObject);
            for (team in appInfo.teamsObject) {
                document.documentElement.style
                    .setProperty('--'.concat(appInfo.teamsObject[team].name.replace(/\W/g, '')).concat('-primary'), appInfo.teamsObject[team].colors.primary);
                document.documentElement.style
                    .setProperty('--'.concat(appInfo.teamsObject[team].name.replace(/\W/g, '')).concat('-secondary'), appInfo.teamsObject[team].colors.secondary);
            }
            resolve(appInfo.teamsObject);
        }, function() { reject("Error"); });
    } catch {
        reject("Error loading team info");
    }
}

function getTurns(resolve, reject) {
    try {
        doAjaxGetRequest('/api/turns', 'Turns', function(team_data) {
            window.turnsObject = JSON.parse(team_data.response);
            appInfo.rollTime = new Date(window.turnsObject[window.turnsObject.length - 1].rollTime + "Z");
            window.turn = window.turnsObject[window.turnsObject.length - 1];
            resolve(window.turnsObject);
        }, function() { reject("Error"); });
    } catch {
        reject("Error loading team info");
    }
}

function makeMove(id) {
    appInfo.doubleOrNothing = false;
    if (appInfo.defendable_territory_names.length == 1) {
        //Prompt the player if they want to double or nothing their move
        doubleOrNothingText = window.prompt("Type YES to quintuple-or-nothing your move's power. Otherwise type NO.");
        appInfo.doubleOrNothing = (doubleOrNothingText.toLowerCase() == 'yes');
    }
    let endCycleColor = getComputedStyle(document.documentElement).getPropertyValue('--theme-bg').concat("");
    let endCycleColor05 = getComputedStyle(document.documentElement).getPropertyValue('--theme-bg-05').concat("");
    document.documentElement.style.setProperty("--theme-bg", "rgba(255,0,255,1)");
    document.documentElement.style.setProperty("--theme-bg-05", "rgba(255,0,255,0.5)");
    var timeStamp = Math.floor(Date.now() / 1000); //use timestamp to override cache
    doAjaxGetRequest("/auth/move?target=".concat(id, '&timestamp=', timeStamp.toString(), "&aon=", appInfo.doubleOrNothing), 'Make Move', function() {
            document.documentElement.style.setProperty('--theme-bg', endCycleColor);
            document.documentElement.style.setProperty('--theme-bg-05', endCycleColor05);
            doAjaxGetRequest("/auth/my_move", 'Load Move', function(data) {
                highlightTerritory(data.response.replace(/"/g, ''));
                errorNotif('Move Submitted', 'Your move was on territory <b>{{Territory}}</b>.'.replace(/{{Territory}}/, data.response.replace(/"/g, '')), {
                    text: "Okay"
                }, {
                    display: "none"
                });
            });
            return 0;
        },
        function() {
            document.documentElement.style.setProperty('--theme-bg', 'rgba(255,0,0,1)');
            document.documentElement.style.setProperty('--theme-bg-05', 'rgba(255,0,0,0.5)');
            errorNotif('Could not make move', 'Hmm, couldn\'t set that as your move for the day.', {
                text: "Okay"
            }, {
                display: "none"
            });
        });
}

function drawActionBoard(resolve, reject) {
    let territories = window.territories;
    if (window.turnsObject[window.turnsObject.length - 1].finale) {
        _('last-day-notice').innerHTML = 'Today is the final roll! Make it count!';
    }
    if (!window.turnsObject[window.turnsObject.length - 1].active) {
        _('last-day-notice').innerHTML = 'This season is over. Thank you for playing!';
        _('action-container').innerHTML = '<iframe src="https://docs.google.com/forms/d/e/1FAIpQLSej4xCIqU7o0WnZV59J7at48BVKCJW3-bcV75wn1H-guDHFtQ/viewform?embedded=true" width="640" height="2903" frameborder="0" marginheight="0" marginwidth="0">Loading…</iframe>';
    } else {
        try {
            console.log("Drawing Actions.");
            let userteam = appInfo.userObject.active_team.name;
            console.log(userteam);
            appInfo.attackable_territories = {};
            appInfo.attackable_territory_names = [];
            appInfo.defendable_territories = {};
            appInfo.defendable_territory_names = [];
            console.log(territories);
            for (i in territories) {
                if (territories[i].owner == userteam) {
                    neighbors = 0;
                    for (j in territories[i].neighbors) {
                        if (territories[i].neighbors[j].owner != userteam) {
                            appInfo.attackable_territories[territories[i].neighbors[j].id] = territories[i].neighbors[j];
                            appInfo.attackable_territory_names.push(territories[i].neighbors[j].name);
                            neighbors += 1;
                        }
                    }
                    if (neighbors != 0) {
                        appInfo.defendable_territories[territories[i].id] = territories[i];
                        appInfo.defendable_territory_names.push(territories[i].name);
                    }
                }
            }
            _('action-container').style.display = "flex";
            let action_item = "<button onclick=\"makeMove({{id}});\">{{name}}</button>"
            for (k in appInfo.attackable_territories) {
                _('attack-list').innerHTML += action_item.replace(/{{name}}/, appInfo.attackable_territories[k].name).replace(/{{id}}/, appInfo.attackable_territories[k].id);
            }
            for (l in appInfo.defendable_territories) {
                _('defend-list').innerHTML += action_item.replace(/{{name}}/, appInfo.defendable_territories[l].name).replace(/{{id}}/, appInfo.defendable_territories[l].id);
            }
            console.log("Territory actions drawn");
            resolve("Okay");
        } catch (error) {
            console.log('could not do territory analysis');
            //console.log(error);
            reject("Error");
        }
    }
}

function resizeMap() {
    let width = _('map-container').clientWidth;
    if (width < 1000) {
        _('map').setAttribute('width', width);
        _('map').setAttribute('height', width);
    }
    _('map').setAttribute('preserveAspectRatio', 'xMinYMin');
    _('map').setAttribute('viewBox', appInfo.viewbox);
}

function seasonDayObject(season = 0, day = 0, autoup = false, fn, turnsObject) {
    //TODO: implement season stuff plz
    opt = "<option value=\"{{val}}\" {{sel}}>Season {{season}}, Day {{day}}</option>";
    days = "<select onchange=\"" + fn + "(this.value); \" name=\"day_select\" id=\"day_select\">";
    for (turnb in turnsObject) {
        if (turnb == 0) {
            continue;
        }
        turn = turnsObject.length - turnb - 1;
        sel = ((turnsObject[turn].season == season && turnsObject[turn].day == day) || (day == 0 && turn == turnsObject.length - 1)) ? "selected" : "";
        days += opt.replace(/{{val}}/gi, turnsObject[turn].season + "." + turnsObject[turn].day).replace(/{{sel}}/, sel).replace(/{{season}}/, turnsObject[turn].season).replace(/{{day}}/, turnsObject[turn].day);
    }
    days += "</select>";
    if (autoup == false) {
        return "{{day}}".replace(/{{day}}/, days);
    } else {
        //yay! time to redraw stuffs: 
        _('day_select').outerHTML = days;
    }
}

function drawMap(resolve, reject, source = 'territories', season = 0, day = 0) {
    // source should be either 'heat' or 'territories'
    var addendum = (season > 0 && day > 0) ? "?season=" + season + "&day=" + day : "";
    doAjaxGetRequest(appInfo.map, 'Map', function(data) {
        _('map-container').innerHTML = data.response;
        //now to fetch territory ownership or heat data
        switch (source) {
            case 'heat':
                doAjaxGetRequest('/api/heat' + addendum, 'Heat', function(heat_data) {
                    heat = JSON.parse(heat_data.response);
                    // find maximum
                    maxmin = getMaxMin(heat, "power");
                    for (territory in heat) {
                        //red = Math.round(160 + 200 * (heat[territory].power - maxmin[1].power) / (maxmin[0].power - maxmin[1].power)) | 60;
                        _('map').getElementById(heat[territory].territory.replace(/ /, "")).style.fill = getColorForPercentage((heat[territory].power - maxmin[1].power) / (maxmin[0].power - maxmin[1].power));
                        _('map').getElementById(heat[territory].territory.replace(/ /, "")).setAttribute('owner', heat[territory].winner);
                        _("old-map-county-info").innerHTML = "Leaderboard";
                        _("old-map-owner-info").innerHTML = seasonDayObject(season || 1, day || 0, false, "page_leaderboard_update", window.turnsObject);
                        _("old-map-owner-info").setAttribute('selectitem', 'true')
                    }
                    resizeMap();
                    resolve(heat);
                }, function() {
                    reject("Error");
                });
                break;
            case 'leaderboard':
                doAjaxGetRequest('/api/heat' + addendum, 'Heat', function(heat_data) {
                    heat = JSON.parse(heat_data.response);
                    // find maximum
                    maxmin = getMaxMin(heat, "power");
                    console.log("Maxmin", maxmin);
                    for (territory in heat) {
                        red = (heat[territory].power - maxmin[1].power) / (maxmin[0].power - maxmin[1].power) || 0;
                        _('map').getElementById(heat[territory].territory.replace(/ /, "")).style.fill = getColorForPercentage(red);
                        _('map').getElementById(heat[territory].territory.replace(/ /, "")).setAttribute("owner", heat[territory].winner);
                        _('map').getElementById(heat[territory].territory.replace(/ /, "")).setAttribute("power", heat[territory].power);
                        _('map').getElementById(heat[territory].territory.replace(/ /, "")).setAttribute("players", heat[territory].players);
                        _('map').getElementById(heat[territory].territory.replace(/ /, "")).setAttribute("mapname", "leaderboard");
                        _("old-map-county-info").innerHTML = "Leaderboard";
                        _("old-map-owner-info").innerHTML = seasonDayObject(season || 1, day || 0, false, "page_leaderboard_update", window.turnsObject);
                        _("old-map-owner-info").setAttribute('selectitem', 'true')
                    }
                    var li = "<br/><br/><ul id=\"spot\">";
                    for (var i = 0, l = 10; i <= l; i++) {
                        li += "<li style=\"background:" + getColorForPercentage(i / l) + "\">" + (((i / l) * (maxmin[0].power - maxmin[1].power)) + maxmin[1].power).toFixed(0) + "</li>";
                    }
                    li += "</ul>";
                    _("map-container").innerHTML += li;
                    resizeMap();
                    resolve(heat);
                }, function() {
                    reject("Error");
                });
                break;
            case 'territories':
                doAjaxGetRequest('/api/territories' + addendum, 'Territories', function(territory_data) {
                    window.territories = JSON.parse(territory_data.response);
                    for (territory in window.territories) {
                        console.log(window.territories[territory].name);
                        _('map').getElementById(window.territories[territory].name.replace(/ /, "")).style.fill = 'var(--'.concat(territories[territory].owner.replace(/\W/g, '').concat('-primary)'));
                        _('map').getElementById(window.territories[territory].name.replace(/ /, "")).setAttribute('owner', territories[territory].owner);
                        _('map').getElementById(window.territories[territory].name.replace(/ /, "")).setAttribute('mapname', "map");
                        _('map').getElementById(window.territories[territory].name.replace(/ /, "")).setAttribute('territoryid', territories[territory].id);
                    }
                    resizeMap();
                    resolve(window.territories);
                }, function() {
                    reject("Error");
                });
                break;
            default:
                break;
        }
    });
}


function drawUserTurnHistory(playerObject) {
    let turnHistoryObject = playerObject.turns;
    let display_headings = ["season", "day", "stars", "team", "territory", "mvp"];

    var obj = {
        // Quickly get the headings
        headings: ["Season", "Day", "Stars", "MVP", "Territory", "Team"],

        // data array
        data: []
    };

    // Loop over the objects to get the values
    for (var i = 0; i < turnHistoryObject.length; i++) {

        obj.data[i] = [];

        for (var p in turnHistoryObject[i]) {
            if (turnHistoryObject[i].hasOwnProperty(p) && display_headings.indexOf(p) != -1) {
                if (p == 'territory') {
                    obj.data[i].push("<a href=\"/territory/{{terr}}\">{{terr}}</a>".replace(/{{terr}}/gi, turnHistoryObject[i][p]));
                } else if (p == 'team') {
                    obj.data[i].push("<a href=\"/team/{{team}}\">{{team}}</a>".replace(/{{team}}/gi, turnHistoryObject[i][p]));
                } else {
                    obj.data[i].push(turnHistoryObject[i][p]);
                }
            }
        }
    }
    try {
        window.datatable.destroy();
    } catch {
        // don't do anything, nor output to table ;)
    } finally {
        window.datatable = new DataTable("#history-table", {
            data: obj,
            columns: obj.columns,
            searchable: false,
            perPageSelect: false,
            footer: false,
            labels: {
                info: "",
            }
        });
    }
}

function drawLeaderboard(season, day) {
    var addendum = (season > 0 && day > 0) ? "?season=" + season + "&day=" + day : "";
    doAjaxGetRequest('/api/stats/leaderboard' + addendum, 'leaderboard request', function(leaderboard_data) {
        let leaderboardObject = JSON.parse(leaderboard_data.response);
        let display_headings = ["rank", "name", "territoryCount", "playerCount", "mercCount", "starPower", "efficiency"];

        var obj = {
            // Quickly get the headings
            headings: ["Rank", "Name", "Territories", "Team<br/> Players", "Mercenaries", "Star<br/> Power", "Efficiency"],

            // data array
            data: []
        };

        // Loop over the objects to get the values
        for (var i = 0; i < leaderboardObject.length; i++) {

            obj.data[i] = [];

            for (var p in leaderboardObject[i]) {
                if (leaderboardObject[i].hasOwnProperty(p) && display_headings.indexOf(p) != -1) {
                    if (p == 'name') {
                        obj.data[i].push("<a href=\"/team/" + leaderboardObject[i][p] + "\"><img width='30px' src='" + leaderboardObject[i]['logo'] + "'/>".concat(leaderboardObject[i][p]));
                    } else if (p == 'efficiency') {
                        obj.data[i].push((leaderboardObject[i][p] || 0).toFixed(2));
                    } else {
                        obj.data[i].push(leaderboardObject[i][p] || 0);
                    }
                }
            }
        }

        try {
            window.datatable.destroy();
        } catch {
            // don't do anything, nor output to table ;)
        } finally {
            window.datatable = new DataTable("#leaderboard-table", {
                data: obj,
                columns: obj.columns,
                searchable: false,
                perPageSelect: false,
                footer: false,
                perPage: 20,
                labels: {
                    info: "",
                }
            });
            window.datatable.columns().sort(1);
        }
    });
}

function page_leaderboard_update(seasonday) {
    //decouple to ints
    seasonday = seasonday.split(".");
    season = Number(seasonday[0]) || 0;
    day = Number(seasonday[1]) || 0;
    drawLeaderboard(season, day, templateLeaderboard, contentTag, season, day);
    drawMap(console.log, console.log, 'leaderboard', season, day);
    let selectOpt = _('day_select').getElementsByTagName('option');
    for (ely = 0; ely < selectOpt.length; ely++) {
        selectOpt[ely].removeAttribute("selected");
    }
    _('day_select').value = season + "." + day;
}

function page_info(contentTag) {
    updateLoaderVisibility();
    var templateInfo = _("templateInfo");
    contentTag.innerHTML += templateInfo.innerHTML;
    console.log(contentTag);
}

function page_leaderboard(contentTag) {
    /* objects:
        1. map (heat)
        2. leaderboard

    First, we fetch the heat data for turn
        */
    var templateLeaderboard = _("templateLeaderboard");
    templateLeaderboard = templateLeaderboard.innerHTML;
    var templateMap = _("templateMap");
    templateMap = templateMap.innerHTML;
    contentTag.innerHTML += templateMap;
    contentTag.innerHTML += templateLeaderboard;
    drawLeaderboard(0, 0, templateLeaderboard, contentTag);
    let leaderboard = new Promise((resolve, reject) => {
        getTurns(resolve, reject);
        getTeamInfo(resolve, reject);
    }).then(() => {
        return new Promise((resolve, reject) => {
            drawMap(resolve, reject, "leaderboard");
        })
    }).then(() => {
        return new Promise((resolve, reject) => {
            setupMapHover(resolve, reject);
        })
    });

}

function page_territory(contentTag, t_object) {
    territory = t_object.name;
    season = t_object.season;
    day = t_object.day;
    contentTag.innerHTML = _('templateTerritoryComplete').innerHTML;
    if (season > 0 && day > 0) {
        //attempt to fetch the data for that day & season
        doAjaxGetRequest('/api/territory/turn?season=' + season + '&day=' + day + '&territory=' + territory,
            'TerritoryFetch',
            function(territoryData) {
                //Fill the table!
                territoryTurn = JSON.parse(territoryData.response);
                territoryCompleteHeader = _('templateTerritoryCompleteHeader').innerHTML;
                _('territoryCompleteHeader').innerHTML = territoryCompleteHeader
                    .replace(/{{TerritoryName}}/, decodeURIComponent(territory).replace(/(^\w{1})|(\s+\w{1})/g, letter => letter.toUpperCase()))
                    .replace(/{{owner}}/, territoryTurn.occupier)
                    .replace(/{{winner}}/, territoryTurn.winner)
                let display_headings = ["team", "players", "power", "chance"];
                var obj = {
                    // Quickly get the headings
                    headings: ["Team", "Players", "Power", "Chance"],

                    // data array
                    data: []
                };

                chart = {
                    team: [],
                    power: [],
                    background: [],
                    hover: []
                };

                // Loop over the objects to get the values
                for (var i = 0; i < territoryTurn.teams.length; i++) {

                    chart.team.push(territoryTurn.teams[i].team);
                    chart.power.push(territoryTurn.teams[i].power);
                    chart.background.push(territoryTurn.teams[i].color);
                    chart.hover.push(territoryTurn.teams[i].secondaryColor);

                    obj.data[i] = [];

                    for (var p in territoryTurn.teams[i]) {
                        if (territoryTurn.teams[i].hasOwnProperty(p) && display_headings.indexOf(p) != -1) {
                            if (p == 'chance') {
                                obj.data[i].push(territoryTurn.teams[i][p].toFixed(2));
                            } else if (p == 'team') {
                                obj.data[i].push("<a href=\"/team/{{team}}\" >{{team}}</a>".replace(/{{team}}/gi, territoryTurn.teams[i][p]));
                            } else {
                                obj.data[i].push(territoryTurn.teams[i][p]);
                            }
                        }
                    }
                }
                try {
                    window.datatable.destroy();
                } catch {
                    // don't do anything, nor output to table ;)
                } finally {
                    window.datatable = new DataTable("#owner-table", {
                        data: obj,
                        columns: obj.columns,
                        searchable: false,
                        perPageSelect: false,
                        footer: false,
                        labels: {
                            info: "",
                        }
                    });
                }
                territoryPie = _('territory-complete-pie');
                Chart.defaults.global.defaultFontColor = 'white';
                new Chart(territoryPie, {
                    "type": "doughnut",
                    "data": {
                        "labels": chart.team,
                        "datasets": [{
                            "label": "Win Odds",
                            "data": chart.power,
                            "backgroundColor": chart.background,
                            "hoverBackgroundColor": chart.hover
                        }],
                        font: {
                            color: 'white',
                        }
                    }
                });
                let display_headings_players = ['team', 'player', 'stars', 'weight', 'multiplier', 'power'];
                var obj_players = {
                    // Quickly get the headings
                    headings: ['Team', 'Player', 'Stars', 'Weight', 'Multiplier', 'Power'],

                    // data array
                    data: []
                };
                for (var i = 0; i < territoryTurn.players.length; i++) {

                    obj_players.data[i] = [];

                    for (var p in territoryTurn.players[i]) {
                        if (territoryTurn.players[i].hasOwnProperty(p) && display_headings_players.indexOf(p) != -1) {
                            if (p == 'team') {
                                obj_players.data[i].push("<a href=\"/team/{{team}}\" >{{team}}</a>".replace(/{{team}}/gi, territoryTurn.players[i][p]));
                            } else if (p == 'player') {
                                obj_players.data[i].push("<a href=\"/player/{{player}}\" {{star_style}}>{{star}}{{player}}</a>".replace(/{{player}}/gi, territoryTurn.players[i][p]).replace(/{{star_style}}/, (territoryTurn.players[i]['mvp']) ? "style=\"color:var(--theme-accent-1);\"" : "").replace(/{{star}}/, (territoryTurn.players[i]['mvp']) ? '✯' : ''));
                            } else {
                                obj_players.data[i].push(territoryTurn.players[i][p]);
                            }
                        }
                    }
                }
                try {
                    window.datatable2.destroy();
                } catch {
                    // don't do anything, nor output to table ;)
                } finally {
                    console.log(obj_players);
                    window.datatable2 = new DataTable("#territory-complete-players-table", {
                        data: obj_players,
                        columns: obj_players.columns,
                        searchable: false,
                        perPageSelect: false,
                        footer: false,
                        labels: {
                            info: "",
                        }
                    });
                }

            }, console.log
        )
    }
}

function page_territory_cover(contentTag, tname) {
    let territory_history = new Promise((resolve, reject) => {
        getTurns(resolve, reject);
    }).then(() => {
        //get MaxMin
        turn_maxmin = getMaxMin(window.turnsObject, "season");
        max_season = turn_maxmin[0].season;
        //fetch territory's history ;)
        doAjaxGetRequest("/api/territory/history?territory=" + tname + "&season=" + max_season, 'Territory Cover', function(territoryResponse) {
            var templateTerritoryHistory = _("templateTerritoryHistory");
            var box = _("templateTerritoryHistoryBox");
            var str = "";
            territoryHistoryObject = JSON.parse(territoryResponse.response);
            for (obj in territoryHistoryObject) {
                var objr = territoryHistoryObject.length - obj - 1;
                str += box.innerHTML.replace(/{{day}}/gi, territoryHistoryObject[objr].day).replace(/{{team}}/, territoryHistoryObject[objr].owner).replace(/{{season}}/, territoryHistoryObject[objr].season);
            }
            if (typeof territoryHistoryObject[0].territory === 'undefined' || territoryHistoryObject[0].territory === null) {
                contentTag.innerHTML = templateTerritoryHistory.innerHTML.replace(/{{objs}}/, str).replace(/{{TerritoryName}}/gi, decodeURIComponent(tname));
            } else {
                contentTag.innerHTML = templateTerritoryHistory.innerHTML.replace(/{{objs}}/, str).replace(/{{TerritoryName}}/gi, territoryHistoryObject[0].territory); // use the first element to capitalize if the url requires it. otherwise territoryHistoryObject[objr].day
            }
        }, console.log)
    });
}

function page_index(contentTag) {
    /*objects:
        1. map
        2. userinfo / team info
        3. roll
        */
    var templateMap = _("templateMap");
    var templateRoll = _("templateRoll");
    contentTag.innerHTML += templateMap.innerHTML;
    contentTag.innerHTML += templateRoll.innerHTML;
    let index = Promise.all([new Promise(drawMap), new Promise(getTeamInfo)])
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
                getTurns(resolve, reject);
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
        .then(() => {
            return new Promise((resolve, reject) => {
                setUpCounter(resolve, reject);
            })
        })
        .then(() => {
            return new Promise((resolve, reject) => {
                if (typeof appInfo.userObject != 'undefined') {
                    getAndHighlightMove(resolve, reject);
                }
                _('map-note').style.display = 'unset';
                doPoll(false);
            })
        })
        .catch((values) => { console.log(values) });
}

function hideUnselectableTeams(season) {
    Array.from(document.querySelector("#team_select").options).forEach(function(option_element) {
        if (option_element.getAttribute("season") != season || option_element.value == "Unjoinable Placeholder") {
            option_element.style.display = "none";
        } else {
            //console.log(option_element);
            option_element.style.display = "flex";
        }
    });
}

function drawOddsPage(junk) {
    // get value of team_select
    // get value of day_select and break into season, day
    // show heat map, odds map
    // GET /team/odds?team=Texas&day=1&season=1
    // GET doAjaxGetRequest('/images/map.svg', 'Map', function(data) {
    // add all the chances together to get Expected terrritories,
    var team = _('team_select').value;
    var seasonday = _('day_select').value.split('.');
    var day = seasonday[1];
    var season = seasonday[0];
    //update the team select to only have this season's teams!
    hideUnselectableTeams(season);
    _("heat-notif").innerHTML = "Where " + team + " deployed forces";
    _("odds-notif").innerHTML = "Where " + team + " had the highest odds";
    doAjaxGetRequest('/api/team/odds?team=' + team.replace('&', '%26') + '&day=' + day + '&season=' + season, 'oddsfetch', function(oddsObject) {
        var territory_count = 0;
        var territory_expected = 0;
        var survival_odds = 1;
        oddsObject = JSON.parse(oddsObject.response);
        var obj = {
            // Quickly get the headings
            headings: ["Territory", "Owner", "Winner", "MVPs", "Players", "1✯", "2✯", "3✯", "4✯", "5✯", "Team<br/> Power", "Territory<br/> Power", "Chance"],

            // data array
            data: []
        };
        let player_mm = getMaxMin(oddsObject, 'players');
        var chance_max;
        var chance_min;
        for (var i = 0; i < oddsObject.length; i++) {
            if (chance_max == null || oddsObject[i]['chance'] > chance_max['chance'])
                chance_max = oddsObject[i];
            if (chance_min == null || oddsObject[i]['chance'] < chance_min['chance'])
                chance_min = oddsObject[i];
        }
        let chance_mm = [chance_max, chance_min];
        _('heat-map').innerHTML = window.mapTemplate.replace(/id="/gi, 'id="heatmap_');
        heat_paths = _('heat-map').getElementsByTagName('path');
        for (hp = 0; hp < heat_paths.length; hp++) {
            heat_paths[hp].setAttribute('mapname', 'heat');
        }
        _('odds-map').innerHTML = window.mapTemplate.replace(/id="/gi, 'id="oddmap_');
        odds_paths = _('odds-map').getElementsByTagName('path');
        for (op = 0; op < odds_paths.length; op++) {
            odds_paths[op].setAttribute('mapname', 'odds');
        }
        for (i in oddsObject) {
            territory_count += (oddsObject[i].winner.replace(/\W/g, '') == team.replace(/\W/g, '')) ? 1 : 0;
            territory_expected += oddsObject[i].chance;
            survival_odds = survival_odds * (1 - oddsObject[i].chance);
            player_red = (oddsObject[i].players - player_mm[1].players) / (player_mm[0].players - player_mm[1].players) || 0;
            odds_red = (oddsObject[i].chance - chance_mm[1].chance) / (chance_mm[0].chance - chance_mm[1].chance) || 0;
            _("heatmap_".concat(oddsObject[i].territory.replace(/ /, ""))).style.fill = getColorForPercentage(player_red);
            _("heatmap_".concat(oddsObject[i].territory.replace(/ /, ""))).setAttribute('players', oddsObject[i].players);
            _("oddmap_".concat(oddsObject[i].territory.replace(/ /, ""))).style.fill = getColorForPercentage(odds_red);
            _("oddmap_".concat(oddsObject[i].territory.replace(/ /, ""))).setAttribute('odds', oddsObject[i].chance);
            obj.data.push(["<a href=\"/territory/{{terr}}\">{{terr}}</a>".replace(/{{terr}}/gi, oddsObject[i]['territory']),
                "<a href=\"/team/{{team}}\">{{team}}</a>".replace(/{{team}}/gi, oddsObject[i]["owner"]),
                "<a href=\"/team/{{team}}\">{{team}}</a>".replace(/{{team}}/gi, oddsObject[i]["winner"]),
                "<a href=\"/player/{{player}}\">{{player}}</a>".replace(/{{player}}/gi, oddsObject[i]["mvp"]),
                oddsObject[i]["players"],
                oddsObject[i]["starBreakdown"]["ones"],
                oddsObject[i]["starBreakdown"]["twos"],
                oddsObject[i]["starBreakdown"]["threes"],
                oddsObject[i]["starBreakdown"]["fours"],
                oddsObject[i]["starBreakdown"]["fives"],
                oddsObject[i]["teamPower"],
                oddsObject[i]["territoryPower"],
                oddsObject[i]["chance"].toFixed(2)
            ]);
        }
        //resizeMap();
        try {
            window.datatable.destroy();
        } catch {
            // don't do anything, nor output to table ;)
        } finally {
            window.datatable = new DataTable("#odds-players-table", {
                data: obj,
                columns: obj.columns,
                searchable: false,
                perPageSelect: false,
                footer: false,
                labels: {
                    info: "",
                }
            });
        }

        _('odds-survival').innerHTML = Math.floor(100 * (1 - survival_odds)) + "%";
        _('odds-expect').innerHTML = territory_expected.toFixed(2);
        _('odds-actual').innerHTML = territory_count.toFixed(2);
        _('leaderboard-wrapper').style.display = 'flex';
        _('action-container').style.display = 'flex';
    });
}

function page_odds(contentTag) {
    // We just dump the grid and such, then let the user sort out what they want
    contentTag.innerHTML = _('templateOdds').innerHTML;
    doAjaxGetRequest(appInfo.map, 'Map', function(data) { window.mapTemplate = data.response; });
    // we now populate the two lists with options, need a list of teams and a list of turns
    Promise.all([new Promise(getTeamInfo), new Promise((resolve, reject) => {
            getTurns(resolve, reject);
        })])
        .then((values) => {
            //make pretty thingy 
            str = '<select onchange="drawOddsPage(this.value); " name="team_select" id="team_select">';
            maxSeason = 0;
            for (i in values[0]) {
                str += "<option name=\"team_select\" season = \"" + values[0][i].seasons[0] + "\" value=\"" + values[0][i].name + "\">" + values[0][i].name + "</option>";
                if (values[0][i].seasons[0] > maxSeason) {
                    maxSeason = values[0][i].seasons[0];
                }
            }
            _("map-owner-info").innerHTML = seasonDayObject(0, 0, autoup = false, 'drawOddsPage', values[1]);
            _("map-owner-teams").innerHTML = str;
            _("map-owner-info").setAttribute('selectitem', 'true');
            hideUnselectableTeams(maxSeason);
            console.log(values);
        }).then(() => {
            return new Promise((resolve, reject) => {
                setupMapHover(resolve, reject);
            })
        });
}


function drawTeamPage(teamsObject, teamTurnsObject, team) {
    var capname = decodeURIComponent(team);
    for (x in teamsObject) {
        console.log(team, teamsObject[x].name);
        if (teamsObject[x].name.replace(/\W/g, '').toLowerCase() == capname.replace(/\W/g, '').toLowerCase()) {
            _("team-logo").setAttribute('src', teamsObject[x].logo);
            capname = teamsObject[x].name.replace(/\W/g, '');
            _('team-header').innerHTML = "<h1>" + capname + "</h1>";
            break;
        }
    }

    teamTurnsObject = JSON.parse(teamTurnsObject.response);
    var lastTeamTurn = teamTurnsObject[teamTurnsObject.length - 1];
    _('team-prev-players').innerHTML = "Players: " + lastTeamTurn.players;
    _('team-prev-stars').innerHTML = "Star power: " + lastTeamTurn.starPower;
    let display_headings = ["season", "day", "territories", "players", "starPower", "effectivePower"];

    var power_data = [];

    var player_counts = [
        [],
        [],
        [],
        [],
        []
    ];

    var obj = {
        // Quickly get the headings
        headings: ["Season", "Day", "Players", "Territories", "Star Power", "Effective Power"],

        // data array
        data: []
    };

    // Loop over the objects to get the values
    for (var i = 0; i < teamTurnsObject.length; i++) {
        obj.data[i] = [];
        power_data.push({ x: i, y: teamTurnsObject[i]['players'] });
        player_counts[0].push({ x: i, y: teamTurnsObject[i]['starbreakdown']['ones'] });
        player_counts[1].push({ x: i, y: teamTurnsObject[i]['starbreakdown']['twos'] });
        player_counts[2].push({ x: i, y: teamTurnsObject[i]['starbreakdown']['threes'] });
        player_counts[3].push({ x: i, y: teamTurnsObject[i]['starbreakdown']['fours'] });
        player_counts[4].push({ x: i, y: teamTurnsObject[i]['starbreakdown']['fives'] });
        for (var p in teamTurnsObject[i]) {
            if (teamTurnsObject[i].hasOwnProperty(p) && display_headings.indexOf(p) != -1) {
                obj.data[i].push(teamTurnsObject[i][p]);
            }
        }
    }
    //first we fill the charts
    Chart.defaults.global.defaultFontColor = 'black';
    var starChart = new Chart(_("star-power-history"), {
        type: 'line',
        backgroundColor: 'white',
        data: {
            datasets: [{
                label: 'Star Power',
                data: power_data,
                borderColor: 'rgba(255,0,0,0.5)',
                backgroundColor: 'rgba(255,0,0,0)'
            }]
        },
        options: {
            scales: {
                xAxes: [{
                    type: 'linear',
                    position: 'bottom',
                    scaleLabel: {
                        display: true,
                        labelString: 'Day'
                    }
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: 'Star Power'
                    }
                }]
            }
        }
    });
    var playerHistory = new Chart(_("player-history"), {
        type: 'line',
        data: {
            datasets: [{
                    label: 'Ones',
                    data: player_counts[0],
                    borderColor: 'rgba(255,0,0,0.5)',
                    backgroundColor: 'rgba(255,0,0,0)'
                },
                {
                    label: 'Twos',
                    data: player_counts[1],
                    borderColor: 'rgba(0,255,0,0.5)',
                    backgroundColor: 'rgba(255,0,0,0)'
                },
                {
                    label: 'Threes',
                    data: player_counts[2],
                    borderColor: 'rgba(0,0,255,0.5)',
                    backgroundColor: 'rgba(255,0,0,0)'
                },
                {
                    label: 'Fours',
                    data: player_counts[3],
                    borderColor: 'rgba(0,255,255,0.5)',
                    backgroundColor: 'rgba(255,0,0,0)'
                },
                {
                    label: 'Fives',
                    data: player_counts[4],
                    borderColor: 'rgba(255,0,255,0.5)',
                    backgroundColor: 'rgba(255,0,0,0)'
                }
            ]
        },
        options: {
            scales: {
                xAxes: [{
                    type: 'linear',
                    position: 'bottom',
                    scaleLabel: {
                        display: true,
                        labelString: 'Day'
                    }
                }],
                yAxes: [{
                    scaleLabel: {
                        display: true,
                        labelString: 'Players'
                    }
                }]
            }
        }
    });
    //then we fill the table
    try {
        window.datatable.destroy();
    } catch {
        // don't do anything, nor output to table ;)
    } finally {
        window.datatable = new DataTable("#team-turns-table", {
            data: obj,
            columns: obj.columns,
            searchable: false,
            perPageSelect: false,
            footer: false,
            labels: {
                info: "",
            }
        });
    }
    //then we fill the header

    _('teamPlayerHint').innerHTML = "<center><a href = \"/team/{{team}}/players\"> See all of {{team_c}}'s players </a></center>".replace(/{{team}}/gi, team).replace(/{{team_c}}/gi, capname);
}

function drawTeamPlayersPage(teamsObject, teamPlayersObject, team) {
    teamPlayersObject = JSON.parse(teamPlayersObject.response);
    let display_headings = ["player", "turnsPlayed", "mvps"];

    var obj = {
        // Quickly get the headings
        headings: ["Player", "Turns Played", "MVPs", "Last Turn"],

        // data array
        data: []
    };

    for (var i = 0; i < teamPlayersObject.length; i++) {

        obj.data[i] = [];

        for (var p in teamPlayersObject[i]) {
            if (teamPlayersObject[i].hasOwnProperty(p) && display_headings.indexOf(p) != -1) {
                if (p == 'player') {
                    obj.data[i].push("<a href=\"/player/{{player}}\">{{player}}</a>".replace(/{{player}}/gi, teamPlayersObject[i][p]));
                } else {
                    obj.data[i].push(teamPlayersObject[i][p]);
                }
            }
        }
        obj.data[i].push("Season: {{s}}, Day: {{d}}".replace(/{{s}}/gi, teamPlayersObject[i]['lastTurn']['season']).replace(/{{d}}/gi, teamPlayersObject[i]['lastTurn']['day']));
    }

    console.log(obj);

    try {
        _('team-header').innerHTML = "<h1>" + teamPlayersObject[0]['team'] + "</h1>";
    } catch {
        // eh
    }

    try {
        window.datatable.destroy();
    } catch {
        // don't do anything, nor output to table ;)
    } finally {
        window.datatable = new DataTable("#team-turns-table", {
            data: obj,
            columns: obj.columns,
            searchable: true,
            perPageSelect: false,
            footer: false,
            labels: {
                info: "",
            }
        });
    }
}

function page_team_players(contentTag, team) {
    var templateTeam = _("templateTeamPlayers");
    contentTag.innerHTML += templateTeam.innerHTML;
    _('team-header').innerHTML = "<h1>" + decodeURIComponent(team) + "</h1>";
    var templateTeamPage = _("templateTeamPage");
    let team_page_2 = new Promise((resolve, reject) => {
            getTeamInfo(resolve, reject);
        })
        .then((values) => {
            doAjaxGetRequest('/api/players?team=' + team.replace('&', '%26'), 'TeamPlayersFetch', function(data) {
                drawTeamPlayersPage(values, data, team);
            });
        })
}

function page_team(contentTag, team) {
    // load the teams info and save to tag
    // /api/stats/team/history
    var templateTeam = _("templateTeam");
    contentTag.innerHTML += templateTeam.innerHTML;
    _('team-header').innerHTML = "<h1>" + decodeURIComponent(team) + "</h1>";
    var templateTeamPage = _("templateTeamPage");
    let team_page_2 = new Promise((resolve, reject) => {
            getTeamInfo(resolve, reject);
        })
        .then((values) => {
            doAjaxGetRequest('/api/stats/team/history?team=' + team.replace('&', '%26'), 'TeamFetch', function(data) {
                drawTeamPage(values, data, team);
            });
        })
}

function page_player(contentTag, pid) {
    //fetch player info
    let leaderboard = new Promise((resolve, reject) => {
        getTeamInfo(resolve, reject);
    });
    var templatePlayerCardWrap = _("templatePlayerCardWrap");
    var templateHistory = _("templateHistory");
    contentTag.innerHTML += templatePlayerCardWrap.innerHTML;
    contentTag.innerHTML += templateHistory.innerHTML;
    doAjaxGetRequest('/api/player?player=' + pid, 'UserLoader', function(playerObject) {
            //Get team
            playerObject = JSON.parse(playerObject.response);
            console.log(playerObject);
            let active_team = playerObject.team || {
                name: null
            };
            if (active_team.name == null) {
                _('playerCard').innerHTML = "Sorry, user doesn't have a team yet.";
            } else {
                doAjaxGetRequest(encodeURI('/api/stats/team?team='.concat(playerObject.team.name)).replace(/&/, '%26'), 'TeamLoader', function(pteamObject) {
                    pteamObject = JSON.parse(pteamObject.response);
                    drawPlayerCard(playerObject, pteamObject);
                    drawUserTurnHistory(playerObject);
                }, function() {});
            }

        },
        function() {
            _('playerCard').innerHTML = "Hmm, user does not exist";
        });
}

function page_bug() {
    console.log("buggy!");
    if (typeof BrowserInfo === "undefined") {
        var Browserinfo = {
            init: function() {
                this.browser = this.searchString(this.dataBrowser) || "An unknown browser";
                this.version = this.searchVersion(navigator.userAgent) || this.searchVersion(navigator.appVersion) || "an unknown version";
                this.OS = this.searchString(this.dataOS) || "an unknown OS";
                this.cookies = navigator.cookieEnabled;
                this.language = (this.browser === "Explorer" ? navigator.userLanguage : navigator.language);
                this.colors = window.screen.colorDepth;
                this.browserWidth = window.screen.width;
                this.browserHeight = window.screen.height;
                this.java = (navigator.javaEnabled() == 1 ? true : false);
                this.codeName = navigator.appCodeName;
                this.cpu = navigator.oscpu;
                this.useragent = navigator.userAgent;
                this.plugins = navigator.plugins;
                this.ipAddress();
            },
            searchString: function(data) {
                for (var i = 0; i < data.length; i++) {
                    var dataString = data[i].string;
                    var dataProp = data[i].prop;
                    this.versionSearchString = data[i].versionSearch || data[i].identity;
                    if (dataString) {
                        if (dataString.indexOf(data[i].subString) != -1) return data[i].identity;
                    } else if (dataProp) return data[i].identity;
                }
            },
            searchVersion: function(dataString) {
                var index = dataString.indexOf(this.versionSearchString);
                if (index == -1) return;
                return parseFloat(dataString.substring(index + this.versionSearchString.length + 1));
            },

            ipAddress: function() {

                if (navigator.javaEnabled() && (navigator.appName != "Microsoft Internet Explorer")) {
                    vartool = java.awt.Toolkit.getDefaultToolkit();
                    addr = java.net.InetAddress.getLocalHost();
                    this.host = addr.getHostName();
                    this.ip = addr.getHostAddress();
                } else {
                    this.host = false;;
                    this.ip = false;
                }

            },

            screenSize: function() {
                var myWidth = 0,
                    myHeight = 0;
                if (typeof(window.innerWidth) == 'number') {
                    //Non-IE
                    this.browserWidth = window.innerWidth;
                    this.browserHeight = window.innerHeight;
                } else if (document.documentElement && (document.documentElement.clientWidth || document.documentElement.clientHeight)) {
                    //IE 6+ in 'standards compliant mode'
                    this.browserWidth = document.documentElement.clientWidth;
                    this.browserHeight = document.documentElement.clientHeight;
                } else if (document.body && (document.body.clientWidth || document.body.clientHeight)) {
                    //IE 4 compatible
                    this.browserWidth = document.body.clientWidth;
                    this.browserHeight = document.body.clientHeight;
                }
            },
            dataBrowser: [{
                string: navigator.userAgent,
                subString: "Chrome",
                identity: "Chrome"
            }, {
                string: navigator.userAgent,
                subString: "OmniWeb",
                versionSearch: "OmniWeb/",
                identity: "OmniWeb"
            }, {
                string: navigator.vendor,
                subString: "Apple",
                identity: "Safari",
                versionSearch: "Version"
            }, {
                prop: window.opera,
                identity: "Opera"
            }, {
                string: navigator.vendor,
                subString: "iCab",
                identity: "iCab"
            }, {
                string: navigator.vendor,
                subString: "KDE",
                identity: "Konqueror"
            }, {
                string: navigator.userAgent,
                subString: "Firefox",
                identity: "Firefox"
            }, {
                string: navigator.vendor,
                subString: "Camino",
                identity: "Camino"
            }, { // for newer Netscapes (6+)
                string: navigator.userAgent,
                subString: "Netscape",
                identity: "Netscape"
            }, {
                string: navigator.userAgent,
                subString: "MSIE",
                identity: "Explorer",
                versionSearch: "MSIE"
            }, {
                string: navigator.userAgent,
                subString: "Gecko",
                identity: "Mozilla",
                versionSearch: "rv"
            }, { // for older Netscapes (4-)
                string: navigator.userAgent,
                subString: "Mozilla",
                identity: "Netscape",
                versionSearch: "Mozilla"
            }],
            dataOS: [{
                string: navigator.platform,
                subString: "Win",
                identity: "Windows"
            }, {
                string: navigator.platform,
                subString: "Mac",
                identity: "Mac"
            }, {
                string: navigator.userAgent,
                subString: "iPhone",
                identity: "iPhone/iPod"
            }, {
                string: navigator.platform,
                subString: "Linux",
                identity: "Linux"
            }]

        }
        Browserinfo.init();

        BrowserInfo = {
            os: Browserinfo.OS,
            browser: Browserinfo.browser,
            version: Browserinfo.version,
            cookies: Browserinfo.cookies,
            language: Browserinfo.language,
            browserWidth: Browserinfo.browserWidth,
            browserHeight: Browserinfo.browserHeight,
            java: Browserinfo.java,
            colors: Browserinfo.colors,
            codeName: Browserinfo.codeName,
            host: Browserinfo.host,
            cpu: Browserinfo.cpu,
            useragent: Browserinfo.useragent,
            cookies: document.cookie
        };
    }

    bug_form = _("bug_form");
    bug_form = bug_form.innerHTML;
    bug_form = bug_form.replace(/{{uinf}}/, encodeURI(JSON.stringify(BrowserInfo)))
        .replace(/{{errors}}/, encodeURI(JSON.stringify(appInfo.errorNotifications))).replace(/{{pending}}/, encodeURI(JSON.stringify(appInfo.outstandingRequests)));
    errorNotif('Bug Report', bug_form, {
        text: "Okay",
        action: function() {
            console.log("Submit");
            window.history.back();
        },
    }, {
        display: "none",
        action: function() { window.history.back(); }
    });
}

function page_map(content, data = { season: 0, day: 0 }) {
    //collect turninfo if it does not yet exist
    //draw <- Season: # Day: # ->
    //draw map
    //apply filters if requested by user
    var templateMap = _("templateMap");
    templateMap = templateMap.innerHTML;
    content.innerHTML += templateMap;
    let map = new Promise((resolve, reject) => {
        getTurns(resolve, reject);
        getTeamInfo(resolve, reject);
    }).then(() => {
        return new Promise((resolve, reject) => {
            drawMap(resolve, reject, "territories", data.season, data.day);
        })
    }).then(() => {
        return new Promise((resolve, reject) => {
            setupMapHover(resolve, reject);
            // find the turn element
            const dayId = !(data.season == 0) ? window.turnsObject.find(el => el.season == data.season && el.day == data.day).id : window.turnsObject[window.turnsObject.length - 1].id;
            console.log(dayId);
            var tagtemplate = '';
            if (typeof window.turnsObject.find(el => el.id == dayId - 1) != "undefined") {
                tagtemplate += '<a href="/map/{{pseason}}/{{pday}}">&#11160;</a>'
                    .replace(/{{pseason}}/, window.turnsObject.find(el => el.id == dayId - 1).season)
                    .replace(/{{pday}}/, window.turnsObject.find(el => el.id == dayId - 1).day);
            }
            tagtemplate += '  Season {{season}}, Day {{day}}  '
                .replace(/{{season}}/, window.turnsObject.find(el => el.id == dayId).season)
                .replace(/{{day}}/, window.turnsObject.find(el => el.id == dayId).day);
            if (typeof window.turnsObject.find(el => el.id == dayId + 1) != "undefined") {
                tagtemplate += '<a href="/map/{{nseason}}/{{nday}}">&#11162;</a>'
                    .replace(/{{nseason}}/, window.turnsObject.find(el => el.id == dayId + 1).season)
                    .replace(/{{nday}}/, window.turnsObject.find(el => el.id == dayId + 1).day);
            }
            _('map-day-info').innerHTML = tagtemplate;
            _('old-map-owner-info').style.display = 'none';
            _('map-day-info').style.display = 'unset';
        })
    });
}

function handleNewPage(title, contentTag, call, vari) {
    if (new Date() > appInfo.dDay) {
        clearInterval(window.pulse);
        sky();
    }
    contentTag.innerHTML = "";
    appInfo.lockDisplay = false;
    document.title = "Aggie Risk | " + title;
    clearInterval(window.pulse);
    call(contentTag, vari);
    if (appInfo.burgerTrigger) {
        appInfo.burger = false;
        _('nav').style.display = (appInfo.burger) ? 'flex' : 'none';
    }
}

function paintPoll() {
    if (appInfo.pollResponses.length == appInfo.pollData.length) {
        //whoop!
        console.log("Okay.");
        //present them with the poll machine!
        askPoll(0);
    } else {
        console.log("Shoot! Couldn't get poll responses.");
    }
}

function askPoll(number) {
    numberp1 = number + 1;
    early = (appInfo.pollData[0].day + 7).toString();
    late = (appInfo.pollData[0].day + 14).toString();
    appInfo.currentPoll = appInfo.pollData[0].id;
    currResp = "Not responded";
    for (j = 0; j < appInfo.pollResponses.length; j++) {
        if (appInfo.pollResponses[j].length > 0) {
            if (appInfo.pollResponses[j][0].response == true) {
                currResp = "Yes";
            } else {
                currResp = "No";
            }
        }
    }
    errorNotif('Polls ' + numberp1 +
        ' of ' + appInfo.pollData.length, appInfo.pollData[0].question + "<br />This would take the season from " + early + " to " + late + " days. <br/><br/> Your current response is: <b>" + currResp + " </b><div id='pollResponseError'></div>", {
            text: "Yes",
            action: function() {
                doAjaxGetRequest('/auth/poll/respond?poll=' + appInfo.currentPoll + '&response=' + true, 'Poll Responder', function(data) {
                    if (data.status == 200) {
                        for (ei = 0; ei < appInfo.errorNotifications.length; ei++) {
                            if (appInfo.errorNotifications[ei].status == 1 && appInfo.errorNotifications[ei].title.includes("Poll")) {
                                appInfo.errorNotifications[ei].status = 0;
                                errorOver(ei);
                            }
                        }
                    } else {
                        _('pollResponseError').innerHTML = "<br/><br/><b style=\"red\">1 Hmm, something went wrong. Try again.</b>";
                    }
                }, function() {
                    _('pollResponseError').innerHTML = "<br/><br/><b style=\"red\">2 Hmm, something went wrong. Try again.</b>";
                });
            }
        }, {
            text: "No",
            action: function() {
                doAjaxGetRequest('/auth/poll/respond?poll=' + appInfo.currentPoll + '&response=' + false, 'Poll Responder', function(data) {
                    if (data.status == 200) {
                        for (ei = 0; ei < appInfo.errorNotifications.length; ei++) {
                            if (appInfo.errorNotifications[ei].status == 1 && appInfo.errorNotifications[ei].title.includes("Poll")) {
                                appInfo.errorNotifications[ei].status = 0;
                                errorOver(ei);
                            }
                        }
                    } else {
                        _('pollResponseError').innerHTML = "<br/><br/><b style=\"red\">1 Hmm, something went wrong. Try again.</b>";
                    }
                }, function() {
                    _('pollResponseError').innerHTML = "<br/><br/><b style=\"red\">2 Hmm, something went wrong. Try again.</b>";
                });
            }
        },
        false);
}

function doPoll(realize = true) {
    doAjaxGetRequest('/auth/polls', 'Poll Requests', function(pollData) {
        try {
            pollData = JSON.parse(pollData.response);
            appInfo.pollData = pollData;
            console.log(pollData);
            appInfo.pollResponses = [];
            console.log("Polling...");
            for (i = 0; i < pollData.length; i++) {
                if (realize || (pollData[i].season == window.turnsObject[window.turnsObject.length - 1].season && pollData[i].day == window.turnsObject[window.turnsObject.length - 1].day)) {
                    doAjaxGetRequest('/auth/poll/response?poll=' + pollData[i].id, 'Poll Response Requests', function(data) {
                        appInfo.pollResponses.push(JSON.parse(data.response));
                        paintPoll();
                    }, function() {
                        appInfo.pollResponses.push([]);
                        errorNotif('Error Parsing Polls', 'Hmm, appears somebody stole our voter rolls. Try again?', {
                            text: "Okay"
                        }, {
                            display: "none"
                        });
                    });
                }
            }
        } catch {
            errorNotif('Error Parsing Polls', 'Hmm, appears somebody stole our voter rolls. Try again?', {
                text: "Okay"
            }, {
                display: "none"
            });
        }
    }, function() {
        errorNotif('Could Not Fetch Polls', 'We could not fetch the polls. Try again?', {
            text: "Okay"
        }, {
            display: "none"
        });
    });
}

class Router {

    constructor(options) {
        this.routes = [];

        this.mode = null;

        this.root = '/';
        this.mode = window.history.pushState ? 'history' : 'hash';
        if (options.mode) this.mode = options.mode;
        if (options.root) this.root = options.root;



        this.add = (path, cb) => {
            this.routes.push({ path, cb });
            return this;
        };

        this.remove = path => {
            for (let i = 0; i < this.routes.length; i += 1) {
                if (this.routes[i].path === path) {
                    this.routes.slice(i, 1);
                    return this;
                }
            }
            return this;
        };

        this.flush = () => {
            this.routes = [];
            return this;
        };

        this.clearSlashes = path =>
            path
            .toString();
        //  .replace(/\/$/, '')
        // .replace(/^\//, '');

        this.getFragment = () => {
            let fragment = '';
            if (this.mode === 'history') {
                fragment = this.clearSlashes(decodeURI(window.location.pathname + window.location.search));
                console.log(fragment);
                fragment = fragment.replace(/\?(.*)$/, '');
                fragment = this.root !== '/' ? fragment.replace(this.root, '') : fragment;
            } else {
                const match = window.location.href.match(/(.*)$/);
                fragment = match ? match[1] : '';
            }
            return this.clearSlashes(fragment);
        };

        this.navigate = (path = '') => {
            if (this.mode === 'history') {
                window.history.pushState(null, null, this.root + this.clearSlashes(path));
            } else {
                window.location.href = `${window.location.href.replace(/(.*)$/, '')}#${path}`;
            }
            return this;
        };

        this.listen = () => {
            clearInterval(this.interval);
            this.interval = setInterval(this.interval, 50);
        };

        this.interval = () => {
            if (this.current === this.getFragment() || this.current + "#" === this.getFragment()) return;
            this.current = this.getFragment();

            this.routes.some(route => {
                const match = this.current.match(route.path);
                if (match) {
                    match.shift();
                    route.cb.apply({}, match);
                    return match;
                }
                return false;
            });
        };
        this.listen();
    }
}

const router = new Router({
    mode: 'hash',
    root: '/'
});

var contentTag = _('content-wrapper');

router
    .add('/leaderboard', () => {
        handleNewPage('Leaderboard', contentTag, page_leaderboard);
    })
    .add('/odds', () => {
        handleNewPage('Odds', contentTag, page_odds);
    })
    .add('/info', () => {
        handleNewPage('Information', contentTag, page_info);
    })
    .add(/team\/(.*)\/players/, (team) => {
        handleNewPage(team, contentTag, page_team_players, team.replace('#', ''));
    })
    .add('/team/(.*)', (team) => {
        handleNewPage(team, contentTag, page_team, team.replace('#', ''));
    })
    .add('/territory/(.*)/(.*)/(.*)', (territoryName, season, day) => {
        console.log(territoryName, season, day);
        handleNewPage(territoryName, contentTag, page_territory, { name: territoryName, season: season.replace('#', ''), day: day.replace('#', '') });
    })
    .add('/territory/(.*)', (territoryName) => {
        handleNewPage(territoryName, contentTag, page_territory_cover, territoryName);
    })
    .add('/map/(.*)/(.*)', (season, day) => {
        console.log("Loading map: season {{season}} day {{day}}".replace(/{{season}}/, season).replace(/{{day}}/, day));
        handleNewPage('Map', contentTag, page_map, { season: season.replace('#', ''), day: day.replace('#', '') });
    })
    .add('/map', () => {
        console.log("Loading map");
        handleNewPage('Map', contentTag, page_map);
    })
    .add('/bug', () => {
        console.log("Loading Bug Page");
        page_bug();
    })
    .add('/player/(.*)', (pid) => {
        handleNewPage(pid, contentTag, page_player, pid);
    })
    .add('/', () => {
        // general controller
        handleNewPage('Home', contentTag, page_index);
    })
    .add('', () => {
        console.log('404');
    });


/*** UTILITIES ***/

function _(id) {
    return document.getElementById(id);
}

function doDate() {
    if (new Date() > appInfo.dDay) {
        clearInterval(window.pulse);
        sky();
    }
    var templateRollInfo = _("templateRollInfo");
    templateRollInfo = templateRollInfo.innerHTML;
    var now = new Date();
    var str = ""
    var difference = appInfo.rollTime - now;
    var days = 0;
    var days = Math.floor(difference / 1000 / 24 / 60 / 60)
    difference -= days * 1000 * 24 * 60 * 60;
    var hours = Math.floor(difference / 1000 / 60 / 60);
    difference -= hours * 1000 * 60 * 60;
    var minutes = Math.floor(difference / 1000 / 60);
    difference -= minutes * 1000 * 60;
    var seconds = Math.floor(difference / 1000);
    difference -= seconds * 1000;
    str += templateRollInfo
        .replace(/{{day}}/, window.turn.day)
        .replace(/{{days}}/, pad(days, 'days', false, false, 0))
        .replace(/{{hours}}/, pad(hours, 'hours', false, false, days))
        .replace(/{{minutes}}/, pad(minutes, 'minutes', true, false, hours + days))
        .replace(/{{seconds}}/, pad(seconds, 'seconds', true, true, minutes + days + hours));
    _("rollInfo").innerHTML = str;
}

function doDate2() {
    var now = new Date();
    var str = "";
    var difference = appInfo.rollTime - now;
    //var days = 0;
    //var days = Math.floor(difference / 1000 / 24 / 60 / 60)
    //difference -= days * 1000 * 24 * 60 * 60;
    var phours = Math.floor(difference / 1000 / 60 / 60);
    if (phours < 0) {
        merry();
    }
    difference -= phours * 1000 * 60 * 60;
    var minutes = Math.floor(difference / 1000 / 60);
    difference -= minutes * 1000 * 60;
    var seconds = Math.floor(difference / 1000);
    difference -= seconds * 1000;
    str = pad(phours, '', true, false, 1).slice(0, -2) + ":" + pad(minutes, '', true, false, 1).slice(0, -2) + ":" + pad(seconds, '', true, false, 1).slice(0, -2);
    _('big-clock').innerHTML = str;
}

function getAndHighlightMove() {
    console.log("Making request for current move.");
    doAjaxGetRequest('/auth/my_move', 'Plotting Pretty Map', function(data) {
        highlightTerritory(data.response.replace(/"/g, ''));
    });
}

function getCookie(cname) {
    var name = cname + "=";
    var decodedCookie = decodeURIComponent(document.cookie);
    var ca = decodedCookie.split(';');
    for (var i = 0; i < ca.length; i++) {
        var c = ca[i];
        while (c.charAt(0) == ' ') {
            c = c.substring(1);
        }
        if (c.indexOf(name) == 0) {
            return c.substring(name.length, c.length);
        }
    }
    return "";
}

function getMaxMin(arr, prop) {
    var max;
    var min;
    for (var i = 0; i < arr.length; i++) {
        if (max == null || parseInt(arr[i][prop]) > parseInt(max[prop]))
            max = arr[i];
        if (min == null || parseInt(arr[i][prop]) < parseInt(min[prop]))
            min = arr[i];
    }
    return [max, min];
}

function highlightTerritory(territory) {
    console.log("Highlighting {{Territory}}".replace(/{{Territory}}/, territory));
    let highlighted = document.getElementsByClassName('map-animated-highlight');
    for (i = 0; i < highlighted.length; i++) {
        highlighted[i].classList.remove('map-animated-highlight');
    }
    _('map').getElementById(territory.replace(/ /, '')).classList = 'map-animated-highlight';
}

function link_is_external(link_element) {
    return (link_element.host !== window.location.host);
}

function pad(number, notion, final, next, prev) {
    if (number != 0 || prev != 0) {
        return (next == true && prev != 0 ? "and " : "") + (number < 10 ? "0" : "") + number + " " + notion + (final == false ? ", " : " ");
    } else {
        return '';
    }
    if (prev == 0 && number == 0) {
        appInfo.rollTime.setUTCDate(appInfo.rollTime.getUTCDate() + 1)
    }
}

function resizeGlobal() {
    try {
        resizeMap();
    } catch {
        //we're not on the main page. :shrug:
        console.log("Could not resize map. Not on main page.");
    }
}

function setUpCounter(resolve, reject) {
    window.pulse = setInterval(doDate, 1000);
    resolve();
}

var percentColors = [
    { pct: 0.0, color: { r: 0x00, g: 0xff, b: 0 } },
    { pct: 0.5, color: { r: 0xff, g: 0xff, b: 0 } },
    { pct: 1.0, color: { r: 0xff, g: 0x00, b: 0 } },
];
var getColorForPercentage = function(pct) {
    for (var i = 1; i < percentColors.length - 1; i++) {
        if (pct < percentColors[i].pct) {
            break;
        }
    }
    var lower = percentColors[i - 1];
    var upper = percentColors[i];
    var range = upper.pct - lower.pct;
    var rangePct = (pct - lower.pct) / range;
    var pctLower = 1 - rangePct;
    var pctUpper = rangePct;
    var color = {
        r: Math.floor(lower.color.r * pctLower + upper.color.r * pctUpper),
        g: Math.floor(lower.color.g * pctLower + upper.color.g * pctUpper),
        b: Math.floor(lower.color.b * pctLower + upper.color.b * pctUpper)
    };
    return 'rgba(' + [color.r, color.g, color.b].join(',') + ',0.5)';
    // or output as hex if preferred
}


/*** Holiday Special ***/
function merry() {
    appInfo.rollTime = new Date("December 26, 2020 04:00:00");
    appInfo.rollTime.setUTCHours(4, 0, 0, 0);

    if (appInfo.rollTime < new Date()) {
        appInfo.rollTime = new Date();
        appInfo.rollTime.setUTCHours(4, 0, 0, 0);
        if (appInfo.rollTime < new Date()) {
            appInfo.rollTime.setUTCDate(appInfo.rollTime.getUTCDate() + 1)
        }
    }
}

function sky() {
    //fade:
    try {
        clearInterval(window.pulse);
        clearTimeout(appInfo.fadeTimer);
        clearTimeout(sky2t);
        clearTimeout(window.pulse2);
    } catch {

    }
    if (getCookie('seen') == 'true') {
        sky2();
    } else {
        appInfo.fullOpacity = 1;
        appInfo.fadeTimer = setInterval(
            function() {
                appInfo.fullOpacity = appInfo.fullOpacity - 0.1;
                _('reddit-login-top').style.opacity = appInfo.fullOpacity;
                _('nav').style.opacity = appInfo.fullOpacity;
                _('content-wrapper').style.opacity = appInfo.fullOpacity;
                document.getElementsByTagName('footer')[0].style.opacity = appInfo.fullOpacity;
                if (appInfo.fullOpacity <= 0) {
                    console.log("Exit");
                    document.cookie = "seen=true; expires=Thu, 28 Dec 2020 12:00:00 UTC; path=/; samesite=lax;";
                    clearInterval(window.pulse);
                    clearTimeout(appInfo.fadeTimer);
                    clearTimeout(window.pulse2);
                    clearTimeout(sky2t);
                    var sky2t = setTimeout(sky2, 200);
                }
            }, 200);
    }
}

function sky2() {
    try {
        _('reddit-login-top').style.display = "none";
    } finally {
        _('nav').style.display = "none";
        _('content-wrapper').style.display = "none";
        document.getElementsByTagName('footer')[0].style.display = "none";
        document.getElementsByTagName('body')[0].style.background = 'black';
        document.getElementsByTagName('body')[0].innerHTML += "<h1 style=\"color:var(--theme-accent-1);font-family:digitalClock; font-size:35vh;text-align:center;margin-top:32.5vh;\" id=\"big-clock\">00:00:00</h1><h2 style=\"text-align: center;margin-top: 10vh;\"><a href=\"https://tx.ag/AggieDiscord\">Join the Discord</a></h2>";
        appInfo.rollTime = new Date("December 26, 2020 04:00:00");
        appInfo.rollTime.setUTCHours(4, 0, 0, 0);

        merry();

        window.pulse2 = setInterval(doDate2, 1000);
    }
}
// @license-end