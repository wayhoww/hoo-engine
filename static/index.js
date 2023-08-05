import * as hooEngine from '../pkg/hoo_engine.js';


var now = new Date();

function nextFrame(engineInstance) {
    var newTime = new Date();
    var deltaTime = newTime - now;
    now = newTime;
    document.getElementsByTagName("title")[0].innerText = deltaTime;

    engineInstance.next_frame();

    requestAnimationFrame(() => {
        nextFrame(engineInstance);
    });
}

let canvas = document.getElementById('mainCanvas');
let context = canvas.getContext("webgpu")

await hooEngine.default();
let engineInstance = await hooEngine.JsHooEngine.new_async(context);
// let engineInstance = hooEngine.JsHooEngine.new(context);
await nextFrame(engineInstance);
