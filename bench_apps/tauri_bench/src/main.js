const { invoke } = window.__TAURI__.core;

let outputEl;
let runBtn;
let runsSelect;

async function runBenchmark() {
  const runs = parseInt(runsSelect.value, 10);
  runBtn.disabled = true;
  outputEl.textContent = `Running native verification (${runs} runs) via Tauri...\n`;

  try {
    // Call the rust backend
    const [success, totalTimeMs] = await invoke("run_benchmark", { runs });
    
    const avgTime = totalTimeMs / runs;
    
    outputEl.textContent += `Result: ${success ? '✅ VALID' : '❌ INVALID'}\n`;
    outputEl.textContent += `Total time: ${(totalTimeMs / 1000).toFixed(2)} seconds\n`;
    outputEl.textContent += `Average time per verification: ${avgTime.toFixed(2)} ms\n`;
    outputEl.textContent += `(Measured over ${runs} runs natively)`;
  } catch (e) {
    outputEl.textContent = `Error: ${e}`;
  } finally {
    runBtn.disabled = false;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  outputEl = document.querySelector("#output");
  runBtn = document.querySelector("#runBtn");
  runsSelect = document.querySelector("#runsSelect");
  
  runBtn.addEventListener("click", (e) => {
    e.preventDefault();
    runBenchmark();
  });
});
