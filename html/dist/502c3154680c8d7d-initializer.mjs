function initializeProgressBar() {
  const canvas = document.getElementById('canvas');
  const gl = canvas.getContext('webgl2');
  if (!gl) throw new Error("Failed to get WebGL2 context to render progress bar");

  const vertexShader = gl.createShader(gl.VERTEX_SHADER)
  const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER)
  gl.shaderSource(vertexShader, `
  attribute vec4 a_position;

  void main() {
    gl_Position = a_position;
  }
  `)

  gl.shaderSource(fragmentShader, `
  precision mediump float;
  uniform vec3 u_color;

  void main() {
    gl_FragColor = vec4(u_color, 1.0);
  }
  `)

  gl.compileShader(vertexShader)
  let success = gl.getShaderParameter(vertexShader, gl.COMPILE_STATUS)
  if (!success) throw new Error(gl.getShaderInfoLog(vertexShader))

  gl.compileShader(fragmentShader)
  success = gl.getShaderParameter(fragmentShader, gl.COMPILE_STATUS)
  if (!success) throw new Error(gl.getShaderInfoLog(fragmentShader))

  const program = gl.createProgram()

  gl.attachShader(program, vertexShader)
  gl.attachShader(program, fragmentShader)

  gl.linkProgram(program)
  gl.useProgram(program)

  let positionAttributeLocation = gl.getAttribLocation(program, "a_position");
  let positionBuffer = gl.createBuffer();

  let colorUniformLocation = gl.getUniformLocation(program, "u_color");

  return {
    canvas,
    gl,
    program,
    positionAttributeLocation,
    positionBuffer,
    colorUniformLocation
  };
}

function drawRect({ gl, program, positionAttributeLocation, positionBuffer, colorUniformLocation }, x, y, width, height, color) {
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  let positions = [
    x, y,
    x, y + height,
    x + width, y + height,
    x + width, y,
  ];
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.DYNAMIC_DRAW);

  gl.useProgram(program);
  gl.enableVertexAttribArray(positionAttributeLocation);
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.vertexAttribPointer(positionAttributeLocation, 2, gl.FLOAT, false, 0, 0);

  gl.uniform3f(colorUniformLocation, color[0], color[1], color[2]);

  let primitiveType = gl.TRIANGLES;
  gl.drawArrays(gl.TRIANGLE_FAN, 0, 4);
}

function drawProgressBar(gldata, progress) {
  const gl = gldata.gl;
  const canvas = gldata.canvas;
  const rect = canvas.getBoundingClientRect();
  const width = rect.width;
  const height = rect.height;
  canvas.width = width;
  canvas.height = height;

  gl.viewport(0, 0, canvas.width, canvas.height);
  gl.clearColor(0, 0, 0, 0);
  gl.clear(gl.COLOR_BUFFER_BIT);

  const border = height * 0.02;
  drawRect(gldata, -0.5 - border / width, -0.1 - border / height, 1.0 + border / width * 2, 0.2 + border / height * 2, [1.0, 0.0, 0.5]);
  drawRect(gldata, -0.5 + progress, -0.1, 1.0 - progress, 0.2, [1.0, 1.0, 1.0]);
}

let gldata;
export default function myInitializer () {
  return {
    onStart: () => {
      gldata = initializeProgressBar();
      drawProgressBar(gldata, 0);
    },
    onProgress: ({current, total}) => {
      drawProgressBar(gldata, current / total);
    },
    onComplete: () => {},
    onSuccess: (wasm) => {},
    onFailure: (error) => {}
  }
};
