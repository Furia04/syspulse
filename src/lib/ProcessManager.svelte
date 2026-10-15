<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { ProcessInfo } from './types';
  import { Search, RefreshCw, XCircle, AlertTriangle, CheckCircle, ArrowUpDown } from '@lucide/svelte';

  let processes = $state<ProcessInfo[]>([]);
  let searchQuery = $state('');
  let sortBy = $state<'cpu' | 'mem' | 'pid' | 'name'>('cpu');
  let sortAsc = $state(false);
  let isLoading = $state(false);
  let autoRefresh = $state(true);
  let refreshTimer: number | null = null;

  // Process Kill Modal State
  let targetProcess = $state<ProcessInfo | null>(null);
  let isKilling = $state(false);
  let toastMessage = $state<{ type: 'success' | 'error'; text: string } | null>(null);

  async function fetchProcesses() {
    try {
      isLoading = true;
      const list = await invoke<ProcessInfo[]>('get_process_list');
      processes = list;
    } catch (e) {
      console.error('Error fetching processes:', e);
    } finally {
      isLoading = false;
    }
  }

  function handleSort(column: 'cpu' | 'mem' | 'pid' | 'name') {
    if (sortBy === column) {
      sortAsc = !sortAsc;
    } else {
      sortBy = column;
      sortAsc = false;
    }
  }

  const filteredProcesses = $derived(() => {
    let result = [...processes];
    const query = searchQuery.trim().toLowerCase();
    if (query) {
      result = result.filter(
        (p) => p.name.toLowerCase().includes(query) || p.pid.toString().includes(query)
      );
    }

    result.sort((a, b) => {
      let comparison = 0;
      if (sortBy === 'cpu') comparison = a.cpu_usage - b.cpu_usage;
      else if (sortBy === 'mem') comparison = a.memory_bytes - b.memory_bytes;
      else if (sortBy === 'pid') comparison = a.pid - b.pid;
      else if (sortBy === 'name') comparison = a.name.localeCompare(b.name);
      return sortAsc ? comparison : -comparison;
    });

    return result;
  });

  async function confirmKillProcess() {
    if (!targetProcess) return;
    try {
      isKilling = true;
      await invoke('kill_process', { pid: targetProcess.pid });
      toastMessage = {
        type: 'success',
        text: `Proceso ${targetProcess.name} (PID: ${targetProcess.pid}) finalizado con éxito.`,
      };
      targetProcess = null;
      await fetchProcesses();
    } catch (err: any) {
      toastMessage = {
        type: 'error',
        text: typeof err === 'string' ? err : 'Error al intentar terminar el proceso.',
      };
    } finally {
      isKilling = false;
      setTimeout(() => {
        toastMessage = null;
      }, 4000);
    }
  }

  onMount(() => {
    fetchProcesses();
    refreshTimer = window.setInterval(() => {
      if (autoRefresh) fetchProcesses();
    }, 2500);
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });
</script>

