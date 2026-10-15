<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Activity, Minus, Square, X, ShieldAlert } from '@lucide/svelte';

  let { 
    activeTab = $bindable('dashboard'), 
    hasAlert = false,
    alertText = ''
  }: { 
    activeTab: string;
    hasAlert?: boolean;
    alertText?: string;
  } = $props();

  const appWindow = getCurrentWindow();

  async function handleMinimize() {
    await appWindow.minimize();
  }

  async function handleMaximize() {
    await appWindow.toggleMaximize();
  }

  async function handleClose() {
    await appWindow.close();
  }
</script>

<header class="h-11 bg-slate-950/90 border-b border-slate-800/80 flex items-center justify-between select-none z-50 sticky top-0 backdrop-blur-md">
  <!-- Draggable Title & Brand Area -->
  <div class="flex items-center gap-3 px-3.5 h-full cursor-default" data-tauri-drag-region>
    <div class="flex items-center gap-2 text-sky-400 font-semibold tracking-wide text-sm" data-tauri-drag-region>
      <Activity class="w-4 h-4 text-cyan-400 animate-pulse" />
      <span class="bg-gradient-to-r from-sky-400 to-indigo-400 bg-clip-text text-transparent font-bold">SysPulse</span>
      <span class="text-[10px] px-1.5 py-0.5 rounded bg-sky-950/80 text-sky-300 font-mono border border-sky-800/50">v0.1.0</span>
    </div>

    {#if hasAlert}
      <div class="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full bg-rose-950/80 border border-rose-800/60 text-rose-300 text-xs animate-bounce" data-tauri-drag-region>
        <ShieldAlert class="w-3.5 h-3.5 text-rose-400" />
        <span class="font-medium text-[11px]">{alertText}</span>
      </div>
    {/if}
  </div>

  <!-- Navigation Tabs -->
  <nav class="flex items-center gap-1 bg-slate-900/90 p-1 rounded-lg border border-slate-800 text-xs font-medium">
    <button
      class="px-3 py-1 rounded-md transition-all {activeTab === 'dashboard' ? 'bg-sky-500/20 text-sky-300 font-semibold shadow-xs' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'}"
      onclick={() => (activeTab = 'dashboard')}
    >
      Dashboard
    </button>
    <button
      class="px-3 py-1 rounded-md transition-all {activeTab === 'processes' ? 'bg-sky-500/20 text-sky-300 font-semibold shadow-xs' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'}"
      onclick={() => (activeTab = 'processes')}
    >
      Procesos
    </button>
    <button
      class="px-3 py-1 rounded-md transition-all {activeTab === 'network' ? 'bg-sky-500/20 text-sky-300 font-semibold shadow-xs' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'}"
      onclick={() => (activeTab = 'network')}
    >
      Red & Puertos
    </button>
    <button
      class="px-3 py-1 rounded-md transition-all {activeTab === 'diagnostics' ? 'bg-sky-500/20 text-sky-300 font-semibold shadow-xs' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50'}"
      onclick={() => (activeTab = 'diagnostics')}
    >
      Diagnósticos
    </button>
  </nav>

  <!-- Window Action Controls -->
  <div class="flex items-center h-full">
    <button
      class="w-11 h-full flex items-center justify-center text-slate-400 hover:text-slate-200 hover:bg-slate-800/70 transition-colors"
      onclick={handleMinimize}
      title="Minimizar"
      aria-label="Minimizar"
    >
      <Minus class="w-3.5 h-3.5" />
    </button>
    <button
      class="w-11 h-full flex items-center justify-center text-slate-400 hover:text-slate-200 hover:bg-slate-800/70 transition-colors"
      onclick={handleMaximize}
      title="Maximizar"
      aria-label="Maximizar"
    >
      <Square class="w-3 h-3" />
    </button>
    <button
      class="w-11 h-full flex items-center justify-center text-slate-400 hover:text-white hover:bg-rose-600 transition-colors"
      onclick={handleClose}
      title="Cerrar"
      aria-label="Cerrar"
    >
      <X class="w-4 h-4" />
    </button>
  </div>
</header>
