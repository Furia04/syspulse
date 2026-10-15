<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import type { LatencyResult } from './types';
  import { Gauge, FileText, CheckCircle2, AlertCircle, Play, Download, ExternalLink, Zap } from '@lucide/svelte';

  let customTarget = $state('1.1.1.1');
  let isTestingLatency = $state(false);
  let latencyHistory = $state<LatencyResult[]>([]);
  let currentResult = $state<LatencyResult | null>(null);

  // Export State
  let exportFormat = $state<'html' | 'json'>('html');
  let isExporting = $state(false);
  let exportedFilePath = $state<string | null>(null);
  let exportError = $state<string | null>(null);

  const presetTargets = [
    { label: 'Cloudflare DNS', host: '1.1.1.1' },
    { label: 'Google DNS', host: '8.8.8.8' },
    { label: 'Quad9 DNS', host: '9.9.9.9' },
    { label: 'GitHub Host', host: 'github.com' },
  ];

  async function runLatencyTest(targetHost?: string) {
    const host = targetHost || customTarget.trim();
    if (!host) return;

    try {
      isTestingLatency = true;
      const res = await invoke<LatencyResult>('run_ping_test', { host });
      currentResult = res;
      latencyHistory = [res, ...latencyHistory.slice(0, 9)];
    } catch (e: any) {
      currentResult = {
        target: host,
        success: false,
        rtt_ms: 0,
        error: typeof e === 'string' ? e : 'Fallo en la prueba de conexión',
      };
    } finally {
      isTestingLatency = false;
    }
  }

  async function handleExportReport() {
    try {
      isExporting = true;
      exportError = null;
      exportedFilePath = null;
      const path = await invoke<string>('export_system_report', { format: exportFormat });
      exportedFilePath = path;
    } catch (e: any) {
      exportError = typeof e === 'string' ? e : 'Error exportando informe';
    } finally {
      isExporting = false;
    }
  }
</script>

