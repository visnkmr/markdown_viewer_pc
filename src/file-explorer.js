"use strict";
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;
var interval;
window.addEventListener("DOMContentLoaded", () => {
    window.__TAURI__.invoke("list_files", { path: "/home/roger/Downloads/github/"
    });
    const backButton = document.getElementById("back-button");
    var lastfolder = "/home/roger/Downloads/github/";
    backButton.addEventListener("click", () => {
        if (lastfolder === "")
            lastfolder = ".";
        pathInput.value = lastfolder;
        htmlbase.innerHTML = "";
        window.__TAURI__.invoke("list_files", { path: lastfolder
        });
    });
    const pathInput = document.getElementById("path-input");
    const listButton = document.getElementById("list-button");
    const fileList = document.getElementById("file-list");
    const htmlbase = document.getElementById("htmlbase");
    const parentsize = document.getElementById("parent-size");
    listButton.addEventListener("click", async () => {
        let path = pathInput.value;
        await window.__TAURI__.invoke("list_files", { path: path
        });
        pathInput.value = path;
    });
    fileList.addEventListener("click", async (event) => {
        console.log("here");
        let target = event.target;
        console.log(target.tagName);
        if (target.tagName === "TD") {
            console.log(target.dataset);
            let name = target.dataset.name;
            let path = target.dataset.path;
            let isDir = target.dataset.isDir;
            if (isDir === "true") {
                console.log("dir");
                pathInput.value = path;
                parentsize.innerHTML = target.dataset.parentsize;
                window.__TAURI__.invoke("list_files", {
                    path: path
                });
            }
            else if (name.toLowerCase().endsWith(".md")) {
                {
                    fileList.innerHTML = "";
                    htmlbase.innerHTML = await window.__TAURI__.invoke("loadmarkdown", { name: path });
                    var links = document.getElementsByTagName("a");
                    for (var i = 0; i < links.length; i++) {
                        var link = links[i];
                        link.setAttribute("target", "_blank");
                    }
                }
            }
            else {
                window.__TAURI__.invoke("openpath", {
                    path: path
                });
            }
        }
    });
    const datalist = document.getElementById("path-list");
    pathInput.addEventListener("input", async () => {
        console.log("here");
        const path = pathInput.value;
        console.log(path);
        await window.__TAURI__.invoke("get_path_options", {
            path: path,
        })
            .then((options) => {
            console.log(options);
            if (options !== null) {
                datalist.innerHTML = "";
                for (const option of options) {
                    const optionElement = document.createElement("option");
                    console.log("here#1");
                    optionElement.value = option;
                    datalist.appendChild(optionElement);
                }
            }
        })
            .catch((error) => {
            console.error(error);
        });
    });
    window.__TAURI__.event.listen("list-files", (data) => {
        let files = JSON.parse(data.payload);
        console.log("files");
        fileList.innerHTML = "";
        let thead = document.createElement("thead");
        let tr = document.createElement("tr");
        let th1 = document.createElement("th");
        let th2 = document.createElement("th");
        th1.textContent = "Filename";
        th2.textContent = "Filesize";
        th1.id = "filename";
        th2.id = "filesize";
        tr.appendChild(th1);
        tr.appendChild(th2);
        thead.appendChild(tr);
        fileList.appendChild(thead);
        let tbody = document.createElement("tbody");
        for (let file of files) {
            let tr = document.createElement("tr");
            let td1 = document.createElement("td");
            td1.textContent = file.name;
            td1.dataset.value = file.name;
            td1.dataset.name = file.name;
            td1.dataset.path = file.path;
            td1.dataset.isDir = file.is_dir.toString();
            if (file.is_dir) {
                td1.id = "folder";
            }
            td1.dataset.size = file.size.toString();
            tr.appendChild(td1);
            let td2 = document.createElement("td");
            td2.textContent = file.size.toString();
            td2.dataset.value = file.rawfs.toString();
            tr.appendChild(td2);
            tbody.appendChild(tr);
        }
        fileList.appendChild(tbody);
        let order = "asc";
        function compare(a, b) {
            if (order === "asc") {
                return a < b ? -1 : a > b ? 1 : 0;
            }
            else {
                return a > b ? -1 : a < b ? 1 : 0;
            }
        }
        function sortTable(index) {
            let rows = Array.from(tbody.rows);
            rows.sort(function (a, b) {
                return compare(a.cells[index].dataset.value, b.cells[index].dataset.value);
            });
            for (let row of rows) {
                tbody.appendChild(row);
            }
            order = order === "asc" ? "desc" : "asc";
        }
        let filename = document.getElementById("filename");
        let filesize = document.getElementById("filesize");
        filename.addEventListener("click", function () {
            sortTable(0);
        });
        filesize.addEventListener("click", function () {
            sortTable(1);
        });
    });
    window.__TAURI__.event.listen("folder-size", (data) => {
        parentsize.innerHTML = data.payload.toString();
        console.log(data.payload.toString());
    });
    window.__TAURI__.event.listen("grandparent-loc", (data) => {
        lastfolder = data.payload.toString();
        console.log(data.payload.toString());
    });
    window.__TAURI__.event.listen("parent-loc", (data) => {
        pathInput.value = data.payload.toString();
        console.log(data.payload.toString());
    });
    window.__TAURI__.event.listen("start-timer", (data) => {
        updatetimer();
    });
    window.__TAURI__.event.listen("stop-timer", (data) => {
        clearInterval(interval);
    });
});
function updatetimer() {
    let timer = document.getElementById("timer");
    let startTime = new Date();
    interval = setInterval(function () {
        let currentTime = new Date();
        let elapsedTime = currentTime.getTime() - startTime.getTime();
        let minutes = Math.floor(elapsedTime / 1000 / 60);
        let seconds = Math.floor((elapsedTime / 1000) % 60);
        let paddedMinutes = minutes < 10 ? "0" + minutes : minutes.toString();
        let paddedSeconds = seconds < 10 ? "0" + seconds : seconds.toString();
        timer.textContent = paddedMinutes + ":" + paddedSeconds;
    }, 1000);
}
