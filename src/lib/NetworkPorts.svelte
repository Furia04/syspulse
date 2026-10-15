<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { PortInfo, NetworkMetrics } from './types';
  import { Network, Radio, Search, RefreshCw, Shield, ArrowDown, ArrowUp } from '@lucide/svelte';

  let { network }: { network?: NetworkMetrics } = $props();

  let ports = $state<PortInfo[]>([]);
  let searchQuery = $state('');
  let isLoading = $state(false);

  async function fetchPorts() {
    try {
      isLoading = true;
      const list = await invoke<PortInfo[]>('get_open_ports');
      ports = list;
    } catch (e) {
      console.error('Error fetching ports:', e);
    } finally {
      isLoading = false;
    }
  }

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  }

  const filteredPorts = $derived(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return ports;
    return ports.filter(
      (p) =>
        p.port.toString().includes(q) ||
        p.local_address.toLowerCase().includes(q) ||
        (p.process_name && p.process_name.toLowerCase().includes(q)) ||
        (p.pid && p.pid.toString().includes(q))
    );
  });

  onMount(() => {
    fetchPorts();
  });
</script>

<div class="space-y-4 pb-8">
  <!-- Network Bandwidth Banner -->
  {#if network}
    <div class="grid grid-cols-1 md:grid-cols-4 gap-3 bg-slate-900/70 border border-slate-800 rounded-xl p-4">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-lg bg-emerald-950/60 border border-emerald-800/60 text-emerald-400">
          <ArrowDown class="w-5 h-5" />
        </div>
        <div>
          <div class="text-[11px] text-slate-400 font-medium">VELOCIDAD DESCARGA</div>
          <div class="text-sm font-bold font-mono text-emerald-300">
            {formatBytes(network.download_speed_bps)}/s
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-lg bg-cyan-950/60 border border-cyan-800/60 text-cyan-400">
          <ArrowUp class="w-5 h-5" />
        </div>
        <div>
          <div class="text-[11px] text-slate-400 font-medium">VELOCIDAD SUBIDA</div>
          <div class="text-sm font-bold font-mono text-cyan-300">
            {formatBytes(network.upload_speed_bps)}/s
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-lg bg-slate-800 text-slate-300">
          <Network class="w-5 h-5" />
        </div>
        <div>
          <div class="text-[11px] text-slate-400 font-medium">DATOS RECIBIDOS (SESIÓN)</div>
          <div class="text-sm font-bold font-mono text-slate-200">
            {formatBytes(network.total_received_bytes)}
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-lg bg-slate-800 text-slate-300">
          <Radio class="w-5 h-5" />
        </div>
        <div>
          <div class="text-[11px] text-slate-400 font-medium">DATOS ENVIADOS (SESIÓN)</div>
          <div class="text-sm font-bold font-mono text-slate-200">
            {formatBytes(network.total_transmitted_bytes)}
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Open Ports Bar & Filter -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-3 flex flex-wrap items-center justify-between gap-3">
    <div class="relative flex-1 min-w-[240px]">
      <Search class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Filtrar por puerto (ej: 80, 443, 3000) o servicio..."
        class="w-full bg-slate-950/80 border border-slate-800 rounded-lg pl-9 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-sky-500 transition-colors"
      />
    </div>

    <div class="flex items-center gap-3">
      <button
        onclick={fetchPorts}
        disabled={isLoading}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium transition-colors disabled:opacity-50"
      >
        <RefreshCw class="w-3.5 h-3.5 {isLoading ? 'animate-spin text-sky-400' : ''}" />
        <span>Escanear Puertos</span>
      </button>

      <span class="text-slate-400 font-mono text-[11px] bg-slate-950 px-2.5 py-1 rounded-md border border-slate-800">
        {filteredPorts().length} sockets en escucha
      </span>
    </div>
  </div>

  <!-- Ports Table -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl overflow-hidden">
    <div class="overflow-x-auto max-h-[500px]">
      <table class="w-full text-left text-xs border-collapse">
        <thead class="sticky top-0 bg-slate-950 border-b border-slate-800 z-10 select-none">
          <tr>
            <th class="p-3 font-semibold uppercase tracking-wider text-[11px] text-slate-400">Puerto</th>
            <th class="p-3 font-semibold uppercase tracking-wider text-[11px] text-slate-400">Protocolo</th>
            <th class="p-3 font-semibold uppercase tracking-wider text-[11px] text-slate-400">Dirección Local</th>
            <th class="p-3 font-semibold uppercase tracking-wider text-[11px] text-slate-400">Proceso Asociado</th>
            <th class="p-3 font-semibold uppercase tracking-wider text-[11px] text-slate-400">PID</th>
            <th class="p-3 text-right font-semibold uppercase tracking-wider text-[11px] text-slate-400">Estado</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-mono text-xs">
          {#if filteredPorts().length === 0}
            <tr>
              <td colspan="6" class="p-6 text-center text-slate-500 font-sans">
                No se encontraron sockets abiertos que coincidan con la búsqueda.
              </td>
            </tr>
          {:else}
            {#each filteredPorts() as port}
              <tr class="hover:bg-slate-800/30 transition-colors">
                <td class="p-3 font-bold text-sky-400">:{port.port}</td>
                <td class="p-3 text-slate-400">{port.protocol}</td>
                <td class="p-3 text-slate-300">{port.local_address}</td>
                <td class="p-3 font-sans font-medium text-slate-200">
                  <div class="flex items-center gap-1.5">
                    <Shield class="w-3.5 h-3.5 text-slate-400 shrink-0" />
                    <span class="truncate max-w-[200px]">{port.process_name || 'Sistema / Servicio Windows'}</span>
                  </div>
                </td>
                <td class="p-3 text-slate-400">{port.pid || '-'}</td>
                <td class="p-3 text-right">
                  <span class="px-2 py-0.5 rounded-full text-[10px] bg-emerald-950/70 border border-emerald-800/50 text-emerald-300 font-sans font-semibold">
                    {port.state}
                  </span>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>
