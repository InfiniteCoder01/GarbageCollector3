export default function myInitializer () {
  return {
    // called when the initialization starts 
    onStart: () => {},
    // called when the download progresses, including a last update at the ennd
    onProgress: ({current, total}) => {console.log(current, total)},
    // called when the process is complete (succesful or not) 
    onComplete: () => {},
    // called when the process has completed successfully, including the WebAssembly instance
    onSuccess: (wasm) => {},
    // called when the process failed, including the error 
    onFailure: (error) => {}
  }
};
