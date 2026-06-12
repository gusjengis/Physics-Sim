import init, {webmain} from "./pkg/WGPU.js";

// wgpu 0.16's web backend does not forward requiredLimits to
// requestDevice(), so the device comes back with WebGPU defaults
// (8 storage buffers/stage, 4 bind groups) — too small for this sim
// (compute uses 10 storage buffers, render uses 8 bind groups).
// Inject the limits here, clamped to what the adapter actually offers.
if (navigator.gpu) {
  const origRequestDevice = GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice = function (desc = {}) {
    const want = {
      maxBindGroups: 8,
      maxStorageBuffersPerShaderStage: 16,
      maxStorageBufferBindingSize: 134217728, // 128 MiB
      maxBufferSize: 268435456,               // 256 MiB
    };
    const limits = {};
    for (const [k, v] of Object.entries(want)) {
      const supported = this.limits[k];
      if (supported !== undefined) limits[k] = Math.min(v, supported);
    }
    desc.requiredLimits = Object.assign(limits, desc.requiredLimits || {});
    return origRequestDevice.call(this, desc);
  };
}

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
