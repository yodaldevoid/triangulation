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
    if (coords.length >= 6) {
        ctx.fillStyle = "#fff";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        let triangles = wasm.triangulate(new Float32Array(coords));

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
    }
};

let mouseDown = false;

canvas.addEventListener("mousedown", () => mouseDown = true);
canvas.addEventListener("mouseup", () => mouseDown = false);

canvas.addEventListener("mousemove", (event) => {
    if (!mouseDown)
        return;

    const rect = canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    coords.push(x);
    coords.push(y);
    redraw();
});
