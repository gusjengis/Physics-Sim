import init, {webmain} from "./pkg/WGPU.js";

init().then(() => {
  webmain();
  
}
).catch((error) => {
    if (!error.message.startsWith("Using exceptions for control flow,")) {
        throw error;
    }
});

window.onload = function(){
  // console.log(window.devicePixelRatio)

  window.innerWidth = window.devicePixelRatio*document.documentElement.clientWidth;
  window.innerHeight = window.devicePixelRatio*document.documentElement.clientHeight;
}

window.onresize = function() {
  let WGPUWindow = document.getElementById("winit");  
  // document.body.style.zoom=1/window.devicePixelRatio;
  // console.log(window.devicePixelRatio)
  window.innerWidth = window.devicePixelRatio*document.documentElement.clientWidth;
  window.innerHeight = window.devicePixelRatio*document.documentElement.clientHeight;
    // WGPUWindow.requestFullscreen();
//     // WGPUWindow.exitFullscreen();
// console.log("resize")
  // WGPUWindow.width = window.innerWidth;
  // WGPUWindow.height = window.innerHeight;
  // WGPUWindow.style = "  "
}