<div class="space-y-4 pb-8">
  <!-- Controls Bar -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl p-3 flex flex-wrap items-center justify-between gap-3">
    <!-- Search Box -->
    <div class="relative flex-1 min-w-[240px]">
      <Search class="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Buscar proceso por nombre o PID..."
        class="w-full bg-slate-950/80 border border-slate-800 rounded-lg pl-9 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-sky-500 transition-colors"
      />
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-3 text-xs">
      <label class="flex items-center gap-2 cursor-pointer text-slate-400 hover:text-slate-300 select-none">
        <input type="checkbox" bind:checked={autoRefresh} class="rounded bg-slate-800 border-slate-700 text-sky-500 focus:ring-0" />
        <span>Auto-refresco (2.5s)</span>
      </label>

      <button
        onclick={fetchProcesses}
        disabled={isLoading}
        class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium transition-colors disabled:opacity-50"
      >
        <RefreshCw class="w-3.5 h-3.5 {isLoading ? 'animate-spin text-sky-400' : ''}" />
        <span>Actualizar</span>
      </button>

      <span class="text-slate-400 font-mono text-[11px] bg-slate-950 px-2.5 py-1 rounded-md border border-slate-800">
        {filteredProcesses().length} procesos
      </span>
    </div>
  </div>

  <!-- Toast Notification -->
  {#if toastMessage}
    <div class="p-3 rounded-lg flex items-center gap-2 text-xs font-medium {toastMessage.type === 'success' ? 'bg-emerald-950/80 border border-emerald-800 text-emerald-300' : 'bg-rose-950/80 border border-rose-800 text-rose-300'}">
      {#if toastMessage.type === 'success'}
        <CheckCircle class="w-4 h-4 shrink-0" />
      {:else}
        <AlertTriangle class="w-4 h-4 shrink-0" />
      {/if}
      <span>{toastMessage.text}</span>
    </div>
  {/if}

  <!-- Processes Table -->
  <div class="bg-slate-900/70 border border-slate-800 rounded-xl overflow-hidden">
    <div class="overflow-x-auto max-h-[520px]">
      <table class="w-full text-left text-xs border-collapse">
        <thead class="sticky top-0 bg-slate-950 border-b border-slate-800 z-10 select-none">
          <tr>
            <th class="p-3 cursor-pointer hover:text-sky-400 transition-colors" onclick={() => handleSort('pid')}>
              <div class="flex items-center gap-1 font-semibold uppercase tracking-wider text-[11px] text-slate-400">
                <span>PID</span>
                <ArrowUpDown class="w-3 h-3" />
              </div>
            </th>
            <th class="p-3 cursor-pointer hover:text-sky-400 transition-colors" onclick={() => handleSort('name')}>
              <div class="flex items-center gap-1 font-semibold uppercase tracking-wider text-[11px] text-slate-400">
                <span>Nombre del Proceso</span>
                <ArrowUpDown class="w-3 h-3" />
              </div>
            </th>
            <th class="p-3 cursor-pointer hover:text-sky-400 transition-colors" onclick={() => handleSort('cpu')}>
              <div class="flex items-center gap-1 font-semibold uppercase tracking-wider text-[11px] text-slate-400">
                <span>CPU %</span>
                <ArrowUpDown class="w-3 h-3" />
              </div>
            </th>
            <th class="p-3 cursor-pointer hover:text-sky-400 transition-colors" onclick={() => handleSort('mem')}>
              <div class="flex items-center gap-1 font-semibold uppercase tracking-wider text-[11px] text-slate-400">
                <span>Memoria RAM</span>
                <ArrowUpDown class="w-3 h-3" />
              </div>
            </th>
            <th class="p-3 font-semibold uppercase tracking-wider text-[11px] text-slate-400">Estado</th>
            <th class="p-3 text-right font-semibold uppercase tracking-wider text-[11px] text-slate-400">Acción</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-800/60 font-mono text-xs">
          {#each filteredProcesses().slice(0, 100) as proc (proc.pid)}
            <tr class="hover:bg-slate-800/30 transition-colors group">
              <td class="p-3 text-slate-400">{proc.pid}</td>
              <td class="p-3 font-sans font-medium text-slate-200">
                <span class="truncate max-w-[220px] inline-block">{proc.name}</span>
              </td>
              <td class="p-3 font-semibold {proc.cpu_usage > 50 ? 'text-rose-400' : proc.cpu_usage > 20 ? 'text-amber-400' : 'text-sky-400'}">
                {proc.cpu_usage.toFixed(1)}%
              </td>
              <td class="p-3 text-slate-300">
                {proc.memory_mb.toFixed(1)} MB
              </td>
              <td class="p-3">
                <span class="px-2 py-0.5 rounded-full text-[10px] bg-slate-800 text-slate-300 font-sans">
                  {proc.status}
                </span>
              </td>
              <td class="p-3 text-right">
                <button
                  onclick={() => (targetProcess = proc)}
                  class="px-2 py-1 rounded bg-rose-950/60 hover:bg-rose-900 border border-rose-800/60 text-rose-300 text-[11px] font-sans transition-colors"
                  title="Terminar proceso"
                >
                  Terminar
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- Modal Confirm Kill Process -->
{#if targetProcess}
  <div class="fixed inset-0 bg-black/70 backdrop-blur-xs flex items-center justify-center p-4 z-50">
    <div class="bg-slate-900 border border-slate-800 rounded-xl max-w-sm w-full p-5 shadow-2xl space-y-4">
      <div class="flex items-center gap-3 text-rose-400">
        <XCircle class="w-6 h-6 shrink-0" />
        <h4 class="font-bold text-sm text-slate-100">¿Finalizar Proceso?</h4>
      </div>

      <p class="text-xs text-slate-300 leading-relaxed">
        Estás a punto de forzar el cierre de:
        <strong class="text-white block mt-1 font-mono text-sm">{targetProcess.name} (PID: {targetProcess.pid})</strong>
        Los cambios no guardados en esta aplicación podrían perderse.
      </p>

      <div class="flex justify-end gap-2.5 pt-2">
        <button
          onclick={() => (targetProcess = null)}
          class="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs font-medium transition-colors"
        >
          Cancelar
        </button>
        <button
          onclick={confirmKillProcess}
          disabled={isKilling}
          class="px-3.5 py-1.5 rounded-lg bg-rose-600 hover:bg-rose-500 text-white text-xs font-medium transition-colors disabled:opacity-50"
        >
          {isKilling ? 'Finalizando...' : 'Confirmar Cierre'}
        </button>
      </div>
    </div>
  </div>
{/if}
