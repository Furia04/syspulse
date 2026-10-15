<script lang="ts">
  import type { SystemOverview } from './types';
  import { Cpu, HardDrive, Network, Layers, Thermometer, Clock, ShieldCheck, ArrowDown, ArrowUp } from '@lucide/svelte';

  let { 
    overview,
    cpuHistory = [],
    ramHistory = []
  }: { 
    overview: SystemOverview | null;
    cpuHistory: number[];
    ramHistory: number[];
  } = $props();

  function formatBytes(bytes: number, decimals = 1): string {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(decimals))} ${sizes[i]}`;
  }

  function formatSpeed(bytesPerSec: number): string {
    return `${formatBytes(bytesPerSec)}/s`;
  }

  function formatUptime(seconds: number): string {
    const d = Math.floor(seconds / 86400);
    const h = Math.floor((seconds % 86400) / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    if (d > 0) return `${d}d ${h}h ${m}m`;
    if (h > 0) return `${h}h ${m}m`;
    return `${m}m`;
  }

  function createSvgPath(data: number[], width = 260, height = 50): string {
    if (data.length < 2) return '';
    const maxVal = 100;
    const step = width / (data.length - 1);
    const points = data.map((val, idx) => {
      const x = idx * step;
      const y = height - (Math.min(100, Math.max(0, val)) / maxVal) * (height - 6) - 3;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    });
    return `M ${points.join(' L ')}`;
  }

  function createAreaPath(data: number[], width = 260, height = 50): string {
    if (data.length < 2) return '';
    const linePath = createSvgPath(data, width, height);
    return `${linePath} L ${width},${height} L 0,${height} Z`;
  }
</script>

<div class="space-y-5 pb-8">
  {#if !overview}
    <div class="flex items-center justify-center h-64 text-slate-500 gap-2">
      <div class="w-5 h-5 border-2 border-sky-400 border-t-transparent rounded-full animate-spin"></div>
      <span class="text-sm">Iniciando telemetría del sistema...</span>
    </div>
  {:else}
    <!-- Top System Spec Bar -->
    <div class="bg-slate-900/60 border border-slate-800/80 rounded-xl p-3.5 flex flex-wrap items-center justify-between gap-4 text-xs">
      <div class="flex items-center gap-2">
        <ShieldCheck class="w-4 h-4 text-emerald-400" />
        <span class="text-slate-400">Host:</span>
        <strong class="text-slate-200 font-mono">{overview.system_info.host_name}</strong>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-slate-400">SO:</span>
        <strong class="text-slate-200">{overview.system_info.os_name} {overview.system_info.os_version} ({overview.system_info.cpu_arch})</strong>
      </div>
      <div class="flex items-center gap-2">
        <Clock class="w-4 h-4 text-sky-400" />
        <span class="text-slate-400">Uptime:</span>
        <strong class="text-slate-200">{formatUptime(overview.system_info.uptime_seconds)}</strong>
      </div>
    </div>

    <!-- Main Telemetry KPI Grid -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <!-- CPU KPI Card -->
      <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between relative overflow-hidden group hover:border-sky-500/50 transition-all">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2 text-slate-300 font-medium text-xs">
            <Cpu class="w-4 h-4 text-sky-400" />
            <span>CPU TOTAL</span>
          </div>
          <span class="text-xs font-mono font-bold {overview.cpu.global_usage > 90 ? 'text-rose-400' : overview.cpu.global_usage > 70 ? 'text-amber-400' : 'text-sky-400'}">
            {overview.cpu.global_usage.toFixed(1)}%
          </span>
        </div>

        <div class="my-2 h-14 w-full">
          <svg viewBox="0 0 260 50" class="w-full h-full overflow-visible">
            <defs>
              <linearGradient id="cpuGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#38bdf8" stop-opacity="0.3" />
                <stop offset="100%" stop-color="#38bdf8" stop-opacity="0.0" />
              </linearGradient>
            </defs>
            <path d={createAreaPath(cpuHistory)} fill="url(#cpuGrad)" />
            <path d={createSvgPath(cpuHistory)} fill="none" stroke="#38bdf8" stroke-width="2" stroke-linecap="round" />
          </svg>
        </div>

        <div class="text-[11px] text-slate-400 truncate mt-1">
          {overview.cpu.brand} ({overview.cpu.core_count} Cores)
        </div>
      </div>

      <!-- RAM Memory KPI Card -->
      <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between relative overflow-hidden group hover:border-indigo-500/50 transition-all">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2 text-slate-300 font-medium text-xs">
            <Layers class="w-4 h-4 text-indigo-400" />
            <span>MEMORIA RAM</span>
          </div>
          <span class="text-xs font-mono font-bold {overview.memory.usage_percent > 90 ? 'text-rose-400' : overview.memory.usage_percent > 75 ? 'text-amber-400' : 'text-indigo-400'}">
            {overview.memory.usage_percent.toFixed(1)}%
          </span>
        </div>

        <div class="my-2 h-14 w-full">
          <svg viewBox="0 0 260 50" class="w-full h-full overflow-visible">
            <defs>
              <linearGradient id="ramGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#818cf8" stop-opacity="0.3" />
                <stop offset="100%" stop-color="#818cf8" stop-opacity="0.0" />
              </linearGradient>
            </defs>
            <path d={createAreaPath(ramHistory)} fill="url(#ramGrad)" />
            <path d={createSvgPath(ramHistory)} fill="none" stroke="#818cf8" stroke-width="2" stroke-linecap="round" />
          </svg>
        </div>

        <div class="flex justify-between text-[11px] text-slate-400 mt-1">
          <span>{formatBytes(overview.memory.used_bytes)} / {formatBytes(overview.memory.total_bytes)}</span>
          <span>Swap: {overview.memory.swap_usage_percent.toFixed(0)}%</span>
        </div>
      </div>

      <!-- Network Throughput Card -->
      <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between group hover:border-emerald-500/50 transition-all">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2 text-slate-300 font-medium text-xs">
            <Network class="w-4 h-4 text-emerald-400" />
            <span>RED EN TIEMPO REAL</span>
          </div>
        </div>

        <div class="space-y-2.5 my-auto">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1.5 text-xs text-slate-400">
              <ArrowDown class="w-3.5 h-3.5 text-emerald-400" />
              <span>Bajada:</span>
            </div>
            <strong class="font-mono text-emerald-300 text-sm">{formatSpeed(overview.network.download_speed_bps)}</strong>
          </div>

          <div class="flex items-center justify-between">
            <div class="flex items-center gap-1.5 text-xs text-slate-400">
              <ArrowUp class="w-3.5 h-3.5 text-cyan-400" />
              <span>Subida:</span>
            </div>
            <strong class="font-mono text-cyan-300 text-sm">{formatSpeed(overview.network.upload_speed_bps)}</strong>
          </div>
        </div>

        <div class="flex justify-between text-[11px] text-slate-400 mt-2 pt-2 border-t border-slate-800/80">
          <span>Total: ↓{formatBytes(overview.network.total_received_bytes)}</span>
          <span>↑{formatBytes(overview.network.total_transmitted_bytes)}</span>
        </div>
      </div>

      <!-- Thermal & Storage Quick View -->
      <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-4 flex flex-col justify-between group hover:border-amber-500/50 transition-all">
        <div class="flex items-center justify-between mb-2">
          <div class="flex items-center gap-2 text-slate-300 font-medium text-xs">
            <HardDrive class="w-4 h-4 text-amber-400" />
            <span>DISCOS PRINCIPALES</span>
          </div>
          <span class="text-xs text-slate-400">{overview.disks.length} Unidades</span>
        </div>

        <div class="space-y-2 my-auto">
          {#each overview.disks.slice(0, 2) as disk}
            <div>
              <div class="flex justify-between text-[11px] text-slate-300 mb-1">
                <span class="font-mono">{disk.mount_point}</span>
                <span>{disk.usage_percent.toFixed(0)}% ({formatBytes(disk.available_space)} libres)</span>
              </div>
              <div class="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
                <div 
                  class="h-full rounded-full transition-all duration-500 {disk.usage_percent > 90 ? 'bg-rose-500' : disk.usage_percent > 75 ? 'bg-amber-500' : 'bg-amber-400'}"
                  style="width: {disk.usage_percent}%"
                ></div>
              </div>
            </div>
          {/each}
        </div>

        <div class="flex items-center justify-between text-[11px] text-slate-400 mt-2 pt-2 border-t border-slate-800/80">
          {#if overview.temperatures.length > 0}
            <div class="flex items-center gap-1 text-slate-300">
              <Thermometer class="w-3.5 h-3.5 text-rose-400" />
              <span>{overview.temperatures[0].label}: {overview.temperatures[0].temperature.toFixed(0)}°C</span>
            </div>
          {:else}
            <span>Sensores ACPI estándar</span>
          {/if}
          <span class="text-emerald-400 font-medium">Estado Normal</span>
        </div>
      </div>
    </div>

    <!-- CPU Cores Breakdown Grid -->
    <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-4">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-slate-300 flex items-center gap-2">
          <Cpu class="w-4 h-4 text-sky-400" />
          Carga por Núcleo de Procesador ({overview.cpu.cores.length} Hilos Lógicos)
        </h3>
        <span class="text-[11px] text-slate-400 font-mono">Frecuencia Base: {overview.cpu.cores[0]?.frequency || 0} MHz</span>
      </div>

      <div class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-6 lg:grid-cols-8 gap-2.5">
        {#each overview.cpu.cores as core}
          <div class="bg-slate-950/60 border border-slate-800/70 rounded-lg p-2.5 flex flex-col justify-between">
            <div class="flex justify-between items-center text-[11px] mb-1.5">
              <span class="text-slate-400 font-mono">C#{core.id}</span>
              <span class="font-mono font-bold {core.usage > 90 ? 'text-rose-400' : core.usage > 70 ? 'text-amber-400' : 'text-slate-300'}">
                {core.usage.toFixed(0)}%
              </span>
            </div>
            <div class="w-full bg-slate-800 rounded-full h-1.5 overflow-hidden">
              <div 
                class="h-full rounded-full transition-all duration-300 {core.usage > 90 ? 'bg-rose-500' : core.usage > 70 ? 'bg-amber-400' : 'bg-sky-400'}"
                style="width: {core.usage}%"
              ></div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Storage Partitions Section -->
    <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-4">
      <h3 class="text-xs font-semibold uppercase tracking-wider text-slate-300 mb-3 flex items-center gap-2">
        <HardDrive class="w-4 h-4 text-amber-400" />
        Unidades de Almacenamiento y Particiones
      </h3>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        {#each overview.disks as disk}
          <div class="bg-slate-950/60 border border-slate-800/80 rounded-lg p-3">
            <div class="flex justify-between items-center mb-1">
              <div class="flex items-center gap-1.5">
                <strong class="text-slate-200 text-xs">{disk.name || disk.mount_point}</strong>
                <span class="text-[10px] px-1 rounded bg-slate-800 text-slate-400 font-mono">{disk.file_system}</span>
              </div>
              <span class="text-xs font-mono font-bold {disk.usage_percent > 90 ? 'text-rose-400' : 'text-slate-300'}">
                {disk.usage_percent.toFixed(1)}%
              </span>
            </div>

            <div class="w-full bg-slate-800 rounded-full h-2 my-2 overflow-hidden">
              <div 
                class="h-full rounded-full transition-all duration-500 {disk.usage_percent > 90 ? 'bg-rose-500' : disk.usage_percent > 75 ? 'bg-amber-500' : 'bg-sky-500'}"
                style="width: {disk.usage_percent}%"
              ></div>
            </div>

            <div class="flex justify-between text-[11px] text-slate-400 font-mono">
              <span>Usado: {formatBytes(disk.used_space)}</span>
              <span>Libre: {formatBytes(disk.available_space)}</span>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>