<div class="space-y-6 pb-8">
  <!-- Latency Test Tool Card -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <div class="p-2 rounded-lg bg-sky-950/60 border border-sky-800/60 text-sky-400">
          <Zap class="w-5 h-5" />
        </div>
        <div>
          <h3 class="text-sm font-bold text-slate-100">Prueba de Latencia y Conectividad de Red</h3>
          <p class="text-xs text-slate-400">Verifica el tiempo de ida y vuelta (RTT) y la resolución DNS hacia servidores externos.</p>
        </div>
      </div>
    </div>

    <!-- Presets -->
    <div class="flex flex-wrap items-center gap-2 pt-1">
      <span class="text-xs text-slate-400 mr-1">Servidores Populares:</span>
      {#each presetTargets as preset}
        <button
          onclick={() => {
            customTarget = preset.host;
            runLatencyTest(preset.host);
          }}
          disabled={isTestingLatency}
          class="px-2.5 py-1 rounded-md bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-mono border border-slate-700/60 transition-colors disabled:opacity-50"
        >
          {preset.label} ({preset.host})
        </button>
      {/each}
    </div>

    <!-- Custom Target Input & Trigger -->
    <div class="flex gap-2.5">
      <input
        type="text"
        bind:value={customTarget}
        placeholder="IP o nombre de host (ej: 1.1.1.1, google.com, gateway local)..."
        class="flex-1 bg-slate-950/80 border border-slate-800 rounded-lg px-3.5 py-2 text-xs font-mono text-slate-200 placeholder-slate-500 focus:outline-none focus:border-sky-500 transition-colors"
      />
      <button
        onclick={() => runLatencyTest()}
        disabled={isTestingLatency}
        class="flex items-center gap-2 px-4 py-2 rounded-lg bg-sky-600 hover:bg-sky-500 text-white text-xs font-semibold transition-colors disabled:opacity-50 shadow-xs shadow-sky-600/30"
      >
        <Play class="w-3.5 h-3.5 fill-current {isTestingLatency ? 'animate-pulse' : ''}" />
        <span>{isTestingLatency ? 'Probando...' : 'Lanzar Ping'}</span>
      </button>
    </div>

    <!-- Live Test Result Banner -->
    {#if currentResult}
      <div class="p-4 rounded-xl border {currentResult.success ? 'bg-slate-950/80 border-slate-800' : 'bg-rose-950/50 border-rose-800/60'} flex items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          {#if currentResult.success}
            <div class="p-2 rounded-full {currentResult.rtt_ms < 35 ? 'bg-emerald-950/80 text-emerald-400' : currentResult.rtt_ms < 90 ? 'bg-amber-950/80 text-amber-400' : 'bg-rose-950/80 text-rose-400'}">
              <CheckCircle2 class="w-5 h-5" />
            </div>
            <div>
              <div class="text-xs text-slate-400">Respuesta desde <span class="font-mono text-slate-200 font-bold">{currentResult.target}</span></div>
              <div class="text-lg font-bold font-mono {currentResult.rtt_ms < 35 ? 'text-emerald-300' : currentResult.rtt_ms < 90 ? 'text-amber-300' : 'text-rose-300'}">
                {currentResult.rtt_ms.toFixed(1)} ms
                <span class="text-xs font-sans font-normal text-slate-400 ml-2">
                  ({currentResult.rtt_ms < 35 ? 'Excelente (Fibra/Baja latencia)' : currentResult.rtt_ms < 90 ? 'Aceptable' : 'Latencia Elevada'})
                </span>
              </div>
            </div>
          {:else}
            <div class="p-2 rounded-full bg-rose-950 text-rose-400">
              <AlertCircle class="w-5 h-5" />
            </div>
            <div>
              <div class="text-xs text-rose-300 font-bold">Error de Conexión a {currentResult.target}</div>
              <div class="text-xs text-rose-400 mt-0.5">{currentResult.error || 'Host inaccesible o timeout agotado'}</div>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <!-- Export Report Tool Card -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-5 space-y-4">
    <div class="flex items-center gap-2">
      <div class="p-2 rounded-lg bg-indigo-950/60 border border-indigo-800/60 text-indigo-400">
        <FileText class="w-5 h-5" />
      </div>
      <div>
        <h3 class="text-sm font-bold text-slate-100">Exportación de Auditoría e Informe del Sistema</h3>
        <p class="text-xs text-slate-400">Genera una captura completa del hardware, métricas de carga, procesos principales y puertos abiertos.</p>
      </div>
    </div>

    <div class="bg-slate-950/60 border border-slate-800/70 rounded-xl p-4 flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <span class="text-xs font-medium text-slate-300">Formato del Reporte:</span>
        <div class="flex items-center bg-slate-900 p-1 rounded-lg border border-slate-800 text-xs">
          <button
            onclick={() => (exportFormat = 'html')}
            class="px-3 py-1 rounded-md transition-all {exportFormat === 'html' ? 'bg-sky-500/20 text-sky-300 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
          >
            HTML Interactivo (Dashboard Web Autónomo)
          </button>
          <button
            onclick={() => (exportFormat = 'json')}
            class="px-3 py-1 rounded-md transition-all {exportFormat === 'json' ? 'bg-sky-500/20 text-sky-300 font-semibold' : 'text-slate-400 hover:text-slate-200'}"
          >
            JSON Raw (Datos Estructurados)
          </button>
        </div>
      </div>

      <button
        onclick={handleExportReport}
        disabled={isExporting}
        class="flex items-center gap-2 px-4 py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-colors disabled:opacity-50 shadow-xs shadow-indigo-600/30"
      >
        <Download class="w-4 h-4 {isExporting ? 'animate-bounce' : ''}" />
        <span>{isExporting ? 'Generando Informe...' : 'Exportar Informe en 1-Clic'}</span>
      </button>
    </div>

    <!-- Export Result Feedback -->
    {#if exportedFilePath}
      <div class="p-3.5 rounded-lg bg-emerald-950/70 border border-emerald-800 text-emerald-300 text-xs space-y-1">
        <div class="flex items-center gap-2 font-bold">
          <CheckCircle2 class="w-4 h-4 shrink-0" />
          <span>¡Informe exportado exitosamente!</span>
        </div>
        <p class="text-slate-300 break-all pl-6">
          Guardado en: <span class="font-mono text-emerald-200 select-all font-semibold">{exportedFilePath}</span>
        </p>
      </div>
    {/if}

    {#if exportError}
      <div class="p-3 rounded-lg bg-rose-950/80 border border-rose-800 text-rose-300 text-xs flex items-center gap-2">
        <AlertCircle class="w-4 h-4 shrink-0" />
        <span>{exportError}</span>
      </div>
    {/if}
  </div>
</div>
