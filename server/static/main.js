// Error and Metric Reporting Agent Includes
var outstandingRequests = [];
var globalError = false;
function doAjaxGetRequest(url, source, callback, errorcallback) {
    addUrlFromRequests(url);
    var xhttp = new XMLHttpRequest();
    xhttp.onreadystatechange = function () {
        if (this.readyState == 4 && this.status == 200) {
            callback;
        } else {
            if ()
                globalError = true;
            updateLoaderVisibility();
            document.getElementsById("loadicon").classList.add("blink");
        }
    };
    xhttp.open("GET", url, true);
    xhttp.send();
    removeUrlFromRequests(url);
}

function addUrlFromRequests(url) {
    outstandingRequests.push(url);
    updateLoaderVisibility();
}

function removeUrlFromRequests(url) {
    const index = outstandingRequests.indexOf(url);
    if (index > -1) {
        outstandingRequests.splice(index, 1);
    }
    updateLoaderVisibility();
}

function updateLoaderVisibility(forceHide = false) {
    if (outstandingRequests.length == 0 && forceHide === false) {
        //stop loader
        document.getElementsById("loadicon").classList.remove("spin");
    } else {
        //start loader
        //check if globalError
        if (!globalError) {
            document.getElementsById("loadicon").classList.add("spin");
        }
    }
}