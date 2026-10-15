<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { sendNotification, isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

  import type { SystemOverview } from './lib/types';
  import TitleBar from './lib/TitleBar.svelte';
  import Dashboard from './lib/Dashboard.svelte';
  import ProcessManager from './lib/ProcessManager.svelte';
  import NetworkPorts from './lib/NetworkPorts.svelte';
  import Diagnostics from './lib/Diagnostics.svelte';

  let activeTab = $state('dashboard');
  let overview = $state<SystemOverview | null>(null);
  let cpuHistory = $state<number[]>([]);
  let ramHistory = $state<number[]>([]);

  let hasAlert = $state(false);
  let alertText = $state('');
  let lastNotificationTime = 0;
  let unlistenFn: UnlistenFn | null = null;

  async function checkNotificationPermission() {
    try {
      let permissionGranted = await isPermissionGranted();
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
      }
    } catch (e) {
      console.warn('Notification permission check failed:', e);
    }
  }

  function handleMetricsUpdate(data: SystemOverview) {
    overview = data;

    // Update histories (keep last 60 ticks)
    cpuHistory = [...cpuHistory.slice(-59), data.cpu.global_usage];
    ramHistory = [...ramHistory.slice(-59), data.memory.usage_percent];

    // Check thresholds for warnings
    const cpuOverload = data.cpu.global_usage >= 90;
    const ramOverload = data.memory.usage_percent >= 90;

    if (cpuOverload || ramOverload) {
      hasAlert = true;
      alertText = cpuOverload && ramOverload
        ? 'Alerta Crítica: CPU y RAM sobre 90%'
        : cpuOverload
        ? `Alerta: CPU al ${data.cpu.global_usage.toFixed(0)}%`
        : `Alerta: RAM al ${data.memory.usage_percent.toFixed(0)}%`;

      // Trigger desktop notification with 60-second cooldown
      const now = Date.now();
      if (now - lastNotificationTime > 60000) {
        lastNotificationTime = now;
        try {
          sendNotification({
            title: '⚠️ SysPulse - Alerta de Rendimiento',
            body: alertText,
          });
        } catch (err) {
          console.warn('Error sending notification:', err);
        }
      }
    } else {
      hasAlert = false;
      alertText = '';
    }
  }

  onMount(async () => {
    await checkNotificationPermission();

    // Fetch initial metrics snapshot instantly
    try {
      const initial = await invoke<SystemOverview>('get_initial_metrics');
      handleMetricsUpdate(initial);
    } catch (e) {
      console.error('Failed to get initial metrics:', e);
    }

    // Subscribe to real-time telemetry stream event
    try {
      unlistenFn = await listen<SystemOverview>('metrics-stream', (event) => {
        handleMetricsUpdate(event.payload);
      });
    } catch (e) {
      console.error('Failed to listen to metrics-stream:', e);
    }
  });

  onDestroy(() => {
    if (unlistenFn) {
      unlistenFn();
    }
  });
</script>

<div class="h-screen w-screen flex flex-col bg-slate-950 text-slate-100 overflow-hidden font-sans">
  <!-- Frameless Custom TitleBar -->
  <TitleBar bind:activeTab {hasAlert} {alertText} />

  <!-- Main Scrollable Content Area -->
  <main class="flex-1 overflow-y-auto px-6 py-5">
    {#if activeTab === 'dashboard'}
      <Dashboard {overview} {cpuHistory} {ramHistory} />
    {:else if activeTab === 'processes'}
      <ProcessManager />
    {:else if activeTab === 'network'}
      <NetworkPorts network={overview?.network} />
    {:else if activeTab === 'diagnostics'}
      <Diagnostics />
    {/if}
  </main>
</div>
