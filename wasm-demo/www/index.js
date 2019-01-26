import "materialize-css/dist/css/materialize.min.css"
import "materialize-css/dist/js/materialize.min.js"
import * as wasm from "wasm-demo";

let canvas = document.getElementById("canv");

canvas.width = window.innerWidth;
canvas.height = window.innerHeight;

window.addEventListener("onresize", (event) => {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
});

let ctx = canvas.getContext("2d");

let coords = [];

ctx.fillStyle = "#fff";
ctx.fillRect(0, 0, canvas.width, canvas.height);

const redraw = () => {
    ctx.fillStyle = "#fff";
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = "#333";

    for (let i = 0; i < coords.length; i += 2) {
        ctx.beginPath();
        ctx.arc(coords[i], coords[i + 1], 2, 0, 2 * Math.PI);
        ctx.fill();
    }

    document.getElementById("numPoints").innerText = Math.floor(coords.length / 2);
    document.getElementById("numTris").innerText = 0;
    document.getElementById("elapsed").innerText = (0).toFixed(3);
    document.getElementById("rendering").innerText = (0).toFixed(3);

    if (coords.length < 6)
        return;

    const triStart = window.performance.now();
    const triangles = wasm.triangulate(new Float32Array(coords));
    const triEnd = window.performance.now();

    document.getElementById("numTris").innerText = Math.floor(triangles.length / 3);
    document.getElementById("elapsed").innerText = (triEnd - triStart).toFixed(3);

    const renderStart = window.performance.now();

    ctx.strokeStyle = "#777";

    for (let i = 0; i < triangles.length; i += 3) {
        let a = triangles[i];
        let b = triangles[i + 1];
        let c = triangles[i + 2];

        ctx.beginPath();
        ctx.moveTo(coords[2 * a], coords[2 * a + 1]);
        ctx.lineTo(coords[2 * b], coords[2 * b + 1]);
        ctx.lineTo(coords[2 * c], coords[2 * c + 1]);
        ctx.closePath();
        ctx.stroke();
    }

    ctx.fillStyle = "#333";

    for (let i = 0; i < coords.length; i += 2) {
        ctx.beginPath();
        ctx.arc(coords[i], coords[i + 1], 2, 0, 2 * Math.PI);
        ctx.fill();
    }

    const renderEnd = window.performance.now();
    document.getElementById("rendering").innerText = (renderEnd - renderStart).toFixed(3);
};

redraw();

let mouseDown = false;

M.AutoInit();

canvas.addEventListener("mousedown", () => mouseDown = true);
canvas.addEventListener("mouseup", () => mouseDown = false);
canvas.addEventListener("mouseleave", () => mouseDown = false);

const clickDrag = (event) => {
    if (!mouseDown && event.type != "click")
        return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    coords.push(x);
    coords.push(y);
    redraw();
};

canvas.addEventListener("mousemove", clickDrag);
canvas.addEventListener("click", clickDrag);

document.getElementById("btnClear").addEventListener("click", () => {
    coords.splice(0, coords.length);
    redraw();
});

document.getElementById("btnGenerate").addEventListener("click", () => {
    const count = Number(document.getElementById("countRange").value);

    switch (document.getElementById("typeSelector").value) {
        case "uniform":
            for (let i = 0; i < count; i++) {
                coords.push(Math.random() * (canvas.width - 100) + 50);
                coords.push(Math.random() * (canvas.height - 100) + 50);
            }

            break;

        case "grid":
            const size = Math.ceil(Math.sqrt(count))
            const sizeX = (canvas.width - 100) / (size - 1);
            const sizeY = (canvas.height - 100) / (size - 1);

            for (let x = 0; x < size; x++) {
                for (let y = 0; y < size; y++) {
                    coords.push(x * sizeX + 50);
                    coords.push(y * sizeY + 50);
                }
            }

            break;

        case "circle":
            const centerX = canvas.width / 2;
            const centerY = canvas.height / 2;
            const radius = Math.min(canvas.width - 100, canvas.height - 100) / 2;

            for (let i = 0; i < count; i++) {
                const angle = i / count * 2 * Math.PI;
                coords.push(Math.cos(angle) * radius + centerX);
                coords.push(Math.sin(angle) * radius + centerY);
            }

            break;

        default:
            return;
    }

    redraw();
});
